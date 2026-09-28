use crate::types::*;
// InitSafeBootMode is a kernel DATA export — needs raw-dylib so the reference goes
// through the import thunk (__imp_) instead of expecting a local definition.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" { static InitSafeBootMode: ULONG; }
use crate::glue::*;
use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ushort};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_DEVICE_RELATION_TYPE_BusRelations as BusRelations, _WORK_QUEUE_TYPE_CriticalWorkQueue as CriticalWorkQueue, DEVICE_MEDIA_INFO, DISK_CACHE_INFORMATION, DISK_CACHE_SETTING, DISK_GEOMETRY, DISK_GEOMETRY_EX, _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue, _DETECTION_TYPE_DetectNone as DetectNone, _DISK_CACHE_STATE_DiskCacheModifyUnsuccessful as DiskCacheModifyUnsuccessful, _DISK_CACHE_STATE_DiskCacheNormal as DiskCacheNormal, _DISK_CACHE_STATE_DiskCacheWriteThroughNotSupported as DiskCacheWriteThroughNotSupported, _KWAIT_REASON_Executive, _MEDIA_TYPE_FixedMedia as FixedMedia, GETVERSIONINPARAMS, GET_LENGTH_INFORMATION, GET_MEDIA_TYPES, GUID, IDEREGS, IO_ERROR_LOG_PACKET, _IO_STATUS_BLOCK, IRP, IoAllocateErrorLogEntry, IoAllocateIrp, IoAllocateWorkItem, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, IoFreeWorkItem, IoGetNextIrpStackLocation, IoInvalidateDeviceRelations, IoMarkIrpPending, _IO_PRIORITY_HINT_IoPriorityNormal as IoPriorityNormal, IoQueueWorkItem, IoRegisterBootDriverReinitialization, IoRegisterDriverReinitialization, IoReportTargetDeviceChangeAsynchronous, IoSetCompletionRoutine, IoSetHardErrorOrVerifyDevice, IoWriteErrorLogEntry, KEVENT, KeInitializeEvent, KeReleaseMutex, LARGE_INTEGER, LPCGUID, _STORAGE_MEDIA_TYPE_MO_5_RW as MO_5_RW, _STORAGE_MEDIA_TYPE_MO_5_WO as MO_5_WO, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, _PARTITION_STYLE_PARTITION_STYLE_GPT as PARTITION_STYLE_GPT, _PARTITION_STYLE_PARTITION_STYLE_MBR as PARTITION_STYLE_MBR, PCCHAR, PDEVICE_MEDIA_INFO, PDEVICE_OBJECT, PDISK_CACHE_INFORMATION, PDISK_CACHE_SETTING, PDISK_GEOMETRY, PDRIVER_OBJECT, PEVENT_FILTER_DESCRIPTOR, PGETVERSIONINPARAMS, PGET_LENGTH_INFORMATION, PGET_MEDIA_TYPES, _STORAGE_MEDIA_TYPE_PINNACLE_APEX_5_RW as PINNACLE_APEX_5_RW, PIO_COMPLETION_ROUTINE, PIO_ERROR_LOG_PACKET, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PREASSIGN_BLOCKS, PREASSIGN_BLOCKS_EX, PSENDCMDINPARAMS, PSTORAGE_PREDICT_FAILURE, PUNICODE_STRING, PVERIFY_INFORMATION, REASSIGN_BLOCKS, REASSIGN_BLOCKS_EX, _MEDIA_TYPE_RemovableMedia as RemovableMedia, SENDCMDINPARAMS, SENDCMDOUTPARAMS, STORAGE_PREDICT_FAILURE, _EVENT_TYPE_SynchronizationEvent, TARGET_DEVICE_CUSTOM_NOTIFICATION, VERIFY_INFORMATION};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_DISK_USER_WRITE_CACHE_SETTING_DiskWriteCacheEnable as DiskWriteCacheEnable, _STORAGE_FAILURE_PREDICT_DATA as STORAGE_FAILURE_PREDICT_DATA, _STORAGE_FAILURE_PREDICT_STATUS as STORAGE_FAILURE_PREDICT_STATUS};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, DISK_DETECTION_INFO, DISK_PARTITION_INFO, DRIVER_OBJECT, IO_PRIORITY_HINT, IO_STACK_LOCATION, KPRIORITY, KPROCESSOR_MODE, KWAIT_REASON, PCSTR, STORAGE_MEDIA_TYPE, _GUID, _IO_STACK_LOCATION, _KEVENT, _LARGE_INTEGER, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16};
use ntkrnl_sys::ddk::{PDRIVER_REINITIALIZE};
use ntkrnl_sys::mm::{NonPagedPool, NonPagedPoolCacheAligned};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{_DEVICE_OBJECT};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{POOL_TYPE};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{DISPATCHER_HEADER, IO_STATUS_BLOCK, _EVENT_TYPE_NotificationEvent as NotificationEvent, _KWAIT_REASON_Executive as Executive};

extern "C" {
    static DiskMediaTypesExclude: DISK_MEDIA_TYPES_LIST;
    static DiskMediaTypes: DISK_MEDIA_TYPES_LIST;
    fn StorDebugPrint(level: i32, flags: u32, format: *const i8, ...);
    fn DbgPrint(format: *const i8, ...);
    // NOT `-> !` — the in-crate definition (geometry.rs / ddk_shims.rs) RETURNS; a noreturn
    // decl poisons the merged LLVM symbol and folds callers' fall-through into ud2.
    fn __int2c();
    fn TracePrint(level: i32, flag: u32, format: *const i8, ...);
    fn memmove(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}
extern "system" {
    // ClassIoComplete is declared below in extern "system" for correct i386 stdcall ABI
    fn ClassReleaseRemoveLock(Fdo: PDEVICE_OBJECT, Irp: PIRP);
    fn ClassCompleteRequest(Fdo: PDEVICE_OBJECT, Irp: PIRP, priority: i8);
    fn KeGetCurrentIrql() -> u8; // KIRQL is UCHAR — the kernel only defines al; an i32 decl reads garbage upper bits
    fn ClassModeSense( DeviceObject: PDEVICE_OBJECT, buffer: *mut i8, bufferSize: u32, pageCode: u8, ) -> u32;
    fn ClassFindModePage( buffer: *mut i8, bufferSize: u32, pageCode: u8, pageSavable: BOOLEAN, ) -> *mut MODE_CACHING_PAGE;
    fn ExFreePool(P: *mut core::ffi::c_void);
}
extern "C" {
    fn WPP_INIT_TRACING( DriverObject: PDRIVER_OBJECT, RegistryPath: PUNICODE_STRING, );
    fn RtlStringCchPrintfW( pszDest: *mut u16, cchDest: usize, pszFormat: *const u16, ... ) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn PAGED_CODE();
    fn CHECK_IRQL();
    fn max(a: usize, b: usize) -> usize;
    fn IsListEmpty(ListHead: *mut LIST_ENTRY) -> BOOLEAN;
    fn InsertTailList(ListHead: *mut LIST_ENTRY, Entry: *mut LIST_ENTRY);
    fn RemoveHeadList(ListHead: *mut LIST_ENTRY) -> *mut LIST_ENTRY;
    fn GetSrbScsiData( srb: *mut STORAGE_REQUEST_BLOCK, data_out: *mut u8, data_length: *mut u32, scsi_status: *mut u8, sense_buffer: *mut *mut core::ffi::c_void, sense_buffer_length: *mut u8, ) -> *mut CDB;
    fn ScsiGetSenseKeyAndCodes( sense_buffer: *const core::ffi::c_void, sense_buffer_length: u8, options: u8, sense_key: *mut u8, asc: *mut u8, ascq: *mut u8, ) -> u8;
}
unsafe extern "system" {
    fn _InterlockedExchange(Target: *mut i32, Value: i32) -> i32;
}
extern "system" {
    fn ClassInitialize( DriverObject: PVOID, RegistryPath: PVOID, InitializationData: *mut CLASS_INIT_DATA, ) -> NTSTATUS;
    fn ClassInitializeEx( DriverObject: PDRIVER_OBJECT, Guid: *mut GUID, Data: PVOID, ) -> NTSTATUS;
    fn IoAttachDeviceToDeviceStack( SourceDevice: *mut DEVICE_OBJECT, TargetDevice: *mut DEVICE_OBJECT, ) -> *mut DEVICE_OBJECT;
    fn IoDeleteDevice(DeviceObject: *mut DEVICE_OBJECT);
    fn IoGetAttachedDeviceReference(DeviceObject: *mut DEVICE_OBJECT) -> *mut DEVICE_OBJECT;
    fn ClassClaimDevice(DeviceObject: *mut DEVICE_OBJECT, Exclusive: i32) -> i32;
    fn ClassCreateDeviceObject( DriverObject: *mut DRIVER_OBJECT, DeviceName: *const i8, PhysicalDeviceObject: *mut DEVICE_OBJECT, Exclusive: i32, DeviceObject: *mut *mut DEVICE_OBJECT, ) -> i32;
    fn ZwMakeTemporaryObject(Handle: *mut core::ffi::c_void);
    fn ZwClose(Handle: *mut core::ffi::c_void);
    fn ZwCreateDirectoryObject( DirectoryHandle: *mut *mut core::ffi::c_void, DesiredAccess: u32, ObjectAttributes: *mut _OBJECT_ATTRIBUTES, ) -> i32;
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const u16);
    fn ClassSendSrbSynchronous( Fdo: PDEVICE_OBJECT, srb: *mut SCSI_REQUEST_BLOCK, data: *mut ::core::ffi::c_void, length: ULONG, fWait: BOOLEAN, ) -> NTSTATUS;
    fn IoGetIoPriorityHint(Irp: PIRP) -> IO_PRIORITY_HINT;
    fn KeWaitForSingleObject(Object: *mut core::ffi::c_void, WaitReason: KWAIT_REASON, WaitMode: KPROCESSOR_MODE, Alertable: BOOLEAN, Timeout: *mut LARGE_INTEGER) -> NTSTATUS;
    fn ClassAcquireRemoveLockEx(Fdo: PDEVICE_OBJECT, Irp: PIRP, File: *const i8, Line: u32);
    // ClassAsynchronousCompletion is declared below in extern "system" for correct i386 stdcall ABI
    fn DiskEnableInfoExceptions( fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION, enable: u8, ) -> i32;
    fn ClassSendDeviceIoControlSynchronous( ioControlCode: u32, targetDevice: PDEVICE_OBJECT, inputBuffer: *mut core::ffi::c_void, inputBufferLength: u32, outputBufferLength: u32, override_: BOOLEAN, ioStatus: *mut _IO_STATUS_BLOCK, );
    fn ExAllocatePoolWithTag(PoolType: u32, NumberOfBytes: SIZE_T, Tag: u32) -> *mut core::ffi::c_void;
    fn ClassSetDeviceParameter( fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, paramName: PWSTR, valueName: PWSTR, value: ULONG, );
    fn ClassDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
// ClassIoComplete and ClassAsynchronousCompletion use extern "system" so they get the correct
// calling convention per arch: stdcall on i386 (matches PIO_COMPLETION_ROUTINE), C on 64-bit.
extern "system" {
    fn ClassIoComplete(Fdo: PDEVICE_OBJECT, Irp: PIRP, Srb: *mut core::ffi::c_void) -> NTSTATUS;
    fn ClassAsynchronousCompletion(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, Context: *mut core::ffi::c_void) -> NTSTATUS;
}

/// IoIsErrorUserInduced — C macro from iofuncs.h, implemented as Rust inline.
#[inline(always)]
unsafe fn IoIsErrorUserInduced(Status: NTSTATUS) -> u8 {
    ((Status == STATUS_DEVICE_NOT_READY)
        || (Status == STATUS_IO_TIMEOUT)
        || (Status == STATUS_MEDIA_WRITE_PROTECTED)
        || (Status == STATUS_NO_MEDIA_IN_DEVICE)
        || (Status == STATUS_VERIFY_REQUIRED)
        || (Status == STATUS_UNRECOGNIZED_MEDIA)
        || (Status == STATUS_WRONG_VOLUME)) as u8
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const DiskDeviceParameterSubkey: [::core::ffi::c_ushort; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_ushort; 5]>(*b"D\0i\0s\0k\0\0\0") };

pub const DiskDeviceUserWriteCacheSetting: [::core::ffi::c_ushort; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_ushort; 22]>(
        *b"U\0s\0e\0r\0W\0r\0i\0t\0e\0C\0a\0c\0h\0e\0S\0e\0t\0t\0i\0n\0g\0\0\0",
    )
};

pub const DiskDeviceCacheIsPowerProtected: [::core::ffi::c_ushort; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_ushort; 22]>(
        *b"C\0a\0c\0h\0e\0I\0s\0P\0o\0w\0e\0r\0P\0r\0o\0t\0e\0c\0t\0e\0d\0\0\0",
    )
};

pub static mut GUID_IO_VOLUME_CHANGE: GUID = _GUID {
    Data1: 0x7373654a as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x812a as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_DISMOUNT: GUID = _GUID {
    Data1: 0xd16a55e8 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x1059 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_DISMOUNT_FAILED: GUID = _GUID {
    Data1: 0xe3c5b178 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x105d as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_MOUNT: GUID = _GUID {
    Data1: 0xb5804878 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x1a96 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_LOCK: GUID = _GUID {
    Data1: 0x50708874 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc9af as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_LOCK_FAILED: GUID = _GUID {
    Data1: 0xae2eed10 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xba8 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_UNLOCK: GUID = _GUID {
    Data1: 0x9a8c3d68 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xd0cb as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_NAME_CHANGE: GUID = _GUID {
    Data1: 0x2de97f83 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4c06 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_PREPARING_EJECT: GUID = _GUID {
    Data1: 0xc79eb16e as ::core::ffi::c_uint,
    Data2: 0xdac as ::core::ffi::c_ushort,
    Data3: 0x4e7a as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_PHYSICAL_CONFIGURATION_CHANGE: GUID = _GUID {
    Data1: 0x2de97f84 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4c06 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_FVE_STATUS_CHANGE: GUID = _GUID {
    Data1: 0x62998b2 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xee1f as ::core::ffi::c_ushort,
    Data3: 0x4b6a as ::core::ffi::c_ushort,
    Data4: [
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_DEVICE_INTERFACE: GUID = _GUID {
    Data1: 0x53f5630d as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb6bf as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_CHANGE_SIZE: GUID = _GUID {
    Data1: 0x3a1625be as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xad03 as ::core::ffi::c_ushort,
    Data3: 0x49f1 as ::core::ffi::c_ushort,
    Data4: [
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_MEDIA_ARRIVAL: GUID = _GUID {
    Data1: 0xd07433c0 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_MEDIA_REMOVAL: GUID = _GUID {
    Data1: 0xd07433c1 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_CDROM_EXCLUSIVE_LOCK: GUID = _GUID {
    Data1: 0xbc56c139 as ::core::ffi::c_uint,
    Data2: 0x7a10 as ::core::ffi::c_ushort,
    Data3: 0x47ee as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_CDROM_EXCLUSIVE_UNLOCK: GUID = _GUID {
    Data1: 0xa3b6d27d as ::core::ffi::c_uint,
    Data2: 0x5e35 as ::core::ffi::c_ushort,
    Data3: 0x4885 as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DEVICE_BECOMING_READY: GUID = _GUID {
    Data1: 0xd07433f0 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DEVICE_EXTERNAL_REQUEST: GUID = _GUID {
    Data1: 0xd07433d0 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_MEDIA_EJECT_REQUEST: GUID = _GUID {
    Data1: 0xd07433d1 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DRIVE_REQUIRES_CLEANING: GUID = _GUID {
    Data1: 0x7207877c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x90ed as ::core::ffi::c_ushort,
    Data3: 0x44e5 as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_TAPE_ERASE: GUID = _GUID {
    Data1: 0x852d11eb as ::core::ffi::c_uint,
    Data2: 0x4bb8 as ::core::ffi::c_ushort,
    Data3: 0x4507 as ::core::ffi::c_ushort,
    Data4: [
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DISK_CLONE_ARRIVAL: GUID = _GUID {
    Data1: 0x6a61885b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7c39 as ::core::ffi::c_ushort,
    Data3: 0x43dd as ::core::ffi::c_ushort,
    Data4: [
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DISK_LAYOUT_CHANGE: GUID = _GUID {
    Data1: 0x11dff54c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8469 as ::core::ffi::c_ushort,
    Data3: 0x41f9 as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut DiskETWEnabled: BOOLEAN = FALSE as BOOLEAN;

pub static mut DiskIsPastReinit: BOOLEAN = FALSE as BOOLEAN;

pub static mut GUID_NULL: GUID = _GUID {
    Data1: 0 as ::core::ffi::c_uint,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

pub const DEBUG_MAIN_SOURCE: i32 = 1;

#[repr(C)]
pub struct DISK_GEOMETRY_EX_INTERNAL {
    pub Geometry: DISK_GEOMETRY,
    pub DiskSize: LARGE_INTEGER,
    pub Partition: DISK_PARTITION_INFO,
    pub Detection: DISK_DETECTION_INFO,
}
pub type PDISK_GEOMETRY_EX_INTERNAL = *mut DISK_GEOMETRY_EX_INTERNAL;

extern "C" {
    // defined in geometry.rs / ddk_shims.rs (#[no_mangle] extern "C", cdecl per types_i386).
    pub fn DiskDriverReinitialization(DriverObject: PDRIVER_OBJECT, Nothing: PVOID, Count: ULONG);
}

// IoGetCurrentIrpStackLocation is a header FORCEINLINE — bindgen_i386 declares it cdecl,
// and the in-crate ddk_shims definition is extern "C"; this decl must match BOTH or the
// in-module CC mismatch folds callers into ud2 (the classpnp lesson).
extern "C" {
    fn IoGetCurrentIrpStackLocation(Irp: *mut IRP) -> *mut IO_STACK_LOCATION;
}

// ObfDereferenceObject is FASTCALL on i386 (@ObfDereferenceObject@4), cdecl on 64-bit —
// matches the ntkrnl-sys bindgen decl per arch.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ObfDereferenceObject(Object: *mut core::ffi::c_void);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ObfDereferenceObject(Object: *mut core::ffi::c_void);
}


// Forward declarations for unknown externals
unsafe extern "system" {
}

// Constants
use ntkrnl_sys::ddk::ULONG_PTR; // per-arch: u64 on amd64/arm64, u32 on i386 (matches IoStatus.Information)

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

// moved from extern "system": types_i386 declares these cdecl (no NTAPI in disk.h);
// the defs are extern "C" — CC must agree or LLVM folds callers to ud2.
extern "C" {
    fn DiskReadFailurePredictStatus( FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION, SmartStatus: *mut STORAGE_FAILURE_PREDICT_STATUS, ) -> NTSTATUS;
    fn DiskReadFailurePredictData( FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION, Buffer: *mut STORAGE_FAILURE_PREDICT_DATA, ) -> NTSTATUS;
    pub fn DiskInitializeReregistration() -> NTSTATUS;
    pub fn DiskSaveDetectInfo(DriverObject: PDRIVER_OBJECT) -> NTSTATUS;
    pub fn DiskReadDriveCapacity(Fdo: PDEVICE_OBJECT) -> NTSTATUS;
    fn DiskGetDetectInfo( fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, Detection: *mut DISK_DETECTION_INFO, ) -> NTSTATUS;
    fn DiskEnableDisableFailurePrediction( fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION, enable: u8, ) -> i32;
}
// Header consts the translation referenced as extern statics (harvested from the headers):
const IOCTL_SCSI_MINIPORT: u32 = 0x0004D008; // CTL_CODE(IOCTL_SCSI_BASE, 0x0402, METHOD_BUFFERED, FILE_READ|WRITE_ACCESS)
const IOCTL_SCSI_MINIPORT_SMART_VERSION: u32 = 0x001B0500; // (FILE_DEVICE_SCSI << 16) + 0x0500
const DISK_TAG_SMART: u32 = 0x61446353; // 'aDcS' (disk.h)




const SCSIOP_TEST_UNIT_READY: UCHAR = 0x00;
const MODE_DSP_WRITE_PROTECT: i32 = 0x80;


const SCSI_REQUEST_BLOCK_SIZE: ULONG = 0x58;


const DISK_TAG_SRB: ULONG = 0x53446353;
const DISK_TAG_MODE_DATA: ULONG = 0x4D446353;

const STATUS_NO_MEDIA_IN_DEVICE: NTSTATUS = 0xC0000013u32 as i32 ;

// Forward declarations for functions not available in ntkrnl_sys

// Constants used by the function (values typical of Windows DDK; adjust as needed)
// ddk/srb.h:55 `#define SP_UNTAGGED ((UCHAR) ~0)` == 0xFF. Not ULONG_MAX: the
// neighbouring SP_UNINITIALIZED_VALUE is the ((ULONG) ~0) one.
const SP_UNTAGGED: u32 = 0xFF;
const SRB_SIMPLE_TAG_REQUEST: u16 = 0x20; // srb.h
const CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE: u32 = 0x60;
const DEV_POWER_PROTECTED: i32 = 0x10;
const SCSIOP_SYNCHRONIZE_CACHE: u8 = 0x35;
const SCSIOP_MEDIUM_REMOVAL: u8 = 0x1E;
const IO_NO_INCREMENT: i8 = 0;
const TRUE: BOOLEAN = 1;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_PENDING: NTSTATUS = 0x00000103i32;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as i32 ;

#[no_mangle]


pub unsafe extern "system" fn DiskDriverReinit(
    DriverObject: PDRIVER_OBJECT,
    Nothing: PVOID,
    Count: ULONG,
) {
    // UNREFERENCED_PARAMETER(DriverObject);
    let _ = DriverObject;
    // UNREFERENCED_PARAMETER(Nothing);
    let _ = Nothing;
    // UNREFERENCED_PARAMETER(Count);
    let _ = Count;

    DiskIsPastReinit = 1;
}
#[no_mangle]
pub unsafe extern "system" fn DiskBootDriverReinit(
    DriverObject: PDRIVER_OBJECT, // C: PDRIVER_OBJECT — NOT a device object; no deref
    Nothing: PVOID,
    Count: ULONG,
) {
    IoRegisterDriverReinitialization(
        DriverObject,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDRIVER_OBJECT, PVOID, ULONG),
            _,
        >(DiskDriverReinit)),
        core::ptr::null_mut(),
    );
    DiskDriverReinitialization(DriverObject.cast(), Nothing, Count);
}





#[no_mangle] // NTAPI: extern "system" → _DriverEntry@8 on i386, DriverEntry on 64-bit
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut InitializationData: CLASS_INIT_DATA = CLASS_INIT_DATA {
        InitializationDataSize: 0 as ULONG,
        FdoData: CLASS_DEV_INFO {
            DeviceExtensionSize: 0,
            DeviceType: 0,
            StackSize: 0,
            DeviceCharacteristics: 0,
            ClassError: None,
            ClassReadWriteVerification: None,
            ClassDeviceControl: None,
            ClassShutdownFlush: None,
            ClassCreateClose: None,
            ClassInitDevice: None,
            ClassStartDevice: None,
            ClassPowerDevice: None,
            ClassStopDevice: None,
            ClassRemoveDevice: None,
            ClassQueryPnpCapabilities: None,
            ClassWmiInfo: CLASS_WMI_INFO {
                GuidCount: 0,
                GuidRegInfo: core::ptr::null_mut::<GUIDREGINFO>(),
                ClassQueryWmiRegInfo: None,
                ClassQueryWmiDataBlock: None,
                ClassSetWmiDataBlock: None,
                ClassSetWmiDataItem: None,
                ClassExecuteWmiMethod: None,
                ClassWmiFunctionControl: None,
            },
        },
        PdoData: CLASS_DEV_INFO {
            DeviceExtensionSize: 0,
            DeviceType: 0,
            StackSize: 0,
            DeviceCharacteristics: 0,
            ClassError: None,
            ClassReadWriteVerification: None,
            ClassDeviceControl: None,
            ClassShutdownFlush: None,
            ClassCreateClose: None,
            ClassInitDevice: None,
            ClassStartDevice: None,
            ClassPowerDevice: None,
            ClassStopDevice: None,
            ClassRemoveDevice: None,
            ClassQueryPnpCapabilities: None,
            ClassWmiInfo: CLASS_WMI_INFO {
                GuidCount: 0,
                GuidRegInfo: core::ptr::null_mut::<GUIDREGINFO>(),
                ClassQueryWmiRegInfo: None,
                ClassQueryWmiDataBlock: None,
                ClassSetWmiDataBlock: None,
                ClassSetWmiDataItem: None,
                ClassExecuteWmiMethod: None,
                ClassWmiFunctionControl: None,
            },
        },
        ClassAddDevice: None,
        ClassEnumerateDevice: None,
        ClassQueryId: None,
        ClassStartIo: None,
        ClassUnload: None,
        ClassTick: None,
    };
    let mut classQueryWmiRegInfoExList: CLASS_QUERY_WMI_REGINFO_EX_LIST = CLASS_QUERY_WMI_REGINFO_EX_LIST {
        Size: 0 as ULONG,
        ClassFdoQueryWmiRegInfoEx: None,
        ClassPdoQueryWmiRegInfoEx: None,
    };
    let mut guidQueryRegInfoEx: GUID = GUID {
        Data1: 0xe34b11 as c_uint,
        Data2: 0x2444 as c_ushort,
        Data3: 0x4745 as c_ushort,
        Data4: [
            0xa5 as c_uchar,
            0x3d as c_uchar,
            0x62 as c_uchar,
            0x1 as c_uchar,
            0 as c_uchar,
            0xcd as c_uchar,
            0x82 as c_uchar,
            0xf7 as c_uchar,
        ],
    };
    let mut guidSrbSupport: GUID = GUID {
        Data1: 0xa483941 as c_uint,
        Data2: 0xbdfd as c_ushort,
        Data3: 0x4f7b as c_ushort,
        Data4: [
            0xbe as c_uchar,
            0x95 as c_uchar,
            0xce as c_uchar,
            0xe2 as c_uchar,
            0xa2 as c_uchar,
            0x16 as c_uchar,
            0x9 as c_uchar,
            0xc as c_uchar,
        ],
    };
    let mut srbSupport: ULONG = 0;
    let mut status: NTSTATUS;

    WPP_INIT_TRACING(DriverObject, RegistryPath);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    DiskSaveDetectInfo(DriverObject);

    InitializationData.InitializationDataSize =
        ::core::mem::size_of::<CLASS_INIT_DATA>() as ULONG;

    InitializationData.FdoData.DeviceExtensionSize = FUNCTIONAL_EXTENSION_SIZE as ULONG;
    InitializationData.FdoData.DeviceType = FILE_DEVICE_DISK as ULONG;
    InitializationData.FdoData.DeviceCharacteristics = FILE_DEVICE_SECURE_OPEN as ULONG;
    InitializationData.FdoData.ClassInitDevice =
        Some(DiskInitFdo);
    InitializationData.FdoData.ClassStartDevice =
        Some(DiskStartFdo);
    InitializationData.FdoData.ClassStopDevice =
        Some(DiskStopDevice);
    InitializationData.FdoData.ClassRemoveDevice =
        Some(DiskRemoveDevice);
    InitializationData.FdoData.ClassPowerDevice =
        Some(ClassSpinDownPowerHandler);
    InitializationData.FdoData.ClassError = Some(core::mem::transmute::<
        unsafe extern "system" fn(PDEVICE_OBJECT, *mut SCSI_REQUEST_BLOCK, *mut NTSTATUS, *mut BOOLEAN),
        _,
    >(DiskFdoProcessError));
    InitializationData.FdoData.ClassReadWriteVerification =
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            _,
        >(DiskReadWriteVerification));
    InitializationData.FdoData.ClassDeviceControl =
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            _,
        >(DiskDeviceControl));
    InitializationData.FdoData.ClassShutdownFlush =
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            _,
        >(DiskShutdownFlush));
    InitializationData.FdoData.ClassCreateClose = None;
    InitializationData.FdoData.ClassWmiInfo.GuidCount = 7 as ULONG;
    InitializationData.FdoData.ClassWmiInfo.GuidRegInfo =
        core::ptr::addr_of_mut!(DiskWmiFdoGuidList) as *mut GUIDREGINFO as PGUIDREGINFO;
    InitializationData.FdoData.ClassWmiInfo.ClassQueryWmiRegInfo = Some(
        DiskFdoQueryWmiRegInfo,
    );
    InitializationData
        .FdoData
        .ClassWmiInfo
        .ClassQueryWmiDataBlock = Some(
        DiskFdoQueryWmiDataBlock,
    );
    InitializationData.FdoData.ClassWmiInfo.ClassSetWmiDataBlock = Some(
        DiskFdoSetWmiDataBlock,
    );
    InitializationData.FdoData.ClassWmiInfo.ClassSetWmiDataItem = Some(
        DiskFdoSetWmiDataItem,
    );
    InitializationData
        .FdoData
        .ClassWmiInfo
        .ClassExecuteWmiMethod = Some(
        DiskFdoExecuteWmiMethod,
    );
    InitializationData
        .FdoData
        .ClassWmiInfo
        .ClassWmiFunctionControl = Some(
        DiskWmiFunctionControl,
    );
    InitializationData.ClassAddDevice =
        Some(DiskAddDevice);
    InitializationData.ClassUnload =
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDRIVER_OBJECT),
            _,
        >(DiskUnload));

    DiskInitializeReregistration();

    status = ClassInitialize(
        DriverObject as PVOID,
        RegistryPath as PVOID,
        core::ptr::addr_of_mut!(InitializationData),
    ) as NTSTATUS;

    if status >= 0 as c_long {
        IoRegisterBootDriverReinitialization(
            DriverObject,
            Some(core::mem::transmute::<
                unsafe extern "system" fn(PDRIVER_OBJECT, PVOID, ULONG),
                _,
            >(DiskBootDriverReinit)),
            core::ptr::null_mut(),
        );
    }

    classQueryWmiRegInfoExList.Size =
        ::core::mem::size_of::<CLASS_QUERY_WMI_REGINFO_EX_LIST>() as ULONG;
    classQueryWmiRegInfoExList.ClassFdoQueryWmiRegInfoEx = Some(
        DiskFdoQueryWmiRegInfoEx,
    );

    ClassInitializeEx(
        DriverObject,
        &raw mut guidQueryRegInfoEx,
        &raw mut classQueryWmiRegInfoExList as PVOID,
    );

    srbSupport = (CLASS_SRB_SCSI_REQUEST_BLOCK | CLASS_SRB_STORAGE_REQUEST_BLOCK) as ULONG;

    if !(ClassInitializeEx(
        DriverObject,
        &raw mut guidSrbSupport,
        &raw mut srbSupport as PVOID,
    ) as NTSTATUS
        >= 0 as c_long)
    {
        if 0 as c_int == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const c_char,
                270 as c_int,
                b"FALSE\0" as *const u8 as *const c_char,
            );
            __int2c();
        }
    }

    return status;
}





#[no_mangle]
pub unsafe extern "system" fn DiskUnload(DriverObject: PDRIVER_OBJECT) {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                285i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        } else {
        }
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    DiskCleanupDetectInfo((DriverObject).cast());
}
#[no_mangle]
pub unsafe extern "C" fn DiskCreateFdo(
    DriverObject: *mut DRIVER_OBJECT,
    PhysicalDeviceObject: *mut DEVICE_OBJECT,
    DeviceCount: *mut u32,
    DasdAccessOnly: i32,
) -> i32 {
    const STATUS_OBJECT_NAME_COLLISION: i32 = 0xC0000035u32 as i32;
    const STATUS_OBJECT_NAME_EXISTS: i32 = 0x40000000u32 as i32;
    const STATUS_SUCCESS: i32 = 0;
    const STATUS_UNSUCCESSFUL: i32 = 0xC0000001u32 as i32;
    const DIRECTORY_ALL_ACCESS: u32 = 0x000F000F; // obtypes.h: STANDARD_RIGHTS_REQUIRED | 0xF (was 0xF01FF)
    const APC_LEVEL: u32 = 1;
    const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let mut deviceName: *const i8 = core::ptr::null();
    let mut handle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut lowerDevice: *mut DEVICE_OBJECT = core::ptr::null_mut();
    let mut deviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();
    let mut fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut status: i32 = 0;

    if KeGetCurrentIrql() as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"home/curi/Projects/os-playground/vibeos-wt-disk/drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                348i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }
    *DeviceCount = 0u32;
    loop {
        let mut dirBuffer: [u16; 64] = [0u16; 64];
        let mut dirName: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        let mut objectAttribs: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
            Length: 0,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: core::ptr::null_mut(),
            Attributes: 0,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        };
        status = RtlStringCchPrintfW(
            (&raw mut dirBuffer) as *mut u16,
            (core::mem::size_of::<[u16; 64]>() / core::mem::size_of::<u16>()).wrapping_sub(1),
            w!("\\Device\\Harddisk%d").as_ptr() as *const u16 ,
            *DeviceCount,
        );
        if !(status >= 0) {
            StorDebugPrint(
                1i32,
                TRACE_FLAG_PNP,
                b"DiskCreateFdo: Format symbolic link failed with error: 0x%X\n\0" as *const u8
                    as *const i8,
                status,
            );
            return status;
        }
        RtlInitUnicodeString(
            &raw mut dirName,
            (&raw mut dirBuffer as *mut u16) as *const u16,
        );
        objectAttribs.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        objectAttribs.RootDirectory = core::ptr::null_mut();
        objectAttribs.ObjectName = &raw mut dirName;
        objectAttribs.Attributes = (0x40u32 | 0x10u32 | 0x200u32) as u32;
        objectAttribs.SecurityDescriptor = core::ptr::null_mut();
        objectAttribs.SecurityQualityOfService = core::ptr::null_mut();
        status = ZwCreateDirectoryObject(
            &raw mut handle,
            DIRECTORY_ALL_ACCESS,
            &raw mut objectAttribs,
        );
        *DeviceCount = (*DeviceCount).wrapping_add(1);
        if !(status == STATUS_OBJECT_NAME_COLLISION || status == STATUS_OBJECT_NAME_EXISTS) {
            break;
        }
    }
    if !(status >= 0) {
        StorDebugPrint(
            1i32,
            TRACE_FLAG_PNP,
            b"DiskCreateFdo: Could not create directory - %lx\n\0" as *const u8 as *const i8,
            status,
        );
        return status;
    }
    *DeviceCount = (*DeviceCount).wrapping_sub(1);
    lowerDevice = IoGetAttachedDeviceReference(PhysicalDeviceObject);
    status = ClassClaimDevice(lowerDevice, 0i32);
    if !(status >= 0) {
        ZwMakeTemporaryObject(handle);
        ZwClose(handle);
        ObfDereferenceObject(lowerDevice as *mut core::ffi::c_void);
        return status;
    }
    status = DiskGenerateDeviceName(*DeviceCount, (&raw mut deviceName).cast());
    if !(status >= 0) {
        StorDebugPrint(
            1i32,
            TRACE_FLAG_PNP,
            b"DiskCreateFdo - couldn't create name %lx\n\0" as *const u8 as *const i8,
            status,
        );
    } else {
        status = ClassCreateDeviceObject(
            DriverObject,
            deviceName,
            PhysicalDeviceObject,
            1i32,
            &raw mut deviceObject,
        );
        if !(status >= 0) {
            StorDebugPrint(
                1i32,
                TRACE_FLAG_PNP,
                b"DiskCreateFdo: Can not create device object %s\n\0" as *const u8 as *const i8,
                deviceName,
            );
        } else {
            if !deviceName.is_null() {
                ExFreePool(deviceName as *mut core::ffi::c_void);
                deviceName = core::ptr::null();
            }
            (*deviceObject).Flags |= 0x10u32;
            fdoExtension = (*deviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
            if DasdAccessOnly != 0 {
                (*(*deviceObject).Vpb).Flags = ((*(*deviceObject).Vpb).Flags as i32 | 0x10i32)
                    as u16;
            }
            (*fdoExtension).LockCount = 0i32;
            (*fdoExtension).DeviceNumber = *DeviceCount as u32;
            if (*lowerDevice).AlignmentRequirement > (*deviceObject).AlignmentRequirement {
                (*deviceObject).AlignmentRequirement = (*lowerDevice).AlignmentRequirement;
            }
            (*fdoExtension).LowerPdo = PhysicalDeviceObject;
            (*fdoExtension)
                .__bindgen_anon_1
                .CommonExtension
                .as_mut().LowerDeviceObject = IoAttachDeviceToDeviceStack(deviceObject, PhysicalDeviceObject);
            if (*fdoExtension)
                .__bindgen_anon_1
                .CommonExtension
                .as_ref().LowerDeviceObject
                .is_null()
            {
                status = STATUS_UNSUCCESSFUL;
            } else {
                (*deviceObject).Flags &= !(0x80u32);
                (*fdoExtension).DeviceDirectory = handle;
                ObfDereferenceObject(lowerDevice as *mut core::ffi::c_void);
                return STATUS_SUCCESS;
            }
        }
    }
    if !deviceObject.is_null() {
        IoDeleteDevice(deviceObject);
    }
    if !deviceName.is_null() {
        ExFreePool(deviceName as *mut core::ffi::c_void);
        deviceName = core::ptr::null();
    }
    ObfDereferenceObject(lowerDevice as *mut core::ffi::c_void);
    ZwMakeTemporaryObject(handle);
    ZwClose(handle);
    return status;
}





#[no_mangle]
pub unsafe extern "system" fn DiskReadWriteVerification(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let irpSp = IoGetCurrentIrpStackLocation(Irp);
    let bytes_per_sector = (*(*commonExtension).PartitionZeroExtension).DiskGeometry.BytesPerSector as u32;
    // Read the low 32 bits of ByteOffset without taking a reference to a packed field (E0793).
    let low_part = (*irpSp).Parameters.Read.ByteOffset.QuadPart as u32;
    let residualBytes = (*irpSp).Parameters.Read.Length & (bytes_per_sector.wrapping_sub(1));
    let residualOffset = low_part & (bytes_per_sector.wrapping_sub(1));
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if (*irpSp).Parameters.Read.ByteOffset.QuadPart > (*commonExtension).PartitionLength.QuadPart
        || (*irpSp).Parameters.Read.ByteOffset.QuadPart < 0
        || residualBytes != 0
        || residualOffset != 0
    {
        if residualOffset != 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                589i32,
                b"residualOffset == 0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        status = STATUS_INVALID_PARAMETER;
    } else {
        let bytesRemaining = ((*commonExtension).PartitionLength.QuadPart
            - (*irpSp).Parameters.Read.ByteOffset.QuadPart) as u64;
        if (*irpSp).Parameters.Read.Length as u64 > bytesRemaining {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status < 0 {
        let disk_data = (*commonExtension).DriverData as *mut DISK_DATA;
        status = (*disk_data).ReadyStatus;
        if status < 0 {
            TracePrint(
                0i32,
                0u32,
                b"DiskReadWriteVerification: ReadyStatus is %lx\n\0" as *const u8 as *const i8,
                status,
            );
            if IoIsErrorUserInduced(status) != 0 && !(*Irp).Tail.Overlay.Thread.is_null() {
                IoSetHardErrorOrVerifyDevice(Irp, DeviceObject);
            }
        } else if residualBytes == 0 && residualOffset == 0 {
            status = STATUS_SUCCESS;
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    status
}






#[no_mangle]
pub unsafe extern "C" fn DiskDetermineMediaTypes(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP,
    MediumType: u8,
    DensityCode: u8,
    MediaPresent: u8,   // BOOLEAN → u8 (0/1)
    IsWritable: u8,
) -> NTSTATUS {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mediaTypes = (*Irp).AssociatedIrp.SystemBuffer as PGET_MEDIA_TYPES;
    let mut mediaInfo = &raw mut (*mediaTypes).MediaInfo as *mut DEVICE_MEDIA_INFO;
    let mut deviceMatched: u8 = 0;   // BOOLEAN FALSE

    // PAGED_CODE(): assert IRQL <= APC_LEVEL
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        // This always-true branch triggers a breakpoint.
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                693i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    // NT_ASSERT for buffer size
    if !((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize
        >= core::mem::size_of::<GET_MEDIA_TYPES>())
    {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
            b"drivers/storage/class/disk/disk.c\0"
                as *const u8 as *const i8,
            701i32,
            b"irpStack->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(GET_MEDIA_TYPES)\0"
                as *const u8 as *const i8,
        );
        __int2c();
    }

    // Check file removable flag
    if (*Fdo).Characteristics & FILE_REMOVABLE_MEDIA == 0 {
        // Fixed disk.
        (*mediaTypes).DeviceType = FILE_DEVICE_DISK;
        (*mediaTypes).MediaInfoCount = 1;

        (*mediaInfo).DeviceSpecific.DiskInfo.Cylinders.QuadPart =
            (*fdoExtension).DiskGeometry.Cylinders.QuadPart;
        (*mediaInfo).DeviceSpecific.DiskInfo.MediaType = FixedMedia as STORAGE_MEDIA_TYPE;
        (*mediaInfo).DeviceSpecific.DiskInfo.TracksPerCylinder =
            (*fdoExtension).DiskGeometry.TracksPerCylinder;
        (*mediaInfo).DeviceSpecific.DiskInfo.SectorsPerTrack =
            (*fdoExtension).DiskGeometry.SectorsPerTrack;
        (*mediaInfo).DeviceSpecific.DiskInfo.BytesPerSector =
            (*fdoExtension).DiskGeometry.BytesPerSector;
        (*mediaInfo).DeviceSpecific.DiskInfo.NumberMediaSides = 1;
        (*mediaInfo).DeviceSpecific.DiskInfo.MediaCharacteristics =
            MEDIA_CURRENTLY_MOUNTED | MEDIA_READ_WRITE;

        if IsWritable == 0 {
            (*mediaInfo).DeviceSpecific.DiskInfo.MediaCharacteristics |= MEDIA_WRITE_PROTECTED;
        }
    } else {
        // Removable disk.
        let deviceDescriptor = (*fdoExtension).DeviceDescriptor;
        let vendorId =
            (deviceDescriptor as *const i8).offset((*deviceDescriptor).VendorIdOffset as isize);
        let productId =
            (deviceDescriptor as *const i8).offset((*deviceDescriptor).ProductIdOffset as isize);
        let productRevision =
            (deviceDescriptor as *const i8).offset((*deviceDescriptor).ProductRevisionOffset as isize);

        StorDebugPrint(
            4i32,
            TRACE_FLAG_IOCTL,
            b"DiskDetermineMediaTypes: Vendor %s, Product %s\n\0" as *const u8 as *const i8,
            vendorId,
            productId,
        );

        // Look for match in exclusion table; if found, skip the main table entirely.
        let mut skip_second_loop = false;
        {
            let exclude_list =
                &raw const DiskMediaTypesExclude as *const DISK_MEDIA_TYPES_LIST;
            let mut i: u32 = 0;
            while !(*exclude_list.offset(i as isize)).VendorId.is_null() {
                let entry = &*exclude_list.offset(i as isize);
                if strncmp(
                    (*entry).VendorId as *const i8,
                    vendorId as *const i8,
                    strlen((*entry).VendorId as *const i8),
                ) != 0
                {
                    i = i.wrapping_add(1);
                    continue;
                }
                if !(*entry).ProductId.is_null()
                    && strncmp(
                        (*entry).ProductId as *const i8,
                        productId as *const i8,
                        strlen((*entry).ProductId as *const i8),
                    ) != 0
                {
                    i = i.wrapping_add(1);
                    continue;
                }
                skip_second_loop = true;
                break; // exit exclusion loop
            }
        }

        if !skip_second_loop {
            // Main media type table.
            let media_list =
                &raw const DiskMediaTypes as *const DISK_MEDIA_TYPES_LIST;
            let mut i: u32 = 0;
            while !(*media_list.offset(i as isize)).VendorId.is_null() {
                let entry = &*media_list.offset(i as isize);

                // Match vendor
                if strncmp(
                    (*entry).VendorId as *const i8,
                    vendorId as *const i8,
                    strlen((*entry).VendorId as *const i8),
                ) != 0
                {
                    i = i.wrapping_add(1);
                    continue;
                }
                // Match product
                if !(*entry).ProductId.is_null()
                    && strncmp(
                        (*entry).ProductId as *const i8,
                        productId as *const i8,
                        strlen((*entry).ProductId as *const i8),
                    ) != 0
                {
                    i = i.wrapping_add(1);
                    continue;
                }
                // Match revision
                if !(*entry).Revision.is_null()
                    && strncmp(
                        (*entry).Revision as *const i8,
                        productRevision as *const i8,
                        strlen((*entry).Revision as *const i8),
                    ) != 0
                {
                    i = i.wrapping_add(1);
                    continue;
                }

                // All criteria matched.
                deviceMatched = 1; // TRUE

                (*mediaTypes).DeviceType = FILE_DEVICE_DISK;
                (*mediaTypes).MediaInfoCount = (*entry).NumberOfTypes;

                // Ensure buffer large enough.
                let sizeNeeded = (core::mem::offset_of!(GET_MEDIA_TYPES, MediaInfo)
                    + ((*entry).NumberOfTypes as usize) * core::mem::size_of::<DEVICE_MEDIA_INFO>())
                    as u32;
                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength < sizeNeeded {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                    return STATUS_BUFFER_TOO_SMALL;
                }

                // Fill in media info for each type.
                let mut j: u32 = 0;
                while j < (*entry).NumberOfTypes {
                    (*mediaInfo)
                        .DeviceSpecific
                        .RemovableDiskInfo
                        .Cylinders
                        .QuadPart = (*fdoExtension).DiskGeometry.Cylinders.QuadPart;
                    (*mediaInfo)
                        .DeviceSpecific
                        .RemovableDiskInfo
                        .TracksPerCylinder = (*fdoExtension).DiskGeometry.TracksPerCylinder;
                    (*mediaInfo)
                        .DeviceSpecific
                        .RemovableDiskInfo
                        .SectorsPerTrack = (*fdoExtension).DiskGeometry.SectorsPerTrack;
                    (*mediaInfo)
                        .DeviceSpecific
                        .RemovableDiskInfo
                        .BytesPerSector = (*fdoExtension).DiskGeometry.BytesPerSector;
                    (*mediaInfo)
                        .DeviceSpecific
                        .RemovableDiskInfo
                        .NumberMediaSides = (*entry).NumberOfSides;

                    let mediaType = (*entry).MediaTypes[j as usize];
                    (*mediaInfo)
                        .DeviceSpecific
                        .RemovableDiskInfo
                        .MediaType = mediaType;

                    if mediaType as u32 == MO_5_WO as u32 {
                        (*mediaInfo)
                            .DeviceSpecific
                            .RemovableDiskInfo
                            .MediaCharacteristics = MEDIA_WRITE_ONCE;
                    } else {
                        (*mediaInfo)
                            .DeviceSpecific
                            .RemovableDiskInfo
                            .MediaCharacteristics = MEDIA_READ_WRITE;
                    }

                    if MediaPresent != 0 {
                        let mut currentMedia: u32 = 0;
                        if MediumType as i32 == 2 {
                            currentMedia = MO_5_WO as u32;
                        } else if MediumType as i32 == 3 {
                            currentMedia = MO_5_RW as u32;
                            if DensityCode as i32 == 0x87 {
                                currentMedia = PINNACLE_APEX_5_RW as u32;
                            }
                        }

                        if currentMedia != 0 {
                            if mediaType as u32 == currentMedia {
                                (*mediaInfo)
                                    .DeviceSpecific
                                    .RemovableDiskInfo
                                    .MediaCharacteristics |= MEDIA_CURRENTLY_MOUNTED;
                            }
                        } else {
                            (*mediaInfo)
                                .DeviceSpecific
                                .RemovableDiskInfo
                                .MediaCharacteristics |= MEDIA_CURRENTLY_MOUNTED;
                        }
                    }

                    if IsWritable == 0 {
                        (*mediaInfo)
                            .DeviceSpecific
                            .RemovableDiskInfo
                            .MediaCharacteristics |= MEDIA_WRITE_PROTECTED;
                    }

                    mediaInfo = mediaInfo.offset(1);
                    j = j.wrapping_add(1);
                }

                i = i.wrapping_add(1);
                // Loop continues normally (no break) to process the next entry.
            }
        }

        // If no device matched, build entry for unknown.
        if deviceMatched == 0 {
            StorDebugPrint(
                3i32,
                TRACE_FLAG_IOCTL,
                b"DiskDetermineMediaTypes: Unknown device. Vendor: %s Product: %s Revision: %s\n\0"
                    as *const u8 as *const i8,
                vendorId,
                productId,
                productRevision,
            );

            (*mediaTypes).DeviceType = FILE_DEVICE_DISK;
            (*mediaTypes).MediaInfoCount = 1;

            (*mediaInfo)
                .DeviceSpecific
                .RemovableDiskInfo
                .Cylinders
                .QuadPart = (*fdoExtension).DiskGeometry.Cylinders.QuadPart;
            (*mediaInfo)
                .DeviceSpecific
                .RemovableDiskInfo
                .MediaType = RemovableMedia as STORAGE_MEDIA_TYPE;
            (*mediaInfo)
                .DeviceSpecific
                .RemovableDiskInfo
                .TracksPerCylinder = (*fdoExtension).DiskGeometry.TracksPerCylinder;
            (*mediaInfo)
                .DeviceSpecific
                .RemovableDiskInfo
                .SectorsPerTrack = (*fdoExtension).DiskGeometry.SectorsPerTrack;
            (*mediaInfo)
                .DeviceSpecific
                .RemovableDiskInfo
                .BytesPerSector = (*fdoExtension).DiskGeometry.BytesPerSector;
            (*mediaInfo)
                .DeviceSpecific
                .RemovableDiskInfo
                .NumberMediaSides = 1;
            (*mediaInfo)
                .DeviceSpecific
                .RemovableDiskInfo
                .MediaCharacteristics = MEDIA_READ_WRITE;

            if MediaPresent != 0 {
                (*mediaInfo)
                    .DeviceSpecific
                    .RemovableDiskInfo
                    .MediaCharacteristics |= MEDIA_CURRENTLY_MOUNTED;
            }
            if IsWritable == 0 {
                (*mediaInfo)
                    .DeviceSpecific
                    .RemovableDiskInfo
                    .MediaCharacteristics |= MEDIA_WRITE_PROTECTED;
            }
        }
    }

    // Set information size.
    (*Irp).IoStatus.Information = (core::mem::offset_of!(GET_MEDIA_TYPES, MediaInfo)
        + ((*mediaTypes).MediaInfoCount as usize) * core::mem::size_of::<DEVICE_MEDIA_INFO>()) as _;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DiskDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut ioctlCode: u32 = 0;
    if DeviceObject.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
            b"drivers/storage/class/disk/disk.c\0"
                .as_ptr() as *const i8,
            964i32,
            b"DeviceObject != NULL\0".as_ptr() as *const i8,
        );
        __int2c();
    }
    (*Irp).IoStatus.Information = 0;
    ioctlCode = (*irpStack).Parameters.DeviceIoControl.IoControlCode;
    StorDebugPrint(
        5i32,
        TRACE_FLAG_IOCTL,
        b"DiskDeviceControl: Received IOCTL 0x%X for device %p through IRP %p\n\0".as_ptr()
            as *const i8 as PCCHAR,
        ioctlCode,
        DeviceObject,
        Irp,
    );
    match ioctlCode {
        475348 => {
            status = DiskIoctlGetCacheInformation(DeviceObject, Irp);
        }
        508120 => {
            status = DiskIoctlSetCacheInformation(DeviceObject, Irp);
        }
        475360 => {
            status = DiskIoctlGetCacheSetting(DeviceObject, Irp);
        }
        508132 => {
            status = DiskIoctlSetCacheSetting(DeviceObject, Irp);
        }
        458752 => {
            status = DiskIoctlGetDriveGeometry(DeviceObject, Irp);
        }
        458912 => {
            status = DiskIoctlGetDriveGeometryEx(DeviceObject, Irp);
        }
        458772 => {
            status = DiskIoctlVerify(DeviceObject, Irp);
        }
        475228 => {
            status = DiskIoctlGetLengthInfo(DeviceObject, Irp);
        }
        458788 => {
            status = DiskIoctlIsWritable(DeviceObject, Irp);
        }
        508104 => {
            status = DiskIoctlUpdateDriveSize(DeviceObject, Irp);
        }
        507932 => {
            status = DiskIoctlReassignBlocks(DeviceObject, Irp);
        }
        508068 => {
            status = DiskIoctlReassignBlocksEx(DeviceObject, Irp);
        }
        459779 => {
            status = DiskIoctlSetVerify(DeviceObject, Irp);
        }
        459783 => {
            status = DiskIoctlClearVerify(DeviceObject, Irp);
        }
        2952196 => {
            status = DiskIoctlGetMediaTypesEx(DeviceObject, (Irp).cast());
        }
        2953472 => {
            status = DiskIoctlPredictFailure(DeviceObject, Irp);
        }
        475264 => {
            status = DiskIoctlSmartGetVersion(DeviceObject, Irp);
        }
        508040 => {
            status = DiskIoctlSmartReceiveDriveData(DeviceObject, Irp);
        }
        508036 => {
            status = DiskIoctlSmartSendDriveCommand(DeviceObject, Irp);
        }
        5636096 | 5636100 => {
            status = DiskIoctlGetVolumeDiskExtents(DeviceObject, Irp);
        }
        _ => return ClassDeviceControl(DeviceObject, Irp),
    }
    if !(status >= 0) {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskDeviceControl: IOCTL 0x%X to device %p failed with error 0x%X\n\0".as_ptr()
                as *const i8 as PCCHAR,
            ioctlCode,
            DeviceObject,
            status,
        );
        if (status == STATUS_DEVICE_NOT_READY
            || status == STATUS_IO_TIMEOUT
            || status == STATUS_MEDIA_WRITE_PROTECTED
            || status == STATUS_NO_MEDIA_IN_DEVICE
            || status == STATUS_VERIFY_REQUIRED
            || status == STATUS_UNRECOGNIZED_MEDIA
            || status == STATUS_WRONG_VOLUME)
            && !(*Irp).Tail.Overlay.Thread.is_null()
        {
            IoSetHardErrorOrVerifyDevice(Irp, DeviceObject);
        }
    }
    if status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as i8);
    }
    status
}


#[no_mangle]
pub unsafe extern "system" fn DiskShutdownFlush(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension: *mut _COMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION =
        (*commonExtension).PartitionZeroExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let diskData: *mut _DISK_DATA = (*commonExtension).DriverData as *mut _DISK_DATA;
    let mut irpStack: *mut _IO_STACK_LOCATION;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut srbSize: u32;
    let mut srb: *mut _SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbEx: *mut _STORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: *mut _STOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: *mut _SRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();
    let mut cdb: *mut _CDB = core::ptr::null_mut();
    let mut irql: KIRQL = 0;

    irpStack = IoGetCurrentIrpStackLocation(Irp);

    if (*irpStack).MajorFunction as i32 == IRP_MJ_FLUSH_BUFFERS as i32 {
        if ((*fdoExtension).DeviceFlags as i32 & DEV_POWER_PROTECTED as i32) != 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as i8);
            return STATUS_SUCCESS;
        }

        KeAcquireSpinLock(&raw mut (*diskData).FlushContext.Spinlock as *mut usize, &raw mut irql);

        TracePrint(
            (TRACE_LEVEL_VERBOSE) as i32,
            TRACE_FLAG_SCSI,
            b"DiskShutdownFlush: IRP %p flags = 0x%x\n\0".as_ptr() as *const i8,
            Irp,
            (*irpStack).Flags as i32,
        );

        IoMarkIrpPending(Irp);

        if !(*diskData).FlushContext.CurrIrp.is_null() {
            if !(*diskData).FlushContext.NextIrp.is_null() {
                // #if DBG
                (*diskData).FlushContext.DbgTagCount += 1;
                // #endif

                InsertTailList(
                    &mut (*diskData).FlushContext.NextList,
                    &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
                );

                KeReleaseSpinLock(&raw mut (*diskData).FlushContext.Spinlock as *mut usize, irql);
            } else {
                // #if DBG
                if (*diskData).FlushContext.DbgTagCount < 64 {
                    (*diskData).FlushContext.DbgRefCount
                        [(*diskData).FlushContext.DbgTagCount as usize] += 1;
                }
                (*diskData).FlushContext.DbgSavCount = (*diskData).FlushContext.DbgSavCount
                    .wrapping_add((*diskData).FlushContext.DbgTagCount as u32);
                (*diskData).FlushContext.DbgTagCount = 0;
                // #endif

                (*diskData).FlushContext.NextIrp = Irp;

                // NT_ASSERT(IsListEmpty(&diskData->FlushContext.NextList))
                if IsListEmpty(&mut (*diskData).FlushContext.NextList) == 0 {
                    DbgPrint(
                        b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                        b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                        1232i32,
                        b"IsListEmpty(&diskData->FlushContext.NextList)\0".as_ptr() as *const i8,
                    );
                    __int2c();
                }

                KeReleaseSpinLock(&raw mut (*diskData).FlushContext.Spinlock as *mut usize, irql);

                TracePrint(
                    (TRACE_LEVEL_VERBOSE) as i32,
                    TRACE_FLAG_SCSI,
                    b"DiskShutdownFlush: waiting for event\n\0".as_ptr() as *const i8,
                );

                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!((*diskData).FlushContext.Event) as *mut ::core::ffi::c_void,
                    _KWAIT_REASON_Executive as u32,
                    (0u8) as i8, // KernelMode
                    FALSE as u8,
                    core::ptr::null_mut::<LARGE_INTEGER>(),
                );

                TracePrint(
                    (TRACE_LEVEL_VERBOSE) as i32,
                    TRACE_FLAG_SCSI,
                    b"DiskShutdownFlush: event signal\n\0".as_ptr() as *const i8,
                );

                KeAcquireSpinLock(&raw mut (*diskData).FlushContext.Spinlock as *mut usize, &raw mut irql);

                // NT_ASSERT(IsListEmpty(&diskData->FlushContext.CurrList))
                if IsListEmpty(&mut (*diskData).FlushContext.CurrList) == 0 {
                    DbgPrint(
                        b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                        b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                        1253i32,
                        b"IsListEmpty(&diskData->FlushContext.CurrList)\0".as_ptr() as *const i8,
                    );
                    __int2c();
                }

                while IsListEmpty(&mut (*diskData).FlushContext.NextList) == 0 {
                    let listEntry = RemoveHeadList(&mut (*diskData).FlushContext.NextList);
                    InsertTailList(&mut (*diskData).FlushContext.CurrList, listEntry);
                }

                // ReactOS note: assert removed to match original
                (*diskData).FlushContext.CurrIrp = (*diskData).FlushContext.NextIrp;
                (*diskData).FlushContext.NextIrp = core::ptr::null_mut();

                KeReleaseSpinLock(&raw mut (*diskData).FlushContext.Spinlock as *mut usize, irql);

                DiskFlushDispatch(DeviceObject, &mut (*diskData).FlushContext);
            }
        } else {
            (*diskData).FlushContext.CurrIrp = Irp;

            // NT_ASSERT(IsListEmpty(&diskData->FlushContext.CurrList))
            if IsListEmpty(&mut (*diskData).FlushContext.CurrList) == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                    1280i32,
                    b"IsListEmpty(&diskData->FlushContext.CurrList)\0".as_ptr() as *const i8,
                );
                __int2c();
            }

            // NT_ASSERT(diskData->FlushContext.NextIrp == NULL)
            if !((*diskData).FlushContext.NextIrp).is_null() {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                    1282i32,
                    b"diskData->FlushContext.NextIrp == NULL\0".as_ptr() as *const i8,
                );
                __int2c();
            }

            // NT_ASSERT(IsListEmpty(&diskData->FlushContext.NextList))
            if IsListEmpty(&mut (*diskData).FlushContext.NextList) == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                    1283i32,
                    b"IsListEmpty(&diskData->FlushContext.NextList)\0".as_ptr() as *const i8,
                );
                __int2c();
            }

            KeReleaseSpinLock(&raw mut (*diskData).FlushContext.Spinlock as *mut usize, irql);
            DiskFlushDispatch(DeviceObject, &mut (*diskData).FlushContext);
        }
    } else {
        // Allocate SCSI request block
        if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
            srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
        } else {
            srbSize = core::mem::size_of::<_SCSI_REQUEST_BLOCK>() as u32;
        }

        srb = ExAllocatePoolWithTag((NonPagedPoolNx) as u32, srbSize as usize, DISK_TAG_SRB) as *mut _SCSI_REQUEST_BLOCK;
        if srb.is_null() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as i8);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::write_bytes(srb as *mut u8, 0u8, srbSize as usize);

        if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
            srbEx = srb as *mut _STORAGE_REQUEST_BLOCK;

            // Set up STORAGE_REQUEST_BLOCK fields
            (*srbEx).Length = 8u16; // FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature) literal
            (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK as u8;
            (*srbEx).Signature = SRB_SIGNATURE;
            (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1;
            (*srbEx).SrbLength = srbSize;
            (*srbEx).SrbFunction = (SRB_FUNCTION_EXECUTE_SCSI as u8) as u32;
            (*srbEx).RequestPriority = IoGetIoPriorityHint(Irp) as u16;
            (*srbEx).AddressOffset = core::mem::size_of::<_STORAGE_REQUEST_BLOCK>() as u32;
            (*srbEx).NumSrbExData = 1u32;

            (*srbEx).TimeOutValue = ((*fdoExtension).TimeOutValue as u32).wrapping_mul(4);
            (*srbEx).RequestTag = (SP_UNTAGGED as u8) as u32;
            (*srbEx).RequestAttribute = (SRB_SIMPLE_TAG_REQUEST as u8) as u16;
            (*srbEx).SrbFlags = (*fdoExtension).SrbFlags;

            storAddrBtl8 = (srbEx as *mut u8).offset((*srbEx).AddressOffset as isize) as *mut _STOR_ADDR_BTL8;
            (*storAddrBtl8).Type = ((STOR_ADDRESS_TYPE_BTL8) as u8) as u16;
            (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;

            *(*srbEx).SrbExDataOffset.as_mut_ptr().offset(0isize) =
                (core::mem::size_of::<_STORAGE_REQUEST_BLOCK>() as u32)
                    .wrapping_add(core::mem::size_of::<_STOR_ADDR_BTL8>() as u32);

            if (*(*srbEx).SrbExDataOffset.as_mut_ptr().offset(0isize) as usize)
                .wrapping_add(core::mem::size_of::<_SRBEX_DATA_SCSI_CDB16>() as usize)
                <= (*srbEx).SrbLength as usize
            {
                srbExDataCdb16 = (srbEx as *mut u8).offset(
                    *(*srbEx).SrbExDataOffset.as_mut_ptr().offset(0isize) as isize,
                ) as *mut _SRBEX_DATA_SCSI_CDB16;
                (*srbExDataCdb16).Type = (SrbExDataTypeScsiCdb16) as u32;
                (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
                cdb = core::ptr::addr_of_mut!((*srbExDataCdb16).Cdb) as *mut u8 as *mut _CDB;
            } else {
                // NT_ASSERT(FALSE)
                if false {
                    DbgPrint(
                        b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                        b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                        1365i32,
                        b"FALSE\0".as_ptr() as *const i8,
                    );
                    __int2c();
                }

                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INTERNAL_ERROR;
                ClassReleaseRemoveLock(DeviceObject, Irp);
                ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as i8);
                return STATUS_INTERNAL_ERROR;
            }
        } else {
            (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as u16;
            (*srb).TimeOutValue = ((*fdoExtension).TimeOutValue as u32).wrapping_mul(4);
            (*srb).QueueTag = (SP_UNTAGGED) as u8;
            (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as u8;
            (*srb).SrbFlags = (*fdoExtension).SrbFlags;
            (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
            cdb = core::ptr::addr_of_mut!((*srb).Cdb) as *mut u8 as *mut _CDB;
        }

        // If write cache enabled, send synchronize cache
        if ((*fdoExtension).DeviceFlags as i32 & (DEV_WRITE_CACHE as i32)) != 0 {
            if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
                (*srbExDataCdb16).CdbLength = 10u8;
            } else {
                (*srb).CdbLength = 10u8;
            }
            (*cdb).CDB10.as_mut().OperationCode = SCSIOP_SYNCHRONIZE_CACHE as u8;

            status = ClassSendSrbSynchronous(DeviceObject, srb, core::ptr::null_mut(), 0u32, 1u8);

            TracePrint(
                (TRACE_LEVEL_INFORMATION) as i32,
                TRACE_FLAG_GENERAL,
                b"DiskShutdownFlush: Synchonize cache sent. Status = %lx\n\0".as_ptr() as *const i8,
                status,
            );
        }

        // Unlock if removable media
        if (*DeviceObject).Characteristics & FILE_REMOVABLE_MEDIA != 0 {
            if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
                (*srbEx).SrbStatus = 0u8;
                (*srbExDataCdb16).ScsiStatus = 0u8;
                (*srbExDataCdb16).CdbLength = 6u8;
                (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;
            } else {
                (*srb).SrbStatus = 0u8;
                (*srb).ScsiStatus = 0u8;
                (*srb).CdbLength = 6u8;
                (*srb).TimeOutValue = (*fdoExtension).TimeOutValue;
            }

            (*cdb).MEDIA_REMOVAL.as_mut().OperationCode = SCSIOP_MEDIUM_REMOVAL as u8;
            (*cdb).MEDIA_REMOVAL.as_mut().set_Prevent(0u8); // FALSE

            status = ClassSendSrbSynchronous(DeviceObject, srb, core::ptr::null_mut(), 0u32, 1u8);

            TracePrint(
                (TRACE_LEVEL_INFORMATION) as i32,
                TRACE_FLAG_GENERAL,
                b"DiskShutdownFlush: Unlock device request sent. Status = %lx\n\0".as_ptr() as *const i8,
                status,
            );
        }

        // Set up shutdown SRB
        if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
            (*srbEx).NumSrbExData = 0u32;
            *(*srbEx).SrbExDataOffset.as_mut_ptr().offset(0isize) = 0u32;
            (*srbEx).SrbFunction = (SRB_FUNCTION_SHUTDOWN as u8) as u32;
            (*srbEx).OriginalRequest = Irp as *mut ::core::ffi::c_void;
            (*srbEx).SrbLength = CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE;
            (*srbEx).SrbStatus = 0u8;
        } else {
            (*srb).CdbLength = 0u8;
            (*srb).Function = SRB_FUNCTION_SHUTDOWN as u8;
            (*srb).SrbStatus = 0u8;
            (*srb).OriginalRequest = Irp as *mut ::core::ffi::c_void;
        }

        // Set retry count to zero
        (*irpStack).Parameters.Others.Argument4 = core::ptr::null_mut::<::core::ffi::c_void>();

        // Set up completion routine
        IoSetCompletionRoutine(
            Irp,
            Some(core::mem::transmute::<
                unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, *mut core::ffi::c_void) -> NTSTATUS,
                _,
            >(ClassIoComplete)),
            srb as *mut ::core::ffi::c_void,
            TRUE as u8,
            TRUE as u8,
            TRUE as u8,
        );

        irpStack = IoGetNextIrpStackLocation(Irp);
        (*irpStack).MajorFunction = (IRP_MJ_SCSI) as u8;
        (*irpStack).Parameters.Scsi.Srb = srb as *mut _;

        IoMarkIrpPending(Irp);
        IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
    }

    STATUS_PENDING
}


#[no_mangle]
pub unsafe extern "C" fn DiskFlushDispatch(
    Fdo: PDEVICE_OBJECT,
    FlushContext: *mut DISK_GROUP_CONTEXT,
) {
    let fdoExt: *mut FUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as *mut _;
    let srb: *mut SCSI_REQUEST_BLOCK = (&raw mut (*FlushContext).Srb.Srb).cast();
    let srbEx: *mut STORAGE_REQUEST_BLOCK = (&raw mut (*FlushContext).Srb.SrbEx).cast();
    let mut irpSp: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut storAddrBtl8: *mut STOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: *mut SRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();
    let mut SyncCacheStatus: NTSTATUS = STATUS_SUCCESS;

    if (*(*fdoExt).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        core::ptr::write_bytes(
            srbEx as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<[u8; 184]>(),
        );
        (*srbEx).Length = core::mem::offset_of!(STORAGE_REQUEST_BLOCK, Signature) as USHORT;
        (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK as UCHAR;
        (*srbEx).Signature = SRB_SIGNATURE as ULONG;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1 as ULONG;
        (*srbEx).SrbLength = core::mem::size_of::<[u8; 184]>() as ULONG;
        (*srbEx).RequestPriority = IoGetIoPriorityHint((*FlushContext).CurrIrp) as USHORT;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as ULONG;
        (*srbEx).TimeOutValue = ((*fdoExt).TimeOutValue as ULONG).wrapping_mul(4) as ULONG;
        (*srbEx).RequestTag = SP_UNTAGGED as ULONG;
        (*srbEx).RequestAttribute = SRB_SIMPLE_TAG_REQUEST as USHORT;
        (*srbEx).SrbFlags = (*fdoExt).SrbFlags;

        storAddrBtl8 = (srbEx as *mut u8)
            .add((*srbEx).AddressOffset as usize) as *mut STOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8 as USHORT;
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH as ULONG;
    } else {
        core::ptr::write_bytes(
            srb as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
        );
        (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as USHORT;
        (*srb).TimeOutValue = ((*fdoExt).TimeOutValue as ULONG).wrapping_mul(4) as ULONG;
        (*srb).QueueTag = (SP_UNTAGGED) as u8;
        (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as UCHAR;
        (*srb).SrbFlags = (*fdoExt).SrbFlags;
    }

    if ((*fdoExt).DeviceFlags & (DEV_WRITE_CACHE) as u16) != 0 {
        if (*(*fdoExt).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
            (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI as ULONG;
            (*srbEx).NumSrbExData = 1;
            (*srbEx).SrbExDataOffset[0] = (core::mem::size_of::<STORAGE_REQUEST_BLOCK>()
                .wrapping_add(core::mem::size_of::<STOR_ADDR_BTL8>()))
                as ULONG;
            if ((*srbEx).SrbExDataOffset[0] as usize)
                .wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>())
                <= (*srbEx).SrbLength as usize
            {
                srbExDataCdb16 = (srbEx as *mut u8)
                    .add((*srbEx).SrbExDataOffset[0] as usize) as *mut SRBEX_DATA_SCSI_CDB16;
                (*srbExDataCdb16).Type = (SrbExDataTypeScsiCdb16) as u32;
                (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH as ULONG;
                (*srbExDataCdb16).CdbLength = 10;
                (*srbExDataCdb16).Cdb[0] = SCSIOP_SYNCHRONIZE_CACHE as UCHAR;
            } else {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"drivers/storage/class/disk/disk.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    1627i32,
                    b"FALSE\0" as *const u8 as *const ::core::ffi::c_char,
                );
                __int2c();
                return;
            }
        } else {
            (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
            (*srb).CdbLength = 10;
            (*srb).Cdb[0] = SCSIOP_SYNCHRONIZE_CACHE as UCHAR;
        }
        TracePrint(
            (TRACE_LEVEL_VERBOSE) as i32,
            TRACE_FLAG_SCSI,
            b"DiskFlushDispatch: sending sync cache\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        SyncCacheStatus = ClassSendSrbSynchronous(Fdo, srb, core::ptr::null_mut(), 0 as ULONG, 1u8);
    }

    if (*(*fdoExt).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        (*srbEx).SrbFunction = SRB_FUNCTION_FLUSH as ULONG;
        (*srbEx).NumSrbExData = 0;
        (*srbEx).SrbExDataOffset[0] = 0;
        (*srbEx).OriginalRequest = (*FlushContext).CurrIrp as *mut ::core::ffi::c_void;
        (*srbEx).SrbStatus = 0;
        (*srbEx).SrbFlags |= (SRB_CLASS_FLAGS_PERSISTANT as USHORT) as u32;
    } else {
        (*srb).Function = SRB_FUNCTION_FLUSH as UCHAR;
        (*srb).CdbLength = 0;
        (*srb).OriginalRequest = (*FlushContext).CurrIrp as *mut ::core::ffi::c_void;
        (*srb).SrbStatus = 0;
        (*srb).ScsiStatus = 0;
        (*srb).SrbFlags |= (SRB_CLASS_FLAGS_PERSISTANT as USHORT) as u32;
    }

    irpSp = IoGetCurrentIrpStackLocation((*FlushContext).CurrIrp);
    (*irpSp).Parameters.Others.Argument4 = core::ptr::null_mut::<::core::ffi::c_void>();

    irpSp = IoGetNextIrpStackLocation((*FlushContext).CurrIrp);
    (*irpSp).MajorFunction = IRP_MJ_SCSI as UCHAR;
    (*irpSp).Parameters.Scsi.Srb = srb as *mut _;

    IoSetCompletionRoutine(
        (*FlushContext).CurrIrp,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS,
            _,
        >(DiskFlushComplete)),
        SyncCacheStatus as usize as PVOID,
        1u8,
        1u8,
        1u8,
    );

    TracePrint(
        (TRACE_LEVEL_VERBOSE) as i32,
        TRACE_FLAG_SCSI,
        b"DiskFlushDispatch: sending srb flush on irp %p\n\0" as *const u8 as *const ::core::ffi::c_char,
        (*FlushContext).CurrIrp,
    );

    IoCallDriver(
        (*(fdoExt as *mut COMMON_DEVICE_EXTENSION)).LowerDeviceObject,
        (*FlushContext).CurrIrp,
    );
}







#[no_mangle]
pub unsafe extern "system" fn DiskFlushComplete(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut FlushContext: PDISK_GROUP_CONTEXT = core::ptr::null_mut::<_DISK_GROUP_CONTEXT>();
    let mut status: NTSTATUS = 0;
    let mut fdoExt: PFUNCTIONAL_DEVICE_EXTENSION =
        core::ptr::null_mut::<_FUNCTIONAL_DEVICE_EXTENSION>();
    let mut diskData: PDISK_DATA = core::ptr::null_mut::<_DISK_DATA>();
    let mut SyncCacheStatus: NTSTATUS = Context as usize as NTSTATUS;

    StorDebugPrint(
        TRACE_LEVEL_VERBOSE as i32,
        TRACE_FLAG_GENERAL as u32,
        b"DiskFlushComplete: %p %p\n\0".as_ptr() as *const i8,
        Fdo,
        Irp,
    );

    fdoExt = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    diskData = (*fdoExt).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;

    if diskData.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
            b"drivers/storage/class/disk/disk.c\0"
                .as_ptr() as *const i8,
            1744 as i32,
            b"diskData != NULL\0".as_ptr() as *const i8,
        );
        __int2c();
    } else {
    }

    FlushContext = &raw mut (*diskData).FlushContext as PDISK_GROUP_CONTEXT;

    if !(Irp == (*FlushContext).CurrIrp) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
            b"drivers/storage/class/disk/disk.c\0"
                .as_ptr() as *const i8,
            1752 as i32,
            b"Irp == FlushContext->CurrIrp\0".as_ptr() as *const i8,
        );
        __int2c();
    } else {
    }

    StorDebugPrint(
        TRACE_LEVEL_VERBOSE as i32,
        TRACE_FLAG_SCSI as u32,
        b"DiskFlushComplete: completing irp %p\n\0".as_ptr() as *const i8,
        Irp,
    );

    status = ClassIoComplete(Fdo, Irp, &raw mut (*FlushContext).Srb.Srb as *mut core::ffi::c_void);

    if !(status != 0xc0000016u32 as NTSTATUS) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
            b"drivers/storage/class/disk/disk.c\0"
                .as_ptr() as *const i8,
            1760 as i32,
            b"status != STATUS_MORE_PROCESSING_REQUIRED\0".as_ptr() as *const i8,
        );
        __int2c();
    } else {
    }

    if status >= 0 && !(SyncCacheStatus >= 0) {
        status = SyncCacheStatus;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    }

    while IsListEmpty(&raw mut (*FlushContext).CurrList) == 0 {
        let listEntry: PLIST_ENTRY = RemoveHeadList(&raw mut (*FlushContext).CurrList);
        let tempIrp: PIRP = (listEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as ULONG_PTR,
        ) as PIRP;

        InitializeListHead(&raw mut (*tempIrp).Tail.Overlay.__bindgen_anon_2.ListEntry);
        (*tempIrp).IoStatus = (*Irp).IoStatus;

        ClassReleaseRemoveLock(Fdo, tempIrp);
        ClassCompleteRequest(Fdo, tempIrp, IO_NO_INCREMENT as i8);
    }

    KeSetEvent(
        (&raw mut (*FlushContext).Event).cast(),
        IO_NO_INCREMENT as KPRIORITY,
        0u8, // FALSE
    );

    StorDebugPrint(
        TRACE_LEVEL_VERBOSE as i32,
        TRACE_FLAG_SCSI as u32,
        b"DiskFlushComplete: irp %p status = 0x%x\n\0".as_ptr() as *const i8,
        Irp,
        status,
    );

    status
}






#[no_mangle]
pub unsafe extern "C" fn DiskModeSelect(
    Fdo: PDEVICE_OBJECT,
    ModeSelectBuffer: PCHAR,
    Length: ULONG,
    SavePage: BOOLEAN,
) -> NTSTATUS {
    // Forward declarations for symbols not in the provided import list
    // Constants needed from the original C / anchor
    const APC_LEVEL: KIRQL = 1;
    const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
    const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    // disk.h:75: `#define NON_PAGED_POOL_NX_CACHE_ALIGNED NonPagedPoolCacheAligned` under
    // `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
    // Windows value (0x200/0x204) is not what any TU here compiles.
    const NON_PAGED_POOL_NX_CACHE_ALIGNED: u32 = 4;
                                                // NonPagedPoolCacheAligned
    const DISK_TAG_MODE_DATA: u32 = 0x4D446353;         // 'MODE' placeholder
    const SRB_TYPE_STORAGE_REQUEST_BLOCK: u32 = 1;     // srb.h:664
    // ddk/srb.h:108 == 0x28. 0xF0 was an INVENTED value labelled "placeholder";
    // it is not any SRB_FUNCTION_* and compiled perfectly happily.
    const SRB_FUNCTION_STORAGE_REQUEST_BLOCK: u8 = 0x28;
    const SRB_SIGNATURE: u32 = 0x53524258; // 'SRBX', ddk/srb.h:616
    const STORAGE_REQUEST_BLOCK_VERSION_1: u32 = 1;
    const SRB_FUNCTION_EXECUTE_SCSI: u32 = 0x00;        // placeholder
    const STOR_ADDRESS_TYPE_BTL8: u16 = 1;              // scsi.h:3571
    const STOR_ADDR_BTL8_ADDRESS_LENGTH: u32 = 4;       // scsi.h:3574
    const SRB_EX_DATA_TYPE_SCSI_CDB16: u16 = 0x40;      // srb.h:459 SrbExDataTypeScsiCdb16
    // srb.h:489 `((20 * sizeof(UCHAR)) + sizeof(ULONG) + sizeof(PVOID))` -- POINTER
    // WIDTH dependent, so a frozen 32 is right on 64-bit and 4 bytes too long on
    // i386, where it would describe an SRB extended-data block past its own end.
    const SRBEX_DATA_SCSI_CDB16_LENGTH: u32 =
        (20 + core::mem::size_of::<u32>() + core::mem::size_of::<*mut core::ffi::c_void>()) as u32;
    const SCSIOP_MODE_SELECT: u8 = 0x15;                // typical
    const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016u32 as i32;
    const SRB_STATUS_DATA_OVERRUN: u8 = 0x12;
    const SRB_STATUS_AUTOSENSE_VALID: u8 = 0x80;
    const SRB_STATUS_QUEUE_FROZEN: u8 = 0x40;
    const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5u32 as i32;

    // Local variables, preserving C names and pointer types
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut cdb: PCDB = core::ptr::null_mut();
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut retries: ULONG = 1;
    let mut length2: ULONG = 0;
    let mut status: NTSTATUS = 0;
    let mut buffer: PULONG = core::ptr::null_mut();
    let mut blockDescriptor: PMODE_PARAMETER_BLOCK = core::ptr::null_mut();
    let mut srbExBuffer: [UCHAR; 184] = [0u8; 184];
    let srbEx: PSTORAGE_REQUEST_BLOCK =
        &raw mut srbExBuffer as *mut UCHAR as PSTORAGE_REQUEST_BLOCK;
    let mut storAddrBtl8: PSTOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: PSRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();
    let mut srbPtr: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();

    // PAGED_CODE: check IRQL (from anchor)
    if KeGetCurrentIrql() as u8 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                1846,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    // Check whether block length is available
    if (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskModeSelect: Block length is not available. Unable to send mode select\n\0"
                .as_ptr() as PCCHAR,
        );
        // NT_ASSERT(fdoExtension->DiskGeometry.BytesPerSector != 0)
        if !((*fdoExtension).DiskGeometry.BytesPerSector != 0) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                1855,
                b"fdoExtension->DiskGeometry.BytesPerSector != 0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_PARAMETER;
    }

    length2 = (Length as usize)
        .wrapping_add(core::mem::size_of::<MODE_PARAMETER_HEADER>())
        .wrapping_add(core::mem::size_of::<MODE_PARAMETER_BLOCK>()) as ULONG;

    buffer = ExAllocatePoolWithTag(
        NON_PAGED_POOL_NX_CACHE_ALIGNED,
        length2 as SIZE_T,
        DISK_TAG_MODE_DATA,
    ) as PULONG;
    if buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(buffer as *mut u8, 0, length2 as usize);

    (*(buffer as PMODE_PARAMETER_HEADER)).BlockDescriptorLength =
        core::mem::size_of::<MODE_PARAMETER_BLOCK>() as UCHAR;

    blockDescriptor = (buffer.offset(1)) as PMODE_PARAMETER_BLOCK;

    (*blockDescriptor).BlockLength[2] =
        ((*fdoExtension).DiskGeometry.BytesPerSector >> 16) as UCHAR;
    (*blockDescriptor).BlockLength[1] =
        ((*fdoExtension).DiskGeometry.BytesPerSector >> 8) as UCHAR;
    (*blockDescriptor).BlockLength[0] =
        (*fdoExtension).DiskGeometry.BytesPerSector as UCHAR;

    core::ptr::copy_nonoverlapping(
        ModeSelectBuffer as *const u8,
        (buffer.offset(3)) as *mut u8,
        Length as usize,
    );

    if (*(*fdoExtension).AdapterDescriptor).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        // Set up STORAGE_REQUEST_BLOCK fields
        (*srbEx).Length = 8u16; // FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature) = 8
        (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK as UCHAR;
        (*srbEx).Signature = SRB_SIGNATURE as ULONG;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1 as ULONG;
        (*srbEx).SrbLength = core::mem::size_of::<[UCHAR; 184]>() as ULONG;
        (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI as ULONG;
        (*srbEx).RequestPriority = IoPriorityNormal as USHORT;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as ULONG;
        (*srbEx).NumSrbExData = 1;
        (*srbEx).TimeOutValue = ((*fdoExtension).TimeOutValue as ULONG) * 2;

        // Set up address fields
        storAddrBtl8 =
            (srbEx as PUCHAR).offset((*srbEx).AddressOffset as isize) as PSTOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8 as USHORT;
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH as ULONG;

        // Set up SCSI SRB extended data fields
        *(&raw mut (*srbEx).SrbExDataOffset as *mut ULONG).offset(0) =
            (core::mem::size_of::<STORAGE_REQUEST_BLOCK>() + core::mem::size_of::<STOR_ADDR_BTL8>())
                as ULONG;
        if *(&raw mut (*srbEx).SrbExDataOffset as *mut ULONG).offset(0) as usize
            + core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() <= (*srbEx).SrbLength as usize
        {
            srbExDataCdb16 = (srbEx as PUCHAR)
                .offset(*(&raw mut (*srbEx).SrbExDataOffset as *mut ULONG).offset(0) as isize)
                as PSRBEX_DATA_SCSI_CDB16;
            (*srbExDataCdb16).Type = (SRB_EX_DATA_TYPE_SCSI_CDB16) as u32;
            (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
            (*srbExDataCdb16).CdbLength = 6;
            cdb = &raw mut (*srbExDataCdb16).Cdb as *mut UCHAR as PCDB;
        } else {
            // Should not happen
            if 0 == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                    b"drivers/storage/class/disk/disk.c\0"
                        as *const u8 as *const i8,
                    1945,
                    b"FALSE\0" as *const u8 as *const i8,
                );
                __int2c();
            }
            ExFreePool(buffer as PVOID);
            buffer = core::ptr::null_mut();
            TracePrint(
                (TRACE_LEVEL_ERROR) as i32,
                TRACE_FLAG_IOCTL,
                b"DiskModeSelect: Insufficient extended SRB size\n\0".as_ptr() as PCCHAR,
            );
            return STATUS_INTERNAL_ERROR;
        }
        srbPtr = srbEx as PSCSI_REQUEST_BLOCK;
    } else {
        srb.CdbLength = 6;
        cdb = &raw mut srb.Cdb as *mut UCHAR as PCDB;
        srb.TimeOutValue = ((*fdoExtension).TimeOutValue) * 2;
        srbPtr = &raw mut srb as PSCSI_REQUEST_BLOCK;
    }

    (*cdb).MODE_SELECT.as_mut().OperationCode = SCSIOP_MODE_SELECT as UCHAR;
    (*cdb).MODE_SELECT.as_mut().set_SPBit(SavePage as UCHAR as UCHAR);
    (*cdb).MODE_SELECT.as_mut().set_PFBit(1 as UCHAR as UCHAR);
    (*cdb).MODE_SELECT.as_mut().ParameterListLength = length2 as UCHAR;

    loop {
        status = ClassSendSrbSynchronous(Fdo, srbPtr, buffer as PVOID, length2, 1);
        if status == STATUS_VERIFY_REQUIRED {
            let fresh2 = retries;
            retries = retries.wrapping_sub(1);
            if !(fresh2 != 0) {
                break;
            }
        } else {
            if (*srbPtr).SrbStatus as u8
                & !(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)
                == SRB_STATUS_DATA_OVERRUN
            {
                status = STATUS_SUCCESS;
            }
            break;
        }
    }

    ExFreePool(buffer as PVOID);
    buffer = core::ptr::null_mut();

    status
}


#[no_mangle]
pub unsafe extern "system" fn DisableWriteCache(
    Fdo: PDEVICE_OBJECT,
    Context: PVOID,
) {

    const APC_LEVEL: i32 = 1;
    const TRUE: BOOLEAN = 1;
    const FALSE: BOOLEAN = 0;

    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut cacheInfo: DISK_CACHE_INFORMATION = core::mem::zeroed();
    let mut status: NTSTATUS = 0;
    let WorkItem: PIO_WORKITEM = Context as PIO_WORKITEM;

    // PAGED_CODE expanded (anchor lines 2022-2038)
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                2023i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    // NT_ASSERT(WorkItem != NULL) expanded (anchor lines 2039-2049)
    if WorkItem.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
            b"drivers/storage/class/disk/disk.c\0"
                as *const u8 as *const i8,
            2025i32,
            b"WorkItem != NULL\0" as *const u8 as *const i8,
        );
        __int2c();
    } else {
        // _Analysis_assume_ has no runtime effect
    }

    status = DiskGetCacheInformation(fdoExtension, &raw mut cacheInfo);

    if status >= 0 && cacheInfo.WriteCacheEnabled == TRUE {
        cacheInfo.WriteCacheEnabled = FALSE;
        DiskSetCacheInformation(fdoExtension, &raw mut cacheInfo);
    }

    IoFreeWorkItem(WorkItem);
}
#[no_mangle]
pub unsafe extern "system" fn DiskIoctlVerifyThread(
    Fdo: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let mut WorkContext: PDISK_VERIFY_WORKITEM_CONTEXT = Context as PDISK_VERIFY_WORKITEM_CONTEXT;
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut DiskData: PDISK_DATA =
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;
    let mut verifyInfo: PVERIFY_INFORMATION = core::ptr::null_mut();
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut Cdb: PCDB = core::ptr::null_mut();
    let mut byteOffset: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut sectorOffset: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut sectorCount: ULONG = 0;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut srbEx: PSTORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: PSTOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: PSRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        // this is DbgPrint and __int2c; use pointers to byte strings
        let msg = b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCCHAR;
        DbgPrint(msg, KeGetCurrentIrql() as i32);
        let assert_msg = b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR;
        let file = b"disk.c\0".as_ptr() as PCCHAR;
        let line = 2066i32;
        let condition = b"0\0".as_ptr() as PCCHAR;
        DbgPrint(assert_msg, file, line, condition);
        __int2c(); // breakpoint
    }

    // NT_ASSERT(WorkContext != NULL)
    if WorkContext.is_null() {
        let assert_msg = b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR;
        let file = b"disk.c\0".as_ptr() as PCCHAR;
        let line = 2068i32;
        let condition = b"WorkContext != NULL\0".as_ptr() as PCCHAR;
        DbgPrint(assert_msg, file, line, condition);
        __int2c();
    }

    Srb = (*WorkContext).Srb;
    Irp = (*WorkContext).Irp;
    verifyInfo = (*Irp).AssociatedIrp.SystemBuffer as PVERIFY_INFORMATION;

    IoFreeWorkItem((*WorkContext).WorkItem);

    TracePrint(
        4, // TRACE_LEVEL_INFORMATION
        TRACE_FLAG_IOCTL,
        b"DiskIoctlVerifyThread: Spliting up the request\n\0".as_ptr() as PCCHAR,
    );

    byteOffset.QuadPart = (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().StartingOffset.QuadPart
        + (*verifyInfo).StartingOffset.QuadPart;
    sectorOffset.QuadPart = byteOffset.QuadPart >> (*FdoExtension).SectorShift as i32;
    sectorCount = (*verifyInfo).Length >> (*FdoExtension).SectorShift as i32;

    KeWaitForSingleObject(
        &raw mut (*DiskData).VerifyMutex as PVOID,
        _KWAIT_REASON_Executive,
        (0i32) as i8,
        0, // FALSE
        core::ptr::null_mut::<LARGE_INTEGER>(),
    );

    if (*(*FdoExtension).AdapterDescriptor).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        // zero out SRB + headers
        core::ptr::write_bytes(
            Srb as *mut u8,
            0u8,
            (core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as usize)
                .wrapping_add(core::mem::size_of::<STOR_ADDR_BTL8>() as usize)
                .wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as usize),
        );
        srbEx = Srb as PSTORAGE_REQUEST_BLOCK;

        (*srbEx).Length = 8u16; // FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature) = 8
        (*srbEx).Function = (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8;
        (*srbEx).Signature = SRB_SIGNATURE;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1;
        (*srbEx).SrbLength = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
        (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        (*srbEx).RequestPriority = IoGetIoPriorityHint(Irp) as u16;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
        (*srbEx).NumSrbExData = 1u32;

        storAddrBtl8 = (srbEx as *mut u8)
            .offset((*srbEx).AddressOffset as isize) as PSTOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8 as u16; // assume constant
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH as u32;

        *(&raw mut (*srbEx).SrbExDataOffset as *mut u32).offset(0) =
            (core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as usize)
                .wrapping_add(core::mem::size_of::<STOR_ADDR_BTL8>() as usize) as u32;

        if (*(&raw mut (*srbEx).SrbExDataOffset as *mut u32).offset(0) as usize)
            .wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as usize)
            <= (*srbEx).SrbLength as usize
        {
            srbExDataCdb16 = (srbEx as *mut u8).offset(
                *(&raw mut (*srbEx).SrbExDataOffset as *mut u32).offset(0) as isize,
            ) as PSRBEX_DATA_SCSI_CDB16;
            (*srbExDataCdb16).Type = (SrbExDataTypeScsiCdb16) as u32;
            (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH as u32;
            Cdb = &raw mut (*srbExDataCdb16).Cdb as *mut u8 as PCDB;

            if (*FdoExtension).DeviceFlags as u32 & DEV_USE_16BYTE_CDB != 0 {
                (*srbExDataCdb16).CdbLength = 16u8;
                (&mut (*Cdb).CDB16).as_mut().OperationCode = (SCSIOP_VERIFY16) as u8;
            } else {
                (*srbExDataCdb16).CdbLength = 10u8;
                (&mut (*Cdb).CDB10).as_mut().OperationCode = (SCSIOP_VERIFY) as u8;
            }
        } else {
            // NT_ASSERT(FALSE)
            if 0i32 == 0 {
                let assert_msg = b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR;
                let file = b"disk.c\0".as_ptr() as PCCHAR;
                let line = 2164i32;
                let condition = b"FALSE\0".as_ptr() as PCCHAR;
                DbgPrint(assert_msg, file, line, condition);
                __int2c();
            };
            if !Srb.is_null() {
                ExFreePool(Srb as PVOID);
                Srb = core::ptr::null_mut();
            }
            if !WorkContext.is_null() {
                ExFreePool(WorkContext as PVOID);
                WorkContext = core::ptr::null_mut();
            }
            status = STATUS_INTERNAL_ERROR;
        }
    } else {
        core::ptr::write_bytes(
            Srb as *mut u8,
            0u8,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>() as usize,
        );
        (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
        (*Srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        Cdb = &raw mut (*Srb).Cdb as *mut u8 as PCDB;

        if (*FdoExtension).DeviceFlags as u32 & DEV_USE_16BYTE_CDB != 0 {
            (*Srb).CdbLength = 16u8;
            (&mut (*Cdb).CDB16).as_mut().OperationCode = (SCSIOP_VERIFY16) as u8;
        } else {
            (*Srb).CdbLength = 10u8;
            (&mut (*Cdb).CDB10).as_mut().OperationCode = (SCSIOP_VERIFY) as u8;
        }
    }

    while status >= 0 && sectorCount != 0 {
        let numSectors: u16 = (if sectorCount < MAX_SECTORS_PER_VERIFY {
            sectorCount
        } else {
            MAX_SECTORS_PER_VERIFY
        }) as u16;

        if (*(*FdoExtension).AdapterDescriptor).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
            (*srbEx).SrbStatus = 0u8;
            (*srbExDataCdb16).ScsiStatus = 0u8;
            (*srbEx).TimeOutValue = ((numSectors as i32 + 0x7Fi32 >> 7i32) as u32)
                .wrapping_mul((*FdoExtension).TimeOutValue);
        } else {
            (*Srb).SrbStatus = 0u8;
            (*Srb).ScsiStatus = 0u8;
            (*Srb).TimeOutValue = ((numSectors as i32 + 0x7Fi32 >> 7i32) as u32)
                .wrapping_mul((*FdoExtension).TimeOutValue);
        }

        if (*FdoExtension).DeviceFlags as u32 & DEV_USE_16BYTE_CDB != 0 {
            // REVERSE_BYTES_QUAD(&Cdb->CDB16.LogicalBlock, &sectorOffset)
            let _val1: *mut u64 = &raw mut (&mut (*Cdb).CDB16).as_mut().LogicalBlock as *mut u64;
            let _val2: *const u64 = &raw const sectorOffset as *const u64;
            // byte-reverse a 64-bit quantity (big-endian storage)
            let inverted = _val2.read_unaligned().swap_bytes();
            _val1.write_unaligned(inverted);

            // REVERSE_BYTES_SHORT(&Cdb->CDB16.TransferLength[2], &numSectors)
            let dest2: *mut u16 = (&raw mut (&mut (*Cdb).CDB16).as_mut().TransferLength as *mut u8)
                .offset(2) as *mut u16;
            let src2: *const u16 = &raw const numSectors as *const u16;
            let val2 = src2.read_unaligned().swap_bytes();
            dest2.write_unaligned(val2);
        } else {
            // Manual big-endian placement for 10-byte CDB
            let sector_bytes: *const u32 = &raw const sectorOffset as *const u32;
            let raw_sector: u32 = sector_bytes.read_unaligned();
            (&mut (*Cdb).CDB10).as_mut().LogicalBlockByte0 = (raw_sector >> 24) as u8;
            (&mut (*Cdb).CDB10).as_mut().LogicalBlockByte1 = (raw_sector >> 16) as u8;
            (&mut (*Cdb).CDB10).as_mut().LogicalBlockByte2 = (raw_sector >> 8) as u8;
            (&mut (*Cdb).CDB10).as_mut().LogicalBlockByte3 = raw_sector as u8;

            let num_bytes: *const u16 = &raw const numSectors as *const u16;
            let raw_num: u16 = num_bytes.read_unaligned();
            (&mut (*Cdb).CDB10).as_mut().TransferBlocksMsb = (raw_num >> 8) as u8;
            (&mut (*Cdb).CDB10).as_mut().TransferBlocksLsb = raw_num as u8;
        }

        status = ClassSendSrbSynchronous(
            Fdo,
            Srb,
            core::ptr::null_mut(),
            0u32,
            0, // FALSE
        );

        // NT_ASSERT(status != STATUS_NONEXISTENT_SECTOR)
        if !(status != STATUS_NONEXISTENT_SECTOR) {
            let assert_msg = b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR;
            let file = b"disk.c\0".as_ptr() as PCCHAR;
            let line = 2255i32;
            let condition = b"status != STATUS_NONEXISTENT_SECTOR\0".as_ptr() as PCCHAR;
            DbgPrint(assert_msg, file, line, condition);
            __int2c();
        }

        sectorCount = sectorCount.wrapping_sub(numSectors as u32);
        sectorOffset.QuadPart += numSectors as i64;
    }

    KeReleaseMutex(&raw mut (*DiskData).VerifyMutex, 0);

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    (*Irp).IoStatus.Information = 0;

    ClassReleaseRemoveLock(Fdo, Irp);
    ClassCompleteRequest(Fdo, Irp, IO_NO_INCREMENT);

    if !Srb.is_null() {
        ExFreePool(Srb as PVOID);
        Srb = core::ptr::null_mut();
    }
    if !WorkContext.is_null() {
        ExFreePool(WorkContext as PVOID);
        WorkContext = core::ptr::null_mut();
    }
}





#[no_mangle]
pub unsafe extern "system" fn DiskFdoProcessError(
    Fdo: PDEVICE_OBJECT,
    Srb: *mut SCSI_REQUEST_BLOCK,
    Status: *mut NTSTATUS,
    Retry: *mut BOOLEAN,
) {
    let fdoExtension = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let mut srbEx = core::ptr::null_mut::<STORAGE_REQUEST_BLOCK>();
    let mut cdb: *mut CDB = core::ptr::null_mut();
    let mut scsiStatus: u8 = 0;
    let mut senseBufferLength: u8 = 0;
    let mut senseBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut noOp: CDB = core::mem::zeroed();

    if (*Srb).Function as i32 == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as i32 {
        srbEx = Srb as *mut _;
        if (*srbEx).SrbFunction == SRB_FUNCTION_EXECUTE_SCSI as u32
            && (*srbEx).NumSrbExData > 0
        {
            cdb = GetSrbScsiData(
                srbEx,
                core::ptr::null_mut::<u8>(),
                core::ptr::null_mut::<u32>(),
                &mut scsiStatus,
                &mut senseBuffer,
                &mut senseBufferLength,
            );
            if cdb.is_null() {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                    b"drivers/storage/class/disk/disk.c\0"
                        as *const u8 as *const i8,
                    2332i32,
                    b"cdb != NULL\0" as *const u8 as *const i8,
                );
                __int2c();
            }
            if senseBuffer.is_null() {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                    b"drivers/storage/class/disk/disk.c\0"
                        as *const u8 as *const i8,
                    2333i32,
                    b"senseBuffer != NULL\0" as *const u8 as *const i8,
                );
                __int2c();
            }
        }
        if cdb.is_null() {
            cdb = &raw mut noOp as *mut CDB;
        }
    } else {
        cdb = &raw mut (*Srb).Cdb as *mut u8 as *mut CDB;
        scsiStatus = (*Srb).ScsiStatus;
        senseBufferLength = (*Srb).SenseInfoBufferLength;
        senseBuffer = (*Srb).SenseInfoBuffer;
    }

    if *Status == STATUS_DATA_OVERRUN
        && !cdb.is_null()
        && ((*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_READ6) as i32
            || (*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_READ) as i32
            || (*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_READ12) as i32
            || (*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_READ16) as i32
            || ((*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_WRITE6) as i32
                || (*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_WRITE) as i32
                || (*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_WRITE12) as i32
                || (*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_WRITE16) as i32))
    {
        *Retry = 1u8;
        (*fdoExtension).ErrorCount = (*fdoExtension).ErrorCount.wrapping_add(1);
    } else if (*Srb).SrbStatus as i32
        & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32
        == (SRB_STATUS_ERROR) as i32
        && scsiStatus as i32 == (SCSISTAT_BUSY) as i32
    {
        ResetBus(Fdo);
        (*fdoExtension).ErrorCount = (*fdoExtension).ErrorCount.wrapping_add(1);
    } else {
        let mut invalidatePartitionTable: BOOLEAN = 0u8;
        if (*Srb).SrbStatus as i32 & (SRB_STATUS_AUTOSENSE_VALID) as i32 != 0
            && !senseBuffer.is_null()
            && !cdb.is_null()
        {
            let mut validSense: BOOLEAN = 0u8;
            let mut senseKey: u8 = 0;
            let mut asc: u8 = 0;
            let mut ascq: u8 = 0;
            validSense = ScsiGetSenseKeyAndCodes(
                senseBuffer,
                senseBufferLength,
                (SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED) as u8,
                &raw mut senseKey,
                &raw mut asc,
                &raw mut ascq,
            );
            if validSense != 0 {
                match senseKey as i32 {
                    __mv if __mv == (SCSI_SENSE_ILLEGAL_REQUEST) as i32 => match asc as i32 {
                        __mv if __mv == (SCSI_ADSENSE_INVALID_CDB) as i32 => {
                            if ((*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_WRITE) as i32
                                || (*cdb).CDB10.as_ref().OperationCode as i32 == (SCSIOP_WRITE16) as i32)
                                && (*cdb).CDB10.as_ref().ForceUnitAccess() as i32 != 0
                            {
                                let diskData = (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as *mut DISK_DATA;
                                if (*diskData).WriteCacheOverride as i32 == DiskWriteCacheEnable as i32 {
                                    let mut logEntry: *mut IO_ERROR_LOG_PACKET = core::ptr::null_mut();
                                    logEntry = IoAllocateErrorLogEntry(
                                        ((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject) as *mut core::ffi::c_void,
                                        (core::mem::size_of::<IO_ERROR_LOG_PACKET>()
                                            + 4 * core::mem::size_of::<u32>()) as u8,
                                    ) as *mut _;
                                    if !logEntry.is_null() {
                                        (*logEntry).FinalStatus = *Status;
                                        (*logEntry).ErrorCode = IO_WARNING_WRITE_FUA_PROBLEM;
                                        (*logEntry).SequenceNumber = 0u32;
                                        (*logEntry).MajorFunctionCode = IRP_MJ_SCSI as u8;
                                        (*logEntry).IoControlCode = 0u32;
                                        (*logEntry).RetryCount = 0u8;
                                        (*logEntry).UniqueErrorValue = 0u32;
                                        (*logEntry).DumpDataSize = (4 * core::mem::size_of::<u32>()) as u16;
                                        *((*logEntry).DumpData.as_mut_ptr().add(0) as *mut u32) =
                                            (*diskData).ScsiAddress.PortNumber as u32;
                                        *((*logEntry).DumpData.as_mut_ptr().add(1) as *mut u32) =
                                            (*diskData).ScsiAddress.PathId as u32;
                                        *((*logEntry).DumpData.as_mut_ptr().add(2) as *mut u32) =
                                            (*diskData).ScsiAddress.TargetId as u32;
                                        *((*logEntry).DumpData.as_mut_ptr().add(3) as *mut u32) =
                                            (*diskData).ScsiAddress.Lun as u32;
                                        IoWriteErrorLogEntry(logEntry as *mut core::ffi::c_void);
                                    }
                                } else {
                                    let workItem = IoAllocateWorkItem(Fdo);
                                    if !workItem.is_null() {
                                        IoQueueWorkItem(
                                            workItem,
                                            Some(core::mem::transmute::<
                                                unsafe extern "system" fn(PDEVICE_OBJECT, PVOID),
                                                _,
                                            >(DisableWriteCache)),
                                            CriticalWorkQueue,
                                            workItem as *mut core::ffi::c_void,
                                        );
                                    }
                                }
                                (*fdoExtension).ScanForSpecialFlags |= 0x80u32;
                                if (*fdoExtension).DeviceFlags as i32 & 0x1 != 0
                                    && (*fdoExtension).DeviceFlags as i32 & 0x10 == 0
                                    && (*fdoExtension).ScanForSpecialFlags as u32 & 0x80 == 0
                                {
                                    (*fdoExtension).CdbForceUnitAccess = 1u8;
                                } else {
                                    (*fdoExtension).CdbForceUnitAccess = 0u8;
                                }
                                (*cdb).CDB10.as_mut().set_ForceUnitAccess(0u8);
                                *Retry = 1u8;
                            } else if (*cdb).CDB6FORMAT.as_ref().OperationCode as i32 == (SCSIOP_MODE_SENSE) as i32
                                && (*cdb).MODE_SENSE.as_ref().PageCode() as i32 == (MODE_SENSE_RETURN_ALL) as i32
                            {
                                (*cdb).MODE_SENSE.as_mut().set_PageCode(MODE_PAGE_CACHING as u8);
                                *Retry = 1u8;
                            }
                        }
                        _ => {}
                    },
                    __mv if __mv == (SCSI_SENSE_NOT_READY) as i32 => match asc as i32 {
                        __mv if __mv == (SCSI_ADSENSE_LUN_NOT_READY) as i32 => match ascq as i32 {
                            __mv if __mv == (SCSI_SENSEQ_BECOMING_READY) as i32 || __mv == (SCSI_SENSEQ_MANUAL_INTERVENTION_REQUIRED) as i32 || __mv == (SCSI_SENSEQ_CAUSE_NOT_REPORTABLE) as i32 => {
                                invalidatePartitionTable = 1u8;
                            }
                            _ => {}
                        },
                        __mv if __mv == (SCSI_ADSENSE_NO_MEDIA_IN_DEVICE) as i32 => {
                            invalidatePartitionTable = 1u8;
                        }
                        _ => {}
                    },
                    __mv if __mv == (SCSI_SENSE_MEDIUM_ERROR) as i32 => {
                        invalidatePartitionTable = 1u8;
                    }
                    __mv if __mv == (SCSI_SENSE_HARDWARE_ERROR) as i32 => {
                        invalidatePartitionTable = 1u8;
                    }
                    __mv if __mv == (SCSI_SENSE_UNIT_ATTENTION) as i32 => {
                        invalidatePartitionTable = 1u8;
                    }
                    __mv if __mv == (SCSI_SENSE_RECOVERED_ERROR) as i32 => {
                        invalidatePartitionTable = 1u8;
                    }
                    _ => {}
                }
            }
        } else {
            match (*Srb).SrbStatus as i32
                & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32
            {
                __mv if __mv == (SRB_STATUS_INVALID_LUN) as i32 || __mv == (SRB_STATUS_INVALID_TARGET_ID) as i32 || __mv == (SRB_STATUS_NO_DEVICE) as i32 || __mv == (SRB_STATUS_NO_HBA) as i32 || __mv == (SRB_STATUS_INVALID_PATH_ID) as i32 || __mv == (SRB_STATUS_COMMAND_TIMEOUT) as i32 || __mv == (SRB_STATUS_TIMEOUT) as i32 || __mv == (SRB_STATUS_SELECTION_TIMEOUT) as i32 || __mv == (SRB_STATUS_REQUEST_FLUSHED) as i32 || __mv == (SRB_STATUS_UNEXPECTED_BUS_FREE) as i32 || __mv == (SRB_STATUS_PARITY_ERROR) as i32 => {
                    invalidatePartitionTable = 1u8;
                }
                __mv if __mv == (SRB_STATUS_ERROR) as i32 => {
                    if scsiStatus as i32 == (SCSISTAT_RESERVATION_CONFLICT) as i32 {
                        invalidatePartitionTable = 1u8;
                    }
                }
                _ => {}
            }
        }
        if invalidatePartitionTable != 0
            && (*Fdo).Characteristics as u32 & 0x1 != 0
        {
            (*Fdo).Flags |= 0x2u32;
        }
    }
}


const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
#[no_mangle]
pub unsafe extern "system" fn DiskSetSpecialHacks(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Data: ULONG_PTR,
) {
    let fdo: PDEVICE_OBJECT = (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if true {
            // always taken in original macro
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"disk.c\0".as_ptr() as *const i8,
                2631,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    TracePrint(
        4,
        TRACE_FLAG_INIT,
        b"Disk SetSpecialHacks, Setting Hacks %p\n\0".as_ptr() as *const i8,
        Data as *mut core::ffi::c_void,
    );

    // HackDisableTaggedQueuing (0x1)
    if (Data & 0x1) != 0 {
        (*FdoExtension).SrbFlags &= !(0x2 as i32) as core::ffi::c_ulong; // SRB_FLAGS_QUEUE_ACTION_ENABLE
    }
    // HackDisableSynchronousTransfers (0x2)
    if (Data & 0x2) != 0 {
        (*FdoExtension).SrbFlags |= 0x8 as core::ffi::c_ulong; // SRB_FLAGS_DISABLE_SYNCH_TRANSFER
    }
    // HackDisableSpinDown (0x4)
    if (Data & 0x4) != 0 {
        (*FdoExtension).ScanForSpecialFlags |= 0x1 as u32; // CLASS_SPECIAL_DISABLE_SPIN_DOWN
    }
    // HackDisableWriteCache (0x8)
    if (Data & 0x8) != 0 {
        (*FdoExtension).ScanForSpecialFlags |= 0x10 as u32; // CLASS_SPECIAL_DISABLE_WRITE_CACHE
    }
    // HackCauseNotReportableHack (0x10)
    if (Data & 0x10) != 0 {
        (*FdoExtension).ScanForSpecialFlags |= 0x20 as u32; // CLASS_SPECIAL_CAUSE_NOT_REPORTABLE_HACK
    }
    // FILE_REMOVABLE_MEDIA and HackRequiresStartUnitCommand (0x20)
    if ((*fdo).Characteristics & 0x1) != 0 && (Data & 0x20) != 0 {
        TracePrint(
            4,
            TRACE_FLAG_INIT,
            b"DiskScanForSpecial (%p) => This unit requires START_UNITS\n\0".as_ptr() as *const i8,
            fdo,
        );
        (*FdoExtension).DeviceFlags = ((*FdoExtension).DeviceFlags as i32 | 0x4) as u16; // DEV_SAFE_START_UNIT
    }
}

#[no_mangle]
pub unsafe extern "C" fn ResetBus(Fdo: PDEVICE_OBJECT) {
    let mut irpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut irp: PIRP = core::ptr::null_mut();
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut context: PCOMPLETION_CONTEXT = core::ptr::null_mut();
    let mut srbEx: PSTORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: PSTOR_ADDR_BTL8 = core::ptr::null_mut();

    StorDebugPrint(
        TRACE_LEVEL_INFORMATION as i32,
        TRACE_FLAG_GENERAL as u32,
        b"Disk ResetBus: Sending reset bus request to port driver.\n\0".as_ptr() as *const i8,
    );

    context = ExAllocatePoolWithTag(
        NonPagedPoolNx as POOL_TYPE,
        (core::mem::size_of::<COMPLETION_CONTEXT>() as u32) as usize,
        DISK_TAG_CCONTEXT,
    ) as PCOMPLETION_CONTEXT;

    if context.is_null() {
        return;
    }

    (*context).DeviceObject = Fdo;
    srb = &raw mut (*context).Srb.Srb as PSCSI_REQUEST_BLOCK;

    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbEx = &raw mut (*context).Srb.SrbEx as PSTORAGE_REQUEST_BLOCK;
        core::ptr::write_bytes(srbEx as *mut u8, 0u8, 184usize);
        (*srbEx).Length = 8u16;
        (*srbEx).Function = (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8;
        (*srbEx).Signature = SRB_SIGNATURE as u32;
        (*srbEx).Version = (STORAGE_REQUEST_BLOCK_VERSION_1 as u8) as u32;
        (*srbEx).SrbLength = 184u32;
        (*srbEx).SrbFunction = SRB_FUNCTION_RESET_BUS as u32;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
        storAddrBtl8 = (srbEx as *mut u8).add((*srbEx).AddressOffset as usize) as PSTOR_ADDR_BTL8;
        (*storAddrBtl8).Type = (STOR_ADDRESS_TYPE_BTL8 as u8) as u16;
        (*storAddrBtl8).AddressLength = (STOR_ADDR_BTL8_ADDRESS_LENGTH as u8) as u32;
    } else {
        core::ptr::write_bytes(srb as *mut u8, 0u8, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
        (*srb).Length = (core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u8) as u16;
        (*srb).Function = SRB_FUNCTION_RESET_BUS as u8;
    }

    irp = IoAllocateIrp((*Fdo).StackSize as i8, 0u8);

    if irp.is_null() {
        ExFreePool(context as *mut core::ffi::c_void);
        context = core::ptr::null_mut();
        return;
    }

    ClassAcquireRemoveLockEx(Fdo, irp, b"disk.c\0".as_ptr() as *const i8, 2817u32);

    IoSetCompletionRoutine(
        irp,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, *mut core::ffi::c_void) -> NTSTATUS,
            _,
        >(ClassAsynchronousCompletion)),
        context as *mut core::ffi::c_void,
        1u8,
        1u8,
        1u8,
    );

    irpStack = IoGetNextIrpStackLocation(irp);
    (*irpStack).MajorFunction = IRP_MJ_SCSI as u8;

    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        (*srbEx).RequestPriority = (IoGetIoPriorityHint(irp) as u8) as u16;
        (*srbEx).OriginalRequest = irp as *mut core::ffi::c_void;
    } else {
        (*srb).OriginalRequest = irp as *mut core::ffi::c_void;
    }

    (*irpStack).Parameters.Scsi.Srb = srb as *mut _;

    IoCallDriver(&raw mut (*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_mut()).LowerDeviceObject as PDEVICE_OBJECT, irp);
}







#[no_mangle]
pub unsafe extern "C" fn DiskLogCacheInformation(
    mut FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    mut CacheInfo: *mut DISK_CACHE_INFORMATION,
    mut Status: NTSTATUS,
) {
    let mut logEntry: *mut IO_ERROR_LOG_PACKET = core::ptr::null_mut();

    if (KeGetCurrentIrql() as i32) > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                2864i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    logEntry = IoAllocateErrorLogEntry(
        ((*FdoExtension).__bindgen_anon_1).CommonExtension.as_ref().DeviceObject as *mut core::ffi::c_void,
        (core::mem::size_of::<IO_ERROR_LOG_PACKET>() as usize)
            .wrapping_add((4usize).wrapping_mul(core::mem::size_of::<u32>() as usize)) as u8,
    ) as *mut IO_ERROR_LOG_PACKET;

    if !logEntry.is_null() {
        let mut diskData: *mut DISK_DATA =
            ((*FdoExtension).__bindgen_anon_1).CommonExtension.as_ref().DriverData as *mut DISK_DATA;
        let mut bIsEnabled: i32 =
            (((*FdoExtension).DeviceFlags as i32 & (DEV_WRITE_CACHE) as i32) != 0) as i32;

        (*logEntry).FinalStatus = Status;
        (*logEntry).ErrorCode = if bIsEnabled != 0 { IO_WRITE_CACHE_ENABLED } else { IO_WRITE_CACHE_DISABLED } ;
        (*logEntry).SequenceNumber = 0u32;
        (*logEntry).MajorFunctionCode = (IRP_MJ_SCSI) as u8;
        (*logEntry).IoControlCode = 0u32;
        (*logEntry).RetryCount = 0u8;
        (*logEntry).UniqueErrorValue = 0x1u32;
        (*logEntry).DumpDataSize = (4usize).wrapping_mul(core::mem::size_of::<u32>() as usize) as u16;

        *((core::ptr::addr_of_mut!((*logEntry).DumpData) as *mut u32).offset(0isize)) =
            (*diskData).ScsiAddress.PathId as u32;
        *((core::ptr::addr_of_mut!((*logEntry).DumpData) as *mut u32).offset(1isize)) =
            (*diskData).ScsiAddress.TargetId as u32;
        *((core::ptr::addr_of_mut!((*logEntry).DumpData) as *mut u32).offset(2isize)) =
            (*diskData).ScsiAddress.Lun as u32;
        *((core::ptr::addr_of_mut!((*logEntry).DumpData) as *mut u32).offset(3isize)) =
            (*CacheInfo).WriteCacheEnabled as u32;

        IoWriteErrorLogEntry(logEntry as *mut core::ffi::c_void);
    }
}




#[no_mangle]
pub unsafe extern "C" fn DiskGetInfoExceptionInformation(
    mut FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    mut ReturnPageData: *mut MODE_INFO_EXCEPTIONS,
) -> NTSTATUS {
    let mut modeData: *mut MODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut pageData: *mut MODE_INFO_EXCEPTIONS = core::ptr::null_mut();
    let mut length: u32 = 0;
    let mut status: NTSTATUS = 0;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const core::ffi::c_char,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const core::ffi::c_char,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const core::ffi::c_char,
                2907 as i32,
                b"0\0" as *const u8 as *const core::ffi::c_char,
            );
            __int2c();
        }
    }

    modeData = ExAllocatePoolWithTag(
        (NonPagedPoolNxCacheAligned) as u32,
        MODE_DATA_SIZE as usize,
        DISK_TAG_INFO_EXCEPTION,
    ) as *mut MODE_PARAMETER_HEADER;

    if modeData.is_null() {
        TracePrint(
            TRACE_LEVEL_ERROR as i32,
            TRACE_FLAG_WMI as u32,
            b"DiskGetInfoExceptionInformation: Unable to allocate mode data buffer\n\0"
                as *const u8 as *const core::ffi::c_char,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(modeData as *mut u8, 0, (MODE_DATA_SIZE) as usize);

    length = ClassModeSense(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        modeData as *mut i8,
        MODE_DATA_SIZE as u32,
        (MODE_PAGE_FAULT_REPORTING) as u8,
    );

    if (length as usize) < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
        length = ClassModeSense(
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            modeData as *mut i8,
            MODE_DATA_SIZE as u32,
            (MODE_PAGE_FAULT_REPORTING) as u8,
        );

        if (length as usize) < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
            TracePrint(
                TRACE_LEVEL_ERROR as i32,
                TRACE_FLAG_WMI as u32,
                b"DiskGetInfoExceptionInformation: Mode Sense failed\n\0"
                    as *const u8 as *const core::ffi::c_char,
            );
            if !modeData.is_null() {
                ExFreePool(modeData as *mut core::ffi::c_void);
                modeData = core::ptr::null_mut();
            }
            return STATUS_IO_DEVICE_ERROR;
        }
    }

    if length > ((*modeData).ModeDataLength as u32 + 1) {
        length = (*modeData).ModeDataLength as u32 + 1;
    }

    pageData = ClassFindModePage(
        modeData as *mut i8,
        length,
        (MODE_PAGE_FAULT_REPORTING) as u8,
        1 as u8, // TRUE
    ) as *mut MODE_INFO_EXCEPTIONS;

    if !pageData.is_null() {
        core::ptr::copy_nonoverlapping(
            pageData as *const u8,
            ReturnPageData as *mut u8,
            core::mem::size_of::<MODE_INFO_EXCEPTIONS>(),
        );
        status = STATUS_SUCCESS;
    } else {
        status = STATUS_NOT_SUPPORTED;
    }

    let msg: *const core::ffi::c_char = if status >= 0 {
        b"does\0" as *const u8 as *const core::ffi::c_char
    } else {
        b"does not\0" as *const u8 as *const core::ffi::c_char
    };

    TracePrint(
        TRACE_LEVEL_INFORMATION as i32,
        TRACE_FLAG_WMI as u32,
        b"DiskGetInfoExceptionInformation: %s support SMART for device %p\n\0"
            as *const u8 as *const core::ffi::c_char,
        msg,
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject as *const core::ffi::c_void,
    );

    if !modeData.is_null() {
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
    }

    return status;
}





#[no_mangle]
pub unsafe extern "C" fn DiskSetInfoExceptionInformation(
    mut FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    mut PageData: PMODE_INFO_EXCEPTIONS,
) -> NTSTATUS {
    let mut i: u32;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE() inline
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                2994i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    i = 0;
    while i < 2 {
        status = DiskModeSelect(
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            PageData as *mut i8,
            core::mem::size_of::<MODE_INFO_EXCEPTIONS>() as u32,
            1u8,
        );
        i = i.wrapping_add(1);
    }

    TracePrint(
        4i32,
        TRACE_FLAG_WMI,
        b"DiskSetInfoExceptionInformation: %s for device %p\n\0".as_ptr() as *const i8,
        if status >= 0 {
            b"succeeded\0".as_ptr() as *const i8
        } else {
            b"failed\0".as_ptr() as *const i8
        },
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
    );

    return status;
}

#[no_mangle]
pub unsafe extern "system" fn DiskGetCacheInformation(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    CacheInfo: *mut DISK_CACHE_INFORMATION,
) -> NTSTATUS {
    let mut modeData: *mut MODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut pageData: *mut MODE_CACHING_PAGE = core::ptr::null_mut();
    let mut length: u32;

    // PAGED_CODE expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            c"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                c"Assertion failed at %s(%d): %s\n".as_ptr() as *const i8,
                c"drivers/storage/class/disk/disk.c".as_ptr() as *const i8,
                3050i32,
                c"0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    modeData = ExAllocatePoolWithTag(
        NonPagedPoolCacheAligned,
        192usize,
        DISK_TAG_DISABLE_CACHE,
    ) as *mut MODE_PARAMETER_HEADER;
    if modeData.is_null() {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            c"DiskGetSetCacheInformation: Unable to allocate mode data buffer\n".as_ptr()
                as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(modeData as *mut u8, 0u8, 192);

    length = ClassModeSense(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        modeData as *mut i8,
        192u32,
        (MODE_PAGE_CACHING) as u8,
    );
    if (length as usize) < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
        length = ClassModeSense(
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            modeData as *mut i8,
            192u32,
            (MODE_PAGE_CACHING) as u8,
        );
        if (length as usize) < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
            StorDebugPrint(
                2,
                TRACE_FLAG_IOCTL,
                c"DiskGetCacheInformation: Mode Sense failed\n".as_ptr() as *const i8,
            );
            if !modeData.is_null() {
                ExFreePool(modeData as *mut core::ffi::c_void);
                modeData = core::ptr::null_mut();
            }
            return STATUS_IO_DEVICE_ERROR;
        }
    }
    if length > ((*modeData).ModeDataLength as i32 + 1) as u32 {
        length = ((*modeData).ModeDataLength as i32 + 1) as u32;
    }
    pageData = ClassFindModePage(
        modeData as *mut i8,
        length,
        (MODE_PAGE_CACHING) as u8,
        1i32 as BOOLEAN,
    ) as *mut MODE_CACHING_PAGE;
    if pageData.is_null() {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            c"DiskGetCacheInformation: Unable to find caching mode page.\n".as_ptr()
                as *const i8,
        );
        if !modeData.is_null() {
            ExFreePool(modeData as *mut core::ffi::c_void);
            modeData = core::ptr::null_mut();
        }
        return STATUS_NOT_SUPPORTED;
    }
    core::ptr::write_bytes(
        CacheInfo as *mut u8,
        0u8,
        core::mem::size_of::<DISK_CACHE_INFORMATION>(),
    );
    (*CacheInfo).ParametersSavable = (*pageData).PageSavable() as BOOLEAN;
    (*CacheInfo).ReadCacheEnabled = ((*pageData).ReadDisableCache() == 0) as i32 as BOOLEAN;
    (*CacheInfo).WriteCacheEnabled = (*pageData).WriteCacheEnable() as BOOLEAN;
    (*CacheInfo).ReadRetentionPriority = (if (*pageData).ReadRetensionPriority() as i32 == 0xf {
        0x2
    } else if (*pageData).ReadRetensionPriority() as i32 == 0x2 {
        0xf
    } else {
        (*pageData).ReadRetensionPriority() as i32
    }) as u32;
    (*CacheInfo).WriteRetentionPriority = (if (*pageData).WriteRetensionPriority() as i32 == 0xf {
        0x2
    } else if (*pageData).WriteRetensionPriority() as i32 == 0x2 {
        0xf
    } else {
        (*pageData).WriteRetensionPriority() as i32
    }) as u32;
    (*CacheInfo).DisablePrefetchTransferLength = ((((*pageData).DisablePrefetchTransfer[0] as i32)
        << 8)
        + (*pageData).DisablePrefetchTransfer[1] as i32) as u16;
    (*CacheInfo).__bindgen_anon_1.ScalarPrefetch.Minimum =
        ((((*pageData).MinimumPrefetch[0] as i32) << 8) + (*pageData).MinimumPrefetch[1] as i32)
            as u16;
    (*CacheInfo).__bindgen_anon_1.ScalarPrefetch.Maximum =
        ((((*pageData).MaximumPrefetch[0] as i32) << 8) + (*pageData).MaximumPrefetch[1] as i32)
            as u16;
    if (*pageData).MultiplicationFactor() != 0 {
        (*CacheInfo).PrefetchScalar = 1i32 as BOOLEAN;
        (*CacheInfo).__bindgen_anon_1.ScalarPrefetch.MaximumBlocks =
            ((((*pageData).MaximumPrefetchCeiling[0] as i32) << 8)
                + (*pageData).MaximumPrefetchCeiling[1] as i32) as u16;
    }
    if !modeData.is_null() {
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DiskSetCacheInformation(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    CacheInfo: *mut DISK_CACHE_INFORMATION,
) -> NTSTATUS {
    let mut modeData: *mut MODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut length: ULONG;
    let mut pageData: *mut MODE_CACHING_PAGE;
    let mut i: ULONG;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                3197,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    modeData = ExAllocatePoolWithTag(
        0x204, // NonPagedPoolCacheAligned (placeholder)
        MODE_DATA_SIZE as SIZE_T,
        DISK_TAG_DISABLE_CACHE,
    ) as *mut MODE_PARAMETER_HEADER;

    if modeData.is_null() {
        TracePrint(2, TRACE_FLAG_IOCTL,
            b"DiskSetCacheInformation: Unable to allocate mode data buffer\n\0" as *const u8 as *const i8);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(modeData.cast::<u8>(), 0, (MODE_DATA_SIZE) as usize);

    length = ClassModeSense(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        modeData as PCHAR,
        MODE_DATA_SIZE as ULONG,
        (MODE_PAGE_CACHING) as u8,
    );

    if (length as usize) < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
        length = ClassModeSense(
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            modeData as PCHAR,
            MODE_DATA_SIZE as ULONG,
            (MODE_PAGE_CACHING) as u8,
        );

        if (length as usize) < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
            TracePrint(2, TRACE_FLAG_IOCTL,
                b"DiskSetCacheInformation: Mode Sense failed\n\0" as *const u8 as *const i8);
            ExFreePool(modeData as *mut core::ffi::c_void);
            modeData = core::ptr::null_mut();
            return STATUS_IO_DEVICE_ERROR;
        }
    }

    if length > ((*modeData).ModeDataLength as i32 + 1) as ULONG {
        length = ((*modeData).ModeDataLength as i32 + 1) as ULONG;
    }

    pageData = ClassFindModePage(
        modeData as PCHAR,
        length,
        (MODE_PAGE_CACHING) as u8,
        TRUE,
    ) as *mut MODE_CACHING_PAGE;

    if pageData.is_null() {
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
        return STATUS_NOT_SUPPORTED;
    }

    (*pageData).set_PageSavable(0u8);
    (*pageData).set_ReadDisableCache(if (*CacheInfo).ReadCacheEnabled == 0 { 1u8 } else { 0u8 });
    (*pageData).set_MultiplicationFactor((*CacheInfo).PrefetchScalar as u8);
    (*pageData).set_WriteCacheEnable((*CacheInfo).WriteCacheEnabled as u8);

    // TRANSLATE_RETENTION_PRIORITY for Write
    let w_val = (*CacheInfo).WriteRetentionPriority as u32;
    let w_mapped = if w_val == 0xf { 0x2u32 } else if w_val == 0x2 { 0xfu32 } else { w_val };
    (*pageData).set_WriteRetensionPriority(w_mapped as u8);

    // TRANSLATE_RETENTION_PRIORITY for Read
    let r_val = (*CacheInfo).ReadRetentionPriority as u32;
    let r_mapped = if r_val == 0xf { 0x2u32 } else if r_val == 0x2 { 0xfu32 } else { r_val };
    (*pageData).set_ReadRetensionPriority(r_mapped as u8);

    (*pageData).DisablePrefetchTransfer[0] =
        ((*CacheInfo).DisablePrefetchTransferLength as u16 >> 8) as u8;
    (*pageData).DisablePrefetchTransfer[1] =
        ((*CacheInfo).DisablePrefetchTransferLength as u16 & 0x00ff) as u8;

    (*pageData).MinimumPrefetch[0] =
        ((*CacheInfo).__bindgen_anon_1.ScalarPrefetch.Minimum as u16 >> 8) as u8;
    (*pageData).MinimumPrefetch[1] =
        ((*CacheInfo).__bindgen_anon_1.ScalarPrefetch.Minimum as u16 & 0x00ff) as u8;

    (*pageData).MaximumPrefetch[0] =
        ((*CacheInfo).__bindgen_anon_1.ScalarPrefetch.Maximum as u16 >> 8) as u8;
    (*pageData).MaximumPrefetch[1] =
        ((*CacheInfo).__bindgen_anon_1.ScalarPrefetch.Maximum as u16 & 0x00ff) as u8;

    if (*pageData).MultiplicationFactor() != 0 {
        (*pageData).MaximumPrefetchCeiling[0] =
            ((*CacheInfo).__bindgen_anon_1.ScalarPrefetch.MaximumBlocks as u16 >> 8) as u8;
        (*pageData).MaximumPrefetchCeiling[1] =
            ((*CacheInfo).__bindgen_anon_1.ScalarPrefetch.MaximumBlocks as u16 & 0x00ff) as u8;
    }

    i = 0;
    while i < 2 {
        status = DiskModeSelect(
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            pageData as PCHAR,
            ((*pageData).PageLength as i32 + 2) as ULONG,
            (*CacheInfo).ParametersSavable,
        );

        if NT_SUCCESS(status) {
            if (*CacheInfo).WriteCacheEnabled != 0 {
                (*FdoExtension).DeviceFlags |= 0x1;
            } else {
                (*FdoExtension).DeviceFlags &= !0x1;
            }

            // ADJUST_FUA_FLAG expansion
            if ((*FdoExtension).DeviceFlags & 0x1) != 0
                && ((*FdoExtension).DeviceFlags & 0x10) == 0
                && ((*FdoExtension).ScanForSpecialFlags & 0x80) == 0
            {
                (*FdoExtension).CdbForceUnitAccess = 1u8;
            } else {
                (*FdoExtension).CdbForceUnitAccess = 0u8;
            }

            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    if NT_SUCCESS(status) {
        // empty
    } else {
        // SET_FLAG for cache modify unsuccessful
        (*FdoExtension).ScanForSpecialFlags |= 0x40;
    }

    ExFreePool(modeData as *mut core::ffi::c_void);
    modeData = core::ptr::null_mut();
    return status;
}


#[no_mangle]
pub unsafe extern "system" fn DiskEtwEnableCallback(
    SourceId: *const GUID,
    IsEnabled: u32,
    Level: u8,
    MatchAnyKeyword: u64,
    MatchAllKeyword: u64,
    FilterData: *mut core::ffi::c_void,
    CallbackContext: *mut core::ffi::c_void,
) {
    let _ = SourceId;
    let _ = Level;
    let _ = MatchAnyKeyword;
    let _ = MatchAllKeyword;
    let _ = FilterData;
    let _ = CallbackContext;
    DiskETWEnabled = if IsEnabled != 0 { 1u8 } else { 0u8 };
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
//     disk.c
//
// Abstract:
//
//     SCSI disk class driver
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
// #define DEBUG_MAIN_SOURCE   1
// #include "disk.h"
//
//
// //
// // Now instantiate the GUIDs
// //
//
// #include "initguid.h"
// #include "ntddstor.h"
// #include "ntddvol.h"
// #include "ioevent.h"
//
// #ifdef DEBUG_USE_WPP
// #include "disk.tmh"
// #endif
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(INIT, DriverEntry)
// #pragma alloc_text(PAGE, DiskUnload)
// #pragma alloc_text(PAGE, DiskCreateFdo)
// #pragma alloc_text(PAGE, DiskDetermineMediaTypes)
// #pragma alloc_text(PAGE, DiskModeSelect)
// #pragma alloc_text(PAGE, DisableWriteCache)
// #pragma alloc_text(PAGE, DiskSetSpecialHacks)
// #pragma alloc_text(PAGE, DiskGetCacheInformation)
// #pragma alloc_text(PAGE, DiskSetCacheInformation)
// #pragma alloc_text(PAGE, DiskLogCacheInformation)
// #pragma alloc_text(PAGE, DiskSetInfoExceptionInformation)
// #pragma alloc_text(PAGE, DiskGetInfoExceptionInformation)
// #pragma alloc_text(PAGE, DiskIoctlGetCacheSetting)
// #pragma alloc_text(PAGE, DiskIoctlSetCacheSetting)
// #pragma alloc_text(PAGE, DiskIoctlGetLengthInfo)
// #pragma alloc_text(PAGE, DiskIoctlGetDriveGeometry)
// #pragma alloc_text(PAGE, DiskIoctlGetDriveGeometryEx)
// #pragma alloc_text(PAGE, DiskIoctlGetCacheInformation)
// #pragma alloc_text(PAGE, DiskIoctlSetCacheInformation)
// #pragma alloc_text(PAGE, DiskIoctlGetMediaTypesEx)
// #pragma alloc_text(PAGE, DiskIoctlPredictFailure)
// #pragma alloc_text(PAGE, DiskIoctlEnableFailurePrediction)
// #pragma alloc_text(PAGE, DiskIoctlReassignBlocks)
// #pragma alloc_text(PAGE, DiskIoctlReassignBlocksEx)
// #pragma alloc_text(PAGE, DiskIoctlIsWritable)
// #pragma alloc_text(PAGE, DiskIoctlUpdateDriveSize)
// #pragma alloc_text(PAGE, DiskIoctlGetVolumeDiskExtents)
// #pragma alloc_text(PAGE, DiskIoctlSmartGetVersion)
// #pragma alloc_text(PAGE, DiskIoctlSmartReceiveDriveData)
// #pragma alloc_text(PAGE, DiskIoctlSmartSendDriveCommand)
// #pragma alloc_text(PAGE, DiskIoctlVerifyThread)
//
// #endif
//
// //
// //  ETW related globals
// //
// BOOLEAN DiskETWEnabled = FALSE;
//
// BOOLEAN DiskIsPastReinit = FALSE;
//
// const GUID GUID_NULL = { 0 };
// #define DiskCompareGuid(_First,_Second) \
//     (memcmp ((_First),(_Second), sizeof (GUID)))
//
// //
// // This macro is used to work around a bug in the definition of
// // DISK_CACHE_RETENTION_PRIORITY.  The value KeepReadData should be
// // assigned 0xf rather than 0x2.  Since the interface was already published
// // when this was discovered the disk driver has been modified to translate
// // between the interface value and the correct scsi value.
// //
// // 0x2 is turned into 0xf
// // 0xf is turned into 0x2 - this ensures that future SCSI defintions can be
// //                          accomodated.
// //
//
// #define TRANSLATE_RETENTION_PRIORITY(_x)\
//         ((_x) == 0xf ?  0x2 :           \
//             ((_x) == 0x2 ? 0xf : _x)    \
//         )
//
// #define IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS_ADMIN  CTL_CODE(IOCTL_VOLUME_BASE, 0, METHOD_BUFFERED, FILE_READ_ACCESS)
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskDriverReinit(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PVOID Nothing,
//     IN ULONG Count
//     )
// {
//     UNREFERENCED_PARAMETER(DriverObject);
//     UNREFERENCED_PARAMETER(Nothing);
//     UNREFERENCED_PARAMETER(Count);
//
//     DiskIsPastReinit = TRUE;
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskBootDriverReinit(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PVOID Nothing,
//     IN ULONG Count
//     )
// {
//     IoRegisterDriverReinitialization(DriverObject, DiskDriverReinit, NULL);
//
// #if defined(_X86_) || defined(_AMD64_)
//
//     DiskDriverReinitialization(DriverObject, Nothing, Count);
//
// #else
//
//     UNREFERENCED_PARAMETER(Nothing);
//     UNREFERENCED_PARAMETER(Count);
//
// #endif
//
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DriverEntry(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PUNICODE_STRING RegistryPath
//     )
//
// /*++
//
// Routine Description:
//
//     This routine initializes the SCSI hard disk class driver.
//
// Arguments:
//
//     DriverObject - Pointer to driver object created by system.
//
//     RegistryPath - Pointer to the name of the services node for this driver.
//
// Return Value:
//
//     The function value is the final status from the initialization operation.
//
// --*/
//
// {
//     CLASS_INIT_DATA InitializationData = { 0 };
//     CLASS_QUERY_WMI_REGINFO_EX_LIST classQueryWmiRegInfoExList = { 0 };
//     GUID guidQueryRegInfoEx = GUID_CLASSPNP_QUERY_REGINFOEX;
//     GUID guidSrbSupport = GUID_CLASSPNP_SRB_SUPPORT;
//     ULONG srbSupport;
//
//     NTSTATUS status;
//
//     //
//     // Initializes tracing
//     //
//     WPP_INIT_TRACING(DriverObject, RegistryPath);
//
// #if defined(_X86_) || defined(_AMD64_)
//
//     //
//     // Read the information NtDetect squirreled away about the disks in this
//     // system.
//     //
//
//     DiskSaveDetectInfo(DriverObject);
//
// #endif
//
//     InitializationData.InitializationDataSize = sizeof(CLASS_INIT_DATA);
//
//     //
//     // Setup sizes and entry points for functional device objects
//     //
//
//     InitializationData.FdoData.DeviceExtensionSize   = FUNCTIONAL_EXTENSION_SIZE;
//     InitializationData.FdoData.DeviceType            = FILE_DEVICE_DISK;
//     InitializationData.FdoData.DeviceCharacteristics = FILE_DEVICE_SECURE_OPEN;
//
//     InitializationData.FdoData.ClassInitDevice    = DiskInitFdo;
//     InitializationData.FdoData.ClassStartDevice   = DiskStartFdo;
//     InitializationData.FdoData.ClassStopDevice    = DiskStopDevice;
//     InitializationData.FdoData.ClassRemoveDevice  = DiskRemoveDevice;
//     InitializationData.FdoData.ClassPowerDevice   = ClassSpinDownPowerHandler;
//
//     InitializationData.FdoData.ClassError         = DiskFdoProcessError;
//     InitializationData.FdoData.ClassReadWriteVerification = DiskReadWriteVerification;
//     InitializationData.FdoData.ClassDeviceControl = DiskDeviceControl;
//     InitializationData.FdoData.ClassShutdownFlush = DiskShutdownFlush;
//     InitializationData.FdoData.ClassCreateClose   = NULL;
//
//
//     InitializationData.FdoData.ClassWmiInfo.GuidCount               = 7;
//     InitializationData.FdoData.ClassWmiInfo.GuidRegInfo             = DiskWmiFdoGuidList;
//     InitializationData.FdoData.ClassWmiInfo.ClassQueryWmiRegInfo    = DiskFdoQueryWmiRegInfo;
//     InitializationData.FdoData.ClassWmiInfo.ClassQueryWmiDataBlock  = DiskFdoQueryWmiDataBlock;
//     InitializationData.FdoData.ClassWmiInfo.ClassSetWmiDataBlock    = DiskFdoSetWmiDataBlock;
//     InitializationData.FdoData.ClassWmiInfo.ClassSetWmiDataItem     = DiskFdoSetWmiDataItem;
//     InitializationData.FdoData.ClassWmiInfo.ClassExecuteWmiMethod   = DiskFdoExecuteWmiMethod;
//     InitializationData.FdoData.ClassWmiInfo.ClassWmiFunctionControl = DiskWmiFunctionControl;
//
//     InitializationData.ClassAddDevice = DiskAddDevice;
//     InitializationData.ClassUnload = DiskUnload;
//
//     //
//     // Initialize regregistration data structures
//     //
//
//     DiskInitializeReregistration();
//
//     //
//     // Call the class init routine
//     //
//
//     status = ClassInitialize(DriverObject, RegistryPath, &InitializationData);
//
//     if (NT_SUCCESS(status)) {
//
//         IoRegisterBootDriverReinitialization(DriverObject,
//                                              DiskBootDriverReinit,
//                                              NULL);
//     }
//
//     //
//     // Call class init Ex routine to register a
//     // PCLASS_QUERY_WMI_REGINFO_EX routine
//     //
//
//     classQueryWmiRegInfoExList.Size = sizeof(CLASS_QUERY_WMI_REGINFO_EX_LIST);
//     classQueryWmiRegInfoExList.ClassFdoQueryWmiRegInfoEx = DiskFdoQueryWmiRegInfoEx;
//
//     (VOID)ClassInitializeEx(DriverObject,
//                             &guidQueryRegInfoEx,
//                             &classQueryWmiRegInfoExList);
//
//     //
//     // Call class init Ex routine to register SRB support
//     //
//     srbSupport = CLASS_SRB_SCSI_REQUEST_BLOCK | CLASS_SRB_STORAGE_REQUEST_BLOCK;
//     if (!NT_SUCCESS(ClassInitializeEx(DriverObject,
//                                       &guidSrbSupport,
//                                       &srbSupport))) {
//         //
//         // Should not fail
//         //
//         NT_ASSERT(FALSE);
//     }
//
//
//     return status;
//
// } // end DriverEntry()
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskUnload(
//     IN PDRIVER_OBJECT DriverObject
//     )
// {
//     PAGED_CODE();
//
// #if defined(_X86_) || defined(_AMD64_)
//     DiskCleanupDetectInfo(DriverObject);
// #else
//     // NB: Need to use UNREFERENCED_PARAMETER to prevent build error
//     //     DriverObject is not referenced below in WPP_CLEANUP.
//     //     WPP_CLEANUP is currently an "NOOP" marco.
//     UNREFERENCED_PARAMETER(DriverObject);
// #endif
//
//
//     //
//     // Cleans up tracing
//     //
//     WPP_CLEANUP(DriverObject);
//
//     return;
// }
//
//
// NTSTATUS
// DiskCreateFdo(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PDEVICE_OBJECT PhysicalDeviceObject,
//     IN PULONG DeviceCount,
//     IN BOOLEAN DasdAccessOnly
//     )
//
// /*++
//
// Routine Description:
//
//     This routine creates an object for the functional device
//
// Arguments:
//
//     DriverObject - Pointer to driver object created by system.
//
//     PhysicalDeviceObject - Lower level driver we should attach to
//
//     DeviceCount  - Number of previously installed devices.
//
//     DasdAccessOnly - indicates whether or not a file system is allowed to mount
//                      on this device object.  Used to avoid double-mounting of
//                      file systems on super-floppies (which can unfortunately be
//                      fixed disks).  If set the i/o system will only allow rawfs
//                      to be mounted.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
//
// {
//     PCCHAR deviceName = NULL;
//     HANDLE handle = NULL;
//     PDEVICE_OBJECT lowerDevice  = NULL;
//     PDEVICE_OBJECT deviceObject = NULL;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     *DeviceCount = 0;
//
//     //
//     // Set up an object directory to contain the objects for this
//     // device and all its partitions.
//     //
//
//     do {
//
//         WCHAR dirBuffer[64] = { 0 };
//         _UNICODE_STRING dirName;
//         _OBJECT_ATTRIBUTES objectAttribs;
//
//         status = RtlStringCchPrintfW(dirBuffer, sizeof(dirBuffer) / sizeof(dirBuffer[0]) - 1, L"\\Device\\Harddisk%d", *DeviceCount);
//         if (!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP, "DiskCreateFdo: Format symbolic link failed with error: 0x%X\n", status));
//             return status;
//         }
//
//         RtlInitUnicodeString(&dirName, dirBuffer);
//
//         InitializeObjectAttributes(&objectAttribs,
//                                    &dirName,
//                                    OBJ_CASE_INSENSITIVE | OBJ_PERMANENT | OBJ_KERNEL_HANDLE,
//                                    NULL,
//                                    NULL);
//
//         status = ZwCreateDirectoryObject(&handle,
//                                          DIRECTORY_ALL_ACCESS,
//                                          &objectAttribs);
//
//         (*DeviceCount)++;
//
//     } while((status == STATUS_OBJECT_NAME_COLLISION) ||
//             (status == STATUS_OBJECT_NAME_EXISTS));
//
//     if (!NT_SUCCESS(status)) {
//
//         TracePrint((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP, "DiskCreateFdo: Could not create directory - %lx\n", status));
//
//         return(status);
//     }
//
//     //
//     // When this loop exits the count is inflated by one - fix that.
//     //
//
//     (*DeviceCount)--;
//
//     //
//     // Claim the device.
//     //
//
//     lowerDevice = IoGetAttachedDeviceReference(PhysicalDeviceObject);
//
//     status = ClassClaimDevice(lowerDevice, FALSE);
//
//     if (!NT_SUCCESS(status)) {
//         ZwMakeTemporaryObject(handle);
//         ZwClose(handle);
//         ObDereferenceObject(lowerDevice);
//         return status;
//     }
//
//     //
//     // Create a device object for this device. Each physical disk will
//     // have at least one device object. The required device object
//     // describes the entire device. Its directory path is
//     // \Device\HarddiskN\Partition0, where N = device number.
//     //
//
//     status = DiskGenerateDeviceName(*DeviceCount, &deviceName);
//
//     if(!NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP, "DiskCreateFdo - couldn't create name %lx\n", status));
//
//         goto DiskCreateFdoExit;
//
//     }
//
//     status = ClassCreateDeviceObject(DriverObject,
//                                      deviceName,
//                                      PhysicalDeviceObject,
//                                      TRUE,
//                                      &deviceObject);
//
//     if (!NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP, "DiskCreateFdo: Can not create device object %s\n", deviceName));
//         goto DiskCreateFdoExit;
//     }
//
//     FREE_POOL(deviceName);
//
//     //
//     // Indicate that IRPs should include MDLs for data transfers.
//     //
//
//     SET_FLAG(deviceObject->Flags, DO_DIRECT_IO);
//
//     fdoExtension = deviceObject->DeviceExtension;
//
//     if(DasdAccessOnly) {
//
//         //
//         // Inidicate that only RAW should be allowed to mount on the root
//         // partition object.  This ensures that a file system can't doubly
//         // mount on a super-floppy by mounting once on P0 and once on P1.
//         //
//
// #ifdef _MSC_VER
// #pragma prefast(suppress:28175);
// #endif
//         SET_FLAG(deviceObject->Vpb->Flags, VPB_RAW_MOUNT);
//     }
//
//     //
//     // Initialize lock count to zero. The lock count is used to
//     // disable the ejection mechanism on devices that support
//     // removable media. Only the lock count in the physical
//     // device extension is used.
//     //
//
//     fdoExtension->LockCount = 0;
//
//     //
//     // Save system disk number.
//     //
//
//     fdoExtension->DeviceNumber = *DeviceCount;
//
//     //
//     // Set the alignment requirements for the device based on the
//     // host adapter requirements
//     //
//
//     if (lowerDevice->AlignmentRequirement > deviceObject->AlignmentRequirement) {
//         deviceObject->AlignmentRequirement = lowerDevice->AlignmentRequirement;
//     }
//
//     //
//     // Finally, attach to the pdo
//     //
//
//     fdoExtension->LowerPdo = PhysicalDeviceObject;
//
//     fdoExtension->CommonExtension.LowerDeviceObject =
//         IoAttachDeviceToDeviceStack(deviceObject, PhysicalDeviceObject);
//
//
//     if(fdoExtension->CommonExtension.LowerDeviceObject == NULL) {
//
//         //
//         // Uh - oh, we couldn't attach
//         // cleanup and return
//         //
//
//         status = STATUS_UNSUCCESSFUL;
//         goto DiskCreateFdoExit;
//     }
//
//     //
//     // Clear the init flag.
//     //
//
//     CLEAR_FLAG(deviceObject->Flags, DO_DEVICE_INITIALIZING);
//
//     //
//     // Store a handle to the device object directory for this disk
//     //
//
//     fdoExtension->DeviceDirectory = handle;
//
//     ObDereferenceObject(lowerDevice);
//
//     return STATUS_SUCCESS;
//
// DiskCreateFdoExit:
//
//     if (deviceObject != NULL)
//     {
//         IoDeleteDevice(deviceObject);
//     }
//
//     FREE_POOL(deviceName);
//
//     ObDereferenceObject(lowerDevice);
//
//     ZwMakeTemporaryObject(handle);
//     ZwClose(handle);
//
//     return status;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskReadWriteVerification(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     I/O System entry for read and write requests to SCSI disks.
//
// Arguments:
//
//     DeviceObject - Pointer to driver object created by system.
//     Irp - IRP involved.
//
// Return Value:
//
//     NT Status
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpSp = IoGetCurrentIrpStackLocation(Irp);
//     ULONG residualBytes;
//     ULONG residualOffset;
//     NTSTATUS status = STATUS_SUCCESS;
//
//     //
//     // Make sure that the request is within the bounds of the partition,
//     // the number of bytes to transfer and the byte offset are a
//     // multiple of the sector size.
//     //
//
//     residualBytes = irpSp->Parameters.Read.Length & (commonExtension->PartitionZeroExtension->DiskGeometry.BytesPerSector - 1);
//     residualOffset = irpSp->Parameters.Read.ByteOffset.LowPart & (commonExtension->PartitionZeroExtension->DiskGeometry.BytesPerSector - 1);
//
//     if ((irpSp->Parameters.Read.ByteOffset.QuadPart > commonExtension->PartitionLength.QuadPart) ||
//         (irpSp->Parameters.Read.ByteOffset.QuadPart < 0) ||
//         (residualBytes != 0) ||
//         (residualOffset != 0))
//     {
//         NT_ASSERT(residualOffset == 0);
//         status = STATUS_INVALID_PARAMETER;
//     }
//     else
//     {
//         ULONGLONG bytesRemaining = commonExtension->PartitionLength.QuadPart - irpSp->Parameters.Read.ByteOffset.QuadPart;
//
//         if ((ULONGLONG)irpSp->Parameters.Read.Length > bytesRemaining)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         //
//         // This error may be caused by the fact that the drive is not ready.
//         //
//
//         status = ((PDISK_DATA) commonExtension->DriverData)->ReadyStatus;
//
//         if (!NT_SUCCESS(status)) {
//
//             //
//             // Flag this as a user error so that a popup is generated.
//             //
//
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_RW, "DiskReadWriteVerification: ReadyStatus is %lx\n", status));
//
//             if (IoIsErrorUserInduced(status) && Irp->Tail.Overlay.Thread != NULL) {
//                 IoSetHardErrorOrVerifyDevice(Irp, DeviceObject);
//             }
//
//             //
//             // status will keep the current error
//             //
//
//         } else if ((residualBytes == 0) && (residualOffset == 0)) {
//
//             //
//             // This failed because we think the physical disk is too small.
//             // Send it down to the drive and let the hardware decide for
//             // itself.
//             //
//
//             status = STATUS_SUCCESS;
//
//         } else {
//
//             //
//             // Note fastfat depends on this parameter to determine when to
//             // remount due to a sector size change.
//             //
//
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     Irp->IoStatus.Status = status;
//
//     return status;
//
// } // end DiskReadWrite()
//
//
// NTSTATUS
// DiskDetermineMediaTypes(
//     IN PDEVICE_OBJECT Fdo,
//     IN PIRP     Irp,
//     IN UCHAR    MediumType,
//     IN UCHAR    DensityCode,
//     IN BOOLEAN  MediaPresent,
//     IN BOOLEAN  IsWritable
//     )
//
// /*++
//
// Routine Description:
//
//     Determines number of types based on the physical device, validates the user buffer
//     and builds the MEDIA_TYPE information.
//
// Arguments:
//
//     DeviceObject - Pointer to functional device object created by system.
//     Irp - IOCTL_STORAGE_GET_MEDIA_TYPES_EX Irp.
//     MediumType - byte returned in mode data header.
//     DensityCode - byte returned in mode data block descriptor.
//     NumberOfTypes - pointer to be updated based on actual device.
//
// Return Value:
//
//     Status is returned.
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation(Irp);
//
//     PGET_MEDIA_TYPES  mediaTypes = Irp->AssociatedIrp.SystemBuffer;
//     PDEVICE_MEDIA_INFO mediaInfo = &mediaTypes->MediaInfo[0];
//     BOOLEAN deviceMatched = FALSE;
//
//     PAGED_CODE();
//
//     //
//     // this should be checked prior to calling into this routine
//     // as we use the buffer as mediaTypes
//     //
//
//     NT_ASSERT(irpStack->Parameters.DeviceIoControl.OutputBufferLength >=
//            sizeof(GET_MEDIA_TYPES));
//
//     //
//     // Determine if this device is removable or fixed.
//     //
//
//     if (!TEST_FLAG(Fdo->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//         //
//         // Fixed disk.
//         //
//
//         mediaTypes->DeviceType = FILE_DEVICE_DISK;
//         mediaTypes->MediaInfoCount = 1;
//
//         mediaInfo->DeviceSpecific.DiskInfo.Cylinders.QuadPart   = fdoExtension->DiskGeometry.Cylinders.QuadPart;
//         mediaInfo->DeviceSpecific.DiskInfo.MediaType            = (STORAGE_MEDIA_TYPE)FixedMedia;
//         mediaInfo->DeviceSpecific.DiskInfo.TracksPerCylinder    = fdoExtension->DiskGeometry.TracksPerCylinder;
//         mediaInfo->DeviceSpecific.DiskInfo.SectorsPerTrack      = fdoExtension->DiskGeometry.SectorsPerTrack;
//         mediaInfo->DeviceSpecific.DiskInfo.BytesPerSector       = fdoExtension->DiskGeometry.BytesPerSector;
//         mediaInfo->DeviceSpecific.DiskInfo.NumberMediaSides     = 1;
//         mediaInfo->DeviceSpecific.DiskInfo.MediaCharacteristics = (MEDIA_CURRENTLY_MOUNTED | MEDIA_READ_WRITE);
//
//         if (!IsWritable) {
//
//             SET_FLAG(mediaInfo->DeviceSpecific.DiskInfo.MediaCharacteristics,
//                      MEDIA_WRITE_PROTECTED);
//         }
//
//     } else {
//
//         PCCHAR vendorId = (PCCHAR) fdoExtension->DeviceDescriptor + fdoExtension->DeviceDescriptor->VendorIdOffset;
//         PCCHAR productId = (PCCHAR) fdoExtension->DeviceDescriptor + fdoExtension->DeviceDescriptor->ProductIdOffset;
//         PCCHAR productRevision = (PCCHAR) fdoExtension->DeviceDescriptor + fdoExtension->DeviceDescriptor->ProductRevisionOffset;
//         DISK_MEDIA_TYPES_LIST const *mediaListEntry;
//         ULONG  currentMedia;
//         ULONG  i;
//         ULONG  j;
//         ULONG  sizeNeeded;
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                    "DiskDetermineMediaTypes: Vendor %s, Product %s\n",
//                    vendorId,
//                    productId));
//
//
//         //
//         // If there's an entry with such vendorId & ProductId in the DiskMediaTypesExclude list,
//         // this device shouldn't be looked up in the DiskMediaTypes list to determine a medium type.
//         // The exclude table allows to narrow down the set of devices described by the DiskMediaTypes
//         // list (e.g.: DiskMediaTypes says "all HP devices" and DiskMediaTypesExlclude says
//         // "except for HP RDX")
//         //
//
//         for (i = 0; DiskMediaTypesExclude[i].VendorId != NULL; i++) {
//             mediaListEntry = &DiskMediaTypesExclude[i];
//
//             if (strncmp(mediaListEntry->VendorId,vendorId,strlen(mediaListEntry->VendorId))) {
//                 continue;
//             }
//
//             if ((mediaListEntry->ProductId != NULL) &&
//                  strncmp(mediaListEntry->ProductId, productId, strlen(mediaListEntry->ProductId))) {
//                 continue;
//             }
//
//             goto SkipTable;
//         }
//
//         //
//         // Run through the list until we find the entry with a NULL Vendor Id.
//         //
//
//         for (i = 0; DiskMediaTypes[i].VendorId != NULL; i++) {
//
//             mediaListEntry = &DiskMediaTypes[i];
//
//             if (strncmp(mediaListEntry->VendorId,vendorId,strlen(mediaListEntry->VendorId))) {
//                 continue;
//             }
//
//             if ((mediaListEntry->ProductId != NULL) &&
//                  strncmp(mediaListEntry->ProductId, productId, strlen(mediaListEntry->ProductId))) {
//                 continue;
//             }
//
//             if ((mediaListEntry->Revision != NULL) &&
//                  strncmp(mediaListEntry->Revision, productRevision, strlen(mediaListEntry->Revision))) {
//                 continue;
//             }
//
//             deviceMatched = TRUE;
//
//             mediaTypes->DeviceType = FILE_DEVICE_DISK;
//             mediaTypes->MediaInfoCount = mediaListEntry->NumberOfTypes;
//
//             //
//             // Ensure that buffer is large enough.
//             //
//
//             sizeNeeded = FIELD_OFFSET(GET_MEDIA_TYPES, MediaInfo[0]) +
//                          (mediaListEntry->NumberOfTypes *
//                           sizeof(DEVICE_MEDIA_INFO)
//                           );
//
//             if (irpStack->Parameters.DeviceIoControl.OutputBufferLength <
//                 sizeNeeded) {
//
//                 //
//                 // Buffer too small
//                 //
//
//                 Irp->IoStatus.Status = STATUS_BUFFER_TOO_SMALL;
//                 return STATUS_BUFFER_TOO_SMALL;
//             }
//
//             for (j = 0; j < mediaListEntry->NumberOfTypes; j++) {
//
//                 mediaInfo->DeviceSpecific.RemovableDiskInfo.Cylinders.QuadPart = fdoExtension->DiskGeometry.Cylinders.QuadPart;
//                 mediaInfo->DeviceSpecific.RemovableDiskInfo.TracksPerCylinder = fdoExtension->DiskGeometry.TracksPerCylinder;
//                 mediaInfo->DeviceSpecific.RemovableDiskInfo.SectorsPerTrack = fdoExtension->DiskGeometry.SectorsPerTrack;
//                 mediaInfo->DeviceSpecific.RemovableDiskInfo.BytesPerSector = fdoExtension->DiskGeometry.BytesPerSector;
//                 mediaInfo->DeviceSpecific.RemovableDiskInfo.NumberMediaSides = mediaListEntry->NumberOfSides;
//
//                 //
//                 // Set the type.
//                 //
//
//                 mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaType = mediaListEntry->MediaTypes[j];
//
//                 if (mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaType == MO_5_WO) {
//                     mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics = MEDIA_WRITE_ONCE;
//                 } else {
//                     mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics = MEDIA_READ_WRITE;
//                 }
//
//                 //
//                 // Status will either be success, if media is present, or no media.
//                 // It would be optimal to base from density code and medium type, but not all devices
//                 // have values for these fields.
//                 //
//
//                 if (MediaPresent) {
//
//                     //
//                     // The usage of MediumType and DensityCode is device specific, so this may need
//                     // to be extended to further key off of product/vendor ids.
//                     // Currently, the MO units are the only devices that return this information.
//                     //
//
//                     if (MediumType == 2) {
//                         currentMedia = MO_5_WO;
//                     } else if (MediumType == 3) {
//                         currentMedia = MO_5_RW;
//
//                         if (DensityCode == 0x87) {
//
//                             //
//                             // Indicate that the pinnacle 4.6 G media
//                             // is present. Other density codes will default to normal
//                             // RW MO media.
//                             //
//
//                             currentMedia = PINNACLE_APEX_5_RW;
//                         }
//                     } else {
//                         currentMedia = 0;
//                     }
//
//                     if (currentMedia) {
//                         if (mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaType == (STORAGE_MEDIA_TYPE)currentMedia) {
//                             SET_FLAG(mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics, MEDIA_CURRENTLY_MOUNTED);
//                         }
//
//                     } else {
//                         SET_FLAG(mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics, MEDIA_CURRENTLY_MOUNTED);
//                     }
//                 }
//
//                 if (!IsWritable) {
//                     SET_FLAG(mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics, MEDIA_WRITE_PROTECTED);
//                 }
//
//                 //
//                 // Advance to next entry.
//                 //
//
//                 mediaInfo++;
//             }
//         }
//
// SkipTable:
//
//         if (!deviceMatched) {
//
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                        "DiskDetermineMediaTypes: Unknown device. Vendor: %s Product: %s Revision: %s\n",
//                                    vendorId,
//                                    productId,
//                                    productRevision));
//             //
//             // Build an entry for unknown.
//             //
//
//             mediaTypes->DeviceType = FILE_DEVICE_DISK;
//             mediaTypes->MediaInfoCount = 1;
//
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.Cylinders.QuadPart   = fdoExtension->DiskGeometry.Cylinders.QuadPart;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaType            = (STORAGE_MEDIA_TYPE)RemovableMedia;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.TracksPerCylinder    = fdoExtension->DiskGeometry.TracksPerCylinder;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.SectorsPerTrack      = fdoExtension->DiskGeometry.SectorsPerTrack;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.BytesPerSector       = fdoExtension->DiskGeometry.BytesPerSector;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.NumberMediaSides     = 1;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics = MEDIA_READ_WRITE;
//
//             if (MediaPresent) {
//
//                 SET_FLAG(mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics, MEDIA_CURRENTLY_MOUNTED);
//             }
//
//             if (!IsWritable) {
//
//                 SET_FLAG(mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics, MEDIA_WRITE_PROTECTED);
//             }
//         }
//     }
//
//     Irp->IoStatus.Information =
//         FIELD_OFFSET(GET_MEDIA_TYPES, MediaInfo[0]) +
//         (mediaTypes->MediaInfoCount * sizeof(DEVICE_MEDIA_INFO));
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskDeviceControl(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     I/O system entry for device controls to SCSI disks.
//
// Arguments:
//
//     Fdo - Pointer to functional device object created by system.
//     Irp - IRP involved.
//
// Return Value:
//
//     Status is returned.
//
// --*/
//
// {
//     PIO_STACK_LOCATION  irpStack = IoGetCurrentIrpStackLocation(Irp);
//     NTSTATUS            status = STATUS_SUCCESS;
//     ULONG               ioctlCode;
//
//     NT_ASSERT(DeviceObject != NULL);
//
//     Irp->IoStatus.Information = 0;
//     ioctlCode = irpStack->Parameters.DeviceIoControl.IoControlCode;
//
//     TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL, "DiskDeviceControl: Received IOCTL 0x%X for device %p through IRP %p\n",
//                 ioctlCode, DeviceObject, Irp));
//
//
//     switch (ioctlCode) {
//
//         case IOCTL_DISK_GET_CACHE_INFORMATION: {
//             status = DiskIoctlGetCacheInformation(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_SET_CACHE_INFORMATION: {
//             status = DiskIoctlSetCacheInformation(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_GET_CACHE_SETTING: {
//             status = DiskIoctlGetCacheSetting(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_SET_CACHE_SETTING: {
//             status = DiskIoctlSetCacheSetting(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_GET_DRIVE_GEOMETRY: {
//             status = DiskIoctlGetDriveGeometry(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: {
//             status = DiskIoctlGetDriveGeometryEx( DeviceObject, Irp );
//             break;
//         }
//
//         case IOCTL_DISK_VERIFY: {
//             status = DiskIoctlVerify(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_GET_LENGTH_INFO: {
//             status = DiskIoctlGetLengthInfo(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_IS_WRITABLE: {
//             status = DiskIoctlIsWritable(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_UPDATE_DRIVE_SIZE: {
//             status = DiskIoctlUpdateDriveSize(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_REASSIGN_BLOCKS: {
//             status = DiskIoctlReassignBlocks(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_REASSIGN_BLOCKS_EX: {
//             status = DiskIoctlReassignBlocksEx(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_INTERNAL_SET_VERIFY: {
//             status = DiskIoctlSetVerify(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_DISK_INTERNAL_CLEAR_VERIFY: {
//             status = DiskIoctlClearVerify(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_STORAGE_GET_MEDIA_TYPES_EX: {
//             status = DiskIoctlGetMediaTypesEx(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_STORAGE_PREDICT_FAILURE : {
//             status = DiskIoctlPredictFailure(DeviceObject, Irp);
//             break;
//         }
//
//         #if (NTDDI_VERSION >= NTDDI_WINBLUE)
//         case IOCTL_STORAGE_FAILURE_PREDICTION_CONFIG : {
//             status = DiskIoctlEnableFailurePrediction(DeviceObject, Irp);
//             break;
//         }
//         #endif
//
//         case SMART_GET_VERSION: {
//             status = DiskIoctlSmartGetVersion(DeviceObject, Irp);
//             break;
//         }
//
//         case SMART_RCV_DRIVE_DATA: {
//             status = DiskIoctlSmartReceiveDriveData(DeviceObject, Irp);
//             break;
//         }
//
//         case SMART_SEND_DRIVE_COMMAND: {
//             status = DiskIoctlSmartSendDriveCommand(DeviceObject, Irp);
//             break;
//         }
//
//         case IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS:
//         case IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS_ADMIN: {
//             status = DiskIoctlGetVolumeDiskExtents(DeviceObject, Irp);
//             break;
//         }
//
//         default: {
//
//             //
//             // Pass the request to the common device control routine.
//             //
//             return(ClassDeviceControl(DeviceObject, Irp));
//             break;
//         }
//     } // end switch
//
//     if (!NT_SUCCESS(status)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskDeviceControl: IOCTL 0x%X to device %p failed with error 0x%X\n",
//                     ioctlCode, DeviceObject, status));
//         if (IoIsErrorUserInduced(status) &&
//             (Irp->Tail.Overlay.Thread != NULL)) {
//             IoSetHardErrorOrVerifyDevice(Irp, DeviceObject);
//         }
//     }
//
//     //
//     // DiskIoctlVerify() (IOCTL_DISK_VERIFY) function returns STATUS_PENDING
//     // and completes the IRP in the work item. Do not touch or complete
//     // the IRP if STATUS_PENDING is returned.
//     //
//
//     if (status != STATUS_PENDING) {
//
//
//         Irp->IoStatus.Status = status;
//         ClassReleaseRemoveLock(DeviceObject, Irp);
//         ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//     }
//
//     return(status);
// } // end DiskDeviceControl()
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskShutdownFlush(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the handler for shutdown and flush requests. It sends
//     down a synch cache command to the device if its cache is enabled.  If
//     the request is a  shutdown and the media is removable,  it sends down
//     an unlock request
//
//     Finally,  an SRB_FUNCTION_SHUTDOWN or SRB_FUNCTION_FLUSH is sent down
//     the stack
//
// Arguments:
//
//     DeviceObject - The device object processing the request
//     Irp - The shutdown | flush request being serviced
//
// Return Value:
//
//     STATUS_PENDING if successful, an error code otherwise
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = commonExtension->PartitionZeroExtension;
//     PDISK_DATA diskData = (PDISK_DATA) commonExtension->DriverData;
//     PIO_STACK_LOCATION irpStack;
//     NTSTATUS status = STATUS_SUCCESS;
//     ULONG srbSize;
//     PSCSI_REQUEST_BLOCK srb;
//     PSTORAGE_REQUEST_BLOCK srbEx = NULL;
//     PSTOR_ADDR_BTL8 storAddrBtl8 = NULL;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16 = NULL;
//     PCDB cdb;
//     KIRQL irql;
//
//     //
//     // Flush requests are combined and need to be handled in a special manner
//     //
//
//     irpStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (irpStack->MajorFunction == IRP_MJ_FLUSH_BUFFERS) {
//
//         if (TEST_FLAG(fdoExtension->DeviceFlags, DEV_POWER_PROTECTED)) {
//
//             //
//             // We've been assured that both the disk
//             // and adapter caches are battery-backed
//             //
//
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             ClassReleaseRemoveLock(DeviceObject, Irp);
//             ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//             return STATUS_SUCCESS;
//         }
//
//         KeAcquireSpinLock(&diskData->FlushContext.Spinlock, &irql);
//
//         TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_SCSI, "DiskShutdownFlush: IRP %p flags = 0x%x\n", Irp, irpStack->Flags));
//
//         //
//         // This request will most likely be completed asynchronously
//         //
//         IoMarkIrpPending(Irp);
//
//         //
//         // Look to see if a flush is in progress
//         //
//
//         if (diskData->FlushContext.CurrIrp != NULL) {
//
//             //
//             // There is an outstanding flush. Queue this
//             // request to the group that is next in line
//             //
//
//             if (diskData->FlushContext.NextIrp != NULL) {
//
//                 #if DBG
//                     diskData->FlushContext.DbgTagCount++;
//                 #endif
//
//                 InsertTailList(&diskData->FlushContext.NextList, &Irp->Tail.Overlay.ListEntry);
//
//                 KeReleaseSpinLock(&diskData->FlushContext.Spinlock, irql);
//
//                 //
//                 // This request will be completed by its representative
//                 //
//
//             } else {
//
//                 #if DBG
//                     if (diskData->FlushContext.DbgTagCount < 64) {
//
//                         diskData->FlushContext.DbgRefCount[diskData->FlushContext.DbgTagCount]++;
//                     }
//
//                     diskData->FlushContext.DbgSavCount += diskData->FlushContext.DbgTagCount;
//                     diskData->FlushContext.DbgTagCount  = 0;
//                 #endif
//
//                 diskData->FlushContext.NextIrp = Irp;
//                 NT_ASSERT(IsListEmpty(&diskData->FlushContext.NextList));
//
//
//                 KeReleaseSpinLock(&diskData->FlushContext.Spinlock, irql);
//
//
//                     TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_SCSI, "DiskShutdownFlush: waiting for event\n"));
//
//                     //
//                     // Wait for the outstanding flush to complete
//                     //
//                     KeWaitForSingleObject(&diskData->FlushContext.Event, Executive, KernelMode, FALSE, NULL);
//
//                     TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_SCSI, "DiskShutdownFlush: event signal\n"));
//
//                     //
//                     // Make this group the outstanding one and free up the next slot
//                     //
//
//                     KeAcquireSpinLock(&diskData->FlushContext.Spinlock, &irql);
//
//                     NT_ASSERT(IsListEmpty(&diskData->FlushContext.CurrList));
//
//                     while (!IsListEmpty(&diskData->FlushContext.NextList)) {
//
//                         PLIST_ENTRY listEntry = RemoveHeadList(&diskData->FlushContext.NextList);
//                         InsertTailList(&diskData->FlushContext.CurrList, listEntry);
//                     }
//
// #ifndef __REACTOS__
//                     // ReactOS hits this assert, because CurrIrp can already be freed at this point
//                     // and it's possible that NextIrp has the same pointer value
//                     NT_ASSERT(diskData->FlushContext.CurrIrp != diskData->FlushContext.NextIrp);
// #endif
//                     diskData->FlushContext.CurrIrp = diskData->FlushContext.NextIrp;
//                     diskData->FlushContext.NextIrp = NULL;
//
//                     KeReleaseSpinLock(&diskData->FlushContext.Spinlock, irql);
//
//                     //
//                     // Send this request down to the device
//                     //
//                     DiskFlushDispatch(DeviceObject, &diskData->FlushContext);
//             }
//
//         } else {
//
//             diskData->FlushContext.CurrIrp = Irp;
//             NT_ASSERT(IsListEmpty(&diskData->FlushContext.CurrList));
//
//             NT_ASSERT(diskData->FlushContext.NextIrp == NULL);
//             NT_ASSERT(IsListEmpty(&diskData->FlushContext.NextList));
//
//
//             KeReleaseSpinLock(&diskData->FlushContext.Spinlock, irql);
//
//                 DiskFlushDispatch(DeviceObject, &diskData->FlushContext);
//         }
//
//     } else {
//
//         //
//         // Allocate SCSI request block.
//         //
//
//         if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//             srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//         } else {
//             srbSize = sizeof(SCSI_REQUEST_BLOCK);
//         }
//
//         srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                     srbSize,
//                                     DISK_TAG_SRB);
//         if (srb == NULL) {
//
//             //
//             // Set the status and complete the request.
//             //
//
//             Irp->IoStatus.Status = STATUS_INSUFFICIENT_RESOURCES;
//             ClassReleaseRemoveLock(DeviceObject, Irp);
//             ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//             return(STATUS_INSUFFICIENT_RESOURCES);
//         }
//
//         RtlZeroMemory(srb, srbSize);
//         if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//
//             srbEx = (PSTORAGE_REQUEST_BLOCK)srb;
//
//             //
//             // Set up STORAGE_REQUEST_BLOCK fields
//             //
//
//             srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//             srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//             srbEx->Signature = SRB_SIGNATURE;
//             srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//             srbEx->SrbLength = srbSize;
//             srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//             srbEx->RequestPriority = IoGetIoPriorityHint(Irp);
//             srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//             srbEx->NumSrbExData = 1;
//
//             // Set timeout value and mark the request as not being a tagged request.
//             srbEx->TimeOutValue = fdoExtension->TimeOutValue * 4;
//             srbEx->RequestTag = SP_UNTAGGED;
//             srbEx->RequestAttribute = SRB_SIMPLE_TAG_REQUEST;
//             srbEx->SrbFlags = fdoExtension->SrbFlags;
//
//             //
//             // Set up address fields
//             //
//
//             storAddrBtl8 = (PSTOR_ADDR_BTL8) ((PUCHAR)srbEx + srbEx->AddressOffset);
//             storAddrBtl8->Type = STOR_ADDRESS_TYPE_BTL8;
//             storAddrBtl8->AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
//
//             //
//             // Set up SCSI SRB extended data fields
//             //
//
//             srbEx->SrbExDataOffset[0] = sizeof(STORAGE_REQUEST_BLOCK) +
//                 sizeof(STOR_ADDR_BTL8);
//             if ((srbEx->SrbExDataOffset[0] + sizeof(SRBEX_DATA_SCSI_CDB16)) <= srbEx->SrbLength) {
//                 srbExDataCdb16 = (PSRBEX_DATA_SCSI_CDB16)((PUCHAR)srbEx + srbEx->SrbExDataOffset[0]);
//                 srbExDataCdb16->Type = SrbExDataTypeScsiCdb16;
//                 srbExDataCdb16->Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
//
//                 cdb = (PCDB)srbExDataCdb16->Cdb;
//             } else {
//                 // Should not happen
//                 NT_ASSERT(FALSE);
//
//                 //
//                 // Set the status and complete the request.
//                 //
//
//                 Irp->IoStatus.Status = STATUS_INTERNAL_ERROR;
//                 ClassReleaseRemoveLock(DeviceObject, Irp);
//                 ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//                 return(STATUS_INTERNAL_ERROR);
//             }
//
//         } else {
//
//             //
//             // Write length to SRB.
//             //
//
//             srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//
//             //
//             // Set timeout value and mark the request as not being a tagged request.
//             //
//
//             srb->TimeOutValue = fdoExtension->TimeOutValue * 4;
//             srb->QueueTag = SP_UNTAGGED;
//             srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//             srb->SrbFlags = fdoExtension->SrbFlags;
//             srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//
//             cdb = (PCDB)srb->Cdb;
//         }
//
//         //
//         // If the write cache is enabled then send a synchronize cache request.
//         //
//
//         if (TEST_FLAG(fdoExtension->DeviceFlags, DEV_WRITE_CACHE)) {
//
//             if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//                 srbExDataCdb16->CdbLength = 10;
//             } else {
//                 srb->CdbLength = 10;
//             }
//
//             cdb->CDB10.OperationCode = SCSIOP_SYNCHRONIZE_CACHE;
//
//             status = ClassSendSrbSynchronous(DeviceObject,
//                                              srb,
//                                              NULL,
//                                              0,
//                                              TRUE);
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskShutdownFlush: Synchonize cache sent. Status = %lx\n", status));
//         }
//
//         //
//         // Unlock the device if it contains removable media
//         //
//
//         if (TEST_FLAG(DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA))
//         {
//
//             if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//
//                 //
//                 // Reinitialize status fields to 0 in case there was a previous request
//                 //
//
//                 srbEx->SrbStatus = 0;
//                 srbExDataCdb16->ScsiStatus = 0;
//
//                 srbExDataCdb16->CdbLength = 6;
//
//                 //
//                 // Set timeout value
//                 //
//
//                 srbEx->TimeOutValue = fdoExtension->TimeOutValue;
//
//             } else {
//
//                 //
//                 // Reinitialize status fields to 0 in case there was a previous request
//                 //
//
//                 srb->SrbStatus = 0;
//                 srb->ScsiStatus = 0;
//
//                 srb->CdbLength = 6;
//
//                 //
//                 // Set timeout value.
//                 //
//
//                 srb->TimeOutValue = fdoExtension->TimeOutValue;
//             }
//
//             cdb->MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
//             cdb->MEDIA_REMOVAL.Prevent = FALSE;
//
//             status = ClassSendSrbSynchronous(DeviceObject,
//                                              srb,
//                                              NULL,
//                                              0,
//                                              TRUE);
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskShutdownFlush: Unlock device request sent. Status = %lx\n", status));
//         }
//
//         //
//         // Set up a SHUTDOWN SRB
//         //
//
//         if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//             srbEx->NumSrbExData = 0;
//             srbEx->SrbExDataOffset[0] = 0;
//             srbEx->SrbFunction = SRB_FUNCTION_SHUTDOWN;
//             srbEx->OriginalRequest = Irp;
//             srbEx->SrbLength = CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE;
//             srbEx->SrbStatus = 0;
//         } else {
//             srb->CdbLength = 0;
//             srb->Function = SRB_FUNCTION_SHUTDOWN;
//             srb->SrbStatus = 0;
//             srb->OriginalRequest = Irp;
//         }
//
//         //
//         // Set the retry count to zero.
//         //
//
//         irpStack->Parameters.Others.Argument4 = (PVOID) 0;
//
//         //
//         // Set up IoCompletion routine address.
//         //
//
//         IoSetCompletionRoutine(Irp, ClassIoComplete, srb, TRUE, TRUE, TRUE);
//
//         //
//         // Get next stack location and
//         // set major function code.
//         //
//
//         irpStack = IoGetNextIrpStackLocation(Irp);
//
//         irpStack->MajorFunction = IRP_MJ_SCSI;
//
//         //
//         // Set up SRB for execute scsi request.
//         // Save SRB address in next stack for port driver.
//         //
//
//         irpStack->Parameters.Scsi.Srb = srb;
//
//         //
//         // Call the port driver to process the request.
//         //
//
//         IoMarkIrpPending(Irp);
//         IoCallDriver(commonExtension->LowerDeviceObject, Irp);
//     }
//
//     return STATUS_PENDING;
// }
//
//
// VOID
// DiskFlushDispatch(
//     IN PDEVICE_OBJECT Fdo,
//     IN PDISK_GROUP_CONTEXT FlushContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the handler for flush requests. It sends down a synch
//     cache command to the device if its cache is enabled. This is followed
//     by an SRB_FUNCTION_FLUSH
//
// Arguments:
//
//     Fdo - The device object processing the flush request
//     FlushContext - The flush group context
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = Fdo->DeviceExtension;
//     PSCSI_REQUEST_BLOCK srb = &FlushContext->Srb.Srb;
//     PSTORAGE_REQUEST_BLOCK srbEx = &FlushContext->Srb.SrbEx;
//     PIO_STACK_LOCATION  irpSp = NULL;
//     PSTOR_ADDR_BTL8 storAddrBtl8;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16;
//     NTSTATUS SyncCacheStatus = STATUS_SUCCESS;
//
//     //
//     // Fill in the srb fields appropriately
//     //
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         RtlZeroMemory(srbEx, sizeof(FlushContext->Srb.SrbExBuffer));
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = sizeof(FlushContext->Srb.SrbExBuffer);
//         srbEx->RequestPriority = IoGetIoPriorityHint(FlushContext->CurrIrp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->TimeOutValue = fdoExt->TimeOutValue * 4;
//         srbEx->RequestTag = SP_UNTAGGED;
//         srbEx->RequestAttribute  = SRB_SIMPLE_TAG_REQUEST;
//         srbEx->SrbFlags = fdoExt->SrbFlags;
//
//         //
//         // Set up address fields
//         //
//
//         storAddrBtl8 = (PSTOR_ADDR_BTL8) ((PUCHAR)srbEx + srbEx->AddressOffset);
//         storAddrBtl8->Type = STOR_ADDRESS_TYPE_BTL8;
//         storAddrBtl8->AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
//
//     } else {
//         RtlZeroMemory(srb, SCSI_REQUEST_BLOCK_SIZE);
//
//         srb->Length       = SCSI_REQUEST_BLOCK_SIZE;
//         srb->TimeOutValue = fdoExt->TimeOutValue * 4;
//         srb->QueueTag     = SP_UNTAGGED;
//         srb->QueueAction  = SRB_SIMPLE_TAG_REQUEST;
//         srb->SrbFlags     = fdoExt->SrbFlags;
//     }
//
//     //
//     // If write caching is enabled then send down a synchronize cache request
//     //
//     if (TEST_FLAG(fdoExt->DeviceFlags, DEV_WRITE_CACHE))
//     {
//
//         if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//             srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//             srbEx->NumSrbExData = 1;
//
//             //
//             // Set up SCSI SRB extended data fields
//             //
//
//             srbEx->SrbExDataOffset[0] = sizeof(STORAGE_REQUEST_BLOCK) +
//                 sizeof(STOR_ADDR_BTL8);
//             if ((srbEx->SrbExDataOffset[0] + sizeof(SRBEX_DATA_SCSI_CDB16)) <= srbEx->SrbLength) {
//                 srbExDataCdb16 = (PSRBEX_DATA_SCSI_CDB16)((PUCHAR)srbEx + srbEx->SrbExDataOffset[0]);
//                 srbExDataCdb16->Type = SrbExDataTypeScsiCdb16;
//                 srbExDataCdb16->Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
//                 srbExDataCdb16->CdbLength = 10;
//                 srbExDataCdb16->Cdb[0] = SCSIOP_SYNCHRONIZE_CACHE;
//             } else {
//                 // Should not happen
//                 NT_ASSERT(FALSE);
//                 return;
//             }
//
//         } else {
//             srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//             srb->CdbLength = 10;
//             srb->Cdb[0] = SCSIOP_SYNCHRONIZE_CACHE;
//         }
//
//         TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_SCSI, "DiskFlushDispatch: sending sync cache\n"));
//
//         SyncCacheStatus = ClassSendSrbSynchronous(Fdo, srb, NULL, 0, TRUE);
//     }
//
//     //
//     // Set up a FLUSH SRB
//     //
//     if (fdoExt->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx->SrbFunction = SRB_FUNCTION_FLUSH;
//         srbEx->NumSrbExData = 0;
//         srbEx->SrbExDataOffset[0] = 0;
//         srbEx->OriginalRequest = FlushContext->CurrIrp;
//         srbEx->SrbStatus = 0;
//
//         //
//         // Make sure that this srb does not get freed
//         //
//         SET_FLAG(srbEx->SrbFlags, SRB_CLASS_FLAGS_PERSISTANT);
//
//    } else {
//         srb->Function  = SRB_FUNCTION_FLUSH;
//         srb->CdbLength = 0;
//         srb->OriginalRequest = FlushContext->CurrIrp;
//         srb->SrbStatus = 0;
//         srb->ScsiStatus = 0;
//
//         //
//         // Make sure that this srb does not get freed
//         //
//         SET_FLAG(srb->SrbFlags, SRB_CLASS_FLAGS_PERSISTANT);
//     }
//
//     //
//     // Make sure that this request does not get retried
//     //
//     irpSp = IoGetCurrentIrpStackLocation(FlushContext->CurrIrp);
//
//     irpSp->Parameters.Others.Argument4 = (PVOID) 0;
//
//     //
//     // Fill in the irp fields appropriately
//     //
//     irpSp = IoGetNextIrpStackLocation(FlushContext->CurrIrp);
//
//     irpSp->MajorFunction       = IRP_MJ_SCSI;
//     irpSp->Parameters.Scsi.Srb = srb;
//
//     IoSetCompletionRoutine(FlushContext->CurrIrp, DiskFlushComplete, (PVOID)(ULONG_PTR)SyncCacheStatus, TRUE, TRUE, TRUE);
//
//     TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_SCSI, "DiskFlushDispatch: sending srb flush on irp %p\n", FlushContext->CurrIrp));
//
//     //
//     // Send down the flush request
//     //
//     IoCallDriver(((PCOMMON_DEVICE_EXTENSION)fdoExt)->LowerDeviceObject, FlushContext->CurrIrp);
// }
//
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFlushComplete(
//     IN PDEVICE_OBJECT Fdo,
//     IN PIRP Irp,
//     IN PVOID Context
//     )
//
// /*++
//
// Routine Description:
//
//     This completion routine is a wrapper around ClassIoComplete. It
//     will complete all the flush requests that are tagged to it, set
//     an event to signal the next group to proceed and return
//
// Arguments:
//
//     Fdo - The device object which requested the completion routine
//     Irp - The irp that is being completed
//     Context - If disk had write cache enabled and SYNC CACHE command was sent as 1st part of FLUSH processing
//                    then context must carry the completion status of SYNC CACHE request,
//               else context must be set to STATUS_SUCCESS.
//
// Return Value:
//
//     STATUS_SUCCESS if successful, an error code otherwise
//
// --*/
//
// {
//     PDISK_GROUP_CONTEXT FlushContext;
//     NTSTATUS status;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
//     PDISK_DATA diskData;
// #ifdef _MSC_VER
//     #pragma warning(suppress:4311) // pointer truncation from 'PVOID' to 'NTSTATUS'
// #endif
//     NTSTATUS SyncCacheStatus = (NTSTATUS)(ULONG_PTR)Context;
//
//     TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL, "DiskFlushComplete: %p %p\n", Fdo, Irp));
//
//     //
//     // Get the flush context from the device extension
//     //
//     fdoExt = (PFUNCTIONAL_DEVICE_EXTENSION)Fdo->DeviceExtension;
//     diskData = (PDISK_DATA)fdoExt->CommonExtension.DriverData;
//     NT_ASSERT(diskData != NULL);
//     _Analysis_assume_(diskData != NULL);
//
//     FlushContext = &diskData->FlushContext;
//
//     //
//     // Make sure everything is in order
//     //
//     NT_ASSERT(Irp == FlushContext->CurrIrp);
//
//     TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_SCSI, "DiskFlushComplete: completing irp %p\n", Irp));
//     status = ClassIoComplete(Fdo, Irp, &FlushContext->Srb.Srb);
//
//     //
//     // Make sure that ClassIoComplete did not decide to retry this request
//     //
//     NT_ASSERT(status != STATUS_MORE_PROCESSING_REQUIRED);
//
//     //
//     // If sync cache failed earlier, final status of the flush request needs to be failure
//     // even if SRB_FUNCTION_FLUSH srb request succeeded
//     //
//     if (NT_SUCCESS(status) &&
//         (!NT_SUCCESS(SyncCacheStatus))) {
//         Irp->IoStatus.Status = status = SyncCacheStatus;
//     }
//
//     //
//     // Complete the flush requests tagged to this one
//     //
//
//     while (!IsListEmpty(&FlushContext->CurrList)) {
//
//         PLIST_ENTRY listEntry = RemoveHeadList(&FlushContext->CurrList);
//         PIRP tempIrp = CONTAINING_RECORD(listEntry, IRP, Tail.Overlay.ListEntry);
//
//         InitializeListHead(&tempIrp->Tail.Overlay.ListEntry);
//         tempIrp->IoStatus = Irp->IoStatus;
//
//         ClassReleaseRemoveLock(Fdo, tempIrp);
//         ClassCompleteRequest(Fdo, tempIrp, IO_NO_INCREMENT);
//     }
//
//
//         //
//         // Notify the next group's representative that it may go ahead now
//         //
//         KeSetEvent(&FlushContext->Event, IO_NO_INCREMENT, FALSE);
//
//
//     TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_SCSI, "DiskFlushComplete: irp %p status = 0x%x\n", Irp, status));
//
//     return status;
// }
//
//
//
// NTSTATUS
// DiskModeSelect(
//     IN PDEVICE_OBJECT Fdo,
//      PCHAR ModeSelectBuffer,
//     IN ULONG Length,
//     IN BOOLEAN SavePage
//     )
//
// /*++
//
// Routine Description:
//
//     This routine sends a mode select command.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     ModeSelectBuffer - Supplies a buffer containing the page data.
//
//     Length - Supplies the length in bytes of the mode select buffer.
//
//     SavePage - Indicates that parameters should be written to disk.
//
// Return Value:
//
//     Length of the transferred data is returned.
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PCDB cdb;
//     SCSI_REQUEST_BLOCK srb = {0};
//     ULONG retries = 1;
//     ULONG length2;
//     NTSTATUS status;
//     PULONG buffer;
//     PMODE_PARAMETER_BLOCK blockDescriptor;
//     UCHAR srbExBuffer[CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE] = {0};
//     PSTORAGE_REQUEST_BLOCK srbEx = (PSTORAGE_REQUEST_BLOCK)srbExBuffer;
//     PSTOR_ADDR_BTL8 storAddrBtl8;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16;
//     PSCSI_REQUEST_BLOCK srbPtr;
//
//     PAGED_CODE();
//
//     //
//     // Check whether block length is available
//     //
//
//     if (fdoExtension->DiskGeometry.BytesPerSector == 0) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskModeSelect: Block length is not available. Unable to send mode select\n"));
//         NT_ASSERT(fdoExtension->DiskGeometry.BytesPerSector != 0);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//
//
//     length2 = Length + sizeof(MODE_PARAMETER_HEADER) + sizeof(MODE_PARAMETER_BLOCK);
//
//     //
//     // Allocate buffer for mode select header, block descriptor, and mode page.
//     //
//
//     buffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                    length2,
//                                    DISK_TAG_MODE_DATA);
//
//     if (buffer == NULL) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(buffer, length2);
//
//     //
//     // Set length in header to size of mode page.
//     //
//
//     ((PMODE_PARAMETER_HEADER)buffer)->BlockDescriptorLength = sizeof(MODE_PARAMETER_BLOCK);
//
//     blockDescriptor = (PMODE_PARAMETER_BLOCK)(buffer + 1);
//
//     //
//     // Set block length from the cached disk geometry
//     //
//
//     blockDescriptor->BlockLength[2] = (UCHAR) (fdoExtension->DiskGeometry.BytesPerSector >> 16);
//     blockDescriptor->BlockLength[1] = (UCHAR) (fdoExtension->DiskGeometry.BytesPerSector >> 8);
//     blockDescriptor->BlockLength[0] = (UCHAR) (fdoExtension->DiskGeometry.BytesPerSector);
//
//     //
//     // Copy mode page to buffer.
//     //
//
//     RtlCopyMemory(buffer + 3, ModeSelectBuffer, Length);
//
//     //
//     // Build the MODE SELECT CDB.
//     //
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
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
//         srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//         srbEx->RequestPriority = IoPriorityNormal;
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->NumSrbExData = 1;
//
//         // Set timeout value from device extension.
//         srbEx->TimeOutValue = fdoExtension->TimeOutValue * 2;
//
//        //
//        // Set up address fields
//        //
//
//        storAddrBtl8 = (PSTOR_ADDR_BTL8) ((PUCHAR)srbEx + srbEx->AddressOffset);
//        storAddrBtl8->Type = STOR_ADDRESS_TYPE_BTL8;
//        storAddrBtl8->AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
//
//        //
//        // Set up SCSI SRB extended data fields
//        //
//
//        srbEx->SrbExDataOffset[0] = sizeof(STORAGE_REQUEST_BLOCK) +
//            sizeof(STOR_ADDR_BTL8);
//        if ((srbEx->SrbExDataOffset[0] + sizeof(SRBEX_DATA_SCSI_CDB16)) <= srbEx->SrbLength) {
//            srbExDataCdb16 = (PSRBEX_DATA_SCSI_CDB16)((PUCHAR)srbEx + srbEx->SrbExDataOffset[0]);
//            srbExDataCdb16->Type = SrbExDataTypeScsiCdb16;
//            srbExDataCdb16->Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
//            srbExDataCdb16->CdbLength = 6;
//
//            cdb = (PCDB)srbExDataCdb16->Cdb;
//        } else {
//            // Should not happen
//            NT_ASSERT(FALSE);
//
//            FREE_POOL(buffer);
//            TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskModeSelect: Insufficient extended SRB size\n"));
//            return STATUS_INTERNAL_ERROR;
//        }
//
//        srbPtr = (PSCSI_REQUEST_BLOCK)srbEx;
//
//     } else {
//
//         srb.CdbLength = 6;
//         cdb = (PCDB)srb.Cdb;
//
//         //
//         // Set timeout value from device extension.
//         //
//
//         srb.TimeOutValue = fdoExtension->TimeOutValue * 2;
//
//         srbPtr = &srb;
//     }
//
//     cdb->MODE_SELECT.OperationCode = SCSIOP_MODE_SELECT;
//     cdb->MODE_SELECT.SPBit = SavePage;
//     cdb->MODE_SELECT.PFBit = 1;
//     cdb->MODE_SELECT.ParameterListLength = (UCHAR)(length2);
//
// Retry:
//
//     status = ClassSendSrbSynchronous(Fdo,
//                                      srbPtr,
//                                      buffer,
//                                      length2,
//                                      TRUE);
//
//     if (status == STATUS_VERIFY_REQUIRED) {
//
//         //
//         // Routine ClassSendSrbSynchronous does not retry requests returned with
//         // this status.
//         //
//
//         if (retries--) {
//
//             //
//             // Retry request.
//             //
//
//             goto Retry;
//         }
//
//     } else if (SRB_STATUS(srbPtr->SrbStatus) == SRB_STATUS_DATA_OVERRUN) {
//         status = STATUS_SUCCESS;
//     }
//
//     FREE_POOL(buffer);
//
//     return status;
// } // end DiskModeSelect()
//
//
// //
// // This routine is structured as a work-item routine
// //
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DisableWriteCache(
//     IN PDEVICE_OBJECT Fdo,
//     IN PVOID Context
//     )
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = (PFUNCTIONAL_DEVICE_EXTENSION)Fdo->DeviceExtension;
//     DISK_CACHE_INFORMATION cacheInfo = { 0 };
//     NTSTATUS status;
//     PIO_WORKITEM WorkItem = (PIO_WORKITEM)Context;
//
//     PAGED_CODE();
//
//     NT_ASSERT(WorkItem != NULL);
//     _Analysis_assume_(WorkItem != NULL);
//
//     status = DiskGetCacheInformation(fdoExtension, &cacheInfo);
//
//     if (NT_SUCCESS(status) && (cacheInfo.WriteCacheEnabled == TRUE)) {
//
//         cacheInfo.WriteCacheEnabled = FALSE;
//
//         DiskSetCacheInformation(fdoExtension, &cacheInfo);
//     }
//
//     IoFreeWorkItem(WorkItem);
// }
//
//
// //
// // This routine is structured as a work-item routine
// //
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskIoctlVerifyThread(
//     IN PDEVICE_OBJECT Fdo,
//     IN PVOID Context
//     )
// {
//     PDISK_VERIFY_WORKITEM_CONTEXT WorkContext = (PDISK_VERIFY_WORKITEM_CONTEXT)Context;
//     PIRP Irp = NULL;
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension = (PFUNCTIONAL_DEVICE_EXTENSION)Fdo->DeviceExtension;
//     PDISK_DATA DiskData = (PDISK_DATA)FdoExtension->CommonExtension.DriverData;
//     PVERIFY_INFORMATION verifyInfo = NULL;
//     PSCSI_REQUEST_BLOCK Srb = NULL;
//     PCDB Cdb = NULL;
//     LARGE_INTEGER byteOffset;
//     LARGE_INTEGER sectorOffset;
//     ULONG sectorCount;
//     NTSTATUS status = STATUS_SUCCESS;
//     PSTORAGE_REQUEST_BLOCK srbEx = NULL;
//     PSTOR_ADDR_BTL8 storAddrBtl8 = NULL;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16 = NULL;
//
//     PAGED_CODE();
//
//     NT_ASSERT(WorkContext != NULL);
//     _Analysis_assume_(WorkContext != NULL);
//
//     Srb = WorkContext->Srb;
//     Irp = WorkContext->Irp;
//     verifyInfo = (PVERIFY_INFORMATION)Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     // We don't need to hold on to this memory as
//     // the following operation may take some time
//     //
//
//     IoFreeWorkItem(WorkContext->WorkItem);
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlVerifyThread: Spliting up the request\n"));
//
//     //
//     // Add disk offset to starting the sector
//     //
//
//     byteOffset.QuadPart = FdoExtension->CommonExtension.StartingOffset.QuadPart +
//                           verifyInfo->StartingOffset.QuadPart;
//
//     //
//     // Convert byte offset to the sector offset
//     //
//
//     sectorOffset.QuadPart = byteOffset.QuadPart >> FdoExtension->SectorShift;
//
//     //
//     // Convert byte count to sector count.
//     //
//
//     sectorCount = verifyInfo->Length >> FdoExtension->SectorShift;
//
//     //
//     // Make sure  that all previous verify requests have indeed completed
//     // This greatly reduces the possibility of a Denial-of-Service attack
//     //
//
//     KeWaitForMutexObject(&DiskData->VerifyMutex,
//                          Executive,
//                          KernelMode,
//                          FALSE,
//                          NULL);
//
//     //
//     // Initialize SCSI SRB for a verify CDB
//     //
//     if (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         RtlZeroMemory(Srb, CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE);
//         srbEx = (PSTORAGE_REQUEST_BLOCK)Srb;
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//         srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//         srbEx->RequestPriority = IoGetIoPriorityHint(Irp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->NumSrbExData = 1;
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
//                sizeof(STOR_ADDR_BTL8);
//         if ((srbEx->SrbExDataOffset[0] + sizeof(SRBEX_DATA_SCSI_CDB16)) <= srbEx->SrbLength) {
//             srbExDataCdb16 = (PSRBEX_DATA_SCSI_CDB16)((PUCHAR)srbEx + srbEx->SrbExDataOffset[0]);
//             srbExDataCdb16->Type = SrbExDataTypeScsiCdb16;
//             srbExDataCdb16->Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
//
//             Cdb = (PCDB)srbExDataCdb16->Cdb;
//             if (TEST_FLAG(FdoExtension->DeviceFlags, DEV_USE_16BYTE_CDB)) {
//                 srbExDataCdb16->CdbLength = 16;
//                 Cdb->CDB16.OperationCode = SCSIOP_VERIFY16;
//             } else {
//                 srbExDataCdb16->CdbLength = 10;
//                 Cdb->CDB10.OperationCode = SCSIOP_VERIFY;
//             }
//         } else {
//             // Should not happen
//             NT_ASSERT(FALSE);
//
//             FREE_POOL(Srb);
//             FREE_POOL(WorkContext);
//             status = STATUS_INTERNAL_ERROR;
//         }
//
//     } else {
//         RtlZeroMemory(Srb, SCSI_REQUEST_BLOCK_SIZE);
//
//         Srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//         Srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//
//         Cdb = (PCDB)Srb->Cdb;
//         if (TEST_FLAG(FdoExtension->DeviceFlags, DEV_USE_16BYTE_CDB)) {
//             Srb->CdbLength = 16;
//             Cdb->CDB16.OperationCode = SCSIOP_VERIFY16;
//         } else {
//             Srb->CdbLength = 10;
//             Cdb->CDB10.OperationCode = SCSIOP_VERIFY;
//         }
//
//     }
//
//     while (NT_SUCCESS(status) && (sectorCount != 0)) {
//
//         USHORT numSectors = (USHORT) min(sectorCount, MAX_SECTORS_PER_VERIFY);
//
//         if (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//
//             //
//             // Reset status fields
//             //
//
//             srbEx->SrbStatus = 0;
//             srbExDataCdb16->ScsiStatus = 0;
//
//             //
//             // Calculate the request timeout value based
//             // on  the number of sectors  being verified
//             //
//
//             srbEx->TimeOutValue = ((numSectors + 0x7F) >> 7) * FdoExtension->TimeOutValue;
//         } else {
//
//             //
//             // Reset status fields
//             //
//
//             Srb->SrbStatus = 0;
//             Srb->ScsiStatus = 0;
//
//             //
//             // Calculate the request timeout value based
//             // on  the number of sectors  being verified
//             //
//
//             Srb->TimeOutValue = ((numSectors + 0x7F) >> 7) * FdoExtension->TimeOutValue;
//         }
//
//         //
//         // Update verify CDB info.
//         // NOTE - CDB opcode and length has been initialized prior to entering
//         // the while loop
//         //
//
//         if (TEST_FLAG(FdoExtension->DeviceFlags, DEV_USE_16BYTE_CDB)) {
//
//             REVERSE_BYTES_QUAD(&Cdb->CDB16.LogicalBlock, &sectorOffset);
//             REVERSE_BYTES_SHORT(&Cdb->CDB16.TransferLength[2], &numSectors);
//         } else {
//
//             //
//             // Move little endian values into CDB in big endian format
//             //
//
//             Cdb->CDB10.LogicalBlockByte0 = ((PFOUR_BYTE)&sectorOffset)->Byte3;
//             Cdb->CDB10.LogicalBlockByte1 = ((PFOUR_BYTE)&sectorOffset)->Byte2;
//             Cdb->CDB10.LogicalBlockByte2 = ((PFOUR_BYTE)&sectorOffset)->Byte1;
//             Cdb->CDB10.LogicalBlockByte3 = ((PFOUR_BYTE)&sectorOffset)->Byte0;
//
//             Cdb->CDB10.TransferBlocksMsb = ((PFOUR_BYTE)&numSectors)->Byte1;
//             Cdb->CDB10.TransferBlocksLsb = ((PFOUR_BYTE)&numSectors)->Byte0;
//         }
//
//         status = ClassSendSrbSynchronous(Fdo,
//                                          Srb,
//                                          NULL,
//                                          0,
//                                          FALSE);
//
//         NT_ASSERT(status != STATUS_NONEXISTENT_SECTOR);
//
//         sectorCount  -= numSectors;
//         sectorOffset.QuadPart += numSectors;
//     }
//
//     KeReleaseMutex(&DiskData->VerifyMutex, FALSE);
//
//     Irp->IoStatus.Status = status;
//     Irp->IoStatus.Information = 0;
//
//     ClassReleaseRemoveLock(Fdo, Irp);
//     ClassCompleteRequest(Fdo, Irp, IO_NO_INCREMENT);
//
//     FREE_POOL(Srb);
//     FREE_POOL(WorkContext);
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFdoProcessError(
//     PDEVICE_OBJECT Fdo,
//     PSCSI_REQUEST_BLOCK Srb,
//     NTSTATUS *Status,
//     BOOLEAN *Retry
//     )
//
// /*++
//
// Routine Description:
//
//    This routine checks the type of error.  If the error indicates an underrun
//    then indicate the request should be retried.
//
// Arguments:
//
//     Fdo - Supplies a pointer to the functional device object.
//
//     Srb - Supplies a pointer to the failing Srb.
//
//     Status - Status with which the IRP will be completed.
//
//     Retry - Indication of whether the request will be retried.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PSTORAGE_REQUEST_BLOCK srbEx;
//     PCDB cdb = NULL;
//     UCHAR scsiStatus = 0;
//     UCHAR senseBufferLength = 0;
//     PVOID senseBuffer = NULL;
//     CDB noOp = {0};
//
//     //
//     // Get relevant fields from SRB
//     //
//     if (Srb->Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK) {
//
//         srbEx = (PSTORAGE_REQUEST_BLOCK)Srb;
//
//         //
//         // Look for SCSI SRB specific fields
//         //
//         if ((srbEx->SrbFunction == SRB_FUNCTION_EXECUTE_SCSI) &&
//             (srbEx->NumSrbExData > 0)) {
//             cdb = GetSrbScsiData(srbEx, NULL, NULL, &scsiStatus, &senseBuffer, &senseBufferLength);
//
//             //
//             // cdb and sense buffer should not be NULL
//             //
//             NT_ASSERT(cdb != NULL);
//             NT_ASSERT(senseBuffer != NULL);
//
//         }
//
//         if (cdb == NULL) {
//
//             //
//             // Use a cdb that is all 0s
//             //
//             cdb = &noOp;
//         }
//
//     } else {
//
//         cdb = (PCDB)(Srb->Cdb);
//         scsiStatus = Srb->ScsiStatus;
//         senseBufferLength = Srb->SenseInfoBufferLength;
//         senseBuffer = Srb->SenseInfoBuffer;
//     }
//
//     if (*Status == STATUS_DATA_OVERRUN &&
//         (cdb != NULL) &&
//         (IS_SCSIOP_READWRITE(cdb->CDB10.OperationCode))) {
//
//             *Retry = TRUE;
//
//             //
//             // Update the error count for the device.
//             //
//
//             fdoExtension->ErrorCount++;
//
//     } else if (SRB_STATUS(Srb->SrbStatus) == SRB_STATUS_ERROR &&
//                scsiStatus == SCSISTAT_BUSY) {
//
//         //
//         // a disk drive should never be busy this long. Reset the scsi bus
//         // maybe this will clear the condition.
//         //
//
//         ResetBus(Fdo);
//
//         //
//         // Update the error count for the device.
//         //
//
//         fdoExtension->ErrorCount++;
//
//     } else {
//
//         BOOLEAN invalidatePartitionTable = FALSE;
//
//         //
//         // See if this might indicate that something on the drive has changed.
//         //
//
//         if ((Srb->SrbStatus & SRB_STATUS_AUTOSENSE_VALID) &&
//             (senseBuffer != NULL) && (cdb != NULL)) {
//
//             BOOLEAN validSense = FALSE;
//             UCHAR senseKey = 0;
//             UCHAR asc = 0;
//             UCHAR ascq = 0;
//
//             validSense = ScsiGetSenseKeyAndCodes(senseBuffer,
//                                                  senseBufferLength,
//                                                  SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
//                                                  &senseKey,
//                                                  &asc,
//                                                  &ascq);
//
//             if (validSense) {
//
//                 switch (senseKey) {
//
//                     case SCSI_SENSE_ILLEGAL_REQUEST: {
//
//                         switch (asc) {
//
//                             case SCSI_ADSENSE_INVALID_CDB:
//                             {
//                                 //
//                                 // Look to see if this is an Io request with the ForceUnitAccess flag set
//                                 //
//                                 if (((cdb->CDB10.OperationCode == SCSIOP_WRITE)   ||
//                                     (cdb->CDB10.OperationCode == SCSIOP_WRITE16)) &&
//                                     (cdb->CDB10.ForceUnitAccess))
//                                 {
//                                     PDISK_DATA diskData = (PDISK_DATA)fdoExtension->CommonExtension.DriverData;
//
//                                     if (diskData->WriteCacheOverride == DiskWriteCacheEnable)
//                                     {
//                                         PIO_ERROR_LOG_PACKET logEntry = NULL;
//
//                                         //
//                                         // The user has explicitly requested that write caching be turned on.
//                                         // Warn the user that writes with FUA enabled are not working and that
//                                         // they should disable write cache.
//                                         //
//
//                                         logEntry = IoAllocateErrorLogEntry(fdoExtension->DeviceObject,
//                                                                            sizeof(IO_ERROR_LOG_PACKET) + (4 * sizeof(ULONG)));
//
//                                         if (logEntry != NULL)
//                                         {
//                                             logEntry->FinalStatus       = *Status;
//                                             logEntry->ErrorCode         = IO_WARNING_WRITE_FUA_PROBLEM;
//                                             logEntry->SequenceNumber    = 0;
//                                             logEntry->MajorFunctionCode = IRP_MJ_SCSI;
//                                             logEntry->IoControlCode     = 0;
//                                             logEntry->RetryCount        = 0;
//                                             logEntry->UniqueErrorValue  = 0;
//                                             logEntry->DumpDataSize      = 4 * sizeof(ULONG);
//
//                                             logEntry->DumpData[0] = diskData->ScsiAddress.PortNumber;
//                                             logEntry->DumpData[1] = diskData->ScsiAddress.PathId;
//                                             logEntry->DumpData[2] = diskData->ScsiAddress.TargetId;
//                                             logEntry->DumpData[3] = diskData->ScsiAddress.Lun;
//
//                                             //
//                                             // Write the error log packet.
//                                             //
//
//                                             IoWriteErrorLogEntry(logEntry);
//                                         }
//                                     }
//                                     else
//                                     {
//                                         //
//                                         // Turn off write caching on this device. This is so that future
//                                         // critical requests need not be sent down with  ForceUnitAccess
//                                         //
//                                         PIO_WORKITEM workItem = IoAllocateWorkItem(Fdo);
//
//                                         if (workItem)
//                                         {
//                                             IoQueueWorkItem(workItem, DisableWriteCache, CriticalWorkQueue, workItem);
//                                         }
//                                     }
//
//                                     SET_FLAG(fdoExtension->ScanForSpecialFlags, CLASS_SPECIAL_FUA_NOT_SUPPORTED);
//                                     ADJUST_FUA_FLAG(fdoExtension);
//
//
//                                     cdb->CDB10.ForceUnitAccess = FALSE;
//                                     *Retry = TRUE;
//
//                                 } else if ((cdb->CDB6FORMAT.OperationCode == SCSIOP_MODE_SENSE) &&
//                                            (cdb->MODE_SENSE.PageCode == MODE_SENSE_RETURN_ALL)) {
//
//                                     //
//                                     // Mode sense for all pages failed. This command could fail with
//                                     // SCSI_SENSE_ILLEGAL_REQUEST / SCSI_ADSENSE_INVALID_CDB if the data
//                                     // to be returned is more than 256 bytes. In which case, try to get
//                                     // only MODE_PAGE_CACHING since we only need the block descriptor.
//                                     //
//                                     // Simply change the page code and retry the request
//                                     //
//
//                                     cdb->MODE_SENSE.PageCode = MODE_PAGE_CACHING;
//                                     *Retry = TRUE;
//                                 }
//
//                                 break;
//                             }
//                         } // end switch(asc)
//                         break;
//                     }
//
//                     case SCSI_SENSE_NOT_READY: {
//
//                         switch (asc) {
//                         case SCSI_ADSENSE_LUN_NOT_READY: {
//                             switch (ascq) {
//                             case SCSI_SENSEQ_BECOMING_READY:
//                             case SCSI_SENSEQ_MANUAL_INTERVENTION_REQUIRED:
//                             case SCSI_SENSEQ_CAUSE_NOT_REPORTABLE: {
//                                 invalidatePartitionTable = TRUE;
//                                 break;
//                             }
//                             } // end switch(ascq)
//                             break;
//                         }
//
//                         case SCSI_ADSENSE_NO_MEDIA_IN_DEVICE: {
//                             invalidatePartitionTable = TRUE;
//                             break;
//                         }
//                         } // end switch(asc)
//                         break;
//                     }
//
//                     case SCSI_SENSE_MEDIUM_ERROR: {
//                         invalidatePartitionTable = TRUE;
//                         break;
//                     }
//
//                     case SCSI_SENSE_HARDWARE_ERROR: {
//                         invalidatePartitionTable = TRUE;
//                         break;
//                     }
//
//                     case SCSI_SENSE_UNIT_ATTENTION:
//                     {
//                         invalidatePartitionTable = TRUE;
//                         break;
//                     }
//
//                     case SCSI_SENSE_RECOVERED_ERROR: {
//                         invalidatePartitionTable = TRUE;
//                         break;
//                     }
//
//                 } // end switch(senseKey)
//             } // end if (validSense)
//         } else {
//
//             //
//             // On any exceptional scsi condition which might indicate that the
//             // device was changed we will flush out the state of the partition
//             // table.
//             //
//
//             switch (SRB_STATUS(Srb->SrbStatus)) {
//                 case SRB_STATUS_INVALID_LUN:
//                 case SRB_STATUS_INVALID_TARGET_ID:
//                 case SRB_STATUS_NO_DEVICE:
//                 case SRB_STATUS_NO_HBA:
//                 case SRB_STATUS_INVALID_PATH_ID:
//                 case SRB_STATUS_COMMAND_TIMEOUT:
//                 case SRB_STATUS_TIMEOUT:
//                 case SRB_STATUS_SELECTION_TIMEOUT:
//                 case SRB_STATUS_REQUEST_FLUSHED:
//                 case SRB_STATUS_UNEXPECTED_BUS_FREE:
//                 case SRB_STATUS_PARITY_ERROR:
//                 {
//                     invalidatePartitionTable = TRUE;
//                     break;
//                 }
//
//                 case SRB_STATUS_ERROR:
//                 {
//                     if (scsiStatus == SCSISTAT_RESERVATION_CONFLICT)
//                     {
//                         invalidatePartitionTable = TRUE;
//                     }
//
//                     break;
//                 }
//             } // end switch(Srb->SrbStatus)
//         }
//
//         if (invalidatePartitionTable && TEST_FLAG(Fdo->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//             //
//             // Inform the upper layers that the volume
//             // on this disk is in need of verification
//             //
//
//             SET_FLAG(Fdo->Flags, DO_VERIFY_VOLUME);
//         }
//     }
//
//     return;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskSetSpecialHacks(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN ULONG_PTR Data
//     )
//
// /*++
//
// Routine Description:
//
//     This function checks to see if an SCSI logical unit requires speical
//     flags to be set.
//
// Arguments:
//
//     Fdo - Supplies the device object to be tested.
//
//     InquiryData - Supplies the inquiry data returned by the device of interest.
//
//     AdapterDescriptor - Supplies the capabilities of the device object.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PDEVICE_OBJECT fdo = FdoExtension->DeviceObject;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, "Disk SetSpecialHacks, Setting Hacks %p\n", (void*) Data));
//
//     //
//     // Found a listed controller.  Determine what must be done.
//     //
//
//     if (TEST_FLAG(Data, HackDisableTaggedQueuing)) {
//
//         //
//         // Disable tagged queuing.
//         //
//
//         CLEAR_FLAG(FdoExtension->SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//     }
//
//     if (TEST_FLAG(Data, HackDisableSynchronousTransfers)) {
//
//         //
//         // Disable synchronous data transfers.
//         //
//
//         SET_FLAG(FdoExtension->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//
//     }
//
//     if (TEST_FLAG(Data, HackDisableSpinDown)) {
//
//         //
//         // Disable spinning down of drives.
//         //
//
//         SET_FLAG(FdoExtension->ScanForSpecialFlags,
//                  CLASS_SPECIAL_DISABLE_SPIN_DOWN);
//
//     }
//
//     if (TEST_FLAG(Data, HackDisableWriteCache)) {
//
//         //
//         // Disable the drive's write cache
//         //
//
//         SET_FLAG(FdoExtension->ScanForSpecialFlags,
//                  CLASS_SPECIAL_DISABLE_WRITE_CACHE);
//
//     }
//
//     if (TEST_FLAG(Data, HackCauseNotReportableHack)) {
//
//         SET_FLAG(FdoExtension->ScanForSpecialFlags,
//                  CLASS_SPECIAL_CAUSE_NOT_REPORTABLE_HACK);
//     }
//
//     if (TEST_FLAG(fdo->Characteristics, FILE_REMOVABLE_MEDIA) &&
//         TEST_FLAG(Data, HackRequiresStartUnitCommand)
//         ) {
//
//         //
//         // this is a list of vendors who require the START_UNIT command
//         //
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, "DiskScanForSpecial (%p) => This unit requires "
//                     " START_UNITS\n", fdo));
//         SET_FLAG(FdoExtension->DeviceFlags, DEV_SAFE_START_UNIT);
//
//     }
//
//     return;
// }
//
//
// VOID
// ResetBus(
//     IN PDEVICE_OBJECT Fdo
//     )
//
// /*++
//
// Routine Description:
//
//     This command sends a reset bus command to the SCSI port driver.
//
// Arguments:
//
//     Fdo - The functional device object for the logical unit with hardware problem.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIO_STACK_LOCATION irpStack;
//     PIRP irp;
//
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PSCSI_REQUEST_BLOCK srb;
//     PCOMPLETION_CONTEXT context;
//     PSTORAGE_REQUEST_BLOCK srbEx = NULL;
//     PSTOR_ADDR_BTL8 storAddrBtl8 = NULL;
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "Disk ResetBus: Sending reset bus request to port driver.\n"));
//
//     //
//     // Allocate Srb from nonpaged pool.
//     //
//
//     context = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                     sizeof(COMPLETION_CONTEXT),
//                                     DISK_TAG_CCONTEXT);
//
//     if(context == NULL) {
//         return;
//     }
//
//     //
//     // Save the device object in the context for use by the completion
//     // routine.
//     //
//
//     context->DeviceObject = Fdo;
//     srb = &context->Srb.Srb;
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx = &context->Srb.SrbEx;
//
//         //
//         // Zero out srb
//         //
//
//         RtlZeroMemory(srbEx, sizeof(context->Srb.SrbExBuffer));
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = sizeof(context->Srb.SrbExBuffer);
//         srbEx->SrbFunction = SRB_FUNCTION_RESET_BUS;
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//
//         //
//         // Set up address fields
//         //
//
//         storAddrBtl8 = (PSTOR_ADDR_BTL8) ((PUCHAR)srbEx + srbEx->AddressOffset);
//         storAddrBtl8->Type = STOR_ADDRESS_TYPE_BTL8;
//         storAddrBtl8->AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
//
//     } else {
//
//         //
//         // Zero out srb.
//         //
//
//         RtlZeroMemory(srb, SCSI_REQUEST_BLOCK_SIZE);
//
//         //
//         // Write length to SRB.
//         //
//
//         srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//
//         srb->Function = SRB_FUNCTION_RESET_BUS;
//
//     }
//
//     //
//     // Build the asynchronous request to be sent to the port driver.
//     // Since this routine is called from a DPC the IRP should always be
//     // available.
//     //
//
//     irp = IoAllocateIrp(Fdo->StackSize, FALSE);
//
//     if (irp == NULL) {
//         FREE_POOL(context);
//         return;
//     }
//
//     ClassAcquireRemoveLock(Fdo, irp);
//
//     IoSetCompletionRoutine(irp,
//                            (PIO_COMPLETION_ROUTINE)ClassAsynchronousCompletion,
//                            context,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     irpStack = IoGetNextIrpStackLocation(irp);
//
//     irpStack->MajorFunction = IRP_MJ_SCSI;
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx->RequestPriority = IoGetIoPriorityHint(irp);
//         srbEx->OriginalRequest = irp;
//     } else {
//         srb->OriginalRequest = irp;
//     }
//
//     //
//     // Store the SRB address in next stack for port driver.
//     //
//
//     irpStack->Parameters.Scsi.Srb = srb;
//
//     //
//     // Call the port driver with the IRP.
//     //
//
//     IoCallDriver(fdoExtension->CommonExtension.LowerDeviceObject, irp);
//
//     return;
//
// } // end ResetBus()
//
//
//
// VOID
// DiskLogCacheInformation(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN PDISK_CACHE_INFORMATION CacheInfo,
//     IN NTSTATUS Status
//     )
// {
//     PIO_ERROR_LOG_PACKET logEntry = NULL;
//
//     PAGED_CODE();
//
//     logEntry = IoAllocateErrorLogEntry(FdoExtension->DeviceObject, sizeof(IO_ERROR_LOG_PACKET) + (4 * sizeof(ULONG)));
//
//     if (logEntry != NULL)
//     {
//         PDISK_DATA diskData = FdoExtension->CommonExtension.DriverData;
//         BOOLEAN bIsEnabled  = TEST_FLAG(FdoExtension->DeviceFlags, DEV_WRITE_CACHE);
//
//         logEntry->FinalStatus       = Status;
//         logEntry->ErrorCode         = (bIsEnabled) ? IO_WRITE_CACHE_ENABLED : IO_WRITE_CACHE_DISABLED;
//         logEntry->SequenceNumber    = 0;
//         logEntry->MajorFunctionCode = IRP_MJ_SCSI;
//         logEntry->IoControlCode     = 0;
//         logEntry->RetryCount        = 0;
//         logEntry->UniqueErrorValue  = 0x1;
//         logEntry->DumpDataSize      = 4 * sizeof(ULONG);
//
//         logEntry->DumpData[0] = diskData->ScsiAddress.PathId;
//         logEntry->DumpData[1] = diskData->ScsiAddress.TargetId;
//         logEntry->DumpData[2] = diskData->ScsiAddress.Lun;
//         logEntry->DumpData[3] = CacheInfo->WriteCacheEnabled;
//
//         //
//         // Write the error log packet.
//         //
//
//         IoWriteErrorLogEntry(logEntry);
//     }
// }
//
// NTSTATUS
// DiskGetInfoExceptionInformation(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN PMODE_INFO_EXCEPTIONS ReturnPageData
//     )
// {
//     PMODE_PARAMETER_HEADER modeData;
//     PMODE_INFO_EXCEPTIONS pageData;
//     ULONG length;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     //
//     // ReturnPageData is allocated by the caller
//     //
//
//     modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                          MODE_DATA_SIZE,
//                                          DISK_TAG_INFO_EXCEPTION);
//
//     if (modeData == NULL) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_WMI, "DiskGetInfoExceptionInformation: Unable to allocate mode "
//                        "data buffer\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(modeData, MODE_DATA_SIZE);
//
//     length = ClassModeSense(FdoExtension->DeviceObject,
//                             (PCHAR) modeData,
//                             MODE_DATA_SIZE,
//                             MODE_PAGE_FAULT_REPORTING);
//
//     if (length < sizeof(MODE_PARAMETER_HEADER)) {
//
//         //
//         // Retry the request in case of a check condition.
//         //
//
//         length = ClassModeSense(FdoExtension->DeviceObject,
//                                 (PCHAR) modeData,
//                                 MODE_DATA_SIZE,
//                                 MODE_PAGE_FAULT_REPORTING);
//
//         if (length < sizeof(MODE_PARAMETER_HEADER)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_WMI, "DiskGetInfoExceptionInformation: Mode Sense failed\n"));
//             FREE_POOL(modeData);
//             return STATUS_IO_DEVICE_ERROR;
//         }
//     }
//
//     //
//     // If the length is greater than length indicated by the mode data reset
//     // the data to the mode data.
//     //
//
//     if (length > (ULONG) (modeData->ModeDataLength + 1)) {
//         length = modeData->ModeDataLength + 1;
//     }
//
//     //
//     // Find the mode page for info exceptions
//     //
//
//     pageData = ClassFindModePage((PCHAR) modeData,
//                                  length,
//                                  MODE_PAGE_FAULT_REPORTING,
//                                  TRUE);
//
//     if (pageData != NULL) {
//         RtlCopyMemory(ReturnPageData, pageData, sizeof(MODE_INFO_EXCEPTIONS));
//         status =  STATUS_SUCCESS;
//     } else {
//         status = STATUS_NOT_SUPPORTED;
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskGetInfoExceptionInformation: %s support SMART for device %p\n",
//                   NT_SUCCESS(status) ? "does" : "does not",
//                   FdoExtension->DeviceObject));
//
//     FREE_POOL(modeData);
//
//     return(status);
// }
//
//
// NTSTATUS
// DiskSetInfoExceptionInformation(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN PMODE_INFO_EXCEPTIONS PageData
//     )
//
// {
//     ULONG i;
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     // We will attempt (twice) to issue the mode select with the page.
//     // Make the setting persistant so that we don't have to turn it back
//     // on after a bus reset.
//     //
//
//     for (i = 0; i < 2; i++)
//     {
//         status = DiskModeSelect(FdoExtension->DeviceObject,
//                                 (PCHAR) PageData,
//                                 sizeof(MODE_INFO_EXCEPTIONS),
//                                 TRUE);
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskSetInfoExceptionInformation: %s for device %p\n",
//                         NT_SUCCESS(status) ? "succeeded" : "failed",
//                         FdoExtension->DeviceObject));
//
//     return status;
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskGetCacheInformation(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN PDISK_CACHE_INFORMATION CacheInfo
//     )
// /*++
//
// Routine Description:
//
//     This function gets the caching mode page from the drive. This function
//     is called from DiskIoctlGetCacheInformation() in response to the IOCTL
//     IOCTL_DISK_GET_CACHE_INFORMATION. This is also called from the
//     DisableWriteCache() worker thread to disable caching when write commands fail.
//
// Arguments:
//
//     FdoExtension - The device extension for this device.
//
//     CacheInfo - Buffer to receive the Cache Information.
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PMODE_PARAMETER_HEADER modeData;
//     PMODE_CACHING_PAGE pageData;
//
//     ULONG length;
//
//     PAGED_CODE();
//
//
//
//     modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                      MODE_DATA_SIZE,
//                                      DISK_TAG_DISABLE_CACHE);
//
//     if (modeData == NULL) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskGetSetCacheInformation: Unable to allocate mode "
//                        "data buffer\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(modeData, MODE_DATA_SIZE);
//
//     length = ClassModeSense(FdoExtension->DeviceObject,
//                             (PCHAR) modeData,
//                             MODE_DATA_SIZE,
//                             MODE_PAGE_CACHING);
//
//     if (length < sizeof(MODE_PARAMETER_HEADER)) {
//
//         //
//         // Retry the request in case of a check condition.
//         //
//
//         length = ClassModeSense(FdoExtension->DeviceObject,
//                                 (PCHAR) modeData,
//                                 MODE_DATA_SIZE,
//                                 MODE_PAGE_CACHING);
//
//         if (length < sizeof(MODE_PARAMETER_HEADER)) {
//
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskGetCacheInformation: Mode Sense failed\n"));
//
//             FREE_POOL(modeData);
//             return STATUS_IO_DEVICE_ERROR;
//         }
//     }
//
//     //
//     // If the length is greater than length indicated by the mode data reset
//     // the data to the mode data.
//     //
//
//     if (length > (ULONG) (modeData->ModeDataLength + 1)) {
//         length = modeData->ModeDataLength + 1;
//     }
//
//     //
//     // Check to see if the write cache is enabled.
//     //
//
//     pageData = ClassFindModePage((PCHAR) modeData,
//                                  length,
//                                  MODE_PAGE_CACHING,
//                                  TRUE);
//
//     //
//     // Check if valid caching page exists.
//     //
//
//     if (pageData == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskGetCacheInformation: Unable to find caching mode page.\n"));
//         FREE_POOL(modeData);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // Copy the parameters over.
//     //
//
//     RtlZeroMemory(CacheInfo, sizeof(DISK_CACHE_INFORMATION));
//
//     CacheInfo->ParametersSavable = pageData->PageSavable;
//
//     CacheInfo->ReadCacheEnabled = !(pageData->ReadDisableCache);
//     CacheInfo->WriteCacheEnabled = pageData->WriteCacheEnable;
//
//
//     //
//     // Translate the values in the mode page into the ones defined in
//     // ntdddisk.h.
//     //
//
//     CacheInfo->ReadRetentionPriority =
//         TRANSLATE_RETENTION_PRIORITY(pageData->ReadRetensionPriority);
//     CacheInfo->WriteRetentionPriority =
//         TRANSLATE_RETENTION_PRIORITY(pageData->WriteRetensionPriority);
//
//     CacheInfo->DisablePrefetchTransferLength =
//         ((pageData->DisablePrefetchTransfer[0] << 8) +
//          pageData->DisablePrefetchTransfer[1]);
//
//     CacheInfo->ScalarPrefetch.Minimum =
//         ((pageData->MinimumPrefetch[0] << 8) + pageData->MinimumPrefetch[1]);
//
//     CacheInfo->ScalarPrefetch.Maximum =
//         ((pageData->MaximumPrefetch[0] << 8) + pageData->MaximumPrefetch[1]);
//
//     if(pageData->MultiplicationFactor) {
//         CacheInfo->PrefetchScalar = TRUE;
//         CacheInfo->ScalarPrefetch.MaximumBlocks =
//             ((pageData->MaximumPrefetchCeiling[0] << 8) +
//              pageData->MaximumPrefetchCeiling[1]);
//     }
//
//
//     FREE_POOL(modeData);
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskSetCacheInformation(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN PDISK_CACHE_INFORMATION CacheInfo
//     )
// /*++
//
// Routine Description:
//
//     This function sets the caching mode page in the drive. This function
//     is also called from the DisableWriteCache() worker thread to disable
//     caching when write commands fail.
//
// Arguments:
//
//     FdoExtension - The device extension for this device.
//
//     CacheInfo - Buffer the contains the Cache Information to be set on the drive.
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
// {
//     PMODE_PARAMETER_HEADER modeData;
//     ULONG length;
//     PMODE_CACHING_PAGE pageData;
//     ULONG i;
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                      MODE_DATA_SIZE,
//                                      DISK_TAG_DISABLE_CACHE);
//
//     if (modeData == NULL) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskSetCacheInformation: Unable to allocate mode "
//                        "data buffer\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(modeData, MODE_DATA_SIZE);
//
//     length = ClassModeSense(FdoExtension->DeviceObject,
//                             (PCHAR) modeData,
//                             MODE_DATA_SIZE,
//                             MODE_PAGE_CACHING);
//
//     if (length < sizeof(MODE_PARAMETER_HEADER)) {
//
//         //
//         // Retry the request in case of a check condition.
//         //
//
//         length = ClassModeSense(FdoExtension->DeviceObject,
//                                 (PCHAR) modeData,
//                                 MODE_DATA_SIZE,
//                                 MODE_PAGE_CACHING);
//
//         if (length < sizeof(MODE_PARAMETER_HEADER)) {
//
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskSetCacheInformation: Mode Sense failed\n"));
//
//             FREE_POOL(modeData);
//             return STATUS_IO_DEVICE_ERROR;
//         }
//     }
//
//     //
//     // If the length is greater than length indicated by the mode data reset
//     // the data to the mode data.
//     //
//
//     if (length > (ULONG) (modeData->ModeDataLength + 1)) {
//         length = modeData->ModeDataLength + 1;
//     }
//
//     //
//     // Check to see if the write cache is enabled.
//     //
//
//     pageData = ClassFindModePage((PCHAR) modeData,
//                                  length,
//                                  MODE_PAGE_CACHING,
//                                  TRUE);
//
//     //
//     // Check if valid caching page exists.
//     //
//
//     if (pageData == NULL) {
//         FREE_POOL(modeData);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // Don't touch any of the normal parameters - not all drives actually
//     // use the correct size of caching mode page.  Just change the things
//     // which the user could have modified.
//     //
//
//     pageData->PageSavable = FALSE;
//
//     pageData->ReadDisableCache = !(CacheInfo->ReadCacheEnabled);
//     pageData->MultiplicationFactor = CacheInfo->PrefetchScalar;
//     pageData->WriteCacheEnable = CacheInfo->WriteCacheEnabled;
//
//     pageData->WriteRetensionPriority = (UCHAR)
//         TRANSLATE_RETENTION_PRIORITY(CacheInfo->WriteRetentionPriority);
//     pageData->ReadRetensionPriority = (UCHAR)
//         TRANSLATE_RETENTION_PRIORITY(CacheInfo->ReadRetentionPriority);
//
//     pageData->DisablePrefetchTransfer[0] =
//         (UCHAR) (CacheInfo->DisablePrefetchTransferLength >> 8);
//     pageData->DisablePrefetchTransfer[1] =
//         (UCHAR) (CacheInfo->DisablePrefetchTransferLength & 0x00ff);
//
//     pageData->MinimumPrefetch[0] =
//         (UCHAR) (CacheInfo->ScalarPrefetch.Minimum >> 8);
//     pageData->MinimumPrefetch[1] =
//         (UCHAR) (CacheInfo->ScalarPrefetch.Minimum & 0x00ff);
//
//     pageData->MaximumPrefetch[0] =
//         (UCHAR) (CacheInfo->ScalarPrefetch.Maximum >> 8);
//     pageData->MaximumPrefetch[1] =
//         (UCHAR) (CacheInfo->ScalarPrefetch.Maximum & 0x00ff);
//
//     if(pageData->MultiplicationFactor) {
//
//         pageData->MaximumPrefetchCeiling[0] =
//             (UCHAR) (CacheInfo->ScalarPrefetch.MaximumBlocks >> 8);
//         pageData->MaximumPrefetchCeiling[1] =
//             (UCHAR) (CacheInfo->ScalarPrefetch.MaximumBlocks & 0x00ff);
//     }
//
//     //
//     // We will attempt (twice) to issue the mode select with the page.
//     //
//
//     for (i = 0; i < 2; i++) {
//
//         status = DiskModeSelect(FdoExtension->DeviceObject,
//                                 (PCHAR) pageData,
//                                 (pageData->PageLength + 2),
//                                 CacheInfo->ParametersSavable);
//
//         if (NT_SUCCESS(status)) {
//
//             if (CacheInfo->WriteCacheEnabled)
//             {
//                 SET_FLAG(FdoExtension->DeviceFlags, DEV_WRITE_CACHE);
//             }
//             else
//             {
//                 CLEAR_FLAG(FdoExtension->DeviceFlags, DEV_WRITE_CACHE);
//             }
//             ADJUST_FUA_FLAG(FdoExtension);
//
//             break;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//     } else {
//
//         //
//         // We were unable to modify the disk write cache setting
//         //
//
//         SET_FLAG(FdoExtension->ScanForSpecialFlags, CLASS_SPECIAL_MODIFY_CACHE_UNSUCCESSFUL);
//     }
//
//     FREE_POOL(modeData);
//     return status;
// }
//
// NTSTATUS
// DiskIoctlGetCacheSetting(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine description:
//
//     This routine services IOCTL_DISK_GET_CACHE_SETTING. It looks to
//     see if there are any issues with the disk cache and whether the
//     user had previously indicated that the cache is power-protected
//
// Arguments:
//
//     Fdo - The functional device object processing the request
//     Irp - The ioctl to be processed
//
// Return Value:
//
//     STATUS_SUCCESS if successful, an error code otherwise
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpSp = IoGetCurrentIrpStackLocation(Irp);
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     if (irpSp->Parameters.DeviceIoControl.OutputBufferLength < sizeof(DISK_CACHE_SETTING))
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else
//     {
//         PDISK_CACHE_SETTING cacheSetting = (PDISK_CACHE_SETTING)Irp->AssociatedIrp.SystemBuffer;
//
//         cacheSetting->Version = sizeof(DISK_CACHE_SETTING);
//         cacheSetting->State   = DiskCacheNormal;
//
//         //
//         // Determine whether it is safe to turn on the cache
//         //
//         if (TEST_FLAG(fdoExtension->ScanForSpecialFlags, CLASS_SPECIAL_FUA_NOT_SUPPORTED))
//         {
//             cacheSetting->State = DiskCacheWriteThroughNotSupported;
//         }
//
//         //
//         // Determine whether it is possible to modify the cache setting
//         //
//         if (TEST_FLAG(fdoExtension->ScanForSpecialFlags, CLASS_SPECIAL_MODIFY_CACHE_UNSUCCESSFUL))
//         {
//             cacheSetting->State = DiskCacheModifyUnsuccessful;
//         }
//
//         cacheSetting->IsPowerProtected = TEST_FLAG(fdoExtension->DeviceFlags, DEV_POWER_PROTECTED);
//
//         Irp->IoStatus.Information = sizeof(DISK_CACHE_SETTING);
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// DiskIoctlSetCacheSetting(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine description:
//
//     This routine services IOCTL_DISK_SET_CACHE_SETTING. It allows
//     the user to specify whether the disk cache is power-protected
//     or not
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     Fdo - The functional device object processing the request
//     Irp - The ioctl to be processed
//
// Return Value:
//
//     STATUS_SUCCESS if successful, an error code otherwise
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpSp = IoGetCurrentIrpStackLocation(Irp);
//     NTSTATUS status = STATUS_SUCCESS;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     if (irpSp->Parameters.DeviceIoControl.InputBufferLength < sizeof(DISK_CACHE_SETTING))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else
//     {
//         PDISK_CACHE_SETTING cacheSetting = (PDISK_CACHE_SETTING)Irp->AssociatedIrp.SystemBuffer;
//
//         if (cacheSetting->Version == sizeof(DISK_CACHE_SETTING))
//         {
//             ULONG isPowerProtected;
//
//             //
//             // Save away the user-defined override in our extension and the registry
//             //
//             if (cacheSetting->IsPowerProtected)
//             {
//                 SET_FLAG(fdoExtension->DeviceFlags, DEV_POWER_PROTECTED);
//                 isPowerProtected = 1;
//             }
//             else
//             {
//                 CLEAR_FLAG(fdoExtension->DeviceFlags, DEV_POWER_PROTECTED);
//                 isPowerProtected = 0;
//             }
//             ADJUST_FUA_FLAG(fdoExtension);
//
//             ClassSetDeviceParameter(fdoExtension, DiskDeviceParameterSubkey, DiskDeviceCacheIsPowerProtected, isPowerProtected);
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
// DiskIoctlGetLengthInfo(
//     IN OUT PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_GET_LENGTH_INFO. It returns
//     the disk geometry to the caller.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     NTSTATUS status;
//     PIO_STACK_LOCATION irpStack;
//     PGET_LENGTH_INFORMATION lengthInfo;
//     PFUNCTIONAL_DEVICE_EXTENSION p0Extension;
//     PCOMMON_DEVICE_EXTENSION commonExtension;
//     PDISK_DATA partitionZeroData;
//     NTSTATUS oldReadyStatus;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Initialization
//     //
//
//     commonExtension = DeviceObject->DeviceExtension;
//     irpStack = IoGetCurrentIrpStackLocation(Irp);
//     p0Extension = commonExtension->PartitionZeroExtension;
//     partitionZeroData = ((PDISK_DATA) p0Extension->CommonExtension.DriverData);
//
//     //
//     // Check that the buffer is large enough.
//     //
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(GET_LENGTH_INFORMATION)) {
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     // Update the geometry in case it has changed
//     //
//
//     status = DiskReadDriveCapacity(p0Extension->DeviceObject);
//
//     //
//     // Note whether the drive is ready.  If the status has changed then
//     // notify pnp.
//     //
//
//     oldReadyStatus = InterlockedExchange(&(partitionZeroData->ReadyStatus), status);
//
//     if(partitionZeroData->ReadyStatus != oldReadyStatus) {
//         IoInvalidateDeviceRelations(p0Extension->LowerPdo, BusRelations);
//     }
//
//     if(!NT_SUCCESS(status)) {
//         return status;
//     }
//     lengthInfo = (PGET_LENGTH_INFORMATION) Irp->AssociatedIrp.SystemBuffer;
//
//     lengthInfo->Length = commonExtension->PartitionLength;
//
//     status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = sizeof(GET_LENGTH_INFORMATION);
//
//     return status;
// }
//
// NTSTATUS
// DiskIoctlGetDriveGeometry(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_GET_DRIVE_GEOMETRY. It returns
//     the disk geometry to the caller.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - IRP with a return buffer large enough to receive the
//             extended geometry information.
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(DISK_GEOMETRY)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetDriveGeometry: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     if (TEST_FLAG(DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//         //
//         // Issue ReadCapacity to update device extension
//         // with information for current media.
//         //
//
//         status = DiskReadDriveCapacity(commonExtension->PartitionZeroExtension->DeviceObject);
//
//         //
//         // Note whether the drive is ready.
//         //
//
//         diskData->ReadyStatus = status;
//
//         if (!NT_SUCCESS(status)) {
//             return status;
//         }
//     }
//
//     //
//     // Copy drive geometry information from device extension.
//     //
//
//     RtlMoveMemory(Irp->AssociatedIrp.SystemBuffer,
//                   &(fdoExtension->DiskGeometry),
//                   sizeof(DISK_GEOMETRY));
//
//     if (((PDISK_GEOMETRY)Irp->AssociatedIrp.SystemBuffer)->BytesPerSector == 0) {
//         ((PDISK_GEOMETRY)Irp->AssociatedIrp.SystemBuffer)->BytesPerSector = 512;
//     }
//     Irp->IoStatus.Information = sizeof(DISK_GEOMETRY);
//     return STATUS_SUCCESS;
// }
//
// typedef struct _DISK_GEOMETRY_EX_INTERNAL {
//     DISK_GEOMETRY Geometry;
//     LARGE_INTEGER DiskSize;
//     DISK_PARTITION_INFO Partition;
//     DISK_DETECTION_INFO Detection;
// } DISK_GEOMETRY_EX_INTERNAL, *PDISK_GEOMETRY_EX_INTERNAL;
//
// NTSTATUS
// DiskIoctlGetDriveGeometryEx(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_GET_DRIVE_GEOMETRY_EX. It returns
//     the extended disk geometry to the caller.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - The device object to obtain the geometry for.
//
//     Irp - IRP with a return buffer large enough to receive the
//             extended geometry information.
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     NTSTATUS status;
//     PIO_STACK_LOCATION irpStack;
//     PCOMMON_DEVICE_EXTENSION commonExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension;
//     PDISK_DATA diskData;
//     PDISK_GEOMETRY_EX_INTERNAL geometryEx;
//     ULONG OutputBufferLength;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Setup parameters
//     //
//
//     commonExtension = DeviceObject->DeviceExtension;
//     fdoExtension = DeviceObject->DeviceExtension;
//     diskData = (PDISK_DATA)(commonExtension->DriverData);
//     irpStack = IoGetCurrentIrpStackLocation ( Irp );
//     geometryEx = NULL;
//     OutputBufferLength = irpStack->Parameters.DeviceIoControl.OutputBufferLength;
//
//     //
//     // Check that the buffer is large enough. It must be large enough
//     // to hold at lest the Geometry and DiskSize fields of of the
//     // DISK_GEOMETRY_EX structure.
//     //
//
//     if ( (LONG)OutputBufferLength < FIELD_OFFSET (DISK_GEOMETRY_EX, Data) ) {
//
//         //
//         // Buffer too small. Bail out, telling the caller the required
//         // size.
//         //
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetDriveGeometryEx: Output buffer too small.\n"));
//         status = STATUS_BUFFER_TOO_SMALL;
//         return status;
//     }
//
//     if (TEST_FLAG (DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//         //
//         // Issue a ReadCapacity to update device extension
//         // with information for the current media.
//         //
//
//         status = DiskReadDriveCapacity(commonExtension->PartitionZeroExtension->DeviceObject);
//
//         diskData->ReadyStatus = status;
//
//         if (!NT_SUCCESS (status)) {
//             return status;
//         }
//     }
//
//     //
//     // Copy drive geometry.
//     //
//
//     geometryEx = (PDISK_GEOMETRY_EX_INTERNAL)Irp->AssociatedIrp.SystemBuffer;
//     geometryEx->Geometry = fdoExtension->DiskGeometry;
//     if (geometryEx->Geometry.BytesPerSector == 0) {
//         geometryEx->Geometry.BytesPerSector = 512;
//     }
//     geometryEx->DiskSize = commonExtension->PartitionZeroExtension->CommonExtension.PartitionLength;
//
//     //
//     // If the user buffer is large enough to hold the partition information
//     // then add that as well.
//     //
//
//     if ((LONG)OutputBufferLength >=  FIELD_OFFSET (DISK_GEOMETRY_EX_INTERNAL, Detection)) {
//
//         geometryEx->Partition.SizeOfPartitionInfo = sizeof (geometryEx->Partition);
//         geometryEx->Partition.PartitionStyle = diskData->PartitionStyle;
//
//         switch ( diskData->PartitionStyle ) {
//
//             case PARTITION_STYLE_GPT:
//
//                 //
//                 // Copy GPT signature.
//                 //
//
//                 geometryEx->Partition.Gpt.DiskId = diskData->Efi.DiskId;
//                 break;
//
//             case PARTITION_STYLE_MBR:
//
//                 //
//                 // Copy MBR signature and checksum.
//                 //
//
//                 geometryEx->Partition.Mbr.Signature = diskData->Mbr.Signature;
//                 geometryEx->Partition.Mbr.CheckSum = diskData->Mbr.MbrCheckSum;
//                 break;
//
//             default:
//
//                 //
//                 // This is a raw disk. Zero out the signature area so
//                 // nobody gets confused.
//                 //
//
//                 RtlZeroMemory(&geometryEx->Partition, sizeof (geometryEx->Partition));
//         }
//     }
//
//     //
//     // If the buffer is large enough to hold the detection information,
//     // then also add that.
//     //
//
//     if (OutputBufferLength >= sizeof (DISK_GEOMETRY_EX_INTERNAL)) {
//
//         geometryEx->Detection.SizeOfDetectInfo = sizeof (geometryEx->Detection);
//
//         status = DiskGetDetectInfo(fdoExtension, &geometryEx->Detection);
//
//         //
//         // Failed to obtain detection information, set to none.
//         //
//
//         if (!NT_SUCCESS (status)) {
//             geometryEx->Detection.DetectionType = DetectNone;
//         }
//     }
//
//     status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = min (OutputBufferLength, sizeof (DISK_GEOMETRY_EX_INTERNAL));
//
//     return status;
// }
//
// NTSTATUS
// DiskIoctlGetCacheInformation(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_GET_CACHE_INFORMATION. It reads
//     the caching mode page from the device and returns information to
//     the caller. After validating the user parameter it calls the
//     DiskGetCacheInformation() function to get the mode page.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     PDISK_CACHE_INFORMATION cacheInfo = Irp->AssociatedIrp.SystemBuffer;
//     NTSTATUS status;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlGetCacheInformation: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(DISK_CACHE_INFORMATION)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetCacheInformation: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     status = DiskGetCacheInformation(fdoExtension, cacheInfo);
//
//     if (NT_SUCCESS(status)) {
//         Irp->IoStatus.Information = sizeof(DISK_CACHE_INFORMATION);
//
//         //
//         // Make sure write cache setting is reflected in device extension
//         //
//         if (cacheInfo->WriteCacheEnabled)
//         {
//             SET_FLAG(fdoExtension->DeviceFlags, DEV_WRITE_CACHE);
//         }
//         else
//         {
//             CLEAR_FLAG(fdoExtension->DeviceFlags, DEV_WRITE_CACHE);
//         }
//         ADJUST_FUA_FLAG(fdoExtension);
//
//     }
//     return status;
// }
//
//
// NTSTATUS
// DiskIoctlSetCacheInformation(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_SET_CACHE_INFORMATION. It allows
//     the caller to set the caching mode page on the device. This function
//     validates the user parameter and calls the DiskSetCacheInformation()
//     function to set the mode page. It also stores the cache value in the
//     device extension and registry.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     PDISK_CACHE_INFORMATION cacheInfo = Irp->AssociatedIrp.SystemBuffer;
//     NTSTATUS status;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL is equal or above DISPATCH_LEVEL.
//     //
//
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlSetCacheInformation: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(DISK_CACHE_INFORMATION)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSetCacheInformation: Input buffer length mismatch.\n"));
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     status = DiskSetCacheInformation(fdoExtension, cacheInfo);
//
//     //
//     // Save away the user-defined override in our extension and the registry
//     //
//     if (cacheInfo->WriteCacheEnabled) {
//         diskData->WriteCacheOverride = DiskWriteCacheEnable;
//     } else {
//         diskData->WriteCacheOverride = DiskWriteCacheDisable;
//     }
//
//     ClassSetDeviceParameter(fdoExtension, DiskDeviceParameterSubkey,
//                             DiskDeviceUserWriteCacheSetting, diskData->WriteCacheOverride);
//
//     DiskLogCacheInformation(fdoExtension, cacheInfo, status);
//
//     return status;
// }
//
// NTSTATUS
// DiskIoctlGetMediaTypesEx(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_STORAGE_GET_MEDIA_TYPES_EX. It returns
//     the media type information to the caller. After validating the user
//     parameter it calls DiskDetermineMediaTypes() to get the media type.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status;
//
//     PMODE_PARAMETER_HEADER modeData;
//     PMODE_PARAMETER_BLOCK blockDescriptor;
//     PSCSI_REQUEST_BLOCK srb;
//     PCDB cdb;
//     ULONG modeLength;
//     ULONG retries = 4;
//     UCHAR densityCode = 0;
//     BOOLEAN writable = TRUE;
//     BOOLEAN mediaPresent = FALSE;
//     ULONG srbSize;
//     PSTORAGE_REQUEST_BLOCK srbEx = NULL;
//     PSTOR_ADDR_BTL8 storAddrBtl8 = NULL;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16 = NULL;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlGetMediaTypesEx: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(GET_MEDIA_TYPES)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetMediaTypesEx: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = SCSI_REQUEST_BLOCK_SIZE;
//     }
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 DISK_TAG_SRB);
//
//     if (srb == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetMediaTypesEx: Unable to allocate memory.\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(srb, srbSize);
//
//     //
//     // Send a TUR to determine if media is present.
//     //
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx = (PSTORAGE_REQUEST_BLOCK)srb;
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = srbSize;
//         srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//         srbEx->RequestPriority = IoGetIoPriorityHint(Irp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->NumSrbExData = 1;
//
//         // Set timeout value.
//         srbEx->TimeOutValue = fdoExtension->TimeOutValue;
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
//             cdb = (PCDB)srbExDataCdb16->Cdb;
//         } else {
//             // Should not happen
//             NT_ASSERT(FALSE);
//
//             FREE_POOL(srb);
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetMediaTypesEx: Insufficient extended SRB size.\n"));
//             return STATUS_INTERNAL_ERROR;
//         }
//
//     } else {
//
//         srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         srb->CdbLength = 6;
//         cdb = (PCDB)srb->Cdb;
//
//         //
//         // Set timeout value.
//         //
//
//         srb->TimeOutValue = fdoExtension->TimeOutValue;
//
//     }
//     cdb->CDB6GENERIC.OperationCode = SCSIOP_TEST_UNIT_READY;
//
//     status = ClassSendSrbSynchronous(DeviceObject,
//                                      srb,
//                                      NULL,
//                                      0,
//                                      FALSE);
//
//     if (NT_SUCCESS(status)) {
//         mediaPresent = TRUE;
//     }
//
//     modeLength = MODE_DATA_SIZE;
//     modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                      modeLength,
//                                      DISK_TAG_MODE_DATA);
//
//     if (modeData == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetMediaTypesEx: Unable to allocate memory.\n"));
//         FREE_POOL(srb);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(modeData, modeLength);
//
//     //
//     // Build the MODE SENSE CDB using previous SRB.
//     //
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx->SrbStatus = 0;
//         srbExDataCdb16->ScsiStatus = 0;
//         srbExDataCdb16->CdbLength = 6;
//
//         //
//         // Set timeout value from device extension.
//         //
//
//         srbEx->TimeOutValue = fdoExtension->TimeOutValue;
//     } else {
//         srb->SrbStatus = 0;
//         srb->ScsiStatus = 0;
//         srb->CdbLength = 6;
//
//         //
//         // Set timeout value from device extension.
//         //
//
//         srb->TimeOutValue = fdoExtension->TimeOutValue;
//     }
//
//     //
//     // Page code of 0x3F will return all pages.
//     // This command could fail if the data to be returned is
//     // more than 256 bytes. In which case, we should get only
//     // the caching page since we only need the block descriptor.
//     // DiskFdoProcessError will change the page code to
//     // MODE_PAGE_CACHING if there is an error.
//     //
//
//     cdb->MODE_SENSE.OperationCode    = SCSIOP_MODE_SENSE;
//     cdb->MODE_SENSE.PageCode         = MODE_SENSE_RETURN_ALL;
//     cdb->MODE_SENSE.AllocationLength = (UCHAR)modeLength;
//
// Retry:
//     status = ClassSendSrbSynchronous(DeviceObject,
//                                      srb,
//                                      modeData,
//                                      modeLength,
//                                      FALSE);
//
//     if (status == STATUS_VERIFY_REQUIRED) {
//
//         if (retries--) {
//
//             //
//             // Retry request.
//             //
//
//             goto Retry;
//         }
//     } else if (SRB_STATUS(srb->SrbStatus) == SRB_STATUS_DATA_OVERRUN) {
//         status = STATUS_SUCCESS;
//     }
//
//     if (NT_SUCCESS(status) || (status == STATUS_NO_MEDIA_IN_DEVICE)) {
//
//         //
//         // Get the block descriptor.
//         //
//
//         if (modeData->BlockDescriptorLength != 0) {
//
//             blockDescriptor = (PMODE_PARAMETER_BLOCK)((ULONG_PTR)modeData + sizeof(MODE_PARAMETER_HEADER));
//             densityCode = blockDescriptor->DensityCode;
//         }
//
//         if (TEST_FLAG(modeData->DeviceSpecificParameter,
//                       MODE_DSP_WRITE_PROTECT)) {
//
//             writable = FALSE;
//         }
//
//         status = DiskDetermineMediaTypes(DeviceObject,
//                                          Irp,
//                                          modeData->MediumType,
//                                          densityCode,
//                                          mediaPresent,
//                                          writable);
//         //
//         // If the buffer was too small, DetermineMediaTypes updated the status and information and the request will fail.
//         //
//
//     } else {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetMediaTypesEx: Mode sense for header/bd failed. %lx\n", status));
//     }
//
//     FREE_POOL(srb);
//     FREE_POOL(modeData);
//
//     return status;
// }
//
// NTSTATUS
// DiskIoctlPredictFailure(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_STORAGE_PREDICT_FAILURE. If the device
//     supports SMART then it returns any available failure data.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PSTORAGE_PREDICT_FAILURE checkFailure;
//     STORAGE_FAILURE_PREDICT_STATUS diskSmartStatus;
//     _IO_STATUS_BLOCK ioStatus = { 0 };
//     KEVENT event;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlPredictFailure: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(STORAGE_PREDICT_FAILURE)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlPredictFailure: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     // See if the disk is predicting failure
//     //
//
//     checkFailure = (PSTORAGE_PREDICT_FAILURE)Irp->AssociatedIrp.SystemBuffer;
//
//     if (diskData->FailurePredictionCapability == FailurePredictionSense) {
//         ULONG readBufferSize;
//         PUCHAR readBuffer;
//         PIRP readIrp;
//         PDEVICE_OBJECT topOfStack;
//
//         checkFailure->PredictFailure = 0;
//
//         KeInitializeEvent(&event, SynchronizationEvent, FALSE);
//
//         topOfStack = IoGetAttachedDeviceReference(DeviceObject);
//
//         //
//         // SCSI disks need to have a read sent down to provoke any
//         // failures to be reported.
//         //
//         // Issue a normal read operation.  The error-handling code in
//         // classpnp will take care of a failure prediction by logging the
//         // correct event.
//         //
//
//         readBufferSize = fdoExtension->DiskGeometry.BytesPerSector;
//         readBuffer = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                            readBufferSize,
//                                            DISK_TAG_SMART);
//
//         if (readBuffer != NULL) {
//             LARGE_INTEGER offset;
//
//             offset.QuadPart = 0;
//             readIrp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                                    topOfStack,
//                                                    readBuffer,
//                                                    readBufferSize,
//                                                    &offset,
//                                                    &event,
//                                                    &ioStatus);
//
//             if (readIrp != NULL) {
//
//                 status = IoCallDriver(topOfStack, readIrp);
//                 if (status == STATUS_PENDING) {
//                     KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//                     status = ioStatus.Status;
//                 }
//
//
//             } else {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             FREE_POOL(readBuffer);
//         } else {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         ObDereferenceObject(topOfStack);
//     }
//
//     if (status != STATUS_INSUFFICIENT_RESOURCES)
//     {
//         if ((diskData->FailurePredictionCapability == FailurePredictionSmart) ||
//             (diskData->FailurePredictionCapability == FailurePredictionSense)) {
//
//             status = DiskReadFailurePredictStatus(fdoExtension, &diskSmartStatus);
//
//             if (NT_SUCCESS(status)) {
//
//                 status = DiskReadFailurePredictData(fdoExtension,
//                                                     Irp->AssociatedIrp.SystemBuffer);
//
//                 if (diskSmartStatus.PredictFailure) {
//                     checkFailure->PredictFailure = 1;
//                 } else {
//                     checkFailure->PredictFailure = 0;
//                 }
//
//                 Irp->IoStatus.Information = sizeof(STORAGE_PREDICT_FAILURE);
//             }
//         } else {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//     return status;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WINBLUE)
// NTSTATUS
// DiskIoctlEnableFailurePrediction(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_STORAGE_FAILURE_PREDICTION_CONFIG. If the device
//     supports SMART then it returns any available failure data.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status = STATUS_SUCCESS;
//     PSTORAGE_FAILURE_PREDICTION_CONFIG enablePrediction;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlEnableFailurePrediction: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(STORAGE_FAILURE_PREDICTION_CONFIG) ||
//         irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(STORAGE_FAILURE_PREDICTION_CONFIG)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlEnableFailurePrediction: Buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     enablePrediction = (PSTORAGE_FAILURE_PREDICTION_CONFIG)Irp->AssociatedIrp.SystemBuffer;
//
//     if (enablePrediction->Version != STORAGE_FAILURE_PREDICTION_CONFIG_V1 ||
//         enablePrediction->Size < sizeof(STORAGE_FAILURE_PREDICTION_CONFIG)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlEnableFailurePrediction: Buffer version or size is incorrect.\n"));
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     if (enablePrediction->Reserved != 0) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlEnableFailurePrediction: Reserved bytes are not zero!\n"));
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // Default to success.  This might get overwritten on failure below.
//     //
//     status = STATUS_SUCCESS;
//
//     //
//     // If this is a "set" and the current state (enabled/disabled) is
//     // different from the sender's desired state,
//     //
//     if (enablePrediction->Set && enablePrediction->Enabled != diskData->FailurePredictionEnabled) {
//         if (diskData->FailurePredictionCapability == FailurePredictionSmart ||
//             diskData->FailurePredictionCapability == FailurePredictionIoctl) {
//             //
//             // SMART or IOCTL based failure prediction is being used so call
//             // the generic function that is normally called in the WMI path.
//             //
//             status = DiskEnableDisableFailurePrediction(fdoExtension, enablePrediction->Enabled);
//         } else if (diskData->ScsiInfoExceptionsSupported) {
//             //
//             // If we know that the device supports the Informational Exceptions
//             // mode page, try to enable/disable failure prediction that way.
//             //
//             status = DiskEnableInfoExceptions(fdoExtension, enablePrediction->Enabled);
//         }
//     }
//
//     //
//     // Return the current state regardless if this was a "set" or a "get".
//     //
//     enablePrediction->Enabled = diskData->FailurePredictionEnabled;
//
//     if (NT_SUCCESS(status)) {
//         Irp->IoStatus.Information = sizeof(STORAGE_FAILURE_PREDICTION_CONFIG);
//     }
//
//     return status;
// }
// #endif //(NTDDI_VERSION >= NTDDI_WINBLUE)
//
// NTSTATUS
// DiskIoctlVerify(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_VERIFY. After verifying
//     user input, it starts the worker thread DiskIoctlVerifyThread()
//     to verify the device.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     PVERIFY_INFORMATION verifyInfo = Irp->AssociatedIrp.SystemBuffer;
//     PDISK_VERIFY_WORKITEM_CONTEXT Context = NULL;
//     PSCSI_REQUEST_BLOCK srb;
//     LARGE_INTEGER byteOffset;
//     ULONG srbSize;
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlVerify: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(VERIFY_INFORMATION)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlVerify: Input buffer length mismatch.\n"));
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = SCSI_REQUEST_BLOCK_SIZE;
//     }
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 DISK_TAG_SRB);
//
//     if (srb == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlVerify: Unable to allocate memory.\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(srb, srbSize);
//
//     //
//     // Add disk offset to starting sector.
//     //
//
//     byteOffset.QuadPart = commonExtension->StartingOffset.QuadPart +
//                           verifyInfo->StartingOffset.QuadPart;
//
//     //
//     // Perform a bounds check on the sector range
//     //
//
//     if ((verifyInfo->StartingOffset.QuadPart > commonExtension->PartitionLength.QuadPart) ||
//         (verifyInfo->StartingOffset.QuadPart < 0)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlVerify: Verify request to invalid sector.\n"));
//         FREE_POOL(srb)
//         return STATUS_NONEXISTENT_SECTOR;
//     } else {
//
//         ULONGLONG bytesRemaining = commonExtension->PartitionLength.QuadPart - verifyInfo->StartingOffset.QuadPart;
//
//         if ((ULONGLONG)verifyInfo->Length > bytesRemaining) {
//
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlVerify: Verify request to invalid sector.\n"));
//             FREE_POOL(srb)
//             return STATUS_NONEXISTENT_SECTOR;
//         }
//     }
//
//     Context = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                     sizeof(DISK_VERIFY_WORKITEM_CONTEXT),
//                                     DISK_TAG_WI_CONTEXT);
//     if (Context) {
//
//         Context->Irp = Irp;
//         Context->Srb = srb;
//         Context->WorkItem = IoAllocateWorkItem(DeviceObject);
//
//         if (Context->WorkItem) {
//
//             //
//             // Queue the work item and return.
//             //
//
//             IoMarkIrpPending(Irp);
//
//             IoQueueWorkItem(Context->WorkItem,
//                             DiskIoctlVerifyThread,
//                             DelayedWorkQueue,
//                             Context);
//
//             return STATUS_PENDING;
//         }
//         FREE_POOL(Context);
//     }
//     FREE_POOL(srb)
//     return STATUS_INSUFFICIENT_RESOURCES;
// }
//
// NTSTATUS
// DiskIoctlReassignBlocks(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_REASSIGN_BLOCKS. This IOCTL
//     allows the caller to remap the defective blocks to a new
//     location on the disk.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status;
//     PREASSIGN_BLOCKS badBlocks = Irp->AssociatedIrp.SystemBuffer;
//     PSCSI_REQUEST_BLOCK srb;
//     PCDB cdb;
//     ULONG bufferSize;
//     ULONG blockNumber;
//     ULONG blockCount;
//     ULONG srbSize;
//     PSTORAGE_REQUEST_BLOCK srbEx;
//     PSTOR_ADDR_BTL8 storAddrBtl8;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(REASSIGN_BLOCKS)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: Input buffer length mismatch.\n"));
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     //
//     // Make sure we have some data in the input buffer.
//     //
//
//     if (badBlocks->Count == 0) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: Invalid block count\n"));
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     bufferSize = sizeof(REASSIGN_BLOCKS) + ((badBlocks->Count - 1) * sizeof(ULONG));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < bufferSize) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: Input buffer length mismatch for bad blocks.\n"));
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = SCSI_REQUEST_BLOCK_SIZE;
//     }
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 DISK_TAG_SRB);
//
//     if (srb == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: Unable to allocate memory.\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(srb, srbSize);
//
//     //
//     // Build the data buffer to be transferred in the input buffer.
//     // The format of the data to the device is:
//     //
//     //      2 bytes Reserved
//     //      2 bytes Length
//     //      x * 4 btyes Block Address
//     //
//     // All values are big endian.
//     //
//
//     badBlocks->Reserved = 0;
//     blockCount = badBlocks->Count;
//
//     //
//     // Convert # of entries to # of bytes.
//     //
//
//     blockCount *= 4;
//     badBlocks->Count = (USHORT) ((blockCount >> 8) & 0XFF);
//     badBlocks->Count |= (USHORT) ((blockCount << 8) & 0XFF00);
//
//     //
//     // Convert back to number of entries.
//     //
//
//     blockCount /= 4;
//
//     for (; blockCount > 0; blockCount--) {
//
//         blockNumber = badBlocks->BlockNumber[blockCount-1];
//         REVERSE_BYTES((PFOUR_BYTE) &badBlocks->BlockNumber[blockCount-1], (PFOUR_BYTE) &blockNumber);
//     }
//
//     //
//     // Build a SCSI SRB containing a SCSIOP_REASSIGN_BLOCKS cdb
//     //
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx = (PSTORAGE_REQUEST_BLOCK)srb;
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = srbSize;
//         srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//         srbEx->RequestPriority = IoGetIoPriorityHint(Irp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->NumSrbExData = 1;
//
//         // Set timeout value.
//         srbEx->TimeOutValue = fdoExtension->TimeOutValue;
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
//             cdb = (PCDB)srbExDataCdb16->Cdb;
//         } else {
//             // Should not happen
//             NT_ASSERT(FALSE);
//
//             FREE_POOL(srb);
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: Insufficient extended SRB size.\n"));
//             return STATUS_INTERNAL_ERROR;
//         }
//
//     } else {
//         srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         srb->CdbLength = 6;
//
//         //
//         // Set timeout value.
//         //
//
//         srb->TimeOutValue = fdoExtension->TimeOutValue;
//
//         cdb = (PCDB)srb->Cdb;
//     }
//
//     cdb->CDB6GENERIC.OperationCode = SCSIOP_REASSIGN_BLOCKS;
//
//     status = ClassSendSrbSynchronous(DeviceObject,
//                                      srb,
//                                      badBlocks,
//                                      bufferSize,
//                                      TRUE);
//
//     FREE_POOL(srb);
//     return status;
// }
//
// NTSTATUS
// DiskIoctlReassignBlocksEx(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_REASSIGN_BLOCKS_EX. This IOCTL
//     allows the caller to remap the defective blocks to a new
//     location on the disk. The input buffer contains 8-byte LBAs.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status;
//     PREASSIGN_BLOCKS_EX badBlocks = Irp->AssociatedIrp.SystemBuffer;
//     PSCSI_REQUEST_BLOCK srb;
//     PCDB cdb;
//     LARGE_INTEGER blockNumber;
//     ULONG bufferSize;
//     ULONG blockCount;
//     ULONG srbSize;
//     PSTORAGE_REQUEST_BLOCK srbEx;
//     PSTOR_ADDR_BTL8 storAddrBtl8;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocksEx: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(REASSIGN_BLOCKS_EX)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocksEx: Input buffer length mismatch.\n"));
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     //
//     // Make sure we have some data in the input buffer.
//     //
//
//     if (badBlocks->Count == 0) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocksEx: Invalid block count\n"));
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     bufferSize = sizeof(REASSIGN_BLOCKS_EX) + ((badBlocks->Count - 1) * sizeof(LARGE_INTEGER));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < bufferSize) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocksEx: Input buffer length mismatch for bad blocks.\n"));
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = SCSI_REQUEST_BLOCK_SIZE;
//     }
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 DISK_TAG_SRB);
//
//     if (srb == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: Unable to allocate memory.\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(srb, srbSize);
//
//     //
//     // Build the data buffer to be transferred in the input buffer.
//     // The format of the data to the device is:
//     //
//     //      2 bytes Reserved
//     //      2 bytes Length
//     //      x * 8 btyes Block Address
//     //
//     // All values are big endian.
//     //
//
//     badBlocks->Reserved = 0;
//     blockCount = badBlocks->Count;
//
//     //
//     // Convert # of entries to # of bytes.
//     //
//
//     blockCount *= 8;
//     badBlocks->Count = (USHORT) ((blockCount >> 8) & 0XFF);
//     badBlocks->Count |= (USHORT) ((blockCount << 8) & 0XFF00);
//
//     //
//     // Convert back to number of entries.
//     //
//
//     blockCount /= 8;
//
//     for (; blockCount > 0; blockCount--) {
//
//         blockNumber = badBlocks->BlockNumber[blockCount-1];
//         REVERSE_BYTES_QUAD(&badBlocks->BlockNumber[blockCount-1], &blockNumber);
//     }
//
//     //
//     // Build a SCSI SRB containing a SCSIOP_REASSIGN_BLOCKS cdb
//     //
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx = (PSTORAGE_REQUEST_BLOCK)srb;
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = srbSize;
//         srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//         srbEx->RequestPriority = IoGetIoPriorityHint(Irp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->NumSrbExData = 1;
//
//         // Set timeout value.
//         srbEx->TimeOutValue = fdoExtension->TimeOutValue;
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
//             cdb = (PCDB)srbExDataCdb16->Cdb;
//         } else {
//             // Should not happen
//             NT_ASSERT(FALSE);
//
//             FREE_POOL(srb);
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlReassignBlocks: Insufficient extended SRB size.\n"));
//             return STATUS_INTERNAL_ERROR;
//         }
//
//     } else {
//         srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         srb->CdbLength = 6;
//
//         //
//         // Set timeout value.
//         //
//
//         srb->TimeOutValue = fdoExtension->TimeOutValue;
//
//         cdb = (PCDB)srb->Cdb;
//     }
//
//     cdb->CDB6GENERIC.OperationCode = SCSIOP_REASSIGN_BLOCKS;
//     cdb->CDB6GENERIC.CommandUniqueBits =  1; // LONGLBA
//
//     status = ClassSendSrbSynchronous(DeviceObject,
//                                      srb,
//                                      badBlocks,
//                                      bufferSize,
//                                      TRUE);
//
//     FREE_POOL(srb);
//     return status;
// }
//
// NTSTATUS
// DiskIoctlIsWritable(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_IS_WRITABLE. This function
//     returns whether the disk is writable. If the device is not
//     writable then STATUS_MEDIA_WRITE_PROTECTED will be returned.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PMODE_PARAMETER_HEADER modeData;
//     PSCSI_REQUEST_BLOCK srb;
//     PCDB cdb = NULL;
//     ULONG modeLength;
//     ULONG retries = 4;
//     ULONG srbSize;
//     PSTORAGE_REQUEST_BLOCK srbEx;
//     PSTOR_ADDR_BTL8 storAddrBtl8;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlIsWritable: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = SCSI_REQUEST_BLOCK_SIZE;
//     }
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 DISK_TAG_SRB);
//
//     if (srb == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlIsWritable: Unable to allocate memory.\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(srb, srbSize);
//
//     //
//     // Allocate memory for a mode header and then some
//     // for port drivers that need to convert to MODE10
//     // or always return the MODE_PARAMETER_BLOCK (even
//     // when memory was not allocated for this purpose)
//     //
//
//     modeLength = MODE_DATA_SIZE;
//     modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                      modeLength,
//                                      DISK_TAG_MODE_DATA);
//
//     if (modeData == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlIsWritable: Unable to allocate memory.\n"));
//         FREE_POOL(srb);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(modeData, modeLength);
//
//     //
//     // Build the MODE SENSE CDB
//     //
//
//     if (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbEx = (PSTORAGE_REQUEST_BLOCK)srb;
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = srbSize;
//         srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//         srbEx->RequestPriority = IoGetIoPriorityHint(Irp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->NumSrbExData = 1;
//
//         // Set timeout value.
//         srbEx->TimeOutValue = fdoExtension->TimeOutValue;
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
//             cdb = (PCDB)srbExDataCdb16->Cdb;
//         } else {
//             // Should not happen
//             NT_ASSERT(FALSE);
//
//             FREE_POOL(srb);
//             FREE_POOL(modeData);
//             return STATUS_INTERNAL_ERROR;
//         }
//
//     } else {
//         srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         srb->CdbLength = 6;
//
//         //
//         // Set timeout value.
//         //
//
//         srb->TimeOutValue = fdoExtension->TimeOutValue;
//
//         cdb = (PCDB)srb->Cdb;
//     }
//
//     //
//     // Page code of 0x3F will return all pages.
//     // This command could fail if the data to be returned is
//     // more than 256 bytes. In which case, we should get only
//     // the caching page since we only need the block descriptor.
//     // DiskFdoProcessError will change the page code to
//     // MODE_PAGE_CACHING if there is an error.
//     //
//
//     cdb->MODE_SENSE.OperationCode    = SCSIOP_MODE_SENSE;
//     cdb->MODE_SENSE.PageCode         = MODE_SENSE_RETURN_ALL;
//     cdb->MODE_SENSE.AllocationLength = (UCHAR)modeLength;
//
//     while (retries != 0) {
//
//         status = ClassSendSrbSynchronous(DeviceObject,
//                                          srb,
//                                          modeData,
//                                          modeLength,
//                                          FALSE);
//
//         if (status != STATUS_VERIFY_REQUIRED) {
//             if (SRB_STATUS(srb->SrbStatus) == SRB_STATUS_DATA_OVERRUN) {
//                 status = STATUS_SUCCESS;
//             }
//             break;
//         }
//         retries--;
//     }
//
//     if (NT_SUCCESS(status)) {
//
//         if (TEST_FLAG(modeData->DeviceSpecificParameter, MODE_DSP_WRITE_PROTECT)) {
//             status = STATUS_MEDIA_WRITE_PROTECTED;
//         }
//     }
//
//     FREE_POOL(srb);
//     FREE_POOL(modeData);
//     return status;
// }
//
// NTSTATUS
// DiskIoctlSetVerify(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_INTERNAL_SET_VERIFY.
//     This is an internal function used to set the DO_VERIFY_VOLUME
//     device object flag. Only a kernel mode component can send this
//     IOCTL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     NTSTATUS status = STATUS_NOT_SUPPORTED;
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlSetVerify: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     //
//     // If the caller is kernel mode, set the verify bit.
//     //
//
//     if (Irp->RequestorMode == KernelMode) {
//
//         SET_FLAG(DeviceObject->Flags, DO_VERIFY_VOLUME);
//         status = STATUS_SUCCESS;
//
//     }
//     return status;
// }
//
// NTSTATUS
// DiskIoctlClearVerify(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_INTERNAL_CLEAR_VERIFY.
//     This is an internal function used to clear the DO_VERIFY_VOLUME
//     device object flag. Only a kernel mode component can send this
//     IOCTL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     NTSTATUS status = STATUS_NOT_SUPPORTED;
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlClearVerify: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     //
//     // If the caller is kernel mode, set the verify bit.
//     //
//
//     if (Irp->RequestorMode == KernelMode) {
//
//         CLEAR_FLAG(DeviceObject->Flags, DO_VERIFY_VOLUME);
//         status = STATUS_SUCCESS;
//
//     }
//     return status;
// }
//
// NTSTATUS
// DiskIoctlUpdateDriveSize(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_DISK_UPDATE_DRIVE_SIZE.
//     This function is used to inform the disk driver to update
//     the device geometry information cached in the device extension
//     This is normally initiated from the drivers layers above disk
//     driver.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     TARGET_DEVICE_CUSTOM_NOTIFICATION Notification = {0};
//     NTSTATUS status;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlUpdateDriveSize: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(DISK_GEOMETRY)) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlUpdateDriveSize: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     status = DiskReadDriveCapacity(DeviceObject);
//
//     //
//     // Note whether the drive is ready.
//     //
//
//     diskData->ReadyStatus = status;
//
//     if (NT_SUCCESS(status)) {
//
//         //
//         // Copy drive geometry information from the device extension.
//         //
//
//         RtlMoveMemory(Irp->AssociatedIrp.SystemBuffer,
//                       &(fdoExtension->DiskGeometry),
//                       sizeof(DISK_GEOMETRY));
//
//         if (((PDISK_GEOMETRY)Irp->AssociatedIrp.SystemBuffer)->BytesPerSector == 0) {
//             ((PDISK_GEOMETRY)Irp->AssociatedIrp.SystemBuffer)->BytesPerSector = 512;
//         }
//         Irp->IoStatus.Information = sizeof(DISK_GEOMETRY);
//         status = STATUS_SUCCESS;
//
//         //
//         // Notify everyone that the disk layout may have changed
//         //
//
//         Notification.Event   = GUID_IO_DISK_LAYOUT_CHANGE;
//         Notification.Version = 1;
//         Notification.Size    = (USHORT)FIELD_OFFSET(TARGET_DEVICE_CUSTOM_NOTIFICATION, CustomDataBuffer);
//         Notification.FileObject = NULL;
//         Notification.NameBufferOffset = -1;
//
//         IoReportTargetDeviceChangeAsynchronous(fdoExtension->LowerPdo,
//                                                &Notification,
//                                                NULL,
//                                                NULL);
//     }
//     return status;
// }
//
// NTSTATUS
// DiskIoctlGetVolumeDiskExtents(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS
//     and IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS_ADMIN. This function
//     returns the physical location of a volume.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status = STATUS_INVALID_DEVICE_REQUEST;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlGetVolumeDiskExtents: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//
//     if (TEST_FLAG(DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//         if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(VOLUME_DISK_EXTENTS)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlGetVolumeDiskExtents: Output buffer too small.\n"));
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         status = DiskReadDriveCapacity(commonExtension->PartitionZeroExtension->DeviceObject);
//
//         //
//         // Note whether the drive is ready.
//         //
//
//         diskData->ReadyStatus = status;
//
//         if (NT_SUCCESS(status)) {
//
//             PVOLUME_DISK_EXTENTS pVolExt = (PVOLUME_DISK_EXTENTS)Irp->AssociatedIrp.SystemBuffer;
//
//             pVolExt->NumberOfDiskExtents = 1;
//             pVolExt->Extents[0].DiskNumber     = commonExtension->PartitionZeroExtension->DeviceNumber;
//             pVolExt->Extents[0].StartingOffset = commonExtension->StartingOffset;
//             pVolExt->Extents[0].ExtentLength   = commonExtension->PartitionLength;
//
//             Irp->IoStatus.Information = sizeof(VOLUME_DISK_EXTENTS);
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// DiskIoctlSmartGetVersion(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services SMART_GET_VERSION. It returns the
//     SMART version information.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status;
//
//     PGETVERSIONINPARAMS versionParams;
//     PSRB_IO_CONTROL srbControl;
//     _IO_STATUS_BLOCK ioStatus = { 0 };
//     PUCHAR buffer;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlSmartGetVersion: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(GETVERSIONINPARAMS)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartGetVersion: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     srbControl = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                        sizeof(SRB_IO_CONTROL) +
//                                        sizeof(GETVERSIONINPARAMS),
//                                        DISK_TAG_SMART);
//
//     if (srbControl == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartGetVersion: Unable to allocate memory.\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(srbControl, sizeof(SRB_IO_CONTROL) + sizeof(GETVERSIONINPARAMS));
//
//     //
//     // fill in srbControl fields
//     //
//
//     srbControl->HeaderLength = sizeof(SRB_IO_CONTROL);
//     RtlMoveMemory (srbControl->Signature, "SCSIDISK", 8);
//     srbControl->Timeout = fdoExtension->TimeOutValue;
//     srbControl->Length = sizeof(GETVERSIONINPARAMS);
//     srbControl->ControlCode = IOCTL_SCSI_MINIPORT_SMART_VERSION;
//
//     //
//     // Point to the 'buffer' portion of the SRB_CONTROL
//     //
//
//     buffer = (PUCHAR)srbControl + srbControl->HeaderLength;
//
//     //
//     // Ensure correct target is set in the cmd parameters.
//     //
//
//     versionParams = (PGETVERSIONINPARAMS)buffer;
//     versionParams->bIDEDeviceMap = diskData->ScsiAddress.TargetId;
//
//     ClassSendDeviceIoControlSynchronous(
//                                     IOCTL_SCSI_MINIPORT,
//                                     commonExtension->LowerDeviceObject,
//                                     srbControl,
//                                     sizeof(SRB_IO_CONTROL) + sizeof(GETVERSIONINPARAMS),
//                                     sizeof(SRB_IO_CONTROL) + sizeof(GETVERSIONINPARAMS),
//                                     FALSE,
//                                     &ioStatus);
//
//     status = ioStatus.Status;
//
//     //
//     // If successful, copy the data received into the output buffer.
//     // This should only fail in the event that the IDE driver is older
//     // than this driver.
//     //
//
//     if (NT_SUCCESS(status)) {
//
//         buffer = (PUCHAR)srbControl + srbControl->HeaderLength;
//
//         RtlMoveMemory (Irp->AssociatedIrp.SystemBuffer, buffer, sizeof(GETVERSIONINPARAMS));
//         Irp->IoStatus.Information = sizeof(GETVERSIONINPARAMS);
//     }
//
//     FREE_POOL(srbControl);
//
//     return status;
// }
//
// NTSTATUS
// DiskIoctlSmartReceiveDriveData(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services SMART_RCV_DRIVE_DATA. This function
//     allows the caller to read SMART information from the device.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status = STATUS_INVALID_PARAMETER;
//
//     PSENDCMDINPARAMS cmdInParameters = ((PSENDCMDINPARAMS)Irp->AssociatedIrp.SystemBuffer);
//     PSRB_IO_CONTROL srbControl;
//     _IO_STATUS_BLOCK ioStatus = { 0 };
//     ULONG controlCode = 0;
//     PUCHAR buffer;
//     PIRP irp2;
//     KEVENT event;
//     ULONG length = 0;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < (sizeof(SENDCMDINPARAMS) - 1)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: Input buffer length invalid.\n"));
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < (sizeof(SENDCMDOUTPARAMS) + 512 - 1)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     // Create notification event object to be used to signal the
//     // request completion.
//     //
//
//     KeInitializeEvent(&event, NotificationEvent, FALSE);
//
//     //
//     // use controlCode as a sort of 'STATUS_SUCCESS' to see if it's
//     // a valid request type
//     //
//
//     if (cmdInParameters->irDriveRegs.bCommandReg == ID_CMD) {
//
//         length = IDENTIFY_BUFFER_SIZE + sizeof(SENDCMDOUTPARAMS);
//         controlCode = IOCTL_SCSI_MINIPORT_IDENTIFY;
//
//     } else if (cmdInParameters->irDriveRegs.bCommandReg == SMART_CMD) {
//
//         switch (cmdInParameters->irDriveRegs.bFeaturesReg) {
//
//             case READ_ATTRIBUTES:
//                 controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS;
//                 length = READ_ATTRIBUTE_BUFFER_SIZE + sizeof(SENDCMDOUTPARAMS);
//                 break;
//
//             case READ_THRESHOLDS:
//                 controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS;
//                 length = READ_THRESHOLD_BUFFER_SIZE + sizeof(SENDCMDOUTPARAMS);
//                 break;
//
//             case SMART_READ_LOG: {
//
//                 if (diskData->FailurePredictionCapability != FailurePredictionSmart) {
//                     TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: SMART failure prediction not supported.\n"));
//                     return STATUS_INVALID_DEVICE_REQUEST;
//                 }
//
//                 //
//                 // Calculate additional length based on number of sectors to be read.
//                 // Then verify the output buffer is large enough.
//                 //
//
//                 length = cmdInParameters->irDriveRegs.bSectorCountReg * SMART_LOG_SECTOR_SIZE;
//
//                 //
//                 // Ensure at least 1 sector is going to be read
//                 //
//                 if (length == 0) {
//                     return STATUS_INVALID_PARAMETER;
//                 }
//
//                 length += max(sizeof(SENDCMDOUTPARAMS), sizeof(SENDCMDINPARAMS));
//
//                 if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < length - 1) {
//                     TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: Output buffer too small for SMART_READ_LOG.\n"));
//                     return STATUS_BUFFER_TOO_SMALL;
//                 }
//
//                 controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_LOG;
//                 break;
//             }
//         }
//     }
//
//     if (controlCode == 0) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: Invalid request.\n"));
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     srbControl = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                        sizeof(SRB_IO_CONTROL) + length,
//                                        DISK_TAG_SMART);
//
//     if (srbControl == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: Unable to allocate memory.\n"));
//         return  STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // fill in srbControl fields
//     //
//
//     srbControl->HeaderLength = sizeof(SRB_IO_CONTROL);
//     RtlMoveMemory (srbControl->Signature, "SCSIDISK", 8);
//     srbControl->Timeout = fdoExtension->TimeOutValue;
//     srbControl->Length = length;
//     srbControl->ControlCode = controlCode;
//
//     //
//     // Point to the 'buffer' portion of the SRB_CONTROL
//     //
//
//     buffer = (PUCHAR)srbControl + srbControl->HeaderLength;
//
//     //
//     // Ensure correct target is set in the cmd parameters.
//     //
//
//     cmdInParameters->bDriveNumber = diskData->ScsiAddress.TargetId;
//
//     //
//     // Copy the IOCTL parameters to the srb control buffer area.
//     //
//
//     RtlMoveMemory(buffer,
//                   Irp->AssociatedIrp.SystemBuffer,
//                   sizeof(SENDCMDINPARAMS) - 1);
//
//     irp2 = IoBuildDeviceIoControlRequest(IOCTL_SCSI_MINIPORT,
//                                         commonExtension->LowerDeviceObject,
//                                         srbControl,
//                                         sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDINPARAMS) - 1,
//                                         srbControl,
//                                         sizeof(SRB_IO_CONTROL) + length,
//                                         FALSE,
//                                         &event,
//                                         &ioStatus);
//
//     if (irp2 == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartReceiveDriveData: Unable to allocate IRP.\n"));
//         FREE_POOL(srbControl);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // Call the port driver with the request and wait for it to complete.
//     //
//
//     status = IoCallDriver(commonExtension->LowerDeviceObject, irp2);
//
//     if (status == STATUS_PENDING) {
//         KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//         status = ioStatus.Status;
//     }
//
//     //
//     // Copy the data received into the output buffer. Since the status buffer
//     // contains error information also, always perform this copy. IO will
//     // either pass this back to the app, or zero it, in case of error.
//     //
//
//     buffer = (PUCHAR)srbControl + srbControl->HeaderLength;
//
//     if (NT_SUCCESS(status)) {
//
//         RtlMoveMemory ( Irp->AssociatedIrp.SystemBuffer, buffer, length - 1);
//         Irp->IoStatus.Information = length - 1;
//
//     } else {
//
//         RtlMoveMemory ( Irp->AssociatedIrp.SystemBuffer, buffer, (sizeof(SENDCMDOUTPARAMS) - 1));
//         Irp->IoStatus.Information = sizeof(SENDCMDOUTPARAMS) - 1;
//
//     }
//
//     FREE_POOL(srbControl);
//
//     return status;
// }
//
// NTSTATUS
// DiskIoctlSmartSendDriveCommand(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine services SMART_SEND_DRIVE_COMMAND. This function
//     allows the caller to send SMART commands to the device.
//
//     This function must be called at IRQL < DISPATCH_LEVEL.
//
// Arguments:
//
//     DeviceObject - Supplies the device object associated with this request.
//
//     Irp - The IRP to be processed
//
// Return Value:
//
//     NTSTATUS code
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation (Irp);
//     NTSTATUS status = STATUS_INVALID_PARAMETER;
//
//     PSENDCMDINPARAMS cmdInParameters = ((PSENDCMDINPARAMS)Irp->AssociatedIrp.SystemBuffer);
//     PSRB_IO_CONTROL srbControl;
//     _IO_STATUS_BLOCK ioStatus = { 0 };
//     ULONG controlCode = 0;
//     PUCHAR buffer;
//     PIRP irp2;
//     KEVENT event;
//     ULONG length = 0;
//
//     //
//     // This function must be called at less than dispatch level.
//     // Fail if IRQL >= DISPATCH_LEVEL.
//     //
//     PAGED_CODE();
//     CHECK_IRQL();
//
//     //
//     // Validate the request.
//     //
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: DeviceObject %p Irp %p\n", DeviceObject, Irp));
//
//     if (irpStack->Parameters.DeviceIoControl.InputBufferLength < (sizeof(SENDCMDINPARAMS) - 1)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: Input buffer size invalid.\n"));
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (irpStack->Parameters.DeviceIoControl.OutputBufferLength < (sizeof(SENDCMDOUTPARAMS) - 1)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: Output buffer too small.\n"));
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     // Create notification event object to be used to signal the
//     // request completion.
//     //
//
//     KeInitializeEvent(&event, NotificationEvent, FALSE);
//
//     if (cmdInParameters->irDriveRegs.bCommandReg == SMART_CMD) {
//
//         switch (cmdInParameters->irDriveRegs.bFeaturesReg) {
//
//             case SMART_WRITE_LOG: {
//
//                 if (diskData->FailurePredictionCapability != FailurePredictionSmart) {
//
//                     TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: SMART failure prediction not supported.\n"));
//                     return STATUS_INVALID_DEVICE_REQUEST;
//                 }
//
//                 //
//                 // Calculate additional length based on number of sectors to be written.
//                 // Then verify the input buffer is large enough.
//                 //
//
//                 length = cmdInParameters->irDriveRegs.bSectorCountReg * SMART_LOG_SECTOR_SIZE;
//
//                 //
//                 // Ensure at least 1 sector is going to be written
//                 //
//
//                 if (length == 0) {
//                     return STATUS_INVALID_PARAMETER;
//                 }
//
//                 if (irpStack->Parameters.DeviceIoControl.InputBufferLength <
//                         (sizeof(SENDCMDINPARAMS) - 1) + length) {
//
//                     TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: Input buffer too small for SMART_WRITE_LOG.\n"));
//                     return STATUS_BUFFER_TOO_SMALL;
//                 }
//
//                 controlCode = IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG;
//                 break;
//             }
//
//             case ENABLE_SMART:
//                 controlCode = IOCTL_SCSI_MINIPORT_ENABLE_SMART;
//                 break;
//
//             case DISABLE_SMART:
//                 controlCode = IOCTL_SCSI_MINIPORT_DISABLE_SMART;
//                 break;
//
//             case RETURN_SMART_STATUS:
//
//                 //
//                 // Ensure bBuffer is at least 2 bytes (to hold the values of
//                 // cylinderLow and cylinderHigh).
//                 //
//
//                 if (irpStack->Parameters.DeviceIoControl.OutputBufferLength <
//                     (sizeof(SENDCMDOUTPARAMS) - 1 + sizeof(IDEREGS))) {
//
//                     return STATUS_BUFFER_TOO_SMALL;
//                     break;
//                 }
//
//                 controlCode = IOCTL_SCSI_MINIPORT_RETURN_STATUS;
//                 length = sizeof(IDEREGS);
//                 break;
//
//             case ENABLE_DISABLE_AUTOSAVE:
//                 controlCode = IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE;
//                 break;
//
//             case SAVE_ATTRIBUTE_VALUES:
//                 controlCode = IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES;
//                 break;
//
//             case EXECUTE_OFFLINE_DIAGS:
//                 //
//                 // Validate that this is an ok self test command
//                 //
//                 if (DiskIsValidSmartSelfTest(cmdInParameters->irDriveRegs.bSectorNumberReg)) {
//
//                     controlCode = IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS;
//                 }
//                 break;
//
//             case ENABLE_DISABLE_AUTO_OFFLINE:
//                 controlCode = IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTO_OFFLINE;
//                 break;
//         }
//     }
//
//     if (controlCode == 0) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: Invalid request.\n"));
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     length += max(sizeof(SENDCMDOUTPARAMS), sizeof(SENDCMDINPARAMS));
//     srbControl = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                        sizeof(SRB_IO_CONTROL) + length,
//                                        DISK_TAG_SMART);
//
//     if (srbControl == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: Unable to allocate memory.\n"));
//         return  STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // fill in srbControl fields
//     //
//
//     srbControl->HeaderLength = sizeof(SRB_IO_CONTROL);
//     RtlMoveMemory (srbControl->Signature, "SCSIDISK", 8);
//     srbControl->Timeout = fdoExtension->TimeOutValue;
//     srbControl->Length = length;
//     srbControl->ControlCode = controlCode;
//
//     //
//     // Point to the 'buffer' portion of the SRB_CONTROL
//     //
//
//     buffer = (PUCHAR)srbControl + srbControl->HeaderLength;
//
//     //
//     // Ensure correct target is set in the cmd parameters.
//     //
//
//     cmdInParameters->bDriveNumber = diskData->ScsiAddress.TargetId;
//
//     //
//     // Copy the IOCTL parameters to the srb control buffer area.
//     //
//
//     if (cmdInParameters->irDriveRegs.bFeaturesReg == SMART_WRITE_LOG) {
//         RtlMoveMemory(buffer,
//                      Irp->AssociatedIrp.SystemBuffer,
//                      sizeof(SENDCMDINPARAMS) - 1 +
//                         cmdInParameters->irDriveRegs.bSectorCountReg * SMART_LOG_SECTOR_SIZE);
//     } else {
//         RtlMoveMemory(buffer, Irp->AssociatedIrp.SystemBuffer, sizeof(SENDCMDINPARAMS) - 1);
//     }
//
//     irp2 = IoBuildDeviceIoControlRequest(IOCTL_SCSI_MINIPORT,
//                                         commonExtension->LowerDeviceObject,
//                                         srbControl,
//                                         sizeof(SRB_IO_CONTROL) + length,
//                                         srbControl,
//                                         sizeof(SRB_IO_CONTROL) + length,
//                                         FALSE,
//                                         &event,
//                                         &ioStatus);
//
//     if (irp2 == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DiskIoctlSmartSendDriveCommand: Unable to allocate IRP.\n"));
//         FREE_POOL(srbControl);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // Call the port driver with the request and wait for it to complete.
//     //
//
//     status = IoCallDriver(commonExtension->LowerDeviceObject, irp2);
//
//     if (status == STATUS_PENDING) {
//         KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//         status = ioStatus.Status;
//     }
//
//     //
//     // Copy the data received into the output buffer. Since the status buffer
//     // contains error information also, always perform this copy. IO will
//     // either pass this back to the app, or zero it, in case of error.
//     //
//
//     buffer = (PUCHAR)srbControl + srbControl->HeaderLength;
//
//     //
//     // Update the return buffer size based on the sub-command.
//     //
//
//     if (cmdInParameters->irDriveRegs.bFeaturesReg == RETURN_SMART_STATUS) {
//         length = sizeof(SENDCMDOUTPARAMS) - 1 + sizeof(IDEREGS);
//     } else {
//         length = sizeof(SENDCMDOUTPARAMS) - 1;
//     }
//
//     RtlMoveMemory ( Irp->AssociatedIrp.SystemBuffer, buffer, length);
//     Irp->IoStatus.Information = length;
//
//     FREE_POOL(srbControl);
//
//     return status;
// }
//
//
// VOID
// DiskEtwEnableCallback (
//      LPCGUID                      SourceId,
//      ULONG                        IsEnabled,
//      UCHAR                        Level,
//      ULONGLONG                    MatchAnyKeyword,
//      ULONGLONG                    MatchAllKeyword,
//      PEVENT_FILTER_DESCRIPTOR FilterData,
//      PVOID                    CallbackContext
//     )
// /*+++
//
// Routine Description:
//
//     This routine is the enable callback routine for ETW. It gets called when
//     tracing is enabled or disabled for our provider.
//
// Arguments:
//
//     As per the ETW callback.
//
// Return Value:
//
//     None.
// --*/
//
// {
//     //
//     // Initialize locals.
//     //
//     UNREFERENCED_PARAMETER(SourceId);
//     UNREFERENCED_PARAMETER(Level);
//     UNREFERENCED_PARAMETER(MatchAnyKeyword);
//     UNREFERENCED_PARAMETER(MatchAllKeyword);
//     UNREFERENCED_PARAMETER(FilterData);
//     UNREFERENCED_PARAMETER(CallbackContext);
//
//     //
//     // Set the ETW tracing enable state.
//     //
//     DiskETWEnabled = IsEnabled ? TRUE : FALSE;
//
//     return;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================


// ===== re-integrated dropped DiskIoctl* handlers

// STORAGE_FAILURE_PREDICTION_CONFIG — not in bindgen, defined locally
#[repr(C)]
struct STORAGE_FAILURE_PREDICTION_CONFIG {
    Version: u32,
    Size: u32,
    Set: u8,
    Enabled: u8,
    Reserved: u16,
}
type PSTORAGE_FAILURE_PREDICTION_CONFIG = *mut STORAGE_FAILURE_PREDICTION_CONFIG;

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlClearVerify(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_NOT_SUPPORTED;

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlClearVerify: DeviceObject %p Irp %p\n\0".as_ptr() as PCCHAR,
        DeviceObject,
        Irp,
    );

    if (*Irp).RequestorMode as i32 == (KernelMode) as i32 {
        (*DeviceObject).Flags &= !DO_VERIFY_VOLUME;
        status = STATUS_SUCCESS;
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlEnableFailurePrediction(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> i32 {
    let commonExtension: *mut COMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let diskData: *mut DISK_DATA = (*commonExtension).DriverData as *mut DISK_DATA;
    let irpStack: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: i32 = STATUS_SUCCESS;
    let enablePrediction: *mut STORAGE_FAILURE_PREDICTION_CONFIG;

    PAGED_CODE();
    CHECK_IRQL();

    // TracePrint first call: %p %p -> two pointer arguments
    TracePrint(
        /*TRACE_LEVEL_INFORMATION*/ 4,
        /*TRACE_FLAG_IOCTL*/ 1,
        (b"DiskIoctlEnableFailurePrediction: DeviceObject %p Irp %p\n\0".as_ptr() as *const u8).cast(),
        DeviceObject as *mut core::ffi::c_void,
        Irp as *mut core::ffi::c_void,
    );

    if (*irpStack)
        .Parameters
        .DeviceIoControl
        .InputBufferLength
        < (core::mem::size_of::<STORAGE_FAILURE_PREDICTION_CONFIG>()) as u32
        || (*irpStack)
            .Parameters
            .DeviceIoControl
            .OutputBufferLength
            < (core::mem::size_of::<STORAGE_FAILURE_PREDICTION_CONFIG>()) as u32
    {
        TracePrint(
            /*TRACE_LEVEL_ERROR*/ 2,
            /*TRACE_FLAG_IOCTL*/ 1,
            (b"DiskIoctlEnableFailurePrediction: Buffer too small.\n\0".as_ptr() as *const u8).cast(),
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    enablePrediction =
        (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_FAILURE_PREDICTION_CONFIG;

    // STORAGE_FAILURE_PREDICTION_CONFIG_V1 placeholder
    const STORAGE_FAILURE_PREDICTION_CONFIG_V1: u32 = 1;
    if (*enablePrediction).Version != STORAGE_FAILURE_PREDICTION_CONFIG_V1
        || (*enablePrediction).Size < core::mem::size_of::<STORAGE_FAILURE_PREDICTION_CONFIG>() as u32
    {
        TracePrint(
            /*TRACE_LEVEL_ERROR*/ 2,
            /*TRACE_FLAG_IOCTL*/ 1,
            (b"DiskIoctlEnableFailurePrediction: Buffer version or size is incorrect.\n\0".as_ptr()
                as *const u8).cast(),
        );
        status = STATUS_INVALID_PARAMETER;
    }

    if (*enablePrediction).Reserved != 0 {
        TracePrint(
            /*TRACE_LEVEL_ERROR*/ 2,
            /*TRACE_FLAG_IOCTL*/ 1,
            (b"DiskIoctlEnableFailurePrediction: Reserved bytes are not zero!\n\0".as_ptr()
                as *const u8).cast(),
        );
        status = STATUS_INVALID_PARAMETER;
    }

    status = STATUS_SUCCESS;

    if (*enablePrediction).Set != 0
        && (*enablePrediction).Enabled != (*diskData).FailurePredictionEnabled
    {
        // classpnp.h:233 -- these two were PERMUTED (Ioctl is 1, Smart is 2), which a
        // set-based constant seal cannot detect: the set is right, the slots are not.
        const FAILURE_PREDICTION_SMART: u32 = 2;
        const FAILURE_PREDICTION_IOCTL: u32 = 1;

        if (*diskData).FailurePredictionCapability == FAILURE_PREDICTION_SMART
            || (*diskData).FailurePredictionCapability == FAILURE_PREDICTION_IOCTL
        {
            status = DiskEnableDisableFailurePrediction(fdoExtension, (*enablePrediction).Enabled);
        } else if (*diskData).ScsiInfoExceptionsSupported != 0 {
            status = DiskEnableInfoExceptions(fdoExtension, (*enablePrediction).Enabled);
        }
    }

    (*enablePrediction).Enabled = (*diskData).FailurePredictionEnabled;

    if status >= 0 {
        (*Irp).IoStatus.Information =
            core::mem::size_of::<STORAGE_FAILURE_PREDICTION_CONFIG>() as ULONG_PTR;
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlGetCacheInformation(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irpStack: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let cacheInfo: PDISK_CACHE_INFORMATION =
        (*Irp).AssociatedIrp.SystemBuffer as PDISK_CACHE_INFORMATION;
    let mut status: NTSTATUS;

    // PAGED_CODE();
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                3878i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // CHECK_IRQL();
    if KeGetCurrentIrql() as i32 >= DISPATCH_LEVEL as i32 {
        if !((KeGetCurrentIrql() as i32) < 2i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                3879i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    StorDebugPrint(
        4i32,
        0x00000002u32, // TRACE_FLAG_IOCTL
        b"DiskIoctlGetCacheInformation: DeviceObject %p Irp %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<DISK_CACHE_INFORMATION>()
    {
        StorDebugPrint(
            2i32,
            0x00000002u32, // TRACE_FLAG_IOCTL
            b"DiskIoctlGetCacheInformation: Output buffer too small.\n\0".as_ptr() as *const i8,
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    status = DiskGetCacheInformation(fdoExtension, cacheInfo);

    if status >= 0 {
        (*Irp).IoStatus.Information =
            core::mem::size_of::<DISK_CACHE_INFORMATION>() as ULONG_PTR;

        if (*cacheInfo).WriteCacheEnabled != 0 {
            (*fdoExtension).DeviceFlags =
                ((*fdoExtension).DeviceFlags as u16 | 0x1u16) as USHORT;
        } else {
            (*fdoExtension).DeviceFlags =
                ((*fdoExtension).DeviceFlags as u16 & !0x1u16) as USHORT;
        }

        // ADJUST_FUA_FLAG (classpnp.h): DEV_WRITE_CACHE && !DEV_POWER_PROTECTED &&
        // !CLASS_SPECIAL_FUA_NOT_SUPPORTED. The two negated guards were `!(x & FLAG) != 0`,
        // which is Rust's BITWISE not — never zero for a flag test, so both were always
        // true and the whole condition collapsed to DEV_WRITE_CACHE alone. That forced FUA
        // on power-protected devices and on devices that had explicitly reported it
        // unsupported. C's `!` is logical; the faithful spelling is `== 0`.
        if ((*fdoExtension).DeviceFlags as i32 & 0x1i32) != 0
            && ((*fdoExtension).DeviceFlags as i32 & 0x10i32) == 0
            && ((*fdoExtension).ScanForSpecialFlags as u32 & 0x80u32) == 0
        {
            (*fdoExtension).CdbForceUnitAccess = 1u8; // TRUE
        } else {
            (*fdoExtension).CdbForceUnitAccess = 0u8; // FALSE
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlGetCacheSetting(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = 
        (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let irpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // inline expansion of PAGED_CODE()
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL as i32) {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0")
                .as_ptr() as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0")
                    .as_ptr() as *const u8 as *const i8,
                (b"drivers/storage/class/disk/disk.c\0")
                    .as_ptr() as *const u8 as *const i8,
                3376i32,
                (b"0\0").as_ptr() as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    if ((*irpSp).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<DISK_CACHE_SETTING>()
    {
        status = STATUS_BUFFER_TOO_SMALL;
    } else {
        let cacheSetting: *mut DISK_CACHE_SETTING =
            (*Irp).AssociatedIrp.SystemBuffer as *mut DISK_CACHE_SETTING;
        (*cacheSetting).Version = core::mem::size_of::<DISK_CACHE_SETTING>() as u32;
        (*cacheSetting).State = DiskCacheNormal;

        if ((*fdoExtension).ScanForSpecialFlags as u32 & 0x80u32) != 0 {
            (*cacheSetting).State = DiskCacheWriteThroughNotSupported;
        }
        if ((*fdoExtension).ScanForSpecialFlags as u32 & 0x40u32) != 0 {
            (*cacheSetting).State = DiskCacheModifyUnsuccessful;
        }

        (*cacheSetting).IsPowerProtected = (((*fdoExtension).DeviceFlags as i32 & 0x10i32) != 0) as i32 as BOOLEAN;
        (*Irp).IoStatus.Information = core::mem::size_of::<DISK_CACHE_SETTING>() as ULONG_PTR;
    }

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlGetDriveGeometry(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let status: NTSTATUS;

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                3620i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    if KeGetCurrentIrql() as i32 >= (DISPATCH_LEVEL) as i32 {
        if !((KeGetCurrentIrql() as i32) < 2i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                3621i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<DISK_GEOMETRY>() as usize
    {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlGetDriveGeometry: Output buffer too small.\n\0".as_ptr() as *const i8,
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*DeviceObject).Characteristics as u32 & FILE_REMOVABLE_MEDIA != 0 {
        status = DiskReadDriveCapacity((*(*commonExtension).PartitionZeroExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject);
        (*diskData).ReadyStatus = status;
        if !NT_SUCCESS(status) {
            return status;
        }
    }

    core::ptr::copy(
        &raw mut (*fdoExtension).DiskGeometry as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        core::mem::size_of::<DISK_GEOMETRY>(),
    );

    let geo_ptr = (*Irp).AssociatedIrp.SystemBuffer as *mut DISK_GEOMETRY;
    if (*geo_ptr).BytesPerSector == 0 {
        (*geo_ptr).BytesPerSector = 512;
    }

    (*Irp).IoStatus.Information = core::mem::size_of::<DISK_GEOMETRY>() as ULONG_PTR;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlGetDriveGeometryEx(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let mut irpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut commonExtension: PCOMMON_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut diskData: PDISK_DATA = core::ptr::null_mut();
    let mut geometryEx: PDISK_GEOMETRY_EX_INTERNAL = core::ptr::null_mut();
    let mut OutputBufferLength: u32 = 0;

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                3712,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    if KeGetCurrentIrql() as i32 >= (DISPATCH_LEVEL) as i32 {
        if !((KeGetCurrentIrql() as i32) < 2) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                3713,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    diskData = (*commonExtension).DriverData as PDISK_DATA;
    irpStack = IoGetCurrentIrpStackLocation(Irp);
    geometryEx = core::ptr::null_mut();
    OutputBufferLength = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;

    if (OutputBufferLength as i32) < core::mem::offset_of!(DISK_GEOMETRY_EX, Data) as i32 {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlGetDriveGeometryEx: Output buffer too small.\n\0".as_ptr() as *const i8,
        );
        status = STATUS_BUFFER_TOO_SMALL;
        return status;
    }

    if ((*DeviceObject).Characteristics & FILE_REMOVABLE_MEDIA) != 0 {
        status = DiskReadDriveCapacity(
            (*(*commonExtension).PartitionZeroExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        );
        (*diskData).ReadyStatus = status;
        if !(status >= 0) {
            return status;
        }
    }

    geometryEx = (*Irp).AssociatedIrp.SystemBuffer as PDISK_GEOMETRY_EX_INTERNAL;
    (*geometryEx).Geometry = (*fdoExtension).DiskGeometry;
    if (*geometryEx).Geometry.BytesPerSector == 0 {
        (*geometryEx).Geometry.BytesPerSector = 512;
    }
    (*geometryEx).DiskSize = (*(*commonExtension).PartitionZeroExtension)
        .__bindgen_anon_1.CommonExtension
        .as_ref().PartitionLength;

    if (OutputBufferLength as i32) >= core::mem::offset_of!(DISK_GEOMETRY_EX_INTERNAL, Detection) as i32 {
        (*geometryEx).Partition.SizeOfPartitionInfo =
            core::mem::size_of::<DISK_PARTITION_INFO>() as u32;
        (*geometryEx).Partition.PartitionStyle = (*diskData).PartitionStyle;
        match (*diskData).PartitionStyle {
            PARTITION_STYLE_GPT => {
                (*geometryEx).Partition.__bindgen_anon_1.Gpt.DiskId = (*diskData).__bindgen_anon_1.Efi.as_ref().DiskId;
            }
            PARTITION_STYLE_MBR => {
                (*geometryEx).Partition.__bindgen_anon_1.Mbr.Signature = (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature;
                (*geometryEx).Partition.__bindgen_anon_1.Mbr.CheckSum = (*diskData).__bindgen_anon_1.Mbr.as_ref().MbrCheckSum;
            }
            _ => {
                core::ptr::write_bytes(
                    &raw mut (*geometryEx).Partition as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<DISK_PARTITION_INFO>(),
                );
            }
        }
    }

    if (OutputBufferLength as usize) >= core::mem::size_of::<DISK_GEOMETRY_EX_INTERNAL>() {
        (*geometryEx).Detection.SizeOfDetectInfo =
            core::mem::size_of::<DISK_DETECTION_INFO>() as u32;
        status = DiskGetDetectInfo(fdoExtension, &raw mut (*geometryEx).Detection);
        if !(status >= 0) {
            (*geometryEx).Detection.DetectionType = DetectNone;
        }
    }

    status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = if (OutputBufferLength as usize)
        < core::mem::size_of::<DISK_GEOMETRY_EX_INTERNAL>()
    {
        OutputBufferLength as ULONG_PTR
    } else {
        core::mem::size_of::<DISK_GEOMETRY_EX_INTERNAL>() as ULONG_PTR
    };

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlGetLengthInfo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let mut irpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut lengthInfo: PGET_LENGTH_INFORMATION = core::ptr::null_mut();
    let mut p0Extension: PFUNCTIONAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut commonExtension: PCOMMON_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut partitionZeroData: PDISK_DATA = core::ptr::null_mut();
    let mut oldReadyStatus: NTSTATUS = 0;

    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                3531i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    if KeGetCurrentIrql() as i32 >= DISPATCH_LEVEL as i32 {
        if !((KeGetCurrentIrql() as i32) < 2i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                3532i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    irpStack = IoGetCurrentIrpStackLocation(Irp);
    p0Extension = (*commonExtension).PartitionZeroExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    partitionZeroData = (*p0Extension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;

    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize) < core::mem::size_of::<GET_LENGTH_INFORMATION>() {
        return STATUS_BUFFER_TOO_SMALL;
    }

    status = DiskReadDriveCapacity((*p0Extension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject);

    oldReadyStatus = _InterlockedExchange(
        &raw mut (*partitionZeroData).ReadyStatus as *mut i32,
        status as i32,
    ) as NTSTATUS;

    if (*partitionZeroData).ReadyStatus != oldReadyStatus {
        IoInvalidateDeviceRelations((*p0Extension).LowerPdo, BusRelations);
    }

    if !(status >= 0) {
        return status;
    }

    lengthInfo = (*Irp).AssociatedIrp.SystemBuffer as PGET_LENGTH_INFORMATION;
    (*lengthInfo).Length = (*commonExtension).PartitionLength;
    status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = core::mem::size_of::<GET_LENGTH_INFORMATION>() as ULONG_PTR;

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlGetMediaTypesEx(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut core::ffi::c_void,
) -> NTSTATUS {
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irpStack = IoGetCurrentIrpStackLocation((Irp).cast());
    let mut status: NTSTATUS = 0;
    let mut modeData: *mut MODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut blockDescriptor: *mut MODE_PARAMETER_BLOCK = core::ptr::null_mut();
    let mut srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: *mut CDB = core::ptr::null_mut();
    let mut modeLength: ULONG = 0;
    let mut retries: ULONG = 4;
    let mut densityCode: UCHAR = 0;
    let mut writable: BOOLEAN = 1;
    let mut mediaPresent: BOOLEAN = 0;
    let mut srbSize: ULONG = 0;
    let mut srbEx: *mut STORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: *mut STOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: *mut SRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                4044,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }
    if KeGetCurrentIrql() as i32 >= (DISPATCH_LEVEL) as i32 {
        if !((KeGetCurrentIrql() as i32) < 2) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                4045,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }
    StorDebugPrint(
        4,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlGetMediaTypesEx: DeviceObject %p Irp %p\n\0" as *const u8 as *const i8,
        DeviceObject,
        Irp,
    );
    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize) <
        core::mem::size_of::<GET_MEDIA_TYPES>() {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlGetMediaTypesEx: Output buffer too small.\n\0" as *const u8 as *const i8,
        );
        return STATUS_BUFFER_TOO_SMALL;
    }
    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = SCSI_REQUEST_BLOCK_SIZE;
    }
    srb = ExAllocatePoolWithTag(NonPagedPool, srbSize as SIZE_T, DISK_TAG_SRB) as *mut SCSI_REQUEST_BLOCK;
    if srb.is_null() {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlGetMediaTypesEx: Unable to allocate memory.\n\0" as *const u8 as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(srb as *mut u8, 0, srbSize as usize);
    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbEx = srb as *mut STORAGE_REQUEST_BLOCK;
        (*srbEx).Length = core::mem::offset_of!(STORAGE_REQUEST_BLOCK, Signature) as USHORT;
        (*srbEx).Function = (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8;
        (*srbEx).Signature = SRB_SIGNATURE;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1;
        (*srbEx).SrbLength = srbSize;
        (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        (*srbEx).RequestPriority = IoGetIoPriorityHint((Irp).cast()) as USHORT;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as ULONG;
        (*srbEx).NumSrbExData = 1;
        (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;
        storAddrBtl8 =
            (srbEx as *mut u8).offset((*srbEx).AddressOffset as isize) as *mut STOR_ADDR_BTL8;
        (*storAddrBtl8).Type = (STOR_ADDRESS_TYPE_BTL8) as u16;
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
        (*srbEx).SrbExDataOffset[0] =
            (core::mem::size_of::<STORAGE_REQUEST_BLOCK>() + core::mem::size_of::<STOR_ADDR_BTL8>()) as u32;
        if (*srbEx).SrbExDataOffset[0] as usize + core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() <=
            (*srbEx).SrbLength as usize {
            srbExDataCdb16 = (srbEx as *mut u8).offset(
                (*srbEx).SrbExDataOffset[0] as isize,
            ) as *mut SRBEX_DATA_SCSI_CDB16;
            (*srbExDataCdb16).Type = SrbExDataTypeScsiCdb16;
            (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
            (*srbExDataCdb16).CdbLength = 6;
            cdb = &mut (*srbExDataCdb16).Cdb as *mut u8 as *mut CDB;
        } else {
            if 0 == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                    b"drivers/storage/class/disk/disk.c\0"
                        as *const u8 as *const i8,
                    4123,
                    b"FALSE\0" as *const u8 as *const i8,
                );
                __int2c();
            }
            if !srb.is_null() {
                ExFreePool(srb as *mut core::ffi::c_void);
                srb = core::ptr::null_mut();
            }
            StorDebugPrint(
                2,
                TRACE_FLAG_IOCTL,
                b"DiskIoctlGetMediaTypesEx: Insufficient extended SRB size.\n\0" as *const u8 as *const i8,
            );
            return STATUS_INTERNAL_ERROR;
        }
    } else {
        (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as USHORT;
        (*srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
        (*srb).CdbLength = 6;
        cdb = &mut (*srb).Cdb as *mut u8 as *mut CDB;
        (*srb).TimeOutValue = (*fdoExtension).TimeOutValue;
    }
    (*cdb).CDB6GENERIC.as_mut().OperationCode = SCSIOP_TEST_UNIT_READY;
    status = ClassSendSrbSynchronous(DeviceObject, srb, core::ptr::null_mut(), 0, 0);
    if status >= 0 {
        mediaPresent = 1;
    }
    modeLength = MODE_DATA_SIZE;
    modeData = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, modeLength as SIZE_T, DISK_TAG_MODE_DATA)
        as *mut MODE_PARAMETER_HEADER;
    if modeData.is_null() {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlGetMediaTypesEx: Unable to allocate memory.\n\0" as *const u8 as *const i8,
        );
        if !srb.is_null() {
            ExFreePool(srb as *mut core::ffi::c_void);
            srb = core::ptr::null_mut();
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(modeData as *mut u8, 0, modeLength as usize);
    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        (*srbEx).SrbStatus = 0;
        (*srbExDataCdb16).ScsiStatus = 0;
        (*srbExDataCdb16).CdbLength = 6;
        (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;
    } else {
        (*srb).SrbStatus = 0;
        (*srb).ScsiStatus = 0;
        (*srb).CdbLength = 6;
        (*srb).TimeOutValue = (*fdoExtension).TimeOutValue;
    }
    (*cdb).MODE_SENSE.as_mut().OperationCode = (SCSIOP_MODE_SENSE) as u8;
    (*cdb).MODE_SENSE.as_mut().set_PageCode(MODE_SENSE_RETURN_ALL as u8);
    (*cdb).MODE_SENSE.as_mut().AllocationLength = modeLength as UCHAR;
    loop {
        status = ClassSendSrbSynchronous(
            DeviceObject,
            srb,
            modeData as *mut core::ffi::c_void,
            modeLength,
            0,
        );
        if status == STATUS_VERIFY_REQUIRED {
            let fresh1 = retries;
            retries = retries.wrapping_sub(1);
            if fresh1 == 0 {
                break;
            }
        } else {
            if (*srb).SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32
                == (SRB_STATUS_DATA_OVERRUN) as i32 {
                status = STATUS_SUCCESS;
            }
            break;
        }
    }
    if status >= 0 || status == STATUS_NO_MEDIA_IN_DEVICE {
        if (*modeData).BlockDescriptorLength != 0 {
            blockDescriptor = (modeData as usize)
                .wrapping_add(core::mem::size_of::<MODE_PARAMETER_HEADER>()) as *mut MODE_PARAMETER_BLOCK;
            densityCode = (*blockDescriptor).DensityCode;
        }
        if (*modeData).DeviceSpecificParameter as i32 & MODE_DSP_WRITE_PROTECT != 0 {
            writable = 0;
        }
        status = DiskDetermineMediaTypes(
            DeviceObject,
            (Irp).cast(),
            (*modeData).MediumType,
            densityCode,
            mediaPresent,
            writable,
        );
    } else {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlGetMediaTypesEx: Mode sense for header/bd failed. %lx\n\0" as *const u8 as *const i8,
            status,
        );
    }
    if !srb.is_null() {
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
    }
    if !modeData.is_null() {
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
    }
    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlGetVolumeDiskExtents(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_INVALID_DEVICE_REQUEST;

    // PAGED_CODE expansion (anchor)
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0"
                .as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                5498i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // CHECK_IRQL expansion (anchor)
    if KeGetCurrentIrql() as i32 >= (DISPATCH_LEVEL) as i32 {
        if !((KeGetCurrentIrql() as i32) < 2) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                5499i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0"
                    .as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    StorDebugPrint(
        4i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlGetVolumeDiskExtents: DeviceObject %p Irp %p\n\0"
            .as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*DeviceObject).Characteristics & FILE_REMOVABLE_MEDIA) != 0 {
        if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
            < core::mem::size_of::<VOLUME_DISK_EXTENTS>()
        {
            StorDebugPrint(
                2i32,
                TRACE_FLAG_IOCTL,
                b"DiskIoctlGetVolumeDiskExtents: Output buffer too small.\n\0"
                    .as_ptr() as *const i8,
            );
            return STATUS_BUFFER_TOO_SMALL;
        }

        status = DiskReadDriveCapacity(
            (*(*commonExtension).PartitionZeroExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        );

        (*diskData).ReadyStatus = status;

        if status >= 0 {
            let pVolExt: PVOLUME_DISK_EXTENTS =
                (*Irp).AssociatedIrp.SystemBuffer as PVOLUME_DISK_EXTENTS;

            (*pVolExt).NumberOfDiskExtents = 1u32;
            (*pVolExt).Extents[0].DiskNumber =
                (*(*commonExtension).PartitionZeroExtension).DeviceNumber;
            (*pVolExt).Extents[0].StartingOffset = (*commonExtension).StartingOffset;
            (*pVolExt).Extents[0].ExtentLength = (*commonExtension).PartitionLength;

            (*Irp).IoStatus.Information =
                core::mem::size_of::<VOLUME_DISK_EXTENTS>() as ULONG_PTR;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlIsWritable(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE / CHECK_IRQL expansion (inline)
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                5108 as i32,
                b"0\0" as *const u8 as *const i8,
            );
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            core::arch::asm!("int 0x2C");
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("brk #0xF001");
            // NT_ASSERT resumes after the break; no spin.
        }
    }
    if KeGetCurrentIrql() as i32 >= DISPATCH_LEVEL as i32 {
        if !((KeGetCurrentIrql() as i32) < 2 as i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                5109 as i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0" as *const u8 as *const i8,
            );
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            core::arch::asm!("int 0x2C");
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("brk #0xF001");
            // NT_ASSERT resumes after the break; no spin.
        }
        return STATUS_INVALID_LEVEL;
    }

    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut modeData: PMODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: PCDB = core::ptr::null_mut();
    let mut modeLength: u32 = 0;
    let mut retries: u32 = 4;
    let mut srbSize: u32 = 0;
    let mut srbEx: PSTORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: PSTOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: PSRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlIsWritable: DeviceObject %p Irp %p\n\0" as *const u8 as *const i8,
        DeviceObject as *mut core::ffi::c_void,
        Irp as *mut core::ffi::c_void,
    );

    if (*(*fdoExtension).AdapterDescriptor).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = SCSI_REQUEST_BLOCK_SIZE;
    }

    srb = ExAllocatePoolWithTag(NonPagedPool, srbSize as usize, DISK_TAG_SRB) as PSCSI_REQUEST_BLOCK;
    if srb.is_null() {
        TracePrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlIsWritable: Unable to allocate memory.\n\0" as *const u8 as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(srb as *mut u8, 0, srbSize as usize);

    modeLength = MODE_DATA_SIZE;
    modeData = ExAllocatePoolWithTag(
        NonPagedPoolCacheAligned,
        modeLength as usize,
        DISK_TAG_MODE_DATA,
    ) as PMODE_PARAMETER_HEADER;
    if modeData.is_null() {
        TracePrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlIsWritable: Unable to allocate memory.\n\0" as *const u8 as *const i8,
        );
        if !srb.is_null() {
            ExFreePool(srb as *mut core::ffi::c_void);
            srb = core::ptr::null_mut();
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(modeData as *mut u8, 0, modeLength as usize);

    if (*(*fdoExtension).AdapterDescriptor).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbEx = srb as PSTORAGE_REQUEST_BLOCK;
        (*srbEx).Length = 8; // FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature) expanded
        (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK as u8;
        (*srbEx).Signature = SRB_SIGNATURE;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1;
        (*srbEx).SrbLength = srbSize;
        (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        (*srbEx).RequestPriority = IoGetIoPriorityHint(Irp) as u16;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
        (*srbEx).NumSrbExData = 1;
        (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;

        storAddrBtl8 =
            (srbEx as *mut u8).offset((*srbEx).AddressOffset as isize) as PSTOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8 as u16;
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;

        (*srbEx).SrbExDataOffset[0] =
            (core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as usize)
                .wrapping_add(core::mem::size_of::<STOR_ADDR_BTL8>() as usize) as u32;

        if ((*srbEx).SrbExDataOffset[0] as usize)
            .wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as usize)
            <= (*srbEx).SrbLength as usize
        {
            srbExDataCdb16 = (srbEx as *mut u8)
                .offset((*srbEx).SrbExDataOffset[0] as isize)
                as PSRBEX_DATA_SCSI_CDB16;
            (*srbExDataCdb16).Type = SrbExDataTypeScsiCdb16;
            (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
            (*srbExDataCdb16).CdbLength = 6;
            cdb = &raw mut (*srbExDataCdb16).Cdb as *mut u8 as PCDB;
        } else {
            if 0 == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                    b"drivers/storage/class/disk/disk.c\0"
                        as *const u8 as *const i8,
                    5200 as i32,
                    b"FALSE\0" as *const u8 as *const i8,
                );
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                core::arch::asm!("int 0x2C");
                #[cfg(target_arch = "aarch64")]
                core::arch::asm!("brk #0xF001");
                // NT_ASSERT resumes after the break; no spin.
            }
            if !srb.is_null() {
                ExFreePool(srb as *mut core::ffi::c_void);
                srb = core::ptr::null_mut();
            }
            if !modeData.is_null() {
                ExFreePool(modeData as *mut core::ffi::c_void);
                modeData = core::ptr::null_mut();
            }
            return STATUS_INTERNAL_ERROR;
        }
    } else {
        (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as u16;
        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*srb).CdbLength = 6;
        (*srb).TimeOutValue = (*fdoExtension).TimeOutValue;
        cdb = &raw mut (*srb).Cdb as *mut u8 as PCDB;
    }

    (*cdb).MODE_SENSE.as_mut().OperationCode = SCSIOP_MODE_SENSE as u8;
    (*cdb).MODE_SENSE.as_mut().set_PageCode(MODE_SENSE_RETURN_ALL as u8);
    (*cdb).MODE_SENSE.as_mut().AllocationLength = modeLength as u8;

    while retries != 0 {
        status = ClassSendSrbSynchronous(
            DeviceObject,
            srb,
            modeData as *mut core::ffi::c_void,
            modeLength,
            0,
        );
        if status != STATUS_VERIFY_REQUIRED {
            if (*srb).SrbStatus as i32
                & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32
                == (SRB_STATUS_DATA_OVERRUN) as i32
            {
                status = STATUS_SUCCESS;
            }
            break;
        }
        retries = retries.wrapping_sub(1);
    }

    if status >= 0 {
        if (*modeData).DeviceSpecificParameter as i32 & 0x80 != 0 {
            status = STATUS_MEDIA_WRITE_PROTECTED;
        }
    }

    if !srb.is_null() {
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
    }
    if !modeData.is_null() {
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlPredictFailure(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    // Constants that are not available as imports; caller will provide correct values.
    // Placeholder definitions: these must be overridden by the environment.
    const APC_LEVEL: KIRQL = 1;
    const DISPATCH_LEVEL: KIRQL = 2;
    const STATUS_INVALID_LEVEL: NTSTATUS = 0xC0000148_u32 as i32;
    const FailurePredictionSense: i32 = 3; // classpnp.h:237 (None 0, Ioctl 1, Smart 2, Sense 3);
                                       // 1 is FailurePredictionIoctl, so the SMART-sense
                                       // capability test below could never match
    const FailurePredictionSmart: i32 = 2; // placeholder
    const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const DISK_TAG_SMART: u32 = 0x61446353; // 'aDcS' (disk.h)
    const NonPagedPool: u32 = 0; // placeholder
    const SynchronizationEvent: i32 = 0x01; // placeholder

    // Extern declarations for functions not in the available imports list.
    // CC must match the in-crate definitions exactly (extern "system" for NTAPI defs) —
    // an in-module CC mismatch is UB that LLVM folds to ud2 (the classpnp lesson).
    extern "C" {
        // variadic — matches the crate's other decl and the drvtrace-shim contract
        fn TracePrint(level: i32, flag: u32, format: *const i8, ...);
        fn DbgPrint(format: *const i8, ...) -> NTSTATUS;
        fn __int2c();
        // cdecl per types_i386 (no NTAPI in disk.h) — must match the extern "C" defs.
        fn DiskReadFailurePredictStatus(
            FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
            SmartStatus: *mut STORAGE_FAILURE_PREDICT_STATUS,
        ) -> NTSTATUS;
        fn DiskReadFailurePredictData(
            FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
            Buffer: *mut STORAGE_FAILURE_PREDICT_DATA,
        ) -> NTSTATUS;
    }
    extern "system" {
        fn KeGetCurrentIrql() -> KIRQL;
    }

    let commonExtension: *mut COMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let diskData: *mut DISK_DATA = (*commonExtension).DriverData as *mut DISK_DATA;
    let irpStack: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_SUCCESS;

    let mut checkFailure: *mut STORAGE_PREDICT_FAILURE = core::ptr::null_mut();
    let mut diskSmartStatus: STORAGE_FAILURE_PREDICT_STATUS = core::mem::zeroed();
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut event: KEVENT = core::mem::zeroed();

    // PAGED_CODE() macro expansion
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if (0 as i32) == (0 as i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0" as *const u8 as *const i8,
                4310 as i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    // CHECK_IRQL macro expansion
    if (KeGetCurrentIrql() as i32) >= (DISPATCH_LEVEL as i32) {
        if !((KeGetCurrentIrql() as i32) < (2 as i32)) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0" as *const u8 as *const i8,
                4311 as i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    // Validate the request.
    TracePrint(
        4, TRACE_FLAG_IOCTL,
        b"DiskIoctlPredictFailure: DeviceObject %p Irp %p\n\0" as *const u8 as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<STORAGE_PREDICT_FAILURE>()
    {
        TracePrint(
            2, TRACE_FLAG_IOCTL,
            b"DiskIoctlPredictFailure: Output buffer too small.\n\0" as *const u8 as *const i8,
            // No additional arguments besides the format in this call.
            core::ptr::null_mut::<DEVICE_OBJECT>(),
            core::ptr::null_mut::<IRP>(),
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    checkFailure = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_PREDICT_FAILURE;

    if ((*diskData).FailurePredictionCapability as u32) == (FailurePredictionSense as u32) {
        let mut readBufferSize: u32 = 0;
        let mut readBuffer: *mut u8 = core::ptr::null_mut();
        let mut readIrp: *mut IRP = core::ptr::null_mut();
        let mut topOfStack: *mut DEVICE_OBJECT = core::ptr::null_mut();

        (*checkFailure).PredictFailure = 0u32;

        KeInitializeEvent(&raw mut event, (SynchronizationEvent as u32 as u8) as u32, 0);

        topOfStack = IoGetAttachedDeviceReference(DeviceObject);

        readBufferSize = (*fdoExtension).DiskGeometry.BytesPerSector;
        readBuffer = ExAllocatePoolWithTag(
            NonPagedPool as u32 as u32,
            readBufferSize as usize,
            DISK_TAG_SMART,
        ) as *mut u8;

        if !readBuffer.is_null() {
            let mut offset: LARGE_INTEGER = core::mem::zeroed();
            offset.QuadPart = 0i64;
            readIrp = IoBuildSynchronousFsdRequest(
                IRP_MJ_READ as u32,
                topOfStack,
                readBuffer as *mut c_void,
                readBufferSize,
                &raw mut offset,
                &raw mut event,
                &raw mut ioStatus,
            );

            if !readIrp.is_null() {
                status = IoCallDriver(topOfStack, readIrp);
                if status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        &raw mut event as *mut c_void,
                        Executive,
                        KernelMode as i32 as KPROCESSOR_MODE,
                        0,
                        core::ptr::null_mut(),
                    );
                    status = ioStatus.__bindgen_anon_1.Status;
                }
            } else {
                status = STATUS_INSUFFICIENT_RESOURCES;
            }

            // Inline expansion of FREE_POOL(readBuffer)
            if !readBuffer.is_null() {
                ExFreePool(readBuffer as *mut c_void);
                readBuffer = core::ptr::null_mut();
            }
        } else {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }

        ObDereferenceObject(topOfStack as *mut c_void);
    }

    if status != STATUS_INSUFFICIENT_RESOURCES {
        if ((*diskData).FailurePredictionCapability as u32) == (FailurePredictionSmart as u32)
            || ((*diskData).FailurePredictionCapability as u32) == (FailurePredictionSense as u32)
        {
            status = DiskReadFailurePredictStatus(fdoExtension, &mut diskSmartStatus);
            if status >= 0 {
                status = DiskReadFailurePredictData(
                    fdoExtension,
                    (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_FAILURE_PREDICT_DATA,
                );
                if diskSmartStatus.PredictFailure != 0 {
                    (*checkFailure).PredictFailure = 1u32;
                } else {
                    (*checkFailure).PredictFailure = 0u32;
                }
                (*Irp).IoStatus.Information =
                    core::mem::size_of::<STORAGE_PREDICT_FAILURE>() as ULONG_PTR;
            }
        } else {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlReassignBlocks(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS;
    let badBlocks: PREASSIGN_BLOCKS = (*Irp).AssociatedIrp.SystemBuffer as PREASSIGN_BLOCKS;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: PCDB = core::ptr::null_mut();
    let mut bufferSize: ULONG;
    let mut blockNumber: ULONG;
    let mut blockCount: ULONG;
    let mut srbSize: ULONG;
    let mut srbEx: PSTORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: PSTOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: PSRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();

    // PAGED_CODE(); CHECK_IRQL();
    if KeGetCurrentIrql() as i32 > 1 /* APC_LEVEL */ {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                4694i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    if KeGetCurrentIrql() as i32 >= 2 /* DISPATCH_LEVEL */ {
        if !((KeGetCurrentIrql() as i32) < 2) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                4695i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    StorDebugPrint(
        4i32, // TRACE_LEVEL_INFORMATION
        TRACE_FLAG_IOCTL,
        b"DiskIoctlReassignBlocks: DeviceObject %p Irp %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.InputBufferLength as usize) < core::mem::size_of::<REASSIGN_BLOCKS>() as usize {
        StorDebugPrint(
            2i32, // TRACE_LEVEL_ERROR
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocks: Input buffer length mismatch.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    if (*badBlocks).Count as i32 == 0i32 {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocks: Invalid block count\n\0".as_ptr() as *const i8,
        );
        return STATUS_INVALID_PARAMETER;
    }

    bufferSize = (core::mem::size_of::<REASSIGN_BLOCKS>() as usize)
        .wrapping_add(
            ((*badBlocks).Count as i32 - 1i32) as usize * core::mem::size_of::<ULONG>() as usize,
        ) as ULONG;

    if (*irpStack).Parameters.DeviceIoControl.InputBufferLength < bufferSize {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocks: Input buffer length mismatch for bad blocks.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG;
    } else {
        srbSize = SCSI_REQUEST_BLOCK_SIZE as ULONG;
    }

    srb = ExAllocatePoolWithTag(0 /* NonPagedPool */, srbSize as usize, DISK_TAG_SRB) as PSCSI_REQUEST_BLOCK;
    if srb.is_null() {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocks: Unable to allocate memory.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(srb as *mut u8, 0u8, srbSize as usize);

    (*badBlocks).Reserved = 0u16;
    blockCount = (*badBlocks).Count as ULONG;
    // Convert # of entries to # of bytes (blockCount * 4)
    blockCount = (blockCount).wrapping_mul(4u32);
    (*badBlocks).Count = ((blockCount >> 8) & 0xff) as u16;
    (*badBlocks).Count = ((*badBlocks).Count as u32 | ((blockCount << 8) & 0xff00)) as u16;
    // Convert back to number of entries (blockCount / 4)
    blockCount = blockCount.wrapping_div(4);

    while blockCount > 0u32 {
        let block_ptr = (&raw mut (*badBlocks).BlockNumber as *mut u32)
            .offset((blockCount.wrapping_sub(1)) as isize);
        let bytes = &mut *(block_ptr as *mut [u8; 4]);
        let t0 = bytes[0];
        let t1 = bytes[1];
        bytes[0] = bytes[3];
        bytes[3] = t0;
        bytes[1] = bytes[2];
        bytes[2] = t1;
        blockCount = blockCount.wrapping_sub(1);
    }

    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbEx = srb as PSTORAGE_REQUEST_BLOCK;
        (*srbEx).Length = 8u16; // FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature)
        (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK as u8;
        (*srbEx).Signature = SRB_SIGNATURE as u32;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1 as u32;
        (*srbEx).SrbLength = srbSize;
        (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI as u32;
        (*srbEx).RequestPriority = IoGetIoPriorityHint(Irp) as u16;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
        (*srbEx).NumSrbExData = 1u32;
        (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;

        storAddrBtl8 = (srbEx as *mut u8)
            .offset((*srbEx).AddressOffset as isize) as *mut STOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8 as u16;
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH as u32;

        (*srbEx).SrbExDataOffset[0] = (core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as usize)
            .wrapping_add(core::mem::size_of::<STOR_ADDR_BTL8>() as usize) as u32;

        if ((*srbEx).SrbExDataOffset[0] as usize)
            .wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as usize)
            <= (*srbEx).SrbLength as usize
        {
            srbExDataCdb16 = (srbEx as *mut u8)
                .offset((*srbEx).SrbExDataOffset[0] as isize)
                as *mut SRBEX_DATA_SCSI_CDB16;
            (*srbExDataCdb16).Type = SrbExDataTypeScsiCdb16;
            (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH as u32;
            (*srbExDataCdb16).CdbLength = 6u8;
            cdb = &raw mut (*srbExDataCdb16).Cdb as *mut u8 as *mut CDB;
        } else {
            if 0 == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                    4821i32,
                    b"FALSE\0".as_ptr() as *const i8,
                );
                __int2c();
            }
            if !srb.is_null() {
                ExFreePool(srb as *mut core::ffi::c_void);
                srb = core::ptr::null_mut();
            }
            StorDebugPrint(
                2i32,
                TRACE_FLAG_IOCTL,
                b"DiskIoctlReassignBlocks: Insufficient extended SRB size.\n\0".as_ptr() as *const i8,
            );
            return STATUS_INTERNAL_ERROR;
        }
    } else {
        (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as u16;
        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*srb).CdbLength = 6u8;
        (*srb).TimeOutValue = (*fdoExtension).TimeOutValue;
        cdb = &raw mut (*srb).Cdb as *mut u8 as *mut CDB;
    }

    (*cdb).CDB6GENERIC.as_mut().OperationCode = SCSIOP_REASSIGN_BLOCKS as u8;

    status = ClassSendSrbSynchronous(
        DeviceObject,
        srb,
        badBlocks as *mut core::ffi::c_void,
        bufferSize,
        1u8, // TRUE
    );

    if !srb.is_null() {
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlReassignBlocksEx(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = 0;
    let mut badBlocks: *mut REASSIGN_BLOCKS_EX =
        (*Irp).AssociatedIrp.SystemBuffer as *mut REASSIGN_BLOCKS_EX;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: PCDB = core::ptr::null_mut();
    let mut blockNumber: LARGE_INTEGER = core::mem::zeroed();
    let mut bufferSize: u32 = 0;
    let mut blockCount: u32 = 0;
    let mut srbSize: u32 = 0;
    let mut srbEx: PSTORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: PSTOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: PSRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();

    // CHECK_IRQL expansion (APC_LEVEL = 1, DISPATCH_LEVEL = 2)
    if KeGetCurrentIrql() > 1 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if false {
            // dead condition to preserve NT_ASSERT(FALSE) style
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                4901 as i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    if KeGetCurrentIrql() >= 2 {
        if !(KeGetCurrentIrql() < 2) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                4902 as i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    StorDebugPrint(
        4,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlReassignBlocksEx: DeviceObject %p Irp %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<REASSIGN_BLOCKS_EX>()
    {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocksEx: Input buffer length mismatch.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    if (*badBlocks).Count == 0 {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocksEx: Invalid block count\n\0".as_ptr() as *const i8,
        );
        return STATUS_INVALID_PARAMETER;
    }

    bufferSize = (core::mem::size_of::<REASSIGN_BLOCKS_EX>() as usize)
        .wrapping_add(
            ((*badBlocks).Count as usize).wrapping_sub(1)
                .wrapping_mul(core::mem::size_of::<LARGE_INTEGER>() as usize),
        ) as u32;

    if (*irpStack).Parameters.DeviceIoControl.InputBufferLength < bufferSize {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocksEx: Input buffer length mismatch for bad blocks.\n\0"
                .as_ptr() as *const i8,
        );
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    if (*(*fdoExtension).AdapterDescriptor).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32;
    } else {
        srbSize = SCSI_REQUEST_BLOCK_SIZE as u32;
    }

    srb = ExAllocatePoolWithTag((NonPagedPoolNx) as u32, srbSize as usize, DISK_TAG_SRB as u32)
        as PSCSI_REQUEST_BLOCK;
    if srb.is_null() {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlReassignBlocks: Unable to allocate memory.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(srb as *mut u8, 0, srbSize as usize);

    (*badBlocks).Reserved = 0;
    blockCount = (*badBlocks).Count as u32;

    // Convert # of entries to # of bytes (big endian swap)
    blockCount = blockCount.wrapping_mul(8);
    (*badBlocks).Count = ((blockCount >> 8) & 0xFF) as u16;
    (*badBlocks).Count = ((*badBlocks).Count as u32
        | (((blockCount << 8) & 0xFF00) as u16) as u32) as u16;

    blockCount = blockCount.wrapping_div(8);

    while blockCount > 0 {
        blockNumber = (*((&raw mut (*badBlocks).BlockNumber) as *mut LARGE_INTEGER)
            .offset(blockCount.wrapping_sub(1) as isize)) as LARGE_INTEGER;

        let mut val1: *mut _EIGHT_BYTE = (&raw mut (*badBlocks).BlockNumber as *mut LARGE_INTEGER)
            .offset(blockCount.wrapping_sub(1) as isize) as *mut _EIGHT_BYTE;
        let mut val2: *mut _EIGHT_BYTE = &raw mut blockNumber as *mut _EIGHT_BYTE;

        (*val1).__bindgen_anon_1.as_mut().Byte7 = (*val2).__bindgen_anon_1.as_ref().Byte0;
        (*val1).__bindgen_anon_1.as_mut().Byte6 = (*val2).__bindgen_anon_1.as_ref().Byte1;
        (*val1).__bindgen_anon_1.as_mut().Byte5 = (*val2).__bindgen_anon_1.as_ref().Byte2;
        (*val1).__bindgen_anon_1.as_mut().Byte4 = (*val2).__bindgen_anon_1.as_ref().Byte3;
        (*val1).__bindgen_anon_1.as_mut().Byte3 = (*val2).__bindgen_anon_1.as_ref().Byte4;
        (*val1).__bindgen_anon_1.as_mut().Byte2 = (*val2).__bindgen_anon_1.as_ref().Byte5;
        (*val1).__bindgen_anon_1.as_mut().Byte1 = (*val2).__bindgen_anon_1.as_ref().Byte6;
        (*val1).__bindgen_anon_1.as_mut().Byte0 = (*val2).__bindgen_anon_1.as_ref().Byte7;

        blockCount = blockCount.wrapping_sub(1);
    }

    if (*(*fdoExtension).AdapterDescriptor).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        srbEx = srb as PSTORAGE_REQUEST_BLOCK;
        (*srbEx).Length = 8 as u32 as u16;
        (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK as u8;
        (*srbEx).Signature = SRB_SIGNATURE as u32;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1 as u32;
        (*srbEx).SrbLength = srbSize;
        (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI as u32;
        (*srbEx).RequestPriority = IoGetIoPriorityHint(Irp) as u16;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
        (*srbEx).NumSrbExData = 1;
        (*srbEx).TimeOutValue = (*fdoExtension).TimeOutValue;

        storAddrBtl8 =
            (srbEx as *mut u8).offset((*srbEx).AddressOffset as isize) as PSTOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8 as u16;
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH as u32;

        (*srbEx).SrbExDataOffset[0] =
            (core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as usize)
                .wrapping_add(core::mem::size_of::<STOR_ADDR_BTL8>() as usize) as u32;

        if ((*srbEx).SrbExDataOffset[0] as usize)
            .wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as usize)
            <= (*srbEx).SrbLength as usize
        {
            srbExDataCdb16 = (srbEx as *mut u8).offset(
                (*srbEx).SrbExDataOffset[0] as isize,
            ) as PSRBEX_DATA_SCSI_CDB16;
            (*srbExDataCdb16).Type = SrbExDataTypeScsiCdb16;
            (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH as u32;
            (*srbExDataCdb16).CdbLength = 6;
            cdb = &raw mut (*srbExDataCdb16).Cdb as *mut u8 as PCDB;
        } else {
            // NT_ASSERT(FALSE) expansion
            if false {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/disk.c\0"
                        .as_ptr() as *const i8,
                    5028 as i32,
                    b"FALSE\0".as_ptr() as *const i8,
                );
                __int2c();
            }
            if !srb.is_null() {
                ExFreePool(srb as *mut core::ffi::c_void);
                srb = core::ptr::null_mut();
            }
            StorDebugPrint(
                2,
                TRACE_FLAG_IOCTL,
                b"DiskIoctlReassignBlocks: Insufficient extended SRB size.\n\0".as_ptr()
                    as *const i8,
            );
            return STATUS_INTERNAL_ERROR;
        }
    } else {
        (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as u16;
        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*srb).CdbLength = 6;
        (*srb).TimeOutValue = (*fdoExtension).TimeOutValue;
        cdb = &raw mut (*srb).Cdb as *mut u8 as PCDB;
    }

    (*cdb).CDB6GENERIC.as_mut().OperationCode = SCSIOP_REASSIGN_BLOCKS as u8;
    (*cdb)
        .CDB6GENERIC
        .as_mut()
        .set_CommandUniqueBits(1 as u8);

    status = ClassSendSrbSynchronous(
        DeviceObject,
        srb,
        badBlocks as *mut core::ffi::c_void,
        bufferSize,
        1, // TRUE as BOOLEAN
    );

    if !srb.is_null() {
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlSetCacheInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let cacheInfo: PDISK_CACHE_INFORMATION =
        (*Irp).AssociatedIrp.SystemBuffer as PDISK_CACHE_INFORMATION;
    let status: NTSTATUS;

    // CHECK_IRQL macro expansion
    if KeGetCurrentIrql() as i32 > 1i32 {
        // APC_LEVEL = 1
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/disk/disk.c\0"
                .as_ptr() as *const i8,
            3959i32,
            b"0\0".as_ptr() as *const i8,
        );
        __int2c();
    }
    if KeGetCurrentIrql() as i32 >= 2i32 {
        // DISPATCH_LEVEL = 2
        if !((KeGetCurrentIrql() as i32) < 2i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                3960i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return 0xC0000148u32 as i32; // STATUS_INVALID_LEVEL
    }

    // TracePrint (actually StorDebugPrint)
    StorDebugPrint(
        4i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlSetCacheInformation: DeviceObject %p Irp %p\n\0".as_ptr() as PCCHAR,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<DISK_CACHE_INFORMATION>()
    {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSetCacheInformation: Input buffer length mismatch.\n\0".as_ptr()
                as PCCHAR,
        );
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    status = DiskSetCacheInformation(fdoExtension, cacheInfo);

    if (*cacheInfo).WriteCacheEnabled != 0 {
        (*diskData).WriteCacheOverride = 1; // DiskWriteCacheEnable
    } else {
        (*diskData).WriteCacheOverride = 0; // DiskWriteCacheDisable
    }

    ClassSetDeviceParameter(
        fdoExtension,
        DiskDeviceParameterSubkey.as_ptr() as PWSTR,
        DiskDeviceUserWriteCacheSetting.as_ptr() as PWSTR,
        (*diskData).WriteCacheOverride as ULONG,
    );

    DiskLogCacheInformation(fdoExtension, cacheInfo, status);

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlSetCacheSetting(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_SUCCESS;
    if (KeGetCurrentIrql() as i32) > APC_LEVEL as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                3450i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    if (KeGetCurrentIrql() as i32) >= DISPATCH_LEVEL as i32 {
        if !((KeGetCurrentIrql() as i32) < 2i32) {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
                b"drivers/storage/class/disk/disk.c\0".as_ptr() as *const i8,
                3451i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }
    if ((*irpSp).Parameters.DeviceIoControl.InputBufferLength as usize) < core::mem::size_of::<DISK_CACHE_SETTING>() {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else {
        let cacheSetting: PDISK_CACHE_SETTING = (*Irp).AssociatedIrp.SystemBuffer as PDISK_CACHE_SETTING;
        if (*cacheSetting).Version as usize == core::mem::size_of::<DISK_CACHE_SETTING>() {
            let mut isPowerProtected: ULONG;
            if (*cacheSetting).IsPowerProtected != 0 {
                (*fdoExtension).DeviceFlags = ((*fdoExtension).DeviceFlags as i32 | 0x10) as u16;
                isPowerProtected = 1u32;
            } else {
                (*fdoExtension).DeviceFlags = ((*fdoExtension).DeviceFlags as i32 & !0x10) as u16;
                isPowerProtected = 0u32;
            }
            if ((*fdoExtension).DeviceFlags as i32 & 0x1) != 0
                && !((*fdoExtension).DeviceFlags as i32 & 0x10 != 0)
                && !((*fdoExtension).ScanForSpecialFlags as u32 & 0x80 != 0)
            {
                (*fdoExtension).CdbForceUnitAccess = 1u8;
            } else {
                (*fdoExtension).CdbForceUnitAccess = 0u8;
            }
            ClassSetDeviceParameter(
                fdoExtension,
                DiskDeviceParameterSubkey.as_ptr() as PWSTR,
                DiskDeviceCacheIsPowerProtected.as_ptr() as PWSTR,
                isPowerProtected,
            );
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    }
    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlSetVerify(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_NOT_SUPPORTED;
    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlSetVerify: DeviceObject %p Irp %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );
    if (*Irp).RequestorMode as u32 == (KernelMode) as u32 {
        (*DeviceObject).Flags |= DO_VERIFY_VOLUME as u32;
        status = STATUS_SUCCESS;
    }
    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlSmartGetVersion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // Local declarations
    let commonExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = 0;
    let mut versionParams: PGETVERSIONINPARAMS = core::ptr::null_mut::<GETVERSIONINPARAMS>();
    let mut srbControl: PSRB_IO_CONTROL = core::ptr::null_mut::<SRB_IO_CONTROL>();
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut buffer: PUCHAR = core::ptr::null_mut::<u8>();

    // PAGED_CODE() and CHECK_IRQL() expansions
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        // assertion equivalent
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
            b"drivers/storage/class/disk/disk.c\0"
                .as_ptr() as *const i8,
            5582i32,
            b"0\0".as_ptr() as *const i8,
        );
        __int2c();
    }
    if KeGetCurrentIrql() as i32 >= (DISPATCH_LEVEL) as i32 {
        if (KeGetCurrentIrql() as i32) < 2i32 {
            // unreachable if condition true, but present in anchor
        } else {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                5583i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    // Trace output
    StorDebugPrint(
        4i32, // TRACE_LEVEL_INFORMATION
        TRACE_FLAG_IOCTL,
        b"DiskIoctlSmartGetVersion: DeviceObject %p Irp %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    // Validate output buffer size
    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<GETVERSIONINPARAMS>()
    {
        StorDebugPrint(
            2i32, // TRACE_LEVEL_ERROR
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartGetVersion: Output buffer too small.\n\0".as_ptr() as *const i8,
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    // Allocate SRB + version params
    srbControl = ExAllocatePoolWithTag(
        NonPagedPoolNx as u32,
        (core::mem::size_of::<SRB_IO_CONTROL>() as usize)
            .wrapping_add(core::mem::size_of::<GETVERSIONINPARAMS>() as usize),
        DISK_TAG_SMART,
    ) as PSRB_IO_CONTROL;

    if srbControl.is_null() {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartGetVersion: Unable to allocate memory.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Zero the allocated block
    core::ptr::write_bytes(
        srbControl as *mut u8,
        0u8,
        core::mem::size_of::<SRB_IO_CONTROL>().wrapping_add(core::mem::size_of::<GETVERSIONINPARAMS>()),
    );

    // Fill in srbControl fields
    (*srbControl).HeaderLength = core::mem::size_of::<SRB_IO_CONTROL>() as u32;
    // RtlMoveMemory(Signature, "SCSIDISK", 8) — 8 bytes copy
    core::ptr::copy_nonoverlapping(
        b"SCSIDISK\0".as_ptr() as *const u8 as *const core::ffi::c_void,
        &raw mut (*srbControl).Signature as *mut u8 as *mut core::ffi::c_void,
        8usize,
    );
    (*srbControl).Timeout = (*fdoExtension).TimeOutValue;
    (*srbControl).Length = core::mem::size_of::<GETVERSIONINPARAMS>() as u32;
    (*srbControl).ControlCode = IOCTL_SCSI_MINIPORT_SMART_VERSION as u32;

    // Calculate buffer pointer (after SRB header)
    buffer = (srbControl as *mut u8).offset((*srbControl).HeaderLength as isize);
    versionParams = buffer as PGETVERSIONINPARAMS;
    (*versionParams).bIDEDeviceMap = (*diskData).ScsiAddress.TargetId;

    // Send synchronous IOCTL
    ClassSendDeviceIoControlSynchronous(
        IOCTL_SCSI_MINIPORT as u32,
        (*commonExtension).LowerDeviceObject,
        srbControl as *mut core::ffi::c_void,
        (core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<GETVERSIONINPARAMS>()) as u32,
        (core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<GETVERSIONINPARAMS>()) as u32,
        (0i32) as u8, // FALSE as BOOLEAN
        &raw mut ioStatus,
    );

    status = ioStatus.__bindgen_anon_1.Status;

    if status >= 0 {
        // Success
        buffer = (srbControl as *mut u8).offset((*srbControl).HeaderLength as isize);
        core::ptr::copy_nonoverlapping(
            buffer as *const core::ffi::c_void,
            (*Irp).AssociatedIrp.SystemBuffer as *mut core::ffi::c_void,
            core::mem::size_of::<GETVERSIONINPARAMS>(),
        );
        (*Irp).IoStatus.Information = core::mem::size_of::<GETVERSIONINPARAMS>() as ULONG_PTR;
    }

    // FREE_POOL(srbControl) — ExFreePool and set to null
    if !srbControl.is_null() {
        ExFreePool(srbControl as *mut core::ffi::c_void);
        srbControl = core::ptr::null_mut::<SRB_IO_CONTROL>();
    }

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlSmartReceiveDriveData(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                5709i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }
    // CHECK_IRQL() expansion
    if KeGetCurrentIrql() as i32 >= (DISPATCH_LEVEL) as i32 {
        if !((KeGetCurrentIrql() as i32) < (DISPATCH_LEVEL) as i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    as *const u8 as *const i8,
                5710i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    let commonExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_INVALID_PARAMETER;

    let cmdInParameters: PSENDCMDINPARAMS =
        (*Irp).AssociatedIrp.SystemBuffer as PSENDCMDINPARAMS;
    let mut srbControl: PSRB_IO_CONTROL = core::ptr::null_mut::<SRB_IO_CONTROL>();
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut controlCode: ULONG = 0;
    let mut buffer: PUCHAR = core::ptr::null_mut::<u8>();
    let mut irp2: PIRP = core::ptr::null_mut::<IRP>();
    let mut event: KEVENT = core::mem::zeroed();
    let mut length: ULONG = 0;

    StorDebugPrint(
        4i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlSmartReceiveDriveData: DeviceObject %p Irp %p\n\0" as *const u8 as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < (core::mem::size_of::<SENDCMDINPARAMS>() as usize).wrapping_sub(1)
    {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartReceiveDriveData: Input buffer length invalid.\n\0" as *const u8 as *const i8,
        );
        return STATUS_INVALID_PARAMETER;
    }

    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize)
            .wrapping_add(512)
            .wrapping_sub(1)
    {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartReceiveDriveData: Output buffer too small.\n\0" as *const u8 as *const i8,
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(event),
        NotificationEvent,
        0u8,
    );

    if (*cmdInParameters).irDriveRegs.bCommandReg as i32 == (ID_CMD) as i32 {
        length = (IDENTIFY_BUFFER_SIZE as usize)
            .wrapping_add(core::mem::size_of::<SENDCMDOUTPARAMS>() as usize) as ULONG;
        controlCode = IOCTL_SCSI_MINIPORT_IDENTIFY as ULONG;
    } else if (*cmdInParameters).irDriveRegs.bCommandReg as i32 == (SMART_CMD) as i32 {
        const FailurePredictionSmart: u32 = FAILURE_PREDICTION_METHOD_FailurePredictionSmart;
        let features_reg = (*cmdInParameters).irDriveRegs.bFeaturesReg as i32;
        match features_reg {
            x if x == READ_ATTRIBUTES as i32 => {
                controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS as ULONG;
                length = (READ_ATTRIBUTE_BUFFER_SIZE as usize)
                    .wrapping_add(core::mem::size_of::<SENDCMDOUTPARAMS>() as usize) as ULONG;
            }
            x if x == READ_THRESHOLDS as i32 => {
                controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS as ULONG;
                length = (READ_THRESHOLD_BUFFER_SIZE as usize)
                    .wrapping_add(core::mem::size_of::<SENDCMDOUTPARAMS>() as usize) as ULONG;
            }
            x if x == SMART_READ_LOG as i32 => {
                if (*diskData).FailurePredictionCapability != FailurePredictionSmart {
                    StorDebugPrint(
                        2i32,
                        TRACE_FLAG_IOCTL,
                        b"DiskIoctlSmartReceiveDriveData: SMART failure prediction not supported.\n\0"
                            as *const u8 as *const i8,
                    );
                    return STATUS_INVALID_DEVICE_REQUEST;
                }

                length = ((*cmdInParameters).irDriveRegs.bSectorCountReg as i32
                    * (SMART_LOG_SECTOR_SIZE) as i32) as ULONG;

                if length == 0 {
                    return STATUS_INVALID_PARAMETER;
                }

                length = (length as u64).wrapping_add(
                    if core::mem::size_of::<SENDCMDOUTPARAMS>() > core::mem::size_of::<SENDCMDINPARAMS>() {
                        core::mem::size_of::<SENDCMDOUTPARAMS>() as u64
                    } else {
                        core::mem::size_of::<SENDCMDINPARAMS>() as u64
                    },
                ) as ULONG;

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength
                    < length.wrapping_sub(1)
                {
                    StorDebugPrint(
                        2i32,
                        TRACE_FLAG_IOCTL,
                        b"DiskIoctlSmartReceiveDriveData: Output buffer too small for SMART_READ_LOG.\n\0"
                            as *const u8 as *const i8,
                    );
                    return STATUS_BUFFER_TOO_SMALL;
                }
                controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_LOG as ULONG;
            }
            _ => {}
        }
    }

    if controlCode == 0 {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartReceiveDriveData: Invalid request.\n\0" as *const u8 as *const i8,
        );
        return STATUS_INVALID_PARAMETER;
    }

    srbControl = ExAllocatePoolWithTag(
        NonPagedPool,
        (core::mem::size_of::<SRB_IO_CONTROL>() as SIZE_T).wrapping_add(length as SIZE_T),
        DISK_TAG_SMART,
    ) as PSRB_IO_CONTROL;

    if srbControl.is_null() {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartReceiveDriveData: Unable to allocate memory.\n\0" as *const u8 as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*srbControl).HeaderLength = core::mem::size_of::<SRB_IO_CONTROL>() as ULONG;
    memmove(
        core::ptr::addr_of_mut!((*srbControl).Signature) as *mut u8 as *mut core::ffi::c_void,
        b"SCSIDISK\0" as *const u8 as *const i8 as *const core::ffi::c_void,
        8usize,
    );
    (*srbControl).Timeout = (*fdoExtension).TimeOutValue;
    (*srbControl).Length = length;
    (*srbControl).ControlCode = controlCode;

    buffer = (srbControl as PUCHAR).offset((*srbControl).HeaderLength as isize);

    (*cmdInParameters).bDriveNumber = (*diskData).ScsiAddress.TargetId;

    memmove(
        buffer as *mut core::ffi::c_void,
        (*Irp).AssociatedIrp.SystemBuffer as *const core::ffi::c_void,
        (core::mem::size_of::<SENDCMDINPARAMS>() as usize).wrapping_sub(1),
    );

    irp2 = IoBuildDeviceIoControlRequest(
        IOCTL_SCSI_MINIPORT as ULONG,
        (*commonExtension).LowerDeviceObject,
        srbControl as PVOID,
        (core::mem::size_of::<SRB_IO_CONTROL>() as usize)
            .wrapping_add(core::mem::size_of::<SENDCMDINPARAMS>() as usize)
            .wrapping_sub(1) as ULONG,
        srbControl as PVOID,
        (core::mem::size_of::<SRB_IO_CONTROL>() as usize).wrapping_add(length as usize) as ULONG,
        0u8,
        core::ptr::addr_of_mut!(event),
        core::ptr::addr_of_mut!(ioStatus),
    );

    if irp2.is_null() {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartReceiveDriveData: Unable to allocate IRP.\n\0" as *const u8 as *const i8,
        );
        if !srbControl.is_null() {
            ExFreePool(srbControl as *mut core::ffi::c_void);
            srbControl = core::ptr::null_mut::<SRB_IO_CONTROL>();
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = IoCallDriver((*commonExtension).LowerDeviceObject, irp2);

    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(event) as PVOID,
            Executive,
            KernelMode,
            0u8,
            core::ptr::null_mut::<LARGE_INTEGER>(),
        );
        status = ioStatus.__bindgen_anon_1.Status;
    }

    buffer = (srbControl as PUCHAR).offset((*srbControl).HeaderLength as isize);

    if status >= 0 {
        memmove(
            (*Irp).AssociatedIrp.SystemBuffer as *mut core::ffi::c_void,
            buffer as *const core::ffi::c_void,
            (length.wrapping_sub(1)) as usize,
        );
        (*Irp).IoStatus.Information = length.wrapping_sub(1) as ULONG_PTR;
    } else {
        memmove(
            (*Irp).AssociatedIrp.SystemBuffer as *mut core::ffi::c_void,
            buffer as *const core::ffi::c_void,
            (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize).wrapping_sub(1),
        );
        (*Irp).IoStatus.Information = (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize).wrapping_sub(1) as ULONG_PTR;
    }

    if !srbControl.is_null() {
        ExFreePool(srbControl as *mut core::ffi::c_void);
        srbControl = core::ptr::null_mut::<SRB_IO_CONTROL>();
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlSmartSendDriveCommand(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // Constants local to this function
    const APC_LVL: u8 = 1;
    const DISP_LVL: u8 = 2;
    const SMART_CMD_B: u8 = 0xB0;
    const SMART_WRITE_LOG_B: u8 = 0xD5;
    const ENABLE_SMART_B: u8 = 0xD8;
    const DISABLE_SMART_B: u8 = 0xD9;
    const RETURN_SMART_STATUS_B: u8 = 0xDA;
    const ENABLE_DISABLE_AUTOSAVE_B: u8 = 0xD2;
    const SAVE_ATTRIBUTE_VALUES_B: u8 = 0xD3;
    const EXECUTE_OFFLINE_DIAGS_B: u8 = 0xD4;
    const ENABLE_DISABLE_AUTO_OFFLINE_B: u8 = 0xDB;
    const SMART_LOG_SECTOR_SIZE_L: i32 = 512;
    const FailurePredictionSmart_L: u32 = FAILURE_PREDICTION_METHOD_FailurePredictionSmart;
    const IOCTL_SCSI_MINIPORT_L: u32 = 0x0004D008;
    const IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG_L: u32 = 0x0004D012;
    const IOCTL_SCSI_MINIPORT_ENABLE_SMART_L: u32 = 0x0004D013;
    const IOCTL_SCSI_MINIPORT_DISABLE_SMART_L: u32 = 0x0004D014;
    const IOCTL_SCSI_MINIPORT_RETURN_STATUS_L: u32 = 0x0004D015;
    const IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE_L: u32 = 0x0004D016;
    const IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES_L: u32 = 0x0004D017;
    const IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS_L: u32 = 0x0004D018;
    const IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTO_OFFLINE_L: u32 = 0x0004D019;
    const SMART_OFFLINE_ROUTINE_OFFLINE_L: u8 = 0x00;
    const SMART_SHORT_SELFTEST_OFFLINE_L: u8 = 0x01;
    const SMART_EXTENDED_SELFTEST_OFFLINE_L: u8 = 0x02;
    const NonPagedPool: u32 = 0; // POOL_TYPE::NonPagedPool

    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let diskData = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_INVALID_PARAMETER;
    let cmdInParameters = (*Irp).AssociatedIrp.SystemBuffer as PSENDCMDINPARAMS;
    let mut srbControl: PSRB_IO_CONTROL = core::ptr::null_mut();
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut controlCode: u32 = 0;
    let mut buffer: PUCHAR = core::ptr::null_mut();
    let mut irp2: PIRP = core::ptr::null_mut();
    let mut event: KEVENT = core::mem::zeroed();
    let mut length: ULONG = 0;

    // CHECK_IRQL expansion
    if KeGetCurrentIrql() as i32 > (APC_LVL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                5936i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    if KeGetCurrentIrql() as i32 >= (DISP_LVL) as i32 {
        if !((KeGetCurrentIrql() as i32) < 2) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                5937i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlSmartSendDriveCommand: DeviceObject %p Irp %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < (core::mem::size_of::<SENDCMDINPARAMS>().wrapping_sub(1))
    {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartSendDriveCommand: Input buffer size invalid.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INVALID_PARAMETER;
    }
    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < (core::mem::size_of::<SENDCMDOUTPARAMS>().wrapping_sub(1))
    {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartSendDriveCommand: Output buffer too small.\n\0".as_ptr() as *const i8,
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    KeInitializeEvent(&raw mut event, NotificationEvent, 0u8);

    if (*cmdInParameters).irDriveRegs.bCommandReg == SMART_CMD_B {
        match (*cmdInParameters).irDriveRegs.bFeaturesReg {
            SMART_WRITE_LOG_B => {
                if (*diskData).FailurePredictionCapability as u32 != FailurePredictionSmart_L {
                    TracePrint(
                        (TRACE_LEVEL_ERROR) as i32,
                        TRACE_FLAG_IOCTL,
                        b"DiskIoctlSmartSendDriveCommand: SMART failure prediction not supported.\n\0"
                            .as_ptr() as *const i8,
                    );
                    return STATUS_INVALID_DEVICE_REQUEST;
                }
                length = ((*cmdInParameters).irDriveRegs.bSectorCountReg as i32
                    * SMART_LOG_SECTOR_SIZE_L) as u32;
                if length == 0 {
                    return STATUS_INVALID_PARAMETER;
                }
                if ((*irpStack).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < (core::mem::size_of::<SENDCMDINPARAMS>()
                        .wrapping_sub(1)
                        .wrapping_add(length as usize))
                {
                    TracePrint(
                        (TRACE_LEVEL_ERROR) as i32,
                        TRACE_FLAG_IOCTL,
                        b"DiskIoctlSmartSendDriveCommand: Input buffer too small for SMART_WRITE_LOG.\n\0"
                            .as_ptr() as *const i8,
                    );
                    return STATUS_BUFFER_TOO_SMALL;
                }
                controlCode = IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG_L;
            }
            ENABLE_SMART_B => controlCode = IOCTL_SCSI_MINIPORT_ENABLE_SMART_L,
            DISABLE_SMART_B => controlCode = IOCTL_SCSI_MINIPORT_DISABLE_SMART_L,
            RETURN_SMART_STATUS_B => {
                if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    < (core::mem::size_of::<SENDCMDOUTPARAMS>()
                        .wrapping_sub(1)
                        .wrapping_add(core::mem::size_of::<IDEREGS>()))
                {
                    return STATUS_BUFFER_TOO_SMALL;
                }
                controlCode = IOCTL_SCSI_MINIPORT_RETURN_STATUS_L;
                length = core::mem::size_of::<IDEREGS>() as u32;
            }
            ENABLE_DISABLE_AUTOSAVE_B => controlCode = IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE_L,
            SAVE_ATTRIBUTE_VALUES_B => controlCode = IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES_L,
            EXECUTE_OFFLINE_DIAGS_B => {
                if (*cmdInParameters).irDriveRegs.bSectorNumberReg == SMART_OFFLINE_ROUTINE_OFFLINE_L
                    || (*cmdInParameters).irDriveRegs.bSectorNumberReg == SMART_SHORT_SELFTEST_OFFLINE_L
                    || (*cmdInParameters).irDriveRegs.bSectorNumberReg == SMART_EXTENDED_SELFTEST_OFFLINE_L
                {
                    controlCode = IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS_L;
                }
            }
            ENABLE_DISABLE_AUTO_OFFLINE_B => {
                controlCode = IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTO_OFFLINE_L;
            }
            _ => {}
        }
    }

    if controlCode == 0 {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartSendDriveCommand: Invalid request.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INVALID_PARAMETER;
    }

    length = (length as usize).wrapping_add(
        if core::mem::size_of::<SENDCMDOUTPARAMS>() > core::mem::size_of::<SENDCMDINPARAMS>() {
            core::mem::size_of::<SENDCMDOUTPARAMS>()
        } else {
            core::mem::size_of::<SENDCMDINPARAMS>()
        },
    ) as u32;

    srbControl = ExAllocatePoolWithTag(
        NonPagedPool,
        (core::mem::size_of::<SRB_IO_CONTROL>() as usize).wrapping_add(length as usize),
        DISK_TAG_SMART,
    ) as PSRB_IO_CONTROL;

    if srbControl.is_null() {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartSendDriveCommand: Unable to allocate memory.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*srbControl).HeaderLength = core::mem::size_of::<SRB_IO_CONTROL>() as u32;
    core::ptr::copy(
        b"SCSIDISK\0".as_ptr() as *const u8,
        &raw mut (*srbControl).Signature as *mut u8,
        8,
    );
    (*srbControl).Timeout = (*fdoExtension).TimeOutValue;
    (*srbControl).Length = length;
    (*srbControl).ControlCode = controlCode;

    buffer = (srbControl as PUCHAR).offset((*srbControl).HeaderLength as isize);
    (*cmdInParameters).bDriveNumber = (*diskData).ScsiAddress.TargetId;

    if (*cmdInParameters).irDriveRegs.bFeaturesReg == SMART_WRITE_LOG_B {
        core::ptr::copy(
            (*Irp).AssociatedIrp.SystemBuffer as *const u8,
            buffer,
            (core::mem::size_of::<SENDCMDINPARAMS>().wrapping_sub(1))
                .wrapping_add(
                    ((*cmdInParameters).irDriveRegs.bSectorCountReg as i32 * 512) as usize,
                ),
        );
    } else {
        core::ptr::copy(
            (*Irp).AssociatedIrp.SystemBuffer as *const u8,
            buffer,
            core::mem::size_of::<SENDCMDINPARAMS>().wrapping_sub(1),
        );
    }

    irp2 = IoBuildDeviceIoControlRequest(
        IOCTL_SCSI_MINIPORT_L,
        (*commonExtension).LowerDeviceObject,
        srbControl as PVOID,
        (core::mem::size_of::<SRB_IO_CONTROL>() as usize)
            .wrapping_add(length as usize) as u32,
        srbControl as PVOID,
        (core::mem::size_of::<SRB_IO_CONTROL>() as usize)
            .wrapping_add(length as usize) as u32,
        0u8,
        &raw mut event,
        &raw mut ioStatus,
    );

    if irp2.is_null() {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlSmartSendDriveCommand: Unable to allocate IRP.\n\0".as_ptr() as *const i8,
        );
        if !srbControl.is_null() {
            ExFreePool(srbControl as PVOID);
            srbControl = core::ptr::null_mut();
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = IoCallDriver((*commonExtension).LowerDeviceObject, irp2);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut event as PVOID,
            Executive,
            KernelMode,
            0u8,
            core::ptr::null_mut::<LARGE_INTEGER>(),
        );
        status = ioStatus.__bindgen_anon_1.Status;
    }

    buffer = (srbControl as PUCHAR).offset((*srbControl).HeaderLength as isize);
    if (*cmdInParameters).irDriveRegs.bFeaturesReg == RETURN_SMART_STATUS_B {
        length = (core::mem::size_of::<SENDCMDOUTPARAMS>()
            .wrapping_sub(1)
            .wrapping_add(core::mem::size_of::<IDEREGS>())) as u32;
    } else {
        length = core::mem::size_of::<SENDCMDOUTPARAMS>().wrapping_sub(1) as u32;
    }
    core::ptr::copy(
        buffer as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        length as usize,
    );
    (*Irp).IoStatus.Information = length as ULONG_PTR;

    if !srbControl.is_null() {
        ExFreePool(srbControl as PVOID);
        srbControl = core::ptr::null_mut();
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlUpdateDriveSize(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Notification: TARGET_DEVICE_CUSTOM_NOTIFICATION = core::mem::zeroed();
    let mut status: NTSTATUS = 0;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCCHAR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                5403i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }
    // CHECK_IRQL() expansion
    if KeGetCurrentIrql() as i32 >= (DISPATCH_LEVEL) as i32 {
        if !((KeGetCurrentIrql() as i32) < (DISPATCH_LEVEL) as i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR,
                b"drivers/storage/class/disk/disk.c\0"
                    .as_ptr() as *const i8,
                5404i32,
                b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        return STATUS_INVALID_LEVEL;
    }

    // Tracing
    TracePrint(
        4i32,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlUpdateDriveSize: DeviceObject %p Irp %p\n\0".as_ptr() as PCCHAR,
        DeviceObject,
        Irp,
    );

    // Validate output buffer length
    if ((*irpStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
        < core::mem::size_of::<DISK_GEOMETRY>()
    {
        TracePrint(
            2i32,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlUpdateDriveSize: Output buffer too small.\n\0".as_ptr() as PCCHAR,
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    status = DiskReadDriveCapacity(DeviceObject);
    (*diskData).ReadyStatus = status;

    if status >= 0 {
        // Copy drive geometry information from the device extension
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*fdoExtension).DiskGeometry) as *const core::ffi::c_void,
            (*Irp).AssociatedIrp.SystemBuffer as *mut core::ffi::c_void,
            core::mem::size_of::<DISK_GEOMETRY>(),
        );

        let geom = &mut *((*Irp).AssociatedIrp.SystemBuffer as *mut DISK_GEOMETRY);
        if geom.BytesPerSector == 0 {
            geom.BytesPerSector = 512;
        }

        (*Irp).IoStatus.Information = core::mem::size_of::<DISK_GEOMETRY>() as ULONG_PTR;
        status = STATUS_SUCCESS;

        // Notify everyone that the disk layout may have changed
        Notification.Event = GUID_IO_DISK_LAYOUT_CHANGE;
        Notification.Version = 1;
        Notification.Size = 36u16;   // FIELD_OFFSET(TARGET_DEVICE_CUSTOM_NOTIFICATION, CustomDataBuffer)
        Notification.FileObject = core::ptr::null_mut();
        Notification.NameBufferOffset = -1;
        IoReportTargetDeviceChangeAsynchronous(
            (*fdoExtension).LowerPdo,
            &raw mut Notification as PVOID,
            None,
            core::ptr::null_mut(),
        );
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DiskIoctlVerify(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut commonExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let mut fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut verifyInfo: PVERIFY_INFORMATION =
        (*Irp).AssociatedIrp.SystemBuffer as PVERIFY_INFORMATION;
    let mut Context: PDISK_VERIFY_WORKITEM_CONTEXT =
        core::ptr::null_mut::<DISK_VERIFY_WORKITEM_CONTEXT>();
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut::<SCSI_REQUEST_BLOCK>();
    let mut byteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut srbSize: ULONG = 0;

    StorDebugPrint(
        4,
        TRACE_FLAG_IOCTL,
        b"DiskIoctlVerify: DeviceObject %p Irp %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    if ((*irpStack).Parameters.DeviceIoControl.InputBufferLength as usize)
        < core::mem::size_of::<VERIFY_INFORMATION>() as usize
    {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlVerify: Input buffer length mismatch.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    if (*(*fdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG;
    } else {
        srbSize = SCSI_REQUEST_BLOCK_SIZE as ULONG;
    }

    srb = ExAllocatePoolWithTag(
        NonPagedPool,
        srbSize as SIZE_T,
        DISK_TAG_SRB as ULONG,
    ) as PSCSI_REQUEST_BLOCK;

    if srb.is_null() {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlVerify: Unable to allocate memory.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(srb as *mut u8, 0, srbSize as usize);

    byteOffset.QuadPart =
        (*commonExtension).StartingOffset.QuadPart + (*verifyInfo).StartingOffset.QuadPart;

    if (*verifyInfo).StartingOffset.QuadPart > (*commonExtension).PartitionLength.QuadPart
        || (*verifyInfo).StartingOffset.QuadPart < 0
    {
        StorDebugPrint(
            2,
            TRACE_FLAG_IOCTL,
            b"DiskIoctlVerify: Verify request to invalid sector.\n\0".as_ptr() as *const i8,
        );
        if !srb.is_null() {
            ExFreePool(srb as PVOID);
            srb = core::ptr::null_mut();
        }
        return STATUS_NONEXISTENT_SECTOR;
    } else {
        let mut bytesRemaining: ULONGLONG = ((*commonExtension).PartitionLength.QuadPart
            - (*verifyInfo).StartingOffset.QuadPart)
            as ULONGLONG;
        if (*verifyInfo).Length as ULONGLONG > bytesRemaining {
            StorDebugPrint(
                2,
                TRACE_FLAG_IOCTL,
                b"DiskIoctlVerify: Verify request to invalid sector.\n\0".as_ptr() as *const i8,
            );
            if !srb.is_null() {
                ExFreePool(srb as PVOID);
                srb = core::ptr::null_mut();
            }
            return STATUS_NONEXISTENT_SECTOR;
        }
    }

    const DISK_TAG_WI_CONTEXT: u32 = 0x57446353; // 'DisI'
    Context = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<DISK_VERIFY_WORKITEM_CONTEXT>() as SIZE_T,
        DISK_TAG_WI_CONTEXT as ULONG,
    ) as PDISK_VERIFY_WORKITEM_CONTEXT;

    if !Context.is_null() {
        (*Context).Irp = Irp;
        (*Context).Srb = srb;
        (*Context).WorkItem = IoAllocateWorkItem(DeviceObject);

        if !(*Context).WorkItem.is_null() {
            IoMarkIrpPending(Irp);
            IoQueueWorkItem(
                (*Context).WorkItem,
                Some(core::mem::transmute::<
                    unsafe extern "system" fn(PDEVICE_OBJECT, PVOID),
                    _, // infers the per-arch bindgen slot ABI (extern "C" on 64-bit, "stdcall" on i386)
                >(DiskIoctlVerifyThread)),
                DelayedWorkQueue,
                Context as PVOID,
            );
            return STATUS_PENDING;
        }

        if !Context.is_null() {
            ExFreePool(Context as PVOID);
            Context = core::ptr::null_mut();
        }
    }

    if !srb.is_null() {
        ExFreePool(srb as PVOID);
        srb = core::ptr::null_mut();
    }
    return STATUS_INSUFFICIENT_RESOURCES;
}

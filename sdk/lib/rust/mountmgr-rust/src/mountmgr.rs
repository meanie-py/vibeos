use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, GUID, _IO_STATUS_BLOCK, IRP, IoAcquireCancelSpinLock, IoBuildDeviceIoControlRequest, IoCancelIrp, IoCreateDevice, IoDeleteDevice, IoDeleteSymbolicLink, IoFreeIrp, IoGetAttachedDeviceReference, IoGetCurrentIrpStackLocation, IoGetDeviceObjectPointer, IoGetNextIrpStackLocation, IoRegisterPlugPlayNotification, IoRegisterShutdownNotification, IoReleaseCancelSpinLock, IoSetSystemPartition, IoUnregisterPlugPlayNotification, IoUnregisterShutdownNotification, KEVENT, KeAreAllApcsDisabled, KeInitializeEvent, KeInitializeSemaphore, KeInitializeSpinLock, KeReleaseSemaphore, KeWaitForSingleObject, LPGUID, MOUNTDEV_NAME, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, PARTITION_INFORMATION_EX, _PARTITION_STYLE_PARTITION_STYLE_MBR, PDEVICE_INTERFACE_CHANGE_NOTIFICATION, PDEVICE_OBJECT, PDRIVER_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PMOUNTDEV_NAME, PUNICODE_STRING, PsGetCurrentThread, PsGetThreadHardErrorsAreDisabled, PsSetThreadHardErrorsAreDisabled, RTL_QUERY_REGISTRY_TABLE, RtlCompareMemory, RtlCompareUnicodeString, RtlCopyUnicodeString, RtlCreateRegistryKey, RtlEqualUnicodeString, RtlPrefixUnicodeString, RtlQueryRegistryValues, RtlWriteRegistryValue, STORAGE_DEVICE_NUMBER, ZwClose, ZwOpenSymbolicLinkObject, ZwQuerySymbolicLinkObject};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{DISPATCHER_HEADER, HANDLE, IO_STACK_LOCATION, KPRIORITY, KPROCESSOR_MODE, PARTITION_INFORMATION_MBR, PDRIVER_DISPATCH, PDRIVER_UNLOAD, PKEVENT, PRTL_QUERY_REGISTRY_TABLE, _DEVICE_OBJECT, _DRIVER_OBJECT, _GUID, _IRP, _LARGE_INTEGER, _LARGE_INTEGER__bindgen_ty_1, _LIST_ENTRY, _UNICODE_STRING, _PARTITION_INFORMATION_EX__bindgen_ty_1};

extern "C" {
    static OfflinePath: *const u16;
    static mut DeviceMount: _UNICODE_STRING;
    static mut DosDevicesMount: _UNICODE_STRING;
    static mut DatabasePath: *mut u16; // PWSTR (C: PWSTR DatabasePath = L"...")
    static GUID_DEVICE_INTERFACE_ARRIVAL: GUID;
    static GUID_DEVICE_INTERFACE_REMOVAL: GUID;
    fn memcmp( s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize, ) -> i32;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedExchange(Target: *mut i32, Value: i32) -> i32;
    fn InterlockedIncrement(Addend: *mut i32) -> i32;
    fn InterlockedDecrement(Addend: *mut i32) -> i32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedExchange(Target: *mut i32, Value: i32) -> i32;
    fn InterlockedIncrement(Addend: *mut i32) -> i32;
    fn InterlockedDecrement(Addend: *mut i32) -> i32;
}
extern "system" {
    fn MountMgrDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn KeGetCurrentIrql() -> KIRQL;
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
    fn ExAllocatePoolWithTag(PoolType: u32, NumberOfBytes: usize, Tag: u32) -> *mut core::ffi::c_void;
    fn ExFreePoolWithTag(p: *mut core::ffi::c_void, Tag: u32);
    fn FreePool(p: *mut core::ffi::c_void);
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: *const u16);
}

// CUNC — c2rust-hoisted drive-letter symlink template L"\\??\\C:" (the letter at
// CUNC_LETTER_POSITION=4 is overwritten per drive). Module-local, not an export.
static CUNC: [u16; 7] = [0x5C, 0x3F, 0x3F, 0x5C, 0x43, 0x3A, 0x00]; // \ ? ? \ C : \0

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut MountedDevicesGuid: GUID = _GUID {
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

pub static mut gdeviceObject: PDEVICE_OBJECT =
    ::core::ptr::null::<_DEVICE_OBJECT>() as *mut _DEVICE_OBJECT;

#[no_mangle]
pub static mut UnloadEvent: KEVENT = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut Unloading: LONG = 0;

static mut Cunc: [WCHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 14], [WCHAR; 7]>(*b"\\\0?\0?\0\\\0C\0:\0\0\0") };

const ZERO_KEVENT: KEVENT = unsafe { core::mem::transmute([0u8; core::mem::size_of::<KEVENT>()]) };

pub const Cunc_LETTER_POSITION: usize = 4;

const IO_NO_INCREMENT: i32 = 0;
const NotificationEvent: i32 = 0;
const Executive: i32 = 0;

const ALLOC_TAG: u32 = 0x41746E4D; // nolint: const-contradiction ('AtnM' -- mntmgr.h:153; framebuf's unrelated ALLOC_TAG is 'FUBF')
const PAGED_POOL: u32 = 1;

const GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER: u64 = 0x8000000000000000;

const PARTITION_NTFT: i32 = 0x80;
const VALID_NTFT: i32 = 0xC0;
const PARTITION_HUGE: i32 = 0x6; // ntdddisk.h
const PARTITION_IFS: i32 = 0x07;
const PARTITION_FAT32: i32 = 0x0B;
const PARTITION_FAT32_XINT13: i32 = 0x0C;

const IOCTL_VOLUME_GET_GPT_ATTRIBUTES: u32 = 0x560038; // CTL_CODE('V',14,BUFFERED,ANY) — was 0x560000 (= GET_VOLUME_DISK_EXTENTS)
const IOCTL_DISK_GET_PARTITION_INFO_EX: u32 = 0x00070048;
const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x2D1080;
const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: u32 = 0x4D0008; // CTL_CODE('M',2,BUFFERED,ANY) — was fabricated 0x4D000B (method bits = NEITHER!)
const IOCTL_MOUNTDEV_QUERY_UNIQUE_ID: u32 = 0x4D0000; // CTL_CODE('M',0,BUFFERED,ANY) — was fabricated 0x4D000F
const IOCTL_MOUNTDEV_QUERY_STABLE_GUID: u32 = 0x4D0018; // CTL_CODE('M',6,BUFFERED,ANY) — was fabricated 0x4D001B

// Forward declarations for unknown externs

const PagedPool: u32 = 1;

// Extern declarations for unknown symbols (gaps)

// Constants not in imports
const MAXLONG: i32 = 0x7FFFFFFF;
const FILE_DEVICE_NETWORK: u32 = 0x00000012;
const EventCategoryDeviceInterfaceChange: u32 = 2; // _IO_NOTIFICATION_EVENT_CATEGORY_EventCategoryDeviceInterfaceChange
const PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES: u32 = 0x00000001;

// Assumed external globals

const TAGC_MOUNTMGR: u32 = 1098149453; // 'MmcM'
const LETTER_POSITION: isize = 0xC; // drivers/storage/mountmgr/mntmgr.h
const CUNC_LETTER_POSITION: isize = 4;
const UNICODE_NULL: u16 = 0;
#[no_mangle]
pub unsafe extern "C" fn MountMgrSendSyncDeviceIoCtl(
    IoControlCode: ULONG,
    DeviceObject: PDEVICE_OBJECT,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Event: KEVENT = unsafe { core::mem::zeroed() };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
    let mut Irp: PIRP;

    if KeGetCurrentIrql() != PASSIVE_LEVEL {
        RtlAssert(
            b"KeGetCurrentIrql()==(0)\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/storage/mountmgr/mountmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            94 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    }

    if KeAreAllApcsDisabled() != 0 {
        RtlAssert(
            b"!KeAreAllApcsDisabled()\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/storage/mountmgr/mountmgr.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            95 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    }

    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_NotificationEvent, FALSE);
    Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        DeviceObject,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        FALSE as BOOLEAN,
        &raw mut Event,
        &raw mut IoStatusBlock,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if !FileObject.is_null() {
        let next_irp_stack = IoGetNextIrpStackLocation(Irp);
        (*next_irp_stack).FileObject = FileObject;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn IsOffline(SymbolicName: PUNICODE_STRING) -> i32 {
    let mut Status: NTSTATUS = 0;
    let mut IsOffline_0: ULONG = 0;
    let mut Default: ULONG = 0;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: None,
        Flags: 0,
        Name: core::ptr::null(),
        EntryContext: core::ptr::null_mut(),
        DefaultType: 0,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    }; 2];
    RtlZeroMemory(
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>() as usize,
    );
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT as ULONG;
    QueryTable[0].Name = (*SymbolicName).Buffer as *const u16;
    QueryTable[0].EntryContext = &raw mut IsOffline_0 as *mut core::ffi::c_void;
    QueryTable[0].DefaultType = REG_DWORD as ULONG;
    QueryTable[0].DefaultLength = core::mem::size_of::<ULONG>() as ULONG;
    QueryTable[0].DefaultData = &raw mut Default as *mut core::ffi::c_void;
    Default = 0 as ULONG;
    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE as ULONG,
        OfflinePath,
        &raw mut QueryTable as PRTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !(Status >= 0) {
        IsOffline_0 = 0 as ULONG;
    }
    return (IsOffline_0 != 0) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn HasDriveLetter(mut DeviceInformation: PDEVICE_INFORMATION) -> BOOLEAN {
    let mut NextEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut SymlinkInfo: PSYMLINK_INFORMATION = core::ptr::null_mut();
    NextEntry = (*DeviceInformation).SymbolicLinksListHead.Flink as PLIST_ENTRY;
    while NextEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
        SymlinkInfo = (NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry),
        ) as *mut SYMLINK_INFORMATION as PSYMLINK_INFORMATION;
        if IsDriveLetter(&raw mut (*SymlinkInfo).Name) != 0
            && (*SymlinkInfo).Online as i32 != 0
        {
            return 1u8; // TRUE
        }
        NextEntry = (*NextEntry).Flink as PLIST_ENTRY;
    }
    return 0u8; // FALSE
}
#[no_mangle]
pub unsafe extern "C" fn CreateNewDriveLetterName(
    DriveLetter: *mut _UNICODE_STRING,
    DeviceName: *const _UNICODE_STRING,
    mut Letter: u8,
    UniqueId: *mut MOUNTDEV_UNIQUE_ID,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let wchar_size = core::mem::size_of::<u16>() as u16;
    let tag: u32 = 1098149453; // 'M t n' tag
    const PAGED_POOL: u32 = 1;

    // Allocate buffer
    (*DriveLetter).MaximumLength = DosDevices.Length + 3 * wchar_size;
    let buffer = ExAllocatePoolWithTag(PAGED_POOL, (*DriveLetter).MaximumLength as usize, tag) as *mut u16;
    (*DriveLetter).Buffer = buffer;
    if buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Copy prefix
    RtlCopyUnicodeString(DriveLetter, &raw const DosDevices as *const _UNICODE_STRING);

    // Update string to reflect real contents
    (*DriveLetter).Length = DosDevices.Length + 2 * wchar_size;
    *buffer.add((DosDevices.Length as usize / core::mem::size_of::<u16>()) + 2) = 0; // UNICODE_NULL
    *buffer.add((DosDevices.Length as usize / core::mem::size_of::<u16>()) + 1) = ':' as u16;

    // If caller wants a no drive entry
    if Letter == 0xFFu8 {
        // Then, create a no letter entry
        CreateNoDriveLetterEntry(UniqueId);
        ExFreePoolWithTag(buffer as *mut core::ffi::c_void, tag);
        return STATUS_UNSUCCESSFUL;
    } else if Letter != 0 {
        // Use the letter given by the caller
        *buffer.add(DosDevices.Length as usize / core::mem::size_of::<u16>()) = Letter as u16;
        status = GlobalCreateSymbolicLink(DriveLetter, DeviceName);
        if status >= 0 {
            DbgPrint(
                c"[MNTMGR] assigned drive %c: to %wZ\n".as_ptr(),
                Letter as u32,
                DeviceName,
            );
            return status;
        }
    }

    // If caller didn't provide a letter, let's find one for him
    if RtlPrefixUnicodeString(&raw const DeviceFloppy as *const _UNICODE_STRING, DeviceName, 1u8) != 0 {
        // If the device is a floppy, start with letter A
        Letter = 'A' as u8;
    } else if RtlPrefixUnicodeString(&raw const DeviceCdRom as *const _UNICODE_STRING, DeviceName, 1u8) != 0 {
        // If the device is a CD-ROM, start with letter D
        Letter = 'D' as u8;
    } else {
        // Finally, if it's a disk, use C
        Letter = 'C' as u8;
    }

    // Try to affect a letter (up to Z, ofc) until it's possible
    while Letter <= b'Z' {
        *buffer.add(DosDevices.Length as usize / core::mem::size_of::<u16>()) = Letter as u16;
        status = GlobalCreateSymbolicLink(DriveLetter, DeviceName);
        if status >= 0 {
            DbgPrint(
                c"[MNTMGR] assigned drive %c: to %wZ\n".as_ptr(),
                Letter as u32,
                DeviceName,
            );
            return status;
        }
        Letter = Letter.wrapping_add(1);
    }

    // We failed to allocate a letter
    ExFreePoolWithTag(buffer as *mut core::ffi::c_void, tag);
    DbgPrint(
        c"[MNTMGR] FAILED to create a drive letter for %wZ (0x%x)\n".as_ptr(),
        DeviceName,
        status as u32,
    );
    status
}
#[no_mangle]
pub unsafe extern "C" fn QueryDeviceInformation(
    SymbolicName: PUNICODE_STRING,
    DeviceName: PUNICODE_STRING,
    UniqueId: *mut PMOUNTDEV_UNIQUE_ID,
    Removable: PBOOLEAN,
    GptDriveLetter: PBOOLEAN,
    HasGuid: PBOOLEAN,
    StableGuid: LPGUID,
    IsFT: PBOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Size: u16 = 0;
    let mut IsRemovable: u8 = 0;
    let mut Name: PMOUNTDEV_NAME = core::ptr::null_mut();
    let mut Id: PMOUNTDEV_UNIQUE_ID = core::ptr::null_mut();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut PartitionInfo: PARTITION_INFORMATION_EX = PARTITION_INFORMATION_EX {
        PartitionStyle: _PARTITION_STYLE_PARTITION_STYLE_MBR,
        StartingOffset: _LARGE_INTEGER {
            __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 {
                LowPart: 0,
                HighPart: 0,
            },
        },
        PartitionLength: _LARGE_INTEGER {
            __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 {
                LowPart: 0,
                HighPart: 0,
            },
        },
        PartitionNumber: 0,
        RewritePartition: 0,
        __bindgen_anon_1: _PARTITION_INFORMATION_EX__bindgen_ty_1 {
            Mbr: PARTITION_INFORMATION_MBR {
                PartitionType: 0,
                BootIndicator: 0,
                RecognizedPartition: 0,
                HiddenSectors: 0,
            },
        },
    };
    let mut StorageDeviceNumber: STORAGE_DEVICE_NUMBER = STORAGE_DEVICE_NUMBER {
        DeviceType: 0,
        DeviceNumber: 0,
        PartitionNumber: 0,
    };
    let mut GptAttributes: VOLUME_GET_GPT_ATTRIBUTES_INFORMATION =
        VOLUME_GET_GPT_ATTRIBUTES_INFORMATION { GptAttributes: 0 };

    Status = IoGetDeviceObjectPointer(
        SymbolicName,
        FILE_READ_ATTRIBUTES as u32,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !(Status >= 0) {
        return Status;
    }
    if (*FileObject).FileName.Length != 0 {
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }
    IsRemovable = ((*(*FileObject).DeviceObject).Characteristics as u32 & FILE_REMOVABLE_MEDIA) as u8;
    if !Removable.is_null() {
        *Removable = IsRemovable;
    }
    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);

    if !GptDriveLetter.is_null() {
        *GptDriveLetter = 1u8;
        if IsRemovable == 0 {
            Status = MountMgrSendSyncDeviceIoCtl(
                IOCTL_VOLUME_GET_GPT_ATTRIBUTES,
                DeviceObject,
                core::ptr::null_mut(),
                0u32,
                (&raw mut GptAttributes) as *mut core::ffi::c_void,
                core::mem::size_of::<VOLUME_GET_GPT_ATTRIBUTES_INFORMATION>() as u32,
                core::ptr::null_mut(),
            );
            if !(Status >= 0) {
                Status = STATUS_SUCCESS;
            } else if GptAttributes.GptAttributes as u64
                & GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER
                != 0
            {
                *GptDriveLetter = 0u8;
            }
        }
    }

    if !IsFT.is_null() {
        *IsFT = 0u8;
        if IsRemovable == 0 {
            Status = MountMgrSendSyncDeviceIoCtl(
                IOCTL_DISK_GET_PARTITION_INFO_EX,
                DeviceObject,
                core::ptr::null_mut(),
                0u32,
                (&raw mut PartitionInfo) as *mut core::ffi::c_void,
                core::mem::size_of::<PARTITION_INFORMATION_EX>() as u32,
                core::ptr::null_mut(),
            );
            if !(Status >= 0) {
                Status = STATUS_SUCCESS;
            } else {
                let part_type = PartitionInfo.__bindgen_anon_1.Mbr.PartitionType as i32;
                if (PartitionInfo.PartitionStyle as u32 == _PARTITION_STYLE_PARTITION_STYLE_MBR as u32)
                    && (part_type & PARTITION_NTFT != 0
                        && ( (part_type & !VALID_NTFT) == PARTITION_HUGE
                            || (part_type & !VALID_NTFT) == PARTITION_IFS
                            || (part_type & !VALID_NTFT) == PARTITION_FAT32
                            || (part_type & !VALID_NTFT) == PARTITION_FAT32_XINT13 ))
                {
                    *IsFT = 1u8;
                }
            }
            if *IsFT != 0 {
                Status = MountMgrSendSyncDeviceIoCtl(
                    IOCTL_STORAGE_GET_DEVICE_NUMBER,
                    DeviceObject,
                    core::ptr::null_mut(),
                    0u32,
                    (&raw mut StorageDeviceNumber) as *mut core::ffi::c_void,
                    core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                    core::ptr::null_mut(),
                );
                if !(Status >= 0) {
                    Status = STATUS_SUCCESS;
                } else {
                    *IsFT = 0u8;
                }
            }
        }
    }

    if !DeviceName.is_null() {
        Name = ExAllocatePoolWithTag(
            PAGED_POOL,
            core::mem::size_of::<MOUNTDEV_NAME>(),
            ALLOC_TAG,
        ) as PMOUNTDEV_NAME;
        if Name.is_null() {
            ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        Status = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
            DeviceObject,
            core::ptr::null_mut(),
            0u32,
            Name as *mut core::ffi::c_void,
            core::mem::size_of::<MOUNTDEV_NAME>() as u32,
            FileObject,
        );
        if Status == STATUS_BUFFER_OVERFLOW {
            Size = ((*Name).NameLength as usize)
                .wrapping_add(core::mem::size_of::<MOUNTDEV_NAME>())
                as u16;
            ExFreePoolWithTag(Name as *mut core::ffi::c_void, ALLOC_TAG);
            Name = ExAllocatePoolWithTag(PAGED_POOL, Size as usize, ALLOC_TAG) as PMOUNTDEV_NAME;
            if Name.is_null() {
                ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
                ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            Status = MountMgrSendSyncDeviceIoCtl(
                IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
                DeviceObject,
                core::ptr::null_mut(),
                0u32,
                Name as *mut core::ffi::c_void,
                Size as u32,
                FileObject,
            );
        }
        if Status >= 0 {
            (*DeviceName).Length = (*Name).NameLength;
            (*DeviceName).MaximumLength = ((*Name).NameLength as usize)
                .wrapping_add(core::mem::size_of::<u16>()) // WCHAR is 2 bytes
                as u16;
            (*DeviceName).Buffer = ExAllocatePoolWithTag(
                PAGED_POOL,
                (*DeviceName).MaximumLength as usize,
                ALLOC_TAG,
            ) as *mut u16;
            if (*DeviceName).Buffer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                core::ptr::copy_nonoverlapping(
                    (&raw mut (*Name).Name) as *mut u16 as *const core::ffi::c_void,
                    (*DeviceName).Buffer as *mut core::ffi::c_void,
                    (*Name).NameLength as usize,
                );
                *(*DeviceName).Buffer.add(
                    ((*Name).NameLength as usize) / core::mem::size_of::<u16>()
                ) = 0; // UNICODE_NULL
            }
        }
        ExFreePoolWithTag(Name as *mut core::ffi::c_void, ALLOC_TAG);
    }

    if !(Status >= 0) {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        return Status;
    }

    if !UniqueId.is_null() {
        Id = ExAllocatePoolWithTag(
            PAGED_POOL,
            core::mem::size_of::<MOUNTDEV_UNIQUE_ID>(),
            ALLOC_TAG,
        ) as PMOUNTDEV_UNIQUE_ID;
        if Id.is_null() {
            ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        Status = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_QUERY_UNIQUE_ID,
            DeviceObject,
            core::ptr::null_mut(),
            0u32,
            Id as *mut core::ffi::c_void,
            core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as u32,
            FileObject,
        );
        if Status == STATUS_BUFFER_OVERFLOW {
            Size = ((*Id).UniqueIdLength as usize)
                .wrapping_add(core::mem::size_of::<MOUNTDEV_UNIQUE_ID>())
                as u16;
            ExFreePoolWithTag(Id as *mut core::ffi::c_void, ALLOC_TAG);
            Id = ExAllocatePoolWithTag(PAGED_POOL, Size as usize, ALLOC_TAG) as PMOUNTDEV_UNIQUE_ID;
            if Id.is_null() {
                ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
                ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            Status = MountMgrSendSyncDeviceIoCtl(
                IOCTL_MOUNTDEV_QUERY_UNIQUE_ID,
                DeviceObject,
                core::ptr::null_mut(),
                0u32,
                Id as *mut core::ffi::c_void,
                Size as u32,
                FileObject,
            );
        }
        if Status >= 0 {
            *UniqueId = Id;
        } else {
            ExFreePoolWithTag(Id as *mut core::ffi::c_void, ALLOC_TAG);
            if !DeviceName.is_null() && (*DeviceName).Length != 0 {
                ExFreePoolWithTag(
                    (*DeviceName).Buffer as *mut core::ffi::c_void,
                    ALLOC_TAG,
                );
            }
            ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
            return Status;
        }
    }

    if !HasGuid.is_null() {
        let mut IntStatus: NTSTATUS = MountMgrSendSyncDeviceIoCtl(
            IOCTL_MOUNTDEV_QUERY_STABLE_GUID,
            DeviceObject,
            core::ptr::null_mut(),
            0u32,
            StableGuid as *mut core::ffi::c_void,
            core::mem::size_of::<GUID>() as u32,
            FileObject,
        );
        *HasGuid = (IntStatus >= 0) as u8;
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FindDeviceInfo(
    DeviceExtension: PDEVICE_EXTENSION,
    SymbolicName: PUNICODE_STRING,
    DeviceNameGiven: BOOLEAN,
    DeviceInformation: *mut PDEVICE_INFORMATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut NextEntry: PLIST_ENTRY;
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut DeviceInfo: PDEVICE_INFORMATION = core::ptr::null_mut();

    if DeviceNameGiven != 0 {
        DeviceName.Length = (*SymbolicName).Length;
        DeviceName.Buffer = (*SymbolicName).Buffer;
    } else {
        Status = QueryDeviceInformation(
            SymbolicName,
            &raw mut DeviceName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if !(Status >= 0) {
            return Status;
        }
    }

    NextEntry = (*DeviceExtension).DeviceListHead.Flink as PLIST_ENTRY;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        let offset = core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry);
        DeviceInfo = ((NextEntry as usize).wrapping_sub(offset)) as PDEVICE_INFORMATION;

        if RtlEqualUnicodeString(
            &raw mut DeviceName,
            core::ptr::addr_of_mut!((*DeviceInfo).DeviceName),
            1u8,
        ) != 0
        {
            break;
        }
        NextEntry = (*NextEntry).Flink as PLIST_ENTRY;
    }

    if DeviceNameGiven == 0 {
        ExFreePoolWithTag(DeviceName.Buffer as *mut core::ffi::c_void, 0x4D6E744Du32);
    }

    if NextEntry == core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }
    *DeviceInformation = DeviceInfo;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn MountMgrFreeDeadDeviceInfo(DeviceInformation: *mut DEVICE_INFORMATION) {
    ExFreePoolWithTag(
        (*DeviceInformation).SymbolicName.Buffer as *mut core::ffi::c_void,
        1098149453u32,
    );
    ExFreePoolWithTag(DeviceInformation as *mut core::ffi::c_void, 1098149453u32);
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrFreeMountedDeviceInfo(mut DeviceInformation: *mut DEVICE_INFORMATION) {
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut SymLink: *mut SYMLINK_INFORMATION = core::ptr::null_mut();
    let mut UniqueId: *mut UNIQUE_ID_REPLICATE = core::ptr::null_mut();
    let mut AssociatedDevice: *mut ASSOCIATED_DEVICE_ENTRY = core::ptr::null_mut();

    unsafe {
        // purge symbolic links list
        while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead)) {
            NextEntry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead));
            SymLink = ((NextEntry as usize).wrapping_sub(
                core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry),
            )) as *mut SYMLINK_INFORMATION;
            GlobalDeleteSymbolicLink(core::ptr::addr_of_mut!((*SymLink).Name));
            ExFreePoolWithTag((*SymLink).Name.Buffer as *mut core::ffi::c_void, 1098149453u32);
        }

        // purge replicated unique IDs list
        while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead)) {
            NextEntry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead));
            UniqueId = ((NextEntry as usize).wrapping_sub(
                core::mem::offset_of!(UNIQUE_ID_REPLICATE, ReplicatedUniqueIdsListEntry),
            )) as *mut UNIQUE_ID_REPLICATE;
            ExFreePoolWithTag((*UniqueId).UniqueId as *mut core::ffi::c_void, 1098149453u32);
            ExFreePoolWithTag(UniqueId as *mut core::ffi::c_void, 1098149453u32);
        }

        // purge associated devices list
        while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceInformation).AssociatedDevicesHead)) {
            NextEntry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceInformation).AssociatedDevicesHead));
            AssociatedDevice = ((NextEntry as usize).wrapping_sub(
                core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry),
            )) as *mut ASSOCIATED_DEVICE_ENTRY;
            ExFreePoolWithTag((*AssociatedDevice).String.Buffer as *mut core::ffi::c_void, 1098149453u32);
            ExFreePoolWithTag(AssociatedDevice as *mut core::ffi::c_void, 1098149453u32);
        }

        // free the rest of the buffers
        ExFreePoolWithTag((*DeviceInformation).SymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
        if (*DeviceInformation).KeepLinks != 0 {
            ExFreePoolWithTag((*DeviceInformation).UniqueId as *mut core::ffi::c_void, 1098149453u32);
        }
        ExFreePoolWithTag((*DeviceInformation).DeviceName.Buffer as *mut core::ffi::c_void, 1098149453u32);

        // finally, stop waiting for notifications for this device
        if !(*DeviceInformation).TargetDeviceNotificationEntry.is_null() {
            IoUnregisterPlugPlayNotification((*DeviceInformation).TargetDeviceNotificationEntry);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrFreeSavedLink(mut SavedLinkInformation: PSAVED_LINK_INFORMATION) {
    let mut NextEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut SymlinkInformation: PSYMLINK_INFORMATION = core::ptr::null_mut();
    while !IsListEmpty(&raw mut (*SavedLinkInformation).SymbolicLinksListHead) {
        NextEntry = RemoveHeadList(&raw mut (*SavedLinkInformation).SymbolicLinksListHead);
        SymlinkInformation = (NextEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry) as ULONG_PTR,
        ) as *mut SYMLINK_INFORMATION as PSYMLINK_INFORMATION;
        GlobalDeleteSymbolicLink(&raw mut (*SymlinkInformation).Name);
        FreePool((*SymlinkInformation).Name.Buffer as *mut core::ffi::c_void);
        FreePool(SymlinkInformation as *mut core::ffi::c_void);
    }
    FreePool((*SavedLinkInformation).UniqueId as *mut core::ffi::c_void);
    FreePool(SavedLinkInformation as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrUnload(DriverObject: PDRIVER_OBJECT) {
    let mut NextEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut WorkItem: PUNIQUE_ID_WORK_ITEM = core::ptr::null_mut();
    let mut DeviceExtension: PDEVICE_EXTENSION = core::ptr::null_mut();
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut SavedLinkInformation: PSAVED_LINK_INFORMATION = core::ptr::null_mut();

    // UNREFERENCED_PARAMETER(DriverObject)
    let _ = DriverObject;

    IoUnregisterShutdownNotification(gdeviceObject);

    DeviceExtension = (*gdeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    if !(*DeviceExtension).RegistryPath.Buffer.is_null() {
        FreePool((*DeviceExtension).RegistryPath.Buffer as *mut core::ffi::c_void);
        (*DeviceExtension).RegistryPath.Buffer = core::ptr::null_mut();
    }

    _InterlockedExchange(&raw mut Unloading as *mut i32, 1i32);

    KeInitializeEvent(&raw mut UnloadEvent, _EVENT_TYPE_NotificationEvent, 0);

    if InterlockedIncrement(&raw mut (*DeviceExtension).WorkerReferences as *mut i32) > 0 {
        KeReleaseSemaphore(
            &raw mut (*DeviceExtension).WorkerSemaphore,
            IO_NO_INCREMENT as KPRIORITY,
            1,
            0,
        );
        KeWaitForSingleObject(
            &raw mut UnloadEvent as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0,
            core::ptr::null_mut(),
        );
    } else {
        InterlockedDecrement(&raw mut (*DeviceExtension).WorkerReferences as *mut i32);
    }

    IoUnregisterPlugPlayNotification((*DeviceExtension).NotificationEntry);

    KeWaitForSingleObject(
        &raw mut (*DeviceExtension).DeviceLock as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0,
        core::ptr::null_mut(),
    );

    while !IsListEmpty(&raw mut (*DeviceExtension).OfflineDeviceListHead) {
        NextEntry = RemoveHeadList(&raw mut (*DeviceExtension).OfflineDeviceListHead);
        DeviceInformation = (NextEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
            as *mut DEVICE_INFORMATION;
        MountMgrFreeDeadDeviceInfo(DeviceInformation);
    }

    while !IsListEmpty(&raw mut (*DeviceExtension).SavedLinksListHead) {
        NextEntry = RemoveHeadList(&raw mut (*DeviceExtension).SavedLinksListHead);
        SavedLinkInformation = (NextEntry as usize)
            .wrapping_sub(core::mem::offset_of!(SAVED_LINK_INFORMATION, SavedLinksListEntry))
            as *mut SAVED_LINK_INFORMATION;
        MountMgrFreeSavedLink(SavedLinkInformation);
    }

    while !IsListEmpty(&raw mut (*DeviceExtension).UniqueIdWorkerItemListHead) {
        NextEntry = RemoveHeadList(&raw mut (*DeviceExtension).UniqueIdWorkerItemListHead);
        WorkItem = (NextEntry as usize)
            .wrapping_sub(core::mem::offset_of!(UNIQUE_ID_WORK_ITEM, UniqueIdWorkerItemListEntry))
            as *mut UNIQUE_ID_WORK_ITEM;

        KeClearEvent(&raw mut UnloadEvent as *mut core::ffi::c_void);
        (*WorkItem).Event = &raw mut UnloadEvent as PKEVENT;

        KeReleaseSemaphore(
            &raw mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT as KPRIORITY,
            1,
            0,
        );

        IoCancelIrp((*WorkItem).Irp);

        KeWaitForSingleObject(
            &raw mut UnloadEvent as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0,
            core::ptr::null_mut(),
        );

        IoFreeIrp((*WorkItem).Irp);
        FreePool((*WorkItem).DeviceName.Buffer as *mut core::ffi::c_void);
        FreePool((*WorkItem).IrpBuffer);
        FreePool(WorkItem as *mut core::ffi::c_void);

        KeWaitForSingleObject(
            &raw mut (*DeviceExtension).DeviceLock as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0,
            core::ptr::null_mut(),
        );
    }

    if !(*DeviceExtension).DriveLetterData.is_null() {
        FreePool((*DeviceExtension).DriveLetterData as *mut core::ffi::c_void);
        (*DeviceExtension).DriveLetterData = core::ptr::null_mut();
    }

    KeReleaseSemaphore(
        &raw mut (*DeviceExtension).DeviceLock,
        IO_NO_INCREMENT as KPRIORITY,
        1,
        0,
    );

    GlobalDeleteSymbolicLink(&raw mut DosDevicesMount);
    IoDeleteDevice(gdeviceObject);
}
#[no_mangle]
pub unsafe extern "C" fn MountmgrReadNoAutoMount(RegistryPath: PUNICODE_STRING) -> BOOLEAN {
    let mut Status: NTSTATUS = 0;
    let mut Result: ULONG = 0;
    let mut Default: ULONG = 0;
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = unsafe { core::mem::zeroed() };
    core::ptr::write_bytes(
        &mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );
    QueryTable[0usize].Flags = RTL_QUERY_REGISTRY_DIRECT as u32;
    // "NoAutoMount\0" as UTF-16 (LE u16 array)
    let no_auto_mount: [u16; 12] = [
        78, 111, 65, 117, 116, 111, 77, 111, 117, 110, 116, 0,
    ];
    QueryTable[0usize].Name = no_auto_mount.as_ptr();
    QueryTable[0usize].EntryContext = &mut Result as *mut ULONG as *mut core::ffi::c_void;
    QueryTable[0usize].DefaultType = REG_DWORD as u32;
    QueryTable[0usize].DefaultData = &mut Default as *mut ULONG as *mut core::ffi::c_void;
    QueryTable[0usize].DefaultLength = core::mem::size_of::<ULONG>() as u32;
    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE as u32,
        (*RegistryPath).Buffer as *const u16,
        &mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status < 0 {
        Result = Default;
    }
    (Result != 0) as u8
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrMountedDeviceArrival(
    DeviceExtension: PDEVICE_EXTENSION,
    SymbolicName: PUNICODE_STRING,
    ManuallyRegistered: BOOLEAN,
) -> NTSTATUS {
    let mut Letter: u16;
    let mut StableGuid: GUID = core::mem::zeroed();
    let mut LinkHandle: HANDLE = core::mem::zeroed();
    let mut SymLinkCount: u32 = core::mem::zeroed();
    let mut i: u32;
    let mut NextEntry: PLIST_ENTRY;
    let mut SymLinks: PUNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut IntStatus: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SymlinkInformation: PSYMLINK_INFORMATION;
    let mut UniqueId: PMOUNTDEV_UNIQUE_ID = core::mem::zeroed();
    let mut NewUniqueId: PMOUNTDEV_UNIQUE_ID;
    let mut SavedLinkInformation: PSAVED_LINK_INFORMATION = core::mem::zeroed();
    let mut DeviceInformation: PDEVICE_INFORMATION = core::mem::zeroed();
    let mut CurrentDevice: PDEVICE_INFORMATION;
    let mut CSymLinkBuffer: [u16; 7] = core::mem::zeroed();
    let mut LinkTargetBuffer: [u16; 260] = core::mem::zeroed();
    let mut TargetDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut SuggestedLinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    DbgPrint(
        c"[MNTMGR] arrival: %wZ (manual=%u)\n".as_ptr(),
        SymbolicName,
        ManuallyRegistered as u32,
    );
    let mut VolumeName: _UNICODE_STRING = core::mem::zeroed();
    let mut DriveLetter: _UNICODE_STRING = core::mem::zeroed();
    let mut LinkTarget: _UNICODE_STRING = core::mem::zeroed();
    let mut CSymLink: _UNICODE_STRING = core::mem::zeroed();
    let mut HasGuid: BOOLEAN = core::mem::zeroed();
    let mut HasGptDriveLetter: BOOLEAN = core::mem::zeroed();
    let mut IsFT: BOOLEAN = core::mem::zeroed();
    let mut UseOnlyIfThereAreNoOtherLinks: BOOLEAN = core::mem::zeroed();
    let mut IsDrvLetter: BOOLEAN;
    let mut IsOff: BOOLEAN;
    let mut IsVolumeName: BOOLEAN;
    let mut SetOnline: BOOLEAN;

    DeviceInformation = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<DEVICE_INFORMATION>(),
        TAGC_MOUNTMGR,
    ) as PDEVICE_INFORMATION;
    if DeviceInformation.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(
        DeviceInformation as *mut u8,
        0,
        core::mem::size_of::<DEVICE_INFORMATION>(),
    );
    InitializeListHead(&raw mut (*DeviceInformation).SymbolicLinksListHead);
    InitializeListHead(&raw mut (*DeviceInformation).ReplicatedUniqueIdsListHead);
    InitializeListHead(&raw mut (*DeviceInformation).AssociatedDevicesHead);
    (*DeviceInformation).SymbolicName.Length = (*SymbolicName).Length;
    (*DeviceInformation).SymbolicName.MaximumLength =
        ((*SymbolicName).Length as usize).wrapping_add(core::mem::size_of::<u16>()) as u16;
    (*DeviceInformation).SymbolicName.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*DeviceInformation).SymbolicName.MaximumLength as usize,
        TAGC_MOUNTMGR,
    ) as *mut u16;
    if (*DeviceInformation).SymbolicName.Buffer.is_null() {
        ExFreePoolWithTag(DeviceInformation as *mut core::ffi::c_void, TAGC_MOUNTMGR);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(
        (*SymbolicName).Buffer as *const u8,
        (*DeviceInformation).SymbolicName.Buffer as *mut u8,
        (*SymbolicName).Length as usize, // Length is in BYTES — cast to u8 (else 2x element overflow)
    );
    *((*DeviceInformation).SymbolicName.Buffer.add(
        ((*DeviceInformation).SymbolicName.Length as usize)
            .wrapping_div(core::mem::size_of::<u16>()),
    )) = UNICODE_NULL;
    (*DeviceInformation).ManuallyRegistered = ManuallyRegistered;
    (*DeviceInformation).DeviceExtension = DeviceExtension;
    Status = QueryDeviceInformation(
        SymbolicName,
        &raw mut TargetDeviceName,
        &raw mut UniqueId,
        &raw mut (*DeviceInformation).Removable,
        &raw mut HasGptDriveLetter,
        &raw mut HasGuid,
        &raw mut StableGuid,
        &raw mut IsFT,
    );
    if Status < 0 {
        DbgPrint(
            c"[MNTMGR] arrival: QueryDeviceInformation FAILED 0x%x -> offline list\n".as_ptr(),
            Status as u32,
        );
        KeWaitForSingleObject(
            &raw mut (*DeviceExtension).DeviceLock as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            core::ptr::null_mut(),
        );
        NextEntry = (*DeviceExtension).OfflineDeviceListHead.Flink;
        while NextEntry != &raw mut (*DeviceExtension).OfflineDeviceListHead {
            CurrentDevice = (NextEntry as usize).wrapping_sub(
                core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
            ) as PDEVICE_INFORMATION;
            if RtlEqualUnicodeString(
                &raw mut (*DeviceInformation).SymbolicName,
                &raw mut (*CurrentDevice).SymbolicName,
                TRUE,
            ) != 0
            {
                break;
            }
            NextEntry = (*NextEntry).Flink;
        }
        if NextEntry != &raw mut (*DeviceExtension).OfflineDeviceListHead {
            MountMgrFreeDeadDeviceInfo(DeviceInformation);
        } else {
            InsertTailList(
                &raw mut (*DeviceExtension).OfflineDeviceListHead,
                &raw mut (*DeviceInformation).DeviceListEntry,
            );
        }
        KeReleaseSemaphore(
            &raw mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT,
            1,
            FALSE,
        );
        return Status;
    }
    (*DeviceInformation).UniqueId = UniqueId;
    (*DeviceInformation).DeviceName = TargetDeviceName;
    (*DeviceInformation).KeepLinks = FALSE;
    if !(*DeviceExtension).DriveLetterData.is_null()
        && (*UniqueId).UniqueIdLength as i32
            == (*(*DeviceExtension).DriveLetterData).UniqueIdLength as i32
    {
        if RtlCompareMemory(
            (&raw mut (*UniqueId).UniqueId) as *const core::ffi::c_void,
            (&raw mut (*(*DeviceExtension).DriveLetterData).UniqueId) as *const core::ffi::c_void,
            (*UniqueId).UniqueIdLength as _,
        ) == (*UniqueId).UniqueIdLength as _
        {
            IoSetSystemPartition(&raw mut TargetDeviceName);
        }
    }
    Status = QuerySuggestedLinkName(
        &raw mut (*DeviceInformation).SymbolicName,
        &raw mut SuggestedLinkName,
        &raw mut UseOnlyIfThereAreNoOtherLinks,
    );
    if Status < 0 {
        SuggestedLinkName.Buffer = core::ptr::null_mut();
    }
    if !SuggestedLinkName.Buffer.is_null() && IsDriveLetter(&raw mut SuggestedLinkName) != 0 {
        (*DeviceInformation).SuggestedDriveLetter =
            *SuggestedLinkName.Buffer.offset(LETTER_POSITION) as u8;
    }
    KeWaitForSingleObject(
        &raw mut (*DeviceExtension).DeviceLock as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i8,
        FALSE,
        core::ptr::null_mut(),
    );
    NextEntry = (*DeviceExtension).DeviceListHead.Flink;
    while NextEntry != &raw mut (*DeviceExtension).DeviceListHead {
        CurrentDevice = (NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
        ) as PDEVICE_INFORMATION;
        if RtlEqualUnicodeString(
            &raw mut (*CurrentDevice).DeviceName,
            &raw mut TargetDeviceName,
            TRUE,
        ) != 0
        {
            break;
        }
        NextEntry = (*NextEntry).Flink;
    }
    if NextEntry != &raw mut (*DeviceExtension).DeviceListHead {
        if !SuggestedLinkName.Buffer.is_null() {
            ExFreePoolWithTag(SuggestedLinkName.Buffer as *mut core::ffi::c_void, TAGC_MOUNTMGR);
        }
        ExFreePoolWithTag(UniqueId as *mut core::ffi::c_void, TAGC_MOUNTMGR);
        ExFreePoolWithTag(
            TargetDeviceName.Buffer as *mut core::ffi::c_void,
            TAGC_MOUNTMGR,
        );
        ExFreePoolWithTag(
            (*DeviceInformation).SymbolicName.Buffer as *mut core::ffi::c_void,
            TAGC_MOUNTMGR,
        );
        ExFreePoolWithTag(DeviceInformation as *mut core::ffi::c_void, TAGC_MOUNTMGR);
        KeReleaseSemaphore(&raw mut (*DeviceExtension).DeviceLock, IO_NO_INCREMENT, 1, FALSE);
        return STATUS_SUCCESS;
    }
    Status = QuerySymbolicLinkNamesFromStorage(
        DeviceExtension,
        DeviceInformation,
        if !SuggestedLinkName.Buffer.is_null() {
            &raw mut SuggestedLinkName
        } else {
            core::ptr::null_mut()
        },
        UseOnlyIfThereAreNoOtherLinks,
        &raw mut SymLinks,
        &raw mut SymLinkCount,
        HasGuid,
        &raw mut StableGuid,
    );
    if RtlPrefixUnicodeString(&raw mut DeviceCdRom, &raw mut TargetDeviceName, TRUE) != 0 {
        LinkTarget.Length = 0;
        LinkTarget.MaximumLength = core::mem::size_of::<[u16; 260]>() as u16;
        LinkTarget.Buffer = &raw mut LinkTargetBuffer as *mut u16;
        core::ptr::copy_nonoverlapping(
            &CUNC as *const [u16; 7] as *const u16,
            &raw mut CSymLinkBuffer as *mut u16,
            7,
        );
        RtlInitUnicodeString(&raw mut CSymLink, &CUNC as *const u16);
        Letter = 'D' as u16;
        while Letter as i32 <= 'Z' as i32 {
            *CSymLink.Buffer.offset(CUNC_LETTER_POSITION) = Letter;
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.ObjectName = &raw mut CSymLink;
            ObjectAttributes.Attributes =
                (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as u32;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
            Status = ZwOpenSymbolicLinkObject(
                &raw mut LinkHandle,
                SYMBOLIC_LINK_QUERY,
                &raw mut ObjectAttributes,
            );
            if Status >= 0 {
                Status = ZwQuerySymbolicLinkObject(
                    LinkHandle,
                    &raw mut LinkTarget,
                    core::ptr::null_mut(),
                );
                ZwClose(LinkHandle);
                if Status >= 0 {
                    IntStatus = STATUS_UNSUCCESSFUL;
                    if RtlEqualUnicodeString(
                        &raw mut LinkTarget,
                        &raw mut (*DeviceInformation).DeviceName,
                        FALSE,
                    ) == 0
                    {
                        // link matches device, but no symlink expected
                        if SymLinkCount == 0 {
                            IoDeleteSymbolicLink(&raw mut CSymLink);
                        } else {
                            i = 0;
                            while i < SymLinkCount {
                                if IsDriveLetter(SymLinks.add(i as usize)) != 0 {
                                    if *(*SymLinks.add(i as usize)).Buffer
                                        .offset(LETTER_POSITION)
                                        as i32
                                        == Letter as i32
                                    {
                                        IntStatus = STATUS_SUCCESS;
                                    }
                                }
                                i = i.wrapping_add(1);
                            }
                            if IntStatus == STATUS_UNSUCCESSFUL {
                                IoDeleteSymbolicLink(&raw mut CSymLink);
                            }
                        }
                    }
                }
            }
            Letter = Letter.wrapping_add(1);
        }
    }
    if !SuggestedLinkName.Buffer.is_null() {
        ExFreePoolWithTag(SuggestedLinkName.Buffer as *mut core::ffi::c_void, TAGC_MOUNTMGR);
    }
    if Status < 0 {
        SymLinks = core::ptr::null_mut();
        SymLinkCount = 0;
    }
    SavedLinkInformation = RemoveSavedLinks(DeviceExtension, UniqueId);
    IsDrvLetter = FALSE;
    IsOff = FALSE;
    IsVolumeName = FALSE;
    i = 0;
    while i < SymLinkCount {
        // MOUNTMGR_IS_VOLUME_NAME expansion (from anchor)
        let is_vol = ((*(SymLinks.add(i as usize))).Length as i32 == 96i32
            || (*(SymLinks.add(i as usize))).Length as i32 == 98i32
                && *(*(SymLinks.add(i as usize)))
                    .Buffer
                    .offset(48i32 as isize) as i32
                    == '\\' as i32)
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(0i32 as isize) as i32
                == '\\' as i32
            && (*(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(1i32 as isize) as i32
                == '?' as i32
                || *(*(SymLinks.add(i as usize)))
                    .Buffer
                    .offset(1i32 as isize) as i32
                    == '\\' as i32)
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(2i32 as isize) as i32
                == '?' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(3i32 as isize) as i32
                == '\\' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(4i32 as isize) as i32
                == 'V' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(5i32 as isize) as i32
                == 'o' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(6i32 as isize) as i32
                == 'l' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(7i32 as isize) as i32
                == 'u' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(8i32 as isize) as i32
                == 'm' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(9i32 as isize) as i32
                == 'e' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(10i32 as isize) as i32
                == '{' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(19i32 as isize) as i32
                == '-' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(24i32 as isize) as i32
                == '-' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(29i32 as isize) as i32
                == '-' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(34i32 as isize) as i32
                == '-' as i32
            && *(*(SymLinks.add(i as usize)))
                .Buffer
                .offset(47i32 as isize) as i32
                == '}' as i32;

        if is_vol {
            IsVolumeName = TRUE;
        } else if IsDriveLetter(SymLinks.add(i as usize)) != 0 {
            if IsDrvLetter != 0 {
                DeleteFromLocalDatabase(SymLinks.add(i as usize), UniqueId);
            } else {
                IsDrvLetter = TRUE;
            }
        }
        let mut need_continue = false;
        let mut block_flag = 0u8;
        // Mark the block we're in: 0 = create/recreate, 1 = skip/continue logic
        Status = GlobalCreateSymbolicLink(SymLinks.add(i as usize), &raw mut TargetDeviceName);
        if Status < 0 {
            let mut LinkError: BOOLEAN = TRUE;
            if (!SavedLinkInformation.is_null()
                && RedirectSavedLink(
                    SavedLinkInformation,
                    SymLinks.add(i as usize),
                    &raw mut TargetDeviceName,
                ) == 0)
                || SavedLinkInformation.is_null()
            {
                Status = QueryDeviceInformation(
                    SymLinks.add(i as usize),
                    &raw mut DeviceName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    LinkError = RtlEqualUnicodeString(
                        &raw mut TargetDeviceName,
                        &raw mut DeviceName,
                        TRUE,
                    );
                    ExFreePoolWithTag(DeviceName.Buffer as *mut core::ffi::c_void, TAGC_MOUNTMGR);
                }
                if LinkError == 0 {
                    if IsDriveLetter(SymLinks.add(i as usize)) != 0 {
                        IsDrvLetter = FALSE;
                        DeleteFromLocalDatabase(SymLinks.add(i as usize), UniqueId);
                    }
                    ExFreePoolWithTag(
                        (*SymLinks.add(i as usize)).Buffer as *mut core::ffi::c_void,
                        TAGC_MOUNTMGR,
                    );
                    need_continue = true;
                }
            }
            if !need_continue {
                if IsOffline(SymLinks.add(i as usize)) != 0 {
                    IsOff = TRUE;
                }
                SymlinkInformation = ExAllocatePoolWithTag(
                    PagedPool,
                    core::mem::size_of::<SYMLINK_INFORMATION>(),
                    TAGC_MOUNTMGR,
                ) as PSYMLINK_INFORMATION;
                if SymlinkInformation.is_null() {
                    GlobalDeleteSymbolicLink(SymLinks.add(i as usize));
                    ExFreePoolWithTag(
                        (*SymLinks.add(i as usize)).Buffer as *mut core::ffi::c_void,
                        TAGC_MOUNTMGR,
                    );
                } else {
                    (*SymlinkInformation).Name = *SymLinks.add(i as usize);
                    (*SymlinkInformation).Online = TRUE;
                    InsertTailList(
                        &raw mut (*DeviceInformation).SymbolicLinksListHead,
                        &raw mut (*SymlinkInformation).SymbolicLinksListEntry,
                    );
                }
            }
        } else {
            if IsOffline(SymLinks.add(i as usize)) != 0 {
                IsOff = TRUE;
            }
            SymlinkInformation = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<SYMLINK_INFORMATION>(),
                TAGC_MOUNTMGR,
            ) as PSYMLINK_INFORMATION;
            if SymlinkInformation.is_null() {
                GlobalDeleteSymbolicLink(SymLinks.add(i as usize));
                ExFreePoolWithTag(
                    (*SymLinks.add(i as usize)).Buffer as *mut core::ffi::c_void,
                    TAGC_MOUNTMGR,
                );
            } else {
                (*SymlinkInformation).Name = *SymLinks.add(i as usize);
                (*SymlinkInformation).Online = TRUE;
                InsertTailList(
                    &raw mut (*DeviceInformation).SymbolicLinksListHead,
                    &raw mut (*SymlinkInformation).SymbolicLinksListEntry,
                );
            }
        }
        if need_continue {
            // We already continued in the earlier block, so just advance loop
        }
        i = i.wrapping_add(1);
    }
    NextEntry = (*DeviceInformation).SymbolicLinksListHead.Flink;
    while NextEntry != &raw mut (*DeviceInformation).SymbolicLinksListHead {
        SymlinkInformation = (NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry),
        ) as PSYMLINK_INFORMATION;
        SendLinkCreated(&raw mut (*SymlinkInformation).Name);
        NextEntry = (*NextEntry).Flink;
    }
    if !SavedLinkInformation.is_null() {
        MountMgrFreeSavedLink(SavedLinkInformation);
    }
    if IsVolumeName == 0 {
        Status = CreateNewVolumeName(&raw mut VolumeName, core::ptr::null());
        if Status >= 0 {
            RtlWriteRegistryValue(
                RTL_REGISTRY_ABSOLUTE,
                DatabasePath,
                VolumeName.Buffer,
                REG_BINARY,
                (&raw mut (*UniqueId).UniqueId) as *mut u8 as *mut core::ffi::c_void,
                (*UniqueId).UniqueIdLength as u32,
            );
            GlobalCreateSymbolicLink(&raw mut VolumeName, &raw mut TargetDeviceName);
            SymlinkInformation = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<SYMLINK_INFORMATION>(),
                TAGC_MOUNTMGR,
            ) as PSYMLINK_INFORMATION;
            if SymlinkInformation.is_null() {
                ExFreePoolWithTag(VolumeName.Buffer as *mut core::ffi::c_void, TAGC_MOUNTMGR);
            } else {
                (*SymlinkInformation).Name = VolumeName;
                (*SymlinkInformation).Online = TRUE;
                InsertTailList(
                    &raw mut (*DeviceInformation).SymbolicLinksListHead,
                    &raw mut (*SymlinkInformation).SymbolicLinksListEntry,
                );
                SendLinkCreated(&raw mut VolumeName);
            }
        }
    }
    DbgPrint(
        c"[MNTMGR] arrival gates: IsDrvLetter=%u AutoDl=%u NoAutoMount=%u Removable=%u HasGptDl=%u Suggested=%u NoDlEntry=%u\n".as_ptr(),
        IsDrvLetter as u32,
        (*DeviceExtension).AutomaticDriveLetter as u32,
        (*DeviceExtension).NoAutoMount as u32,
        (*DeviceInformation).Removable as u32,
        HasGptDriveLetter as u32,
        (*DeviceInformation).SuggestedDriveLetter as u32,
        HasNoDriveLetterEntry(UniqueId) as u32,
    );
    if IsDrvLetter != 0 {
        (*DeviceInformation).SuggestedDriveLetter = 0;
    } else if ((*DeviceExtension).NoAutoMount == 0
        || (*DeviceInformation).Removable as i32 != 0)
        && (*DeviceExtension).AutomaticDriveLetter as i32 != 0
        && (HasGptDriveLetter as i32 != 0
            || (*DeviceInformation).SuggestedDriveLetter as i32 != 0)
        && HasNoDriveLetterEntry(UniqueId) == 0
    {
        Status = CreateNewDriveLetterName(
            &raw mut DriveLetter,
            &raw mut TargetDeviceName,
            (*DeviceInformation).SuggestedDriveLetter,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            CreateNoDriveLetterEntry(UniqueId);
        } else {
            RtlWriteRegistryValue(
                RTL_REGISTRY_ABSOLUTE,
                DatabasePath,
                DriveLetter.Buffer,
                REG_BINARY,
                (&raw mut (*UniqueId).UniqueId) as *mut u8 as *mut core::ffi::c_void,
                (*UniqueId).UniqueIdLength as u32,
            );
            SymlinkInformation = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<SYMLINK_INFORMATION>(),
                TAGC_MOUNTMGR,
            ) as PSYMLINK_INFORMATION;
            if SymlinkInformation.is_null() {
                ExFreePoolWithTag(DriveLetter.Buffer as *mut core::ffi::c_void, TAGC_MOUNTMGR);
            } else {
                (*SymlinkInformation).Name = DriveLetter;
                (*SymlinkInformation).Online = TRUE;
                InsertTailList(
                    &raw mut (*DeviceInformation).SymbolicLinksListHead,
                    &raw mut (*SymlinkInformation).SymbolicLinksListEntry,
                );
                SendLinkCreated(&raw mut DriveLetter);
            }
        }
    }
    if ManuallyRegistered == 0 {
        RegisterForTargetDeviceNotification(DeviceExtension, DeviceInformation);
    }
    InsertTailList(
        &raw mut (*DeviceExtension).DeviceListHead,
        &raw mut (*DeviceInformation).DeviceListEntry,
    );
    DbgPrint(
        c"[MNTMGR] arrival: inserted %wZ (canonical %wZ)\n".as_ptr(),
        SymbolicName,
        &raw mut (*DeviceInformation).DeviceName,
    );
    NewUniqueId = ExAllocatePoolWithTag(
        PagedPool,
        ((*UniqueId).UniqueIdLength as usize)
            .wrapping_add(core::mem::size_of::<MOUNTDEV_UNIQUE_ID>()),
        TAGC_MOUNTMGR,
    ) as PMOUNTDEV_UNIQUE_ID;
    if !NewUniqueId.is_null() {
        (*NewUniqueId).UniqueIdLength = (*UniqueId).UniqueIdLength;
        core::ptr::copy_nonoverlapping(
            (&raw mut (*UniqueId).UniqueId) as *const u8,
            (&raw mut (*NewUniqueId).UniqueId) as *mut u8,
            (*UniqueId).UniqueIdLength as usize,
        );
    }
    if IsOff != 0 || IsFT != 0 {
        (*DeviceInformation).SkipNotifications = TRUE;
    }
    if (*DeviceExtension).NoAutoMount == 0 || IsDrvLetter != 0 {
        SetOnline = ((*DeviceInformation).SkipNotifications == 0) as i32 as BOOLEAN;
    } else {
        SetOnline = FALSE;
    }
    KeReleaseSemaphore(&raw mut (*DeviceExtension).DeviceLock, IO_NO_INCREMENT, 1, FALSE);
    if SetOnline != 0 {
        SendOnlineNotification(SymbolicName);
    }
    if !SymLinks.is_null() {
        ExFreePoolWithTag(SymLinks as *mut core::ffi::c_void, TAGC_MOUNTMGR);
    }
    if !NewUniqueId.is_null() {
        IssueUniqueIdChangeNotify(DeviceExtension, SymbolicName, NewUniqueId);
        ExFreePoolWithTag(NewUniqueId as *mut core::ffi::c_void, TAGC_MOUNTMGR);
    }
    if (*DeviceExtension).AutomaticDriveLetter != 0 {
        KeWaitForSingleObject(
            &raw mut (*DeviceExtension).DeviceLock as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            core::ptr::null_mut(),
        );
        ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
        NextEntry = (*DeviceExtension).DeviceListHead.Flink;
        CurrentDevice = (NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
        ) as PDEVICE_INFORMATION;
        while CurrentDevice != DeviceInformation {
            if (*CurrentDevice).NoDatabase == 0 {
                ReconcileThisDatabaseWithMaster(DeviceExtension, CurrentDevice);
            }
            NextEntry = (*NextEntry).Flink;
            CurrentDevice = (NextEntry as usize).wrapping_sub(
                core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry),
            ) as PDEVICE_INFORMATION;
        }
        KeReleaseSemaphore(&raw mut (*DeviceExtension).DeviceLock, IO_NO_INCREMENT, 1, FALSE);
    }
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrMountedDeviceRemoval(
    DeviceExtension: PDEVICE_EXTENSION,
    DeviceName: PUNICODE_STRING,
) {
    let mut NextEntry: *mut LIST_ENTRY;
    let mut DeviceEntry: *mut LIST_ENTRY;
    let mut UniqueIdReplicate: *mut UNIQUE_ID_REPLICATE;
    let mut SymlinkInformation: *mut SYMLINK_INFORMATION;
    let mut AssociatedDevice: *mut ASSOCIATED_DEVICE_ENTRY;
    let mut SavedLinkInformation: *mut SAVED_LINK_INFORMATION = core::mem::zeroed();
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::mem::zeroed();
    let mut CurrentDevice: *mut DEVICE_INFORMATION;

    DbgPrint(c"[MNTMGR] removal: %wZ\n".as_ptr(), DeviceName);
    // Acquire device exclusively
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock) as *mut ::core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i8,
        0u8, // FALSE
        core::ptr::null_mut(),
    );

    // Look for the leaving device
    NextEntry = (*DeviceExtension).DeviceListHead.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        DeviceInformation = (NextEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
            as *mut DEVICE_INFORMATION;

        if RtlCompareUnicodeString(
            core::ptr::addr_of_mut!((*DeviceInformation).SymbolicName),
            DeviceName,
            1, // TRUE
        ) == 0 {
            break;
        }
        NextEntry = (*NextEntry).Flink;
    }

    // If we found it
    if NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
        // If it's asked to keep links, then, prepare to save them
        if (*DeviceInformation).KeepLinks != 0 {
            SavedLinkInformation = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<SAVED_LINK_INFORMATION>(),
                1098149453u32,
            ) as *mut SAVED_LINK_INFORMATION;
            if SavedLinkInformation.is_null() {
                (*DeviceInformation).KeepLinks = 0; // FALSE
            }
        }

        // If it's possible (and asked), start to save them
        if (*DeviceInformation).KeepLinks != 0 {
            InsertTailList(
                core::ptr::addr_of_mut!((*DeviceExtension).SavedLinksListHead),
                core::ptr::addr_of_mut!((*SavedLinkInformation).SavedLinksListEntry),
            );
            InitializeListHead(core::ptr::addr_of_mut!((*SavedLinkInformation).SymbolicLinksListHead));
            (*SavedLinkInformation).UniqueId = (*DeviceInformation).UniqueId;
        }

        // For all the symlinks
        while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead)) {
            NextEntry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceInformation).SymbolicLinksListHead));
            SymlinkInformation = (NextEntry as usize)
                .wrapping_sub(core::mem::offset_of!(SYMLINK_INFORMATION, SymbolicLinksListEntry))
                as *mut SYMLINK_INFORMATION;

            if (*DeviceInformation).KeepLinks != 0 {
                InsertTailList(
                    core::ptr::addr_of_mut!((*SavedLinkInformation).SymbolicLinksListHead),
                    core::ptr::addr_of_mut!((*SymlinkInformation).SymbolicLinksListEntry),
                );
            } else {
                GlobalDeleteSymbolicLink(core::ptr::addr_of_mut!((*SymlinkInformation).Name));
                ExFreePoolWithTag((*SymlinkInformation).Name.Buffer as *mut ::core::ffi::c_void, 1098149453u32);
                ExFreePoolWithTag(SymlinkInformation as *mut ::core::ffi::c_void, 1098149453u32);
            }
        }

        // Free all the replicated unique IDs
        while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead)) {
            NextEntry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceInformation).ReplicatedUniqueIdsListHead));
            UniqueIdReplicate = (NextEntry as usize)
                .wrapping_sub(core::mem::offset_of!(UNIQUE_ID_REPLICATE, ReplicatedUniqueIdsListEntry))
                as *mut UNIQUE_ID_REPLICATE;

            ExFreePoolWithTag((*UniqueIdReplicate).UniqueId as *mut ::core::ffi::c_void, 1098149453u32);
            ExFreePoolWithTag(UniqueIdReplicate as *mut ::core::ffi::c_void, 1098149453u32);
        }

        while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceInformation).AssociatedDevicesHead)) {
            NextEntry = RemoveHeadList(core::ptr::addr_of_mut!((*DeviceInformation).AssociatedDevicesHead));
            AssociatedDevice = (NextEntry as usize)
                .wrapping_sub(core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry))
                as *mut ASSOCIATED_DEVICE_ENTRY;

            (*DeviceInformation).NoDatabase = 1; // TRUE
            ExFreePoolWithTag((*AssociatedDevice).String.Buffer as *mut ::core::ffi::c_void, 1098149453u32);
            ExFreePoolWithTag(AssociatedDevice as *mut ::core::ffi::c_void, 1098149453u32);
        }

        // Remove device from the device list
        RemoveEntryList(core::ptr::addr_of_mut!((*DeviceInformation).DeviceListEntry));

        // If there are still devices, check if some were associated with ours
        if !IsListEmpty(core::ptr::addr_of_mut!((*DeviceInformation).DeviceListEntry)) {
            NextEntry = (*DeviceExtension).DeviceListHead.Flink;
            while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
                CurrentDevice = (NextEntry as usize)
                    .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
                    as *mut DEVICE_INFORMATION;

                DeviceEntry = (*CurrentDevice).AssociatedDevicesHead.Flink;
                while DeviceEntry != core::ptr::addr_of_mut!((*CurrentDevice).AssociatedDevicesHead) {
                    // DIVERGES from mountmgr.c, which takes CONTAINING_RECORD of `NextEntry` --
                    // the OUTER DeviceListHead link, i.e. a DEVICE_INFORMATION read as an
                    // ASSOCIATED_DEVICE_ENTRY -- so the owner test compared garbage and a match
                    // would unlink and free a live DEVICE_INFORMATION. This loop walks
                    // `DeviceEntry`; the record is taken from it before it advances, so the
                    // free below never touches the link it continues from.
                    AssociatedDevice = (DeviceEntry as usize)
                        .wrapping_sub(core::mem::offset_of!(ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry))
                        as *mut ASSOCIATED_DEVICE_ENTRY;
                    DeviceEntry = (*DeviceEntry).Flink;

                    if (*AssociatedDevice).DeviceInformation != DeviceInformation {
                        continue;
                    }

                    RemoveEntryList(core::ptr::addr_of_mut!((*AssociatedDevice).AssociatedDevicesEntry));
                    ExFreePoolWithTag((*AssociatedDevice).String.Buffer as *mut ::core::ffi::c_void, 1098149453u32);
                    ExFreePoolWithTag(AssociatedDevice as *mut ::core::ffi::c_void, 1098149453u32);
                }
                NextEntry = (*NextEntry).Flink;
            }
        }

        // Finally, clean up device name, symbolic name
        ExFreePoolWithTag((*DeviceInformation).SymbolicName.Buffer as *mut ::core::ffi::c_void, 1098149453u32);
        if (*DeviceInformation).KeepLinks == 0 {
            ExFreePoolWithTag((*DeviceInformation).UniqueId as *mut ::core::ffi::c_void, 1098149453u32);
        }
        ExFreePoolWithTag((*DeviceInformation).DeviceName.Buffer as *mut ::core::ffi::c_void, 1098149453u32);

        // Unregister notifications
        if !(*DeviceInformation).TargetDeviceNotificationEntry.is_null() {
            IoUnregisterPlugPlayNotification((*DeviceInformation).TargetDeviceNotificationEntry);
        }

        // And leave
        ExFreePoolWithTag(DeviceInformation as *mut ::core::ffi::c_void, 1098149453u32);
    } else {
        // We didn't find device, perhaps because it was offline
        NextEntry = (*DeviceExtension).OfflineDeviceListHead.Flink;
        while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).OfflineDeviceListHead) {
            DeviceInformation = (NextEntry as usize)
                .wrapping_sub(core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry))
                as *mut DEVICE_INFORMATION;

            if RtlCompareUnicodeString(
                core::ptr::addr_of_mut!((*DeviceInformation).SymbolicName),
                DeviceName,
                1, // TRUE
            ) == 0 {
                RemoveEntryList(core::ptr::addr_of_mut!((*DeviceInformation).DeviceListEntry));
                MountMgrFreeDeadDeviceInfo(DeviceInformation);
                break;
            }
            NextEntry = (*NextEntry).Flink;
        }
    }

    // Release driver
    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceLock),
        IO_NO_INCREMENT as i32,
        1,
        0, // FALSE
    );
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrMountedDeviceNotification(
    NotificationStructure: *mut core::ffi::c_void,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let mut OldState: BOOLEAN = 0;
    let mut DeviceExtension: PDEVICE_EXTENSION = core::ptr::null_mut();
    let mut Notification: PDEVICE_INTERFACE_CHANGE_NOTIFICATION = core::ptr::null_mut();

    /* Disable hard errors */
    OldState = PsGetThreadHardErrorsAreDisabled(PsGetCurrentThread());
    PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), 1);

    DeviceExtension = Context as PDEVICE_EXTENSION;
    Notification = NotificationStructure as PDEVICE_INTERFACE_CHANGE_NOTIFICATION;

    DbgPrint(
        c"[MNTMGR] notification: link=%wZ\n".as_ptr(),
        (*Notification).SymbolicLinkName,
    );
    /* Dispatch according to the event */
    if memcmp(
        core::ptr::addr_of!((*Notification).Event) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_DEVICE_INTERFACE_ARRIVAL) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as usize,
    ) == 0
    {
        MountMgrMountedDeviceArrival(DeviceExtension, (*Notification).SymbolicLinkName, 0);
    } else if memcmp(
        core::ptr::addr_of!((*Notification).Event) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_DEVICE_INTERFACE_REMOVAL) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as usize,
    ) == 0
    {
        MountMgrMountedDeviceRemoval(DeviceExtension, (*Notification).SymbolicLinkName);
    }

    /* Reset hard errors */
    PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), OldState);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrCreateClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    const STATUS_NOT_A_DIRECTORY: NTSTATUS = 0xC0000103u32 as NTSTATUS;
    const FILE_DIRECTORY_FILE: u32 = 0x00000001;

    let mut Stack: *mut IO_STACK_LOCATION = core::ptr::null_mut();
    // UNREFERENCED_PARAMETER(DeviceObject);
    let _ = &DeviceObject;

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    Stack = IoGetCurrentIrpStackLocation(Irp);

    if (*Stack).MajorFunction == IRP_MJ_CREATE
        && ((*Stack).Parameters.Create.Options & FILE_DIRECTORY_FILE) != 0
    {
        Status = STATUS_NOT_A_DIRECTORY;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrCancel(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let _ = DeviceObject;
    RemoveEntryList(&raw mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry);
    IoReleaseCancelSpinLock((*Irp).CancelIrql);
    (*Irp).IoStatus.Information = 0 as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}
#[no_mangle]
pub unsafe extern "system" fn MountMgrCleanup(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut ListIrp: PIRP = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut NextEntry: PLIST_ENTRY = core::ptr::null_mut();
    let FileObject: PFILE_OBJECT;
    let Stack: PIO_STACK_LOCATION;
    let DeviceExtension: PDEVICE_EXTENSION;

    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    Stack = IoGetCurrentIrpStackLocation(Irp);
    FileObject = (*Stack).FileObject;
    IoAcquireCancelSpinLock(&raw mut OldIrql);
    if IsListEmpty(&raw mut (*DeviceExtension).IrpListHead) {
        IoReleaseCancelSpinLock(OldIrql);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0 as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return STATUS_SUCCESS;
    }
    NextEntry = (*DeviceExtension).IrpListHead.Flink as PLIST_ENTRY;
    loop {
        ListIrp = (NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry),
        ) as *mut IRP as PIRP;
        if (*IoGetCurrentIrpStackLocation(ListIrp)).FileObject == FileObject {
            (*ListIrp).Cancel = 1;
            (*ListIrp).CancelIrql = OldIrql;
            (*ListIrp).CancelRoutine = None;
            MountMgrCancel(DeviceObject, ListIrp);
            IoAcquireCancelSpinLock(&raw mut OldIrql);
        }
        NextEntry = (*NextEntry).Flink as PLIST_ENTRY;
        if !(NextEntry != &raw mut (*DeviceExtension).IrpListHead) {
            break;
        }
    }
    IoReleaseCancelSpinLock(OldIrql);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0 as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrShutdown(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: *mut _DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as *mut _DEVICE_EXTENSION;

    InterlockedExchange(core::ptr::addr_of_mut!(Unloading), 1);

    KeInitializeEvent(core::ptr::addr_of_mut!(UnloadEvent), _EVENT_TYPE_NotificationEvent, 0);

    if InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceExtension).WorkerReferences)) > 0 {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*DeviceExtension).WorkerSemaphore),
            0,
            1,
            0,
        );
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(UnloadEvent) as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            0,
            core::ptr::null_mut(),
        );
    } else {
        InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceExtension).WorkerReferences));
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, 0_i8);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    mut DriverObject: PDRIVER_OBJECT,
    mut RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceExtension: PDEVICE_EXTENSION = core::ptr::null_mut();

    RtlCreateRegistryKey(RTL_REGISTRY_ABSOLUTE as u32, DatabasePath);

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<DEVICE_EXTENSION>() as u32,
        &raw mut DeviceMount,
        FILE_DEVICE_NETWORK,
        FILE_DEVICE_SECURE_OPEN as u32,
        0u8, // FALSE
        &raw mut DeviceObject,
    );

    if Status < 0 {
        return Status;
    }

    (*DriverObject).DriverUnload = Some(
        core::mem::transmute::<unsafe extern "system" fn(PDRIVER_OBJECT), _>(MountMgrUnload)
    );

    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    RtlZeroMemory(
        DeviceExtension as *mut core::ffi::c_void,
        core::mem::size_of::<DEVICE_EXTENSION>(),
    );

    (*DeviceExtension).DeviceObject = DeviceObject;
    (*DeviceExtension).DriverObject = DriverObject;

    InitializeListHead(&raw mut (*DeviceExtension).DeviceListHead);
    InitializeListHead(&raw mut (*DeviceExtension).OfflineDeviceListHead);

    KeInitializeSemaphore(&raw mut (*DeviceExtension).DeviceLock, 1, 1);
    KeInitializeSemaphore(&raw mut (*DeviceExtension).RemoteDatabaseLock, 1, 1);

    InitializeListHead(&raw mut (*DeviceExtension).IrpListHead);
    (*DeviceExtension).EpicNumber = 1;

    InitializeListHead(&raw mut (*DeviceExtension).SavedLinksListHead);

    InitializeListHead(&raw mut (*DeviceExtension).WorkerQueueListHead);
    KeInitializeSemaphore(&raw mut (*DeviceExtension).WorkerSemaphore, 0, MAXLONG);
    (*DeviceExtension).WorkerReferences = -1;
    KeInitializeSpinLock(&raw mut (*DeviceExtension).WorkerLock);

    InitializeListHead(&raw mut (*DeviceExtension).UniqueIdWorkerItemListHead);
    InitializeListHead(&raw mut (*DeviceExtension).OnlineNotificationListHead);
    (*DeviceExtension).OnlineNotificationCount = 1;

    (*DeviceExtension).RegistryPath.Length = (*RegistryPath).Length;
    (*DeviceExtension).RegistryPath.MaximumLength =
        ((*RegistryPath).Length as usize).wrapping_add(core::mem::size_of::<u16>()) as u16;
    (*DeviceExtension).RegistryPath.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*DeviceExtension).RegistryPath.MaximumLength as usize,
        0x4D6E744D, // tag 'MntM'
    ) as *mut u16;

    if (*DeviceExtension).RegistryPath.Buffer.is_null() {
        IoDeleteDevice(DeviceObject);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyUnicodeString(
        &raw mut (*DeviceExtension).RegistryPath,
        RegistryPath,
    );

    (*DeviceExtension).NoAutoMount = MountmgrReadNoAutoMount(&raw mut (*DeviceExtension).RegistryPath);

    GlobalCreateSymbolicLink(
        &raw mut DosDevicesMount,
        &raw mut DeviceMount,
    );

    Status = IoRegisterPlugPlayNotification(
        EventCategoryDeviceInterfaceChange,
        PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
        &raw mut MountedDevicesGuid as *mut core::ffi::c_void,
        DriverObject,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> i32,
            _,
        >(MountMgrMountedDeviceNotification)),
        DeviceExtension as *mut core::ffi::c_void,
        &raw mut (*DeviceExtension).NotificationEntry,
    );

    if Status < 0 {
        IoDeleteDevice(DeviceObject);
        return Status;
    }

    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(
        core::mem::transmute::<
            unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS,
            _,
        >(MountMgrCreateClose)
    );
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize];
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(
        core::mem::transmute::<unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS, _>(MountMgrDeviceControl),
    );
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = Some(
        core::mem::transmute::<
            unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS,
            _,
        >(MountMgrCleanup)
    );
    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN as usize] = Some(
        core::mem::transmute::<
            unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS,
            _,
        >(MountMgrShutdown)
    );

    gdeviceObject = DeviceObject;

    Status = IoRegisterShutdownNotification(DeviceObject);
    if Status < 0 {
        IoDeleteDevice(DeviceObject);
    }

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2011 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA.
//  *
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/filesystem/mountmgr/mountmgr.c
//  * PURPOSE:          Mount Manager
//  * PROGRAMMER:       Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  *                   Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// #include "mntmgr.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* FIXME */
// GUID MountedDevicesGuid = {0x53F5630D, 0xB6BF, 0x11D0, {0x94, 0xF2, 0x00, 0xA0, 0xC9, 0x1E, 0xFB, 0x8B}};
//
// PDEVICE_OBJECT gdeviceObject;
// KEVENT UnloadEvent;
// LONG Unloading;
//
// static const WCHAR Cunc[] = L"\\??\\C:";
// #define Cunc_LETTER_POSITION 4
//
// /**
//  * @brief
//  * Sends a synchronous IOCTL to the specified device object.
//  *
//  * @param[in]   IoControlCode
//  * The IOCTL to send to the device.
//  *
//  * @param[in]   DeviceObject
//  * Pointer to the device object that will handle the IOCTL.
//  *
//  * @param[in]   InputBuffer
//  * Optional pointer to a buffer containing input data for the IOCTL.
//  * When specified, the buffer should be at least of InputBufferLength size.
//  *
//  * @param[in]   InputBufferLength
//  * Size in bytes, of the buffer pointed by InputBuffer.
//  *
//  * @param[out]  OutputBuffer
//  * Optional pointer to a buffer that will receive output data from the IOCTL.
//  * When specified, the buffer should be at least of OutputBufferLength size.
//  *
//  * @param[in]   OutputBufferLength
//  * Size in bytes, of the buffer pointed by OutputBuffer.
//  *
//  * @param[in]   FileObject
//  * Optional pointer to a file object that may be necessary for the IOCTL.
//  *
//  * @return
//  * An NTSTATUS code indicating success or failure of this function.
//  *
//  * @note
//  * Must be called at PASSIVE_LEVEL with all APCs enabled.
//  **/
// _IRQL_requires_(PASSIVE_LEVEL)
// NTSTATUS
// MountMgrSendSyncDeviceIoCtl(
//      ULONG IoControlCode,
//      PDEVICE_OBJECT DeviceObject,
//      PVOID InputBuffer,
//      ULONG InputBufferLength,
//      PVOID OutputBuffer,
//      ULONG OutputBufferLength,
//      PFILE_OBJECT FileObject)
// {
//     NTSTATUS Status;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//
//     /* We must be at passive level as we are using an on-stack event, and
//      * APCs must be enabled for allowing the Special Kernel APC queued by
//      * the IO Manager to run for completing the IRP */
//     ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);
//     ASSERT(!KeAreAllApcsDisabled());
//
//     /* Initialize the on-stack notification event and build the threaded IRP */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IoControlCode,
//                                         DeviceObject,
//                                         InputBuffer,
//                                         InputBufferLength,
//                                         OutputBuffer,
//                                         OutputBufferLength,
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (!Irp)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Set up the FileObject for the IOCTL if required */
//     if (FileObject)
//         IoGetNextIrpStackLocation(Irp)->FileObject = FileObject;
//
//     /* Finally, call the driver and wait for IRP completion if necessary */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// IsOffline(PUNICODE_STRING SymbolicName)
// {
//     NTSTATUS Status;
//     ULONG IsOffline, Default;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     /* Prepare to look in the registry to see if
//      * given volume is offline
//      */
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
//     QueryTable[0].Name = SymbolicName->Buffer;
//     QueryTable[0].EntryContext = &IsOffline;
//     QueryTable[0].DefaultType = REG_DWORD;
//     QueryTable[0].DefaultLength = sizeof(ULONG);
//     QueryTable[0].DefaultData = &Default;
//
//     Default = 0;
//
//     /* Query status */
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                     OfflinePath,
//                                     QueryTable,
//                                     NULL,
//                                     NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         IsOffline = 0;
//     }
//
//     return (IsOffline != 0);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// HasDriveLetter(IN PDEVICE_INFORMATION DeviceInformation)
// {
//     PLIST_ENTRY NextEntry;
//     PSYMLINK_INFORMATION SymlinkInfo;
//
//     /* Browse all the symlinks to check if there is at least a drive letter */
//     for (NextEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//          NextEntry != &DeviceInformation->SymbolicLinksListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         SymlinkInfo = CONTAINING_RECORD(NextEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//         if (IsDriveLetter(&SymlinkInfo->Name) && SymlinkInfo->Online)
//         {
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// CreateNewDriveLetterName(OUT PUNICODE_STRING DriveLetter,
//                          IN PUNICODE_STRING DeviceName,
//                          IN UCHAR Letter,
//                          IN PMOUNTDEV_UNIQUE_ID UniqueId OPTIONAL)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     /* Allocate a big enough buffer to contain the symbolic link */
//     DriveLetter->MaximumLength = DosDevices.Length + 3 * sizeof(WCHAR);
//     DriveLetter->Buffer = AllocatePool(DriveLetter->MaximumLength);
//     if (!DriveLetter->Buffer)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy prefix */
//     RtlCopyUnicodeString(DriveLetter, &DosDevices);
//
//     /* Update string to reflect real contents */
//     DriveLetter->Length = DosDevices.Length + 2 * sizeof(WCHAR);
//     DriveLetter->Buffer[DosDevices.Length / sizeof(WCHAR) + 2] = UNICODE_NULL;
//     DriveLetter->Buffer[DosDevices.Length / sizeof(WCHAR) + 1] = L':';
//
//     /* If caller wants a no drive entry */
//     if (Letter == (UCHAR)-1)
//     {
//         /* Then, create a no letter entry */
//         CreateNoDriveLetterEntry(UniqueId);
//         FreePool(DriveLetter->Buffer);
//         return STATUS_UNSUCCESSFUL;
//     }
//     else if (Letter)
//     {
//         /* Use the letter given by the caller */
//         DriveLetter->Buffer[DosDevices.Length / sizeof(WCHAR)] = (WCHAR)Letter;
//         Status = GlobalCreateSymbolicLink(DriveLetter, DeviceName);
//         if (NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     /* If caller didn't provide a letter, let's find one for him */
//
//     if (RtlPrefixUnicodeString(&DeviceFloppy, DeviceName, TRUE))
//     {
//         /* If the device is a floppy, start with letter A */
//         Letter = 'A';
//     }
//     else if (RtlPrefixUnicodeString(&DeviceCdRom, DeviceName, TRUE))
//     {
//         /* If the device is a CD-ROM, start with letter D */
//         Letter = 'D';
//     }
//     else
//     {
//         /* Finally, if it's a disk, use C */
//         Letter = 'C';
//     }
//
//     /* Try to affect a letter (up to Z, ofc) until it's possible */
//     for (; Letter <= 'Z'; Letter++)
//     {
//         DriveLetter->Buffer[DosDevices.Length / sizeof(WCHAR)] = (WCHAR)Letter;
//         Status = GlobalCreateSymbolicLink(DriveLetter, DeviceName);
//         if (NT_SUCCESS(Status))
//         {
//             DPRINT("Assigned drive %c: to %wZ\n", Letter, DeviceName);
//             return Status;
//         }
//     }
//
//     /* We failed to allocate a letter */
//     FreePool(DriveLetter->Buffer);
//     DPRINT("Failed to create a drive letter for %wZ\n", DeviceName);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// QueryDeviceInformation(
//      PUNICODE_STRING SymbolicName,
//      PUNICODE_STRING DeviceName,
//      PMOUNTDEV_UNIQUE_ID* UniqueId,
//      PBOOLEAN Removable,
//      PBOOLEAN GptDriveLetter,
//      PBOOLEAN HasGuid,
//      LPGUID StableGuid,
//      PBOOLEAN IsFT)
// {
//     NTSTATUS Status;
//     USHORT Size;
//     BOOLEAN IsRemovable;
//     PMOUNTDEV_NAME Name;
//     PMOUNTDEV_UNIQUE_ID Id;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     PARTITION_INFORMATION_EX PartitionInfo;
//     STORAGE_DEVICE_NUMBER StorageDeviceNumber;
//     VOLUME_GET_GPT_ATTRIBUTES_INFORMATION GptAttributes;
//
//     /* Get device associated with the symbolic name */
//     Status = IoGetDeviceObjectPointer(SymbolicName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* The associate FO can't have a file name */
//     if (FileObject->FileName.Length)
//     {
//         ObDereferenceObject(FileObject);
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     /* Check if it's removable & return to the user (if asked to) */
//     IsRemovable = (FileObject->DeviceObject->Characteristics & FILE_REMOVABLE_MEDIA);
//     if (Removable)
//     {
//         *Removable = IsRemovable;
//     }
//
//     /* Get the attached device */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     /* If we've been asked for a GPT drive letter */
//     if (GptDriveLetter)
//     {
//         /* Consider it has one */
//         *GptDriveLetter = TRUE;
//
//         if (!IsRemovable)
//         {
//             /* Query the GPT attributes */
//             Status = MountMgrSendSyncDeviceIoCtl(IOCTL_VOLUME_GET_GPT_ATTRIBUTES,
//                                                  DeviceObject,
//                                                  NULL,
//                                                  0,
//                                                  &GptAttributes,
//                                                  sizeof(GptAttributes),
//                                                  NULL);
//             /* Failure isn't major */
//             if (!NT_SUCCESS(Status))
//             {
//                 Status = STATUS_SUCCESS;
//             }
//             /* Check if it has a drive letter */
//             else if (GptAttributes.GptAttributes & GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER)
//             {
//                 *GptDriveLetter = FALSE;
//             }
//         }
//     }
//
//     /* If caller wants to know if this is a FT volume */
//     if (IsFT)
//     {
//         /* Suppose it's not */
//         *IsFT = FALSE;
//
//         /* FT volume can't be removable */
//         if (!IsRemovable)
//         {
//             /* Query partition information */
//             Status = MountMgrSendSyncDeviceIoCtl(IOCTL_DISK_GET_PARTITION_INFO_EX,
//                                                  DeviceObject,
//                                                  NULL,
//                                                  0,
//                                                  &PartitionInfo,
//                                                  sizeof(PartitionInfo),
//                                                  NULL);
//             /* Failure isn't major */
//             if (!NT_SUCCESS(Status))
//             {
//                 Status = STATUS_SUCCESS;
//             }
//             /* Check if this is a FT volume */
//             else if ((PartitionInfo.PartitionStyle == PARTITION_STYLE_MBR) &&
//                      IsFTPartition(PartitionInfo.Mbr.PartitionType))
//             {
//                 *IsFT = TRUE;
//             }
//
//             /* It looks like a FT volume. Verify it is really one by checking
//              * that it does NOT lie on a specific storage device (i.e. it is
//              * not a basic volume). */
//             if (*IsFT)
//             {
//                 Status = MountMgrSendSyncDeviceIoCtl(IOCTL_STORAGE_GET_DEVICE_NUMBER,
//                                                      DeviceObject,
//                                                      NULL,
//                                                      0,
//                                                      &StorageDeviceNumber,
//                                                      sizeof(StorageDeviceNumber),
//                                                      NULL);
//                 if (!NT_SUCCESS(Status))
//                     Status = STATUS_SUCCESS;
//                 else
//                     *IsFT = FALSE; // Succeeded, so this cannot be a FT volume.
//             }
//         }
//     }
//
//     /* If caller needs device name */
//     if (DeviceName)
//     {
//         /* Allocate a buffer just to request length */
//         Name = AllocatePool(sizeof(MOUNTDEV_NAME));
//         if (!Name)
//         {
//             ObDereferenceObject(DeviceObject);
//             ObDereferenceObject(FileObject);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Query device name */
//         Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
//                                              DeviceObject,
//                                              NULL,
//                                              0,
//                                              Name,
//                                              sizeof(MOUNTDEV_NAME),
//                                              FileObject);
//         /* Retry with appropriate length */
//         if (Status == STATUS_BUFFER_OVERFLOW)
//         {
//             Size = Name->NameLength + sizeof(MOUNTDEV_NAME);
//
//             FreePool(Name);
//
//             /* Allocate proper size */
//             Name = AllocatePool(Size);
//             if (!Name)
//             {
//                 ObDereferenceObject(DeviceObject);
//                 ObDereferenceObject(FileObject);
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* And query name (for real that time) */
//             Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
//                                                  DeviceObject,
//                                                  NULL,
//                                                  0,
//                                                  Name,
//                                                  Size,
//                                                  FileObject);
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             /* Copy back found name to the caller */
//             DeviceName->Length = Name->NameLength;
//             DeviceName->MaximumLength = Name->NameLength + sizeof(WCHAR);
//             DeviceName->Buffer = AllocatePool(DeviceName->MaximumLength);
//             if (!DeviceName->Buffer)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//             else
//             {
//                 RtlCopyMemory(DeviceName->Buffer, Name->Name, Name->NameLength);
//                 DeviceName->Buffer[Name->NameLength / sizeof(WCHAR)] = UNICODE_NULL;
//             }
//         }
//
//         FreePool(Name);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         ObDereferenceObject(DeviceObject);
//         ObDereferenceObject(FileObject);
//         return Status;
//     }
//
//     /* If caller wants device unique ID */
//     if (UniqueId)
//     {
//         /* Prepare buffer to probe length */
//         Id = AllocatePool(sizeof(MOUNTDEV_UNIQUE_ID));
//         if (!Id)
//         {
//             ObDereferenceObject(DeviceObject);
//             ObDereferenceObject(FileObject);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Query unique ID length */
//         Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_QUERY_UNIQUE_ID,
//                                              DeviceObject,
//                                              NULL,
//                                              0,
//                                              Id,
//                                              sizeof(MOUNTDEV_UNIQUE_ID),
//                                              FileObject);
//         /* Retry with appropriate length */
//         if (Status == STATUS_BUFFER_OVERFLOW)
//         {
//             Size = Id->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID);
//
//             FreePool(Id);
//
//             /* Allocate the correct buffer */
//             Id = AllocatePool(Size);
//             if (!Id)
//             {
//                 ObDereferenceObject(DeviceObject);
//                 ObDereferenceObject(FileObject);
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* Query unique ID */
//             Status = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_QUERY_UNIQUE_ID,
//                                                  DeviceObject,
//                                                  NULL,
//                                                  0,
//                                                  Id,
//                                                  Size,
//                                                  FileObject);
//         }
//
//         /* Hands back unique ID */
//         if (NT_SUCCESS(Status))
//         {
//             *UniqueId = Id;
//         }
//         else
//         {
//             /* In case of failure, also free the rest */
//             FreePool(Id);
//             if (DeviceName->Length)
//                 FreePool(DeviceName->Buffer);
//
//             ObDereferenceObject(DeviceObject);
//             ObDereferenceObject(FileObject);
//             return Status;
//         }
//     }
//
//     /* If user wants to know about GUID */
//     if (HasGuid)
//     {
//         /* Query device stable GUID */
//         NTSTATUS IntStatus;
//         IntStatus = MountMgrSendSyncDeviceIoCtl(IOCTL_MOUNTDEV_QUERY_STABLE_GUID,
//                                                 DeviceObject,
//                                                 NULL,
//                                                 0,
//                                                 StableGuid,
//                                                 sizeof(GUID),
//                                                 FileObject);
//         *HasGuid = NT_SUCCESS(IntStatus);
//     }
//
//     ObDereferenceObject(DeviceObject);
//     ObDereferenceObject(FileObject);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// FindDeviceInfo(IN PDEVICE_EXTENSION DeviceExtension,
//                IN PUNICODE_STRING SymbolicName,
//                IN BOOLEAN DeviceNameGiven,
//                OUT PDEVICE_INFORMATION * DeviceInformation)
// {
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     UNICODE_STRING DeviceName;
//     PDEVICE_INFORMATION DeviceInfo = NULL;
//
//     /* If a device name was given, use it */
//     if (DeviceNameGiven)
//     {
//         DeviceName.Length = SymbolicName->Length;
//         DeviceName.Buffer = SymbolicName->Buffer;
//     }
//     else
//     {
//         /* Otherwise, query it */
//         Status = QueryDeviceInformation(SymbolicName,
//                                         &DeviceName,
//                                         NULL, NULL,
//                                         NULL, NULL,
//                                         NULL, NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     /* Look for device information matching devive */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         DeviceInfo = CONTAINING_RECORD(NextEntry,
//                                        DEVICE_INFORMATION,
//                                        DeviceListEntry);
//
//         if (RtlEqualUnicodeString(&DeviceName, &(DeviceInfo->DeviceName), TRUE))
//         {
//             break;
//         }
//     }
//
//     /* Release our buffer if required */
//     if (!DeviceNameGiven)
//     {
//         FreePool(DeviceName.Buffer);
//     }
//
//     /* Return found information */
//     if (NextEntry == &(DeviceExtension->DeviceListHead))
//     {
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     *DeviceInformation = DeviceInfo;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrFreeDeadDeviceInfo(IN PDEVICE_INFORMATION DeviceInformation)
// {
//     FreePool(DeviceInformation->SymbolicName.Buffer);
//     FreePool(DeviceInformation);
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrFreeMountedDeviceInfo(IN PDEVICE_INFORMATION DeviceInformation)
// {
//     PLIST_ENTRY NextEntry;
//     PSYMLINK_INFORMATION SymLink;
//     PUNIQUE_ID_REPLICATE UniqueId;
//     PASSOCIATED_DEVICE_ENTRY AssociatedDevice;
//
//     /* Purge symbolic links list */
//     while (!IsListEmpty(&(DeviceInformation->SymbolicLinksListHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceInformation->SymbolicLinksListHead));
//         SymLink = CONTAINING_RECORD(NextEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//         GlobalDeleteSymbolicLink(&(SymLink->Name));
//         FreePool(SymLink->Name.Buffer);
//     }
//
//     /* Purge replicated unique IDs list */
//     while (!IsListEmpty(&(DeviceInformation->ReplicatedUniqueIdsListHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceInformation->ReplicatedUniqueIdsListHead));
//         UniqueId = CONTAINING_RECORD(NextEntry, UNIQUE_ID_REPLICATE, ReplicatedUniqueIdsListEntry);
//
//         FreePool(UniqueId->UniqueId);
//         FreePool(UniqueId);
//     }
//
//     while (!IsListEmpty(&(DeviceInformation->AssociatedDevicesHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceInformation->AssociatedDevicesHead));
//         AssociatedDevice = CONTAINING_RECORD(NextEntry, ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//
//         FreePool(AssociatedDevice->String.Buffer);
//         FreePool(AssociatedDevice);
//     }
//
//     /* Free the rest of the buffers */
//     FreePool(DeviceInformation->SymbolicName.Buffer);
//     if (DeviceInformation->KeepLinks)
//     {
//         FreePool(DeviceInformation->UniqueId);
//     }
//     FreePool(DeviceInformation->DeviceName.Buffer);
//
//     /* Finally, stop waiting for notifications for this device */
//     if (DeviceInformation->TargetDeviceNotificationEntry)
//     {
//         IoUnregisterPlugPlayNotification(DeviceInformation->TargetDeviceNotificationEntry);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrFreeSavedLink(IN PSAVED_LINK_INFORMATION SavedLinkInformation)
// {
//     PLIST_ENTRY NextEntry;
//     PSYMLINK_INFORMATION SymlinkInformation;
//
//     /* For all the saved links */
//     while (!IsListEmpty(&(SavedLinkInformation->SymbolicLinksListHead)))
//     {
//         NextEntry = RemoveHeadList(&(SavedLinkInformation->SymbolicLinksListHead));
//         SymlinkInformation = CONTAINING_RECORD(NextEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//         /* Remove from system & free */
//         GlobalDeleteSymbolicLink(&(SymlinkInformation->Name));
//         FreePool(SymlinkInformation->Name.Buffer);
//         FreePool(SymlinkInformation);
//     }
//
//     /* And free unique ID & entry */
//     FreePool(SavedLinkInformation->UniqueId);
//     FreePool(SavedLinkInformation);
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MountMgrUnload(IN PDRIVER_OBJECT DriverObject)
// {
//     PLIST_ENTRY NextEntry;
//     PUNIQUE_ID_WORK_ITEM WorkItem;
//     PDEVICE_EXTENSION DeviceExtension;
//     PDEVICE_INFORMATION DeviceInformation;
//     PSAVED_LINK_INFORMATION SavedLinkInformation;
//
//     UNREFERENCED_PARAMETER(DriverObject);
//
//     /* Don't get notification any longer */
//     IoUnregisterShutdownNotification(gdeviceObject);
//
//     /* Free registry buffer */
//     DeviceExtension = gdeviceObject->DeviceExtension;
//     if (DeviceExtension->RegistryPath.Buffer)
//     {
//         FreePool(DeviceExtension->RegistryPath.Buffer);
//         DeviceExtension->RegistryPath.Buffer = NULL;
//     }
//
//     InterlockedExchange(&Unloading, TRUE);
//
//     KeInitializeEvent(&UnloadEvent, NotificationEvent, FALSE);
//
//     /* Wait for workers to finish */
//     if (InterlockedIncrement(&DeviceExtension->WorkerReferences) > 0)
//     {
//         KeReleaseSemaphore(&(DeviceExtension->WorkerSemaphore),
//                            IO_NO_INCREMENT, 1, FALSE);
//
//         KeWaitForSingleObject(&UnloadEvent, Executive, KernelMode, FALSE, NULL);
//     }
//     else
//     {
//         InterlockedDecrement(&(DeviceExtension->WorkerReferences));
//     }
//
//     /* Don't get any notification any longer² */
//     IoUnregisterPlugPlayNotification(DeviceExtension->NotificationEntry);
//
//     /* Acquire the driver exclusively */
//     KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode,
//                           FALSE, NULL);
//
//     /* Clear offline devices list */
//     while (!IsListEmpty(&(DeviceExtension->OfflineDeviceListHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceExtension->OfflineDeviceListHead));
//         DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//         MountMgrFreeDeadDeviceInfo(DeviceInformation);
//     }
//
//     /* Clear saved links list */
//     while (!IsListEmpty(&(DeviceExtension->SavedLinksListHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceExtension->SavedLinksListHead));
//         SavedLinkInformation = CONTAINING_RECORD(NextEntry, SAVED_LINK_INFORMATION, SavedLinksListEntry);
//         MountMgrFreeSavedLink(SavedLinkInformation);
//     }
//
//     /* Clear workers list */
//     while (!IsListEmpty(&(DeviceExtension->UniqueIdWorkerItemListHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceExtension->UniqueIdWorkerItemListHead));
//         WorkItem = CONTAINING_RECORD(NextEntry, UNIQUE_ID_WORK_ITEM, UniqueIdWorkerItemListEntry);
//
//         KeClearEvent(&UnloadEvent);
//         WorkItem->Event = &UnloadEvent;
//
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT,
//                            1, FALSE);
//
//         IoCancelIrp(WorkItem->Irp);
//         KeWaitForSingleObject(&UnloadEvent, Executive, KernelMode, FALSE, NULL);
//
//         IoFreeIrp(WorkItem->Irp);
//         FreePool(WorkItem->DeviceName.Buffer);
//         FreePool(WorkItem->IrpBuffer);
//         FreePool(WorkItem);
//
//         KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode,
//                               FALSE, NULL);
//     }
//
//     /* If we have drive letter data, release */
//     if (DeviceExtension->DriveLetterData)
//     {
//         FreePool(DeviceExtension->DriveLetterData);
//         DeviceExtension->DriveLetterData = NULL;
//     }
//
//     /* Release driver & quit */
//     KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//     GlobalDeleteSymbolicLink(&DosDevicesMount);
//     IoDeleteDevice(gdeviceObject);
// }
//
// /**
//  * @brief   Retrieves the "NoAutoMount" setting.
//  * @return  TRUE if AutoMount is disabled; FALSE if AutoMount is enabled.
//  **/
// CODE_SEG("INIT")
// BOOLEAN
// MountmgrReadNoAutoMount(
//      PUNICODE_STRING RegistryPath)
// {
//     NTSTATUS Status;
//     ULONG Result, Default = 0;
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//
//     /* Retrieve data from registry */
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//     QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
//     QueryTable[0].Name = L"NoAutoMount";
//     QueryTable[0].EntryContext = &Result;
//     QueryTable[0].DefaultType = REG_DWORD;
//     QueryTable[0].DefaultData = &Default;
//     QueryTable[0].DefaultLength = sizeof(Default);
//
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                     RegistryPath->Buffer,
//                                     QueryTable,
//                                     NULL,
//                                     NULL);
//     if (!NT_SUCCESS(Status))
//         Result = Default;
//
//     return (Result != 0);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// MountMgrMountedDeviceArrival(IN PDEVICE_EXTENSION DeviceExtension,
//                              IN PUNICODE_STRING SymbolicName,
//                              IN BOOLEAN ManuallyRegistered)
// {
//     WCHAR Letter;
//     GUID StableGuid;
//     HANDLE LinkHandle;
//     ULONG SymLinkCount, i;
//     PLIST_ENTRY NextEntry;
//     PUNICODE_STRING SymLinks;
//     NTSTATUS Status, IntStatus;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PSYMLINK_INFORMATION SymlinkInformation;
//     PMOUNTDEV_UNIQUE_ID UniqueId, NewUniqueId;
//     PSAVED_LINK_INFORMATION SavedLinkInformation;
//     PDEVICE_INFORMATION DeviceInformation, CurrentDevice;
//     WCHAR CSymLinkBuffer[RTL_NUMBER_OF(Cunc)], LinkTargetBuffer[MAX_PATH];
//     UNICODE_STRING TargetDeviceName, SuggestedLinkName, DeviceName, VolumeName, DriveLetter, LinkTarget, CSymLink;
//     BOOLEAN HasGuid, HasGptDriveLetter, IsFT, UseOnlyIfThereAreNoOtherLinks;
//     BOOLEAN IsDrvLetter, IsOff, IsVolumeName, SetOnline;
//
//     /* New device = new structure to represent it */
//     DeviceInformation = AllocatePool(sizeof(DEVICE_INFORMATION));
//     if (!DeviceInformation)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialise device structure */
//     RtlZeroMemory(DeviceInformation, sizeof(DEVICE_INFORMATION));
//     InitializeListHead(&(DeviceInformation->SymbolicLinksListHead));
//     InitializeListHead(&(DeviceInformation->ReplicatedUniqueIdsListHead));
//     InitializeListHead(&(DeviceInformation->AssociatedDevicesHead));
//     DeviceInformation->SymbolicName.Length = SymbolicName->Length;
//     DeviceInformation->SymbolicName.MaximumLength = SymbolicName->Length + sizeof(UNICODE_NULL);
//     DeviceInformation->SymbolicName.Buffer = AllocatePool(DeviceInformation->SymbolicName.MaximumLength);
//     if (!DeviceInformation->SymbolicName.Buffer)
//     {
//         FreePool(DeviceInformation);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy symbolic name */
//     RtlCopyMemory(DeviceInformation->SymbolicName.Buffer, SymbolicName->Buffer, SymbolicName->Length);
//     DeviceInformation->SymbolicName.Buffer[DeviceInformation->SymbolicName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     DeviceInformation->ManuallyRegistered = ManuallyRegistered;
//     DeviceInformation->DeviceExtension = DeviceExtension;
//
//     /* Query as much data as possible about device */
//     Status = QueryDeviceInformation(SymbolicName,
//                                     &TargetDeviceName,
//                                     &UniqueId,
//                                     &(DeviceInformation->Removable),
//                                     &HasGptDriveLetter,
//                                     &HasGuid,
//                                     &StableGuid,
//                                     &IsFT);
//     if (!NT_SUCCESS(Status))
//     {
//         KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//
//         for (NextEntry = DeviceExtension->OfflineDeviceListHead.Flink;
//              NextEntry != &(DeviceExtension->OfflineDeviceListHead);
//              NextEntry = NextEntry->Flink)
//         {
//             CurrentDevice = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//             if (RtlEqualUnicodeString(&(DeviceInformation->SymbolicName), &(CurrentDevice->SymbolicName), TRUE))
//             {
//                 break;
//             }
//         }
//
//         if (NextEntry != &(DeviceExtension->OfflineDeviceListHead))
//         {
//             MountMgrFreeDeadDeviceInfo(DeviceInformation);
//         }
//         else
//         {
//             InsertTailList(&(DeviceExtension->OfflineDeviceListHead), &(DeviceInformation->DeviceListEntry));
//         }
//
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//         return Status;
//     }
//
//     /* Save gathered data */
//     DeviceInformation->UniqueId = UniqueId;
//     DeviceInformation->DeviceName = TargetDeviceName;
//     DeviceInformation->KeepLinks = FALSE;
//
//     /* If we found system partition, mark it */
//     if (DeviceExtension->DriveLetterData && UniqueId->UniqueIdLength == DeviceExtension->DriveLetterData->UniqueIdLength)
//     {
//         if (RtlCompareMemory(UniqueId->UniqueId, DeviceExtension->DriveLetterData->UniqueId, UniqueId->UniqueIdLength)
//             == UniqueId->UniqueIdLength)
//         {
//             IoSetSystemPartition(&TargetDeviceName);
//         }
//     }
//
//     /* Check suggested link name */
//     Status = QuerySuggestedLinkName(&(DeviceInformation->SymbolicName),
//                                     &SuggestedLinkName,
//                                     &UseOnlyIfThereAreNoOtherLinks);
//     if (!NT_SUCCESS(Status))
//     {
//         SuggestedLinkName.Buffer = NULL;
//     }
//
//     /* If it's OK, set it and save its letter (if any) */
//     if (SuggestedLinkName.Buffer && IsDriveLetter(&SuggestedLinkName))
//     {
//         DeviceInformation->SuggestedDriveLetter = (UCHAR)SuggestedLinkName.Buffer[LETTER_POSITION];
//     }
//
//     /* Acquire driver exclusively */
//     KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//
//     /* Check if we already have device in to prevent double registration */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         CurrentDevice = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         if (RtlEqualUnicodeString(&(CurrentDevice->DeviceName), &TargetDeviceName, TRUE))
//         {
//             break;
//         }
//     }
//
//     /* If we found it, clear ours, and return success, all correct */
//     if (NextEntry != &(DeviceExtension->DeviceListHead))
//     {
//         if (SuggestedLinkName.Buffer)
//         {
//             FreePool(SuggestedLinkName.Buffer);
//         }
//
//         FreePool(UniqueId);
//         FreePool(TargetDeviceName.Buffer);
//         FreePool(DeviceInformation->SymbolicName.Buffer);
//         FreePool(DeviceInformation);
//
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//         return STATUS_SUCCESS;
//     }
//
//     /* Check if there are symlinks associated with our device in registry */
//     Status = QuerySymbolicLinkNamesFromStorage(DeviceExtension,
//                                                DeviceInformation,
//                                                (SuggestedLinkName.Buffer) ? &SuggestedLinkName : NULL,
//                                                UseOnlyIfThereAreNoOtherLinks,
//                                                &SymLinks,
//                                                &SymLinkCount,
//                                                HasGuid,
//                                                &StableGuid);
//
//     /* If our device is a CD-ROM */
//     if (RtlPrefixUnicodeString(&DeviceCdRom, &TargetDeviceName, TRUE))
//     {
//         LinkTarget.Length = 0;
//         LinkTarget.MaximumLength = sizeof(LinkTargetBuffer);
//         LinkTarget.Buffer = LinkTargetBuffer;
//
//         RtlCopyMemory(CSymLinkBuffer, Cunc, sizeof(Cunc));
//         RtlInitUnicodeString(&CSymLink, CSymLinkBuffer);
//
//         /* Start checking all letters that could have been associated */
//         for (Letter = L'D'; Letter <= L'Z'; Letter++)
//         {
//             CSymLink.Buffer[Cunc_LETTER_POSITION] = Letter;
//
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &CSymLink,
//                                        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                        NULL,
//                                        NULL);
//
//             /* Try to open the associated symlink */
//             Status = ZwOpenSymbolicLinkObject(&LinkHandle, SYMBOLIC_LINK_QUERY, &ObjectAttributes);
//             if (!NT_SUCCESS(Status))
//             {
//                 continue;
//             }
//
//             /* And query its target */
//             Status = ZwQuerySymbolicLinkObject(LinkHandle, &LinkTarget, NULL);
//             ZwClose(LinkHandle);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 continue;
//             }
//
//             IntStatus = STATUS_UNSUCCESSFUL;
//             if (!RtlEqualUnicodeString(&LinkTarget, &DeviceInformation->DeviceName, FALSE))
//             {
//                 continue;
//             }
//
//             /* This link is matching our device, whereas it's not supposed to have any
//              * symlink associated.
//              * Delete it
//              */
//             if (!SymLinkCount)
//             {
//                 IoDeleteSymbolicLink(&CSymLink);
//                 continue;
//             }
//
//             /* Now, for all the symlinks, check for ours */
//             for (i = 0; i < SymLinkCount; i++)
//             {
//                 if (IsDriveLetter(&(SymLinks[i])))
//                 {
//                     /* If it exists, that's correct */
//                     if (SymLinks[i].Buffer[LETTER_POSITION] == Letter)
//                     {
//                         IntStatus = STATUS_SUCCESS;
//                     }
//                 }
//             }
//
//             /* Useless link, delete it */
//             if (IntStatus == STATUS_UNSUCCESSFUL)
//             {
//                 IoDeleteSymbolicLink(&CSymLink);
//             }
//         }
//     }
//
//     /* Suggested name is no longer required */
//     if (SuggestedLinkName.Buffer)
//     {
//         FreePool(SuggestedLinkName.Buffer);
//     }
//
//     /* If if failed, ensure we don't take symlinks into account */
//     if (!NT_SUCCESS(Status))
//     {
//         SymLinks = NULL;
//         SymLinkCount = 0;
//     }
//
//     /* Now we queried them, remove the symlinks */
//     SavedLinkInformation = RemoveSavedLinks(DeviceExtension, UniqueId);
//
//     IsDrvLetter = FALSE;
//     IsOff = FALSE;
//     IsVolumeName = FALSE;
//     /* For all the symlinks */
//     for (i = 0; i < SymLinkCount; i++)
//     {
//         /* Check if our device is a volume */
//         if (MOUNTMGR_IS_VOLUME_NAME(&(SymLinks[i])))
//         {
//             IsVolumeName = TRUE;
//         }
//         /* If it has a drive letter */
//         else if (IsDriveLetter(&(SymLinks[i])))
//         {
//             if (IsDrvLetter)
//             {
//                 DeleteFromLocalDatabase(&(SymLinks[i]), UniqueId);
//                 continue;
//             }
//             else
//             {
//                 IsDrvLetter = TRUE;
//             }
//         }
//
//         /* And recreate the symlink to our device */
//         Status = GlobalCreateSymbolicLink(&(SymLinks[i]), &TargetDeviceName);
//         if (!NT_SUCCESS(Status))
//         {
//             BOOLEAN LinkError = TRUE;
//
//             if ((SavedLinkInformation && !RedirectSavedLink(SavedLinkInformation, &(SymLinks[i]), &TargetDeviceName)) ||
//                 !SavedLinkInformation)
//             {
//                 Status = QueryDeviceInformation(&(SymLinks[i]), &DeviceName, NULL, NULL, NULL, NULL, NULL, NULL);
//                 if (NT_SUCCESS(Status))
//                 {
//                     LinkError = RtlEqualUnicodeString(&TargetDeviceName, &DeviceName, TRUE);
//                     FreePool(DeviceName.Buffer);
//                 }
//
//                 if (!LinkError)
//                 {
//                     if (IsDriveLetter(&(SymLinks[i])))
//                     {
//                         IsDrvLetter = FALSE;
//                         DeleteFromLocalDatabase(&(SymLinks[i]), UniqueId);
//                     }
//
//                     FreePool(SymLinks[i].Buffer);
//                     continue;
//                 }
//             }
//         }
//
//         /* Check if was offline */
//         if (IsOffline(&(SymLinks[i])))
//         {
//             IsOff = TRUE;
//         }
//
//         /* Finally, associate this symlink with the device */
//         SymlinkInformation = AllocatePool(sizeof(SYMLINK_INFORMATION));
//         if (!SymlinkInformation)
//         {
//             GlobalDeleteSymbolicLink(&(SymLinks[i]));
//             FreePool(SymLinks[i].Buffer);
//             continue;
//         }
//
//         SymlinkInformation->Name = SymLinks[i];
//         SymlinkInformation->Online = TRUE;
//
//         InsertTailList(&(DeviceInformation->SymbolicLinksListHead),
//                        &(SymlinkInformation->SymbolicLinksListEntry));
//     }
//
//     /* Now, for all the recreated symlinks, notify their recreation */
//     for (NextEntry = DeviceInformation->SymbolicLinksListHead.Flink;
//          NextEntry != &(DeviceInformation->SymbolicLinksListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         SymlinkInformation = CONTAINING_RECORD(NextEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//         SendLinkCreated(&(SymlinkInformation->Name));
//     }
//
//     /* If we had saved links, it's time to free them */
//     if (SavedLinkInformation)
//     {
//         MountMgrFreeSavedLink(SavedLinkInformation);
//     }
//
//     /* If our device doesn't have a volume name */
//     if (!IsVolumeName)
//     {
//         /* It's time to create one */
//         Status = CreateNewVolumeName(&VolumeName, NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Write it to global database */
//             RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                   DatabasePath,
//                                   VolumeName.Buffer,
//                                   REG_BINARY,
//                                   UniqueId->UniqueId,
//                                   UniqueId->UniqueIdLength);
//
//             /* And create the symlink */
//             GlobalCreateSymbolicLink(&VolumeName, &TargetDeviceName);
//
//             SymlinkInformation = AllocatePool(sizeof(SYMLINK_INFORMATION));
//             if (!SymlinkInformation)
//             {
//                 FreePool(VolumeName.Buffer);
//             }
//             /* Finally, associate it with the device and notify creation */
//             else
//             {
//                 SymlinkInformation->Name = VolumeName;
//                 SymlinkInformation->Online = TRUE;
//                 InsertTailList(&(DeviceInformation->SymbolicLinksListHead),
//                                &(SymlinkInformation->SymbolicLinksListEntry));
//
//                 SendLinkCreated(&VolumeName);
//             }
//         }
//     }
//
//     /* If we found a drive letter, then, ignore the suggested one */
//     if (IsDrvLetter)
//     {
//         DeviceInformation->SuggestedDriveLetter = 0;
//     }
//     /* Else, it's time to set up one */
//     else if ((!DeviceExtension->NoAutoMount || DeviceInformation->Removable) &&
//              DeviceExtension->AutomaticDriveLetter &&
//              (HasGptDriveLetter || DeviceInformation->SuggestedDriveLetter) &&
//              !HasNoDriveLetterEntry(UniqueId))
//     {
//         /* Create a new drive letter */
//         Status = CreateNewDriveLetterName(&DriveLetter, &TargetDeviceName,
//                                           DeviceInformation->SuggestedDriveLetter,
//                                           NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             CreateNoDriveLetterEntry(UniqueId);
//         }
//         else
//         {
//             /* Save it to global database */
//             RtlWriteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                   DatabasePath,
//                                   DriveLetter.Buffer,
//                                   REG_BINARY,
//                                   UniqueId->UniqueId,
//                                   UniqueId->UniqueIdLength);
//
//             /* Associate it with the device and notify creation */
//             SymlinkInformation = AllocatePool(sizeof(SYMLINK_INFORMATION));
//             if (!SymlinkInformation)
//             {
//                 FreePool(DriveLetter.Buffer);
//             }
//             else
//             {
//                 SymlinkInformation->Name = DriveLetter;
//                 SymlinkInformation->Online = TRUE;
//                 InsertTailList(&(DeviceInformation->SymbolicLinksListHead),
//                                &(SymlinkInformation->SymbolicLinksListEntry));
//
//                 SendLinkCreated(&DriveLetter);
//             }
//         }
//     }
//
//     /* If that's a PnP device, register for notifications */
//     if (!ManuallyRegistered)
//     {
//         RegisterForTargetDeviceNotification(DeviceExtension, DeviceInformation);
//     }
//
//     /* Finally, insert the device into our devices list */
//     InsertTailList(&(DeviceExtension->DeviceListHead), &(DeviceInformation->DeviceListEntry));
//
//     /* Copy device unique ID */
//     NewUniqueId = AllocatePool(UniqueId->UniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//     if (NewUniqueId)
//     {
//         NewUniqueId->UniqueIdLength = UniqueId->UniqueIdLength;
//         RtlCopyMemory(NewUniqueId->UniqueId, UniqueId->UniqueId, UniqueId->UniqueIdLength);
//     }
//
//     /* Skip online notifications if the device is offline or a FT volume */
//     if (IsOff || IsFT)
//         DeviceInformation->SkipNotifications = TRUE;
//
//     /* If automount is enabled or the device was already mounted, send now
//      * the online notification if needed; otherwise, defer its posting */
//     if (!DeviceExtension->NoAutoMount || IsDrvLetter)
//         SetOnline = !DeviceInformation->SkipNotifications;
//     else
//         SetOnline = FALSE;
//
//     /* Finally, release the exclusive lock */
//     KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//     /* Set the device online now if necessary */
//     if (SetOnline)
//         SendOnlineNotification(SymbolicName);
//
//     /* If we had symlinks (from storage), free them */
//     if (SymLinks)
//     {
//         FreePool(SymLinks);
//     }
//
//     /* Notify about unique id change */
//     if (NewUniqueId)
//     {
//         IssueUniqueIdChangeNotify(DeviceExtension, SymbolicName, NewUniqueId);
//         FreePool(NewUniqueId);
//     }
//
//     /* If this drive was set to have a drive letter automatically
//      * Now it's back, local databases sync will be required
//      */
//     if (DeviceExtension->AutomaticDriveLetter)
//     {
//         KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//
//         ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
//
//         NextEntry = DeviceExtension->DeviceListHead.Flink;
//         CurrentDevice = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//         while (CurrentDevice != DeviceInformation)
//         {
//             if (!CurrentDevice->NoDatabase)
//             {
//                 ReconcileThisDatabaseWithMaster(DeviceExtension, CurrentDevice);
//             }
//
//             NextEntry = NextEntry->Flink;
//             CurrentDevice = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//         }
//
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrMountedDeviceRemoval(IN PDEVICE_EXTENSION DeviceExtension,
//                              IN PUNICODE_STRING DeviceName)
// {
//     PLIST_ENTRY NextEntry, DeviceEntry;
//     PUNIQUE_ID_REPLICATE UniqueIdReplicate;
//     PSYMLINK_INFORMATION SymlinkInformation;
//     PASSOCIATED_DEVICE_ENTRY AssociatedDevice;
//     PSAVED_LINK_INFORMATION SavedLinkInformation = NULL;
//     PDEVICE_INFORMATION DeviceInformation, CurrentDevice;
//
//     /* Acquire device exclusively */
//     KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//
//     /* Look for the leaving device */
//     for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//          NextEntry != &(DeviceExtension->DeviceListHead);
//          NextEntry = NextEntry->Flink)
//     {
//         DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//         if (!RtlCompareUnicodeString(&(DeviceInformation->SymbolicName), DeviceName, TRUE))
//         {
//             break;
//         }
//     }
//
//     /* If we found it */
//     if (NextEntry != &(DeviceExtension->DeviceListHead))
//     {
//         /* If it's asked to keep links, then, prepare to save them */
//         if (DeviceInformation->KeepLinks)
//         {
//             SavedLinkInformation = AllocatePool(sizeof(SAVED_LINK_INFORMATION));
//             if (!SavedLinkInformation)
//             {
//                 DeviceInformation->KeepLinks = FALSE;
//             }
//         }
//
//         /* If it's possible (and asked), start to save them */
//         if (DeviceInformation->KeepLinks)
//         {
//             InsertTailList(&(DeviceExtension->SavedLinksListHead), &(SavedLinkInformation->SavedLinksListEntry));
//             InitializeListHead(&(SavedLinkInformation->SymbolicLinksListHead));
//             SavedLinkInformation->UniqueId = DeviceInformation->UniqueId;
//         }
//
//         /* For all the symlinks */
//         while (!IsListEmpty(&(DeviceInformation->SymbolicLinksListHead)))
//         {
//             NextEntry = RemoveHeadList(&(DeviceInformation->SymbolicLinksListHead));
//             SymlinkInformation = CONTAINING_RECORD(NextEntry, SYMLINK_INFORMATION, SymbolicLinksListEntry);
//
//             /* If we have to, save the link */
//             if (DeviceInformation->KeepLinks)
//             {
//                 InsertTailList(&(SavedLinkInformation->SymbolicLinksListHead), &(SymlinkInformation->SymbolicLinksListEntry));
//             }
//             /* Otherwise, just release it */
//             else
//             {
//                 GlobalDeleteSymbolicLink(&(SymlinkInformation->Name));
//                 FreePool(SymlinkInformation->Name.Buffer);
//                 FreePool(SymlinkInformation);
//             }
//         }
//
//         /* Free all the replicated unique IDs */
//         while (!IsListEmpty(&(DeviceInformation->ReplicatedUniqueIdsListHead)))
//         {
//             NextEntry = RemoveHeadList(&(DeviceInformation->ReplicatedUniqueIdsListHead));
//             UniqueIdReplicate = CONTAINING_RECORD(NextEntry, UNIQUE_ID_REPLICATE, ReplicatedUniqueIdsListEntry);
//
//
//             FreePool(UniqueIdReplicate->UniqueId);
//             FreePool(UniqueIdReplicate);
//         }
//
//         while (!IsListEmpty(&(DeviceInformation->AssociatedDevicesHead)))
//         {
//             NextEntry = RemoveHeadList(&(DeviceInformation->AssociatedDevicesHead));
//             AssociatedDevice = CONTAINING_RECORD(NextEntry, ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//
//             DeviceInformation->NoDatabase = TRUE;
//             FreePool(AssociatedDevice->String.Buffer);
//             FreePool(AssociatedDevice);
//         }
//
//         /* Remove device from the device list */
//         RemoveEntryList(&(DeviceInformation->DeviceListEntry));
//
//         /* If there are still devices, check if some were associated with ours */
//         if (!IsListEmpty(&(DeviceInformation->DeviceListEntry)))
//         {
//             for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//                  NextEntry != &(DeviceExtension->DeviceListHead);
//                  NextEntry = NextEntry->Flink)
//             {
//                 CurrentDevice = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//                 /* And then, remove them */
//                 DeviceEntry = CurrentDevice->AssociatedDevicesHead.Flink;
//                 while (DeviceEntry != &(CurrentDevice->AssociatedDevicesHead))
//                 {
//                     AssociatedDevice = CONTAINING_RECORD(NextEntry, ASSOCIATED_DEVICE_ENTRY, AssociatedDevicesEntry);
//                     DeviceEntry = DeviceEntry->Flink;
//
//                     if (AssociatedDevice->DeviceInformation != DeviceInformation)
//                     {
//                         continue;
//                     }
//
//                     RemoveEntryList(&(AssociatedDevice->AssociatedDevicesEntry));
//                     FreePool(AssociatedDevice->String.Buffer);
//                     FreePool(AssociatedDevice);
//                 }
//             }
//         }
//
//         /* Finally, clean up device name, symbolic name */
//         FreePool(DeviceInformation->SymbolicName.Buffer);
//         if (!DeviceInformation->KeepLinks)
//         {
//             FreePool(DeviceInformation->UniqueId);
//         }
//         FreePool(DeviceInformation->DeviceName.Buffer);
//
//         /* Unregister notifications */
//         if (DeviceInformation->TargetDeviceNotificationEntry)
//         {
//             IoUnregisterPlugPlayNotification(DeviceInformation->TargetDeviceNotificationEntry);
//         }
//
//         /*  And leave */
//         FreePool(DeviceInformation);
//     }
//     else
//     {
//         /* We didn't find device, perhaps because it was offline */
//         for (NextEntry = DeviceExtension->OfflineDeviceListHead.Flink;
//              NextEntry != &(DeviceExtension->OfflineDeviceListHead);
//              NextEntry = NextEntry->Flink)
//         {
//             DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//
//             /* It was, remove it */
//             if (RtlCompareUnicodeString(&(DeviceInformation->SymbolicName), DeviceName, TRUE) == 0)
//             {
//                 RemoveEntryList(&(DeviceInformation->DeviceListEntry));
//                 MountMgrFreeDeadDeviceInfo(DeviceInformation);
//                 break;
//             }
//         }
//     }
//
//     /* Release driver */
//     KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MountMgrMountedDeviceNotification(IN PVOID NotificationStructure,
//                                   IN PVOID Context)
// {
//     BOOLEAN OldState;
//     PDEVICE_EXTENSION DeviceExtension;
//     PDEVICE_INTERFACE_CHANGE_NOTIFICATION Notification;
//
//     /* Notification for a device arrived */
//     /* Disable hard errors */
//     OldState = PsGetThreadHardErrorsAreDisabled(PsGetCurrentThread());
//     PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), TRUE);
//
//     DeviceExtension = Context;
//     Notification = NotificationStructure;
//
//     /* Dispatch according to the event */
//     if (IsEqualGUID(&(Notification->Event), &GUID_DEVICE_INTERFACE_ARRIVAL))
//     {
//         MountMgrMountedDeviceArrival(DeviceExtension, Notification->SymbolicLinkName, FALSE);
//     }
//     else if (IsEqualGUID(&(Notification->Event), &GUID_DEVICE_INTERFACE_REMOVAL))
//     {
//         MountMgrMountedDeviceRemoval(DeviceExtension, Notification->SymbolicLinkName);
//     }
//
//     /* Reset hard errors */
//     PsSetThreadHardErrorsAreDisabled(PsGetCurrentThread(), OldState);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MountMgrCreateClose(IN PDEVICE_OBJECT DeviceObject,
//                     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION Stack;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Allow driver opening for communication
//      * as long as it's not taken for a directory
//      */
//     if (Stack->MajorFunction == IRP_MJ_CREATE &&
//         Stack->Parameters.Create.Options & FILE_DIRECTORY_FILE)
//     {
//         Status = STATUS_NOT_A_DIRECTORY;
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MountMgrCancel(IN PDEVICE_OBJECT DeviceObject,
//                IN PIRP Irp)
// {
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     RemoveEntryList(&(Irp->Tail.Overlay.ListEntry));
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     Irp->IoStatus.Information = 0;
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MountMgrCleanup(IN PDEVICE_OBJECT DeviceObject,
//                 IN PIRP Irp)
// {
//     PIRP ListIrp;
//     KIRQL OldIrql;
//     PLIST_ENTRY NextEntry;
//     PFILE_OBJECT FileObject;
//     PIO_STACK_LOCATION Stack;
//     PDEVICE_EXTENSION DeviceExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     FileObject = Stack->FileObject;
//
//     IoAcquireCancelSpinLock(&OldIrql);
//
//     /* If IRP list if empty, it's OK */
//     if (IsListEmpty(&(DeviceExtension->IrpListHead)))
//     {
//         IoReleaseCancelSpinLock(OldIrql);
//
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//         return STATUS_SUCCESS;
//     }
//
//     /* Otherwise, cancel all the IRPs */
//     NextEntry = DeviceExtension->IrpListHead.Flink;
//     do
//     {
//         ListIrp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//         if (IoGetCurrentIrpStackLocation(ListIrp)->FileObject == FileObject)
//         {
//             ListIrp->Cancel = TRUE;
//             ListIrp->CancelIrql = OldIrql;
//             ListIrp->CancelRoutine = NULL;
//             MountMgrCancel(DeviceObject, ListIrp);
//
//             IoAcquireCancelSpinLock(&OldIrql);
//         }
//
//         NextEntry = NextEntry->Flink;
//     }
//     while (NextEntry != &(DeviceExtension->IrpListHead));
//
//     IoReleaseCancelSpinLock(OldIrql);
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MountMgrShutdown(IN PDEVICE_OBJECT DeviceObject,
//                  IN PIRP Irp)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     InterlockedExchange(&Unloading, TRUE);
//
//     KeInitializeEvent(&UnloadEvent, NotificationEvent, FALSE);
//
//     /* Wait for workers */
//     if (InterlockedIncrement(&(DeviceExtension->WorkerReferences)) > 0)
//     {
//         KeReleaseSemaphore(&(DeviceExtension->WorkerSemaphore),
//                            IO_NO_INCREMENT,
//                            1,
//                            FALSE);
//         KeWaitForSingleObject(&UnloadEvent, Executive, KernelMode, FALSE, NULL);
//     }
//     else
//     {
//         InterlockedDecrement(&(DeviceExtension->WorkerReferences));
//     }
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
// /* FUNCTIONS ****************************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI

// driver-internal cdecl decls (types_*.rs authority = extern "C").

// ddk_shims-provided cdecl fns.
unsafe extern "system" {
    fn _InterlockedExchange(Target: *mut i32, Value: i32) -> i32;
}
unsafe extern "C" {
    fn IsDriveLetter(name: PUNICODE_STRING) -> u8;
    fn GlobalCreateSymbolicLink(SymbolicLinkName: *mut _UNICODE_STRING, DeviceName: *const _UNICODE_STRING) -> NTSTATUS;
    fn CreateNoDriveLetterEntry(UniqueId: *mut MOUNTDEV_UNIQUE_ID);
    fn GlobalDeleteSymbolicLink(DosName: *mut _UNICODE_STRING) -> i32;
    fn QuerySuggestedLinkName( SymbolicName: PUNICODE_STRING, SuggestedLinkName: PUNICODE_STRING, UseOnlyIfThereAreNoOtherLinks: *mut BOOLEAN, ) -> NTSTATUS;
    fn RemoveSavedLinks( DeviceExtension: PDEVICE_EXTENSION, UniqueId: PMOUNTDEV_UNIQUE_ID, ) -> PSAVED_LINK_INFORMATION;
    fn RedirectSavedLink( SavedLinkInformation: PSAVED_LINK_INFORMATION, SymLinkName: PUNICODE_STRING, TargetDeviceName: PUNICODE_STRING, ) -> BOOLEAN;
    fn SendLinkCreated(Name: PUNICODE_STRING);
    fn SendOnlineNotification(SymbolicName: PUNICODE_STRING);
    fn RegisterForTargetDeviceNotification( DeviceExtension: PDEVICE_EXTENSION, DeviceInformation: PDEVICE_INFORMATION, );
    fn IssueUniqueIdChangeNotify( DeviceExtension: PDEVICE_EXTENSION, SymbolicName: PUNICODE_STRING, NewUniqueId: PMOUNTDEV_UNIQUE_ID, );
    fn ReconcileThisDatabaseWithMaster( DeviceExtension: PDEVICE_EXTENSION, DeviceInformation: PDEVICE_INFORMATION, );
    fn HasNoDriveLetterEntry(UniqueId: PMOUNTDEV_UNIQUE_ID) -> BOOLEAN;
    fn DeleteFromLocalDatabase(SymLinkName: PUNICODE_STRING, UniqueId: PMOUNTDEV_UNIQUE_ID);
    fn QuerySymbolicLinkNamesFromStorage( DeviceExtension: PDEVICE_EXTENSION, DeviceInformation: PDEVICE_INFORMATION, SuggestedLinkName: PUNICODE_STRING, UseOnlyIfThereAreNoOtherLinks: BOOLEAN, SymLinks: *mut PUNICODE_STRING, SymLinkCount: *mut u32, HasGuid: BOOLEAN, StableGuid: *mut GUID, ) -> NTSTATUS;
    fn CreateNewVolumeName(VolumeName: PUNICODE_STRING, Guid: *const GUID) -> NTSTATUS;
}
// DriverEntry(IN PDRIVER_OBJECT DriverObject,
//             IN PUNICODE_STRING RegistryPath)
// {
//     NTSTATUS Status;
//     PDEVICE_OBJECT DeviceObject;
//     PDEVICE_EXTENSION DeviceExtension;
//
//     RtlCreateRegistryKey(RTL_REGISTRY_ABSOLUTE, DatabasePath);
//
//     Status = IoCreateDevice(DriverObject,
//                             sizeof(DEVICE_EXTENSION),
//                             &DeviceMount,
//                             FILE_DEVICE_NETWORK,
//                             FILE_DEVICE_SECURE_OPEN,
//                             FALSE,
//                             &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DriverObject->DriverUnload = MountMgrUnload;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     RtlZeroMemory(DeviceExtension, sizeof(DEVICE_EXTENSION));
//     DeviceExtension->DeviceObject = DeviceObject;
//     DeviceExtension->DriverObject = DriverObject;
//
//     InitializeListHead(&(DeviceExtension->DeviceListHead));
//     InitializeListHead(&(DeviceExtension->OfflineDeviceListHead));
//
//     KeInitializeSemaphore(&(DeviceExtension->DeviceLock), 1, 1);
//     KeInitializeSemaphore(&(DeviceExtension->RemoteDatabaseLock), 1, 1);
//
//     InitializeListHead(&(DeviceExtension->IrpListHead));
//     DeviceExtension->EpicNumber = 1;
//
//     InitializeListHead(&(DeviceExtension->SavedLinksListHead));
//
//     InitializeListHead(&(DeviceExtension->WorkerQueueListHead));
//     KeInitializeSemaphore(&(DeviceExtension->WorkerSemaphore), 0, MAXLONG);
//     DeviceExtension->WorkerReferences = -1;
//     KeInitializeSpinLock(&(DeviceExtension->WorkerLock));
//
//     InitializeListHead(&(DeviceExtension->UniqueIdWorkerItemListHead));
//     InitializeListHead(&(DeviceExtension->OnlineNotificationListHead));
//     DeviceExtension->OnlineNotificationCount = 1;
//
//     DeviceExtension->RegistryPath.Length = RegistryPath->Length;
//     DeviceExtension->RegistryPath.MaximumLength = RegistryPath->Length + sizeof(WCHAR);
//     DeviceExtension->RegistryPath.Buffer = AllocatePool(DeviceExtension->RegistryPath.MaximumLength);
//     if (!DeviceExtension->RegistryPath.Buffer)
//     {
//         IoDeleteDevice(DeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyUnicodeString(&(DeviceExtension->RegistryPath), RegistryPath);
//
//     DeviceExtension->NoAutoMount = MountmgrReadNoAutoMount(&(DeviceExtension->RegistryPath));
//
//     GlobalCreateSymbolicLink(&DosDevicesMount, &DeviceMount);
//
//     /* Register for device arrival & removal. Ask to be notified for already
//      * present devices
//      */
//     Status = IoRegisterPlugPlayNotification(EventCategoryDeviceInterfaceChange,
//                                             PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
//                                             &MountedDevicesGuid,
//                                             DriverObject,
//                                             MountMgrMountedDeviceNotification,
//                                             DeviceExtension,
//                                             &(DeviceExtension->NotificationEntry));
//
//     if (!NT_SUCCESS(Status))
//     {
//         IoDeleteDevice(DeviceObject);
//         return Status;
//     }
//
//     DriverObject->MajorFunction[IRP_MJ_CREATE]         =
//     DriverObject->MajorFunction[IRP_MJ_CLOSE]          = MountMgrCreateClose;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = MountMgrDeviceControl;
//     DriverObject->MajorFunction[IRP_MJ_CLEANUP]        = MountMgrCleanup;
//     DriverObject->MajorFunction[IRP_MJ_SHUTDOWN]       = MountMgrShutdown;
//
//     gdeviceObject = DeviceObject;
//
//     Status = IoRegisterShutdownNotification(DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         IoDeleteDevice(DeviceObject);
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    ANSI_STRING, DISK_GEOMETRY, DISK_GEOMETRY_EX, DRIVE_LAYOUT_INFORMATION, GUID,
    _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest,
    IoCreateSymbolicLink, IoDeleteSymbolicLink, IoGetAttachedDeviceReference, IoGetConfigurationInformation,
    IoGetDeviceObjectPointer, IoGetNextIrpStackLocation, KEVENT, KeBugCheck,
    KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, MOUNTDEV_NAME,
    MOUNTMGR_CREATE_POINT_INPUT, MOUNTMGR_DRIVE_LETTER_INFORMATION,
    _MOUNTMGR_DRIVE_LETTER_TARGET, MOUNTMGR_MOUNT_POINT, _OBJECT_ATTRIBUTES,
    PARTITION_INFORMATION, PARTITION_INFORMATION_EX, PBOOT_SECTOR_INFO,
    PCONFIGURATION_INFORMATION, PDEVICE_OBJECT, PDISK_GEOMETRY, PDISK_GEOMETRY_EX,
    PDISK_LAYOUT, PDRIVE_LAYOUT_INFORMATION, PFILE_OBJECT, PIO_STACK_LOCATION,
    PIO_STATUS_BLOCK, PIRP, PKEVENT, PLARGE_INTEGER, PLOADER_PARAMETER_BLOCK,
    PLONGLONG, PMOUNTMGR_CREATE_POINT_INPUT, PMOUNTMGR_DRIVE_LETTER_TARGET,
    PMOUNTMGR_MOUNT_POINT, PMOUNTMGR_MOUNT_POINTS, PPARTITION_DESCRIPTOR,
    PPARTITION_INFORMATION, PPARTITION_TABLE, PPTE, PSTR, PSTRING, PUINT16,
    PUINT32, PUNICODE_STRING, _UNICODE_STRING, RtlCompareMemory,
    RtlEqualUnicodeString, RtlFreeUnicodeString, RtlInitAnsiString,
    RtlInitString, RtlInitUnicodeString, RtlPrefixUnicodeString,
    STORAGE_DEVICE_NUMBER, ZwClose, ZwOpenFile,
    _EVENT_TYPE_NotificationEvent, _KWAIT_REASON_Executive, _MODE_KernelMode,
};
use core::ffi::c_void;

// MOUNTMGR_DRIVE_LETTER_TARGET: bindgen emits the type alias under the
// underscore-prefixed name; keep the friendly alias so the function bodies
// keep compiling.
pub type MOUNTMGR_DRIVE_LETTER_TARGET = _MOUNTMGR_DRIVE_LETTER_TARGET;

// PARTITION_TYPE — local enum from disksup.c (not in bindgen).
#[allow(non_camel_case_types, dead_code)]
pub type PARTITION_TYPE = i32;
pub const BootablePartition: PARTITION_TYPE = 0;
pub const PrimaryPartition: PARTITION_TYPE = 1;
pub const LogicalPartition: PARTITION_TYPE = 2;
pub const FtPartition: PARTITION_TYPE = 3;
pub const UnknownPartition: PARTITION_TYPE = 4;
pub const DataPartition: PARTITION_TYPE = 5;

// PARTITION_BASIC_DATA_GUID — defined in diskguid.h.
pub static PARTITION_BASIC_DATA_GUID: GUID = GUID {
    Data1: 0xEBD0A0A2,
    Data2: 0xB9E5,
    Data3: 0x4433,
    Data4: [0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7],
};

// PARTITION_STYLE values (from winioctl.h).
pub const PARTITION_STYLE_MBR: u32 = 0;
pub const PARTITION_STYLE_GPT: u32 = 1;
pub const PARTITION_STYLE_RAW: u32 = 2;

extern "C" {
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn strcpy(dst: *mut i8, src: *const i8) -> *mut i8;
    fn strlen(s: *const i8) -> usize;
    fn strrchr(s: *const i8, c: i32) -> *mut i8;
    fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8;
    fn _strupr(s: *mut i8) -> *mut i8;
    static mut IoRemoteBootClient: BOOLEAN;
}

// C macro inlines (originally from ndk/iotypes.h, halfuncs.h, etc).
// These are #defines in C, not real functions — must be Rust fns.
#[inline]
unsafe fn BooleanFlagOn(flags: u32, single: u32) -> u8 {
    if (flags & single) != 0 { 1 } else { 0 }
}

#[inline]
unsafe fn IsContainerPartition(partition_type: UCHAR) -> BOOLEAN {
    // PARTITION_EXTENDED=0x05, PARTITION_XINT13_EXTENDED=0x0F
    if partition_type == 0x05 || partition_type == 0x0F { 1 } else { 0 }
}

#[inline]
unsafe fn UInt32x32To64(multiplier: u32, multiplicand: u32) -> u64 {
    (multiplier as u64).wrapping_mul(multiplicand as u64)
}

#[inline]
unsafe fn GET_STARTING_SECTOR(p: PPARTITION_DESCRIPTOR) -> ULONG {
    let p = &*p;
    (p.StartingSectorLsb0 as ULONG)
        | ((p.StartingSectorLsb1 as ULONG) << 8)
        | ((p.StartingSectorMsb0 as ULONG) << 16)
        | ((p.StartingSectorMsb1 as ULONG) << 24)
}

#[inline]
unsafe fn GET_PARTITION_LENGTH(p: PPARTITION_DESCRIPTOR) -> u64 {
    let p = &*p;
    let lo = (p.PartitionLengthLsb0 as ULONG)
        | ((p.PartitionLengthLsb1 as ULONG) << 8)
        | ((p.PartitionLengthMsb0 as ULONG) << 16)
        | ((p.PartitionLengthMsb1 as ULONG) << 24);
    lo as u64
}

#[inline]
unsafe fn SET_PARTITION_LENGTH(p: PPARTITION_DESCRIPTOR, length: u32) {
    let p = &mut *p;
    p.PartitionLengthLsb0 = (length & 0xFF) as UCHAR;
    p.PartitionLengthLsb1 = ((length >> 8) & 0xFF) as UCHAR;
    p.PartitionLengthMsb0 = ((length >> 16) & 0xFF) as UCHAR;
    p.PartitionLengthMsb1 = ((length >> 24) & 0xFF) as UCHAR;
}

extern "system" {
    fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn RtlAnsiStringToUnicodeString(
        DestinationString: *mut _UNICODE_STRING,
        SourceString: *mut ANSI_STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
}

// IsRecognizedPartition / IsFTPartition: macros in C, replicate in Rust.
#[inline(always)]
unsafe fn IsFTPartition(p: UCHAR) -> BOOLEAN {
    const PARTITION_NTFT: UCHAR = 0x80;
    const VALID_NTFT: UCHAR = 0xC0;
    let base_type = p & !VALID_NTFT;

    ((p & PARTITION_NTFT) != 0
        && (base_type == 0x06 || base_type == 0x07 || base_type == 0x0B || base_type == 0x0C)) as BOOLEAN
}
#[inline(always)]
unsafe fn IsRecognizedPartition(p: UCHAR) -> BOOLEAN {
    (IsFTPartition(p) != 0
        || p == 0x01 || p == 0x04 || p == 0x06 || p == 0x07
        || p == 0x0B || p == 0x0C || p == 0x0E
        || p == 0x83 || p == 0x43 || p == 0x96
        || p == 0xA5 || p == 0xA6 || p == 0xA9) as BOOLEAN
}

// HALDISPATCH is exported by HAL; declare here for the FASTCALL wrappers.
extern "C" {
    static mut HalDispatchTable: HAL_DISPATCH;
}
#[allow(non_snake_case)]
unsafe fn HALDISPATCH() -> *mut HAL_DISPATCH {
    core::ptr::addr_of_mut!(HalDispatchTable)
}

// HAL_DISPATCH lives in the bindgen DDK module under the un-prefixed name.
use ntkrnl_sys::ddk::HAL_DISPATCH as _HAL_DISPATCH_BindgenAlias;
type HAL_DISPATCH = _HAL_DISPATCH_BindgenAlias;

const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_REMOVABLE_MEDIA: u32 = 0x00000001;
const STATUS_NO_MEDIA: i32 = 0xC0000178u32 as i32;
const STATUS_PENDING: i32 = 0x00000103u32 as i32;
const STATUS_BUFFER_TOO_SMALL: i32 = 0xC0000023u32 as i32;
const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32;
const STATUS_DISK_CORRUPT_ERROR: i32 = 0xC0000032u32 as i32;
const STATUS_TOO_LATE: i32 = 0xC0000189u32 as i32;
const STATUS_SOME_NOT_MAPPED: i32 = 0x00000107u32 as i32;
const IOCTL_DISK_GET_DRIVE_LAYOUT: u32 = 0x0007400C;
const IOCTL_DISK_GET_PARTITION_INFO_EX: u32 = 0x00070048;
const IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: u32 = 0x000700A0;
const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000;
const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x002D1080;
const IOCTL_MOUNTMGR_CREATE_POINT: u32 = 0x006DC000;
const IOCTL_MOUNTMGR_DELETE_POINTS: u32 = 0x006DC004;
const IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER: u32 = 0x006DC010;
const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: u32 = 0x004D0008;
const TAG_FSTUB: u32 = u32::from_be_bytes(*b"BtsF");
const TAG_FILE_SYSTEM: u32 = u32::from_be_bytes(*b"SYSF");
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x00000020;
const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02;
const PARTITION_TABLE_OFFSET: usize = 0x1BE;
const BOOT_SIGNATURE_OFFSET: usize = 0x200 - 2;
const BOOT_RECORD_SIGNATURE: u16 = 0xAA55;
const PARTITION_DM: UCHAR = 0x54;
const PARTITION_EZDRIVE: UCHAR = 0x55;
const ASSIGN_DRIVE_LETTERS_FAILED: ULONG = 0x72;
#[allow(non_upper_case_globals)]
const NotificationEvent: i32 = 0;
#[allow(non_upper_case_globals)]
const Executive: i32 = 0;
// const KernelMode: i8 = 0; // body uses literal 0i32 directly

const fn wide<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}

static W_ARC_RDISK_FMT: [u16; 34] = wide(b"\\ArcName\\multi(0)disk(0)rdisk(%d)\0");
static W_MOUNTMGR: [u16; 26] = wide(b"\\Device\\MountPointManager\0");
static W_DOS_DEVICE_DRIVE_FMT: [u16; 16] = wide(b"\\DosDevices\\%c:\0");
static W_DEVICE_FLOPPY: [u16; 15] = wide(b"\\Device\\Floppy\0");
static W_DEVICE_CDROM: [u16; 14] = wide(b"\\Device\\CdRom\0");
static W_HARDDISK_PARTITION0_FMT: [u16; 30] = wide(b"\\Device\\Harddisk%d\\Partition0\0");
static W_HARDDISK_PARTITION_FMT: [u16; 31] = wide(b"\\Device\\Harddisk%d\\Partition%d\0");
static W_DEVICE_FLOPPY_FMT: [u16; 17] = wide(b"\\Device\\Floppy%d\0");
static W_DEVICE_CDROM_FMT: [u16; 16] = wide(b"\\Device\\CdRom%d\0");

#[no_mangle]
pub unsafe extern "C" fn HalpQueryDriveLayout(
    DeviceName: PUNICODE_STRING,
    LayoutInfo: *mut PDRIVE_LAYOUT_INFORMATION,
) -> i32 {
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut Status: i32;
    let mut BufferSize: u32;
    let mut Buffer: PDRIVE_LAYOUT_INFORMATION;

    Status = IoGetDeviceObjectPointer(
        DeviceName,
        FILE_READ_ATTRIBUTES,
        core::ptr::addr_of_mut!(FileObject),
        core::ptr::addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    if BooleanFlagOn((*DeviceObject).Characteristics, FILE_REMOVABLE_MEDIA) != 0 {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        return STATUS_NO_MEDIA;
    }

    Buffer = core::ptr::null_mut();
    BufferSize = 0x1000;
    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
    loop {
        if !Buffer.is_null() {
            ExFreePoolWithTag(Buffer as *mut core::ffi::c_void, TAG_FSTUB);
            BufferSize *= 2;
        }

        Buffer = ExAllocatePoolWithTag(NonPagedPool, BufferSize as usize, TAG_FSTUB) as PDRIVE_LAYOUT_INFORMATION;
        if Buffer.is_null() {
            Status = STATUS_NO_MEMORY;
            break;
        }

        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_DISK_GET_DRIVE_LAYOUT,
            DeviceObject,
            core::ptr::null_mut(),
            0,
            Buffer as *mut core::ffi::c_void,
            BufferSize,
            0u8,
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(StatusBlock),
        );
        if Irp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
            Status = StatusBlock.__bindgen_anon_1.Status;
        }

        if Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);

    if NT_SUCCESS(Status) {
        *LayoutInfo = Buffer;
    } else {
        if !Buffer.is_null() {
            ExFreePoolWithTag(Buffer as *mut core::ffi::c_void, TAG_FSTUB);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalpQueryPartitionType(
    DeviceName: PUNICODE_STRING,
    LayoutInfo: PDRIVE_LAYOUT_INFORMATION,
    PartitionType: *mut PARTITION_TYPE,
) -> NTSTATUS {
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PartitionInfo: PARTITION_INFORMATION_EX = core::mem::zeroed();

    Status = IoGetDeviceObjectPointer(
        DeviceName,
        FILE_READ_ATTRIBUTES,
        &mut FileObject,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    if BooleanFlagOn((*DeviceObject).Characteristics, FILE_REMOVABLE_MEDIA) != 0 {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        *PartitionType = LogicalPartition;
        return STATUS_SUCCESS;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_GET_PARTITION_INFO_EX,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        &mut PartitionInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<PARTITION_INFORMATION_EX>() as u32,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(&mut Event as *mut _ as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);

    if !NT_SUCCESS(Status) {
        if LayoutInfo.is_null() {
            *PartitionType = LogicalPartition;
            return STATUS_SUCCESS;
        }

        return Status;
    }

    if PartitionInfo.PartitionStyle != PARTITION_STYLE_MBR {
        if PartitionInfo.PartitionStyle != PARTITION_STYLE_GPT {
            *PartitionType = UnknownPartition;
            return STATUS_SUCCESS;
        }

        if RtlCompareMemory(
            &PartitionInfo.__bindgen_anon_1.Gpt.PartitionType as *const _ as *const core::ffi::c_void,
            &PARTITION_BASIC_DATA_GUID as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<GUID>() as _,
        ) == core::mem::size_of::<GUID>() as _
        {
            *PartitionType = DataPartition;
            return STATUS_SUCCESS;
        }

        *PartitionType = UnknownPartition;
        return STATUS_SUCCESS;
    }

    if IsRecognizedPartition(PartitionInfo.__bindgen_anon_1.Mbr.PartitionType) == 0 {
        *PartitionType = UnknownPartition;
        return STATUS_SUCCESS;
    }

    if IsFTPartition(PartitionInfo.__bindgen_anon_1.Mbr.PartitionType) != 0 {
        *PartitionType = FtPartition;
        return STATUS_SUCCESS;
    }

    if LayoutInfo.is_null() {
        *PartitionType = LogicalPartition;
        return STATUS_SUCCESS;
    }

    for i in 0..4u16 {
        // NB: PartitionEntry is a bindgen flexible array ([T;1]) — indexing [i>=1]
        // bounds-panics even on a large-enough heap buffer; use raw ptr arithmetic.
        let entry_i = unsafe {
            &*core::ptr::addr_of!((*LayoutInfo).PartitionEntry)
                .cast::<PARTITION_INFORMATION>()
                .add(i as usize)
        };
        if entry_i.StartingOffset.QuadPart == PartitionInfo.StartingOffset.QuadPart {
            if PartitionInfo.__bindgen_anon_1.Mbr.BootIndicator != 0 {
                *PartitionType = BootablePartition;
            } else {
                *PartitionType = PrimaryPartition;
            }

            return STATUS_SUCCESS;
        }
    }

    *PartitionType = LogicalPartition;
    STATUS_SUCCESS
}

unsafe fn IopComputeHarddiskDerangements(DiskCount: ULONG) -> PULONG {
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: ULONG;
    let mut Devices: PULONG;
    let mut Status: NTSTATUS;
    let mut Buffer: [WCHAR; 100] = [0; 100];
    let mut ArcName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceNumber: STORAGE_DEVICE_NUMBER = core::mem::zeroed();

    if DiskCount == 0 {
        return core::ptr::null_mut();
    }

    Devices = ExAllocatePoolWithTag(
        PagedPool | POOL_COLD_ALLOCATION,
        (core::mem::size_of::<ULONG>() * DiskCount as usize) as SIZE_T,
        TAG_FSTUB,
    ) as PULONG;
    if Devices.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    while i < DiskCount {
        crate::swprintf!(

            Buffer.as_mut_ptr(),

            Buffer.len(),

            &W_ARC_RDISK_FMT,

            i,

        );
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ArcName), Buffer.as_ptr());
        if NT_SUCCESS(IoGetDeviceObjectPointer(
            core::ptr::addr_of_mut!(ArcName),
            FILE_READ_ATTRIBUTES,
            core::ptr::addr_of_mut!(FileObject),
            core::ptr::addr_of_mut!(DeviceObject),
        )) {
            DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);

            KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
            Irp = IoBuildDeviceIoControlRequest(
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                DeviceObject,
                core::ptr::null_mut(),
                0,
                core::ptr::addr_of_mut!(DeviceNumber) as PVOID,
                core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as ULONG,
                0u8,
                core::ptr::addr_of_mut!(Event),
                core::ptr::addr_of_mut!(IoStatusBlock),
            );
            if !Irp.is_null() {
                Status = IoCallDriver(DeviceObject, Irp);
                if Status == STATUS_PENDING {
                    KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
                    Status = IoStatusBlock.__bindgen_anon_1.Status;
                }

                ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);

                if NT_SUCCESS(Status) {
                    *Devices.add(i as usize) = DeviceNumber.DeviceNumber;
                    i += 1;
                    continue;
                }
            } else {
                ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
            }

            *Devices.add(i as usize) = 0xFFFFFFFF;
        }
        i += 1;
    }

    i = 0;
    while i < DiskCount {
        j = 0;
        while j < DiskCount {
            if *Devices.add(j as usize) == i {
                break;
            }
            j += 1;
        }

        if j >= DiskCount {
            k = 0;
            while *Devices.add(k as usize) != 0xFFFFFFFF {
                k += 1;
                if k >= DiskCount {
                    break;
                }
            }

            if k < DiskCount {
                *Devices.add(k as usize) = i;
            }
        }
        i += 1;
    }

    Devices
}

unsafe fn HalpNextMountLetter(
    DeviceName: PUNICODE_STRING,
    DriveLetter: *mut u8,
) -> NTSTATUS {
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut MountMgr: _UNICODE_STRING = core::mem::zeroed();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Target: PMOUNTMGR_DRIVE_LETTER_TARGET;
    let mut LetterInfo: MOUNTMGR_DRIVE_LETTER_INFORMATION = core::mem::zeroed();

    RtlInitUnicodeString(
        &mut MountMgr,
        W_MOUNTMGR.as_ptr(),
    );
    Status = IoGetDeviceObjectPointer(
        &mut MountMgr,
        FILE_READ_ATTRIBUTES,
        &mut FileObject,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Target = ExAllocatePoolWithTag(
        PagedPool,
        (*DeviceName).Length as usize
            + core::mem::offset_of!(_MOUNTMGR_DRIVE_LETTER_TARGET, DeviceName),
        TAG_FSTUB,
    ) as PMOUNTMGR_DRIVE_LETTER_TARGET;
    if Target.is_null() {
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Target).DeviceNameLength = (*DeviceName).Length;
    core::ptr::copy_nonoverlapping(
        (*DeviceName).Buffer,
        (*Target).DeviceName.as_mut_ptr(),
        (*DeviceName).Length as usize / core::mem::size_of::<u16>(),
    );

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER,
        DeviceObject,
        Target as *mut core::ffi::c_void,
        ((*DeviceName).Length as usize
            + core::mem::offset_of!(_MOUNTMGR_DRIVE_LETTER_TARGET, DeviceName)) as u32,
        &mut LetterInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<MOUNTMGR_DRIVE_LETTER_INFORMATION>() as u32,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        ExFreePoolWithTag(Target as *mut core::ffi::c_void, TAG_FSTUB);
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(&mut Event as *mut _ as *mut core::ffi::c_void, _KWAIT_REASON_Executive, _MODE_KernelMode as i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ExFreePoolWithTag(Target as *mut core::ffi::c_void, TAG_FSTUB);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    *DriveLetter = LetterInfo.CurrentDriveLetter;

    Status
}

unsafe fn HalpSetMountLetter(DeviceName: PUNICODE_STRING, DriveLetter: u8) -> NTSTATUS {
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Buffer: [u16; 30] = [0; 30];
    let InputBufferLength: u32;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DosDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut MountMgr: _UNICODE_STRING = core::mem::zeroed();
    let InputBuffer: PMOUNTMGR_CREATE_POINT_INPUT;

    crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_DOS_DEVICE_DRIVE_FMT, DriveLetter as u32);
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DosDevice), Buffer.as_ptr());

    InputBufferLength = DosDevice.Length as u32 + (*DeviceName).Length as u32 + core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>() as u32;
    InputBuffer = ExAllocatePoolWithTag(PagedPool, InputBufferLength as usize, TAG_FSTUB) as PMOUNTMGR_CREATE_POINT_INPUT;
    if InputBuffer == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*InputBuffer).SymbolicLinkNameOffset = core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>() as u16;
    (*InputBuffer).SymbolicLinkNameLength = DosDevice.Length;
    (*InputBuffer).DeviceNameOffset = DosDevice.Length + core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>() as u16;
    (*InputBuffer).DeviceNameLength = (*DeviceName).Length;
    RtlCopyMemory(InputBuffer.offset(1) as *mut core::ffi::c_void, DosDevice.Buffer as *const core::ffi::c_void, DosDevice.Length as usize);
    RtlCopyMemory((InputBuffer as usize + (*InputBuffer).DeviceNameOffset as usize) as *mut core::ffi::c_void,
                  (*DeviceName).Buffer as *const core::ffi::c_void,
                  (*DeviceName).Length as usize);

    RtlInitUnicodeString(core::ptr::addr_of_mut!(MountMgr), W_MOUNTMGR.as_ptr());
    Status = IoGetDeviceObjectPointer(core::ptr::addr_of_mut!(MountMgr),
                                      FILE_READ_ATTRIBUTES,
                                      core::ptr::addr_of_mut!(FileObject),
                                      core::ptr::addr_of_mut!(DeviceObject));
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(InputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
        return Status;
    }

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
    Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTMGR_CREATE_POINT,
                                        DeviceObject,
                                        InputBuffer as *mut core::ffi::c_void,
                                        InputBufferLength,
                                        core::ptr::null_mut(),
                                        0,
                                        0u8,
                                        core::ptr::addr_of_mut!(Event),
                                        core::ptr::addr_of_mut!(IoStatusBlock));
    if Irp == core::ptr::null_mut() {
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        ExFreePoolWithTag(InputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
    ExFreePoolWithTag(InputBuffer as *mut core::ffi::c_void, TAG_FSTUB);

    ntkrnl_sys::driver_ddk::DbgPrint(
        c"[FSTUB] SetMountLetter %c: -> %wZ = 0x%x\n".as_ptr(),
        DriveLetter as u32,
        DeviceName,
        Status as u32,
    );
    return Status;
}

unsafe fn HalpNextDriveLetter(
    DeviceName: PUNICODE_STRING,
    NtDeviceName: PSTRING,
    NtSystemPath: *mut u8,
    IsRemovable: u8,
) -> u8 {
    let mut i: u8;
    let mut Buffer: [u16; 40] = [0; 40];
    let mut DriveLetter: u8 = 0;
    let mut FloppyString: _UNICODE_STRING = core::mem::zeroed();
    let mut CdString: _UNICODE_STRING = core::mem::zeroed();
    let mut NtDeviceNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut DosDevice: _UNICODE_STRING = core::mem::zeroed();

    if NT_SUCCESS(HalpNextMountLetter(DeviceName, core::ptr::addr_of_mut!(DriveLetter))) {
        return DriveLetter;
    }

    if NtDeviceName.is_null() || NtSystemPath.is_null() {
        return 0xFF;
    }

    if IsRemovable == 0 {
        return 0;
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(FloppyString), W_DEVICE_FLOPPY.as_ptr());
    RtlInitUnicodeString(core::ptr::addr_of_mut!(CdString), W_DEVICE_CDROM.as_ptr());

    if RtlPrefixUnicodeString(core::ptr::addr_of_mut!(FloppyString), DeviceName, 1) != 0 {
        DriveLetter = b'A';
    } else if RtlPrefixUnicodeString(core::ptr::addr_of_mut!(CdString), DeviceName, 1) != 0 {
        DriveLetter = b'D';
    } else {
        DriveLetter = b'C';
    }

    i = DriveLetter;
    while i <= b'Z' {
        if NT_SUCCESS(HalpSetMountLetter(DeviceName, i)) {
            if NT_SUCCESS(RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(NtDeviceNameU), NtDeviceName, 1)) {
                if RtlEqualUnicodeString(core::ptr::addr_of_mut!(NtDeviceNameU), DeviceName, 1) != 0 {
                    *NtSystemPath = i;
                }

                RtlFreeUnicodeString(core::ptr::addr_of_mut!(NtDeviceNameU));
            }

            return i;
        }
        i += 1;
    }

    i = DriveLetter;
    while i <= b'Z' {
        crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_DOS_DEVICE_DRIVE_FMT, i as u32);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DosDevice), Buffer.as_ptr());

        if NT_SUCCESS(IoCreateSymbolicLink(core::ptr::addr_of_mut!(DosDevice), DeviceName)) {
            if NT_SUCCESS(RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(NtDeviceNameU), NtDeviceName, 1)) {
                if RtlEqualUnicodeString(core::ptr::addr_of_mut!(NtDeviceNameU), DeviceName, 1) != 0 {
                    *NtSystemPath = i;
                }

                RtlFreeUnicodeString(core::ptr::addr_of_mut!(NtDeviceNameU));
            }

            return i;
        }
        i += 1;
    }

    return 0;
}

unsafe fn HalpIsOldStyleFloppy(DeviceName: PUNICODE_STRING) -> BOOLEAN {
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut DevName: MOUNTDEV_NAME = core::mem::zeroed();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if !NT_SUCCESS(IoGetDeviceObjectPointer(
        DeviceName,
        FILE_READ_ATTRIBUTES,
        &mut FileObject,
        &mut DeviceObject,
    )) {
        return 0u8;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        &mut DevName as *mut MOUNTDEV_NAME as *mut core::ffi::c_void,
        core::mem::size_of::<MOUNTDEV_NAME>() as u32,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp == core::ptr::null_mut() {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        return 0u8;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(&mut Event as *mut KEVENT as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    if Status != STATUS_BUFFER_OVERFLOW { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDeleteMountLetter(DriveLetter: u8) -> NTSTATUS {
    ntkrnl_sys::driver_ddk::DbgPrint(c"[FSTUB] DeleteMountLetter %c:\n".as_ptr(), DriveLetter as u32);
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Buffer: [u16; 30] = [0; 30];
    let InputBufferLength: u32;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let InputBuffer: PMOUNTMGR_MOUNT_POINT;
    let mut DosDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut MountMgr: _UNICODE_STRING = core::mem::zeroed();
    let OutputBuffer: PMOUNTMGR_MOUNT_POINTS;

    crate::swprintf!(


        Buffer.as_mut_ptr(),


        Buffer.len(),


        &W_DOS_DEVICE_DRIVE_FMT,


        DriveLetter as u32,


    );
    RtlInitUnicodeString(&mut DosDevice, Buffer.as_ptr());

    InputBufferLength = DosDevice.Length as u32 + core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as u32;
    InputBuffer = ExAllocatePoolWithTag(PagedPool, InputBufferLength as usize, TAG_FSTUB) as PMOUNTMGR_MOUNT_POINT;
    if InputBuffer == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(InputBuffer as *mut core::ffi::c_void, InputBufferLength as usize);
    (*InputBuffer).SymbolicLinkNameOffset = core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as u32;
    (*InputBuffer).SymbolicLinkNameLength = DosDevice.Length;
    RtlCopyMemory(
        InputBuffer.offset(1) as *mut core::ffi::c_void,
        DosDevice.Buffer as *const core::ffi::c_void,
        DosDevice.Length as usize,
    );

    OutputBuffer = ExAllocatePoolWithTag(PagedPool, 0x1000, TAG_FSTUB) as PMOUNTMGR_MOUNT_POINTS;
    if OutputBuffer == core::ptr::null_mut() {
        ExFreePoolWithTag(InputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlInitUnicodeString(&mut MountMgr, W_MOUNTMGR.as_ptr());
    Status = IoGetDeviceObjectPointer(
        &mut MountMgr,
        FILE_READ_ATTRIBUTES,
        &mut FileObject,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(OutputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
        ExFreePoolWithTag(InputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
        return Status;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_MOUNTMGR_DELETE_POINTS,
        DeviceObject,
        InputBuffer as *mut core::ffi::c_void,
        InputBufferLength,
        OutputBuffer as *mut core::ffi::c_void,
        0x1000,
        0u8,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp == core::ptr::null_mut() {
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        ExFreePoolWithTag(OutputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
        ExFreePoolWithTag(InputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(&mut Event as *mut KEVENT as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
    ExFreePoolWithTag(OutputBuffer as *mut core::ffi::c_void, TAG_FSTUB);
    ExFreePoolWithTag(InputBuffer as *mut core::ffi::c_void, TAG_FSTUB);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalpEnableAutomaticDriveLetterAssignment() {
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut MountMgr: _UNICODE_STRING = core::mem::zeroed();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(MountMgr),
        W_MOUNTMGR.as_ptr(),
    );
    Status = IoGetDeviceObjectPointer(
        core::ptr::addr_of_mut!(MountMgr),
        0x80u32,
        core::ptr::addr_of_mut!(FileObject),
        core::ptr::addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS(Status) {
        ntkrnl_sys::driver_ddk::DbgPrint(
            c"[FSTUB] AUTO_DL: open MountPointManager FAILED 0x%x\n".as_ptr(),
            Status as u32,
        );
        return;
    }

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0u32, 0u8);
    Irp = IoBuildDeviceIoControlRequest(
        // IOCTL_MOUNTMGR_AUTO_DL_ASSIGNMENTS = CTL_CODE('m',5,BUFFERED,RW) — the
        // previous literal 0x4d0010 was a MOUNTDEV ('M') code; mountmgr ignored it
        // and AutomaticDriveLetter stayed FALSE forever (no drive letters, ever).
        0x6DC014u32,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0u8,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );
    if Irp == core::ptr::null_mut() {
        return;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == 0x00000103u32 as i32 {
        KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void, 0u32, 0i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }
    ntkrnl_sys::driver_ddk::DbgPrint(
        c"[FSTUB] AUTO_DL sent to mountmgr -> 0x%x\n".as_ptr(),
        Status as u32,
    );

    ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    return;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn xHalIoAssignDriveLetters(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    NtDeviceName: PSTRING,
    NtSystemPath: *mut u8,
    NtSystemPathString: PSTRING,
) {
    xHalIoAssignDriveLetters_impl(LoaderBlock, NtDeviceName, NtSystemPath, NtSystemPathString)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn xHalIoAssignDriveLetters(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    NtDeviceName: PSTRING,
    NtSystemPath: *mut u8,
    NtSystemPathString: PSTRING,
) {
    xHalIoAssignDriveLetters_impl(LoaderBlock, NtDeviceName, NtSystemPath, NtSystemPathString)
}

unsafe fn xHalIoAssignDriveLetters_impl(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    NtDeviceName: PSTRING,
    NtSystemPath: *mut u8,
    NtSystemPathString: PSTRING,
) {
    let mut i: u16;
    let mut Devices: PULONG;
    let mut Status: NTSTATUS;
    let mut Buffer: [u16; 50] = [0; 50];
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut DriveLetter: u8 = 0;
    let mut SystemFound: BOOLEAN;
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PartitionType: PARTITION_TYPE = UnknownPartition;
    let mut StringA1: ANSI_STRING = core::mem::zeroed();
    let mut StringA2: ANSI_STRING = core::mem::zeroed();
    let mut Buffer1: PSTR;
    let mut Buffer2: PSTR;
    let mut LoadOptions: PSTR;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LayoutInfo: PDRIVE_LAYOUT_INFORMATION = core::ptr::null_mut();
    let mut ConfigInfo: PCONFIGURATION_INFORMATION;
    let mut StringU1: _UNICODE_STRING = core::mem::zeroed();
    let mut StringU2: _UNICODE_STRING = core::mem::zeroed();
    let mut StringU3: _UNICODE_STRING = core::mem::zeroed();
    let mut Increment: u32;
    let mut DiskCount: u32;
    let mut RealDiskCount: u32;
    let mut HarddiskCount: u32;
    let mut PartitionCount: u32;
    let mut SystemPartition: u32;

    ConfigInfo = IoGetConfigurationInformation();
    DiskCount = (*ConfigInfo).DiskCount;
    RealDiskCount = 0;

    Buffer1 = ExAllocatePoolWithTag(NonPagedPool, 128, TAG_FSTUB) as PSTR;
    Buffer2 = ExAllocatePoolWithTag(NonPagedPool, 64, TAG_FSTUB) as PSTR;
    if Buffer1 == core::ptr::null_mut() || Buffer2 == core::ptr::null_mut() {
        KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
    }

    if IoRemoteBootClient != 0 {
        let mut Last: PSTR;
        let mut Saved: PSTR;

        Last = strrchr((*LoaderBlock).NtBootPathName, '\\' as i32);
        Saved = core::ptr::null_mut();
        if Last == core::ptr::null_mut() {
            KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
        }

        if *Last.offset(1) == 0 {
            *Last = 0;
            Saved = Last;
            Last = strrchr((*LoaderBlock).NtBootPathName, '\\' as i32);
            *Saved = '\\' as i8;
        }

        if Last == core::ptr::null_mut() {
            KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
        }

        *NtSystemPath.offset(0) = 'X' as u8;
        *NtSystemPath.offset(1) = ':' as u8;
        strcpy(NtSystemPath.offset(2) as PSTR, Last);

        if Saved != core::ptr::null_mut() {
            *NtSystemPath.offset(strlen(NtSystemPath as PSTR) as isize - 1) = 0;
        }

        RtlInitString(NtSystemPathString, NtSystemPath as PSTR);
    }

    Increment = 0;
    if DiskCount != 0 {
        i = 0;
        while i < DiskCount as u16 {
            sprintf(Buffer1, b"\\Device\\Harddisk%d\\Partition%d\0".as_ptr() as PSTR, i as u32, 0);
            RtlInitAnsiString(core::ptr::addr_of_mut!(StringA1), Buffer1);
            if !NT_SUCCESS(RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(StringU1), core::ptr::addr_of_mut!(StringA1), 1)) {
                KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
            }

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(StringU1);
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = ZwOpenFile(
                core::ptr::addr_of_mut!(FileHandle),
                SYNCHRONIZE | FILE_READ_DATA,
                core::ptr::addr_of_mut!(ObjectAttributes),
                core::ptr::addr_of_mut!(StatusBlock),
                FILE_SHARE_READ,
                FILE_SYNCHRONOUS_IO_NONALERT,
            );
            if NT_SUCCESS(Status) {
                sprintf(Buffer2, b"\\DosDevices\\PhysicalDrive%d\0".as_ptr() as PSTR, i as u32);
                RtlInitAnsiString(core::ptr::addr_of_mut!(StringA2), Buffer2);
                Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(StringU2), core::ptr::addr_of_mut!(StringA2), 1);
                if NT_SUCCESS(Status) {
                    IoCreateSymbolicLink(core::ptr::addr_of_mut!(StringU2), core::ptr::addr_of_mut!(StringU1));
                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(StringU2));
                }

                ZwClose(FileHandle);

                RealDiskCount = i as u32 + 1;
            }

            RtlFreeUnicodeString(core::ptr::addr_of_mut!(StringU1));

            if !NT_SUCCESS(Status) {
                if Increment < 50 {
                    Increment += 1;
                    DiskCount += 1;
                }
            }

            i += 1;
        }
    }

    ExFreePoolWithTag(Buffer1 as *mut core::ffi::c_void, TAG_FSTUB);
    ExFreePoolWithTag(Buffer2 as *mut core::ffi::c_void, TAG_FSTUB);

    if (*LoaderBlock).LoadOptions != core::ptr::null_mut() {
        LoadOptions = _strupr((*LoaderBlock).LoadOptions);
    } else {
        LoadOptions = core::ptr::null_mut();
    }

    if LoadOptions != core::ptr::null_mut() &&
        strstr(LoadOptions, b"MININT\0".as_ptr() as PSTR) != core::ptr::null_mut() &&
        NT_SUCCESS(RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(StringU1), NtDeviceName, 1)) {
        if NT_SUCCESS(HalpSetMountLetter(core::ptr::addr_of_mut!(StringU1), 'X' as u8)) {
            *NtSystemPath = 'X' as u8;
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(StringU1));
    }

    DiskCount -= Increment;
    if RealDiskCount > DiskCount {
        DiskCount = RealDiskCount;
    }
    Devices = IopComputeHarddiskDerangements(DiskCount);

    HarddiskCount = 0;
    i = 0;
    while i < DiskCount as u16 {
        if Devices != core::ptr::null_mut() {
            HarddiskCount = *Devices.offset(i as isize);
        }

        crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION0_FMT, HarddiskCount);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
        if !NT_SUCCESS(HalpQueryDriveLayout(core::ptr::addr_of_mut!(StringU1), core::ptr::addr_of_mut!(LayoutInfo))) {
            LayoutInfo = core::ptr::null_mut();
        }

        SystemFound = 0;
        crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION_FMT, HarddiskCount, 1);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
        if !NT_SUCCESS(HalpQueryPartitionType(core::ptr::addr_of_mut!(StringU1), LayoutInfo, core::ptr::addr_of_mut!(PartitionType))) {
            PartitionCount = 1;
            loop {
                crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION_FMT, HarddiskCount, PartitionCount);
                RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
                if !NT_SUCCESS(HalpQueryPartitionType(core::ptr::addr_of_mut!(StringU1), LayoutInfo, core::ptr::addr_of_mut!(PartitionType))) {
                    break;
                }

                if PartitionType == PrimaryPartition {
                    HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), NtDeviceName, NtSystemPath, 0);
                    break;
                }

                PartitionCount += 1;
            }
        } else {
            PartitionCount = 2;
            loop {
                if PartitionType == BootablePartition || PartitionType == DataPartition {
                    SystemFound = 1;

                    HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), NtDeviceName, NtSystemPath, 0);
                    if PartitionType == BootablePartition {
                        break;
                    }
                }

                crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION_FMT, HarddiskCount, PartitionCount);
                RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
                if !NT_SUCCESS(HalpQueryPartitionType(core::ptr::addr_of_mut!(StringU1), LayoutInfo, core::ptr::addr_of_mut!(PartitionType))) {
                    if SystemFound == 0 {
                        let mut inner_partition_count = 1;
                        loop {
                            crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION_FMT, HarddiskCount, inner_partition_count);
                            RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
                            if !NT_SUCCESS(HalpQueryPartitionType(core::ptr::addr_of_mut!(StringU1), LayoutInfo, core::ptr::addr_of_mut!(PartitionType))) {
                                break;
                            }

                            if PartitionType == PrimaryPartition {
                                HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), NtDeviceName, NtSystemPath, 0);
                                break;
                            }

                            inner_partition_count += 1;
                        }
                    }

                    break;
                }

                PartitionCount += 1;
            }
        }

        if LayoutInfo != core::ptr::null_mut() {
            ExFreePoolWithTag(LayoutInfo as *mut core::ffi::c_void, TAG_FSTUB);
        }

        HarddiskCount = i as u32 + 1;
        i += 1;
    }

    i = 0;
    while i < DiskCount as u16 {
        if Devices != core::ptr::null_mut() {
            HarddiskCount = *Devices.offset(i as isize);
        } else {
            HarddiskCount = i as u32;
        }

        crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION0_FMT, HarddiskCount);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
        if !NT_SUCCESS(HalpQueryDriveLayout(core::ptr::addr_of_mut!(StringU1), core::ptr::addr_of_mut!(LayoutInfo))) {
            LayoutInfo = core::ptr::null_mut();
        }

        PartitionCount = 1;
        loop {
            crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION_FMT, HarddiskCount, PartitionCount);
            RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
            if !NT_SUCCESS(HalpQueryPartitionType(core::ptr::addr_of_mut!(StringU1), LayoutInfo, core::ptr::addr_of_mut!(PartitionType))) {
                break;
            }

            if PartitionType == LogicalPartition {
                HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), NtDeviceName, NtSystemPath, 0);
            }

            PartitionCount += 1;
        }

        if LayoutInfo != core::ptr::null_mut() {
            ExFreePoolWithTag(LayoutInfo as *mut core::ffi::c_void, 0);
        }

        i += 1;
    }

    i = 0;
    while i < DiskCount as u16 {
        if Devices != core::ptr::null_mut() {
            HarddiskCount = *Devices.offset(i as isize);
        } else {
            HarddiskCount = i as u32;
        }

        crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION0_FMT, HarddiskCount);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
        if !NT_SUCCESS(HalpQueryDriveLayout(core::ptr::addr_of_mut!(StringU1), core::ptr::addr_of_mut!(LayoutInfo))) {
            LayoutInfo = core::ptr::null_mut();
        }

        SystemPartition = 0;
        PartitionCount = 1;
        loop {
            crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION_FMT, HarddiskCount, PartitionCount);
            RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
            if !NT_SUCCESS(HalpQueryPartitionType(core::ptr::addr_of_mut!(StringU1), LayoutInfo, core::ptr::addr_of_mut!(PartitionType))) {
                break;
            }

            if (PartitionType == BootablePartition || PartitionType == PrimaryPartition) && (SystemPartition == 0) {
                SystemPartition = PartitionCount;
            }

            PartitionCount += 1;
        }

        PartitionCount = 1;
        loop {
            if PartitionCount != SystemPartition {
                crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_HARDDISK_PARTITION_FMT, HarddiskCount, PartitionCount);
                RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
                if !NT_SUCCESS(HalpQueryPartitionType(core::ptr::addr_of_mut!(StringU1), LayoutInfo, core::ptr::addr_of_mut!(PartitionType))) {
                    if LayoutInfo != core::ptr::null_mut() {
                        ExFreePoolWithTag(LayoutInfo as *mut core::ffi::c_void, 0);
                    }

                    break;
                }

                if PartitionType == PrimaryPartition || PartitionType == FtPartition {
                    HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), NtDeviceName, NtSystemPath, 0);
                }
            }

            PartitionCount += 1;
        }

        i += 1;
    }

    if Devices != core::ptr::null_mut() {
        ExFreePoolWithTag(Devices as *mut core::ffi::c_void, TAG_FSTUB);
    }

    i = 0;
    while i < (*ConfigInfo).FloppyCount as u16 {
        crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_DEVICE_FLOPPY_FMT, i as u32);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
        if HalpIsOldStyleFloppy(core::ptr::addr_of_mut!(StringU1)) != 0 {
            HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), NtDeviceName, NtSystemPath, 1);
        }

        i += 1;
    }

    i = 0;
    while i < (*ConfigInfo).CdRomCount as u16 {
        crate::swprintf!(Buffer.as_mut_ptr(), Buffer.len(), &W_DEVICE_CDROM_FMT, i as u32);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU1), Buffer.as_ptr());
        HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), NtDeviceName, NtSystemPath, 1);

        i += 1;
    }

    if IoRemoteBootClient == 0 && NT_SUCCESS(RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(StringU1), NtDeviceName, 1)) {
        DriveLetter = HalpNextDriveLetter(core::ptr::addr_of_mut!(StringU1), core::ptr::null_mut(), core::ptr::null_mut(), 1);
        if DriveLetter != 0 {
            if DriveLetter != 0xFF {
                *NtSystemPath = DriveLetter;
            }
        } else {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU2), W_DEVICE_FLOPPY.as_ptr());
            RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU3), W_DEVICE_CDROM.as_ptr());

            if RtlPrefixUnicodeString(core::ptr::addr_of_mut!(StringU2), core::ptr::addr_of_mut!(StringU1), 1) != 0 {
                DriveLetter = 'A' as u8;
            } else if RtlPrefixUnicodeString(core::ptr::addr_of_mut!(StringU3), core::ptr::addr_of_mut!(StringU1), 1) != 0 {
                DriveLetter = 'D' as u8;
            } else {
                DriveLetter = 'C' as u8;
            }

            while HalpSetMountLetter(core::ptr::addr_of_mut!(StringU1), DriveLetter) != STATUS_SUCCESS {
                DriveLetter += 1;

                if DriveLetter > 'Z' as u8 {
                    break;
                }
            }

            if DriveLetter > 'Z' as u8 {
                HalpDeleteMountLetter('Z' as u8);
                HalpSetMountLetter(core::ptr::addr_of_mut!(StringU1), 'Z' as u8);
                *NtSystemPath = 'Z' as u8;
            } else {
                *NtSystemPath = DriveLetter;
            }
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(StringU1));
    }

    if !NtDeviceName.is_null()
        && NT_SUCCESS(RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(StringU1), NtDeviceName, 1))
    {
        let mut system_drive_letter = 0u8;
        let mut is_cdrom = 0u8;

        RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU3), W_DEVICE_CDROM.as_ptr());
        if RtlPrefixUnicodeString(core::ptr::addr_of_mut!(StringU3), core::ptr::addr_of_mut!(StringU1), 1) != 0 {
            is_cdrom = 1;
        }

        if !NtSystemPath.is_null()
            && *NtSystemPath >= b'A'
            && *NtSystemPath <= b'Z'
            && *NtSystemPath.add(1) == b':'
        {
            system_drive_letter = *NtSystemPath;
        } else if is_cdrom != 0 {
            system_drive_letter = b'X';
        }

        if system_drive_letter != 0 {
            crate::swprintf!(
                Buffer.as_mut_ptr(),
                Buffer.len(),
                &W_DOS_DEVICE_DRIVE_FMT,
                system_drive_letter as u32,
            );
            RtlInitUnicodeString(core::ptr::addr_of_mut!(StringU2), Buffer.as_ptr());
            if is_cdrom != 0 && system_drive_letter == b'X' {
                Status = IoDeleteSymbolicLink(core::ptr::addr_of_mut!(StringU2));
            }
            Status = IoCreateSymbolicLink(core::ptr::addr_of_mut!(StringU2), core::ptr::addr_of_mut!(StringU1));
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(StringU1));
    }

    HalpEnableAutomaticDriveLetterAssignment();
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetFullGeometry(
    DeviceObject: PDEVICE_OBJECT,
    Geometry: PDISK_GEOMETRY_EX,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    let Event = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<KEVENT>() as _,
        TAG_FILE_SYSTEM,
    ) as PKEVENT;
    if Event.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(Event, 0, 0u8);

    let Irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
        DeviceObject,
        core::ptr::null_mut(),
        0u32,
        Geometry as *mut core::ffi::c_void,
        core::mem::size_of::<DISK_GEOMETRY_EX>() as u32,
        0u8,
        Event,
        core::ptr::addr_of_mut!(IoStatusBlock),
    );
    if Irp.is_null() {
        ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(Event as *mut core::ffi::c_void, 0, 0i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalpIsValidPartitionEntry(
    Entry: PPARTITION_DESCRIPTOR,
    MaxOffset: u64,
    MaxSector: u64,
) -> u8 {
    let EndingSector: u64;

    if (*Entry).PartitionType == 0 {
        return 1u8;
    }

    EndingSector = GET_STARTING_SECTOR(Entry) as u64 + GET_PARTITION_LENGTH(Entry);

    if EndingSector > MaxSector {
        return 0u8;
    } else if (GET_STARTING_SECTOR(Entry) as u64) > MaxOffset {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn HalpCalculateChsValues(
    PartitionOffset: PLARGE_INTEGER,
    PartitionLength: PLARGE_INTEGER,
    ShiftCount: i8,
    SectorsPerTrack: u32,
    NumberOfTracks: u32,
    ConventionalCylinders: u32,
    PartitionDescriptor: PPARTITION_DESCRIPTOR,
) {
    let mut FirstSector: LARGE_INTEGER = core::mem::zeroed();
    let mut SectorCount: LARGE_INTEGER = core::mem::zeroed();
    let LastSector: u32;
    let mut Remainder: u32;
    let SectorsPerCylinder: u32;
    let mut StartingCylinder: u32;
    let mut EndingCylinder: u32;
    let StartingTrack: u32;
    let EndingTrack: u32;
    let StartingSector: u32;
    let EndingSector: u32;
    let mut ConventionalCylinders = ConventionalCylinders;

    SectorsPerCylinder = SectorsPerTrack * NumberOfTracks;

    FirstSector.QuadPart = (*PartitionOffset).QuadPart >> ShiftCount;
    SectorCount.QuadPart = (*PartitionLength).QuadPart >> ShiftCount;

    LastSector = FirstSector.u.LowPart + SectorCount.u.LowPart - 1;

    StartingCylinder = FirstSector.u.LowPart / SectorsPerCylinder;
    EndingCylinder = LastSector / SectorsPerCylinder;

    if ConventionalCylinders == 0 {
        ConventionalCylinders = 1024;
    }

    if StartingCylinder >= ConventionalCylinders {
        StartingCylinder = ConventionalCylinders - 1;
    }
    if EndingCylinder >= ConventionalCylinders {
        EndingCylinder = ConventionalCylinders - 1;
    }

    Remainder = FirstSector.u.LowPart % SectorsPerCylinder;
    StartingTrack = Remainder / SectorsPerTrack;
    StartingSector = Remainder % SectorsPerTrack;

    Remainder = LastSector % SectorsPerCylinder;
    EndingTrack = Remainder / SectorsPerTrack;
    EndingSector = Remainder % SectorsPerTrack;

    (*PartitionDescriptor).StartingCylinderMsb = StartingCylinder as u8;
    (*PartitionDescriptor).EndingCylinderMsb = EndingCylinder as u8;

    (*PartitionDescriptor).StartingTrack = StartingTrack as u8;
    (*PartitionDescriptor).EndingTrack = EndingTrack as u8;

    StartingCylinder = ((StartingSector + 1) & 0x3F) | ((StartingCylinder >> 2) & 0xC0);
    EndingCylinder = ((EndingSector + 1) & 0x3F) | ((EndingCylinder >> 2) & 0xC0);

    (*PartitionDescriptor).StartingCylinderLsb = StartingCylinder as u8;
    (*PartitionDescriptor).EndingCylinderLsb = EndingCylinder as u8;
}

#[no_mangle]
pub unsafe extern "C" fn xHalGetPartialGeometry(
    DeviceObject: PDEVICE_OBJECT,
    ConventionalCylinders: PULONG,
    DiskSize: PLONGLONG,
) {
    let mut DiskGeometry: PDISK_GEOMETRY = core::ptr::null_mut();
    let mut IoStatusBlock: PIO_STATUS_BLOCK = core::ptr::null_mut();
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut Irp: PIRP;
    let mut Status: NTSTATUS;

    *ConventionalCylinders = 0;
    *DiskSize = 0;

    DiskGeometry = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<DISK_GEOMETRY>() as _,
        TAG_FILE_SYSTEM,
    ) as PDISK_GEOMETRY;
    if DiskGeometry.is_null() {
        'Cleanup: loop {
            if !Event.is_null() {
                ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !IoStatusBlock.is_null() {
                ExFreePoolWithTag(IoStatusBlock as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !DiskGeometry.is_null() {
                ExFreePoolWithTag(DiskGeometry as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            return;
        }
    }

    IoStatusBlock = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_IO_STATUS_BLOCK>() as _,
        TAG_FILE_SYSTEM,
    ) as PIO_STATUS_BLOCK;
    if IoStatusBlock.is_null() {
        'Cleanup: loop {
            if !Event.is_null() {
                ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !IoStatusBlock.is_null() {
                ExFreePoolWithTag(IoStatusBlock as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !DiskGeometry.is_null() {
                ExFreePoolWithTag(DiskGeometry as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            return;
        }
    }

    Event = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<KEVENT>() as _,
        TAG_FILE_SYSTEM,
    ) as PKEVENT;
    if Event.is_null() {
        'Cleanup: loop {
            if !Event.is_null() {
                ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !IoStatusBlock.is_null() {
                ExFreePoolWithTag(IoStatusBlock as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !DiskGeometry.is_null() {
                ExFreePoolWithTag(DiskGeometry as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            return;
        }
    }

    KeInitializeEvent(Event, _EVENT_TYPE_NotificationEvent, 0u8);

    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_GET_DRIVE_GEOMETRY,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        DiskGeometry as *mut core::ffi::c_void,
        core::mem::size_of::<DISK_GEOMETRY>() as u32,
        0u8,
        Event,
        IoStatusBlock,
    );
    if Irp.is_null() {
        'Cleanup: loop {
            if !Event.is_null() {
                ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !IoStatusBlock.is_null() {
                ExFreePoolWithTag(IoStatusBlock as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            if !DiskGeometry.is_null() {
                ExFreePoolWithTag(DiskGeometry as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
            }
            return;
        }
    }

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(Event as *mut core::ffi::c_void, _KWAIT_REASON_Executive, _MODE_KernelMode as i8 as i8, 0u8, core::ptr::null_mut());
        Status = (*IoStatusBlock).__bindgen_anon_1.Status;
    }

    if NT_SUCCESS(Status) {
        *ConventionalCylinders = (*DiskGeometry).Cylinders.__bindgen_anon_1.LowPart;

        if (*DiskGeometry).Cylinders.__bindgen_anon_1.LowPart >= 1024 {
            *ConventionalCylinders = 1024;
        }

        *DiskSize = (*DiskGeometry).Cylinders.QuadPart as i64
            * (*DiskGeometry).TracksPerCylinder as i64
            * (*DiskGeometry).SectorsPerTrack as i64
            * (*DiskGeometry).BytesPerSector as i64;
    }

    if !Event.is_null() {
        ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
    }
    if !IoStatusBlock.is_null() {
        ExFreePoolWithTag(IoStatusBlock as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
    }
    if !DiskGeometry.is_null() {
        ExFreePoolWithTag(DiskGeometry as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
    }
    return;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn xHalExamineMBR(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    MbrTypeIdentifier: ULONG,
    MbrBuffer: *mut PVOID,
) {
    xHalExamineMBR_impl(DeviceObject, SectorSize, MbrTypeIdentifier, MbrBuffer);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn xHalExamineMBR(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    MbrTypeIdentifier: ULONG,
    MbrBuffer: *mut PVOID,
) {
    xHalExamineMBR_impl(DeviceObject, SectorSize, MbrTypeIdentifier, MbrBuffer);
}

unsafe fn xHalExamineMBR_impl(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    MbrTypeIdentifier: ULONG,
    MbrBuffer: *mut PVOID,
) {
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let Buffer: PUCHAR;
    let BufferSize: ULONG;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Irp: PIRP;
    let PartitionDescriptor: PPARTITION_DESCRIPTOR;
    let mut Status: NTSTATUS;
    let mut IoStackLocation: PIO_STACK_LOCATION;

    Offset.u.LowPart = 0;
    Offset.u.HighPart = 0;

    *MbrBuffer = core::ptr::null_mut();

    BufferSize = if 512 > SectorSize { 512 } else { SectorSize };

    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        if PAGE_SIZE > BufferSize as usize { PAGE_SIZE } else { BufferSize as usize },
        TAG_FILE_SYSTEM,
    ) as PUCHAR;
    if Buffer.is_null() {
        return;
    }

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_READ as u32,
        DeviceObject,
        Buffer as PVOID,
        BufferSize,
        &mut Offset,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        ExFreePoolWithTag(Buffer as PVOID, TAG_FILE_SYSTEM);
        return;
    }

    IoStackLocation = IoGetNextIrpStackLocation(Irp);
    (*IoStackLocation).Flags |= SL_OVERRIDE_VERIFY_VOLUME;

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(&mut Event as *mut _ as PVOID, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    if NT_SUCCESS(Status) {
        if *(Buffer.add(BOOT_SIGNATURE_OFFSET as usize) as PUINT16) != BOOT_RECORD_SIGNATURE {
            ExFreePoolWithTag(Buffer as PVOID, TAG_FILE_SYSTEM);
            return;
        }

        PartitionDescriptor = Buffer.add(PARTITION_TABLE_OFFSET as usize) as PPARTITION_DESCRIPTOR;

        if (*PartitionDescriptor).PartitionType != MbrTypeIdentifier as u8 {
            ExFreePoolWithTag(Buffer as PVOID, TAG_FILE_SYSTEM);
        } else {
            if (*PartitionDescriptor).PartitionType == PARTITION_DM {
                *(Buffer as PULONG) = 63;
                *MbrBuffer = Buffer as PVOID;
            } else if (*PartitionDescriptor).PartitionType == PARTITION_EZDRIVE {
                *MbrBuffer = Buffer as PVOID;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FstubFixupEfiPartition(
    PartitionDescriptor: PPARTITION_DESCRIPTOR,
    MaxOffset: u64,
) {

    let PartitionMaxOffset: u32 = (GET_STARTING_SECTOR(PartitionDescriptor) as u64)
        .wrapping_add(GET_PARTITION_LENGTH(PartitionDescriptor)) as u32;

    if (PartitionMaxOffset as u64) > MaxOffset {
        let PartitionLength: u32 = (PartitionMaxOffset as u64)
            .wrapping_sub(GET_STARTING_SECTOR(PartitionDescriptor) as u64) as u32;
        SET_PARTITION_LENGTH(PartitionDescriptor, PartitionLength);
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn xHalIoReadPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: u32,
    ReturnRecognizedPartitions: u8,
    PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> i32 {

    const BOOT_SIGNATURE_OFFSET: usize = 0x1FE;
    const BOOT_RECORD_SIGNATURE: u16 = 0xAA55;
    const DISK_SIGNATURE_OFFSET: usize = 0x1B8;
    const PARTITION_TABLE_OFFSET: usize = 0x1BE;
    const NUM_PARTITION_TABLE_ENTRIES: i8 = 4;
    const PARTITION_EZDRIVE: u32 = 0x55;
    const PARTITION_ENTRY_UNUSED: u8 = 0x00;
    const EFI_PMBR_OSTYPE_EFI: u8 = 0xEE;
    const PARTITION_FAT_16: u8 = 0x4;
    const TAG_FILE_SYSTEM: u32 = u32::from_be_bytes(*b"SYSF");
    const IRP_MJ_READ: u8 = 0x03;
    const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02;
    const NotificationEvent: u32 = 0;
    const Executive: i8 = 0;
    // const KernelMode: i8 = 0; // body uses literal 0i32 directly
    const RemovableMedia: u32 = 11;
    const STATUS_PENDING: i32 = 0x00000103u32 as i32;
    const STATUS_NO_DATA_DETECTED: i32 = 0x80000022u32 as i32;

    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut PartitionDescriptor: PPARTITION_DESCRIPTOR;
    let mut Entry: i8;
    let mut Status: i32;
    let mut PartitionInfo: PPARTITION_INFORMATION;
    let mut Buffer: *mut u8 = core::ptr::null_mut();
    let mut BufferSize: u32 = 2048;
    let InputSize: u32;
    let mut DriveLayoutInfo: PDRIVE_LAYOUT_INFORMATION = core::ptr::null_mut();
    let mut j: i32 = -1;
    let mut i: i32 = -1;
    let mut k: i32;
    let mut DiskGeometryEx: DISK_GEOMETRY_EX = core::mem::zeroed();
    let EndSector: i64;
    let mut MaxSector: i64;
    let StartOffset: i64;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut VolumeOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IsPrimary: u8 = 1u8;
    let mut IsEzDrive: u8 = 0u8;
    let mut MbrFound: u8 = 0u8;
    let mut IsValid: u8;
    let mut IsEmpty: u8 = 1u8;
    let mut MbrBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut IoStackLocation: PIO_STACK_LOCATION;
    let PartitionType: u8;
    let mut HiddenSectors64: LARGE_INTEGER;

    VolumeOffset.QuadPart = 0;
    Offset.QuadPart = 0;

    *PartitionBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        BufferSize as usize,
        TAG_FILE_SYSTEM,
    ) as PDRIVE_LAYOUT_INFORMATION;
    if (*PartitionBuffer).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InputSize = max(512, SectorSize);

    HalExamineMBR(DeviceObject, InputSize, PARTITION_EZDRIVE, &mut MbrBuffer);
    if !MbrBuffer.is_null() {
        IsEzDrive = 1u8;
        ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
        Offset.QuadPart = 512;
    }

    Status = HalpGetFullGeometry(DeviceObject, &mut DiskGeometryEx);
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
        *PartitionBuffer = core::ptr::null_mut();
        return Status;
    }

    EndSector = DiskGeometryEx.DiskSize.QuadPart / (DiskGeometryEx.Geometry.BytesPerSector as i64);
    MaxSector = EndSector << 1;

    Buffer = ExAllocatePoolWithTag(
        NonPagedPoolCacheAligned,
        InputSize as usize,
        TAG_FILE_SYSTEM,
    ) as *mut u8;
    if Buffer.is_null() {
        ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
        *PartitionBuffer = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    loop {
        IsValid = 1u8;

        KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

        RtlZeroMemory(Buffer as *mut core::ffi::c_void, InputSize as usize);
        Irp = IoBuildSynchronousFsdRequest(
            IRP_MJ_READ as u32,
            DeviceObject,
            Buffer as *mut core::ffi::c_void,
            InputSize,
            &mut Offset,
            &mut Event,
            &mut IoStatusBlock,
        );
        if Irp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        IoStackLocation = IoGetNextIrpStackLocation(Irp);
        (*IoStackLocation).Flags |= SL_OVERRIDE_VERIFY_VOLUME;

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(&mut Event as *mut KEVENT as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if Status == STATUS_NO_DATA_DETECTED {
            Status = STATUS_SUCCESS;
        }
        if !NT_SUCCESS(Status) {
            break;
        }

        if IsEzDrive != 0 && Offset.QuadPart == 512 {
            Offset.QuadPart = 0;
        }

        if *(Buffer.add(BOOT_SIGNATURE_OFFSET) as PUINT16) != BOOT_RECORD_SIGNATURE {
            break;
        }

        MbrFound = 1u8;

        if Offset.QuadPart == 0 {
            (**PartitionBuffer).Signature = *(Buffer.add(DISK_SIGNATURE_OFFSET) as PUINT32);
        }

        PartitionDescriptor = Buffer.add(PARTITION_TABLE_OFFSET) as PPARTITION_DESCRIPTOR;

        j += 1;
        Entry = 1;
        k = 0;
        while Entry <= NUM_PARTITION_TABLE_ENTRIES {
            let PartitionType = (*PartitionDescriptor).PartitionType;

            if PartitionType == EFI_PMBR_OSTYPE_EFI {
                FstubFixupEfiPartition(PartitionDescriptor, DiskGeometryEx.DiskSize.QuadPart as u64);
            }

            if HalpIsValidPartitionEntry(
                PartitionDescriptor,
                DiskGeometryEx.DiskSize.QuadPart as u64,
                MaxSector as u64,
            ) == 0 && j == 0
            {
                IsValid = 0u8;
                break;
            }

            if IsContainerPartition(PartitionType) != 0 {
                k += 1;
                if k != 1 {
                    IsValid = 0u8;
                    break;
                }
            }

            if IsEmpty != 0 {
                if GET_STARTING_SECTOR(PartitionDescriptor) != 0
                    || GET_PARTITION_LENGTH(PartitionDescriptor) != 0
                {
                    IsEmpty = 0u8;
                }
            }

            if ReturnRecognizedPartitions != 0 {
                if PartitionType == PARTITION_ENTRY_UNUSED || IsContainerPartition(PartitionType) != 0 {
                    Entry += 1;
                    PartitionDescriptor = PartitionDescriptor.add(1);
                    continue;
                }
            }

            i += 1;
            if (core::mem::size_of::<DRIVE_LAYOUT_INFORMATION>() as u32
                + (i as u32 * core::mem::size_of::<PARTITION_INFORMATION>() as u32))
                > BufferSize
            {
                DriveLayoutInfo = ExAllocatePoolWithTag(
                    NonPagedPool,
                    (BufferSize << 1) as usize,
                    TAG_FILE_SYSTEM,
                ) as PDRIVE_LAYOUT_INFORMATION;
                if DriveLayoutInfo.is_null() {
                    i -= 1;
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }

                core::ptr::copy_nonoverlapping(
                    *PartitionBuffer as *const u8,
                    DriveLayoutInfo as *mut u8,
                    BufferSize as usize,
                );

                ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
                *PartitionBuffer = DriveLayoutInfo;

                BufferSize <<= 1;
            }

            // NB: bindgen flexible array ([T;1]) — [i>=1] bounds-panics; raw ptr add.
            PartitionInfo = core::ptr::addr_of_mut!((**PartitionBuffer).PartitionEntry)
                .cast::<PARTITION_INFORMATION>()
                .add(i as usize);
            (*PartitionInfo).PartitionType = PartitionType;
            (*PartitionInfo).RewritePartition = 0u8;

            if PartitionType != PARTITION_ENTRY_UNUSED {
                (*PartitionInfo).BootIndicator = if (*PartitionDescriptor).ActiveFlag & 0x80 != 0 {
                    1u8
                } else {
                    0u8
                };

                let StartOffset: i64;
                if IsContainerPartition(PartitionType) != 0 {
                    (*PartitionInfo).RecognizedPartition = 0u8;
                    StartOffset = VolumeOffset.QuadPart;
                } else {
                    (*PartitionInfo).RecognizedPartition = 1u8;
                    StartOffset = Offset.QuadPart;
                }

                (*PartitionInfo).StartingOffset.QuadPart = StartOffset
                    + UInt32x32To64(GET_STARTING_SECTOR(PartitionDescriptor), SectorSize) as i64;

                HiddenSectors64 = core::mem::zeroed();
                HiddenSectors64.QuadPart = ((*PartitionInfo).StartingOffset.QuadPart - StartOffset)
                    / SectorSize as i64;
                (*PartitionInfo).HiddenSectors = HiddenSectors64.u.LowPart;

                (*PartitionInfo).PartitionLength.QuadPart =
                    UInt32x32To64(GET_PARTITION_LENGTH(PartitionDescriptor) as u32, SectorSize) as i64;

                (*PartitionInfo).PartitionNumber = if IsContainerPartition(PartitionType) == 0 {
                    (i + 1) as u32
                } else {
                    0
                };
            } else {
                (*PartitionInfo).BootIndicator = 0u8;
                (*PartitionInfo).RecognizedPartition = 0u8;
                (*PartitionInfo).StartingOffset.QuadPart = 0;
                (*PartitionInfo).PartitionLength.QuadPart = 0;
                (*PartitionInfo).HiddenSectors = 0;
                (*PartitionInfo).PartitionNumber = 0;
            }

            Entry += 1;
            PartitionDescriptor = PartitionDescriptor.add(1);
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        if IsValid == 0 {
            j -= 1;
            break;
        }

        Offset.QuadPart = 0;

        PartitionDescriptor = Buffer.add(PARTITION_TABLE_OFFSET) as PPARTITION_DESCRIPTOR;
        Entry = 1;
        while Entry <= NUM_PARTITION_TABLE_ENTRIES {
            if IsContainerPartition((*PartitionDescriptor).PartitionType) != 0 {
                Offset.QuadPart = VolumeOffset.QuadPart
                    + UInt32x32To64(GET_STARTING_SECTOR(PartitionDescriptor), SectorSize) as i64;

                if IsPrimary != 0 {
                    VolumeOffset = Offset;
                }

                MaxSector = GET_PARTITION_LENGTH(PartitionDescriptor) as i64;
                break;
            }
            Entry += 1;
            PartitionDescriptor = PartitionDescriptor.add(1);
        }

        IsPrimary = 0u8;

        if Offset.u.HighPart == 0 && Offset.u.LowPart == 0 {
            break;
        }
    }

    if DiskGeometryEx.Geometry.MediaType == RemovableMedia
        && j == 0
        && MbrFound != 0
        && IsEmpty != 0
    {
        let BootSectorInfo = Buffer as PBOOT_SECTOR_INFO;

        if (*BootSectorInfo).JumpByte[0] == 0xeb || (*BootSectorInfo).JumpByte[0] == 0xe9 {
            j = -1;
        }
    }

    if j == -1 {
        if MbrFound != 0 || DiskGeometryEx.Geometry.MediaType == RemovableMedia {
            if EndSector > 0 {
                PartitionInfo = &mut (**PartitionBuffer).PartitionEntry[0];

                (*PartitionInfo).RewritePartition = 0u8;
                (*PartitionInfo).RecognizedPartition = 1u8;
                (*PartitionInfo).PartitionType = PARTITION_FAT_16;
                (*PartitionInfo).BootIndicator = 0u8;
                (*PartitionInfo).HiddenSectors = 0;
                (*PartitionInfo).StartingOffset.QuadPart = 0;
                (*PartitionInfo).PartitionLength = DiskGeometryEx.DiskSize;
                (*PartitionInfo).PartitionNumber = 0;

                (**PartitionBuffer).Signature = 1;
                i = 0;
            }
        } else {
            i = -1;
        }
    }

    i += 1;
    (**PartitionBuffer).PartitionCount = i as u32;

    if i == 0 {
        (**PartitionBuffer).Signature = 0;
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
    }
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
        *PartitionBuffer = core::ptr::null_mut();
    }

    Status
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn xHalIoReadPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: u32,
    ReturnRecognizedPartitions: u8,
    PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> i32 {

    const BOOT_SIGNATURE_OFFSET: usize = 0x1FE;
    const BOOT_RECORD_SIGNATURE: u16 = 0xAA55;
    const DISK_SIGNATURE_OFFSET: usize = 0x1B8;
    const PARTITION_TABLE_OFFSET: usize = 0x1BE;
    const NUM_PARTITION_TABLE_ENTRIES: i8 = 4;
    const PARTITION_EZDRIVE: u32 = 0x55;
    const PARTITION_ENTRY_UNUSED: u8 = 0x00;
    const EFI_PMBR_OSTYPE_EFI: u8 = 0xEE;
    // ntdddisk.h:270 -- PARTITION_FAT_16 is 0x04; 0x06 is PARTITION_HUGE.
    const PARTITION_FAT_16: u8 = 0x04;
    const TAG_FILE_SYSTEM: u32 = u32::from_be_bytes(*b"SYSF");
    const IRP_MJ_READ: u8 = 0x03;
    const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02;
    const NotificationEvent: u32 = 0;
    const Executive: i8 = 0;
    // const KernelMode: i8 = 0; // body uses literal 0i32 directly
    const RemovableMedia: u32 = 11;
    const STATUS_PENDING: i32 = 0x00000103u32 as i32;
    const STATUS_NO_DATA_DETECTED: i32 = 0x80000022u32 as i32;

    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut PartitionDescriptor: PPARTITION_DESCRIPTOR;
    let mut Entry: i8;
    let mut Status: i32;
    let mut PartitionInfo: PPARTITION_INFORMATION;
    let mut Buffer: *mut u8 = core::ptr::null_mut();
    let mut BufferSize: u32 = 2048;
    let InputSize: u32;
    let mut DriveLayoutInfo: PDRIVE_LAYOUT_INFORMATION = core::ptr::null_mut();
    let mut j: i32 = -1;
    let mut i: i32 = -1;
    let mut k: i32;
    let mut DiskGeometryEx: DISK_GEOMETRY_EX = core::mem::zeroed();
    let EndSector: i64;
    let mut MaxSector: i64;
    let StartOffset: i64;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut VolumeOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IsPrimary: u8 = 1u8;
    let mut IsEzDrive: u8 = 0u8;
    let mut MbrFound: u8 = 0u8;
    let mut IsValid: u8;
    let mut IsEmpty: u8 = 1u8;
    let mut MbrBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut IoStackLocation: PIO_STACK_LOCATION;
    let PartitionType: u8;
    let mut HiddenSectors64: LARGE_INTEGER;

    VolumeOffset.QuadPart = 0;
    Offset.QuadPart = 0;

    *PartitionBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        BufferSize as usize,
        TAG_FILE_SYSTEM,
    ) as PDRIVE_LAYOUT_INFORMATION;
    if (*PartitionBuffer).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InputSize = max(512, SectorSize);

    HalExamineMBR(DeviceObject, InputSize, PARTITION_EZDRIVE, &mut MbrBuffer);
    if !MbrBuffer.is_null() {
        IsEzDrive = 1u8;
        ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
        Offset.QuadPart = 512;
    }

    Status = HalpGetFullGeometry(DeviceObject, &mut DiskGeometryEx);
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
        *PartitionBuffer = core::ptr::null_mut();
        return Status;
    }

    EndSector = DiskGeometryEx.DiskSize.QuadPart / (DiskGeometryEx.Geometry.BytesPerSector as i64);
    MaxSector = EndSector << 1;

    Buffer = ExAllocatePoolWithTag(
        NonPagedPoolCacheAligned,
        InputSize as usize,
        TAG_FILE_SYSTEM,
    ) as *mut u8;
    if Buffer.is_null() {
        ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
        *PartitionBuffer = core::ptr::null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    loop {
        IsValid = 1u8;

        KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

        RtlZeroMemory(Buffer as *mut core::ffi::c_void, InputSize as usize);
        Irp = IoBuildSynchronousFsdRequest(
            IRP_MJ_READ as u32,
            DeviceObject,
            Buffer as *mut core::ffi::c_void,
            InputSize,
            &mut Offset,
            &mut Event,
            &mut IoStatusBlock,
        );
        if Irp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        IoStackLocation = IoGetNextIrpStackLocation(Irp);
        (*IoStackLocation).Flags |= SL_OVERRIDE_VERIFY_VOLUME;

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(&mut Event as *mut KEVENT as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if Status == STATUS_NO_DATA_DETECTED {
            Status = STATUS_SUCCESS;
        }
        if !NT_SUCCESS(Status) {
            break;
        }

        if IsEzDrive != 0 && Offset.QuadPart == 512 {
            Offset.QuadPart = 0;
        }

        if *(Buffer.add(BOOT_SIGNATURE_OFFSET) as PUINT16) != BOOT_RECORD_SIGNATURE {
            break;
        }

        MbrFound = 1u8;

        if Offset.QuadPart == 0 {
            (**PartitionBuffer).Signature = *(Buffer.add(DISK_SIGNATURE_OFFSET) as PUINT32);
        }

        PartitionDescriptor = Buffer.add(PARTITION_TABLE_OFFSET) as PPARTITION_DESCRIPTOR;

        j += 1;
        Entry = 1;
        k = 0;
        while Entry <= NUM_PARTITION_TABLE_ENTRIES {
            let PartitionType = (*PartitionDescriptor).PartitionType;

            if PartitionType == EFI_PMBR_OSTYPE_EFI {
                FstubFixupEfiPartition(PartitionDescriptor, DiskGeometryEx.DiskSize.QuadPart as u64);
            }

            if HalpIsValidPartitionEntry(
                PartitionDescriptor,
                DiskGeometryEx.DiskSize.QuadPart as u64,
                MaxSector as u64,
            ) == 0 && j == 0
            {
                IsValid = 0u8;
                break;
            }

            if IsContainerPartition(PartitionType) != 0 {
                k += 1;
                if k != 1 {
                    IsValid = 0u8;
                    break;
                }
            }

            if IsEmpty != 0 {
                if GET_STARTING_SECTOR(PartitionDescriptor) != 0
                    || GET_PARTITION_LENGTH(PartitionDescriptor) != 0
                {
                    IsEmpty = 0u8;
                }
            }

            if ReturnRecognizedPartitions != 0 {
                if PartitionType == PARTITION_ENTRY_UNUSED || IsContainerPartition(PartitionType) != 0 {
                    Entry += 1;
                    PartitionDescriptor = PartitionDescriptor.add(1);
                    continue;
                }
            }

            i += 1;
            if (core::mem::size_of::<DRIVE_LAYOUT_INFORMATION>() as u32
                + (i as u32 * core::mem::size_of::<PARTITION_INFORMATION>() as u32))
                > BufferSize
            {
                DriveLayoutInfo = ExAllocatePoolWithTag(
                    NonPagedPool,
                    (BufferSize << 1) as usize,
                    TAG_FILE_SYSTEM,
                ) as PDRIVE_LAYOUT_INFORMATION;
                if DriveLayoutInfo.is_null() {
                    i -= 1;
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }

                core::ptr::copy_nonoverlapping(
                    *PartitionBuffer as *const u8,
                    DriveLayoutInfo as *mut u8,
                    BufferSize as usize,
                );

                ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
                *PartitionBuffer = DriveLayoutInfo;

                BufferSize <<= 1;
            }

            // NB: bindgen flexible array ([T;1]) — [i>=1] bounds-panics; raw ptr add.
            PartitionInfo = core::ptr::addr_of_mut!((**PartitionBuffer).PartitionEntry)
                .cast::<PARTITION_INFORMATION>()
                .add(i as usize);
            (*PartitionInfo).PartitionType = PartitionType;
            (*PartitionInfo).RewritePartition = 0u8;

            if PartitionType != PARTITION_ENTRY_UNUSED {
                (*PartitionInfo).BootIndicator = if (*PartitionDescriptor).ActiveFlag & 0x80 != 0 {
                    1u8
                } else {
                    0u8
                };

                let StartOffset: i64;
                if IsContainerPartition(PartitionType) != 0 {
                    (*PartitionInfo).RecognizedPartition = 0u8;
                    StartOffset = VolumeOffset.QuadPart;
                } else {
                    (*PartitionInfo).RecognizedPartition = 1u8;
                    StartOffset = Offset.QuadPart;
                }

                (*PartitionInfo).StartingOffset.QuadPart = StartOffset
                    + UInt32x32To64(GET_STARTING_SECTOR(PartitionDescriptor), SectorSize) as i64;

                HiddenSectors64 = core::mem::zeroed();
                HiddenSectors64.QuadPart = ((*PartitionInfo).StartingOffset.QuadPart - StartOffset)
                    / SectorSize as i64;
                (*PartitionInfo).HiddenSectors = HiddenSectors64.u.LowPart;

                (*PartitionInfo).PartitionLength.QuadPart =
                    UInt32x32To64(GET_PARTITION_LENGTH(PartitionDescriptor) as u32, SectorSize) as i64;

                (*PartitionInfo).PartitionNumber = if IsContainerPartition(PartitionType) == 0 {
                    (i + 1) as u32
                } else {
                    0
                };
            } else {
                (*PartitionInfo).BootIndicator = 0u8;
                (*PartitionInfo).RecognizedPartition = 0u8;
                (*PartitionInfo).StartingOffset.QuadPart = 0;
                (*PartitionInfo).PartitionLength.QuadPart = 0;
                (*PartitionInfo).HiddenSectors = 0;
                (*PartitionInfo).PartitionNumber = 0;
            }

            Entry += 1;
            PartitionDescriptor = PartitionDescriptor.add(1);
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        if IsValid == 0 {
            j -= 1;
            break;
        }

        Offset.QuadPart = 0;

        PartitionDescriptor = Buffer.add(PARTITION_TABLE_OFFSET) as PPARTITION_DESCRIPTOR;
        Entry = 1;
        while Entry <= NUM_PARTITION_TABLE_ENTRIES {
            if IsContainerPartition((*PartitionDescriptor).PartitionType) != 0 {
                Offset.QuadPart = VolumeOffset.QuadPart
                    + UInt32x32To64(GET_STARTING_SECTOR(PartitionDescriptor), SectorSize) as i64;

                if IsPrimary != 0 {
                    VolumeOffset = Offset;
                }

                MaxSector = GET_PARTITION_LENGTH(PartitionDescriptor) as i64;
                break;
            }
            Entry += 1;
            PartitionDescriptor = PartitionDescriptor.add(1);
        }

        IsPrimary = 0u8;

        if Offset.u.HighPart == 0 && Offset.u.LowPart == 0 {
            break;
        }
    }

    if DiskGeometryEx.Geometry.MediaType == RemovableMedia
        && j == 0
        && MbrFound != 0
        && IsEmpty != 0
    {
        let BootSectorInfo = Buffer as PBOOT_SECTOR_INFO;

        if (*BootSectorInfo).JumpByte[0] == 0xeb || (*BootSectorInfo).JumpByte[0] == 0xe9 {
            j = -1;
        }
    }

    if j == -1 {
        if MbrFound != 0 || DiskGeometryEx.Geometry.MediaType == RemovableMedia {
            if EndSector > 0 {
                PartitionInfo = &mut (**PartitionBuffer).PartitionEntry[0];

                (*PartitionInfo).RewritePartition = 0u8;
                (*PartitionInfo).RecognizedPartition = 1u8;
                (*PartitionInfo).PartitionType = PARTITION_FAT_16;
                (*PartitionInfo).BootIndicator = 0u8;
                (*PartitionInfo).HiddenSectors = 0;
                (*PartitionInfo).StartingOffset.QuadPart = 0;
                (*PartitionInfo).PartitionLength = DiskGeometryEx.DiskSize;
                (*PartitionInfo).PartitionNumber = 0;

                (**PartitionBuffer).Signature = 1;
                i = 0;
            }
        } else {
            i = -1;
        }
    }

    i += 1;
    (**PartitionBuffer).PartitionCount = i as u32;

    if i == 0 {
        (**PartitionBuffer).Signature = 0;
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
    }
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(*PartitionBuffer as *mut core::ffi::c_void, TAG_FILE_SYSTEM);
        *PartitionBuffer = core::ptr::null_mut();
    }

    Status
}

unsafe fn max(a: ULONG, b: ULONG) -> ULONG {
    if a > b { a } else { b }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn xHalIoSetPartitionInformation(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    PartitionNumber: ULONG,
    PartitionType: ULONG,
) -> NTSTATUS {
    xHalIoSetPartitionInformation_impl(DeviceObject, SectorSize, PartitionNumber, PartitionType)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn xHalIoSetPartitionInformation(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    PartitionNumber: ULONG,
    PartitionType: ULONG,
) -> NTSTATUS {
    xHalIoSetPartitionInformation_impl(DeviceObject, SectorSize, PartitionNumber, PartitionType)
}

unsafe fn xHalIoSetPartitionInformation_impl(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    PartitionNumber: ULONG,
    PartitionType: ULONG,
) -> NTSTATUS {
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut VolumeOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Buffer: PUCHAR = core::ptr::null_mut();
    let BufferSize: ULONG;
    let mut i: ULONG = 0;
    let mut Entry: ULONG;
    let mut PartitionDescriptor: PPARTITION_DESCRIPTOR;
    let mut IsPrimary: BOOLEAN = 1u8;
    let IsEzDrive: BOOLEAN;
    let mut MbrBuffer: PVOID = core::ptr::null_mut();
    let mut IoStackLocation: PIO_STACK_LOCATION;

    Offset.QuadPart = 0;
    VolumeOffset.QuadPart = 0;

    BufferSize = max(512, SectorSize);

    HalExamineMBR(DeviceObject, BufferSize, 0x55, &mut MbrBuffer);
    if !MbrBuffer.is_null() {
        IsEzDrive = 1u8;
        ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
        Offset.QuadPart = 512;
    } else {
        IsEzDrive = 0u8;
    }

    Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, PAGE_SIZE, TAG_FILE_SYSTEM) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(&mut Event, 0, 0u8);

    loop {
        KeClearEvent(core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void);

        Irp = IoBuildSynchronousFsdRequest(
            3,
            DeviceObject,
            Buffer as PVOID,
            BufferSize,
            &mut Offset,
            &mut Event,
            &mut IoStatusBlock,
        );
        if Irp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        IoStackLocation = IoGetNextIrpStackLocation(Irp);
        (*IoStackLocation).Flags |= 0x02;

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == 0x00000103 {
            KeWaitForSingleObject(&mut Event as *mut KEVENT as PVOID, 0, 0, 0u8, core::ptr::null_mut());
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        if IsEzDrive != 0 && Offset.QuadPart == 512 {
            Offset.QuadPart = 0;
        }

        if *(Buffer.add(0x1FE) as PUINT16) != 0xAA55 {
            Status = STATUS_DISK_CORRUPT_ERROR;
            break;
        }

        PartitionDescriptor = Buffer.add(0x1BE) as PPARTITION_DESCRIPTOR;
        Entry = 1;
        while Entry <= 4 {
            if (*PartitionDescriptor).PartitionType != 0 &&
               IsContainerPartition((*PartitionDescriptor).PartitionType) == 0 {
                i += 1;
                if i == PartitionNumber {
                    (*PartitionDescriptor).PartitionType = PartitionType as UCHAR;

                    KeClearEvent(core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void);

                    Irp = IoBuildSynchronousFsdRequest(
                        4,
                        DeviceObject,
                        Buffer as PVOID,
                        BufferSize,
                        &mut Offset,
                        &mut Event,
                        &mut IoStatusBlock,
                    );
                    if Irp.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }

                    IoStackLocation = IoGetNextIrpStackLocation(Irp);
                    (*IoStackLocation).Flags |= 0x02;

                    Status = IoCallDriver(DeviceObject, Irp);
                    if Status == 0x00000103 {
                        KeWaitForSingleObject(&mut Event as *mut KEVENT as PVOID, 0, 0, 0u8, core::ptr::null_mut());
                        Status = IoStatusBlock.__bindgen_anon_1.Status;
                    }

                    break;
                }
            }
            Entry += 1;
            PartitionDescriptor = PartitionDescriptor.add(1);
        }

        if Entry <= 4 {
            break;
        }

        PartitionDescriptor = Buffer.add(0x1BE) as PPARTITION_DESCRIPTOR;
        Entry = 1;
        while Entry <= 4 {
            if IsContainerPartition((*PartitionDescriptor).PartitionType) != 0 {
                Offset.QuadPart = VolumeOffset.QuadPart +
                    ((GET_STARTING_SECTOR(PartitionDescriptor) as u64) * (SectorSize as u64)) as i64;

                if IsPrimary != 0 {
                    VolumeOffset = Offset;
                }
                break;
            }
            Entry += 1;
            PartitionDescriptor = PartitionDescriptor.add(1);
        }

        if Entry > 4 {
            Status = STATUS_DISK_CORRUPT_ERROR;
            break;
        }

        IsPrimary = 0u8;

        if i >= PartitionNumber {
            break;
        }
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer as PVOID, TAG_FILE_SYSTEM);
    }
    Status
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn xHalIoWritePartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    SectorsPerTrack: ULONG,
    NumberOfHeads: ULONG,
    PartitionBuffer: PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let BufferSize: ULONG;
    let mut Buffer: PUCHAR;
    let Entry: PPTE;
    let mut PartitionTable: PPARTITION_TABLE;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut NextOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ExtendedOffset: LARGE_INTEGER = core::mem::zeroed();
    let SectorOffset: LARGE_INTEGER;
    let mut StartOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut PartitionLength: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: i8;
    let mut IsEzDrive: BOOLEAN = 0u8;
    let mut IsSuperFloppy: BOOLEAN = 0u8;
    let mut DoRewrite: BOOLEAN = 0u8;
    let mut IsMbr: BOOLEAN;
    let mut ConventionalCylinders: ULONG = 0;
    let mut DiskSize: LONGLONG = 0;
    let DiskLayout: PDISK_LAYOUT = PartitionBuffer as PDISK_LAYOUT;
    let mut MbrBuffer: PVOID = core::ptr::null_mut();
    let PartitionType: UCHAR;
    let mut IoStackLocation: PIO_STACK_LOCATION;
    let PartitionInfo: PPARTITION_INFORMATION = (*PartitionBuffer).PartitionEntry.as_mut_ptr();
    let mut TableEntry: PPARTITION_INFORMATION;

    Offset.QuadPart = 0;
    NextOffset.QuadPart = 0;
    ExtendedOffset.QuadPart = 0;

    BufferSize = max(512, SectorSize);

    xHalGetPartialGeometry(DeviceObject, core::ptr::addr_of_mut!(ConventionalCylinders), core::ptr::addr_of_mut!(DiskSize));

    HalExamineMBR(DeviceObject, BufferSize, 0x55, core::ptr::addr_of_mut!(MbrBuffer));
    if !MbrBuffer.is_null() {
        IsEzDrive = 1u8;
        ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
        Offset.QuadPart = 512;
    }

    k = 0;
    while k < 32 {
        if (SectorSize >> k) == 1 {
            break;
        }
        k += 1;
    }

    if (*PartitionBuffer).PartitionCount == 1 {
        if (*PartitionInfo.offset(0)).StartingOffset.QuadPart == 0 &&
           (*PartitionInfo.offset(0)).HiddenSectors == 0 {
            IsSuperFloppy = 1u8;

            if (*PartitionInfo.offset(0)).PartitionNumber != 0 ||
               (*PartitionInfo.offset(0)).PartitionType != 0x06 ||
               (*PartitionInfo.offset(0)).BootIndicator != 0 {
                return 0xC000000Du32 as NTSTATUS;
            }

            if (*PartitionInfo.offset(0)).RewritePartition != 0 {
                DoRewrite = 1u8;
            }
            IsEzDrive = 0u8;
        }
    }

    (*DiskLayout).TableCount = ((*PartitionBuffer).PartitionCount + 4 - 1) / 4;

    Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, PAGE_SIZE, TAG_FILE_SYSTEM) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Entry = Buffer.offset(0x1BE) as PPTE;
    i = 0;
    'outer: loop {
        if i >= (*DiskLayout).TableCount {
            break;
        }

        IsMbr = if i == 0 { 1u8 } else { 0u8 };

        KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);

        Irp = IoBuildSynchronousFsdRequest(
            3,
            DeviceObject,
            Buffer as PVOID,
            BufferSize,
            core::ptr::addr_of_mut!(Offset),
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        IoStackLocation = IoGetNextIrpStackLocation(Irp);
        (*IoStackLocation).Flags |= 0x02;

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == 0x00000103u32 as NTSTATUS {
            KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0, 0u8, core::ptr::null_mut());
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        if IsEzDrive != 0 && Offset.QuadPart == 512 {
            Offset.QuadPart = 0;
        }

        if IsSuperFloppy == 0 {
            *(Buffer.offset(0x1FE) as PUINT16) = 0xAA55u16;

            DoRewrite = 0u8;

            if Offset.QuadPart == 0 {
                if *(Buffer.offset(0x1B8) as PUINT32) != (*PartitionBuffer).Signature {
                    *(Buffer.offset(0x1B8) as PUINT32) = (*PartitionBuffer).Signature;
                    DoRewrite = 1u8;
                }
            }

            PartitionTable = (*DiskLayout).PartitionTable.as_mut_ptr().offset(i as isize);
            j = 0;
            while j < 4 {
                TableEntry = (*PartitionTable).PartitionEntry.as_mut_ptr().offset(j as isize);
                let PartitionType = (*TableEntry).PartitionType;

                if (*TableEntry).RewritePartition != 0 {
                    DoRewrite = 1u8;

                    (*Entry.offset(j as isize)).PartitionType = (*TableEntry).PartitionType;
                    (*Entry.offset(j as isize)).ActiveFlag = if (*TableEntry).BootIndicator != 0 { 0x80 } else { 0 };

                    if PartitionType != 0x00 {
                        if IsMbr != 0 || IsContainerPartition(PartitionType) == 0 {
                            StartOffset.QuadPart = Offset.QuadPart;
                        } else {
                            StartOffset.QuadPart = ExtendedOffset.QuadPart;
                        }

                        let mut SectorOffset: LARGE_INTEGER = core::mem::zeroed();
                        SectorOffset.QuadPart = (*TableEntry).StartingOffset.QuadPart - StartOffset.QuadPart;

                        StartOffset.QuadPart = SectorOffset.QuadPart >> k;
                        (*Entry.offset(j as isize)).StartingSector = StartOffset.u.LowPart;

                        PartitionLength.QuadPart = (*TableEntry).PartitionLength.QuadPart >> k;
                        (*Entry.offset(j as isize)).PartitionLength = PartitionLength.u.LowPart;

                        HalpCalculateChsValues(
                            core::ptr::addr_of_mut!((*TableEntry).StartingOffset),
                            core::ptr::addr_of_mut!((*TableEntry).PartitionLength),
                            k,
                            SectorsPerTrack,
                            NumberOfHeads,
                            ConventionalCylinders,
                            Entry.offset(j as isize) as PPARTITION_DESCRIPTOR,
                        );
                    } else {
                        (*Entry.offset(j as isize)).StartingSector = 0;
                        (*Entry.offset(j as isize)).PartitionLength = 0;
                        (*Entry.offset(j as isize)).StartingTrack = 0;
                        (*Entry.offset(j as isize)).EndingTrack = 0;
                        (*Entry.offset(j as isize)).StartingCylinder = 0;
                        (*Entry.offset(j as isize)).EndingCylinder = 0;
                    }
                }

                if IsContainerPartition(PartitionType) != 0 {
                    NextOffset = (*TableEntry).StartingOffset;
                }

                j += 1;
            }
        }

        if DoRewrite != 0 {
            DoRewrite = 0u8;

            KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);

            if IsEzDrive != 0 && Offset.QuadPart == 0 {
                Offset.QuadPart = 512;
            }

            Irp = IoBuildSynchronousFsdRequest(
                4,
                DeviceObject,
                Buffer as PVOID,
                BufferSize,
                core::ptr::addr_of_mut!(Offset),
                core::ptr::addr_of_mut!(Event),
                core::ptr::addr_of_mut!(IoStatusBlock),
            );
            if Irp.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }

            IoStackLocation = IoGetNextIrpStackLocation(Irp);
            (*IoStackLocation).Flags |= 0x02;

            Status = IoCallDriver(DeviceObject, Irp);
            if Status == 0x00000103u32 as NTSTATUS {
                KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0, 0u8, core::ptr::null_mut());
                Status = IoStatusBlock.__bindgen_anon_1.Status;
            }

            if !NT_SUCCESS(Status) {
                break;
            }

            if IsEzDrive != 0 && Offset.QuadPart == 512 {
                Offset.QuadPart = 0;
            }
        }

        Offset = NextOffset;
        if IsMbr != 0 {
            ExtendedOffset = NextOffset;
        }

        i += 1;
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer as PVOID, TAG_FILE_SYSTEM);
    }
    Status
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn xHalIoWritePartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    SectorsPerTrack: ULONG,
    NumberOfHeads: ULONG,
    PartitionBuffer: PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let BufferSize: ULONG;
    let mut Buffer: PUCHAR;
    let Entry: PPTE;
    let mut PartitionTable: PPARTITION_TABLE;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut NextOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ExtendedOffset: LARGE_INTEGER = core::mem::zeroed();
    let SectorOffset: LARGE_INTEGER;
    let mut StartOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut PartitionLength: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: i8;
    let mut IsEzDrive: BOOLEAN = 0u8;
    let mut IsSuperFloppy: BOOLEAN = 0u8;
    let mut DoRewrite: BOOLEAN = 0u8;
    let mut IsMbr: BOOLEAN;
    let mut ConventionalCylinders: ULONG = 0;
    let mut DiskSize: LONGLONG = 0;
    let DiskLayout: PDISK_LAYOUT = PartitionBuffer as PDISK_LAYOUT;
    let mut MbrBuffer: PVOID = core::ptr::null_mut();
    let PartitionType: UCHAR;
    let mut IoStackLocation: PIO_STACK_LOCATION;
    let PartitionInfo: PPARTITION_INFORMATION = (*PartitionBuffer).PartitionEntry.as_mut_ptr();
    let mut TableEntry: PPARTITION_INFORMATION;

    Offset.QuadPart = 0;
    NextOffset.QuadPart = 0;
    ExtendedOffset.QuadPart = 0;

    BufferSize = max(512, SectorSize);

    xHalGetPartialGeometry(DeviceObject, core::ptr::addr_of_mut!(ConventionalCylinders), core::ptr::addr_of_mut!(DiskSize));

    HalExamineMBR(DeviceObject, BufferSize, 0x55, core::ptr::addr_of_mut!(MbrBuffer));
    if !MbrBuffer.is_null() {
        IsEzDrive = 1u8;
        ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
        Offset.QuadPart = 512;
    }

    k = 0;
    while k < 32 {
        if (SectorSize >> k) == 1 {
            break;
        }
        k += 1;
    }

    if (*PartitionBuffer).PartitionCount == 1 {
        if (*PartitionInfo.offset(0)).StartingOffset.QuadPart == 0 &&
           (*PartitionInfo.offset(0)).HiddenSectors == 0 {
            IsSuperFloppy = 1u8;

            if (*PartitionInfo.offset(0)).PartitionNumber != 0 ||
               (*PartitionInfo.offset(0)).PartitionType != 0x06 ||
               (*PartitionInfo.offset(0)).BootIndicator != 0 {
                return 0xC000000Du32 as NTSTATUS;
            }

            if (*PartitionInfo.offset(0)).RewritePartition != 0 {
                DoRewrite = 1u8;
            }
            IsEzDrive = 0u8;
        }
    }

    (*DiskLayout).TableCount = ((*PartitionBuffer).PartitionCount + 4 - 1) / 4;

    Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, PAGE_SIZE, TAG_FILE_SYSTEM) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Entry = Buffer.offset(0x1BE) as PPTE;
    i = 0;
    'outer: loop {
        if i >= (*DiskLayout).TableCount {
            break;
        }

        IsMbr = if i == 0 { 1u8 } else { 0u8 };

        KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);

        Irp = IoBuildSynchronousFsdRequest(
            3,
            DeviceObject,
            Buffer as PVOID,
            BufferSize,
            core::ptr::addr_of_mut!(Offset),
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        IoStackLocation = IoGetNextIrpStackLocation(Irp);
        (*IoStackLocation).Flags |= 0x02;

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == 0x00000103u32 as NTSTATUS {
            KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0, 0u8, core::ptr::null_mut());
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        if IsEzDrive != 0 && Offset.QuadPart == 512 {
            Offset.QuadPart = 0;
        }

        if IsSuperFloppy == 0 {
            *(Buffer.offset(0x1FE) as PUINT16) = 0xAA55u16;

            DoRewrite = 0u8;

            if Offset.QuadPart == 0 {
                if *(Buffer.offset(0x1B8) as PUINT32) != (*PartitionBuffer).Signature {
                    *(Buffer.offset(0x1B8) as PUINT32) = (*PartitionBuffer).Signature;
                    DoRewrite = 1u8;
                }
            }

            PartitionTable = (*DiskLayout).PartitionTable.as_mut_ptr().offset(i as isize);
            j = 0;
            while j < 4 {
                TableEntry = (*PartitionTable).PartitionEntry.as_mut_ptr().offset(j as isize);
                let PartitionType = (*TableEntry).PartitionType;

                if (*TableEntry).RewritePartition != 0 {
                    DoRewrite = 1u8;

                    (*Entry.offset(j as isize)).PartitionType = (*TableEntry).PartitionType;
                    (*Entry.offset(j as isize)).ActiveFlag = if (*TableEntry).BootIndicator != 0 { 0x80 } else { 0 };

                    if PartitionType != 0x00 {
                        if IsMbr != 0 || IsContainerPartition(PartitionType) == 0 {
                            StartOffset.QuadPart = Offset.QuadPart;
                        } else {
                            StartOffset.QuadPart = ExtendedOffset.QuadPart;
                        }

                        let mut SectorOffset: LARGE_INTEGER = core::mem::zeroed();
                        SectorOffset.QuadPart = (*TableEntry).StartingOffset.QuadPart - StartOffset.QuadPart;

                        StartOffset.QuadPart = SectorOffset.QuadPart >> k;
                        (*Entry.offset(j as isize)).StartingSector = StartOffset.u.LowPart;

                        PartitionLength.QuadPart = (*TableEntry).PartitionLength.QuadPart >> k;
                        (*Entry.offset(j as isize)).PartitionLength = PartitionLength.u.LowPart;

                        HalpCalculateChsValues(
                            core::ptr::addr_of_mut!((*TableEntry).StartingOffset),
                            core::ptr::addr_of_mut!((*TableEntry).PartitionLength),
                            k,
                            SectorsPerTrack,
                            NumberOfHeads,
                            ConventionalCylinders,
                            Entry.offset(j as isize) as PPARTITION_DESCRIPTOR,
                        );
                    } else {
                        (*Entry.offset(j as isize)).StartingSector = 0;
                        (*Entry.offset(j as isize)).PartitionLength = 0;
                        (*Entry.offset(j as isize)).StartingTrack = 0;
                        (*Entry.offset(j as isize)).EndingTrack = 0;
                        (*Entry.offset(j as isize)).StartingCylinder = 0;
                        (*Entry.offset(j as isize)).EndingCylinder = 0;
                    }
                }

                if IsContainerPartition(PartitionType) != 0 {
                    NextOffset = (*TableEntry).StartingOffset;
                }

                j += 1;
            }
        }

        if DoRewrite != 0 {
            DoRewrite = 0u8;

            KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);

            if IsEzDrive != 0 && Offset.QuadPart == 0 {
                Offset.QuadPart = 512;
            }

            Irp = IoBuildSynchronousFsdRequest(
                4,
                DeviceObject,
                Buffer as PVOID,
                BufferSize,
                core::ptr::addr_of_mut!(Offset),
                core::ptr::addr_of_mut!(Event),
                core::ptr::addr_of_mut!(IoStatusBlock),
            );
            if Irp.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }

            IoStackLocation = IoGetNextIrpStackLocation(Irp);
            (*IoStackLocation).Flags |= 0x02;

            Status = IoCallDriver(DeviceObject, Irp);
            if Status == 0x00000103u32 as NTSTATUS {
                KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0, 0u8, core::ptr::null_mut());
                Status = IoStatusBlock.__bindgen_anon_1.Status;
            }

            if !NT_SUCCESS(Status) {
                break;
            }

            if IsEzDrive != 0 && Offset.QuadPart == 512 {
                Offset.QuadPart = 0;
            }
        }

        Offset = NextOffset;
        if IsMbr != 0 {
            ExtendedOffset = NextOffset;
        }

        i += 1;
    }

    if !Buffer.is_null() {
        ExFreePoolWithTag(Buffer as PVOID, TAG_FILE_SYSTEM);
    }
    Status
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalExamineMBR(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    MbrTypeIdentifier: ULONG,
    MbrBuffer: *mut PVOID,
) {
    (((*HALDISPATCH()).HalExamineMBR).unwrap())(
        DeviceObject,
        SectorSize,
        MbrTypeIdentifier,
        MbrBuffer,
    );
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn HalExamineMBR(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    MbrTypeIdentifier: ULONG,
    MbrBuffer: *mut PVOID,
) {
    (((*HALDISPATCH()).HalExamineMBR).unwrap())(
        DeviceObject,
        SectorSize,
        MbrTypeIdentifier,
        MbrBuffer,
    );
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IoReadPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    ReturnRecognizedPartitions: BOOLEAN,
    PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    (((*HALDISPATCH()).HalIoReadPartitionTable).unwrap())(
        DeviceObject,
        SectorSize,
        ReturnRecognizedPartitions,
        PartitionBuffer,
    )
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IoReadPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    ReturnRecognizedPartitions: BOOLEAN,
    PartitionBuffer: *mut PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    (((*HALDISPATCH()).HalIoReadPartitionTable).unwrap())(
        DeviceObject,
        SectorSize,
        ReturnRecognizedPartitions,
        PartitionBuffer,
    )
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IoSetPartitionInformation(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    PartitionNumber: ULONG,
    PartitionType: ULONG,
) -> NTSTATUS {
    ((*HALDISPATCH()).HalIoSetPartitionInformation.unwrap())(
        DeviceObject,
        SectorSize,
        PartitionNumber,
        PartitionType,
    )
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IoSetPartitionInformation(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    PartitionNumber: ULONG,
    PartitionType: ULONG,
) -> NTSTATUS {
    ((*HALDISPATCH()).HalIoSetPartitionInformation.unwrap())(
        DeviceObject,
        SectorSize,
        PartitionNumber,
        PartitionType,
    )
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IoWritePartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    SectorsPerTrack: ULONG,
    NumberOfHeads: ULONG,
    PartitionBuffer: PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    ((*HALDISPATCH()).HalIoWritePartitionTable.unwrap())(
        DeviceObject,
        SectorSize,
        SectorsPerTrack,
        NumberOfHeads,
        PartitionBuffer,
    )
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IoWritePartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    SectorsPerTrack: ULONG,
    NumberOfHeads: ULONG,
    PartitionBuffer: PDRIVE_LAYOUT_INFORMATION,
) -> NTSTATUS {
    ((*HALDISPATCH()).HalIoWritePartitionTable.unwrap())(
        DeviceObject,
        SectorSize,
        SectorsPerTrack,
        NumberOfHeads,
        PartitionBuffer,
    )
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IoAssignDriveLetters(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    NtDeviceName: PSTRING,
    NtSystemPath: PUCHAR,
    NtSystemPathString: PSTRING,
) {
    ((*HALDISPATCH()).HalIoAssignDriveLetters.unwrap())(
        LoaderBlock,
        NtDeviceName,
        NtSystemPath,
        NtSystemPathString,
    );
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IoAssignDriveLetters(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    NtDeviceName: PSTRING,
    NtSystemPath: PUCHAR,
    NtSystemPathString: PSTRING,
) {
    ((*HALDISPATCH()).HalIoAssignDriveLetters.unwrap())(
        LoaderBlock,
        NtDeviceName,
        NtSystemPath,
        NtSystemPathString,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/fstub/disksup.c
// * PURPOSE:         I/O HAL Routines for Disk Access
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
// *                  Eric Kohl
// *                  Casper S. Hornstrup (chorns@users.sourceforge.net)
// *                  Pierre Schweitzer
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
// #include <internal/hal.h>
//
// const WCHAR DiskMountString[] = L"\\DosDevices\\%C:";
//
// #define AUTO_DRIVE         MAXULONG
//
// #define PARTITION_MAGIC    0xaa55
//
// #define EFI_PMBR_OSTYPE_EFI 0xEE
//
// #include <pshpack1.h>
//
// typedef struct _REG_DISK_MOUNT_INFO
// {
//     ULONG Signature;
//     LARGE_INTEGER StartingOffset;
// } REG_DISK_MOUNT_INFO, *PREG_DISK_MOUNT_INFO;
//
// #include <poppack.h>
//
// typedef enum _DISK_MANAGER
// {
//     NoDiskManager,
//     OntrackDiskManager,
//     EZ_Drive
// } DISK_MANAGER;
//
// typedef enum _PARTITION_TYPE
// {
//     BootablePartition,
//     PrimaryPartition,
//     LogicalPartition,
//     FtPartition,
//     UnknownPartition,
//     DataPartition
// } PARTITION_TYPE, *PPARTITION_TYPE;
//
// NTSTATUS
// FASTCALL
// HalpQueryDriveLayout(IN PUNICODE_STRING DeviceName,
//                      OUT PDRIVE_LAYOUT_INFORMATION *LayoutInfo)
// {
//     IO_STATUS_BLOCK StatusBlock;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     PFILE_OBJECT FileObject;
//     KEVENT Event;
//     PIRP Irp;
//     NTSTATUS Status;
//     ULONG BufferSize;
//     PDRIVE_LAYOUT_INFORMATION Buffer;
//     PAGED_CODE();
//
//     /* Get device pointers */
//     Status = IoGetDeviceObjectPointer(DeviceName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Get attached device object */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//     ObDereferenceObject(FileObject);
//
//     /* Do not handle removable media */
//     if (BooleanFlagOn(DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA))
//     {
//         ObDereferenceObject(DeviceObject);
//         return STATUS_NO_MEDIA;
//     }
//
//     /* We'll loop until our buffer is big enough */
//     Buffer = NULL;
//     BufferSize = 0x1000;
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     do
//     {
//         /* If we already had a buffer, it means it's not big
//          * enough, so free and multiply size by two
//          */
//         if (Buffer != NULL)
//         {
//             ExFreePoolWithTag(Buffer, TAG_FSTUB);
//             BufferSize *= 2;
//         }
//
//         /* Allocate buffer for output buffer */
//         Buffer = ExAllocatePoolWithTag(NonPagedPool, BufferSize, TAG_FSTUB);
//         if (Buffer == NULL)
//         {
//             Status = STATUS_NO_MEMORY;
//             break;
//         }
//
//         /* Build the IRP to query drive layout */
//         Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_DRIVE_LAYOUT,
//                                             DeviceObject,
//                                             NULL,
//                                             0,
//                                             Buffer,
//                                             BufferSize,
//                                             FALSE,
//                                             &Event,
//                                             &StatusBlock);
//         if (Irp == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//         }
//
//         /* Call the driver and wait if appropriate */
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&Event, //                                   Executive, //                                   KernelMode, //                                   FALSE, //                                   NULL);
//             Status = StatusBlock.Status;
//         }
//         /* If buffer is too small, keep looping */
//     } while (Status == STATUS_BUFFER_TOO_SMALL);
//
//     /* We're done with the device */
//     ObDereferenceObject(DeviceObject);
//
//     /* If querying worked, then return the buffer to the caller */
//     if (NT_SUCCESS(Status))
//     {
//         ASSERT(Buffer != NULL);
//         *LayoutInfo = Buffer;
//     }
//     /* Else, release the buffer if still allocated and fail */
//     else
//     {
//         if (Buffer != NULL)
//         {
//             ExFreePoolWithTag(Buffer, TAG_FSTUB);
//         }
//     }
//
//     return Status;
// }
//
// NTSTATUS
// HalpQueryPartitionType(IN PUNICODE_STRING DeviceName,
//                        IN PDRIVE_LAYOUT_INFORMATION LayoutInfo,
//                        OUT PPARTITION_TYPE PartitionType)
// {
//     USHORT i;
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PARTITION_INFORMATION_EX PartitionInfo;
//
//     PAGED_CODE();
//
//     /* Get device pointers */
//     Status = IoGetDeviceObjectPointer(DeviceName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Get attached device object */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//     ObDereferenceObject(FileObject);
//
//     /* Assume logical partition for removable devices */
//     if (BooleanFlagOn(DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA))
//     {
//         ObDereferenceObject(DeviceObject);
//         *PartitionType = LogicalPartition;
//         return STATUS_SUCCESS;
//     }
//
//     /* For the others, query partition info */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_PARTITION_INFO_EX,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         &PartitionInfo,
//                                         sizeof(PartitionInfo),
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (Irp == NULL)
//     {
//         ObDereferenceObject(DeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, //                               Executive, //                               KernelMode, //                               FALSE, //                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* We're done with the device */
//     ObDereferenceObject(DeviceObject);
//
//     /* If we failed querying partition info, try to return something
//      * if caller didn't provide a precise layout, assume logical
//      * partition and fake success. Otherwise, just fail.
//      */
//     if (!NT_SUCCESS(Status))
//     {
//         if (LayoutInfo == NULL)
//         {
//             *PartitionType = LogicalPartition;
//             return STATUS_SUCCESS;
//         }
//
//         return Status;
//     }
//
//     /* First, handle non MBR style (easy cases) */
//     if (PartitionInfo.PartitionStyle != PARTITION_STYLE_MBR)
//     {
//         /* If not GPT, we don't know what it is */
//         if (PartitionInfo.PartitionStyle != PARTITION_STYLE_GPT)
//         {
//             *PartitionType = UnknownPartition;
//             return STATUS_SUCCESS;
//         }
//
//         /* Check whether that's data partition */
//         if (RtlCompareMemory(&PartitionInfo.Gpt.PartitionType,
//                              &PARTITION_BASIC_DATA_GUID,
//                              sizeof(GUID)) == sizeof(GUID))
//         {
//             *PartitionType = DataPartition;
//             return STATUS_SUCCESS;
//         }
//
//         /* Otherwise, we don't know */
//         *PartitionType = UnknownPartition;
//         return STATUS_SUCCESS;
//     }
//
//     /* If we don't recognize partition type, return unknown */
//     if (!IsRecognizedPartition(PartitionInfo.Mbr.PartitionType))
//     {
//         *PartitionType = UnknownPartition;
//         return STATUS_SUCCESS;
//     }
//
//     /* Check if that's a FT volume */
//     if (IsFTPartition(PartitionInfo.Mbr.PartitionType))
//     {
//         *PartitionType = FtPartition;
//         return STATUS_SUCCESS;
//     }
//
//     /* If the caller didn't provide the complete layout, just return */
//     if (LayoutInfo == NULL)
//     {
//         *PartitionType = LogicalPartition;
//         return STATUS_SUCCESS;
//     }
//
//     /* Now, evaluate the partition to the 4 in the input layout */
//     for (i = 0; i < 4; ++i)
//     {
//         /* If we find a partition matching */
//         if (LayoutInfo->PartitionEntry[i].StartingOffset.QuadPart == PartitionInfo.StartingOffset.QuadPart)
//         {
//             /* Return boot if boot flag is set */
//             if (PartitionInfo.Mbr.BootIndicator)
//             {
//                 *PartitionType = BootablePartition;
//             }
//             /* Primary otherwise */
//             else
//             {
//                 *PartitionType = PrimaryPartition;
//             }
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Otherwise, assume logical */
//     *PartitionType = LogicalPartition;
//     return STATUS_SUCCESS;
// }
//
// PULONG
// IopComputeHarddiskDerangements(IN ULONG DiskCount)
// {
//     PIRP Irp;
//     KEVENT Event;
//     ULONG i, j, k;
//     PULONG Devices;
//     NTSTATUS Status;
//     WCHAR Buffer[100];
//     UNICODE_STRING ArcName;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     STORAGE_DEVICE_NUMBER DeviceNumber;
//
//     /* No disks, nothing to do */
//     if (DiskCount == 0)
//     {
//         return NULL;
//     }
//
//     /* Allocate a buffer big enough to hold all the disks */
//     Devices = ExAllocatePoolWithTag(PagedPool | POOL_COLD_ALLOCATION,
//                                     sizeof(ULONG) * DiskCount,
//                                     TAG_FSTUB);
//     if (Devices == NULL)
//     {
//         return NULL;
//     }
//
//     /* Now, we'll query all the disks */
//     for (i = 0; i < DiskCount; ++i)
//     {
//         /* Using their ARC name */
//         swprintf(Buffer, L"\\ArcName\\multi(0)disk(0)rdisk(%d)", i as u32);
//         RtlInitUnicodeString(&ArcName, Buffer);
//         /* Get the attached DeviceObject */
//         if (NT_SUCCESS(IoGetDeviceObjectPointer(&ArcName, FILE_READ_ATTRIBUTES, &FileObject, &DeviceObject)))
//         {
//             DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//             ObDereferenceObject(FileObject);
//
//             /* And query it for device number */
//             KeInitializeEvent(&Event, NotificationEvent, FALSE);
//             Irp = IoBuildDeviceIoControlRequest(IOCTL_STORAGE_GET_DEVICE_NUMBER,
//                                                 DeviceObject,
//                                                 NULL,
//                                                 0,
//                                                 &DeviceNumber,
//                                                 sizeof(DeviceNumber),
//                                                 FALSE,
//                                                 &Event,
//                                                 &IoStatusBlock);
//             if (Irp != NULL)
//             {
//                 Status = IoCallDriver(DeviceObject, Irp);
//                 if (Status == STATUS_PENDING)
//                 {
//                     KeWaitForSingleObject(&Event, //                                           Executive, //                                           KernelMode, //                                           FALSE, //                                           NULL);
//                     Status = IoStatusBlock.Status;
//                 }
//
//                 ObDereferenceObject(DeviceObject);
//
//                 /* In case of a success remember device number */
//                 if (NT_SUCCESS(Status))
//                 {
//                     Devices[i] = DeviceNumber.DeviceNumber;
//                     /* Move on, not to fall into our default case */
//                     continue;
//                 }
//             }
//             else
//             {
//                 ObDereferenceObject(DeviceObject);
//             }
//
//             /* Default case, for failures, set -1 */
//             Devices[i] = -1;
//         }
//     }
//
//     /* Now, we'll check all device numbers */
//     for (i = 0; i < DiskCount; ++i)
//     {
//         /* First of all, check if we're at the right place */
//         for (j = 0; j < DiskCount; ++j)
//         {
//             if (Devices[j] == i)
//             {
//                 break;
//             }
//         }
//
//         /* If not, perform the change */
//         if (j >= DiskCount)
//         {
//             k = 0;
//             while (Devices[k] != -1)
//             {
//                 if (++k >= DiskCount)
//                 {
//                     break;
//                 }
//             }
//
//             if (k < DiskCount)
//             {
//                 Devices[k] = i;
//             }
//         }
//     }
//
//     /* Return our device derangement map */
//     return Devices;
// }
//
// NTSTATUS
// HalpNextMountLetter(IN PUNICODE_STRING DeviceName,
//                     OUT PUCHAR DriveLetter)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     UNICODE_STRING MountMgr;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PMOUNTMGR_DRIVE_LETTER_TARGET Target;
//     MOUNTMGR_DRIVE_LETTER_INFORMATION LetterInfo;
//
//     /* To get next mount letter, we need the MountMgr */
//     RtlInitUnicodeString(&MountMgr, L"\\Device\\MountPointManager");
//     Status = IoGetDeviceObjectPointer(&MountMgr,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Allocate our input buffer */
//     Target = ExAllocatePoolWithTag(PagedPool,
//                                    DeviceName->Length + FIELD_OFFSET(MOUNTMGR_DRIVE_LETTER_TARGET, DeviceName),
//                                    TAG_FSTUB);
//     if (Target == NULL)
//     {
//         ObDereferenceObject(FileObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* And fill it with the device hat needs a drive letter */
//     Target->DeviceNameLength = DeviceName->Length;
//     RtlCopyMemory(&Target->DeviceName[0], DeviceName->Buffer, DeviceName->Length);
//
//     /* Call the mount manager */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTMGR_NEXT_DRIVE_LETTER,
//                                         DeviceObject,
//                                         Target,
//                                         DeviceName->Length + FIELD_OFFSET(MOUNTMGR_DRIVE_LETTER_TARGET, DeviceName),
//                                         &LetterInfo,
//                                         sizeof(LetterInfo),
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (Irp == NULL)
//     {
//         ExFreePoolWithTag(Target, TAG_FSTUB);
//         ObDereferenceObject(FileObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, //                               Executive, //                               KernelMode, //                               FALSE, //                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     ExFreePoolWithTag(Target, TAG_FSTUB);
//     ObDereferenceObject(FileObject);
//
//     DPRINT("Done: %d %c\n", LetterInfo.DriveLetterWasAssigned,
//                             LetterInfo.CurrentDriveLetter);
//
//     /* Return the drive letter the MountMgr potentially assigned */
//     *DriveLetter = LetterInfo.CurrentDriveLetter;
//
//     /* Also return the success */
//     return Status;
// }
//
// NTSTATUS
// HalpSetMountLetter(IN PUNICODE_STRING DeviceName,
//                    UCHAR DriveLetter)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     WCHAR Buffer[30];
//     ULONG InputBufferLength;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING DosDevice, MountMgr;
//     PMOUNTMGR_CREATE_POINT_INPUT InputBuffer;
//
//     /* Setup the DosDevice name */
//     swprintf(Buffer, L"\\DosDevices\\%c:", DriveLetter);
//     RtlInitUnicodeString(&DosDevice, Buffer);
//
//     /* Allocate the input buffer for the MountMgr */
//     InputBufferLength = DosDevice.Length + DeviceName->Length + sizeof(MOUNTMGR_CREATE_POINT_INPUT);
//     InputBuffer = ExAllocatePoolWithTag(PagedPool, InputBufferLength, TAG_FSTUB);
//     if (InputBuffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Fill the input buffer */
//     InputBuffer->SymbolicLinkNameOffset = sizeof(MOUNTMGR_CREATE_POINT_INPUT);
//     InputBuffer->SymbolicLinkNameLength = DosDevice.Length;
//     InputBuffer->DeviceNameOffset = DosDevice.Length + sizeof(MOUNTMGR_CREATE_POINT_INPUT);
//     InputBuffer->DeviceNameLength = DeviceName->Length;
//     RtlCopyMemory(&InputBuffer[1], DosDevice.Buffer, DosDevice.Length);
//     RtlCopyMemory((PVOID)((ULONG_PTR)InputBuffer + InputBuffer->DeviceNameOffset),
//                   DeviceName->Buffer,
//                   DeviceName->Length);
//
//     /* Get the MountMgr device pointer, to send the IOCTL */
//     RtlInitUnicodeString(&MountMgr, L"\\Device\\MountPointManager");
//     Status = IoGetDeviceObjectPointer(&MountMgr,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(InputBuffer, TAG_FSTUB);
//         return Status;
//     }
//
//     /* Call the MountMgr */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTMGR_CREATE_POINT,
//                                         DeviceObject,
//                                         InputBuffer,
//                                         InputBufferLength,
//                                         NULL,
//                                         0,
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (Irp == NULL)
//     {
//         ObDereferenceObject(FileObject);
//         ExFreePoolWithTag(InputBuffer, TAG_FSTUB);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, //                               Executive, //                               KernelMode, //                               FALSE, //                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     ObDereferenceObject(FileObject);
//     ExFreePoolWithTag(InputBuffer, TAG_FSTUB);
//
//     /* Return the MountMgr status */
//     return Status;
// }
//
// UCHAR
// HalpNextDriveLetter(IN PUNICODE_STRING DeviceName,
//                     IN PSTRING NtDeviceName,
//                     OUT PUCHAR NtSystemPath,
//                     BOOLEAN IsRemovable)
// {
//     UCHAR i;
//     WCHAR Buffer[40];
//     UCHAR DriveLetter;
//     UNICODE_STRING FloppyString, CdString, NtDeviceNameU, DosDevice;
//
//     /* Quick path, ask directly the mount manager to assign the next
//      * free drive letter
//      */
//     if (NT_SUCCESS(HalpNextMountLetter(DeviceName, &DriveLetter)))
//     {
//         return DriveLetter;
//     }
//
//     /* We'll allow MountMgr to fail only for non vital path */
//     if (NtDeviceName == NULL || NtSystemPath == NULL)
//     {
//         return -1;
//     }
//
//     /* And for removable devices */
//     if (!IsRemovable)
//     {
//         return 0;
//     }
//
//     /* Removable might be floppy or cdrom */
//     RtlInitUnicodeString(&FloppyString, L"\\Device\\Floppy");
//     RtlInitUnicodeString(&CdString, L"\\Device\\CdRom");
//
//     /* If floppy, start at A */
//     if (RtlPrefixUnicodeString(&FloppyString, DeviceName, TRUE))
//     {
//         DriveLetter = 'A';
//     }
//     /* If CD start C */
//     else if (RtlPrefixUnicodeString(&CdString, DeviceName, TRUE))
//     {
//         DriveLetter = 'D';
//     }
//     /* For the rest start at C */
//     else
//     {
//         DriveLetter = 'C';
//     }
//
//     /* Now, try to assign a drive letter manually with the MountMgr */
//     for (i = DriveLetter; i <= 'Z'; ++i)
//     {
//         if (NT_SUCCESS(HalpSetMountLetter(DeviceName, i)))
//         {
//             /* If it worked, if we were managing system path, update manually */
//             if (NT_SUCCESS(RtlAnsiStringToUnicodeString(&NtDeviceNameU, NtDeviceName, TRUE)))
//             {
//                 if (RtlEqualUnicodeString(&NtDeviceNameU, DeviceName, TRUE))
//                 {
//                     *NtSystemPath = i;
//                 }
//
//                 RtlFreeUnicodeString(&NtDeviceNameU);
//             }
//
//             return i;
//         }
//     }
//
//     /* Last fall back, we're not on a PnP device... */
//     for (i = DriveLetter; i <= 'Z'; ++i)
//     {
//         /* We'll link manually, without MountMgr knowing anything about the device */
//         swprintf(Buffer, L"\\DosDevices\\%c:", i as u32);
//         RtlInitUnicodeString(&DosDevice, Buffer);
//
//         /* If linking worked, then the letter was free ;-) */
//         if (NT_SUCCESS(IoCreateSymbolicLink(&DosDevice, DeviceName)))
//         {
//             /* If it worked, if we were managing system path, update manually */
//             if (NT_SUCCESS(RtlAnsiStringToUnicodeString(&NtDeviceNameU, NtDeviceName, TRUE)))
//             {
//                 if (RtlEqualUnicodeString(&NtDeviceNameU, DeviceName, TRUE))
//                 {
//                     *NtSystemPath = i;
//                 }
//
//                 RtlFreeUnicodeString(&NtDeviceNameU);
//             }
//
//             return i;
//         }
//     }
//
//     /* We're done, nothing happened */
//     return 0;
// }
//
// BOOLEAN
// HalpIsOldStyleFloppy(PUNICODE_STRING DeviceName)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     MOUNTDEV_NAME DevName;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PAGED_CODE();
//
//     /* Get the attached device object to our device */
//     if (!NT_SUCCESS(IoGetDeviceObjectPointer(DeviceName,
//                                              FILE_READ_ATTRIBUTES,
//                                              &FileObject,
//                                              &DeviceObject)))
//     {
//         return FALSE;
//     }
//
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//     ObDereferenceObject(FileObject);
//
//     /* Query its device name (ie, check floppy.sys implements MountMgr interface) */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         &DevName,
//                                         sizeof(DevName),
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (Irp == NULL)
//     {
//         ObDereferenceObject(DeviceObject);
//         return FALSE;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, //                               Executive, //                               KernelMode, //                               FALSE, //                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* If status is not STATUS_BUFFER_OVERFLOW, it means
//      * it's pre-mountmgr driver, aka "Old style".
//      */
//     ObDereferenceObject(DeviceObject);
//     return (Status != STATUS_BUFFER_OVERFLOW);
// }
//
// NTSTATUS
// HalpDeleteMountLetter(UCHAR DriveLetter)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     WCHAR Buffer[30];
//     ULONG InputBufferLength;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PMOUNTMGR_MOUNT_POINT InputBuffer;
//     UNICODE_STRING DosDevice, MountMgr;
//     PMOUNTMGR_MOUNT_POINTS OutputBuffer;
//
//     /* Setup the device name of the letter to delete */
//     swprintf(Buffer, L"\\DosDevices\\%c:", DriveLetter);
//     RtlInitUnicodeString(&DosDevice, Buffer);
//
//     /* Allocate the input buffer for MountMgr */
//     InputBufferLength = DosDevice.Length + sizeof(MOUNTMGR_MOUNT_POINT);
//     InputBuffer = ExAllocatePoolWithTag(PagedPool, InputBufferLength, TAG_FSTUB);
//     if (InputBuffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Fill it in */
//     RtlZeroMemory(InputBuffer, InputBufferLength);
//     InputBuffer->SymbolicLinkNameOffset = sizeof(MOUNTMGR_MOUNT_POINT);
//     InputBuffer->SymbolicLinkNameLength = DosDevice.Length;
//     RtlCopyMemory(&InputBuffer[1], DosDevice.Buffer, DosDevice.Length);
//
//     /* Allocate big enough output buffer (we don't care about the output) */
//     OutputBuffer = ExAllocatePoolWithTag(PagedPool, 0x1000, TAG_FSTUB);
//     if (OutputBuffer == NULL)
//     {
//         ExFreePoolWithTag(InputBuffer, TAG_FSTUB);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Get the device pointer to the MountMgr */
//     RtlInitUnicodeString(&MountMgr, L"\\Device\\MountPointManager");
//     Status = IoGetDeviceObjectPointer(&MountMgr,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(OutputBuffer, TAG_FSTUB);
//         ExFreePoolWithTag(InputBuffer, TAG_FSTUB);
//         return Status;
//     }
//
//     /* Call the mount manager to delete the drive letter */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTMGR_DELETE_POINTS,
//                                         DeviceObject,
//                                         InputBuffer,
//                                         InputBufferLength,
//                                         OutputBuffer,
//                                         0x1000,
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (Irp == NULL)
//     {
//         ObDereferenceObject(FileObject);
//         ExFreePoolWithTag(OutputBuffer, TAG_FSTUB);
//         ExFreePoolWithTag(InputBuffer, TAG_FSTUB);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, //                               Executive, //                               KernelMode, //                               FALSE, //                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     ObDereferenceObject(FileObject);
//     ExFreePoolWithTag(OutputBuffer, TAG_FSTUB);
//     ExFreePoolWithTag(InputBuffer, TAG_FSTUB);
//
//     return Status;
// }
//
// VOID
// HalpEnableAutomaticDriveLetterAssignment(VOID)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     UNICODE_STRING MountMgr;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Get the device pointer to the MountMgr */
//     RtlInitUnicodeString(&MountMgr, L"\\Device\\MountPointManager");
//     Status = IoGetDeviceObjectPointer(&MountMgr,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return;
//     }
//
//     /* Just send an IOCTL to enable the feature */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_MOUNTMGR_AUTO_DL_ASSIGNMENTS,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (Irp == NULL)
//     {
//         return;
//     }
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, //                               Executive, //                               KernelMode, //                               FALSE, //                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     ObDereferenceObject(FileObject);
//
//     return;
// }
//
// VOID
// FASTCALL
// xHalIoAssignDriveLetters(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                          IN PSTRING NtDeviceName,
//                          OUT PUCHAR NtSystemPath,
//                          OUT PSTRING NtSystemPathString)
// {
//     USHORT i;
//     PULONG Devices;
//     NTSTATUS Status;
//     WCHAR Buffer[50];
//     HANDLE FileHandle;
//     UCHAR DriveLetter;
//     BOOLEAN SystemFound;
//     IO_STATUS_BLOCK StatusBlock;
//     PARTITION_TYPE PartitionType;
//     ANSI_STRING StringA1, StringA2;
//     PSTR Buffer1, Buffer2, LoadOptions;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PDRIVE_LAYOUT_INFORMATION LayoutInfo;
//     PCONFIGURATION_INFORMATION ConfigInfo;
//     UNICODE_STRING StringU1, StringU2, StringU3;
//     ULONG Increment, DiskCount, RealDiskCount, HarddiskCount, PartitionCount, SystemPartition;
//
//     PAGED_CODE();
//
//     /* Get our disk count */
//     ConfigInfo = IoGetConfigurationInformation();
//     DiskCount = ConfigInfo->DiskCount;
//     RealDiskCount = 0;
//
//     /* Allocate two generic string buffers we'll use and reuser later on */
//     Buffer1 = ExAllocatePoolWithTag(NonPagedPool, 128, TAG_FSTUB);
//     Buffer2 = ExAllocatePoolWithTag(NonPagedPool, 64, TAG_FSTUB);
//     if (Buffer1 == NULL || Buffer2 == NULL)
//     {
//         KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
//     }
//
//     /* In case of a remote boot, setup system path */
//     if (IoRemoteBootClient)
//     {
//         PSTR Last, Saved;
//
//         /* Find last \ */
//         Last = strrchr(LoaderBlock->NtBootPathName, '\\');
//         Saved = NULL;
//         /* Misformed name, fail */
//         if (Last == NULL)
//         {
//             KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
//         }
//
//         /* In case the name was terminated by a \... */
//         if (Last[1] == ANSI_NULL)
//         {
//             /* Erase it, save position and find the previous \ */
//             *Last = ANSI_NULL;
//             Saved = Last;
//             Last = strrchr(LoaderBlock->NtBootPathName, '\\');
//             *Saved = '\\';
//         }
//
//         /* Misformed name, fail */
//         if (Last == NULL)
//         {
//             KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
//         }
//
//         /* For a remote boot, assign X drive letter */
//         NtSystemPath[0] = 'X';
//         NtSystemPath[1] = ':';
//         /* And copy the end of the boot path */
//         strcpy((PSTR)&NtSystemPath[2], Last);
//
//         /* If we had to remove the trailing \, remove it here too */
//         if (Saved != NULL)
//         {
//             NtSystemPath[strlen((PSTR)NtSystemPath) - 1] = ANSI_NULL;
//         }
//
//         /* Setup output string */
//         RtlInitString(NtSystemPathString, (PSTR)NtSystemPath);
//     }
//
//     /* For each of our disks, create the physical device DOS device */
//     Increment = 0;
//     if (DiskCount != 0)
//     {
//         for (i = 0; i < DiskCount; ++i)
//         {
//             /* Setup the origin name */
//             sprintf(Buffer1, "\\Device\\Harddisk%d\\Partition%d", i as u32, 0);
//             RtlInitAnsiString(&StringA1, Buffer1);
//             if (!NT_SUCCESS(RtlAnsiStringToUnicodeString(&StringU1, &StringA1, TRUE)))
//             {
//                 /* We cannot fail */
//                 KeBugCheck(ASSIGN_DRIVE_LETTERS_FAILED);
//             }
//
//             /* Open the device */
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &StringU1,
//                                        OBJ_CASE_INSENSITIVE,
//                                        NULL,
//                                        NULL);
//             Status = ZwOpenFile(&FileHandle,
//                                 SYNCHRONIZE | FILE_READ_DATA,
//                                 &ObjectAttributes,
//                                 &StatusBlock,
//                                 FILE_SHARE_READ,
//                                 FILE_SYNCHRONOUS_IO_NONALERT);
//             if (NT_SUCCESS(Status))
//             {
//                 /* If we managed, create the link */
//                 sprintf(Buffer2, "\\DosDevices\\PhysicalDrive%d", i as u32);
//                 RtlInitAnsiString(&StringA2, Buffer2);
//                 Status = RtlAnsiStringToUnicodeString(&StringU2, &StringA2, TRUE);
//                 if (NT_SUCCESS(Status))
//                 {
//                     IoCreateSymbolicLink(&StringU2, &StringU1);
//                     RtlFreeUnicodeString(&StringU2);
//                 }
//
//                 ZwClose(FileHandle);
//
//                 RealDiskCount = i + 1;
//             }
//
//             RtlFreeUnicodeString(&StringU1);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 if (Increment < 50)
//                 {
//                     ++Increment;
//                     ++DiskCount;
//                 }
//             }
//         }
//     }
//
//     /* We done for our buffers */
//     ExFreePoolWithTag(Buffer1, TAG_FSTUB);
//     ExFreePoolWithTag(Buffer2, TAG_FSTUB);
//
//     /* Upcase our load options, if any */
//     if (LoaderBlock->LoadOptions != NULL)
//     {
//         LoadOptions = _strupr(LoaderBlock->LoadOptions);
//     }
//     else
//     {
//         LoadOptions = NULL;
//     }
//
//     /* If we boot with /MININT (system hive as volatile) option, assign X letter to boot device */
//     if (LoadOptions != NULL &&
//         strstr(LoadOptions, "MININT") != 0 &&
//         NT_SUCCESS(RtlAnsiStringToUnicodeString(&StringU1, NtDeviceName, TRUE)))
//     {
//         if (NT_SUCCESS(HalpSetMountLetter(&StringU1, 'X')))
//         {
//             *NtSystemPath = 'X';
//         }
//
//         RtlFreeUnicodeString(&StringU1);
//     }
//
//     /* Compute our disks derangements */
//     DiskCount -= Increment;
//     if (RealDiskCount > DiskCount)
//     {
//         DiskCount = RealDiskCount;
//     }
//     Devices = IopComputeHarddiskDerangements(DiskCount);
//
//     /* Now, start browsing all our disks for assigning drive letters
//      * Here, we'll only handle boot partition and primary partitions
//      */
//     HarddiskCount = 0;
//     for (i = 0; i < DiskCount; ++i)
//     {
//         /* Get device ID according to derangements map */
//         if (Devices != NULL)
//         {
//             HarddiskCount = Devices[i];
//         }
//
//         /* Query disk layout */
//         swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition0", HarddiskCount);
//         RtlInitUnicodeString(&StringU1, Buffer);
//         if (!NT_SUCCESS(HalpQueryDriveLayout(&StringU1, &LayoutInfo)))
//         {
//             LayoutInfo = NULL;
//         }
//
//         /* Assume we didn't find system */
//         SystemFound = FALSE;
//         swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition%d", HarddiskCount, 1);
//         RtlInitUnicodeString(&StringU1, Buffer);
//         /* Query partition info for our disk */
//         if (!NT_SUCCESS(HalpQueryPartitionType(&StringU1, LayoutInfo, &PartitionType)))
//         {
//             /* It failed, retry for all the partitions */
//             for (PartitionCount = 1; ; ++PartitionCount)
//             {
//                 swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition%d", HarddiskCount, PartitionCount);
//                 RtlInitUnicodeString(&StringU1, Buffer);
//                 if (!NT_SUCCESS(HalpQueryPartitionType(&StringU1, LayoutInfo, &PartitionType)))
//                 {
//                     break;
//                 }
//
//                 /* We found a primary partition, assign a drive letter */
//                 if (PartitionType == PrimaryPartition)
//                 {
//                     HalpNextDriveLetter(&StringU1, NtDeviceName, NtSystemPath, 0);
//                     break;
//                 }
//             }
//         }
//         else
//         {
//             /* All right */
//             for (PartitionCount = 2; ; ++PartitionCount)
//             {
//                 /* If our partition is bootable (MBR) or data (GPT), that's system partition */
//                 if (PartitionType == BootablePartition || PartitionType == DataPartition)
//                 {
//                     SystemFound = TRUE;
//
//                     /* Assign a drive letter and stop here if MBR */
//                     HalpNextDriveLetter(&StringU1, NtDeviceName, NtSystemPath, 0);
//                     if (PartitionType == BootablePartition)
//                     {
//                         break;
//                     }
//                 }
//
//                 /* Keep looping on all the partitions */
//                 swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition%d", HarddiskCount, PartitionCount);
//                 RtlInitUnicodeString(&StringU1, Buffer);
//                 if (!NT_SUCCESS(HalpQueryPartitionType(&StringU1, LayoutInfo, &PartitionType)))
//                 {
//                     /* Mount every primary partition if we didn't find system */
//                     if (!SystemFound)
//                     {
//                         for (PartitionCount = 1; ; ++PartitionCount)
//                         {
//                             swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition%d", HarddiskCount, PartitionCount);
//                             RtlInitUnicodeString(&StringU1, Buffer);
//                             if (!NT_SUCCESS(HalpQueryPartitionType(&StringU1, LayoutInfo, &PartitionType)))
//                             {
//                                 break;
//                             }
//
//                             if (PartitionType == PrimaryPartition)
//                             {
//                                 HalpNextDriveLetter(&StringU1, NtDeviceName, NtSystemPath, 0);
//                                 break;
//                             }
//                         }
//                     }
//
//                     break;
//                 }
//             }
//         }
//
//         /* Free layout, we'll reallocate it for next device */
//         if (LayoutInfo != NULL)
//         {
//             ExFreePoolWithTag(LayoutInfo, TAG_FSTUB);
//         }
//
//         HarddiskCount = i + 1;
//     }
//
//     /* Now, assign logical partitions */
//     for (i = 0; i < DiskCount; ++i)
//     {
//         /* Get device ID according to derangements map */
//         if (Devices != NULL)
//         {
//             HarddiskCount = Devices[i];
//         }
//         else
//         {
//             HarddiskCount = i;
//         }
//
//         /* Query device layout */
//         swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition0", HarddiskCount);
//         RtlInitUnicodeString(&StringU1, Buffer);
//         if (!NT_SUCCESS(HalpQueryDriveLayout(&StringU1, &LayoutInfo)))
//         {
//             LayoutInfo = NULL;
//         }
//
//         /* And assign drive letter to logical partitions */
//         for (PartitionCount = 1; ; ++PartitionCount)
//         {
//             swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition%d", HarddiskCount, PartitionCount);
//             RtlInitUnicodeString(&StringU1, Buffer);
//             if (!NT_SUCCESS(HalpQueryPartitionType(&StringU1, LayoutInfo, &PartitionType)))
//             {
//                 break;
//             }
//
//             if (PartitionType == LogicalPartition)
//             {
//                 HalpNextDriveLetter(&StringU1, NtDeviceName, NtSystemPath, 0);
//             }
//         }
//
//         /* Free layout, we'll reallocate it for next device */
//         if (LayoutInfo != NULL)
//         {
//             ExFreePoolWithTag(LayoutInfo, 0);
//         }
//     }
//
//     /* Now, assign drive letters to everything else */
//     for (i = 0; i < DiskCount; ++i)
//     {
//         /* Get device ID according to derangements map */
//         if (Devices != NULL)
//         {
//             HarddiskCount = Devices[i];
//         }
//         else
//         {
//             HarddiskCount = i;
//         }
//
//         /* Query device layout */
//         swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition0", HarddiskCount);
//         RtlInitUnicodeString(&StringU1, Buffer);
//         if (!NT_SUCCESS(HalpQueryDriveLayout(&StringU1, &LayoutInfo)))
//         {
//             LayoutInfo = NULL;
//         }
//
//         /* Save system partition if any */
//         SystemPartition = 0;
//         for (PartitionCount = 1; ; ++PartitionCount)
//         {
//             swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition%d", HarddiskCount, PartitionCount);
//             RtlInitUnicodeString(&StringU1, Buffer);
//             if (!NT_SUCCESS(HalpQueryPartitionType(&StringU1, LayoutInfo, &PartitionType)))
//             {
//                 break;
//             }
//
//             if ((PartitionType == BootablePartition || PartitionType == PrimaryPartition) && (SystemPartition == 0))
//             {
//                 SystemPartition = PartitionCount;
//             }
//         }
//
//         /* And assign drive letter to anything but system partition */
//         for (PartitionCount = 1; ; ++PartitionCount)
//         {
//             if (PartitionCount != SystemPartition)
//             {
//                 swprintf(Buffer, L"\\Device\\Harddisk%d\\Partition%d", HarddiskCount, PartitionCount);
//                 RtlInitUnicodeString(&StringU1, Buffer);
//                 if (!NT_SUCCESS(HalpQueryPartitionType(&StringU1, LayoutInfo, &PartitionType)))
//                 {
//                     if (LayoutInfo != NULL)
//                     {
//                         ExFreePoolWithTag(LayoutInfo, 0);
//                     }
//
//                     break;
//                 }
//
//                 if (PartitionType == PrimaryPartition || PartitionType == FtPartition)
//                 {
//                     HalpNextDriveLetter(&StringU1, NtDeviceName, NtSystemPath, 0);
//                 }
//             }
//         }
//     }
//
//     /* We're done with disks, if we have a device map, free it */
//     if (Devices != NULL)
//     {
//         ExFreePoolWithTag(Devices, TAG_FSTUB);
//     }
//
//     /* Now, assign drive letter to floppy drives */
//     for (i = 0; i < ConfigInfo->FloppyCount; ++i)
//     {
//         swprintf(Buffer, L"\\Device\\Floppy%d", i as u32);
//         RtlInitUnicodeString(&StringU1, Buffer);
//         if (HalpIsOldStyleFloppy(&StringU1))
//         {
//             HalpNextDriveLetter(&StringU1, NtDeviceName, NtSystemPath, TRUE);
//         }
//     }
//
//     /* And CD drives */
//     for (i = 0; i < ConfigInfo->CdRomCount; ++i)
//     {
//         swprintf(Buffer, L"\\Device\\CdRom%d", i as u32);
//         RtlInitUnicodeString(&StringU1, Buffer);
//         HalpNextDriveLetter(&StringU1, NtDeviceName, NtSystemPath, TRUE);
//     }
//
//     /* If not remote boot, handle NtDeviceName */
//     if (!IoRemoteBootClient && NT_SUCCESS(RtlAnsiStringToUnicodeString(&StringU1, NtDeviceName, TRUE)))
//     {
//         /* Assign it a drive letter */
//         DriveLetter = HalpNextDriveLetter(&StringU1, NULL, NULL, TRUE);
//         if (DriveLetter != 0)
//         {
//             if (DriveLetter != 0xFF)
//             {
//                 *NtSystemPath = DriveLetter;
//             }
//         }
//         /* If it fails through mount manager, retry manually */
//         else
//         {
//             RtlInitUnicodeString(&StringU2, L"\\Device\\Floppy");
//             RtlInitUnicodeString(&StringU3, L"\\Device\\CdRom");
//
//             if (RtlPrefixUnicodeString(&StringU2, &StringU1, TRUE))
//             {
//                 DriveLetter = 'A';
//             }
//             else if (RtlPrefixUnicodeString(&StringU3, &StringU1, TRUE))
//             {
//                 DriveLetter = 'D';
//             }
//             else
//             {
//                 DriveLetter = 'C';
//             }
//
//             /* Try any drive letter */
//             while (HalpSetMountLetter(&StringU1, DriveLetter) != STATUS_SUCCESS)
//             {
//                 ++DriveLetter;
//
//                 if (DriveLetter > 'Z')
//                 {
//                     break;
//                 }
//             }
//
//             /* If we're beyond Z (ie, no slot left) */
//             if (DriveLetter > 'Z')
//             {
//                 /* Delete Z, and reuse it for system */
//                 HalpDeleteMountLetter('Z');
//                 HalpSetMountLetter(&StringU1, 'Z');
//                 *NtSystemPath = 'Z';
//             }
//             else
//             {
//                 /* Return matching drive letter */
//                 *NtSystemPath = DriveLetter;
//             }
//         }
//
//         RtlFreeUnicodeString(&StringU1);
//     }
//
//     /* Enable auto assignement for mountmgr */
//     HalpEnableAutomaticDriveLetterAssignment();
// }
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// NTSTATUS
// NTAPI
// HalpGetFullGeometry(IN PDEVICE_OBJECT DeviceObject,
//                     OUT PDISK_GEOMETRY_EX Geometry)
// {
//     PIRP Irp;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PKEVENT Event;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Allocate a non-paged event */
//     Event = ExAllocatePoolWithTag(NonPagedPool,
//                                   sizeof(KEVENT),
//                                   TAG_FILE_SYSTEM);
//     if (!Event) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Initialize it */
//     KeInitializeEvent(Event, NotificationEvent, FALSE);
//
//     /* Build the IRP */
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
//                                         DeviceObject,
//                                         NULL,
//                                         0UL,
//                                         Geometry,
//                                         sizeof(DISK_GEOMETRY_EX),
//                                         FALSE,
//                                         Event,
//                                         &IoStatusBlock);
//     if (!Irp)
//     {
//         /* Fail, free the event */
//         ExFreePoolWithTag(Event, TAG_FILE_SYSTEM);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Call the driver and check if it's pending */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait on the driver */
//         KeWaitForSingleObject(Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Free the event and return the Status */
//     ExFreePoolWithTag(Event, TAG_FILE_SYSTEM);
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// HalpIsValidPartitionEntry(IN PPARTITION_DESCRIPTOR Entry,
//                           IN ULONGLONG MaxOffset,
//                           IN ULONGLONG MaxSector)
// {
//     ULONGLONG EndingSector;
//     PAGED_CODE();
//
//     /* Unused partitions are considered valid */
//     if (Entry->PartitionType == PARTITION_ENTRY_UNUSED) return TRUE;
//
//     /* Get the last sector of the partition */
//     EndingSector = GET_STARTING_SECTOR(Entry) + GET_PARTITION_LENGTH(Entry);
//
//     /* Check if it's more then the maximum sector */
//     if (EndingSector > MaxSector)
//     {
//         /* Invalid partition */
//         DPRINT1("FSTUB: entry is invalid\n");
//         DPRINT1("FSTUB: offset %#08lx\n", GET_STARTING_SECTOR(Entry));
//         DPRINT1("FSTUB: length %#08lx\n", GET_PARTITION_LENGTH(Entry));
//         DPRINT1("FSTUB: end %#I64x\n", EndingSector);
//         DPRINT1("FSTUB: max %#I64x\n", MaxSector);
//         return FALSE;
//     }
//     else if (GET_STARTING_SECTOR(Entry) > MaxOffset)
//     {
//         /* Invalid partition */
//         DPRINT1("FSTUB: entry is invalid\n");
//         DPRINT1("FSTUB: offset %#08lx\n", GET_STARTING_SECTOR(Entry));
//         DPRINT1("FSTUB: length %#08lx\n", GET_PARTITION_LENGTH(Entry));
//         DPRINT1("FSTUB: end %#I64x\n", EndingSector);
//         DPRINT1("FSTUB: maxOffset %#I64x\n", MaxOffset);
//         return FALSE;
//     }
//
//     /* It's fine, return success */
//     return TRUE;
// }
//
// VOID
// NTAPI
// HalpCalculateChsValues(IN PLARGE_INTEGER PartitionOffset,
//                        IN PLARGE_INTEGER PartitionLength,
//                        IN CCHAR ShiftCount,
//                        IN ULONG SectorsPerTrack,
//                        IN ULONG NumberOfTracks,
//                        IN ULONG ConventionalCylinders,
//                        OUT PPARTITION_DESCRIPTOR PartitionDescriptor)
// {
//     LARGE_INTEGER FirstSector, SectorCount;
//     ULONG LastSector, Remainder, SectorsPerCylinder;
//     ULONG StartingCylinder, EndingCylinder;
//     ULONG StartingTrack, EndingTrack;
//     ULONG StartingSector, EndingSector;
//     PAGED_CODE();
//
//     /* Calculate the number of sectors for each cylinder */
//     SectorsPerCylinder = SectorsPerTrack * NumberOfTracks;
//
//     /* Calculate the first sector, and the sector count */
//     FirstSector.QuadPart = PartitionOffset->QuadPart >> ShiftCount;
//     SectorCount.QuadPart = PartitionLength->QuadPart >> ShiftCount;
//
//     /* Now calculate the last sector */
//     LastSector = FirstSector.LowPart + SectorCount.LowPart - 1;
//
//     /* Calculate the first and last cylinders */
//     StartingCylinder = FirstSector.LowPart / SectorsPerCylinder;
//     EndingCylinder = LastSector / SectorsPerCylinder;
//
//     /* Set the default number of cylinders */
//     if (!ConventionalCylinders) ConventionalCylinders = 1024;
//
//     /* Normalize the values */
//     if (StartingCylinder >= ConventionalCylinders)
//     {
//         /* Set the maximum to 1023 */
//         StartingCylinder = ConventionalCylinders - 1;
//     }
//     if (EndingCylinder >= ConventionalCylinders)
//     {
//         /* Set the maximum to 1023 */
//         EndingCylinder = ConventionalCylinders - 1;
//     }
//
//     /* Calculate the starting head and sector that still remain */
//     Remainder = FirstSector.LowPart % SectorsPerCylinder;
//     StartingTrack = Remainder / SectorsPerTrack;
//     StartingSector = Remainder % SectorsPerTrack;
//
//     /* Calculate the ending head and sector that still remain */
//     Remainder = LastSector % SectorsPerCylinder;
//     EndingTrack = Remainder / SectorsPerTrack;
//     EndingSector = Remainder % SectorsPerTrack;
//
//     /* Set cylinder data for the MSB */
//     PartitionDescriptor->StartingCylinderMsb = (UCHAR)StartingCylinder;
//     PartitionDescriptor->EndingCylinderMsb = (UCHAR)EndingCylinder;
//
//     /* Set the track data */
//     PartitionDescriptor->StartingTrack = (UCHAR)StartingTrack;
//     PartitionDescriptor->EndingTrack = (UCHAR)EndingTrack;
//
//     /* Update cylinder data for the LSB */
//     StartingCylinder = ((StartingSector + 1) & 0x3F) |
//                        ((StartingCylinder >> 2) & 0xC0);
//     EndingCylinder = ((EndingSector + 1) & 0x3F) |
//                      ((EndingCylinder >> 2) & 0xC0);
//
//     /* Set the cylinder data for the LSB */
//     PartitionDescriptor->StartingCylinderLsb = (UCHAR)StartingCylinder;
//     PartitionDescriptor->EndingCylinderLsb = (UCHAR)EndingCylinder;
// }
//
// VOID
// FASTCALL
// xHalGetPartialGeometry(IN PDEVICE_OBJECT DeviceObject,
//                        IN PULONG ConventionalCylinders,
//                        IN PLONGLONG DiskSize)
// {
//     PDISK_GEOMETRY DiskGeometry = NULL;
//     PIO_STATUS_BLOCK IoStatusBlock = NULL;
//     PKEVENT Event = NULL;
//     PIRP Irp;
//     NTSTATUS Status;
//
//     /* Set defaults */
//     *ConventionalCylinders = 0;
//     *DiskSize = 0;
//
//     /* Allocate the structure in nonpaged pool */
//     DiskGeometry = ExAllocatePoolWithTag(NonPagedPool,
//                                          sizeof(DISK_GEOMETRY),
//                                          TAG_FILE_SYSTEM);
//     if (!DiskGeometry) goto Cleanup;
//
//     /* Allocate the status block in nonpaged pool */
//     IoStatusBlock = ExAllocatePoolWithTag(NonPagedPool,
//                                           sizeof(IO_STATUS_BLOCK),
//                                           TAG_FILE_SYSTEM);
//     if (!IoStatusBlock) goto Cleanup;
//
//     /* Allocate the event in nonpaged pool too */
//     Event = ExAllocatePoolWithTag(NonPagedPool,
//                                   sizeof(KEVENT),
//                                   TAG_FILE_SYSTEM);
//     if (!Event) goto Cleanup;
//
//     /* Initialize the event */
//     KeInitializeEvent(Event, NotificationEvent, FALSE);
//
//     /* Build the IRP */
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_DRIVE_GEOMETRY,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         DiskGeometry,
//                                         sizeof(DISK_GEOMETRY),
//                                         FALSE,
//                                         Event,
//                                         IoStatusBlock);
//     if (!Irp) goto Cleanup;
//
//     /* Now call the driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for it to complete */
//         KeWaitForSingleObject(Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock->Status;
//     }
//
//     /* Check driver status */
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the cylinder count */
//         *ConventionalCylinders = DiskGeometry->Cylinders.LowPart;
//
//         /* Make sure it's not larger then 1024 */
//         if (DiskGeometry->Cylinders.LowPart >= 1024)
//         {
//             /* Otherwise, normalize the value */
//             *ConventionalCylinders = 1024;
//         }
//
//         /* Calculate the disk size */
//         *DiskSize = DiskGeometry->Cylinders.QuadPart *
//                     DiskGeometry->TracksPerCylinder *
//                     DiskGeometry->SectorsPerTrack *
//                     DiskGeometry->BytesPerSector;
//     }
//
// Cleanup:
//     /* Free all the pointers */
//     if (Event) ExFreePoolWithTag(Event, TAG_FILE_SYSTEM);
//     if (IoStatusBlock) ExFreePoolWithTag(IoStatusBlock, TAG_FILE_SYSTEM);
//     if (DiskGeometry) ExFreePoolWithTag(DiskGeometry, TAG_FILE_SYSTEM);
//     return;
// }
//
// VOID
// FASTCALL
// xHalExamineMBR(IN PDEVICE_OBJECT DeviceObject,
//                IN ULONG SectorSize,
//                IN ULONG MbrTypeIdentifier,
//                OUT PVOID *MbrBuffer)
// {
//     LARGE_INTEGER Offset;
//     PUCHAR Buffer;
//     ULONG BufferSize;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//     PPARTITION_DESCRIPTOR PartitionDescriptor;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStackLocation;
//
//     Offset.QuadPart = 0;
//
//     /* Assume failure */
//     *MbrBuffer = NULL;
//
//     /* Normalize the buffer size */
//     BufferSize = max(512, SectorSize);
//
//     /* Allocate the buffer */
//     Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                    max(PAGE_SIZE, BufferSize),
//                                    TAG_FILE_SYSTEM);
//     if (!Buffer) return;
//
//     /* Initialize the Event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Build the IRP */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                        DeviceObject,
//                                        Buffer,
//                                        BufferSize,
//                                        &Offset,
//                                        &Event,
//                                        &IoStatusBlock);
//     if (!Irp)
//     {
//         /* Failed */
//         ExFreePoolWithTag(Buffer, TAG_FILE_SYSTEM);
//         return;
//     }
//
//     /* Make sure to override volume verification */
//     IoStackLocation = IoGetNextIrpStackLocation(Irp);
//     IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//     /* Call the driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for completion */
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Check driver Status */
//     if (NT_SUCCESS(Status))
//     {
//         /* Validate the MBR Signature */
//         if (*(PUINT16)&Buffer[BOOT_SIGNATURE_OFFSET] != BOOT_RECORD_SIGNATURE)
//         {
//             /* Failed */
//             ExFreePoolWithTag(Buffer, TAG_FILE_SYSTEM);
//             return;
//         }
//
//         /* Get the partition entry */
//         PartitionDescriptor = (PPARTITION_DESCRIPTOR)&Buffer[PARTITION_TABLE_OFFSET];
//
//         /* Make sure it's what the caller wanted */
//         if (PartitionDescriptor->PartitionType != MbrTypeIdentifier)
//         {
//             /* It's not, free our buffer */
//             ExFreePoolWithTag(Buffer, TAG_FILE_SYSTEM);
//         }
//         else
//         {
//             /* Check for OnTrack Disk Manager 6.0 / EZ-Drive partitions */
//
//             if (PartitionDescriptor->PartitionType == PARTITION_DM)
//             {
//                 /* Return our buffer, but at sector 63 */
//                 *(PULONG)Buffer = 63;
//                 *MbrBuffer = Buffer;
//             }
//             else if (PartitionDescriptor->PartitionType == PARTITION_EZDRIVE)
//             {
//                 /* EZ-Drive, return the buffer directly */
//                 *MbrBuffer = Buffer;
//             }
//             else
//             {
//                 /* Otherwise crash on debug builds */
//                 ASSERT(PartitionDescriptor->PartitionType == PARTITION_EZDRIVE);
//             }
//         }
//     }
// }
//
// VOID
// NTAPI
// FstubFixupEfiPartition(IN PPARTITION_DESCRIPTOR PartitionDescriptor,
//                        IN ULONGLONG MaxOffset)
// {
//     ULONG PartitionMaxOffset, PartitionLength;
//     PAGED_CODE();
//
//     /* Compute partition length (according to MBR entry) */
//     PartitionMaxOffset = GET_STARTING_SECTOR(PartitionDescriptor) + GET_PARTITION_LENGTH(PartitionDescriptor);
//     /* In case the partition length goes beyond disk size... */
//     if (PartitionMaxOffset > MaxOffset)
//     {
//         /* Resize partition to its maximum real length */
//         PartitionLength = (ULONG)(PartitionMaxOffset - GET_STARTING_SECTOR(PartitionDescriptor));
//         SET_PARTITION_LENGTH(PartitionDescriptor, PartitionLength);
//     }
// }
//
// NTSTATUS
// FASTCALL
// xHalIoReadPartitionTable(IN PDEVICE_OBJECT DeviceObject,
//                          IN ULONG SectorSize,
//                          IN BOOLEAN ReturnRecognizedPartitions,
//                          IN OUT PDRIVE_LAYOUT_INFORMATION *PartitionBuffer)
// {
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//     PPARTITION_DESCRIPTOR PartitionDescriptor;
//     CCHAR Entry;
//     NTSTATUS Status;
//     PPARTITION_INFORMATION PartitionInfo;
//     PUCHAR Buffer = NULL;
//     ULONG BufferSize = 2048, InputSize;
//     PDRIVE_LAYOUT_INFORMATION DriveLayoutInfo = NULL;
//     LONG j = -1, i = -1, k;
//     DISK_GEOMETRY_EX DiskGeometryEx;
//     LONGLONG EndSector, MaxSector, StartOffset;
//     LARGE_INTEGER Offset, VolumeOffset;
//     BOOLEAN IsPrimary = TRUE, IsEzDrive = FALSE, MbrFound = FALSE;
//     BOOLEAN IsValid, IsEmpty = TRUE;
//     PVOID MbrBuffer;
//     PIO_STACK_LOCATION IoStackLocation;
//     UCHAR PartitionType;
//     LARGE_INTEGER HiddenSectors64;
//
//     PAGED_CODE();
//
//     VolumeOffset.QuadPart = Offset.QuadPart = 0;
//
//     /* Allocate the buffer */
//     *PartitionBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                              BufferSize,
//                                              TAG_FILE_SYSTEM);
//     if (!(*PartitionBuffer)) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Normalize the buffer size */
//     InputSize = max(512, SectorSize);
//
//     /* Check for EZ-Drive */
//     HalExamineMBR(DeviceObject, InputSize, PARTITION_EZDRIVE, &MbrBuffer);
//     if (MbrBuffer)
//     {
//         /* EZ-Drive found, bias the offset */
//         IsEzDrive = TRUE;
//         ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
//         Offset.QuadPart = 512;
//     }
//
//     /* Get drive geometry */
//     Status = HalpGetFullGeometry(DeviceObject, &DiskGeometryEx);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(*PartitionBuffer, TAG_FILE_SYSTEM);
//         *PartitionBuffer = NULL;
//         return Status;
//     }
//
//     /* Get the end and maximum sector */
//     EndSector = DiskGeometryEx.DiskSize.QuadPart / DiskGeometryEx.Geometry.BytesPerSector;
//     MaxSector = EndSector << 1;
//     DPRINT("FSTUB: DiskSize = %#I64x, MaxSector = %#I64x\n",
//             DiskGeometryEx.DiskSize, MaxSector);
//
//     /* Allocate our buffer */
//     Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, InputSize, TAG_FILE_SYSTEM);
//     if (!Buffer)
//     {
//         /* Fail, free the input buffer */
//         ExFreePoolWithTag(*PartitionBuffer, TAG_FILE_SYSTEM);
//         *PartitionBuffer = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Start partition loop */
//     do
//     {
//         /* Assume the partition is valid */
//         IsValid = TRUE;
//
//         /* Initialize the event */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//         /* Clear the buffer and build the IRP */
//         RtlZeroMemory(Buffer, InputSize);
//         Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                            DeviceObject,
//                                            Buffer,
//                                            InputSize,
//                                            &Offset,
//                                            &Event,
//                                            &IoStatusBlock);
//         if (!Irp)
//         {
//             /* Failed */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//         }
//
//         /* Make sure to disable volume verification */
//         IoStackLocation = IoGetNextIrpStackLocation(Irp);
//         IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//         /* Call the driver */
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion */
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* Normalize status code and check for failure */
//         if (Status == STATUS_NO_DATA_DETECTED) Status = STATUS_SUCCESS;
//         if (!NT_SUCCESS(Status)) break;
//
//         /* If we biased for EZ-Drive, unbias now */
//         if (IsEzDrive && (Offset.QuadPart == 512)) Offset.QuadPart = 0;
//
//         /* Make sure this is a valid MBR */
//         if (*(PUINT16)&Buffer[BOOT_SIGNATURE_OFFSET] != BOOT_RECORD_SIGNATURE)
//         {
//             /* It's not, fail */
//             DPRINT1("FSTUB: (IoReadPartitionTable) No 0xaa55 found in "
//                     "partition table %d\n", j + 1);
//             break;
//         }
//
//         /* At this point we have a valid MBR */
//         MbrFound = TRUE;
//
//         /* Check if we weren't given an offset */
//         if (!Offset.QuadPart)
//         {
//             /* Then read the signature off the disk */
//             (*PartitionBuffer)->Signature = *(PUINT32)&Buffer[DISK_SIGNATURE_OFFSET];
//         }
//
//         /* Get the partition descriptor array */
//         PartitionDescriptor = (PPARTITION_DESCRIPTOR)&Buffer[PARTITION_TABLE_OFFSET];
//
//         /* Start looping partitions */
//         j++;
//         DPRINT("FSTUB: Partition Table %d:\n", j);
//         for (Entry = 1, k = 0; Entry <= NUM_PARTITION_TABLE_ENTRIES; Entry++, PartitionDescriptor++)
//         {
//             /* Get the partition type */
//             PartitionType = PartitionDescriptor->PartitionType;
//
//             /* Print debug messages */
//             DPRINT("Partition Entry %d,%d: type %#x %s\n",
//                     j,
//                     Entry,
//                     PartitionType,
//                     (PartitionDescriptor->ActiveFlag) ? "Active" : "");
//             DPRINT("\tOffset %#08lx for %#08lx Sectors\n",
//                     GET_STARTING_SECTOR(PartitionDescriptor),
//                     GET_PARTITION_LENGTH(PartitionDescriptor));
//
//             /* Check whether we're facing a protective MBR */
//             if (PartitionType == EFI_PMBR_OSTYPE_EFI)
//             {
//                 /* Partition length might be bigger than disk size */
//                 FstubFixupEfiPartition(PartitionDescriptor, DiskGeometryEx.DiskSize.QuadPart);
//             }
//
//             /* Make sure that the partition is valid, unless it's the first */
//             if (!(HalpIsValidPartitionEntry(PartitionDescriptor,
//                                             DiskGeometryEx.DiskSize.QuadPart,
//                                             MaxSector)) && (j == 0))
//             {
//                 /* It's invalid, so fail */
//                 IsValid = FALSE;
//                 break;
//             }
//
//             /* Check if it's a container */
//             if (IsContainerPartition(PartitionType))
//             {
//                 /* Increase the count of containers */
//                 if (++k != 1)
//                 {
//                     /* More then one table is invalid */
//                     DPRINT1("FSTUB: Multiple container partitions found in "
//                             "partition table %d\n - table is invalid\n",
//                             j);
//                     IsValid = FALSE;
//                     break;
//                 }
//             }
//
//             /* Check if the partition is supposedly empty */
//             if (IsEmpty)
//             {
//                 /* But check if it actually has a start and/or length */
//                 if ((GET_STARTING_SECTOR(PartitionDescriptor)) ||
//                     (GET_PARTITION_LENGTH(PartitionDescriptor)))
//                 {
//                     /* So then it's not really empty */
//                     IsEmpty = FALSE;
//                 }
//             }
//
//             /* Check if the caller wanted only recognized partitions */
//             if (ReturnRecognizedPartitions)
//             {
//                 /* Then check if this one is unused, or a container */
//                 if ((PartitionType == PARTITION_ENTRY_UNUSED) ||
//                     IsContainerPartition(PartitionType))
//                 {
//                     /* Skip it, since the caller doesn't want it */
//                     continue;
//                 }
//             }
//
//             /* Increase the structure count and check if they can fit */
//             if ((sizeof(DRIVE_LAYOUT_INFORMATION) +
//                  (++i * sizeof(PARTITION_INFORMATION))) >
//                 BufferSize)
//             {
//                 /* Allocate a new buffer that's twice as big */
//                 DriveLayoutInfo = ExAllocatePoolWithTag(NonPagedPool,
//                                                         BufferSize << 1,
//                                                         TAG_FILE_SYSTEM);
//                 if (!DriveLayoutInfo)
//                 {
//                     /* Out of memory, undo this extra structure */
//                     --i;
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     break;
//                 }
//
//                 /* Copy the contents of the old buffer */
//                 RtlMoveMemory(DriveLayoutInfo,
//                               *PartitionBuffer,
//                               BufferSize);
//
//                 /* Free the old buffer and set this one as the new one */
//                 ExFreePoolWithTag(*PartitionBuffer, TAG_FILE_SYSTEM);
//                 *PartitionBuffer = DriveLayoutInfo;
//
//                 /* Double the size */
//                 BufferSize <<= 1;
//             }
//
//             /* Now get the current structure being filled and initialize it */
//             PartitionInfo = &(*PartitionBuffer)->PartitionEntry[i];
//             PartitionInfo->PartitionType = PartitionType;
//             PartitionInfo->RewritePartition = FALSE;
//
//             /* Check if we're dealing with a partition that's in use */
//             if (PartitionType != PARTITION_ENTRY_UNUSED)
//             {
//                 /* Check if it's bootable */
//                 PartitionInfo->BootIndicator = PartitionDescriptor->
//                                                ActiveFlag & 0x80 ?
//                                                TRUE : FALSE;
//
//                 /* Check if its' a container */
//                 if (IsContainerPartition(PartitionType))
//                 {
//                     /* Then don't recognize it and use the volume offset */
//                     PartitionInfo->RecognizedPartition = FALSE;
//                     StartOffset = VolumeOffset.QuadPart;
//                 }
//                 else
//                 {
//                     /* Then recognize it and use the partition offset */
//                     PartitionInfo->RecognizedPartition = TRUE;
//                     StartOffset = Offset.QuadPart;
//                 }
//
//                 /* Get the starting offset */
//                 PartitionInfo->StartingOffset.QuadPart =
//                     StartOffset +
//                     UInt32x32To64(GET_STARTING_SECTOR(PartitionDescriptor),
//                                   SectorSize);
//
//                 /* Calculate the number of hidden sectors */
//                 HiddenSectors64.QuadPart = (PartitionInfo->
//                                             StartingOffset.QuadPart -
//                                             StartOffset) /
//                                             SectorSize;
//                 PartitionInfo->HiddenSectors = HiddenSectors64.LowPart;
//
//                 /* Get the partition length */
//                 PartitionInfo->PartitionLength.QuadPart =
//                     UInt32x32To64(GET_PARTITION_LENGTH(PartitionDescriptor),
//                                   SectorSize);
//
//                 /* Get the partition number */
//                 /* FIXME: REACTOS HACK -- Needed for xHalIoAssignDriveLetters() */
//                 PartitionInfo->PartitionNumber = (!IsContainerPartition(PartitionType)) ? i + 1 : 0;
//             }
//             else
//             {
//                 /* Otherwise, clear all the relevant fields */
//                 PartitionInfo->BootIndicator = FALSE;
//                 PartitionInfo->RecognizedPartition = FALSE;
//                 PartitionInfo->StartingOffset.QuadPart = 0;
//                 PartitionInfo->PartitionLength.QuadPart = 0;
//                 PartitionInfo->HiddenSectors = 0;
//
//                 /* FIXME: REACTOS HACK -- Needed for xHalIoAssignDriveLetters() */
//                 PartitionInfo->PartitionNumber = 0;
//             }
//         }
//
//         /* Finish debug log, and check for failure */
//         DPRINT("\n");
//         if (!NT_SUCCESS(Status)) break;
//
//         /* Also check if we hit an invalid entry here */
//         if (!IsValid)
//         {
//             /* We did, so break out of the loop minus one entry */
//             j--;
//             break;
//         }
//
//         /* Reset the offset */
//         Offset.QuadPart = 0;
//
//         /* Go back to the descriptor array and loop it */
//         PartitionDescriptor = (PPARTITION_DESCRIPTOR)&Buffer[PARTITION_TABLE_OFFSET];
//         for (Entry = 1; Entry <= NUM_PARTITION_TABLE_ENTRIES; Entry++, PartitionDescriptor++)
//         {
//             /* Check if this is a container partition, since we skipped them */
//             if (IsContainerPartition(PartitionDescriptor->PartitionType))
//             {
//                 /* Get its offset */
//                 Offset.QuadPart = VolumeOffset.QuadPart +
//                                   UInt32x32To64(
//                                      GET_STARTING_SECTOR(PartitionDescriptor),
//                                      SectorSize);
//
//                 /* If this is a primary partition, this is the volume offset */
//                 if (IsPrimary) VolumeOffset = Offset;
//
//                 /* Also update the maximum sector */
//                 MaxSector = GET_PARTITION_LENGTH(PartitionDescriptor);
//                 DPRINT1("FSTUB: MaxSector now = %I64d\n", MaxSector);
//                 break;
//             }
//         }
//
//         /* Loop the next partitions, which are not primary anymore */
//         IsPrimary = FALSE;
//     } while (Offset.HighPart | Offset.LowPart);
//
//     /* Check if this is a removable device that's probably a super-floppy */
//     if ((DiskGeometryEx.Geometry.MediaType == RemovableMedia) &&
//         (j == 0) && (MbrFound) && (IsEmpty))
//     {
//         PBOOT_SECTOR_INFO BootSectorInfo = (PBOOT_SECTOR_INFO)Buffer;
//
//         /* Read the jump bytes to detect super-floppy */
//         if ((BootSectorInfo->JumpByte[0] == 0xeb) ||
//             (BootSectorInfo->JumpByte[0] == 0xe9))
//         {
//             /* Super floppes don't have typical MBRs, so skip them */
//             DPRINT1("FSTUB: Jump byte %#x found along with empty partition "
//                     "table - disk is a super floppy and has no valid MBR\n",
//                     BootSectorInfo->JumpByte);
//             j = -1;
//         }
//     }
//
//     /* Check if we're still at partition -1 */
//     if (j == -1)
//     {
//         /* The likely cause is the super floppy detection above */
//         if ((MbrFound) || (DiskGeometryEx.Geometry.MediaType == RemovableMedia))
//         {
//             /* Print out debugging information */
//             DPRINT1("FSTUB: Drive %#p has no valid MBR. Make it into a "
//                     "super-floppy\n",
//                     DeviceObject);
//             DPRINT1("FSTUB: Drive has %I64d sectors and is %#016I64x "
//                     "bytes large\n",
//                     EndSector, DiskGeometryEx.DiskSize);
//
//             /* We should at least have some sectors */
//             if (EndSector > 0)
//             {
//                 /* Get the entry we'll use */
//                 PartitionInfo = &(*PartitionBuffer)->PartitionEntry[0];
//
//                 /* Fill it out with data for a super-floppy */
//                 PartitionInfo->RewritePartition = FALSE;
//                 PartitionInfo->RecognizedPartition = TRUE;
//                 PartitionInfo->PartitionType = PARTITION_FAT_16;
//                 PartitionInfo->BootIndicator = FALSE;
//                 PartitionInfo->HiddenSectors = 0;
//                 PartitionInfo->StartingOffset.QuadPart = 0;
//                 PartitionInfo->PartitionLength = DiskGeometryEx.DiskSize;
//
//                 /* FIXME: REACTOS HACK -- Needed for xHalIoAssignDriveLetters() */
//                 PartitionInfo->PartitionNumber = 0;
//
//                 /* Set the signature and set the count back to 0 */
//                 (*PartitionBuffer)->Signature = 1;
//                 i = 0;
//             }
//         }
//         else
//         {
//             /* Otherwise, this isn't a super floppy, so set an invalid count */
//             i = -1;
//         }
//     }
//
//     /* Set the partition count */
//     (*PartitionBuffer)->PartitionCount = ++i;
//
//     /* If we have no count, delete the signature */
//     if (!i) (*PartitionBuffer)->Signature = 0;
//
//     /* Free the buffer and check for success */
//     if (Buffer) ExFreePoolWithTag(Buffer, TAG_FILE_SYSTEM);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(*PartitionBuffer, TAG_FILE_SYSTEM);
//         *PartitionBuffer = NULL;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// FASTCALL
// xHalIoSetPartitionInformation(IN PDEVICE_OBJECT DeviceObject,
//                               IN ULONG SectorSize,
//                               IN ULONG PartitionNumber,
//                               IN ULONG PartitionType)
// {
//     PIRP Irp;
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//     LARGE_INTEGER Offset, VolumeOffset;
//     PUCHAR Buffer = NULL;
//     ULONG BufferSize;
//     ULONG i = 0;
//     ULONG Entry;
//     PPARTITION_DESCRIPTOR PartitionDescriptor;
//     BOOLEAN IsPrimary = TRUE, IsEzDrive = FALSE;
//     PVOID MbrBuffer;
//     PIO_STACK_LOCATION IoStackLocation;
//
//     PAGED_CODE();
//
//     VolumeOffset.QuadPart = Offset.QuadPart = 0;
//
//     /* Normalize the buffer size */
//     BufferSize = max(512, SectorSize);
//
//     /* Check for EZ-Drive */
//     HalExamineMBR(DeviceObject, BufferSize, PARTITION_EZDRIVE, &MbrBuffer);
//     if (MbrBuffer)
//     {
//         /* EZ-Drive found, bias the offset */
//         IsEzDrive = TRUE;
//         ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
//         Offset.QuadPart = 512;
//     }
//
//     /* Allocate our partition buffer */
//     Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, PAGE_SIZE, TAG_FILE_SYSTEM);
//     if (!Buffer) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Initialize the event we'll use and loop partitions */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//     do
//     {
//         /* Reset the event since we reuse it */
//         KeClearEvent(&Event);
//
//         /* Build the read IRP */
//         Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                            DeviceObject,
//                                            Buffer,
//                                            BufferSize,
//                                            &Offset,
//                                            &Event,
//                                            &IoStatusBlock);
//         if (!Irp)
//         {
//             /* Fail */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//         }
//
//         /* Make sure to disable volume verification */
//         IoStackLocation = IoGetNextIrpStackLocation(Irp);
//         IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//         /* Call the driver */
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion */
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* Check for failure */
//         if (!NT_SUCCESS(Status)) break;
//
//         /* If we biased for EZ-Drive, unbias now */
//         if (IsEzDrive && (Offset.QuadPart == 512)) Offset.QuadPart = 0;
//
//         /* Make sure this is a valid MBR */
//         if (*(PUINT16)&Buffer[BOOT_SIGNATURE_OFFSET] != BOOT_RECORD_SIGNATURE)
//         {
//             /* It's not, fail */
//             Status = STATUS_BAD_MASTER_BOOT_RECORD;
//             break;
//         }
//
//         /* Get the partition descriptors and loop them */
//         PartitionDescriptor = (PPARTITION_DESCRIPTOR)&Buffer[PARTITION_TABLE_OFFSET];
//         for (Entry = 1; Entry <= NUM_PARTITION_TABLE_ENTRIES; Entry++, PartitionDescriptor++)
//         {
//             /* Check if it's unused or a container partition */
//             if ((PartitionDescriptor->PartitionType == PARTITION_ENTRY_UNUSED) ||
//                 (IsContainerPartition(PartitionDescriptor->PartitionType)))
//             {
//                 /* Go to the next one */
//                 continue;
//             }
//
//             /* It's a valid partition, so increase the partition count */
//             if (++i == PartitionNumber)
//             {
//                 /* We found a match, set the type */
//                 PartitionDescriptor->PartitionType = (UCHAR)PartitionType;
//
//                 /* Reset the reusable event */
//                 KeClearEvent(&Event);
//
//                 /* Build the write IRP */
//                 Irp = IoBuildSynchronousFsdRequest(IRP_MJ_WRITE,
//                                                    DeviceObject,
//                                                    Buffer,
//                                                    BufferSize,
//                                                    &Offset,
//                                                    &Event,
//                                                    &IoStatusBlock);
//                 if (!Irp)
//                 {
//                     /* Fail */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     break;
//                 }
//
//                 /* Disable volume verification */
//                 IoStackLocation = IoGetNextIrpStackLocation(Irp);
//                 IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//                 /* Call the driver */
//                 Status = IoCallDriver(DeviceObject, Irp);
//                 if (Status == STATUS_PENDING)
//                 {
//                     /* Wait for completion */
//                     KeWaitForSingleObject(&Event, //                                           Executive, //                                           KernelMode, //                                           FALSE, //                                           NULL);
//                     Status = IoStatusBlock.Status;
//                 }
//
//                 /* We're done, break out of the loop */
//                 break;
//             }
//         }
//
//         /* If we looped all the partitions, break out */
//         if (Entry <= NUM_PARTITION_TABLE_ENTRIES) break;
//
//         /* Nothing found yet, get the partition array again */
//         PartitionDescriptor = (PPARTITION_DESCRIPTOR)&Buffer[PARTITION_TABLE_OFFSET];
//         for (Entry = 1; Entry <= NUM_PARTITION_TABLE_ENTRIES; Entry++, PartitionDescriptor++)
//         {
//             /* Check if this was a container partition (we skipped these) */
//             if (IsContainerPartition(PartitionDescriptor->PartitionType))
//             {
//                 /* Update the partition offset */
//                 Offset.QuadPart = VolumeOffset.QuadPart +
//                                   GET_STARTING_SECTOR(PartitionDescriptor) *
//                                   SectorSize;
//
//                 /* If this was the primary partition, update the volume too */
//                 if (IsPrimary) VolumeOffset = Offset;
//                 break;
//             }
//         }
//
//         /* Check if we already searched all the partitions */
//         if (Entry > NUM_PARTITION_TABLE_ENTRIES)
//         {
//             /* Then we failed to find a good MBR */
//             Status = STATUS_BAD_MASTER_BOOT_RECORD;
//             break;
//         }
//
//         /* Loop the next partitions, which are not primary anymore */
//         IsPrimary = FALSE;
//     } while (i < PartitionNumber);
//
//     /* Everything done, cleanup */
//     if (Buffer) ExFreePoolWithTag(Buffer, TAG_FILE_SYSTEM);
//     return Status;
// }
//
// NTSTATUS
// FASTCALL
// xHalIoWritePartitionTable(IN PDEVICE_OBJECT DeviceObject,
//                           IN ULONG SectorSize,
//                           IN ULONG SectorsPerTrack,
//                           IN ULONG NumberOfHeads,
//                           IN PDRIVE_LAYOUT_INFORMATION PartitionBuffer)
// {
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIRP Irp;
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG BufferSize;
//     PUCHAR Buffer;
//     PPTE Entry;
//     PPARTITION_TABLE PartitionTable;
//     LARGE_INTEGER Offset, NextOffset, ExtendedOffset, SectorOffset;
//     LARGE_INTEGER StartOffset, PartitionLength;
//     ULONG i, j;
//     CCHAR k;
//     BOOLEAN IsEzDrive = FALSE, IsSuperFloppy = FALSE, DoRewrite = FALSE, IsMbr;
//     ULONG ConventionalCylinders;
//     LONGLONG DiskSize;
//     PDISK_LAYOUT DiskLayout = (PDISK_LAYOUT)PartitionBuffer;
//     PVOID MbrBuffer;
//     UCHAR PartitionType;
//     PIO_STACK_LOCATION IoStackLocation;
//     PPARTITION_INFORMATION PartitionInfo = PartitionBuffer->PartitionEntry;
//     PPARTITION_INFORMATION TableEntry;
//
//     PAGED_CODE();
//
//     ExtendedOffset.QuadPart = NextOffset.QuadPart = Offset.QuadPart = 0;
//
//     /* Normalize the buffer size */
//     BufferSize = max(512, SectorSize);
//
//     /* Get the partial drive geometry */
//     xHalGetPartialGeometry(DeviceObject, &ConventionalCylinders, &DiskSize);
//
//     /* Check for EZ-Drive */
//     HalExamineMBR(DeviceObject, BufferSize, PARTITION_EZDRIVE, &MbrBuffer);
//     if (MbrBuffer)
//     {
//         /* EZ-Drive found, bias the offset */
//         IsEzDrive = TRUE;
//         ExFreePoolWithTag(MbrBuffer, TAG_FILE_SYSTEM);
//         Offset.QuadPart = 512;
//     }
//
//     /* Get the number of bits to shift to multiply by the sector size */
//     for (k = 0; k < 32; k++) if ((SectorSize >> k) == 1) break;
//
//     /* Check if there's only one partition */
//     if (PartitionBuffer->PartitionCount == 1)
//     {
//         /* Check if it has no starting offset or hidden sectors */
//         if (!(PartitionInfo->StartingOffset.QuadPart) &&
//             !(PartitionInfo->HiddenSectors))
//         {
//             /* Then it's a super floppy */
//             IsSuperFloppy = TRUE;
//
//             /* Which also means it must be non-bootable FAT-16 */
//             if ((PartitionInfo->PartitionNumber) ||
//                 (PartitionInfo->PartitionType != PARTITION_FAT_16) ||
//                 (PartitionInfo->BootIndicator))
//             {
//                 /* It's not, so we fail */
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Check if it needs a rewrite, and disable EZ-Drive for sure */
//             if (PartitionInfo->RewritePartition) DoRewrite = TRUE;
//             IsEzDrive = FALSE;
//         }
//     }
//
//     /* Count the number of partition tables */
//     DiskLayout->TableCount = (PartitionBuffer->PartitionCount + NUM_PARTITION_TABLE_ENTRIES - 1) / NUM_PARTITION_TABLE_ENTRIES;
//
//     /* Allocate our partition buffer */
//     Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, PAGE_SIZE, TAG_FILE_SYSTEM);
//     if (!Buffer) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Loop the entries */
//     Entry = (PPTE)&Buffer[PARTITION_TABLE_OFFSET];
//     for (i = 0; i < DiskLayout->TableCount; i++)
//     {
//         /* Set if this is the MBR partition */
//         IsMbr= (BOOLEAN)!i;
//
//         /* Initialize th event */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//         /* Build the read IRP */
//         Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                            DeviceObject,
//                                            Buffer,
//                                            BufferSize,
//                                            &Offset,
//                                            &Event,
//                                            &IoStatusBlock);
//         if (!Irp)
//         {
//             /* Fail */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//         }
//
//         /* Make sure to disable volume verification */
//         IoStackLocation = IoGetNextIrpStackLocation(Irp);
//         IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//         /* Call the driver */
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion */
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* Check for failure */
//         if (!NT_SUCCESS(Status)) break;
//
//         /* If we biased for EZ-Drive, unbias now */
//         if (IsEzDrive && (Offset.QuadPart == 512)) Offset.QuadPart = 0;
//
//         /* Check if this is a normal disk */
//         if (!IsSuperFloppy)
//         {
//             /* Set the boot record signature */
//             *(PUINT16)&Buffer[BOOT_SIGNATURE_OFFSET] = BOOT_RECORD_SIGNATURE;
//
//             /* By default, don't require a rewrite */
//             DoRewrite = FALSE;
//
//             /* Check if we don't have an offset */
//             if (!Offset.QuadPart)
//             {
//                 /* Check if the signature doesn't match */
//                 if (*(PUINT32)&Buffer[DISK_SIGNATURE_OFFSET] != PartitionBuffer->Signature)
//                 {
//                     /* Then write the signature and now we need a rewrite */
//                     *(PUINT32)&Buffer[DISK_SIGNATURE_OFFSET] = PartitionBuffer->Signature;
//                     DoRewrite = TRUE;
//                 }
//             }
//
//             /* Loop the partition table entries */
//             PartitionTable = &DiskLayout->PartitionTable[i];
//             for (j = 0; j < NUM_PARTITION_TABLE_ENTRIES; j++)
//             {
//                 /* Get the current entry and type */
//                 TableEntry = &PartitionTable->PartitionEntry[j];
//                 PartitionType = TableEntry->PartitionType;
//
//                 /* Check if the entry needs a rewrite */
//                 if (TableEntry->RewritePartition)
//                 {
//                     /* Then we need one too */
//                     DoRewrite = TRUE;
//
//                     /* Save the type and if it's a bootable partition */
//                     Entry[j].PartitionType = TableEntry->PartitionType;
//                     Entry[j].ActiveFlag = TableEntry->BootIndicator ? 0x80 : 0;
//
//                     /* Make sure it's used */
//                     if (PartitionType != PARTITION_ENTRY_UNUSED)
//                     {
//                         /* Make sure it's not a container (unless primary) */
//                         if ((IsMbr) || !(IsContainerPartition(PartitionType)))
//                         {
//                             /* Use the partition offset */
//                             StartOffset.QuadPart = Offset.QuadPart;
//                         }
//                         else
//                         {
//                             /* Use the extended logical partition offset */
//                             StartOffset.QuadPart = ExtendedOffset.QuadPart;
//                         }
//
//                         /* Set the sector offset */
//                         SectorOffset.QuadPart = TableEntry->
//                                                 StartingOffset.QuadPart -
//                                                 StartOffset.QuadPart;
//
//                         /* Now calculate the starting sector */
//                         StartOffset.QuadPart = SectorOffset.QuadPart >> k;
//                         Entry[j].StartingSector = StartOffset.LowPart;
//
//                         /* As well as the length */
//                         PartitionLength.QuadPart = TableEntry->PartitionLength.
//                                                    QuadPart >> k;
//                         Entry[j].PartitionLength = PartitionLength.LowPart;
//
//                         /* Calculate the CHS values */
//                         HalpCalculateChsValues(&TableEntry->StartingOffset,
//                                                &TableEntry->PartitionLength,
//                                                k,
//                                                SectorsPerTrack,
//                                                NumberOfHeads,
//                                                ConventionalCylinders,
//                                                (PPARTITION_DESCRIPTOR)
//                                                &Entry[j]);
//                     }
//                     else
//                     {
//                         /* Otherwise set up an empty entry */
//                         Entry[j].StartingSector = 0;
//                         Entry[j].PartitionLength = 0;
//                         Entry[j].StartingTrack = 0;
//                         Entry[j].EndingTrack = 0;
//                         Entry[j].StartingCylinder = 0;
//                         Entry[j].EndingCylinder = 0;
//                     }
//                 }
//
//                 /* Check if this is a container partition */
//                 if (IsContainerPartition(PartitionType))
//                 {
//                     /* Then update the offset to use */
//                     NextOffset = TableEntry->StartingOffset;
//                 }
//             }
//         }
//
//         /* Check if we need to write back the buffer */
//         if (DoRewrite)
//         {
//             /* We don't need to do this again */
//             DoRewrite = FALSE;
//
//             /* Initialize the event */
//             KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//             /* If we unbiased for EZ-Drive, rebias now */
//             if (IsEzDrive && !Offset.QuadPart) Offset.QuadPart = 512;
//
//             /* Build the write IRP */
//             Irp = IoBuildSynchronousFsdRequest(IRP_MJ_WRITE,
//                                                DeviceObject,
//                                                Buffer,
//                                                BufferSize,
//                                                &Offset,
//                                                &Event,
//                                                &IoStatusBlock);
//             if (!Irp)
//             {
//                 /* Fail */
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             /* Make sure to disable volume verification */
//             IoStackLocation = IoGetNextIrpStackLocation(Irp);
//             IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//             /* Call the driver */
//             Status = IoCallDriver(DeviceObject, Irp);
//             if (Status == STATUS_PENDING)
//             {
//                 /* Wait for completion */
//                 KeWaitForSingleObject(&Event, //                                       Executive, //                                       KernelMode, //                                       FALSE, //                                       NULL);
//                 Status = IoStatusBlock.Status;
//             }
//
//             /* Check for failure */
//             if (!NT_SUCCESS(Status)) break;
//
//             /* If we biased for EZ-Drive, unbias now */
//             if (IsEzDrive && (Offset.QuadPart == 512)) Offset.QuadPart = 0;
//         }
//
//         /* Update the partition offset and set the extended offset if needed */
//         Offset = NextOffset;
//         if (IsMbr) ExtendedOffset = NextOffset;
//     }
//
//     /* If we had a buffer, free it, then return status */
//     if (Buffer) ExFreePoolWithTag(Buffer, TAG_FILE_SYSTEM);
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// HalExamineMBR(IN PDEVICE_OBJECT DeviceObject,
//               IN ULONG SectorSize,
//               IN ULONG MbrTypeIdentifier,
//               OUT PVOID *MbrBuffer)
// {
//     HALDISPATCH->HalExamineMBR(DeviceObject,
//                                SectorSize,
//                                MbrTypeIdentifier,
//                                MbrBuffer);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// FASTCALL
// IoReadPartitionTable(IN PDEVICE_OBJECT DeviceObject,
//                      IN ULONG SectorSize,
//                      IN BOOLEAN ReturnRecognizedPartitions,
//                      IN OUT PDRIVE_LAYOUT_INFORMATION *PartitionBuffer)
// {
//     return HALDISPATCH->HalIoReadPartitionTable(DeviceObject,
//                                                 SectorSize,
//                                                 ReturnRecognizedPartitions,
//                                                 PartitionBuffer);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// FASTCALL
// IoSetPartitionInformation(IN PDEVICE_OBJECT DeviceObject,
//                           IN ULONG SectorSize,
//                           IN ULONG PartitionNumber,
//                           IN ULONG PartitionType)
// {
//     return HALDISPATCH->HalIoSetPartitionInformation(DeviceObject,
//                                                      SectorSize,
//                                                      PartitionNumber,
//                                                      PartitionType);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// FASTCALL
// IoWritePartitionTable(IN PDEVICE_OBJECT DeviceObject,
//                       IN ULONG SectorSize,
//                       IN ULONG SectorsPerTrack,
//                       IN ULONG NumberOfHeads,
//                       IN PDRIVE_LAYOUT_INFORMATION PartitionBuffer)
// {
//     return HALDISPATCH->HalIoWritePartitionTable(DeviceObject,
//                                                  SectorSize,
//                                                  SectorsPerTrack,
//                                                  NumberOfHeads,
//                                                  PartitionBuffer);
// }
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// IoAssignDriveLetters(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                      IN PSTRING NtDeviceName,
//                      OUT PUCHAR NtSystemPath,
//                      OUT PSTRING NtSystemPathString)
// {
//     HALDISPATCH->HalIoAssignDriveLetters(LoaderBlock,
//                                          NtDeviceName,
//                                          NtSystemPath,
//                                          NtSystemPathString);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    ANSI_STRING, ARC_DISK_SIGNATURE, BOOTDISK_INFORMATION, BOOTDISK_INFORMATION_EX, DISK_GEOMETRY,
    DISK_GEOMETRY_EX, DRIVE_LAYOUT_INFORMATION, DRIVE_LAYOUT_INFORMATION_EX, ExFreePool, GUID,
    _ARC_DISK_SIGNATURE, _IO_STATUS_BLOCK, _UNICODE_STRING, _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest,
    IoGetConfigurationInformation, IoGetDeviceObjectPointer, IoGetNextIrpStackLocation,
    IoReadPartitionTable, IoSetPartitionInformation, IoWritePartitionTable,
    IofCallDriver, IopVerifyDiskSignature, KEVENT, KeInitializeEvent,
    KeWaitForSingleObject, LARGE_INTEGER, PARC_DISK_INFORMATION, PARC_DISK_SIGNATURE,
    PARTITION_INFORMATION, PARTITION_INFORMATION_EX, PBOOTDISK_INFORMATION,
    PBOOTDISK_INFORMATION_EX, PCREATE_DISK, PCREATE_DISK_GPT, PCREATE_DISK_MBR, PDEVICE_OBJECT,
    PDISK_GEOMETRY_EX, PDISK_SIGNATURE, PDRIVE_LAYOUT_INFORMATION, PDRIVE_LAYOUT_INFORMATION_EX,
    PFILE_OBJECT, PGUID, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PKEVENT,
    PLOADER_PARAMETER_BLOCK, PPARTITION_DESCRIPTOR, PPARTITION_INFORMATION_EX,
    PSET_PARTITION_INFORMATION_EX, PUINT16, PUINT32, PULONGLONG, RtlAnsiStringToUnicodeString,
    RtlComputeCrc32, RtlEqualString, RtlFreeUnicodeString, RtlInitAnsiString,
    SET_PARTITION_INFORMATION_GPT, UINT32,
};
use core::ffi::c_void;

extern "C" {
    fn sprintf(s: PCHAR, format: *const i8, ...) -> i32;
}
extern "system" {
    static IopLoaderBlock: PLOADER_PARAMETER_BLOCK;
}

// IoCallDriver — bindgen exports the implementation as IofCallDriver.
#[inline]
unsafe fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    IofCallDriver(DeviceObject, Irp)
}

// ===== File-local C structs (from fstubex.c) =====

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DISK_INFORMATION {
    pub DeviceObject: PDEVICE_OBJECT,
    pub SectorSize: ULONG,
    pub DiskGeometry: DISK_GEOMETRY_EX,
    pub Buffer: PUCHAR,
    pub SectorCount: ULONGLONG,
}
pub type PDISK_INFORMATION = *mut DISK_INFORMATION;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct EFI_PARTITION_HEADER {
    pub Signature: ULONGLONG,
    pub Revision: ULONG,
    pub HeaderSize: ULONG,
    pub HeaderCRC32: ULONG,
    pub Reserved: ULONG,
    pub MyLBA: ULONGLONG,
    pub AlternateLBA: ULONGLONG,
    pub FirstUsableLBA: ULONGLONG,
    pub LastUsableLBA: ULONGLONG,
    pub DiskGUID: GUID,
    pub PartitionEntryLBA: ULONGLONG,
    pub NumberOfEntries: ULONG,
    pub SizeOfPartitionEntry: ULONG,
    pub PartitionEntryCRC32: ULONG,
}
pub type PEFI_PARTITION_HEADER = *mut EFI_PARTITION_HEADER;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct EFI_PARTITION_ENTRY {
    pub PartitionType: GUID,
    pub UniquePartition: GUID,
    pub StartingLBA: ULONGLONG,
    pub EndingLBA: ULONGLONG,
    pub Attributes: ULONGLONG,
    pub Name: [u16; 0x24],
}
pub type PEFI_PARTITION_ENTRY = *mut EFI_PARTITION_ENTRY;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PARTITION_TABLE_ENTRY {
    pub BootIndicator: UCHAR,
    pub StartHead: UCHAR,
    pub StartSector: UCHAR,
    pub StartCylinder: UCHAR,
    pub SystemIndicator: UCHAR,
    pub EndHead: UCHAR,
    pub EndSector: UCHAR,
    pub EndCylinder: UCHAR,
    pub SectorCountBeforePartition: ULONG,
    pub PartitionSectorCount: ULONG,
}
pub type PPARTITION_TABLE_ENTRY = *mut PARTITION_TABLE_ENTRY;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MASTER_BOOT_RECORD {
    pub MasterBootRecordCodeAndData: [UCHAR; 0x1B8],
    pub Signature: ULONG,
    pub Reserved: USHORT,
    pub PartitionTable: [PARTITION_TABLE_ENTRY; 4],
    pub MasterBootRecordMagic: USHORT,
}
pub type PMASTER_BOOT_RECORD = *mut MASTER_BOOT_RECORD;

// ===== Constants (from fstubex.c and hal.h) =====

const PARTITION_ENTRY_SIZE: ULONG = 128;
const EFI_HEADER_SIGNATURE: ULONGLONG = 0x5452415020494645;
const EFI_HEADER_REVISION_1: ULONG = 0x00010000;
const EFI_PMBR_OSTYPE_EFI: UCHAR = 0xEE;
const TAG_FSTUB: ULONG = u32::from_be_bytes(*b"BtsF");

// From ntoskrnl/include/internal/hal.h
const DISK_SIGNATURE_OFFSET: usize = 0x1B8;
const PARTITION_TABLE_OFFSET: usize = 0x1BE;
const BOOT_SIGNATURE_OFFSET: usize = 0x200 - 2;
const BOOT_RECORD_SIGNATURE: USHORT = 0xAA55;
const NUM_PARTITION_TABLE_ENTRIES: usize = 4;

// PARTITION_STYLE values (bindgen exports as `_PARTITION_STYLE_PARTITION_STYLE_*`).
// PARTITION_STYLE in C is typed; here it's ULONG/u32 based on _PARTITION_INFORMATION_EX.
pub type PARTITION_STYLE = ULONG;
const PARTITION_STYLE_MBR: PARTITION_STYLE = 0;
const PARTITION_STYLE_GPT: PARTITION_STYLE = 1;
const PARTITION_STYLE_RAW: PARTITION_STYLE = 2;

// IOCTL_DISK_GET_DRIVE_GEOMETRY_EX = CTL_CODE(IOCTL_DISK_BASE=0x07, 0x28, METHOD_BUFFERED=0, FILE_ANY_ACCESS=0)
//  = (0x07 << 16) | (0 << 14) | (0x28 << 2) | 0 = 0x000700A0
const IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: ULONG = 0x000700A0;

// SL_OVERRIDE_VERIFY_VOLUME from xdk/iotypes.h
const SL_OVERRIDE_VERIFY_VOLUME: UCHAR = 0x02;

// NTSTATUS constants from ntstatus.h (sdk/include/psdk/ntstatus.h)
const STATUS_DEVICE_NOT_READY: NTSTATUS = 0xC00000A3u32 as i32;
const STATUS_DISK_CORRUPT_ERROR: NTSTATUS = 0xC0000032u32 as i32;
const STATUS_TOO_LATE: NTSTATUS = 0xC0000189u32 as i32;

// FIELD_OFFSET macro replacement: use core::mem::offset_of!
macro_rules! FIELD_OFFSET {
    ($ty:ty, $field:ident) => {
        core::mem::offset_of!($ty, $field)
    };
}

pub unsafe fn FstubAdjustPartitionCount(
    SectorSize: ULONG,
    PartitionCount: PULONG
) {
    let mut Count: ULONG;

    Count = *PartitionCount;
    if Count < 128 {
        Count = 128;
    }

    Count = (Count * PARTITION_ENTRY_SIZE) / SectorSize;
    Count = (Count * SectorSize) / PARTITION_ENTRY_SIZE;

    *PartitionCount = Count;
}

#[no_mangle]
pub unsafe extern "system" fn FstubAllocateDiskInformation(
    DeviceObject: PDEVICE_OBJECT,
    DiskBuffer: *mut PDISK_INFORMATION,
    DiskGeometry: PDISK_GEOMETRY_EX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let DiskInformation: PDISK_INFORMATION;

    DiskInformation = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<DISK_INFORMATION>() as SIZE_T,
        TAG_FSTUB,
    ) as PDISK_INFORMATION;
    if DiskInformation.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if DiskGeometry.is_null() {
        Status = FstubGetDiskGeometry(
            DeviceObject,
            core::ptr::addr_of_mut!((*DiskInformation).DiskGeometry),
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(DiskInformation as *mut core::ffi::c_void, TAG_FSTUB);
            return Status;
        }
    } else {
        core::ptr::copy_nonoverlapping(
            DiskGeometry as *const u8,
            core::ptr::addr_of_mut!((*DiskInformation).DiskGeometry) as *mut u8,
            core::mem::size_of::<DISK_GEOMETRY_EX>(),
        );
    }

    if (*DiskInformation).DiskGeometry.Geometry.BytesPerSector == 0
        || (*DiskInformation).DiskGeometry.DiskSize.QuadPart == 0
    {
        ExFreePoolWithTag(DiskInformation as *mut core::ffi::c_void, TAG_FSTUB);
        return STATUS_DEVICE_NOT_READY;
    }

    (*DiskInformation).DeviceObject = DeviceObject;
    (*DiskInformation).SectorSize = (*DiskInformation).DiskGeometry.Geometry.BytesPerSector;
    (*DiskInformation).SectorCount =
        ((*DiskInformation).DiskGeometry.DiskSize.QuadPart as ULONGLONG) / (*DiskInformation).SectorSize as ULONGLONG;

    (*DiskInformation).Buffer = ExAllocatePoolWithTag(
        NonPagedPoolCacheAligned,
        (*DiskInformation).SectorSize as SIZE_T,
        TAG_FSTUB,
    ) as *mut u8;
    if (*DiskInformation).Buffer.is_null() {
        ExFreePoolWithTag(DiskInformation as *mut core::ffi::c_void, TAG_FSTUB);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *DiskBuffer = DiskInformation;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FstubConvertExtendedToLayout(
    LayoutEx: PDRIVE_LAYOUT_INFORMATION_EX,
) -> PDRIVE_LAYOUT_INFORMATION {
    let mut i: u32;
    let DriveLayout: PDRIVE_LAYOUT_INFORMATION;

    if (*LayoutEx).PartitionStyle != PARTITION_STYLE_MBR {
        return core::ptr::null_mut();
    }

    DriveLayout = ExAllocatePoolWithTag(
        NonPagedPool,
        FIELD_OFFSET!(DRIVE_LAYOUT_INFORMATION, PartitionEntry)
            + (*LayoutEx).PartitionCount as usize * core::mem::size_of::<PARTITION_INFORMATION>(),
        TAG_FSTUB,
    ) as PDRIVE_LAYOUT_INFORMATION;
    
    if DriveLayout.is_null() {
        return core::ptr::null_mut();
    }

    (*DriveLayout).PartitionCount = (*LayoutEx).PartitionCount;
    (*DriveLayout).Signature = (*LayoutEx).__bindgen_anon_1.Mbr.Signature;

    i = 0;
    while i < (*LayoutEx).PartitionCount {
        (*DriveLayout).PartitionEntry[i as usize].StartingOffset = 
            (*LayoutEx).PartitionEntry[i as usize].StartingOffset;
        (*DriveLayout).PartitionEntry[i as usize].PartitionLength = 
            (*LayoutEx).PartitionEntry[i as usize].PartitionLength;
        (*DriveLayout).PartitionEntry[i as usize].HiddenSectors = 
            (*LayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.HiddenSectors;
        (*DriveLayout).PartitionEntry[i as usize].PartitionNumber = 
            (*LayoutEx).PartitionEntry[i as usize].PartitionNumber;
        (*DriveLayout).PartitionEntry[i as usize].PartitionType = 
            (*LayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.PartitionType;
        (*DriveLayout).PartitionEntry[i as usize].BootIndicator = 
            (*LayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.BootIndicator;
        (*DriveLayout).PartitionEntry[i as usize].RecognizedPartition = 
            (*LayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.RecognizedPartition;
        (*DriveLayout).PartitionEntry[i as usize].RewritePartition = 
            (*LayoutEx).PartitionEntry[i as usize].RewritePartition;
        i += 1;
    }

    DriveLayout
}

pub unsafe extern "system" fn FstubCopyEntryEFI(
    Entry: PEFI_PARTITION_ENTRY,
    Partition: PPARTITION_INFORMATION_EX,
    SectorSize: ULONG,
) {
    (*Entry).PartitionType = (*Partition).__bindgen_anon_1.Gpt.PartitionType;
    (*Entry).UniquePartition = (*Partition).__bindgen_anon_1.Gpt.PartitionId;
    (*Entry).StartingLBA = ((*Partition).StartingOffset.QuadPart as ULONGLONG) / SectorSize as ULONGLONG;
    (*Entry).EndingLBA = (((*Partition).StartingOffset.QuadPart + (*Partition).PartitionLength.QuadPart - 1) as ULONGLONG) / SectorSize as ULONGLONG;
    (*Entry).Attributes = (*Partition).__bindgen_anon_1.Gpt.Attributes;
    core::ptr::copy_nonoverlapping(
        (*Partition).__bindgen_anon_1.Gpt.Name.as_ptr(),
        (*Entry).Name.as_mut_ptr(),
        (*Entry).Name.len(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn FstubCreateDiskMBR(
    DeviceObject: PDEVICE_OBJECT,
    DiskInfo: PCREATE_DISK_MBR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disk: PDISK_INFORMATION = core::ptr::null_mut();
    let MasterBootRecord: PMASTER_BOOT_RECORD;

    Status = FstubAllocateDiskInformation(DeviceObject, &mut Disk, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = FstubReadSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        0u64,
        (*Disk).Buffer as PVOID,
    );
    if !NT_SUCCESS(Status) {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    MasterBootRecord = (*Disk).Buffer as PMASTER_BOOT_RECORD;
    (*MasterBootRecord).Signature = (*DiskInfo).Signature;
    RtlZeroMemory(
        core::ptr::addr_of_mut!((*MasterBootRecord).PartitionTable) as PVOID,
        (core::mem::size_of::<PARTITION_TABLE_ENTRY>() * NUM_PARTITION_TABLE_ENTRIES) as SIZE_T,
    );
    (*MasterBootRecord).MasterBootRecordMagic = BOOT_RECORD_SIGNATURE;

    Status = FstubWriteSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        0u64,
        (*Disk).Buffer as PVOID,
    );

    FstubFreeDiskInformation(Disk);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FstubCreateDiskEFI(
    DeviceObject: PDEVICE_OBJECT,
    DiskInfo: PCREATE_DISK_GPT
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disk: PDISK_INFORMATION = core::ptr::null_mut();
    let FirstUsableLBA: ULONGLONG;
    let LastUsableLBA: ULONGLONG;
    let mut MaxPartitionCount: ULONG;
    let SectorsForPartitions: ULONG;

    Status = FstubAllocateDiskInformation(DeviceObject, &mut Disk, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = FstubWriteBootSectorEFI(Disk);
    if !NT_SUCCESS(Status) {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    MaxPartitionCount = (*DiskInfo).MaxPartitionCount;
    FstubAdjustPartitionCount((*Disk).SectorSize, &mut MaxPartitionCount);

    SectorsForPartitions = (MaxPartitionCount * PARTITION_ENTRY_SIZE) / (*Disk).SectorSize;
    FirstUsableLBA = SectorsForPartitions as ULONGLONG + 2;
    LastUsableLBA = (*Disk).SectorCount - SectorsForPartitions as ULONGLONG - 1;

    Status = FstubWritePartitionTableEFI(
        Disk,
        (*DiskInfo).DiskId,
        MaxPartitionCount,
        FirstUsableLBA,
        LastUsableLBA,
        0u8,
        0,
        core::ptr::null_mut()
    );

    if NT_SUCCESS(Status) {
        Status = FstubWritePartitionTableEFI(
            Disk,
            (*DiskInfo).DiskId,
            MaxPartitionCount,
            FirstUsableLBA,
            LastUsableLBA,
            1u8,
            0,
            core::ptr::null_mut()
        );
    }

    FstubFreeDiskInformation(Disk);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FstubCreateDiskRaw(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disk: PDISK_INFORMATION = core::ptr::null_mut();
    let mut PartitionStyle: PARTITION_STYLE = core::mem::zeroed();
    let MasterBootRecord: PMASTER_BOOT_RECORD;

    Status = FstubAllocateDiskInformation(DeviceObject, &mut Disk, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = FstubDetectPartitionStyle(Disk, &mut PartitionStyle);
    if !NT_SUCCESS(Status) {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    Status = FstubReadSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        0u64,
        (*Disk).Buffer as PVOID
    );
    if !NT_SUCCESS(Status) {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    MasterBootRecord = (*Disk).Buffer as PMASTER_BOOT_RECORD;
    (*MasterBootRecord).Signature = 0;
    RtlZeroMemory(
        core::ptr::addr_of_mut!((*MasterBootRecord).PartitionTable) as PVOID,
        core::mem::size_of::<PARTITION_TABLE_ENTRY>()
    );
    (*MasterBootRecord).MasterBootRecordMagic = 0;

    Status = FstubWriteSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        0u64,
        (*Disk).Buffer as PVOID
    );

    if PartitionStyle != PARTITION_STYLE_GPT {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    RtlZeroMemory((*Disk).Buffer as PVOID, (*Disk).SectorSize as SIZE_T);

    Status = FstubWriteSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        1u64,
        (*Disk).Buffer as PVOID
    );

    if NT_SUCCESS(Status) {
        Status = FstubWriteSector(
            (*Disk).DeviceObject,
            (*Disk).SectorSize,
            (*Disk).SectorCount - 1u64,
            (*Disk).Buffer as PVOID
        );
    }

    FstubFreeDiskInformation(Disk);
    Status
}

unsafe fn FstubDbgGuidToString(Guid: PGUID, String: PCHAR) -> PCHAR {

    sprintf(
        String,
        b"{%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x}\0".as_ptr() as *const i8,
        (*Guid).Data1,
        (*Guid).Data2 as u32,
        (*Guid).Data3 as u32,
        (*Guid).Data4[0] as u32,
        (*Guid).Data4[1] as u32,
        (*Guid).Data4[2] as u32,
        (*Guid).Data4[3] as u32,
        (*Guid).Data4[4] as u32,
        (*Guid).Data4[5] as u32,
        (*Guid).Data4[6] as u32,
        (*Guid).Data4[7] as u32,
    );

    String
}

#[no_mangle]
pub unsafe extern "system" fn FstubDbgPrintDriveLayoutEx(DriveLayout: PDRIVE_LAYOUT_INFORMATION_EX) {
    let mut i: ULONG;
    let mut Guid: [CHAR; 39] = [0; 39];

    // DPRINT removed
    match (*DriveLayout).PartitionStyle {
        0 => {
            if (*DriveLayout).PartitionCount % 4 != 0 {
                // DPRINT removed
            }

            // DPRINT removed
            i = 0;
            while i < (*DriveLayout).PartitionCount {
                FstubDbgPrintPartitionEx((*DriveLayout).PartitionEntry.as_mut_ptr(), i);
                i += 1;
            }
        }
        1 => {
            FstubDbgGuidToString(
                core::ptr::addr_of_mut!((*DriveLayout).__bindgen_anon_1.Gpt.DiskId),
                Guid.as_mut_ptr(),
            );
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
            i = 0;
            while i < (*DriveLayout).PartitionCount {
                FstubDbgPrintPartitionEx((*DriveLayout).PartitionEntry.as_mut_ptr(), i);
                i += 1;
            }
        }
        _ => {
            // DPRINT removed
        }
    }
}

pub unsafe fn FstubDbgPrintPartitionEx(
    PartitionEntry: PPARTITION_INFORMATION_EX,
    PartitionNumber: ULONG,
) {
    let mut Guid: [CHAR; 39] = [0; 39];

    // DPRINT removed

    match (*PartitionEntry.offset(PartitionNumber as isize)).PartitionStyle {
        0 => {
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
        }
        1 => {
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
            FstubDbgGuidToString(
                core::ptr::addr_of_mut!(
                    (*PartitionEntry.offset(PartitionNumber as isize))
                        .__bindgen_anon_1.Gpt
                        .PartitionType
                ),
                Guid.as_mut_ptr(),
            );
            // DPRINT removed
            FstubDbgGuidToString(
                core::ptr::addr_of_mut!(
                    (*PartitionEntry.offset(PartitionNumber as isize))
                        .__bindgen_anon_1.Gpt
                        .PartitionId
                ),
                Guid.as_mut_ptr(),
            );
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
        }
        _ => {
            // DPRINT removed
        }
    }
}

pub unsafe fn FstubDbgPrintSetPartitionEx(
    PartitionEntry: PSET_PARTITION_INFORMATION_EX,
    PartitionNumber: ULONG,
) {
    let mut Guid: [CHAR; 39] = [0; 39];

    // DPRINT removed
    // DPRINT removed
    
    match (*PartitionEntry).PartitionStyle {
        PARTITION_STYLE_MBR => {
            // DPRINT removed
        }
        PARTITION_STYLE_GPT => {
            FstubDbgGuidToString(
                core::ptr::addr_of_mut!((*PartitionEntry).__bindgen_anon_1.Gpt.PartitionType),
                Guid.as_mut_ptr(),
            );
            // DPRINT removed
            FstubDbgGuidToString(
                core::ptr::addr_of_mut!((*PartitionEntry).__bindgen_anon_1.Gpt.PartitionId),
                Guid.as_mut_ptr(),
            );
            // DPRINT removed
            // DPRINT removed
            // DPRINT removed
        }
        _ => {
            // DPRINT removed
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FstubDetectPartitionStyle(
    Disk: PDISK_INFORMATION,
    PartitionStyle: *mut PARTITION_STYLE
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let PartitionDescriptor: PPARTITION_DESCRIPTOR;

    Status = FstubReadSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        0,
        (*Disk).Buffer as PVOID
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    PartitionDescriptor = (*Disk).Buffer.offset(PARTITION_TABLE_OFFSET as isize) as PPARTITION_DESCRIPTOR;

    if *((*Disk).Buffer.offset(BOOT_SIGNATURE_OFFSET as isize) as PUINT16) != BOOT_RECORD_SIGNATURE {
        *PartitionStyle = PARTITION_STYLE_RAW;
    } else if (*PartitionDescriptor.offset(0)).PartitionType == EFI_PMBR_OSTYPE_EFI &&
              (*PartitionDescriptor.offset(1)).PartitionType == 0 &&
              (*PartitionDescriptor.offset(2)).PartitionType == 0 &&
              (*PartitionDescriptor.offset(3)).PartitionType == 0 {
        *PartitionStyle = PARTITION_STYLE_GPT;
    } else {
        *PartitionStyle = PARTITION_STYLE_MBR;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FstubFreeDiskInformation(DiskBuffer: PDISK_INFORMATION) {
    if !DiskBuffer.is_null() {
        if !(*DiskBuffer).Buffer.is_null() {
            ExFreePoolWithTag((*DiskBuffer).Buffer as PVOID, TAG_FSTUB);
        }
        ExFreePoolWithTag(DiskBuffer as PVOID, TAG_FSTUB);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FstubGetDiskGeometry(
    DeviceObject: PDEVICE_OBJECT,
    Geometry: PDISK_GEOMETRY_EX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut DiskGeometry: PDISK_GEOMETRY_EX = core::ptr::null_mut();
    let mut IoStatusBlock: PIO_STATUS_BLOCK = core::ptr::null_mut();

    'Cleanup: loop {
        DiskGeometry = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<DISK_GEOMETRY_EX>() as SIZE_T,
            TAG_FSTUB,
        ) as PDISK_GEOMETRY_EX;
        if DiskGeometry.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Cleanup;
        }

        IoStatusBlock = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_IO_STATUS_BLOCK>() as SIZE_T,
            TAG_FSTUB,
        ) as PIO_STATUS_BLOCK;
        if IoStatusBlock.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Cleanup;
        }

        Event = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KEVENT>() as SIZE_T,
            TAG_FSTUB,
        ) as PKEVENT;
        if Event.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Cleanup;
        }

        KeInitializeEvent(Event, _EVENT_TYPE_NotificationEvent, 0u8);

        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
            DeviceObject,
            core::ptr::null_mut(),
            0,
            DiskGeometry as *mut core::ffi::c_void,
            core::mem::size_of::<DISK_GEOMETRY_EX>() as u32,
            0u8,
            Event,
            IoStatusBlock,
        );
        if Irp.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'Cleanup;
        }

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                Event as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = (*IoStatusBlock).__bindgen_anon_1.Status;
        }

        if NT_SUCCESS(Status) {
            *Geometry = *DiskGeometry;
        }

        break 'Cleanup;
    }

    if !DiskGeometry.is_null() {
        ExFreePoolWithTag(DiskGeometry as *mut core::ffi::c_void, TAG_FSTUB);
    }

    if !IoStatusBlock.is_null() {
        ExFreePoolWithTag(IoStatusBlock as *mut core::ffi::c_void, TAG_FSTUB);
    }

    if !Event.is_null() {
        ExFreePoolWithTag(Event as *mut core::ffi::c_void, TAG_FSTUB);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FstubReadHeaderEFI(
    Disk: PDISK_INFORMATION,
    ReadBackupTable: BOOLEAN,
    HeaderBuffer: *mut PEFI_PARTITION_HEADER
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Sector: PUCHAR = core::ptr::null_mut();
    let StartingSector: ULONGLONG;
    let EFIHeader: PEFI_PARTITION_HEADER;
    let mut i: ULONG;
    let HeaderCRC32: ULONG;
    let mut PreviousCRC32: ULONG;
    let SectoredPartitionEntriesSize: ULONG;
    let LonelyPartitions: ULONG;

    if ReadBackupTable != 0 {
        StartingSector = (*Disk).SectorCount - 1u64;
    } else {
        StartingSector = 1u64;
    }

    Status = FstubReadSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        StartingSector,
        (*Disk).Buffer as PVOID
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    EFIHeader = (*Disk).Buffer as PEFI_PARTITION_HEADER;

    if (*EFIHeader).Signature != EFI_HEADER_SIGNATURE ||
        (*EFIHeader).Revision != EFI_HEADER_REVISION_1 ||
        (*EFIHeader).HeaderSize != core::mem::size_of::<EFI_PARTITION_HEADER>() as ULONG {
        return STATUS_DISK_CORRUPT_ERROR;
    }

    HeaderCRC32 = (*EFIHeader).HeaderCRC32;
    (*EFIHeader).HeaderCRC32 = 0;
    if RtlComputeCrc32(0, (*Disk).Buffer as *const u8, core::mem::size_of::<EFI_PARTITION_HEADER>() as ULONG) != HeaderCRC32 {
        return STATUS_DISK_CORRUPT_ERROR;
    }
    (*EFIHeader).HeaderCRC32 = HeaderCRC32;

    if (*EFIHeader).MyLBA != StartingSector {
        return STATUS_DISK_CORRUPT_ERROR;
    }

    Sector = ExAllocatePoolWithTag(
        NonPagedPool,
        (*Disk).SectorSize as SIZE_T,
        TAG_FSTUB
    ) as PUCHAR;
    if Sector.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    SectoredPartitionEntriesSize = ((*EFIHeader).NumberOfEntries * PARTITION_ENTRY_SIZE) / (*Disk).SectorSize;
    i = 0;
    PreviousCRC32 = 0;
    while i < SectoredPartitionEntriesSize {
        Status = FstubReadSector(
            (*Disk).DeviceObject,
            (*Disk).SectorSize,
            (*EFIHeader).PartitionEntryLBA + i as ULONGLONG,
            Sector as PVOID
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(Sector as PVOID, TAG_FSTUB);
            return Status;
        }

        PreviousCRC32 = RtlComputeCrc32(PreviousCRC32, Sector as *const u8, (*Disk).SectorSize);
        i += 1;
    }

    LonelyPartitions = ((*EFIHeader).NumberOfEntries * PARTITION_ENTRY_SIZE) % (*Disk).SectorSize;
    if LonelyPartitions != 0 {
        Status = FstubReadSector(
            (*Disk).DeviceObject,
            (*Disk).SectorSize,
            (*EFIHeader).PartitionEntryLBA + i as ULONGLONG,
            Sector as PVOID
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(Sector as PVOID, TAG_FSTUB);
            return Status;
        }

        i = 0;
        while i < LonelyPartitions {
            PreviousCRC32 = RtlComputeCrc32(
                PreviousCRC32,
                Sector.add((i * PARTITION_ENTRY_SIZE) as usize) as *const u8,
                PARTITION_ENTRY_SIZE
            );
            i += 1;
        }
    }

    ExFreePoolWithTag(Sector as PVOID, TAG_FSTUB);

    if PreviousCRC32 == (*EFIHeader).PartitionEntryCRC32 {
        *HeaderBuffer = EFIHeader;
        return STATUS_SUCCESS;
    } else {
        return STATUS_DISK_CORRUPT_ERROR;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FstubReadPartitionTableEFI(
    Disk: PDISK_INFORMATION,
    ReadBackupTable: BOOLEAN,
    DriveLayout: *mut PDRIVE_LAYOUT_INFORMATION_EX
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let NumberOfEntries: ULONG;
    let mut EfiHeader: PEFI_PARTITION_HEADER = core::ptr::null_mut();
    let PartitionEntry: EFI_PARTITION_ENTRY;
    let PartitionEntryLBA: ULONGLONG;
    let mut DriveLayoutEx: PDRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let mut i: ULONG;
    let mut PartitionCount: ULONG;
    let mut PartitionIndex: ULONG;
    let PartitionsPerSector: ULONG;

    *DriveLayout = core::ptr::null_mut();

    Status = FstubReadHeaderEFI(Disk, ReadBackupTable, &mut EfiHeader);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    NumberOfEntries = (*EfiHeader).NumberOfEntries;

    DriveLayoutEx = ExAllocatePoolWithTag(
        NonPagedPool,
        FIELD_OFFSET!(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry)
            + (*EfiHeader).NumberOfEntries as usize * core::mem::size_of::<PARTITION_INFORMATION_EX>(),
        TAG_FSTUB
    ) as PDRIVE_LAYOUT_INFORMATION_EX;
    if DriveLayoutEx.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DriveLayoutEx).PartitionStyle = PARTITION_STYLE_GPT;
    (*DriveLayoutEx).__bindgen_anon_1.Gpt.StartingUsableOffset.QuadPart = 
        ((*EfiHeader).FirstUsableLBA.wrapping_mul((*Disk).SectorSize as ULONGLONG)) as LONGLONG;
    (*DriveLayoutEx).__bindgen_anon_1.Gpt.UsableLength.QuadPart = 
        (((*EfiHeader).LastUsableLBA.wrapping_sub((*EfiHeader).FirstUsableLBA)).wrapping_mul((*Disk).SectorSize as ULONGLONG)) as LONGLONG;
    (*DriveLayoutEx).__bindgen_anon_1.Gpt.MaxPartitionCount = (*EfiHeader).NumberOfEntries;
    (*DriveLayoutEx).__bindgen_anon_1.Gpt.DiskId = (*EfiHeader).DiskGUID;

    let PartitionEntryLBA = (*EfiHeader).PartitionEntryLBA;
    let PartitionsPerSector = (*Disk).SectorSize / PARTITION_ENTRY_SIZE;

    i = 0;
    PartitionCount = 0;
    PartitionIndex = PartitionsPerSector;
    while i < NumberOfEntries {
        if PartitionIndex == PartitionsPerSector {
            Status = FstubReadSector(
                (*Disk).DeviceObject,
                (*Disk).SectorSize,
                PartitionEntryLBA.wrapping_add((i / PartitionsPerSector) as ULONGLONG),
                (*Disk).Buffer as PVOID
            );
            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(DriveLayoutEx as PVOID, TAG_FSTUB);
                return Status;
            }

            PartitionIndex = 0;
        }

        let PartitionEntry = (*(((*Disk).Buffer as PEFI_PARTITION_ENTRY).offset(PartitionIndex as isize)));
        PartitionIndex = PartitionIndex.wrapping_add(1);

        if PartitionEntry.PartitionType.Data1 == 0 &&
           PartitionEntry.PartitionType.Data2 == 0 &&
           PartitionEntry.PartitionType.Data3 == 0 &&
           (*(PartitionEntry.PartitionType.Data4.as_ptr() as PULONGLONG)) == 0 {
            i = i.wrapping_add(1);
            continue;
        }

        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].StartingOffset.QuadPart = 
            (PartitionEntry.StartingLBA.wrapping_mul((*Disk).SectorSize as ULONGLONG)) as LONGLONG;
        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].PartitionLength.QuadPart = 
            ((PartitionEntry.EndingLBA.wrapping_sub(PartitionEntry.StartingLBA).wrapping_add(1))
             .wrapping_mul((*Disk).SectorSize as ULONGLONG)) as LONGLONG;
        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].PartitionNumber = PartitionCount.wrapping_add(1);
        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].RewritePartition = 0u8;
        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].PartitionStyle = PARTITION_STYLE_GPT;
        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].__bindgen_anon_1.Gpt.PartitionType = PartitionEntry.PartitionType;
        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].__bindgen_anon_1.Gpt.PartitionId = PartitionEntry.UniquePartition;
        (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].__bindgen_anon_1.Gpt.Attributes = PartitionEntry.Attributes;
        RtlCopyMemory(
            (*DriveLayoutEx).PartitionEntry[PartitionCount as usize].__bindgen_anon_1.Gpt.Name.as_mut_ptr() as PVOID,
            PartitionEntry.Name.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&PartitionEntry.Name)
        );

        PartitionCount = PartitionCount.wrapping_add(1);
        i = i.wrapping_add(1);
    }
    (*DriveLayoutEx).PartitionCount = PartitionCount;

    *DriveLayout = DriveLayoutEx;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FstubReadPartitionTableMBR(
    Disk: PDISK_INFORMATION,
    ReturnRecognizedPartitions: BOOLEAN,
    ReturnedDriveLayout: *mut PDRIVE_LAYOUT_INFORMATION_EX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut DriveLayout: PDRIVE_LAYOUT_INFORMATION = core::ptr::null_mut();
    let mut DriveLayoutEx: PDRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();

    *ReturnedDriveLayout = core::ptr::null_mut();

    Status = IoReadPartitionTable(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        ReturnRecognizedPartitions,
        &mut DriveLayout,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    DriveLayoutEx = ExAllocatePoolWithTag(
        NonPagedPool,
        FIELD_OFFSET!(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry)
            + (*DriveLayout).PartitionCount as usize * core::mem::size_of::<PARTITION_INFORMATION_EX>(),
        TAG_FSTUB,
    ) as PDRIVE_LAYOUT_INFORMATION_EX;
    if DriveLayoutEx.is_null() {
        ExFreePool(DriveLayout as *mut core::ffi::c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DriveLayoutEx).PartitionStyle = PARTITION_STYLE_MBR;
    (*DriveLayoutEx).PartitionCount = (*DriveLayout).PartitionCount;
    (*DriveLayoutEx).__bindgen_anon_1.Mbr.Signature = (*DriveLayout).Signature;

    i = 0;
    while i < (*DriveLayout).PartitionCount {
        (*DriveLayoutEx).PartitionEntry[i as usize].PartitionStyle = PARTITION_STYLE_MBR;
        (*DriveLayoutEx).PartitionEntry[i as usize].StartingOffset = (*DriveLayout).PartitionEntry[i as usize].StartingOffset;
        (*DriveLayoutEx).PartitionEntry[i as usize].PartitionLength = (*DriveLayout).PartitionEntry[i as usize].PartitionLength;
        (*DriveLayoutEx).PartitionEntry[i as usize].PartitionNumber = (*DriveLayout).PartitionEntry[i as usize].PartitionNumber;
        (*DriveLayoutEx).PartitionEntry[i as usize].RewritePartition = (*DriveLayout).PartitionEntry[i as usize].RewritePartition;
        (*DriveLayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.PartitionType = (*DriveLayout).PartitionEntry[i as usize].PartitionType;
        (*DriveLayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.BootIndicator = (*DriveLayout).PartitionEntry[i as usize].BootIndicator;
        (*DriveLayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.RecognizedPartition = (*DriveLayout).PartitionEntry[i as usize].RecognizedPartition;
        (*DriveLayoutEx).PartitionEntry[i as usize].__bindgen_anon_1.Mbr.HiddenSectors = (*DriveLayout).PartitionEntry[i as usize].HiddenSectors;
        i += 1;
    }

    *ReturnedDriveLayout = DriveLayoutEx;
    ExFreePool(DriveLayout as *mut core::ffi::c_void);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FstubReadSector(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    StartingSector: ULONGLONG,
    Buffer: PVOID
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut StartingOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    StartingOffset.QuadPart = StartingSector.wrapping_mul(SectorSize as ULONGLONG) as LONGLONG;

    KeInitializeEvent(
        &mut Event,
        _EVENT_TYPE_NotificationEvent,
        0u8
    );

    let Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_READ as ULONG,
        DeviceObject,
        Buffer,
        SectorSize,
        &mut StartingOffset,
        &mut Event,
        &mut IoStatusBlock
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let IoStackLocation: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);
    (*IoStackLocation).Flags |= SL_OVERRIDE_VERIFY_VOLUME;

    let mut Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut()
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FstubSetPartitionInformationEFI(
    Disk: PDISK_INFORMATION,
    PartitionNumber: ULONG,
    PartitionInfo: *mut SET_PARTITION_INFORMATION_GPT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Layout: PDRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let mut PartitionNumber = PartitionNumber;

    if PartitionNumber == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Status = IoReadPartitionTableEx((*Disk).DeviceObject, core::ptr::addr_of_mut!(Layout));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    PartitionNumber -= 1;
    if (*Layout).PartitionCount <= PartitionNumber {
        ExFreePool(Layout as *mut core::ffi::c_void);
        return STATUS_INVALID_PARAMETER;
    }

    (*Layout).PartitionEntry[PartitionNumber as usize].__bindgen_anon_1.Gpt.PartitionType = (*PartitionInfo).PartitionType;
    (*Layout).PartitionEntry[PartitionNumber as usize].__bindgen_anon_1.Gpt.PartitionId = (*PartitionInfo).PartitionId;
    (*Layout).PartitionEntry[PartitionNumber as usize].__bindgen_anon_1.Gpt.Attributes = (*PartitionInfo).Attributes;
    RtlCopyMemory(
        (*Layout).PartitionEntry[PartitionNumber as usize].__bindgen_anon_1.Gpt.Name.as_mut_ptr() as *mut core::ffi::c_void,
        (*PartitionInfo).Name.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&(*PartitionInfo).Name),
    );

    Status = IoWritePartitionTableEx((*Disk).DeviceObject, Layout);

    ExFreePool(Layout as *mut core::ffi::c_void);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FstubVerifyPartitionTableEFI(
    Disk: PDISK_INFORMATION,
    FixErrors: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let EFIHeader: PEFI_PARTITION_HEADER;
    let mut ReadEFIHeader: PEFI_PARTITION_HEADER = core::ptr::null_mut();
    let mut PrimaryValid: BOOLEAN = 0u8;
    let mut BackupValid: BOOLEAN = 0u8;
    let WriteBackup: BOOLEAN;
    let ReadPosition: ULONGLONG;
    let WritePosition: ULONGLONG;
    let SectorsForPartitions: ULONGLONG;
    let mut PartitionIndex: ULONGLONG;

    EFIHeader = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<EFI_PARTITION_HEADER>() as SIZE_T,
        TAG_FSTUB,
    ) as PEFI_PARTITION_HEADER;
    if EFIHeader.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = FstubReadHeaderEFI(Disk, 0u8, &mut ReadEFIHeader);
    if NT_SUCCESS(Status) {
        PrimaryValid = 1u8;
        core::ptr::copy_nonoverlapping(
            ReadEFIHeader as *const u8,
            EFIHeader as *mut u8,
            core::mem::size_of::<EFI_PARTITION_HEADER>(),
        );
    }

    Status = FstubReadHeaderEFI(Disk, 1u8, &mut ReadEFIHeader);
    if NT_SUCCESS(Status) {
        BackupValid = 1u8;
        core::ptr::copy_nonoverlapping(
            ReadEFIHeader as *const u8,
            EFIHeader as *mut u8,
            core::mem::size_of::<EFI_PARTITION_HEADER>(),
        );
    }

    if PrimaryValid != 0 && BackupValid != 0 {
        ExFreePoolWithTag(EFIHeader as PVOID, TAG_FSTUB);
        return STATUS_SUCCESS;
    }

    if (PrimaryValid == 0 && BackupValid == 0) || FixErrors == 0 {
        ExFreePoolWithTag(EFIHeader as PVOID, TAG_FSTUB);
        return STATUS_DISK_CORRUPT_ERROR;
    }

    SectorsForPartitions = (((*EFIHeader).NumberOfEntries as ULONGLONG)
        .wrapping_mul(PARTITION_ENTRY_SIZE as ULONGLONG)
        .wrapping_add((*Disk).SectorSize as ULONGLONG)
        .wrapping_sub(1))
        / ((*Disk).SectorSize as ULONGLONG);

    if PrimaryValid != 0 {
        WriteBackup = 1u8;
        WritePosition = ((*Disk).SectorCount).wrapping_sub(SectorsForPartitions);
        ReadPosition = 2u64;
    } else {
        WriteBackup = 0u8;
        WritePosition = 2u64;
        ReadPosition = ((*Disk).SectorCount).wrapping_sub(SectorsForPartitions);
    }

    PartitionIndex = 0u64;

    'Cleanup: loop {
        if SectorsForPartitions <= 0 {
            Status = FstubWriteHeaderEFI(
                Disk,
                SectorsForPartitions as ULONG,
                (*EFIHeader).DiskGUID,
                (*EFIHeader).NumberOfEntries,
                (*EFIHeader).FirstUsableLBA,
                (*EFIHeader).LastUsableLBA,
                (*EFIHeader).PartitionEntryCRC32,
                WriteBackup,
            );

            break 'Cleanup;
        }

        loop {
            if PartitionIndex >= SectorsForPartitions {
                break;
            }

            Status = FstubReadSector(
                (*Disk).DeviceObject,
                (*Disk).SectorSize,
                ReadPosition.wrapping_add(PartitionIndex),
                (*Disk).Buffer as PVOID,
            );
            if !NT_SUCCESS(Status) {
                break 'Cleanup;
            }

            Status = FstubWriteSector(
                (*Disk).DeviceObject,
                (*Disk).SectorSize,
                WritePosition.wrapping_add(PartitionIndex),
                (*Disk).Buffer as PVOID,
            );
            if !NT_SUCCESS(Status) {
                break 'Cleanup;
            }

            PartitionIndex = PartitionIndex.wrapping_add(1);
        }

        Status = FstubWriteHeaderEFI(
            Disk,
            SectorsForPartitions as ULONG,
            (*EFIHeader).DiskGUID,
            (*EFIHeader).NumberOfEntries,
            (*EFIHeader).FirstUsableLBA,
            (*EFIHeader).LastUsableLBA,
            (*EFIHeader).PartitionEntryCRC32,
            WriteBackup,
        );

        break 'Cleanup;
    }

    ExFreePoolWithTag(EFIHeader as PVOID, TAG_FSTUB);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FstubWriteBootSectorEFI(Disk: PDISK_INFORMATION) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Signature: ULONG = 0;
    let MasterBootRecord: PMASTER_BOOT_RECORD;

    Status = FstubReadSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        0u64,
        (*Disk).Buffer as PVOID,
    );
    MasterBootRecord = (*Disk).Buffer as PMASTER_BOOT_RECORD;

    if NT_SUCCESS(Status) && (*MasterBootRecord).MasterBootRecordMagic == BOOT_RECORD_SIGNATURE {
        Signature = (*MasterBootRecord).Signature;
    }

    RtlZeroMemory(MasterBootRecord as PVOID, (*Disk).SectorSize as SIZE_T);

    (*MasterBootRecord).Signature = Signature;
    (*MasterBootRecord).PartitionTable[0].StartSector = 2;
    (*MasterBootRecord).PartitionTable[0].SystemIndicator = EFI_PMBR_OSTYPE_EFI;
    (*MasterBootRecord).PartitionTable[0].EndHead = 0xFF;
    (*MasterBootRecord).PartitionTable[0].EndSector = 0xFF;
    (*MasterBootRecord).PartitionTable[0].EndCylinder = 0xFF;
    (*MasterBootRecord).PartitionTable[0].SectorCountBeforePartition = 1;
    (*MasterBootRecord).PartitionTable[0].PartitionSectorCount = 0xFFFFFFFF;
    (*MasterBootRecord).MasterBootRecordMagic = BOOT_RECORD_SIGNATURE;

    FstubWriteSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        0,
        (*Disk).Buffer as PVOID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn FstubWriteEntryEFI(
    Disk: PDISK_INFORMATION,
    PartitionsSizeSector: ULONG,
    PartitionEntryNumber: ULONG,
    PartitionEntry: PEFI_PARTITION_ENTRY,
    WriteBackupTable: BOOLEAN,
    ForceWrite: BOOLEAN,
    PartitionEntryCRC32: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Offset: ULONG;
    let FirstEntryLBA: ULONGLONG;

    if WriteBackupTable == 0 {
        FirstEntryLBA = 2u64;
    } else {
        FirstEntryLBA = (*Disk).SectorCount - PartitionsSizeSector as ULONGLONG - 1;
    }

    RtlCopyMemory(
        ((*Disk).Buffer as ULONG_PTR + ((PartitionEntryNumber * PARTITION_ENTRY_SIZE) % (*Disk).SectorSize) as ULONG_PTR) as PVOID,
        PartitionEntry as PVOID,
        core::mem::size_of::<EFI_PARTITION_ENTRY>(),
    );

    Offset = (PartitionEntryNumber * PARTITION_ENTRY_SIZE) % (*Disk).SectorSize + PARTITION_ENTRY_SIZE;

    if Offset == (*Disk).SectorSize || ForceWrite != 0 {
        Status = FstubWriteSector(
            (*Disk).DeviceObject,
            (*Disk).SectorSize,
            FirstEntryLBA + ((PartitionEntryNumber * PARTITION_ENTRY_SIZE) / (*Disk).SectorSize) as ULONGLONG,
            (*Disk).Buffer as PVOID,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        RtlZeroMemory((*Disk).Buffer as PVOID, (*Disk).SectorSize as usize);
    }

    if !PartitionEntryCRC32.is_null() {
        *PartitionEntryCRC32 = RtlComputeCrc32(
            *PartitionEntryCRC32,
            PartitionEntry as PUCHAR,
            PARTITION_ENTRY_SIZE,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FstubWriteHeaderEFI(
    Disk: PDISK_INFORMATION,
    PartitionsSizeSector: ULONG,
    DiskGUID: GUID,
    NumberOfEntries: ULONG,
    FirstUsableLBA: ULONGLONG,
    LastUsableLBA: ULONGLONG,
    PartitionEntryCRC32: ULONG,
    WriteBackupTable: BOOLEAN,
) -> NTSTATUS {
    let EFIHeader: PEFI_PARTITION_HEADER;

    EFIHeader = (*Disk).Buffer as PEFI_PARTITION_HEADER;

    (*EFIHeader).Signature = EFI_HEADER_SIGNATURE;
    (*EFIHeader).Revision = EFI_HEADER_REVISION_1;
    (*EFIHeader).HeaderSize = core::mem::size_of::<EFI_PARTITION_HEADER>() as ULONG;
    (*EFIHeader).HeaderCRC32 = 0;
    (*EFIHeader).Reserved = 0;

    if WriteBackupTable == 0 {
        (*EFIHeader).MyLBA = 1u64;
        (*EFIHeader).AlternateLBA = (*Disk).SectorCount - 1u64;
    } else {
        (*EFIHeader).MyLBA = (*Disk).SectorCount - 1u64;
        (*EFIHeader).AlternateLBA = 1u64;
    }

    (*EFIHeader).FirstUsableLBA = FirstUsableLBA;
    (*EFIHeader).LastUsableLBA = LastUsableLBA;
    (*EFIHeader).DiskGUID = DiskGUID;

    if WriteBackupTable == 0 {
        (*EFIHeader).PartitionEntryLBA = (*EFIHeader).MyLBA + 1u64;
    } else {
        (*EFIHeader).PartitionEntryLBA = (*EFIHeader).MyLBA - PartitionsSizeSector as ULONGLONG;
    }

    (*EFIHeader).NumberOfEntries = NumberOfEntries;
    (*EFIHeader).SizeOfPartitionEntry = PARTITION_ENTRY_SIZE;
    (*EFIHeader).PartitionEntryCRC32 = PartitionEntryCRC32;

    (*EFIHeader).HeaderCRC32 = RtlComputeCrc32(
        0,
        EFIHeader as PUCHAR,
        core::mem::size_of::<EFI_PARTITION_HEADER>() as ULONG,
    );

    FstubWriteSector(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        (*EFIHeader).MyLBA,
        (*Disk).Buffer as PVOID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn FstubWritePartitionTableEFI(
    Disk: PDISK_INFORMATION,
    DiskGUID: GUID,
    MaxPartitionCount: ULONG,
    FirstUsableLBA: ULONGLONG,
    LastUsableLBA: ULONGLONG,
    WriteBackupTable: BOOLEAN,
    PartitionCount: ULONG,
    PartitionEntries: PPARTITION_INFORMATION_EX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Entry: EFI_PARTITION_ENTRY = core::mem::zeroed();
    let mut i: ULONG;
    let mut WrittenPartitions: ULONG;
    let SectoredPartitionEntriesSize: ULONG;
    let mut PartitionEntryCRC32: ULONG;

    PartitionEntryCRC32 = 0;
    SectoredPartitionEntriesSize = (MaxPartitionCount * PARTITION_ENTRY_SIZE) / (*Disk).SectorSize;

    i = 0;
    WrittenPartitions = 0;
    while i < PartitionCount {
        if (*PartitionEntries.offset(i as isize)).__bindgen_anon_1.Gpt.PartitionType.Data1 == 0
            && (*PartitionEntries.offset(i as isize)).__bindgen_anon_1.Gpt.PartitionType.Data2 == 0
            && (*PartitionEntries.offset(i as isize)).__bindgen_anon_1.Gpt.PartitionType.Data3 == 0
            && (*((*PartitionEntries.offset(i as isize)).__bindgen_anon_1.Gpt.PartitionType.Data4.as_ptr() as PULONGLONG)) == 0
        {
            i += 1;
            continue;
        }

        FstubCopyEntryEFI(
            &mut Entry,
            &mut *PartitionEntries.offset(i as isize),
            (*Disk).SectorSize,
        );
        Status = FstubWriteEntryEFI(
            Disk,
            SectoredPartitionEntriesSize,
            WrittenPartitions,
            &mut Entry,
            WriteBackupTable,
            0u8,
            &mut PartitionEntryCRC32,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
        WrittenPartitions += 1;
        i += 1;
    }

    RtlZeroMemory(
        &mut Entry as *mut _ as PVOID,
        core::mem::size_of::<EFI_PARTITION_ENTRY>() as SIZE_T,
    );
    i = WrittenPartitions;
    while i < MaxPartitionCount {
        Status = FstubWriteEntryEFI(
            Disk,
            SectoredPartitionEntriesSize,
            i,
            &mut Entry,
            WriteBackupTable,
            0u8,
            &mut PartitionEntryCRC32,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
        i += 1;
    }

    return FstubWriteHeaderEFI(
        Disk,
        SectoredPartitionEntriesSize,
        DiskGUID,
        MaxPartitionCount,
        FirstUsableLBA,
        LastUsableLBA,
        PartitionEntryCRC32,
        WriteBackupTable,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FstubWritePartitionTableMBR(
    Disk: PDISK_INFORMATION,
    LayoutEx: PDRIVE_LAYOUT_INFORMATION_EX,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let DriveLayout: PDRIVE_LAYOUT_INFORMATION;

    DriveLayout = FstubConvertExtendedToLayout(LayoutEx);
    if DriveLayout.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoWritePartitionTable(
        (*Disk).DeviceObject,
        (*Disk).SectorSize,
        (*Disk).DiskGeometry.Geometry.SectorsPerTrack,
        (*Disk).DiskGeometry.Geometry.TracksPerCylinder,
        DriveLayout,
    );

    ExFreePoolWithTag(DriveLayout as PVOID, TAG_FSTUB);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FstubWriteSector(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    StartingSector: ULONGLONG,
    Buffer: PVOID,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut StartingOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    StartingOffset.QuadPart = StartingSector.wrapping_mul(SectorSize as ULONGLONG) as LONGLONG;

    KeInitializeEvent(
        &mut Event,
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );

    let Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_WRITE as u32,
        DeviceObject,
        Buffer,
        SectorSize,
        &mut StartingOffset,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let IoStackLocation: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);
    (*IoStackLocation).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;

    let mut Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateDisk(
    DeviceObject: PDEVICE_OBJECT,
    Disk: PCREATE_DISK,
) -> NTSTATUS {
    let PartitionStyle: PARTITION_STYLE;

    PartitionStyle = if !Disk.is_null() {
        (*Disk).PartitionStyle
    } else {
        PARTITION_STYLE_RAW
    };

    match PartitionStyle {
        PARTITION_STYLE_MBR => FstubCreateDiskMBR(DeviceObject, core::ptr::addr_of_mut!((*Disk).__bindgen_anon_1.Mbr)),
        PARTITION_STYLE_GPT => FstubCreateDiskEFI(DeviceObject, core::ptr::addr_of_mut!((*Disk).__bindgen_anon_1.Gpt)),
        PARTITION_STYLE_RAW => FstubCreateDiskRaw(DeviceObject),
        _ => STATUS_NOT_SUPPORTED,
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoGetBootDiskInformation(
    BootDiskInformation: PBOOTDISK_INFORMATION,
    Size: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut NextEntry: PLIST_ENTRY;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DiskGeometry: DISK_GEOMETRY = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceStringW: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Buffer: [i8; 128] = [0; 128];
    let mut ArcBuffer: [i8; 128] = [0; 128];
    let SingleDisk: BOOLEAN;
    let IsBootDiskInfoEx: BOOLEAN;
    let mut ArcDiskSignature: PARC_DISK_SIGNATURE;
    let ArcDiskInformation: PARC_DISK_INFORMATION;
    let mut PartitionInformation: PARTITION_INFORMATION_EX = core::mem::zeroed();
    let mut DriveLayout: PDRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let DiskCount: ULONG;
    let mut DiskNumber: ULONG;
    let mut Signature: ULONG = 0;
    let mut PartitionNumber: ULONG;
    let mut ArcBootString: ANSI_STRING = core::mem::zeroed();
    let mut ArcSystemString: ANSI_STRING = core::mem::zeroed();
    let mut DeviceStringA: ANSI_STRING = core::mem::zeroed();
    let mut ArcNameStringA: ANSI_STRING = core::mem::zeroed();

    if IopLoaderBlock.is_null() {
        return 0xC0000100u32 as NTSTATUS;
    }

    if Size < core::mem::size_of::<BOOTDISK_INFORMATION>() as ULONG {
        return 0xC000000Du32 as NTSTATUS;
    }

    ArcDiskInformation = (*IopLoaderBlock).ArcDiskInformation;
    SingleDisk = if (*(*ArcDiskInformation).DiskSignatureListHead.Flink).Flink == 
        core::ptr::addr_of_mut!((*ArcDiskInformation).DiskSignatureListHead) { 1u8 } else { 0u8 };

    IsBootDiskInfoEx = if Size >= core::mem::size_of::<BOOTDISK_INFORMATION_EX>() as ULONG { 1u8 } else { 0u8 };
    RtlInitAnsiString(core::ptr::addr_of_mut!(ArcBootString), (*IopLoaderBlock).ArcBootDeviceName);
    RtlInitAnsiString(core::ptr::addr_of_mut!(ArcSystemString), (*IopLoaderBlock).ArcHalDeviceName);
    DiskCount = (*IoGetConfigurationInformation()).DiskCount;

    if DiskCount == 0 {
        return STATUS_SUCCESS;
    }

    DiskNumber = 0;
    while DiskNumber < DiskCount {
        sprintf(
            Buffer.as_mut_ptr(),
            b"\\Device\\Harddisk%lu\\Partition0\0".as_ptr() as *const i8,
            DiskNumber,
        );
        RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
        Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(DeviceStringW), core::ptr::addr_of_mut!(DeviceStringA), 1u8);
        if !NT_SUCCESS(Status) {
            DiskNumber += 1;
            continue;
        }

        Status = IoGetDeviceObjectPointer(
            core::ptr::addr_of_mut!(DeviceStringW),
            0x80,
            core::ptr::addr_of_mut!(FileObject),
            core::ptr::addr_of_mut!(DeviceObject),
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
        if !NT_SUCCESS(Status) {
            DiskNumber += 1;
            continue;
        }

        KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);
        Irp = IoBuildDeviceIoControlRequest(
            0x70000,
            DeviceObject,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(DiskGeometry) as *mut core::ffi::c_void,
            core::mem::size_of::<DISK_GEOMETRY>() as ULONG,
            0u8,
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
            DiskNumber += 1;
            continue;
        }

        Status = IoCallDriver(DeviceObject, Irp);
        if Status == 0x103u32 as NTSTATUS {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
                0,
                0,
                0u8,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }
        if !NT_SUCCESS(Status) {
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
            DiskNumber += 1;
            continue;
        }

        Status = IoReadPartitionTableEx(DeviceObject, core::ptr::addr_of_mut!(DriveLayout));

        ObDereferenceObject(FileObject as *mut core::ffi::c_void);

        if !NT_SUCCESS(Status) {
            DiskNumber += 1;
            continue;
        }

        if DiskGeometry.BytesPerSector < 512 {
            DiskGeometry.BytesPerSector = 512;
        }

        NextEntry = (*ArcDiskInformation).DiskSignatureListHead.Flink;
        while NextEntry != core::ptr::addr_of_mut!((*ArcDiskInformation).DiskSignatureListHead) {
            ArcDiskSignature = (NextEntry as *mut u8).offset(-(core::mem::offset_of!(_ARC_DISK_SIGNATURE, ListEntry) as isize)) as PARC_DISK_SIGNATURE;
            
            if ((SingleDisk != 0 && DiskCount == 1) ||
                (IopVerifyDiskSignature(DriveLayout, ArcDiskSignature, core::ptr::addr_of_mut!(Signature)) != 0)) &&
                ((*DriveLayout).PartitionStyle == 0) {
                
                sprintf(
                    ArcBuffer.as_mut_ptr(),
                    b"\\ArcName\\%s\0".as_ptr() as *const i8,
                    (*ArcDiskSignature).ArcName,
                );
                RtlInitAnsiString(core::ptr::addr_of_mut!(ArcNameStringA), ArcBuffer.as_ptr());

                PartitionNumber = 1;
                while PartitionNumber <= (*DriveLayout).PartitionCount {
                    sprintf(
                        Buffer.as_mut_ptr(),
                        b"\\Device\\Harddisk%lu\\Partition%lu\0".as_ptr() as *const i8,
                        DiskNumber,
                        PartitionNumber,
                    );
                    RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
                    Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(DeviceStringW), core::ptr::addr_of_mut!(DeviceStringA), 1u8);
                    if !NT_SUCCESS(Status) {
                        PartitionNumber += 1;
                        continue;
                    }

                    if Signature == 0 {
                        Signature = (*DriveLayout).__bindgen_anon_1.Mbr.Signature;
                    }

                    sprintf(
                        ArcBuffer.as_mut_ptr(),
                        b"%spartition(%lu)\0".as_ptr() as *const i8,
                        (*ArcDiskSignature).ArcName,
                        PartitionNumber,
                    );
                    RtlInitAnsiString(core::ptr::addr_of_mut!(ArcNameStringA), ArcBuffer.as_ptr());

                    if RtlEqualString(core::ptr::addr_of_mut!(ArcNameStringA), core::ptr::addr_of_mut!(ArcBootString), 1u8) != 0 {
                        (*BootDiskInformation).BootDeviceSignature = Signature;

                        Status = IoGetDeviceObjectPointer(
                            core::ptr::addr_of_mut!(DeviceStringW),
                            0x80,
                            core::ptr::addr_of_mut!(FileObject),
                            core::ptr::addr_of_mut!(DeviceObject),
                        );
                        if !NT_SUCCESS(Status) {
                            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                            PartitionNumber += 1;
                            continue;
                        }

                        KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);
                        Irp = IoBuildDeviceIoControlRequest(
                            0x7049C,
                            DeviceObject,
                            core::ptr::null_mut(),
                            0,
                            core::ptr::addr_of_mut!(PartitionInformation) as *mut core::ffi::c_void,
                            core::mem::size_of::<PARTITION_INFORMATION_EX>() as ULONG,
                            0u8,
                            core::ptr::addr_of_mut!(Event),
                            core::ptr::addr_of_mut!(IoStatusBlock),
                        );
                        if Irp.is_null() {
                            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                            PartitionNumber += 1;
                            continue;
                        }

                        Status = IoCallDriver(DeviceObject, Irp);
                        if Status == 0x103u32 as NTSTATUS {
                            KeWaitForSingleObject(
                                core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
                                0,
                                0,
                                0u8,
                                core::ptr::null_mut(),
                            );
                            Status = IoStatusBlock.__bindgen_anon_1.Status;
                        }
                        if !NT_SUCCESS(Status) {
                            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                            PartitionNumber += 1;
                            continue;
                        }

                        (*BootDiskInformation).BootPartitionOffset = PartitionInformation.StartingOffset.QuadPart;

                        if IsBootDiskInfoEx != 0 {
                            if (*DriveLayout).PartitionStyle == 1 {
                                (*(BootDiskInformation as PBOOTDISK_INFORMATION_EX)).BootDeviceGuid = (*DriveLayout).__bindgen_anon_1.Gpt.DiskId;
                                (*(BootDiskInformation as PBOOTDISK_INFORMATION_EX)).BootDeviceIsGpt = 1u8;
                            } else {
                                (*(BootDiskInformation as PBOOTDISK_INFORMATION_EX)).BootDeviceIsGpt = 0u8;
                            }
                        }

                        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                    }

                    if RtlEqualString(core::ptr::addr_of_mut!(ArcNameStringA), core::ptr::addr_of_mut!(ArcSystemString), 1u8) != 0 {
                        (*BootDiskInformation).SystemDeviceSignature = Signature;

                        Status = IoGetDeviceObjectPointer(
                            core::ptr::addr_of_mut!(DeviceStringW),
                            0x80,
                            core::ptr::addr_of_mut!(FileObject),
                            core::ptr::addr_of_mut!(DeviceObject),
                        );
                        if !NT_SUCCESS(Status) {
                            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                            PartitionNumber += 1;
                            continue;
                        }

                        KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);
                        Irp = IoBuildDeviceIoControlRequest(
                            0x7049C,
                            DeviceObject,
                            core::ptr::null_mut(),
                            0,
                            core::ptr::addr_of_mut!(PartitionInformation) as *mut core::ffi::c_void,
                            core::mem::size_of::<PARTITION_INFORMATION_EX>() as ULONG,
                            0u8,
                            core::ptr::addr_of_mut!(Event),
                            core::ptr::addr_of_mut!(IoStatusBlock),
                        );
                        if Irp.is_null() {
                            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                            PartitionNumber += 1;
                            continue;
                        }

                        Status = IoCallDriver(DeviceObject, Irp);
                        if Status == 0x103u32 as NTSTATUS {
                            KeWaitForSingleObject(
                                core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
                                0,
                                0,
                                0u8,
                                core::ptr::null_mut(),
                            );
                            Status = IoStatusBlock.__bindgen_anon_1.Status;
                        }
                        if !NT_SUCCESS(Status) {
                            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                            PartitionNumber += 1;
                            continue;
                        }

                        (*BootDiskInformation).SystemPartitionOffset = PartitionInformation.StartingOffset.QuadPart;

                        if IsBootDiskInfoEx != 0 {
                            if (*DriveLayout).PartitionStyle == 1 {
                                (*(BootDiskInformation as PBOOTDISK_INFORMATION_EX)).SystemDeviceGuid = (*DriveLayout).__bindgen_anon_1.Gpt.DiskId;
                                (*(BootDiskInformation as PBOOTDISK_INFORMATION_EX)).SystemDeviceIsGpt = 1u8;
                            } else {
                                (*(BootDiskInformation as PBOOTDISK_INFORMATION_EX)).SystemDeviceIsGpt = 0u8;
                            }
                        }

                        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                    }

                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                    PartitionNumber += 1;
                }
            }
            
            NextEntry = (*NextEntry).Flink;
        }

        ExFreePool(DriveLayout as *mut core::ffi::c_void);
        DiskNumber += 1;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoReadDiskSignature(
    DeviceObject: PDEVICE_OBJECT,
    mut BytesPerSector: ULONG,
    Signature: PDISK_SIGNATURE
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Buffer: PUCHAR;
    let mut HeaderCRC32: ULONG;
    let mut i: ULONG;
    let mut CheckSum: ULONG;
    let EFIHeader: PEFI_PARTITION_HEADER;
    let PartitionDescriptor: PPARTITION_DESCRIPTOR;

    if BytesPerSector < 512 {
        BytesPerSector = 512;
    }

    Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, BytesPerSector as SIZE_T, TAG_FSTUB) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = FstubReadSector(
        DeviceObject,
        BytesPerSector,
        0u64,
        Buffer as PVOID
    );
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(Buffer as PVOID, TAG_FSTUB);
        return Status;
    }

    PartitionDescriptor = Buffer.add(PARTITION_TABLE_OFFSET) as PPARTITION_DESCRIPTOR;
    if (*PartitionDescriptor.add(0)).PartitionType == EFI_PMBR_OSTYPE_EFI &&
        (*PartitionDescriptor.add(1)).PartitionType == 0 &&
        (*PartitionDescriptor.add(2)).PartitionType == 0 &&
        (*PartitionDescriptor.add(3)).PartitionType == 0 {
        
        Status = FstubReadSector(
            DeviceObject,
            BytesPerSector,
            1u64,
            Buffer as PVOID
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(Buffer as PVOID, TAG_FSTUB);
            return Status;
        }
        
        EFIHeader = Buffer as PEFI_PARTITION_HEADER;

        if (*EFIHeader).Signature != EFI_HEADER_SIGNATURE ||
            (*EFIHeader).Revision != EFI_HEADER_REVISION_1 ||
            (*EFIHeader).HeaderSize != core::mem::size_of::<EFI_PARTITION_HEADER>() as ULONG {
            Status = STATUS_DISK_CORRUPT_ERROR;
            ExFreePoolWithTag(Buffer as PVOID, TAG_FSTUB);
            return Status;
        }

        HeaderCRC32 = (*EFIHeader).HeaderCRC32;
        (*EFIHeader).HeaderCRC32 = 0;
        if RtlComputeCrc32(0, Buffer as *const u8, core::mem::size_of::<EFI_PARTITION_HEADER>() as ULONG) != HeaderCRC32 {
            Status = STATUS_DISK_CORRUPT_ERROR;
            ExFreePoolWithTag(Buffer as PVOID, TAG_FSTUB);
            return Status;
        }

        (*Signature).PartitionStyle = PARTITION_STYLE_GPT;
        (*Signature).__bindgen_anon_1.Gpt.DiskId = (*EFIHeader).DiskGUID;
    } else {
        i = 0;
        CheckSum = 0;
        while i < 512 {
            CheckSum = CheckSum.wrapping_add(*(Buffer.add(i as usize) as PUINT32));
            i += core::mem::size_of::<UINT32>() as ULONG;
        }
        CheckSum = (!CheckSum).wrapping_add(1);

        (*Signature).PartitionStyle = PARTITION_STYLE_MBR;
        (*Signature).__bindgen_anon_1.Mbr.Signature = *(Buffer.add(DISK_SIGNATURE_OFFSET) as PUINT32);
        (*Signature).__bindgen_anon_1.Mbr.CheckSum = CheckSum;
    }

    ExFreePoolWithTag(Buffer as PVOID, TAG_FSTUB);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoReadPartitionTableEx(
    DeviceObject: PDEVICE_OBJECT,
    DriveLayout: *mut PDRIVE_LAYOUT_INFORMATION_EX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disk: PDISK_INFORMATION = core::ptr::null_mut();
    let mut PartitionStyle: PARTITION_STYLE = 0;

    Status = FstubAllocateDiskInformation(
        DeviceObject,
        core::ptr::addr_of_mut!(Disk),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = FstubDetectPartitionStyle(Disk, core::ptr::addr_of_mut!(PartitionStyle));
    if !NT_SUCCESS(Status) {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    match PartitionStyle {
        0 | 2 => {
            Status = FstubReadPartitionTableMBR(Disk, 0u8, DriveLayout);
        }
        1 => {
            Status = FstubReadPartitionTableEFI(Disk, 0u8, DriveLayout);
            if !NT_SUCCESS(Status) {
                Status = FstubReadPartitionTableEFI(Disk, 1u8, DriveLayout);
            }
        }
        _ => {
            // DPRINT removed
            Status = STATUS_UNSUCCESSFUL;
        }
    }

    FstubFreeDiskInformation(Disk);

    if NT_SUCCESS(Status) {
        FstubDbgPrintDriveLayoutEx(*DriveLayout);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoSetPartitionInformationEx(
    DeviceObject: PDEVICE_OBJECT,
    PartitionNumber: ULONG,
    PartitionInfo: PSET_PARTITION_INFORMATION_EX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disk: PDISK_INFORMATION = core::ptr::null_mut();
    let mut PartitionStyle: PARTITION_STYLE = 0;

    FstubDbgPrintSetPartitionEx(PartitionInfo, PartitionNumber);

    Status = FstubAllocateDiskInformation(DeviceObject, core::ptr::addr_of_mut!(Disk), core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = FstubDetectPartitionStyle(Disk, core::ptr::addr_of_mut!(PartitionStyle));
    if !NT_SUCCESS(Status) {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    if (*PartitionInfo).PartitionStyle != PartitionStyle {
        FstubFreeDiskInformation(Disk);
        return STATUS_INVALID_PARAMETER;
    }

    Status = match PartitionStyle {
        PARTITION_STYLE_MBR => {
            IoSetPartitionInformation(
                DeviceObject,
                (*Disk).SectorSize,
                PartitionNumber,
                (*PartitionInfo).__bindgen_anon_1.Mbr.PartitionType as ULONG,
            )
        }
        PARTITION_STYLE_GPT => {
            FstubSetPartitionInformationEFI(
                Disk,
                PartitionNumber,
                core::ptr::addr_of_mut!((*PartitionInfo).__bindgen_anon_1.Gpt),
            )
        }
        _ => STATUS_NOT_SUPPORTED,
    };

    FstubFreeDiskInformation(Disk);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoVerifyPartitionTable(
    DeviceObject: PDEVICE_OBJECT,
    FixErrors: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Disk: PDISK_INFORMATION = core::ptr::null_mut();
    let mut PartitionStyle: PARTITION_STYLE = 0;

    Status = FstubAllocateDiskInformation(DeviceObject, &mut Disk, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = FstubDetectPartitionStyle(Disk, &mut PartitionStyle);
    if !NT_SUCCESS(Status) {
        FstubFreeDiskInformation(Disk);
        return Status;
    }

    Status = match PartitionStyle {
        PARTITION_STYLE_MBR => STATUS_SUCCESS,
        PARTITION_STYLE_GPT => FstubVerifyPartitionTableEFI(Disk, FixErrors),
        _ => STATUS_NOT_SUPPORTED,
    };

    FstubFreeDiskInformation(Disk);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoWritePartitionTableEx(
    DeviceObject: PDEVICE_OBJECT,
    DriveLayout: PDRIVE_LAYOUT_INFORMATION_EX,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DiskGuid: GUID = core::mem::zeroed();
    let mut NumberOfEntries: ULONG;
    let mut Disk: PDISK_INFORMATION = core::ptr::null_mut();
    let mut EfiHeader: PEFI_PARTITION_HEADER = core::ptr::null_mut();
    let mut SectorsForPartitions: ULONGLONG;
    let mut FirstUsableLBA: ULONGLONG;
    let mut LastUsableLBA: ULONGLONG;

    FstubDbgPrintDriveLayoutEx(DriveLayout);

    Status = FstubAllocateDiskInformation(DeviceObject, &mut Disk, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        return Status;
    }

    match (*DriveLayout).PartitionStyle {
        0 => {
            Status = FstubWritePartitionTableMBR(Disk, DriveLayout);
        }

        1 => {
            Status = FstubReadHeaderEFI(Disk, 0u8, &mut EfiHeader);
            if !NT_SUCCESS(Status) {
                Status = FstubReadHeaderEFI(Disk, 1u8, &mut EfiHeader);
            }

            if NT_SUCCESS(Status) {
                if (*DriveLayout).PartitionCount <= (*EfiHeader).NumberOfEntries {
                    NumberOfEntries = (*EfiHeader).NumberOfEntries;
                    RtlCopyMemory(
                        core::ptr::addr_of_mut!(DiskGuid) as PVOID,
                        core::ptr::addr_of!((*EfiHeader).DiskGUID) as PVOID,
                        core::mem::size_of::<GUID>() as SIZE_T,
                    );
                    SectorsForPartitions = (NumberOfEntries as ULONGLONG * PARTITION_ENTRY_SIZE as ULONGLONG) / (*Disk).SectorSize as ULONGLONG;
                    FirstUsableLBA = SectorsForPartitions + 2;
                    LastUsableLBA = (*Disk).SectorCount - SectorsForPartitions - 1;
                    Status = FstubWritePartitionTableEFI(
                        Disk,
                        DiskGuid,
                        NumberOfEntries,
                        FirstUsableLBA,
                        LastUsableLBA,
                        0u8,
                        (*DriveLayout).PartitionCount,
                        (*DriveLayout).PartitionEntry.as_mut_ptr(),
                    );
                    if NT_SUCCESS(Status) {
                        Status = FstubWritePartitionTableEFI(
                            Disk,
                            DiskGuid,
                            NumberOfEntries,
                            FirstUsableLBA,
                            LastUsableLBA,
                            1u8,
                            (*DriveLayout).PartitionCount,
                            (*DriveLayout).PartitionEntry.as_mut_ptr(),
                        );
                    }
                } else {
                    Status = STATUS_INVALID_PARAMETER;
                }
            }
        }

        _ => {
            // DPRINT removed
            Status = STATUS_NOT_SUPPORTED;
        }
    }

    FstubFreeDiskInformation(Disk);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/fstub/fstubex.c
// * PURPOSE:         Extended FSTUB Routines (not linked to HAL)
// * PROGRAMMERS:     Pierre Schweitzer (pierre.schweitzer@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// typedef struct _DISK_INFORMATION
// {
//     PDEVICE_OBJECT DeviceObject;
//     ULONG SectorSize;
//     DISK_GEOMETRY_EX DiskGeometry;
//     PUCHAR Buffer;
//     ULONGLONG SectorCount;
// } DISK_INFORMATION, *PDISK_INFORMATION;
//
// #include <pshpack1.h>
// typedef struct _EFI_PARTITION_HEADER
// {
//     ULONGLONG Signature;         // 0
//     ULONG Revision;              // 8
//     ULONG HeaderSize;            // 12
//     ULONG HeaderCRC32;           // 16
//     ULONG Reserved;              // 20
//     ULONGLONG MyLBA;             // 24
//     ULONGLONG AlternateLBA;      // 32
//     ULONGLONG FirstUsableLBA;    // 40
//     ULONGLONG LastUsableLBA;     // 48
//     GUID DiskGUID;               // 56
//     ULONGLONG PartitionEntryLBA; // 72
//     ULONG NumberOfEntries;       // 80
//     ULONG SizeOfPartitionEntry;  // 84
//     ULONG PartitionEntryCRC32;   // 88
// } EFI_PARTITION_HEADER, *PEFI_PARTITION_HEADER;
// C_ASSERT(sizeof(EFI_PARTITION_HEADER) == 92);
// #include <poppack.h>
//
// typedef struct _EFI_PARTITION_ENTRY
// {
//     GUID PartitionType;    // 0
//     GUID UniquePartition;  // 16
//     ULONGLONG StartingLBA; // 32
//     ULONGLONG EndingLBA;   // 40
//     ULONGLONG Attributes;  // 48
//     WCHAR Name[0x24];      // 56
// } EFI_PARTITION_ENTRY, *PEFI_PARTITION_ENTRY;
// C_ASSERT(sizeof(EFI_PARTITION_ENTRY) == 128);
//
// typedef struct _PARTITION_TABLE_ENTRY
// {
//     UCHAR BootIndicator;
//     UCHAR StartHead;
//     UCHAR StartSector;
//     UCHAR StartCylinder;
//     UCHAR SystemIndicator;
//     UCHAR EndHead;
//     UCHAR EndSector;
//     UCHAR EndCylinder;
//     ULONG SectorCountBeforePartition;
//     ULONG PartitionSectorCount;
// } PARTITION_TABLE_ENTRY, *PPARTITION_TABLE_ENTRY;
// C_ASSERT(sizeof(PARTITION_TABLE_ENTRY) == 16);
//
// #include <pshpack1.h>
// typedef struct _MASTER_BOOT_RECORD
// {
//     UCHAR MasterBootRecordCodeAndData[0x1B8]; // 0
//     ULONG Signature;                          // 440
//     USHORT Reserved;                          // 444
//     PARTITION_TABLE_ENTRY PartitionTable[4];  // 446
//     USHORT MasterBootRecordMagic;             // 510
// } MASTER_BOOT_RECORD, *PMASTER_BOOT_RECORD;
// C_ASSERT(sizeof(MASTER_BOOT_RECORD) == 512);
// #include <poppack.h>
//
// /* Partition entry size (bytes) - FIXME: It's hardcoded as Microsoft does, but according to specs, it shouldn't be */
// #define PARTITION_ENTRY_SIZE 128
// /* Defines "EFI PART" */
// #define EFI_HEADER_SIGNATURE  0x5452415020494645ULL
// /* Defines version 1.0 */
// #define EFI_HEADER_REVISION_1 0x00010000
// /* Defines system type for MBR showing that a GPT is following */
// #define EFI_PMBR_OSTYPE_EFI 0xEE
// /* Defines size to store a complete GUID + null char */
// #define EFI_GUID_STRING_SIZE 0x27
//
// #define IS_VALID_DISK_INFO(Disk) \
//   (Disk)               &&        \
//   (Disk->DeviceObject) &&        \
//   (Disk->SectorSize)   &&        \
//   (Disk->Buffer)       &&        \
//   (Disk->SectorCount)
//
// VOID
// NTAPI
// FstubDbgPrintPartitionEx(IN PPARTITION_INFORMATION_EX PartitionEntry,
//                          IN ULONG PartitionNumber
// );
//
// NTSTATUS
// NTAPI
// FstubDetectPartitionStyle(IN PDISK_INFORMATION Disk,
//                           IN PARTITION_STYLE * PartitionStyle
// );
//
// VOID
// NTAPI
// FstubFreeDiskInformation(IN PDISK_INFORMATION DiskBuffer
// );
//
// NTSTATUS
// NTAPI
// FstubGetDiskGeometry(IN PDEVICE_OBJECT DeviceObject,
//                      OUT PDISK_GEOMETRY_EX Geometry
// );
//
// NTSTATUS
// NTAPI
// FstubReadSector(IN PDEVICE_OBJECT DeviceObject,
//                 IN ULONG SectorSize,
//                 IN ULONGLONG StartingSector OPTIONAL,
//                 OUT PVOID Buffer
// );
//
// NTSTATUS
// NTAPI
// FstubWriteBootSectorEFI(IN PDISK_INFORMATION Disk
// );
//
// NTSTATUS
// NTAPI
// FstubWriteHeaderEFI(IN PDISK_INFORMATION Disk,
//                     IN ULONG PartitionsSizeSector,
//                     IN GUID DiskGUID,
//                     IN ULONG NumberOfEntries,
//                     IN ULONGLONG FirstUsableLBA,
//                     IN ULONGLONG LastUsableLBA,
//                     IN ULONG PartitionEntryCRC32,
//                     IN BOOLEAN WriteBackupTable);
//
// NTSTATUS
// NTAPI
// FstubWritePartitionTableEFI(IN PDISK_INFORMATION Disk,
//                             IN GUID DiskGUID,
//                             IN ULONG MaxPartitionCount,
//                             IN ULONGLONG FirstUsableLBA,
//                             IN ULONGLONG LastUsableLBA,
//                             IN BOOLEAN WriteBackupTable,
//                             IN ULONG PartitionCount,
//                             IN PPARTITION_INFORMATION_EX PartitionEntries OPTIONAL
// );
//
// NTSTATUS
// NTAPI
// FstubWriteSector(IN PDEVICE_OBJECT DeviceObject,
//                  IN ULONG SectorSize,
//                  IN ULONGLONG StartingSector OPTIONAL,
//                  IN PVOID Buffer
// );
//
// VOID
// NTAPI
// FstubAdjustPartitionCount(IN ULONG SectorSize,
//                           IN OUT PULONG PartitionCount)
// {
//     ULONG Count;
//
//     PAGED_CODE();
//
//     ASSERT(SectorSize);
//     ASSERT(PartitionCount);
//
//     /* Get partition count */
//     Count = *PartitionCount;
//     /* We need at least 128 entries */
//     if (Count < 128)
//     {
//         Count = 128;
//     }
//
//     /* Then, ensure that we will have a round value,
//      * ie, all sectors will be full of entries
//      * There won't be lonely entries
//      */
//     Count = (Count * PARTITION_ENTRY_SIZE) / SectorSize;
//     Count = (Count * SectorSize) / PARTITION_ENTRY_SIZE;
//     ASSERT(*PartitionCount <= Count);
//     /* Return result */
//     *PartitionCount = Count;
//
//     /* One more sanity check */
//     if (SectorSize == 512)
//     {
//         ASSERT(Count % 4 == 0);
//     }
// }
//
// NTSTATUS
// NTAPI
// FstubAllocateDiskInformation(IN PDEVICE_OBJECT DeviceObject,
//                              OUT PDISK_INFORMATION * DiskBuffer,
//                              IN PDISK_GEOMETRY_EX DiskGeometry OPTIONAL)
// {
//     NTSTATUS Status;
//     PDISK_INFORMATION DiskInformation;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(DiskBuffer);
//
//     /* Allocate internal structure */
//     DiskInformation = ExAllocatePoolWithTag(NonPagedPool, sizeof(DISK_INFORMATION), TAG_FSTUB);
//     if (!DiskInformation)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* If caller don't pass needed information, let's get them */
//     if (!DiskGeometry)
//     {
//         Status = FstubGetDiskGeometry(DeviceObject, &(DiskInformation->DiskGeometry));
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePoolWithTag(DiskInformation, TAG_FSTUB);
//             return Status;
//         }
//     }
//     else
//     {
//         RtlCopyMemory(&DiskInformation->DiskGeometry, DiskGeometry, sizeof(DISK_GEOMETRY_EX));
//     }
//
//     /* Ensure read/received information are correct */
//     if (DiskInformation->DiskGeometry.Geometry.BytesPerSector == 0 ||
//         DiskInformation->DiskGeometry.DiskSize.QuadPart == 0)
//     {
//         ExFreePoolWithTag(DiskInformation, TAG_FSTUB);
//         return STATUS_DEVICE_NOT_READY;
//     }
//
//     /* Store vital information as well */
//     DiskInformation->DeviceObject = DeviceObject;
//     DiskInformation->SectorSize = DiskInformation->DiskGeometry.Geometry.BytesPerSector;
//     DiskInformation->SectorCount = DiskInformation->DiskGeometry.DiskSize.QuadPart / DiskInformation->SectorSize;
//
//     /* Finally, allocate the buffer that will be used for different read */
//     DiskInformation->Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, DiskInformation->SectorSize, TAG_FSTUB);
//     if (!DiskInformation->Buffer)
//     {
//         ExFreePoolWithTag(DiskInformation, TAG_FSTUB);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Return allocated internal structure */
//     *DiskBuffer = DiskInformation;
//
//     return STATUS_SUCCESS;
// }
//
// PDRIVE_LAYOUT_INFORMATION
// NTAPI
// FstubConvertExtendedToLayout(IN PDRIVE_LAYOUT_INFORMATION_EX LayoutEx)
// {
//     ULONG i;
//     PDRIVE_LAYOUT_INFORMATION DriveLayout;
//
//     PAGED_CODE();
//
//     ASSERT(LayoutEx);
//
//     /* Check whether we're dealing with MBR partition table */
//     if (LayoutEx->PartitionStyle != PARTITION_STYLE_MBR)
//     {
//         ASSERT(FALSE);
//         return NULL;
//     }
//
//     /* Allocate needed buffer */
//     DriveLayout = ExAllocatePoolWithTag(NonPagedPool,
//                                         FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION, PartitionEntry) +
//                                         LayoutEx->PartitionCount * sizeof(PARTITION_INFORMATION),
//                                         TAG_FSTUB);
//     if (!DriveLayout)
//     {
//         return NULL;
//     }
//
//     /* Convert information about partition table */
//     DriveLayout->PartitionCount = LayoutEx->PartitionCount;
//     DriveLayout->Signature = LayoutEx->Mbr.Signature;
//
//     /* Convert each partition */
//     for (i = 0; i < LayoutEx->PartitionCount; i++)
//     {
//         DriveLayout->PartitionEntry[i].StartingOffset = LayoutEx->PartitionEntry[i].StartingOffset;
//         DriveLayout->PartitionEntry[i].PartitionLength = LayoutEx->PartitionEntry[i].PartitionLength;
//         DriveLayout->PartitionEntry[i].HiddenSectors = LayoutEx->PartitionEntry[i].Mbr.HiddenSectors;
//         DriveLayout->PartitionEntry[i].PartitionNumber = LayoutEx->PartitionEntry[i].PartitionNumber;
//         DriveLayout->PartitionEntry[i].PartitionType = LayoutEx->PartitionEntry[i].Mbr.PartitionType;
//         DriveLayout->PartitionEntry[i].BootIndicator = LayoutEx->PartitionEntry[i].Mbr.BootIndicator;
//         DriveLayout->PartitionEntry[i].RecognizedPartition = LayoutEx->PartitionEntry[i].Mbr.RecognizedPartition;
//         DriveLayout->PartitionEntry[i].RewritePartition = LayoutEx->PartitionEntry[i].RewritePartition;
//     }
//
//     return DriveLayout;
// }
//
// VOID
// NTAPI
// FstubCopyEntryEFI(OUT PEFI_PARTITION_ENTRY Entry,
//                   IN PPARTITION_INFORMATION_EX Partition,
//                   ULONG SectorSize)
// {
//     PAGED_CODE();
//
//     ASSERT(Entry);
//     ASSERT(Partition);
//     ASSERT(SectorSize);
//
//     /* Just convert data to EFI partition entry type */
//     Entry->PartitionType = Partition->Gpt.PartitionType;
//     Entry->UniquePartition = Partition->Gpt.PartitionId;
//     Entry->StartingLBA = Partition->StartingOffset.QuadPart / SectorSize;
//     Entry->EndingLBA = (Partition->StartingOffset.QuadPart + Partition->PartitionLength.QuadPart - 1) / SectorSize;
//     Entry->Attributes = Partition->Gpt.Attributes;
//     RtlCopyMemory(Entry->Name, Partition->Gpt.Name, sizeof(Entry->Name));
// }
//
// NTSTATUS
// NTAPI
// FstubCreateDiskMBR(IN PDEVICE_OBJECT DeviceObject,
//                    IN PCREATE_DISK_MBR DiskInfo)
// {
//     NTSTATUS Status;
//     PDISK_INFORMATION Disk = NULL;
//     PMASTER_BOOT_RECORD MasterBootRecord;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//
//     /* Allocate internal structure */
//     Status = FstubAllocateDiskInformation(DeviceObject, &Disk, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Read previous MBR, if any */
//     Status = FstubReadSector(Disk->DeviceObject,
//                              Disk->SectorSize,
//                              0ULL,
//                              Disk->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//     /* Fill the buffer with needed information, we won't overwrite boot code */
//     MasterBootRecord = (PMASTER_BOOT_RECORD)Disk->Buffer;
//     MasterBootRecord->Signature = DiskInfo->Signature;
//     RtlZeroMemory(MasterBootRecord->PartitionTable, sizeof(PARTITION_TABLE_ENTRY) * NUM_PARTITION_TABLE_ENTRIES);
//     MasterBootRecord->MasterBootRecordMagic = BOOT_RECORD_SIGNATURE;
//
//     /* Finally, write MBR */
//     Status = FstubWriteSector(Disk->DeviceObject,
//                               Disk->SectorSize,
//                               0ULL,
//                               Disk->Buffer);
//
//     /* Release internal structure and return */
//     FstubFreeDiskInformation(Disk);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubCreateDiskEFI(IN PDEVICE_OBJECT DeviceObject,
//                    IN PCREATE_DISK_GPT DiskInfo)
// {
//     NTSTATUS Status;
//     PDISK_INFORMATION Disk = NULL;
//     ULONGLONG FirstUsableLBA, LastUsableLBA;
//     ULONG MaxPartitionCount, SectorsForPartitions;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(DiskInfo);
//
//     /* Allocate internal structure */
//     Status = FstubAllocateDiskInformation(DeviceObject, &Disk, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//     ASSERT(Disk);
//
//     /* Write legacy MBR */
//     Status = FstubWriteBootSectorEFI(Disk);
//     if (!NT_SUCCESS(Status))
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//
//     /* Get max entries and adjust its number */
//     MaxPartitionCount = DiskInfo->MaxPartitionCount;
//     FstubAdjustPartitionCount(Disk->SectorSize, &MaxPartitionCount);
//
//     /* Count number of sectors needed to store partitions */
//     SectorsForPartitions = (MaxPartitionCount * PARTITION_ENTRY_SIZE) / Disk->SectorSize;
//     /* Set first usable LBA: Legacy MBR + GPT header + Partitions entries */
//     FirstUsableLBA = SectorsForPartitions + 2;
//     /* Set last usable LBA: Last sector - GPT header - Partitions entries */
//     LastUsableLBA = Disk->SectorCount - SectorsForPartitions - 1;
//
//     /* First, write primary table */
//     Status = FstubWritePartitionTableEFI(Disk,
//                                          DiskInfo->DiskId,
//                                          MaxPartitionCount,
//                                          FirstUsableLBA,
//                                          LastUsableLBA,
//                                          FALSE,
//                                          0,
//                                          NULL);
//     /* Then, write backup table */
//     if (NT_SUCCESS(Status))
//     {
//         Status = FstubWritePartitionTableEFI(Disk,
//                                              DiskInfo->DiskId,
//                                              MaxPartitionCount,
//                                              FirstUsableLBA,
//                                              LastUsableLBA,
//                                              TRUE,
//                                              0,
//                                              NULL);
//     }
//
//     /* Release internal structure and return */
//     FstubFreeDiskInformation(Disk);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubCreateDiskRaw(IN PDEVICE_OBJECT DeviceObject)
// {
//     NTSTATUS Status;
//     PDISK_INFORMATION Disk = NULL;
//     PARTITION_STYLE PartitionStyle;
//     PMASTER_BOOT_RECORD MasterBootRecord;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//
//     /* Allocate internal structure */
//     Status = FstubAllocateDiskInformation(DeviceObject, &Disk, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Detect current disk partition style */
//     Status = FstubDetectPartitionStyle(Disk, &PartitionStyle);
//     if (!NT_SUCCESS(Status))
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//
//     /* Read MBR, if any */
//     Status = FstubReadSector(Disk->DeviceObject,
//                              Disk->SectorSize,
//                              0ULL,
//                              Disk->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//
//     /* Only zero useful stuff */
//     MasterBootRecord = (PMASTER_BOOT_RECORD)Disk->Buffer;
//     MasterBootRecord->Signature = 0;
//     RtlZeroMemory(MasterBootRecord->PartitionTable, sizeof(PARTITION_TABLE_ENTRY));
//     MasterBootRecord->MasterBootRecordMagic = 0;
//
//     /* Write back that destroyed MBR */
//     Status = FstubWriteSector(Disk->DeviceObject,
//                               Disk->SectorSize,
//                               0ULL,
//                               Disk->Buffer);
//     /* If previous style wasn't GPT, we're done here */
//     if (PartitionStyle != PARTITION_STYLE_GPT)
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//
//     /* Otherwise, we've to zero the two GPT headers */
//     RtlZeroMemory(Disk->Buffer, Disk->SectorSize);
//     /* Erase primary header */
//     Status = FstubWriteSector(Disk->DeviceObject,
//                               Disk->SectorSize,
//                               1ULL,
//                               Disk->Buffer);
//     /* In case of success, erase backup header */
//     if (NT_SUCCESS(Status))
//     {
//         Status = FstubWriteSector(Disk->DeviceObject,
//                                   Disk->SectorSize,
//                                   Disk->SectorCount - 1ULL,
//                                   Disk->Buffer);
//     }
//
//     /* Release internal structure and return */
//     FstubFreeDiskInformation(Disk);
//     return Status;
// }
//
// PCHAR
// NTAPI
// FstubDbgGuidToString(IN PGUID Guid,
//                      OUT PCHAR String)
// {
//     sprintf(String,
//             "{%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x}",
//             Guid->Data1,
//             Guid->Data2,
//             Guid->Data3,
//             Guid->Data4[0],
//             Guid->Data4[1],
//             Guid->Data4[2],
//             Guid->Data4[3],
//             Guid->Data4[4],
//             Guid->Data4[5],
//             Guid->Data4[6],
//             Guid->Data4[7]);
//
//     return String;
// }
//
// VOID
// NTAPI
// FstubDbgPrintDriveLayoutEx(IN PDRIVE_LAYOUT_INFORMATION_EX DriveLayout)
// {
//     ULONG i;
//     CHAR Guid[EFI_GUID_STRING_SIZE];
//
//     PAGED_CODE();
//
//     DPRINT("FSTUB: DRIVE_LAYOUT_INFORMATION_EX: %p\n", DriveLayout);
//     switch (DriveLayout->PartitionStyle)
//     {
//         case PARTITION_STYLE_MBR:
//             if (DriveLayout->PartitionCount % 4 != 0)
//             {
//                 DPRINT("Warning: Partition count isn't a 4-factor: %lu!\n", DriveLayout->PartitionCount);
//             }
//
//             DPRINT("Signature: %8.8x\n", DriveLayout->Mbr.Signature);
//             for (i = 0; i < DriveLayout->PartitionCount; i++)
//             {
//                 FstubDbgPrintPartitionEx(DriveLayout->PartitionEntry, i);
//             }
//
//             break;
//         case PARTITION_STYLE_GPT:
//             FstubDbgGuidToString(&(DriveLayout->Gpt.DiskId), Guid);
//             DPRINT("DiskId: %s\n", Guid);
//             DPRINT("StartingUsableOffset: %I64x\n", DriveLayout->Gpt.StartingUsableOffset.QuadPart);
//             DPRINT("UsableLength: %I64x\n", DriveLayout->Gpt.UsableLength.QuadPart);
//             DPRINT("MaxPartitionCount: %lu\n", DriveLayout->Gpt.MaxPartitionCount);
//             for (i = 0; i < DriveLayout->PartitionCount; i++)
//             {
//                 FstubDbgPrintPartitionEx(DriveLayout->PartitionEntry, i);
//             }
//
//             break;
//         default:
//             DPRINT("Unsupported partition style: %lu\n", DriveLayout->PartitionStyle);
//     }
// }
//
// VOID
// NTAPI
// FstubDbgPrintPartitionEx(IN PPARTITION_INFORMATION_EX PartitionEntry,
//                          IN ULONG PartitionNumber)
// {
//     CHAR Guid[EFI_GUID_STRING_SIZE];
//
//     PAGED_CODE();
//
//     DPRINT("Printing partition %lu\n", PartitionNumber);
//
//     switch (PartitionEntry[PartitionNumber].PartitionStyle)
//     {
//         case PARTITION_STYLE_MBR:
//             DPRINT("  StartingOffset: %I64x\n", PartitionEntry[PartitionNumber].StartingOffset.QuadPart);
//             DPRINT("  PartitionLength: %I64x\n", PartitionEntry[PartitionNumber].PartitionLength.QuadPart);
//             DPRINT("  RewritePartition: %u\n", PartitionEntry[PartitionNumber].RewritePartition);
//             DPRINT("  PartitionType: %02x\n", PartitionEntry[PartitionNumber].Mbr.PartitionType);
//             DPRINT("  BootIndicator: %u\n", PartitionEntry[PartitionNumber].Mbr.BootIndicator);
//             DPRINT("  RecognizedPartition: %u\n", PartitionEntry[PartitionNumber].Mbr.RecognizedPartition);
//             DPRINT("  HiddenSectors: %lu\n", PartitionEntry[PartitionNumber].Mbr.HiddenSectors);
//
//             break;
//         case PARTITION_STYLE_GPT:
//             DPRINT("  StartingOffset: %I64x\n", PartitionEntry[PartitionNumber].StartingOffset.QuadPart);
//             DPRINT("  PartitionLength: %I64x\n", PartitionEntry[PartitionNumber].PartitionLength.QuadPart);
//             DPRINT("  RewritePartition: %u\n", PartitionEntry[PartitionNumber].RewritePartition);
//             FstubDbgGuidToString(&(PartitionEntry[PartitionNumber].Gpt.PartitionType), Guid);
//             DPRINT("  PartitionType: %s\n", Guid);
//             FstubDbgGuidToString(&(PartitionEntry[PartitionNumber].Gpt.PartitionId), Guid);
//             DPRINT("  PartitionId: %s\n", Guid);
//             DPRINT("  Attributes: %I64x\n", PartitionEntry[PartitionNumber].Gpt.Attributes);
//             DPRINT("  Name: %ws\n", PartitionEntry[PartitionNumber].Gpt.Name);
//
//             break;
//         default:
//             DPRINT("  Unsupported partition style: %ld\n", PartitionEntry[PartitionNumber].PartitionStyle);
//     }
// }
//
// VOID
// NTAPI
// FstubDbgPrintSetPartitionEx(IN PSET_PARTITION_INFORMATION_EX PartitionEntry,
//                             IN ULONG PartitionNumber)
// {
//     CHAR Guid[EFI_GUID_STRING_SIZE];
//
//     PAGED_CODE();
//
//     DPRINT("FSTUB: SET_PARTITION_INFORMATION_EX: %p\n", PartitionEntry);
//     DPRINT("Modifying partition %lu\n", PartitionNumber);
//     switch (PartitionEntry->PartitionStyle)
//     {
//         case PARTITION_STYLE_MBR:
//             DPRINT("  PartitionType: %02x\n", PartitionEntry->Mbr.PartitionType);
//
//             break;
//         case PARTITION_STYLE_GPT:
//             FstubDbgGuidToString(&(PartitionEntry->Gpt.PartitionType), Guid);
//             DPRINT("  PartitionType: %s\n", Guid);
//             FstubDbgGuidToString(&(PartitionEntry->Gpt.PartitionId), Guid);
//             DPRINT("  PartitionId: %s\n", Guid);
//             DPRINT("  Attributes: %I64x\n", PartitionEntry->Gpt.Attributes);
//             DPRINT("  Name: %ws\n", PartitionEntry->Gpt.Name);
//
//             break;
//         default:
//             DPRINT("  Unsupported partition style: %ld\n", PartitionEntry[PartitionNumber].PartitionStyle);
//     }
// }
//
// NTSTATUS
// NTAPI
// FstubDetectPartitionStyle(IN PDISK_INFORMATION Disk,
//                           IN PARTITION_STYLE * PartitionStyle)
// {
//     NTSTATUS Status;
//     PPARTITION_DESCRIPTOR PartitionDescriptor;
//
//     PAGED_CODE();
//
//     ASSERT(IS_VALID_DISK_INFO(Disk));
//     ASSERT(PartitionStyle);
//
//     /* Read disk first sector */
//     Status = FstubReadSector(Disk->DeviceObject,
//                              Disk->SectorSize,
//                              0,
//                              Disk->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Get the partition descriptor array */
//     PartitionDescriptor = (PPARTITION_DESCRIPTOR)&Disk->Buffer[PARTITION_TABLE_OFFSET];
//     /* If we have not the 0xAA55 then it's raw partition */
//     if (*(PUINT16)&Disk->Buffer[BOOT_SIGNATURE_OFFSET] != BOOT_RECORD_SIGNATURE)
//     {
//         *PartitionStyle = PARTITION_STYLE_RAW;
//     }
//     /* Check partitions types: if first is 0xEE and all the others 0, we have GPT */
//     else if (PartitionDescriptor[0].PartitionType == EFI_PMBR_OSTYPE_EFI &&
//              PartitionDescriptor[1].PartitionType == 0 &&
//              PartitionDescriptor[2].PartitionType == 0 &&
//              PartitionDescriptor[3].PartitionType == 0)
//     {
//         *PartitionStyle = PARTITION_STYLE_GPT;
//     }
//     /* Otherwise, partition table is in MBR */
//     else
//     {
//         *PartitionStyle = PARTITION_STYLE_MBR;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// FstubFreeDiskInformation(IN PDISK_INFORMATION DiskBuffer)
// {
//     if (DiskBuffer)
//     {
//         if (DiskBuffer->Buffer)
//         {
//             ExFreePoolWithTag(DiskBuffer->Buffer, TAG_FSTUB);
//         }
//         ExFreePoolWithTag(DiskBuffer, TAG_FSTUB);
//     }
// }
//
// NTSTATUS
// NTAPI
// FstubGetDiskGeometry(IN PDEVICE_OBJECT DeviceObject,
//                      OUT PDISK_GEOMETRY_EX Geometry)
// {
//     NTSTATUS Status;
//     PIRP Irp;
//     PKEVENT Event = NULL;
//     PDISK_GEOMETRY_EX DiskGeometry = NULL;
//     PIO_STATUS_BLOCK IoStatusBlock = NULL;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(Geometry);
//
//     /* Allocate needed components */
//     DiskGeometry = ExAllocatePoolWithTag(NonPagedPool, sizeof(DISK_GEOMETRY_EX), TAG_FSTUB);
//     if (!DiskGeometry)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     IoStatusBlock = ExAllocatePoolWithTag(NonPagedPool, sizeof(IO_STATUS_BLOCK), TAG_FSTUB);
//     if (!IoStatusBlock)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     Event = ExAllocatePoolWithTag(NonPagedPool, sizeof(KEVENT), TAG_FSTUB);
//     if (!Event)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//     /* Initialize the waiting event */
//     KeInitializeEvent(Event, NotificationEvent, FALSE);
//
//     /* Build the request to get disk geometry */
//     Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
//                                         DeviceObject,
//                                         0,
//                                         0,
//                                         DiskGeometry,
//                                         sizeof(DISK_GEOMETRY_EX),
//                                         FALSE,
//                                         Event,
//                                         IoStatusBlock);
//     if (!Irp)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     /* Call the driver and wait for completion if needed */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock->Status;
//     }
//
//     /* In case of a success, return read data */
//     if (NT_SUCCESS(Status))
//     {
//         *Geometry = *DiskGeometry;
//     }
//
// Cleanup:
//     if (DiskGeometry)
//     {
//         ExFreePoolWithTag(DiskGeometry, TAG_FSTUB);
//
//         if (NT_SUCCESS(Status))
//         {
//             ASSERT(Geometry->Geometry.BytesPerSector % PARTITION_ENTRY_SIZE == 0);
//         }
//     }
//
//     if (IoStatusBlock)
//     {
//         ExFreePoolWithTag(IoStatusBlock, TAG_FSTUB);
//     }
//
//     if (Event)
//     {
//         ExFreePoolWithTag(Event, TAG_FSTUB);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubReadHeaderEFI(IN PDISK_INFORMATION Disk,
//                    IN BOOLEAN ReadBackupTable,
//                    PEFI_PARTITION_HEADER * HeaderBuffer)
// {
//     NTSTATUS Status;
//     PUCHAR Sector = NULL;
//     ULONGLONG StartingSector;
//     PEFI_PARTITION_HEADER EFIHeader;
//     ULONG i, HeaderCRC32, PreviousCRC32, SectoredPartitionEntriesSize, LonelyPartitions;
//
//     PAGED_CODE();
//
//     ASSERT(Disk);
//     ASSERT(IS_VALID_DISK_INFO(Disk));
//     ASSERT(HeaderBuffer);
//
//     /* In case we want to read backup table, we read last disk sector */
//     if (ReadBackupTable)
//     {
//         StartingSector = Disk->SectorCount - 1ULL;
//     }
//     else
//     {
//         /* Otherwise we start at first sector (as sector 0 is the MBR) */
//         StartingSector = 1ULL;
//     }
//
//     Status = FstubReadSector(Disk->DeviceObject,
//                              Disk->SectorSize,
//                              StartingSector,
//                              Disk->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("EFI::Failed reading header!\n");
//         return Status;
//     }
//     /* Let's use read buffer as EFI_PARTITION_HEADER */
//     EFIHeader = (PEFI_PARTITION_HEADER)Disk->Buffer;
//
//
//     /* First check signature
//      * Then, check version (we only support v1)
//      * Finally check header size
//      */
//     if (EFIHeader->Signature != EFI_HEADER_SIGNATURE ||
//         EFIHeader->Revision != EFI_HEADER_REVISION_1 ||
//         EFIHeader->HeaderSize != sizeof(EFI_PARTITION_HEADER))
//     {
//         DPRINT("EFI::Wrong signature/version/header size!\n");
//         DPRINT("%I64x (expected: %I64x)\n", EFIHeader->Signature, EFI_HEADER_SIGNATURE);
//         DPRINT("%03x (expected: %03x)\n", EFIHeader->Revision, EFI_HEADER_REVISION_1);
//         DPRINT("%02x (expected: %02x)\n", EFIHeader->HeaderSize, sizeof(EFI_PARTITION_HEADER));
//         return STATUS_DISK_CORRUPT_ERROR;
//     }
//
//     /* Save current checksum */
//     HeaderCRC32 = EFIHeader->HeaderCRC32;
//     /* Then zero the one in EFI header. This is needed to compute header checksum */
//     EFIHeader->HeaderCRC32 = 0;
//     /* Compute header checksum and compare with the one present in partition table */
//     if (RtlComputeCrc32(0, Disk->Buffer, sizeof(EFI_PARTITION_HEADER)) != HeaderCRC32)
//     {
//         DPRINT("EFI::Not matching header checksum!\n");
//         return STATUS_DISK_CORRUPT_ERROR;
//     }
//     /* Put back removed checksum in header */
//     EFIHeader->HeaderCRC32 = HeaderCRC32;
//
//     /* Check if current LBA is matching with ours */
//     if (EFIHeader->MyLBA != StartingSector)
//     {
//         DPRINT("EFI::Not matching starting sector!\n");
//         return STATUS_DISK_CORRUPT_ERROR;
//     }
//
//     /* Allocate a buffer to read a sector on the disk */
//     Sector = ExAllocatePoolWithTag(NonPagedPool,
//                                    Disk->SectorSize,
//                                    TAG_FSTUB);
//     if (!Sector)
//     {
//         DPRINT("EFI::Lacking resources!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Count how much sectors we'll have to read to read the whole partition table */
//     SectoredPartitionEntriesSize = (EFIHeader->NumberOfEntries * PARTITION_ENTRY_SIZE) / Disk->SectorSize;
//     /* Compute partition table checksum */
//     for (i = 0, PreviousCRC32 = 0; i < SectoredPartitionEntriesSize; i++)
//     {
//         Status = FstubReadSector(Disk->DeviceObject,
//                                  Disk->SectorSize,
//                                  EFIHeader->PartitionEntryLBA + i,
//                                  Sector);
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePoolWithTag(Sector, TAG_FSTUB);
//             DPRINT("EFI::Failed reading sector for partition entry!\n");
//             return Status;
//         }
//
//         PreviousCRC32 = RtlComputeCrc32(PreviousCRC32, Sector, Disk->SectorSize);
//     }
//
//     /* Check whether we have a last sector not full of partitions */
//     LonelyPartitions = (EFIHeader->NumberOfEntries * PARTITION_ENTRY_SIZE) % Disk->SectorSize;
//     /* In such case, we have to complete checksum computation */
//     if (LonelyPartitions != 0)
//     {
//         /* Read the sector that contains those partitions */
//         Status = FstubReadSector(Disk->DeviceObject,
//                                  Disk->SectorSize,
//                                  EFIHeader->PartitionEntryLBA + i,
//                                  Sector);
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePoolWithTag(Sector, TAG_FSTUB);
//             DPRINT("EFI::Failed reading sector for partition entry!\n");
//             return Status;
//         }
//
//         /* Then complete checksum by computing on each partition */
//         for (i = 0; i < LonelyPartitions; i++)
//         {
//             PreviousCRC32 = RtlComputeCrc32(PreviousCRC32, Sector + i * PARTITION_ENTRY_SIZE, PARTITION_ENTRY_SIZE);
//         }
//     }
//
//     /* Finally, release memory */
//     ExFreePoolWithTag(Sector, TAG_FSTUB);
//
//     /* Compare checksums */
//     if (PreviousCRC32 == EFIHeader->PartitionEntryCRC32)
//     {
//         /* In case of a success, return read header */
//         *HeaderBuffer = EFIHeader;
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         DPRINT("EFI::Not matching partition table checksum!\n");
//         DPRINT("EFI::Expected: %x, received: %x\n", EFIHeader->PartitionEntryCRC32, PreviousCRC32);
//         return STATUS_DISK_CORRUPT_ERROR;
//     }
// }
//
// NTSTATUS
// NTAPI
// FstubReadPartitionTableEFI(IN PDISK_INFORMATION Disk,
//                            IN BOOLEAN ReadBackupTable,
//                            OUT PDRIVE_LAYOUT_INFORMATION_EX* DriveLayout)
// {
//     NTSTATUS Status;
//     ULONG NumberOfEntries;
//     PEFI_PARTITION_HEADER EfiHeader;
//     EFI_PARTITION_ENTRY PartitionEntry;
// #if 0
//     BOOLEAN UpdatedPartitionTable = FALSE;
//     ULONGLONG SectorsForPartitions, PartitionEntryLBA;
// #else
//     ULONGLONG PartitionEntryLBA;
// #endif
//     PDRIVE_LAYOUT_INFORMATION_EX DriveLayoutEx = NULL;
//     ULONG i, PartitionCount, PartitionIndex, PartitionsPerSector;
//
//     PAGED_CODE();
//
//     ASSERT(Disk);
//
//     /* Zero output */
//     *DriveLayout = NULL;
//
//     /* Read EFI header */
//     Status = FstubReadHeaderEFI(Disk,
//                                 ReadBackupTable,
//                                 &EfiHeader);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Backup the number of entries, will be used later on */
//     NumberOfEntries = EfiHeader->NumberOfEntries;
//
//     /* Allocate a DRIVE_LAYOUT_INFORMATION_EX struct big enough */
//     DriveLayoutEx = ExAllocatePoolWithTag(NonPagedPool,
//                                           FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry) +
//                                           EfiHeader->NumberOfEntries * sizeof(PARTITION_INFORMATION_EX),
//                                           TAG_FSTUB);
//     if (!DriveLayoutEx)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
// #if 0
//     if (!ReadBackupTable)
//     {
//         /* If we weren't ask to read backup table,
//          * check the status of the backup table.
//          * In case it's not where we're expecting it, move it and ask
//          * for a partition table rewrite.
//          */
//         if ((Disk->SectorCount - 1ULL) != EfiHeader->AlternateLBA)
//         {
//             /* We'll update it. First, count number of sectors needed to store partitions */
//             SectorsForPartitions = ((ULONGLONG)EfiHeader->NumberOfEntries * PARTITION_ENTRY_SIZE) / Disk->SectorSize;
//             /* Then set first usable LBA: Legacy MBR + GPT header + Partitions entries */
//             EfiHeader->FirstUsableLBA = SectorsForPartitions + 2;
//             /* Then set last usable LBA: Last sector - GPT header - Partitions entries */
//             EfiHeader->LastUsableLBA = Disk->SectorCount - SectorsForPartitions - 1;
//             /* Inform that we'll rewrite partition table */
//             UpdatedPartitionTable = TRUE;
//         }
//     }
// #endif
//
//     DriveLayoutEx->PartitionStyle = PARTITION_STYLE_GPT;
//     /* Translate LBA -> Offset */
//     DriveLayoutEx->Gpt.StartingUsableOffset.QuadPart = EfiHeader->FirstUsableLBA * Disk->SectorSize;
//     DriveLayoutEx->Gpt.UsableLength.QuadPart = (EfiHeader->LastUsableLBA - EfiHeader->FirstUsableLBA) * Disk->SectorSize;
//     DriveLayoutEx->Gpt.MaxPartitionCount = EfiHeader->NumberOfEntries;
//     DriveLayoutEx->Gpt.DiskId = EfiHeader->DiskGUID;
//
//     /* Backup partition entry position */
//     PartitionEntryLBA = EfiHeader->PartitionEntryLBA;
//     /* Count number of partitions per sector */
//     PartitionsPerSector = (Disk->SectorSize / PARTITION_ENTRY_SIZE);
//     /* Read all partitions and fill in structure
//      * BEWARE! Past that point EfiHeader IS NOT VALID ANYMORE
//      * It will be erased by the reading of the partition entry
//      */
//     for (i = 0, PartitionCount = 0, PartitionIndex = PartitionsPerSector;
//          i < NumberOfEntries;
//          i++)
//     {
//         /* Only read following sector if we finished with previous sector */
//         if (PartitionIndex == PartitionsPerSector)
//         {
//             Status = FstubReadSector(Disk->DeviceObject,
//                                      Disk->SectorSize,
//                                      PartitionEntryLBA + (i / PartitionsPerSector),
//                                      Disk->Buffer);
//             if (!NT_SUCCESS(Status))
//             {
//                 ExFreePoolWithTag(DriveLayoutEx, TAG_FSTUB);
//                 return Status;
//             }
//
//             PartitionIndex = 0;
//         }
//         /* Read following partition */
//         PartitionEntry = ((PEFI_PARTITION_ENTRY)Disk->Buffer)[PartitionIndex];
//         PartitionIndex++;
//
//         /* If partition GUID is 00000000-0000-0000-0000-000000000000, then it's unused, skip it */
//         if (PartitionEntry.PartitionType.Data1 == 0 &&
//             PartitionEntry.PartitionType.Data2 == 0 &&
//             PartitionEntry.PartitionType.Data3 == 0 &&
//             ((PULONGLONG)PartitionEntry.PartitionType.Data4)[0] == 0)
//         {
//             continue;
//         }
//
//         /* Write data to structure. Don't forget GPT is using sectors, Windows offsets */
//         DriveLayoutEx->PartitionEntry[PartitionCount].StartingOffset.QuadPart = PartitionEntry.StartingLBA * Disk->SectorSize;
//         DriveLayoutEx->PartitionEntry[PartitionCount].PartitionLength.QuadPart = (PartitionEntry.EndingLBA -
//                                                                                   PartitionEntry.StartingLBA + 1) *
//                                                                                  Disk->SectorSize;
//         /* This number starts from 1 */
//         DriveLayoutEx->PartitionEntry[PartitionCount].PartitionNumber = PartitionCount + 1;
//         DriveLayoutEx->PartitionEntry[PartitionCount].RewritePartition = FALSE;
//         DriveLayoutEx->PartitionEntry[PartitionCount].PartitionStyle = PARTITION_STYLE_GPT;
//         DriveLayoutEx->PartitionEntry[PartitionCount].Gpt.PartitionType = PartitionEntry.PartitionType;
//         DriveLayoutEx->PartitionEntry[PartitionCount].Gpt.PartitionId = PartitionEntry.UniquePartition;
//         DriveLayoutEx->PartitionEntry[PartitionCount].Gpt.Attributes = PartitionEntry.Attributes;
//         RtlCopyMemory(DriveLayoutEx->PartitionEntry[PartitionCount].Gpt.Name,
//                       PartitionEntry.Name, sizeof(PartitionEntry.Name));
//
//         /* Update partition count */
//         PartitionCount++;
//     }
//     DriveLayoutEx->PartitionCount = PartitionCount;
//
// #if 0
//     /* If we updated partition table using backup table, rewrite partition table */
//     if (UpdatedPartitionTable)
//     {
//         IoWritePartitionTableEx(Disk->DeviceObject,
//                                 DriveLayoutEx);
//     }
// #endif
//
//     /* Finally, return read data */
//     *DriveLayout = DriveLayoutEx;
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubReadPartitionTableMBR(IN PDISK_INFORMATION Disk,
//                            IN BOOLEAN ReturnRecognizedPartitions,
//                            OUT PDRIVE_LAYOUT_INFORMATION_EX* ReturnedDriveLayout)
// {
//     NTSTATUS Status;
//     ULONG i;
//     PDRIVE_LAYOUT_INFORMATION DriveLayout = NULL;
//     PDRIVE_LAYOUT_INFORMATION_EX DriveLayoutEx = NULL;
//
//     PAGED_CODE();
//
//     ASSERT(IS_VALID_DISK_INFO(Disk));
//     ASSERT(ReturnedDriveLayout);
//
//     /* Zero output */
//     *ReturnedDriveLayout = NULL;
//
//     /* Read partition table the old way */
//     Status = IoReadPartitionTable(Disk->DeviceObject,
//                                   Disk->SectorSize,
//                                   ReturnRecognizedPartitions,
//                                   &DriveLayout);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Allocate a DRIVE_LAYOUT_INFORMATION_EX struct big enough */
//     DriveLayoutEx = ExAllocatePoolWithTag(NonPagedPool,
//                                           FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry) +
//                                           DriveLayout->PartitionCount * sizeof(PARTITION_INFORMATION_EX),
//                                           TAG_FSTUB);
//     if (!DriveLayoutEx)
//     {
//         /* Let's not leak memory as in Windows 2003 */
//         ExFreePool(DriveLayout);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Start converting the DRIVE_LAYOUT_INFORMATION structure */
//     DriveLayoutEx->PartitionStyle = PARTITION_STYLE_MBR;
//     DriveLayoutEx->PartitionCount = DriveLayout->PartitionCount;
//     DriveLayoutEx->Mbr.Signature = DriveLayout->Signature;
//
//     /* Convert each found partition */
//     for (i = 0; i < DriveLayout->PartitionCount; i++)
//     {
//         DriveLayoutEx->PartitionEntry[i].PartitionStyle = PARTITION_STYLE_MBR;
//         DriveLayoutEx->PartitionEntry[i].StartingOffset = DriveLayout->PartitionEntry[i].StartingOffset;
//         DriveLayoutEx->PartitionEntry[i].PartitionLength = DriveLayout->PartitionEntry[i].PartitionLength;
//         DriveLayoutEx->PartitionEntry[i].PartitionNumber = DriveLayout->PartitionEntry[i].PartitionNumber;
//         DriveLayoutEx->PartitionEntry[i].RewritePartition = DriveLayout->PartitionEntry[i].RewritePartition;
//         DriveLayoutEx->PartitionEntry[i].Mbr.PartitionType = DriveLayout->PartitionEntry[i].PartitionType;
//         DriveLayoutEx->PartitionEntry[i].Mbr.BootIndicator = DriveLayout->PartitionEntry[i].BootIndicator;
//         DriveLayoutEx->PartitionEntry[i].Mbr.RecognizedPartition = DriveLayout->PartitionEntry[i].RecognizedPartition;
//         DriveLayoutEx->PartitionEntry[i].Mbr.HiddenSectors = DriveLayout->PartitionEntry[i].HiddenSectors;
//     }
//
//     /* Finally, return data and free old structure */
//     *ReturnedDriveLayout = DriveLayoutEx;
//     ExFreePool(DriveLayout);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// FstubReadSector(IN PDEVICE_OBJECT DeviceObject,
//                 IN ULONG SectorSize,
//                 IN ULONGLONG StartingSector OPTIONAL,
//                 OUT PVOID Buffer)
// {
//     NTSTATUS Status;
//     PIRP Irp;
//     KEVENT Event;
//     LARGE_INTEGER StartingOffset;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION IoStackLocation;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(Buffer);
//     ASSERT(SectorSize);
//
//     /* Compute starting offset */
//     StartingOffset.QuadPart = StartingSector * SectorSize;
//
//     /* Initialize waiting event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Prepare IRP */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                        DeviceObject,
//                                        Buffer,
//                                        SectorSize,
//                                        &StartingOffset,
//                                        &Event,
//                                        &IoStatusBlock);
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Override volume verify */
//     IoStackLocation = IoGetNextIrpStackLocation(Irp);
//     IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//     /* Then call driver, and wait for completion if needed */
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
// NTSTATUS
// NTAPI
// FstubSetPartitionInformationEFI(IN PDISK_INFORMATION Disk,
//                                 IN ULONG PartitionNumber,
//                                 IN SET_PARTITION_INFORMATION_GPT * PartitionInfo)
// {
//     NTSTATUS Status;
//     PDRIVE_LAYOUT_INFORMATION_EX Layout = NULL;
//
//     PAGED_CODE();
//
//     ASSERT(Disk);
//     ASSERT(PartitionInfo);
//
//     /* Partition 0 isn't correct (should start at 1) */
//     if (PartitionNumber == 0)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Read partition table */
//     Status = IoReadPartitionTableEx(Disk->DeviceObject, &Layout);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//     ASSERT(Layout);
//
//     /* If our partition (started at 0 now) is higher than partition count, then, there's an issue */
//     if (Layout->PartitionCount <= --PartitionNumber)
//     {
//         ExFreePool(Layout);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Erase actual partition entry data with provided ones */
//     Layout->PartitionEntry[PartitionNumber].Gpt.PartitionType = PartitionInfo->PartitionType;
//     Layout->PartitionEntry[PartitionNumber].Gpt.PartitionId = PartitionInfo->PartitionId;
//     Layout->PartitionEntry[PartitionNumber].Gpt.Attributes = PartitionInfo->Attributes;
//     RtlCopyMemory(Layout->PartitionEntry[PartitionNumber].Gpt.Name, PartitionInfo->Name, sizeof(PartitionInfo->Name));
//
//     /* Rewrite the whole partition table to update the modified entry */
//     Status = IoWritePartitionTableEx(Disk->DeviceObject, Layout);
//
//     /* Free partition table and return */
//     ExFreePool(Layout);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubVerifyPartitionTableEFI(IN PDISK_INFORMATION Disk,
//                              IN BOOLEAN FixErrors)
// {
//     NTSTATUS Status;
//     PEFI_PARTITION_HEADER EFIHeader, ReadEFIHeader;
//     BOOLEAN PrimaryValid = FALSE, BackupValid = FALSE, WriteBackup;
//     ULONGLONG ReadPosition, WritePosition, SectorsForPartitions, PartitionIndex;
//
//     PAGED_CODE();
//
//     EFIHeader = ExAllocatePoolWithTag(NonPagedPool, sizeof(EFI_PARTITION_HEADER), TAG_FSTUB);
//     if (!EFIHeader)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = FstubReadHeaderEFI(Disk, FALSE, &ReadEFIHeader);
//     if (NT_SUCCESS(Status))
//     {
//         PrimaryValid = TRUE;
//         ASSERT(ReadEFIHeader);
//         RtlCopyMemory(EFIHeader, ReadEFIHeader, sizeof(EFI_PARTITION_HEADER));
//     }
//
//     Status = FstubReadHeaderEFI(Disk, TRUE, &ReadEFIHeader);
//     if (NT_SUCCESS(Status))
//     {
//         BackupValid = TRUE;
//         ASSERT(ReadEFIHeader);
//         RtlCopyMemory(EFIHeader, ReadEFIHeader, sizeof(EFI_PARTITION_HEADER));
//     }
//
//     /* If both are sane, just return */
//     if (PrimaryValid && BackupValid)
//     {
//         ExFreePoolWithTag(EFIHeader, TAG_FSTUB);
//         return STATUS_SUCCESS;
//     }
//
//     /* If both are damaged OR if we have not been ordered to fix
//      * Then, quit and warn about disk corruption
//      */
//     if ((!PrimaryValid && !BackupValid) || !FixErrors)
//     {
//         ExFreePoolWithTag(EFIHeader, TAG_FSTUB);
//         return STATUS_DISK_CORRUPT_ERROR;
//     }
//
//     /* Compute sectors taken by partitions */
//     SectorsForPartitions = (((ULONGLONG)EFIHeader->NumberOfEntries * PARTITION_ENTRY_SIZE) + Disk->SectorSize - 1) / Disk->SectorSize;
//     if (PrimaryValid)
//     {
//         WriteBackup = TRUE;
//         /* Take position at backup table for writing */
//         WritePosition = Disk->SectorCount - SectorsForPartitions;
//         /* And read from primary table */
//         ReadPosition = 2ULL;
//
//         DPRINT("EFI::Will repair backup table from primary\n");
//     }
//     else
//     {
//         ASSERT(BackupValid);
//         WriteBackup = FALSE;
//         /* Take position at primary table for writing */
//         WritePosition = 2ULL;
//         /* And read from backup table */
//         ReadPosition = Disk->SectorCount - SectorsForPartitions;
//
//         DPRINT("EFI::Will repair primary table from backup\n");
//     }
//
//     PartitionIndex = 0ULL;
//
//     /* If no partitions are to be copied, just restore header */
//     if (SectorsForPartitions <= 0)
//     {
//         Status = FstubWriteHeaderEFI(Disk,
//                                      SectorsForPartitions,
//                                      EFIHeader->DiskGUID,
//                                      EFIHeader->NumberOfEntries,
//                                      EFIHeader->FirstUsableLBA,
//                                      EFIHeader->LastUsableLBA,
//                                      EFIHeader->PartitionEntryCRC32,
//                                      WriteBackup);
//
//         goto Cleanup;
//     }
//
//     /* Copy all the partitions */
//     for (; PartitionIndex < SectorsForPartitions; ++PartitionIndex)
//     {
//         /* First, read the partition from the first table */
//         Status = FstubReadSector(Disk->DeviceObject,
//                                  Disk->SectorSize,
//                                  ReadPosition + PartitionIndex,
//                                  Disk->Buffer);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//
//         /* Then, write it in the other table */
//         Status = FstubWriteSector(Disk->DeviceObject,
//                                   Disk->SectorSize,
//                                   WritePosition + PartitionIndex,
//                                   Disk->Buffer);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//     }
//
//     /* Now we're done, write the header */
//     Status = FstubWriteHeaderEFI(Disk,
//                                  SectorsForPartitions,
//                                  EFIHeader->DiskGUID,
//                                  EFIHeader->NumberOfEntries,
//                                  EFIHeader->FirstUsableLBA,
//                                  EFIHeader->LastUsableLBA,
//                                  EFIHeader->PartitionEntryCRC32,
//                                  WriteBackup);
//
// Cleanup:
//     ExFreePoolWithTag(EFIHeader, TAG_FSTUB);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubWriteBootSectorEFI(IN PDISK_INFORMATION Disk)
// {
//     NTSTATUS Status;
//     ULONG Signature = 0;
//     PMASTER_BOOT_RECORD MasterBootRecord;
//
//     PAGED_CODE();
//
//     ASSERT(Disk);
//     ASSERT(IS_VALID_DISK_INFO(Disk));
//
//     /* Read if a MBR is already present */
//     Status = FstubReadSector(Disk->DeviceObject,
//                              Disk->SectorSize,
//                              0ULL,
//                              Disk->Buffer);
//     MasterBootRecord = (PMASTER_BOOT_RECORD)Disk->Buffer;
//     /* If one has been found */
//     if (NT_SUCCESS(Status) && MasterBootRecord->MasterBootRecordMagic == BOOT_RECORD_SIGNATURE)
//     {
//         /* Save its signature */
//         Signature = MasterBootRecord->Signature;
//     }
//
//     /* Reset the MBR */
//     RtlZeroMemory(MasterBootRecord, Disk->SectorSize);
//     /* Then create a fake MBR matching those purposes:
//      * It must have only partition. Type of this partition
//      * has to be 0xEE to signal a GPT is following.
//      * This partition has to cover the whole disk. To prevent
//      * any disk modification by a program that wouldn't
//      * understand anything to GPT.
//      */
//     MasterBootRecord->Signature = Signature;
//     MasterBootRecord->PartitionTable[0].StartSector = 2;
//     MasterBootRecord->PartitionTable[0].SystemIndicator = EFI_PMBR_OSTYPE_EFI;
//     MasterBootRecord->PartitionTable[0].EndHead = 0xFF;
//     MasterBootRecord->PartitionTable[0].EndSector = 0xFF;
//     MasterBootRecord->PartitionTable[0].EndCylinder = 0xFF;
//     MasterBootRecord->PartitionTable[0].SectorCountBeforePartition = 1;
//     MasterBootRecord->PartitionTable[0].PartitionSectorCount = 0xFFFFFFFF;
//     MasterBootRecord->MasterBootRecordMagic = BOOT_RECORD_SIGNATURE;
//
//     /* Finally, write that MBR */
//     return FstubWriteSector(Disk->DeviceObject,
//                             Disk->SectorSize,
//                             0,
//                             Disk->Buffer);
// }
//
// NTSTATUS
// NTAPI
// FstubWriteEntryEFI(IN PDISK_INFORMATION Disk,
//                    IN ULONG PartitionsSizeSector,
//                    IN ULONG PartitionEntryNumber,
//                    IN PEFI_PARTITION_ENTRY PartitionEntry,
//                    IN BOOLEAN WriteBackupTable,
//                    IN BOOLEAN ForceWrite,
//                    OUT PULONG PartitionEntryCRC32 OPTIONAL)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Offset;
//     ULONGLONG FirstEntryLBA;
//
//     PAGED_CODE();
//
//     ASSERT(Disk);
//     ASSERT(IS_VALID_DISK_INFO(Disk));
//
//     /* Get the first LBA where the partition table is:
//      * On primary table, it's sector 2 (skip MBR & Header)
//      * On backup table, it's ante last sector (Header) minus partition table size
//      */
//     if (!WriteBackupTable)
//     {
//         FirstEntryLBA = 2ULL;
//     }
//     else
//     {
//         FirstEntryLBA = Disk->SectorCount - PartitionsSizeSector - 1;
//     }
//
//     /* Copy the entry at the proper place into the buffer
//      * That way, we don't erase previous entries
//      */
//     RtlCopyMemory((PVOID)((ULONG_PTR)Disk->Buffer + ((PartitionEntryNumber * PARTITION_ENTRY_SIZE) % Disk->SectorSize)),
//                   PartitionEntry,
//                   sizeof(EFI_PARTITION_ENTRY));
//     /* Compute size of buffer */
//     Offset = (PartitionEntryNumber * PARTITION_ENTRY_SIZE) % Disk->SectorSize + PARTITION_ENTRY_SIZE;
//     ASSERT(Offset <= Disk->SectorSize);
//
//     /* If it's full of partition entries, or if call ask for it, write down the data */
//     if (Offset == Disk->SectorSize || ForceWrite)
//     {
//         /* We will write at first entry LBA + a shift made by already present/written entries */
//         Status = FstubWriteSector(Disk->DeviceObject,
//                                   Disk->SectorSize,
//                                   FirstEntryLBA + ((PartitionEntryNumber * PARTITION_ENTRY_SIZE) / Disk->SectorSize),
//                                   Disk->Buffer);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         /* We clean buffer */
//         RtlZeroMemory(Disk->Buffer, Disk->SectorSize);
//     }
//
//     /* If we have a buffer for CRC32, then compute it */
//     if (PartitionEntryCRC32)
//     {
//         *PartitionEntryCRC32 = RtlComputeCrc32(*PartitionEntryCRC32, (PUCHAR)PartitionEntry, PARTITION_ENTRY_SIZE);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubWriteHeaderEFI(IN PDISK_INFORMATION Disk,
//                     IN ULONG PartitionsSizeSector,
//                     IN GUID DiskGUID,
//                     IN ULONG NumberOfEntries,
//                     IN ULONGLONG FirstUsableLBA,
//                     IN ULONGLONG LastUsableLBA,
//                     IN ULONG PartitionEntryCRC32,
//                     IN BOOLEAN WriteBackupTable)
// {
//     PEFI_PARTITION_HEADER EFIHeader;
//
//     PAGED_CODE();
//
//     ASSERT(Disk);
//     ASSERT(IS_VALID_DISK_INFO(Disk));
//
//     /* Let's use read buffer as EFI_PARTITION_HEADER */
//     EFIHeader = (PEFI_PARTITION_HEADER)Disk->Buffer;
//
//     /* Complete standard header information */
//     EFIHeader->Signature = EFI_HEADER_SIGNATURE;
//     EFIHeader->Revision = EFI_HEADER_REVISION_1;
//     EFIHeader->HeaderSize = sizeof(EFI_PARTITION_HEADER);
//     /* Set no CRC32 checksum at the moment */
//     EFIHeader->HeaderCRC32 = 0;
//     EFIHeader->Reserved = 0;
//     /* Check whether we're writing primary or backup
//      * That way, we can ajust LBA setting:
//      * Primary is on first sector
//      * Backup is on last sector
//      */
//     if (!WriteBackupTable)
//     {
//         EFIHeader->MyLBA = 1ULL;
//         EFIHeader->AlternateLBA = Disk->SectorCount - 1ULL;
//     }
//     else
//     {
//         EFIHeader->MyLBA = Disk->SectorCount - 1ULL;
//         EFIHeader->AlternateLBA = 1ULL;
//     }
//     /* Fill in with received data */
//     EFIHeader->FirstUsableLBA = FirstUsableLBA;
//     EFIHeader->LastUsableLBA = LastUsableLBA;
//     EFIHeader->DiskGUID = DiskGUID;
//     /* Check whether we're writing primary or backup
//      * That way, we can ajust LBA setting:
//      * On primary, partition entries are just after header, so sector 2
//      * On backup, partition entries are just before header, so, last sector minus partition table size
//      */
//     if (!WriteBackupTable)
//     {
//         EFIHeader->PartitionEntryLBA = EFIHeader->MyLBA + 1ULL;
//     }
//     else
//     {
//         EFIHeader->PartitionEntryLBA = EFIHeader->MyLBA - PartitionsSizeSector;
//     }
//     /* Complete filling in */
//     EFIHeader->NumberOfEntries = NumberOfEntries;
//     EFIHeader->SizeOfPartitionEntry = PARTITION_ENTRY_SIZE;
//     EFIHeader->PartitionEntryCRC32 = PartitionEntryCRC32;
//     /* Finally, compute header checksum */
//     EFIHeader->HeaderCRC32 = RtlComputeCrc32(0, (PUCHAR)EFIHeader, sizeof(EFI_PARTITION_HEADER));
//
//     /* Debug the way we'll break disk, to let user pray */
//     DPRINT("FSTUB: About to write the following header for %s table\n", (WriteBackupTable ? "backup" : "primary"));
//     DPRINT(" Signature: %I64x\n", EFIHeader->Signature);
//     DPRINT(" Revision: %x\n", EFIHeader->Revision);
//     DPRINT(" HeaderSize: %x\n", EFIHeader->HeaderSize);
//     DPRINT(" HeaderCRC32: %x\n", EFIHeader->HeaderCRC32);
//     DPRINT(" MyLBA: %I64x\n", EFIHeader->MyLBA);
//     DPRINT(" AlternateLBA: %I64x\n", EFIHeader->AlternateLBA);
//     DPRINT(" FirstUsableLBA: %I64x\n", EFIHeader->FirstUsableLBA);
//     DPRINT(" LastUsableLBA: %I64x\n", EFIHeader->LastUsableLBA);
//     DPRINT(" PartitionEntryLBA: %I64x\n", EFIHeader->PartitionEntryLBA);
//     DPRINT(" NumberOfEntries: %x\n", EFIHeader->NumberOfEntries);
//     DPRINT(" SizeOfPartitionEntry: %x\n", EFIHeader->SizeOfPartitionEntry);
//     DPRINT(" PartitionEntryCRC32: %x\n", EFIHeader->PartitionEntryCRC32);
//
//     /* Write header to disk */
//     return FstubWriteSector(Disk->DeviceObject,
//                             Disk->SectorSize,
//                             EFIHeader->MyLBA,
//                             Disk->Buffer);
// }
//
// NTSTATUS
// NTAPI
// FstubWritePartitionTableEFI(IN PDISK_INFORMATION Disk,
//                             IN GUID DiskGUID,
//                             IN ULONG MaxPartitionCount,
//                             IN ULONGLONG FirstUsableLBA,
//                             IN ULONGLONG LastUsableLBA,
//                             IN BOOLEAN WriteBackupTable,
//                             IN ULONG PartitionCount,
//                             IN PPARTITION_INFORMATION_EX PartitionEntries OPTIONAL)
// {
//     NTSTATUS Status;
//     EFI_PARTITION_ENTRY Entry;
//     ULONG i, WrittenPartitions, SectoredPartitionEntriesSize, PartitionEntryCRC32;
//
//     PAGED_CODE();
//
//     ASSERT(Disk);
//     ASSERT(MaxPartitionCount >= 128);
//     ASSERT(PartitionCount <= MaxPartitionCount);
//
//     PartitionEntryCRC32 = 0;
//     /* Count how much sectors we'll have to read to read the whole partition table */
//     SectoredPartitionEntriesSize = (MaxPartitionCount * PARTITION_ENTRY_SIZE) / Disk->SectorSize;
//
//     for (i = 0, WrittenPartitions = 0; i < PartitionCount; i++)
//     {
//         /* If partition GUID is 00000000-0000-0000-0000-000000000000, then it's unused, skip it */
//         if (PartitionEntries[i].Gpt.PartitionType.Data1 == 0 &&
//             PartitionEntries[i].Gpt.PartitionType.Data2 == 0 &&
//             PartitionEntries[i].Gpt.PartitionType.Data3 == 0 &&
//             ((PULONGLONG)PartitionEntries[i].Gpt.PartitionType.Data4)[0] == 0)
//         {
//             continue;
//         }
//
//         /* Copy the entry in the partition entry format */
//         FstubCopyEntryEFI(&Entry, &PartitionEntries[i], Disk->SectorSize);
//         /* Then write the entry to the disk */
//         Status = FstubWriteEntryEFI(Disk,
//                                     SectoredPartitionEntriesSize,
//                                     WrittenPartitions,
//                                     &Entry,
//                                     WriteBackupTable,
//                                     FALSE,
//                                     &PartitionEntryCRC32);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//         WrittenPartitions++;
//     }
//
//     /* Zero the buffer to write zeros to the disk */
//     RtlZeroMemory(&Entry, sizeof(EFI_PARTITION_ENTRY));
//     /* Write the disks with zeros for every unused remaining partition entry */
//     for (i = WrittenPartitions; i < MaxPartitionCount; i++)
//     {
//         Status = FstubWriteEntryEFI(Disk,
//                                     SectoredPartitionEntriesSize,
//                                     i,
//                                     &Entry,
//                                     WriteBackupTable,
//                                     FALSE,
//                                     &PartitionEntryCRC32);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     /* Once we're done, write the GPT header */
//     return FstubWriteHeaderEFI(Disk,
//                                SectoredPartitionEntriesSize,
//                                DiskGUID,
//                                MaxPartitionCount,
//                                FirstUsableLBA,
//                                LastUsableLBA,
//                                PartitionEntryCRC32,
//                                WriteBackupTable);
// }
//
// NTSTATUS
// NTAPI
// FstubWritePartitionTableMBR(IN PDISK_INFORMATION Disk,
//                             IN PDRIVE_LAYOUT_INFORMATION_EX LayoutEx)
// {
//     NTSTATUS Status;
//     PDRIVE_LAYOUT_INFORMATION DriveLayout;
//
//     PAGED_CODE();
//
//     ASSERT(IS_VALID_DISK_INFO(Disk));
//     ASSERT(LayoutEx);
//
//     /* Convert data to the correct format */
//     DriveLayout = FstubConvertExtendedToLayout(LayoutEx);
//     if (!DriveLayout)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Really write information */
//     Status = IoWritePartitionTable(Disk->DeviceObject,
//                                    Disk->SectorSize,
//                                    Disk->DiskGeometry.Geometry.SectorsPerTrack,
//                                    Disk->DiskGeometry.Geometry.TracksPerCylinder,
//                                    DriveLayout);
//
//     /* Free allocated structure and return */
//     ExFreePoolWithTag(DriveLayout, TAG_FSTUB);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FstubWriteSector(IN PDEVICE_OBJECT DeviceObject,
//                  IN ULONG SectorSize,
//                  IN ULONGLONG StartingSector OPTIONAL,
//                  IN PVOID Buffer)
// {
//     NTSTATUS Status;
//     PIRP Irp;
//     KEVENT Event;
//     LARGE_INTEGER StartingOffset;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION IoStackLocation;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(Buffer);
//     ASSERT(SectorSize);
//
//     /* Compute starting offset */
//     StartingOffset.QuadPart = StartingSector * SectorSize;
//
//     /* Initialize waiting event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Prepare IRP */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_WRITE,
//                                        DeviceObject,
//                                        Buffer,
//                                        SectorSize,
//                                        &StartingOffset,
//                                        &Event,
//                                        &IoStatusBlock);
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Override volume verify */
//     IoStackLocation = IoGetNextIrpStackLocation(Irp);
//     IoStackLocation->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//     /* Then call driver, and wait for completion if needed */
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
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCreateDisk(IN PDEVICE_OBJECT DeviceObject,
//              IN PCREATE_DISK Disk)
// {
//     PARTITION_STYLE PartitionStyle;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//
//     /* Get partition style. If caller didn't provided data, assume it's raw */
//     PartitionStyle = ((Disk) ? Disk->PartitionStyle : PARTITION_STYLE_RAW);
//     /* Then, call appropriate internal function */
//     switch (PartitionStyle)
//     {
//         case PARTITION_STYLE_MBR:
//             return FstubCreateDiskMBR(DeviceObject, &(Disk->Mbr));
//         case PARTITION_STYLE_GPT:
//             return FstubCreateDiskEFI(DeviceObject, &(Disk->Gpt));
//         case PARTITION_STYLE_RAW:
//             return FstubCreateDiskRaw(DeviceObject);
//         default:
//             return STATUS_NOT_SUPPORTED;
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoGetBootDiskInformation(IN OUT PBOOTDISK_INFORMATION BootDiskInformation,
//                          IN ULONG Size)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIRP Irp;
//     KEVENT Event;
//     PLIST_ENTRY NextEntry;
//     PFILE_OBJECT FileObject;
//     DISK_GEOMETRY DiskGeometry;
//     PDEVICE_OBJECT DeviceObject;
//     UNICODE_STRING DeviceStringW;
//     IO_STATUS_BLOCK IoStatusBlock;
//     CHAR Buffer[128], ArcBuffer[128];
//     BOOLEAN SingleDisk, IsBootDiskInfoEx;
//     PARC_DISK_SIGNATURE ArcDiskSignature;
//     PARC_DISK_INFORMATION ArcDiskInformation;
//     PARTITION_INFORMATION_EX PartitionInformation;
//     PDRIVE_LAYOUT_INFORMATION_EX DriveLayout = NULL;
//     ULONG DiskCount, DiskNumber, Signature, PartitionNumber;
//     ANSI_STRING ArcBootString, ArcSystemString, DeviceStringA, ArcNameStringA;
//     extern PLOADER_PARAMETER_BLOCK IopLoaderBlock;
//
//     PAGED_CODE();
//
//     /* Get loader block. If it's null, we come too late */
//     if (!IopLoaderBlock)
//     {
//         return STATUS_TOO_LATE;
//     }
//
//     /* Check buffer size */
//     if (Size < sizeof(BOOTDISK_INFORMATION))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Init some useful stuff:
//      * Get ARC disks information
//      * Check whether we have a single disk on the machine
//      * Check received structure size (extended or not?)
//      * Init boot strings (system/boot)
//      * Finaly, get disk count
//      */
//     ArcDiskInformation = IopLoaderBlock->ArcDiskInformation;
//     SingleDisk = (ArcDiskInformation->DiskSignatureListHead.Flink->Flink ==
//                  &ArcDiskInformation->DiskSignatureListHead);
//
//     IsBootDiskInfoEx = (Size >= sizeof(BOOTDISK_INFORMATION_EX));
//     RtlInitAnsiString(&ArcBootString, IopLoaderBlock->ArcBootDeviceName);
//     RtlInitAnsiString(&ArcSystemString, IopLoaderBlock->ArcHalDeviceName);
//     DiskCount = IoGetConfigurationInformation()->DiskCount;
//
//     /* If no disk, return success */
//     if (DiskCount == 0)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Now, browse all disks */
//     for (DiskNumber = 0; DiskNumber < DiskCount; DiskNumber++)
//     {
//         /* Create the device name */
//         sprintf(Buffer, "\\Device\\Harddisk%lu\\Partition0", DiskNumber);
//         RtlInitAnsiString(&DeviceStringA, Buffer);
//         Status = RtlAnsiStringToUnicodeString(&DeviceStringW, &DeviceStringA, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             continue;
//         }
//
//         /* Get its device object */
//         Status = IoGetDeviceObjectPointer(&DeviceStringW,
//                                           FILE_READ_ATTRIBUTES,
//                                           &FileObject,
//                                           &DeviceObject);
//         RtlFreeUnicodeString(&DeviceStringW);
//         if (!NT_SUCCESS(Status))
//         {
//             continue;
//         }
//
//         /* Prepare for getting disk geometry */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//         Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_DRIVE_GEOMETRY,
//                                             DeviceObject,
//                                             NULL,
//                                             0,
//                                             &DiskGeometry,
//                                             sizeof(DiskGeometry),
//                                             FALSE,
//                                             &Event,
//                                             &IoStatusBlock);
//         if (!Irp)
//         {
//             ObDereferenceObject(FileObject);
//             continue;
//         }
//
//         /* Then, call the drive, and wait for it if needed */
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//         if (!NT_SUCCESS(Status))
//         {
//             ObDereferenceObject(FileObject);
//             continue;
//         }
//
//         /* Read partition table */
//         Status = IoReadPartitionTableEx(DeviceObject,
//                                         &DriveLayout);
//
//         /* FileObject, you can go! */
//         ObDereferenceObject(FileObject);
//
//         if (!NT_SUCCESS(Status))
//         {
//             continue;
//         }
//
//         /* Ensure we have at least 512 bytes per sector */
//         if (DiskGeometry.BytesPerSector < 512)
//         {
//             DiskGeometry.BytesPerSector = 512;
//         }
//
//         /* Now, for each ARC disk, try to find the matching */
//         for (NextEntry = ArcDiskInformation->DiskSignatureListHead.Flink;
//              NextEntry != &ArcDiskInformation->DiskSignatureListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             ArcDiskSignature = CONTAINING_RECORD(NextEntry,
//                                                  ARC_DISK_SIGNATURE,
//                                                  ListEntry);
//             /* If they match, i.e.
//              * - There's only one disk for both BIOS and detected
//              * - Signatures are matching
//              * - This is MBR
//              * (We don't check checksums here)
//              */
//             if (((SingleDisk && DiskCount == 1) ||
//                 (IopVerifyDiskSignature(DriveLayout, ArcDiskSignature, &Signature))) &&
//                 (DriveLayout->PartitionStyle == PARTITION_STYLE_MBR))
//             {
//                 /* Create ARC name */
//                 sprintf(ArcBuffer, "\\ArcName\\%s", ArcDiskSignature->ArcName);
//                 RtlInitAnsiString(&ArcNameStringA, ArcBuffer);
//
//                 /* Browse all partitions */
//                 for (PartitionNumber = 1; PartitionNumber <= DriveLayout->PartitionCount; PartitionNumber++)
//                 {
//                     /* Create its device name */
//                     sprintf(Buffer, "\\Device\\Harddisk%lu\\Partition%lu", DiskNumber, PartitionNumber);
//                     RtlInitAnsiString(&DeviceStringA, Buffer);
//                     Status = RtlAnsiStringToUnicodeString(&DeviceStringW, &DeviceStringA, TRUE);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         continue;
//                     }
//
//                     /* If IopVerifyDiskSignature returned no signature, take the one from DriveLayout */
//                     if (!Signature)
//                     {
//                         Signature = DriveLayout->Mbr.Signature;
//                     }
//
//                     /* Create partial ARC name */
//                     sprintf(ArcBuffer, "%spartition(%lu)", ArcDiskSignature->ArcName, PartitionNumber);
//                     RtlInitAnsiString(&ArcNameStringA, ArcBuffer);
//
//                     /* If it's matching boot string */
//                     if (RtlEqualString(&ArcNameStringA, &ArcBootString, TRUE))
//                     {
//                         /*  Then, fill in information about boot device */
//                         BootDiskInformation->BootDeviceSignature = Signature;
//
//                         /* Get its device object */
//                         Status = IoGetDeviceObjectPointer(&DeviceStringW,
//                                                           FILE_READ_ATTRIBUTES,
//                                                           &FileObject,
//                                                           &DeviceObject);
//                         if (!NT_SUCCESS(Status))
//                         {
//                             RtlFreeUnicodeString(&DeviceStringW);
//                             continue;
//                         }
//
//                         /* And call the drive to get information about partition */
//                         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//                         Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_PARTITION_INFO_EX,
//                                                             DeviceObject,
//                                                             NULL,
//                                                             0,
//                                                             &PartitionInformation,
//                                                             sizeof(PartitionInformation),
//                                                             FALSE,
//                                                             &Event,
//                                                             &IoStatusBlock);
//                         if (!Irp)
//                         {
//                             ObDereferenceObject(FileObject);
//                             RtlFreeUnicodeString(&DeviceStringW);
//                             continue;
//                         }
//
//                         /* Call & wait if needed */
//                         Status = IoCallDriver(DeviceObject, Irp);
//                         if (Status == STATUS_PENDING)
//                         {
//                             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                             Status = IoStatusBlock.Status;
//                         }
//                         if (!NT_SUCCESS(Status))
//                         {
//                             ObDereferenceObject(FileObject);
//                             RtlFreeUnicodeString(&DeviceStringW);
//                             continue;
//                         }
//
//                         /* We get partition offset as demanded and return it */
//                         BootDiskInformation->BootPartitionOffset = PartitionInformation.StartingOffset.QuadPart;
//
//                         /* If called passed a BOOTDISK_INFORMATION_EX structure, give more intel */
//                         if (IsBootDiskInfoEx)
//                         {
//                             /* Is partition style MBR or GPT? */
//                             if (DriveLayout->PartitionStyle == PARTITION_STYLE_GPT)
//                             {
//                                 ((PBOOTDISK_INFORMATION_EX)BootDiskInformation)->BootDeviceGuid = DriveLayout->Gpt.DiskId;
//                                 ((PBOOTDISK_INFORMATION_EX)BootDiskInformation)->BootDeviceIsGpt = TRUE;
//                             }
//                             else
//                             {
//                                 ((PBOOTDISK_INFORMATION_EX)BootDiskInformation)->BootDeviceIsGpt = FALSE;
//                             }
//                         }
//
//                         /* Dereference FileObject */
//                         ObDereferenceObject(FileObject);
//                     }
//
//                     /* If it's matching system string */
//                     if (RtlEqualString(&ArcNameStringA, &ArcSystemString, TRUE))
//                     {
//                         /* Then, fill in information about the system device */
//                         BootDiskInformation->SystemDeviceSignature = Signature;
//
//                         /* Get its device object */
//                         Status = IoGetDeviceObjectPointer(&DeviceStringW,
//                                                           FILE_READ_ATTRIBUTES,
//                                                           &FileObject,
//                                                           &DeviceObject);
//                         if (!NT_SUCCESS(Status))
//                         {
//                             RtlFreeUnicodeString(&DeviceStringW);
//                             continue;
//                         }
//
//                         /* And call the drive to get information about partition */
//                         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//                         Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_PARTITION_INFO_EX,
//                                                             DeviceObject,
//                                                             NULL,
//                                                             0,
//                                                             &PartitionInformation,
//                                                             sizeof(PartitionInformation),
//                                                             FALSE,
//                                                             &Event,
//                                                             &IoStatusBlock);
//                         if (!Irp)
//                         {
//                             ObDereferenceObject(FileObject);
//                             RtlFreeUnicodeString(&DeviceStringW);
//                             continue;
//                         }
//
//                         /* Call & wait if needed */
//                         Status = IoCallDriver(DeviceObject, Irp);
//                         if (Status == STATUS_PENDING)
//                         {
//                             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                             Status = IoStatusBlock.Status;
//                         }
//                         if (!NT_SUCCESS(Status))
//                         {
//                             ObDereferenceObject(FileObject);
//                             RtlFreeUnicodeString(&DeviceStringW);
//                             continue;
//                         }
//
//                         /* We get partition offset as demanded and return it */
//                         BootDiskInformation->SystemPartitionOffset = PartitionInformation.StartingOffset.QuadPart;
//
//                         /* If called passed a BOOTDISK_INFORMATION_EX structure, give more intel */
//                         if (IsBootDiskInfoEx)
//                         {
//                             /* Is partition style MBR or GPT? */
//                             if (DriveLayout->PartitionStyle == PARTITION_STYLE_GPT)
//                             {
//                                 ((PBOOTDISK_INFORMATION_EX)BootDiskInformation)->SystemDeviceGuid = DriveLayout->Gpt.DiskId;
//                                 ((PBOOTDISK_INFORMATION_EX)BootDiskInformation)->SystemDeviceIsGpt = TRUE;
//                             }
//                             else
//                             {
//                                 ((PBOOTDISK_INFORMATION_EX)BootDiskInformation)->SystemDeviceIsGpt = FALSE;
//                             }
//                         }
//
//                         /* Dereference FileObject */
//                         ObDereferenceObject(FileObject);
//                     }
//
//                     /* Release device string */
//                     RtlFreeUnicodeString(&DeviceStringW);
//                 }
//             }
//         }
//
//         /* Finally, release drive layout */
//         ExFreePool(DriveLayout);
//     }
//
//     /* And return */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoReadDiskSignature(IN PDEVICE_OBJECT DeviceObject,
//                     IN ULONG BytesPerSector,
//                     OUT PDISK_SIGNATURE Signature)
// {
//     NTSTATUS Status;
//     PUCHAR Buffer;
//     ULONG HeaderCRC32, i, CheckSum;
//     PEFI_PARTITION_HEADER EFIHeader;
//     PPARTITION_DESCRIPTOR PartitionDescriptor;
//
//     PAGED_CODE();
//
//     /* Ensure we'll read at least 512 bytes */
//     if (BytesPerSector < 512)
//     {
//         BytesPerSector = 512;
//     }
//
//     /* Allocate a buffer for reading operations */
//     Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, BytesPerSector, TAG_FSTUB);
//     if (!Buffer)
//     {
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Read first sector (sector 0) for MBR */
//     Status = FstubReadSector(DeviceObject,
//                              BytesPerSector,
//                              0ULL,
//                              Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     /* Get the partition descriptor array */
//     PartitionDescriptor = (PPARTITION_DESCRIPTOR)
//                           &(Buffer[PARTITION_TABLE_OFFSET]);
//     /* Check partitions types: if first is 0xEE and all the others 0, we have GPT */
//     if (PartitionDescriptor[0].PartitionType == EFI_PMBR_OSTYPE_EFI &&
//         PartitionDescriptor[1].PartitionType == 0 &&
//         PartitionDescriptor[2].PartitionType == 0 &&
//         PartitionDescriptor[3].PartitionType == 0)
//     {
//         /* If we have GPT, read second sector (sector 1) for GPT header */
//         Status = FstubReadSector(DeviceObject,
//                                  BytesPerSector,
//                                  1ULL,
//                                  Buffer);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//         EFIHeader = (PEFI_PARTITION_HEADER)Buffer;
//
//         /* First check signature
//          * Then, check version (we only support v1
//          * Finally check header size
//          */
//         if (EFIHeader->Signature != EFI_HEADER_SIGNATURE ||
//             EFIHeader->Revision != EFI_HEADER_REVISION_1 ||
//             EFIHeader->HeaderSize != sizeof(EFI_PARTITION_HEADER))
//         {
//             Status = STATUS_DISK_CORRUPT_ERROR;
//             goto Cleanup;
//         }
//
//         /* Save current checksum */
//         HeaderCRC32 = EFIHeader->HeaderCRC32;
//         /* Then zero the one in EFI header. This is needed to compute header checksum */
//         EFIHeader->HeaderCRC32 = 0;
//         /* Compute header checksum and compare with the one present in partition table */
//         if (RtlComputeCrc32(0, Buffer, sizeof(EFI_PARTITION_HEADER)) != HeaderCRC32)
//         {
//             Status = STATUS_DISK_CORRUPT_ERROR;
//             goto Cleanup;
//         }
//
//         /* Set partition table style to GPT and return disk GUID */
//         Signature->PartitionStyle = PARTITION_STYLE_GPT;
//         Signature->Gpt.DiskId = EFIHeader->DiskGUID;
//     }
//     else
//     {
//         /* Compute MBR checksum */
//         for (i = 0, CheckSum = 0; i < 512; i += sizeof(UINT32))
//         {
//             CheckSum += *(PUINT32)&Buffer[i];
//         }
//         CheckSum = ~CheckSum + 1;
//
//         /* Set partition table style to MBR and return signature (offset 440) and checksum */
//         Signature->PartitionStyle = PARTITION_STYLE_MBR;
//         Signature->Mbr.Signature = *(PUINT32)&Buffer[DISK_SIGNATURE_OFFSET];
//         Signature->Mbr.CheckSum = CheckSum;
//     }
//
// Cleanup:
//     /* Free buffer and return */
//     ExFreePoolWithTag(Buffer, TAG_FSTUB);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoReadPartitionTableEx(IN PDEVICE_OBJECT DeviceObject,
//                        IN PDRIVE_LAYOUT_INFORMATION_EX* DriveLayout)
// {
//     NTSTATUS Status;
//     PDISK_INFORMATION Disk;
//     PARTITION_STYLE PartitionStyle;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(DriveLayout);
//
//     /* First of all, allocate internal structure */
//     Status = FstubAllocateDiskInformation(DeviceObject, &Disk, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//     ASSERT(Disk);
//
//     /* Then, detect partition style (MBR? GPT/EFI? RAW?) */
//     Status = FstubDetectPartitionStyle(Disk, &PartitionStyle);
//     if (!NT_SUCCESS(Status))
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//
//     /* Here partition table is really read, depending on its style */
//     switch (PartitionStyle)
//     {
//         case PARTITION_STYLE_MBR:
//         case PARTITION_STYLE_RAW:
//             Status = FstubReadPartitionTableMBR(Disk, FALSE, DriveLayout);
//             break;
//
//         case PARTITION_STYLE_GPT:
//             /* Read primary table */
//             Status = FstubReadPartitionTableEFI(Disk, FALSE, DriveLayout);
//             /* If it failed, try reading backup table */
//             if (!NT_SUCCESS(Status))
//             {
//                 Status = FstubReadPartitionTableEFI(Disk, TRUE, DriveLayout);
//             }
//             break;
//
//         default:
//             DPRINT("Unknown partition type\n");
//             Status = STATUS_UNSUCCESSFUL;
//     }
//
//     /* It's over, internal structure not needed anymore */
//     FstubFreeDiskInformation(Disk);
//
//     /* In case of success, print data */
//     if (NT_SUCCESS(Status))
//     {
//         FstubDbgPrintDriveLayoutEx(*DriveLayout);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoSetPartitionInformationEx(IN PDEVICE_OBJECT DeviceObject,
//                             IN ULONG PartitionNumber,
//                             IN PSET_PARTITION_INFORMATION_EX PartitionInfo)
// {
//     NTSTATUS Status;
//     PDISK_INFORMATION Disk;
//     PARTITION_STYLE PartitionStyle;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(PartitionInfo);
//
//     /* Debug given modifications */
//     FstubDbgPrintSetPartitionEx(PartitionInfo, PartitionNumber);
//
//     /* Allocate internal structure */
//     Status = FstubAllocateDiskInformation(DeviceObject, &Disk, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Get partition table style on disk */
//     Status = FstubDetectPartitionStyle(Disk, &PartitionStyle);
//     if (!NT_SUCCESS(Status))
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//
//     /* If it's not matching partition style given in modifications, give up */
//     if (PartitionInfo->PartitionStyle != PartitionStyle)
//     {
//         FstubFreeDiskInformation(Disk);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Finally, handle modifications using proper function */
//     switch (PartitionStyle)
//     {
//         case PARTITION_STYLE_MBR:
//             Status = IoSetPartitionInformation(DeviceObject,
//                                                Disk->SectorSize,
//                                                PartitionNumber,
//                                                PartitionInfo->Mbr.PartitionType);
//             break;
//         case PARTITION_STYLE_GPT:
//             Status = FstubSetPartitionInformationEFI(Disk,
//                                                      PartitionNumber,
//                                                      &(PartitionInfo->Gpt));
//             break;
//         default:
//             Status = STATUS_NOT_SUPPORTED;
//     }
//
//     /* Release internal structure and return */
//     FstubFreeDiskInformation(Disk);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoVerifyPartitionTable(IN PDEVICE_OBJECT DeviceObject,
//                        IN BOOLEAN FixErrors)
// {
//     NTSTATUS Status;
//     PDISK_INFORMATION Disk;
//     PARTITION_STYLE PartitionStyle;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//
//     /* Allocate internal structure */
//     Status = FstubAllocateDiskInformation(DeviceObject, &Disk, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//     ASSERT(Disk);
//
//     /* Get partition table style on disk */
//     Status = FstubDetectPartitionStyle(Disk, &PartitionStyle);
//     if (!NT_SUCCESS(Status))
//     {
//         FstubFreeDiskInformation(Disk);
//         return Status;
//     }
//
//     /* Action will depend on partition style */
//     switch (PartitionStyle)
//     {
//         /* For MBR, assume it's always OK */
//         case PARTITION_STYLE_MBR:
//             Status = STATUS_SUCCESS;
//             break;
//         /* For GPT, call internal function */
//         case PARTITION_STYLE_GPT:
//             Status = FstubVerifyPartitionTableEFI(Disk, FixErrors);
//             break;
//         /* Otherwise, signal we can't work */
//         default:
//             Status = STATUS_NOT_SUPPORTED;
//     }
//
//     /* Release internal structure and return */
//     FstubFreeDiskInformation(Disk);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoWritePartitionTableEx(IN PDEVICE_OBJECT DeviceObject,
//                         IN PDRIVE_LAYOUT_INFORMATION_EX DriveLayout)
// {
//     NTSTATUS Status;
//     GUID DiskGuid;
//     ULONG NumberOfEntries;
//     PDISK_INFORMATION Disk;
//     PEFI_PARTITION_HEADER EfiHeader;
//     ULONGLONG SectorsForPartitions, FirstUsableLBA, LastUsableLBA;
//
//     PAGED_CODE();
//
//     ASSERT(DeviceObject);
//     ASSERT(DriveLayout);
//
//     /* Debug partition table that must be written */
//     FstubDbgPrintDriveLayoutEx(DriveLayout);
//
//     /* Allocate internal structure */
//     Status = FstubAllocateDiskInformation(DeviceObject, &Disk, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//     ASSERT(Disk);
//
//     switch (DriveLayout->PartitionStyle)
//     {
//         case PARTITION_STYLE_MBR:
//             Status = FstubWritePartitionTableMBR(Disk, DriveLayout);
//             break;
//
//         case PARTITION_STYLE_GPT:
//             /* Read primary table header */
//             Status = FstubReadHeaderEFI(Disk,
//                                         FALSE,
//                                         &EfiHeader);
//             /* If it failed, try reading back table header */
//             if (!NT_SUCCESS(Status))
//             {
//                 Status = FstubReadHeaderEFI(Disk,
//                                             TRUE,
//                                             &EfiHeader);
//             }
//
//             /* We have a header! */
//             if (NT_SUCCESS(Status))
//             {
//                 /* Check if there are enough places for the partitions to be written */
//                 if (DriveLayout->PartitionCount <= EfiHeader->NumberOfEntries)
//                 {
//                     /* Backup data */
//                     NumberOfEntries = EfiHeader->NumberOfEntries;
//                     RtlCopyMemory(&DiskGuid, &EfiHeader->DiskGUID, sizeof(GUID));
//                     /* Count number of sectors needed to store partitions */
//                     SectorsForPartitions = ((ULONGLONG)NumberOfEntries * PARTITION_ENTRY_SIZE) / Disk->SectorSize;
//                     /* Set first usable LBA: Legacy MBR + GPT header + Partitions entries */
//                     FirstUsableLBA = SectorsForPartitions + 2;
//                     /* Set last usable LBA: Last sector - GPT header - Partitions entries */
//                     LastUsableLBA = Disk->SectorCount - SectorsForPartitions - 1;
//                     /* Write primary table */
//                     Status = FstubWritePartitionTableEFI(Disk,
//                                                          DiskGuid,
//                                                          NumberOfEntries,
//                                                          FirstUsableLBA,
//                                                          LastUsableLBA,
//                                                          FALSE,
//                                                          DriveLayout->PartitionCount,
//                                                          DriveLayout->PartitionEntry);
//                     /* If it succeeded, also update backup table */
//                     if (NT_SUCCESS(Status))
//                     {
//                         Status = FstubWritePartitionTableEFI(Disk,
//                                                              DiskGuid,
//                                                              NumberOfEntries,
//                                                              FirstUsableLBA,
//                                                              LastUsableLBA,
//                                                              TRUE,
//                                                              DriveLayout->PartitionCount,
//                                                              DriveLayout->PartitionEntry);
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//             break;
//
//         default:
//             DPRINT("Unsupported partition style: %lu\n", DriveLayout->PartitionStyle);
//             Status = STATUS_NOT_SUPPORTED;
//     }
//
//     /* It's over, internal structure not needed anymore */
//     FstubFreeDiskInformation(Disk);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

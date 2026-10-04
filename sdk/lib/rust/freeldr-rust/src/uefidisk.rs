use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER, PCSTR};
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DiskPeripheral as DiskPeripheral;
pub use ntkrnl_sys::ddk::_ARC_CODES_EINVAL as EINVAL;
pub use ntkrnl_sys::ddk::_ARC_CODES_EIO as EIO;
pub use ntkrnl_sys::ddk::_ARC_CODES_ENOMEM as ENOMEM;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFirmwareTemporary as LoaderFirmwareTemporary;
pub use ntkrnl_sys::ddk::_SEEKMODE_SeekAbsolute as SeekAbsolute;
pub use ntkrnl_sys::ddk::_SEEKMODE_SeekRelative as SeekRelative;

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn FsGetDeviceSpecific(FileId: u32) -> *mut core::ffi::c_void;
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn DiskGetPartitionEntry(DriveNumber: UCHAR, PartitionNum: ULONG, PartitionTableEntry: PPARTITION_TABLE_ENTRY) -> BOOLEAN;
    fn ERR(fmt: *const i8, ...);
    // cross-module freeldr fns (FLDRAPI empty → cdecl).
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn FsRegisterDevice(DeviceName: *const i8, FuncTable: *const DEVVTBL);
    fn AddReactOSArcDiskInfo(ArcName: *mut i8, GptDiskGuid: *const c_void, Signature: ULONG, Checksum: ULONG, ValidPartitionTable: BOOLEAN);
    fn MmAllocateMemoryWithType(MemorySize: SIZE_T, MemoryType: TYPE_OF_MEMORY) -> PVOID;
    fn FrLdrTempAlloc(Size: SIZE_T, Tag: ULONG) -> PVOID;
    fn FrLdrTempFree(Allocation: PVOID, Tag: ULONG);
    fn DissectArcPath(ArcPath: PCSTR, Path: *mut PCSTR, DriveNumber: PUCHAR, PartitionNumber: PULONG) -> BOOLEAN;
    fn DiskDetectPartitionType(DriveNumber: UCHAR);
    fn FsSetDeviceSpecific(FileId: ULONG, Specific: PVOID);
    fn RtlStringCbPrintfA(pszDest: *mut i8, cbDest: usize, pszFormat: *const i8, ...) -> i32;
    // cross-module data from uefildr.c.
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
    static mut GlobalImageHandle: EFI_HANDLE;
    static mut PublicBootHandle: EFI_HANDLE;
    // PcBiosDiskCount is #[no_mangle]-owned by crate::hwdisk; extern here.
    static mut PcBiosDiskCount: UCHAR;
    // disk.h: extern CCHAR FrLdrBootPath[MAX_PATH]; defined by another freeldr TU.
    static mut FrLdrBootPath: [CHAR; MAX_PATH];
}

const MAX_SUPPORTED_BLOCK_SIZE: ULONG = 8192;

// uefidisk.c #defines.
const TAG_HW_DISK_CONTEXT: ULONG = u32::from_be_bytes(*b"cDwH");
const FIRST_BIOS_DISK: UCHAR = 0x80;
const FIRST_PARTITION: ULONG = 1;
const EFI_PARTITION_HEADER_SIGNATURE: &[u8; 9] = b"EFI PART\0";
const EFI_PARTITION_NAME_LENGTH: usize = 36;
const UNUSED_MAX_DESCRIPTOR_COUNT: ULONG = 0;
pub const EFI_PART_TYPE_UNUSED_GUID: EFI_GUID = EFI_GUID {
    Data1: 0, Data2: 0, Data3: 0, Data4: [0, 0, 0, 0, 0, 0, 0, 0],
};

// GPT structures (#include <pshpack1.h> → packed(1)).
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct GPT_TABLE_HEADER {
    pub Signature: [CHAR8; 8],
    pub Revision: UINT32,
    pub HeaderSize: UINT32,
    pub HeaderCrc32: UINT32,
    pub Reserved: UINT32,
    pub MyLba: UINT64,
    pub AlternateLba: UINT64,
    pub FirstUsableLba: UINT64,
    pub LastUsableLba: UINT64,
    pub DiskGuid: EFI_GUID,
    pub PartitionEntryLba: UINT64,
    pub NumberOfPartitionEntries: UINT32,
    pub SizeOfPartitionEntry: UINT32,
    pub PartitionEntryArrayCrc32: UINT32,
}
pub type PGPT_TABLE_HEADER = *mut GPT_TABLE_HEADER;

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct GPT_PARTITION_ENTRY {
    pub PartitionTypeGuid: EFI_GUID,
    pub UniquePartitionGuid: EFI_GUID,
    pub StartingLba: UINT64,
    pub EndingLba: UINT64,
    pub Attributes: UINT64,
    pub PartitionName: [CHAR16; EFI_PARTITION_NAME_LENGTH],
}
pub type PGPT_PARTITION_ENTRY = *mut GPT_PARTITION_ENTRY;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISKCONTEXT {
    pub DriveNumber: UCHAR,
    pub SectorSize: ULONG,
    pub SectorOffset: u64,
    pub SectorCount: u64,
    pub SectorNumber: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct INTERNAL_UEFI_DISK {
    pub ArcDriveNumber: UCHAR,
    pub NumOfPartitions: UCHAR,
    pub UefiHandleIndex: ULONG,
    pub IsThisTheBootDrive: BOOLEAN,
    pub Handle: EFI_HANDLE,
}
pub type PINTERNAL_UEFI_DISK = *mut INTERNAL_UEFI_DISK;

// uefidisk.c file-scope globals. Externally-linked ones (DiskReadBuffer,
// DiskReadBufferSize, FrldrBootDrive, FrldrBootPartition, Buffer) are the
// freeldr-rust definers (prev-green hwdisk/pcmem/keystone only extern them).
#[no_mangle]
pub static mut DiskReadBuffer: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut DiskReadBufferSize: SIZE_T = 0;
// FrldrBootDrive/Partition: per-binary-owner split.
// BIOS freeldr: DEFINED by arch/{i386,amd64}/entry.S (asm, in the BIOS
// freeldr_common only) — extern-consume here, do NOT #[no_mangle]
// (that dup'd entry.S at cold link; prev-green hwdisk.rs already externs
// FrldrBootPartition — same precedent).
// uefildr: has no entry.S and uefidisk.c is commented out, so uefidisk.rs
// (the uefidisk.c replacement) IS the UEFI owner — define them here.
#[cfg(not(feature = "uefi"))]
extern "C" {
    static mut FrldrBootDrive: UCHAR;
    static mut FrldrBootPartition: ULONG;
}
#[cfg(feature = "uefi")]
#[no_mangle]
pub static mut FrldrBootDrive: UCHAR = 0;
#[cfg(feature = "uefi")]
#[no_mangle]
pub static mut FrldrBootPartition: ULONG = 0;
// uefidisk.c `PVOID Buffer;` is vestigial (no cross-TU refs); rustconv
// translated the function-local `Buffer` vars as plain `Buffer` — a file
// static would shadow those. Locals are the real intent; no static here.
static mut DiskReadBufferRaw: PVOID = core::ptr::null_mut();
static mut DiskReadBufferAlignment: ULONG = 0;
static mut DiskReadBufferFromPool: BOOLEAN = 0;
// The buffer came from MmAllocateMemoryWithType, not BootServices->AllocatePool (upstream 492aeb639c9).
static mut DiskReadBufferFallbackPool: BOOLEAN = 0;
static Hex: [u8; 17] = *b"0123456789abcdef\0";
static mut PcDiskIdentifier: [[CHAR; 20]; 32] = [[0; 20]; 32];
static mut UefiBootRootIndex: ULONG = 0;
static mut PublicBootArcDisk: ULONG = 0;
static mut InternalUefiDisk: PINTERNAL_UEFI_DISK = core::ptr::null_mut();
static mut BlockIoGuid: EFI_GUID = EFI_BLOCK_IO_PROTOCOL_GUID;
static mut handles: *mut EFI_HANDLE = core::ptr::null_mut();
static mut HandleCount: ULONG = 0;
static mut lReportError: i32 = 0;

// uefidisk.c: static const DEVVTBL UefiDiskVtbl (dropped by translator).
static UefiDiskVtbl: DEVVTBL = DEVVTBL {
    Close: Some(UefiDiskClose),
    GetFileInformation: Some(UefiDiskGetFileInformation),
    Open: Some(UefiDiskOpen),
    Read: Some(UefiDiskRead),
    Seek: Some(UefiDiskSeek),
    ServiceName: core::ptr::null(),
};

unsafe fn UefiIsAlignedPointer(
    Pointer: PVOID,
    Alignment: ULONG,
) -> BOOLEAN {
    let Value: ULONG_PTR;

    if Alignment <= 1 {
        return 1u8;
    }

    Value = Pointer as ULONG_PTR;
    return if (Value & (((Alignment - 1)) as usize)) == 0 { 1u8 } else { 0u8 };
}

unsafe fn UefiEnsureDiskReadBufferAligned(
    Alignment: ULONG,
) -> BOOLEAN {
    let RequiredAlignment: ULONG = if Alignment == 0 { 1 } else { Alignment };
    let Status: EFI_STATUS;

    if DiskReadBuffer != core::ptr::null_mut() && DiskReadBufferAlignment >= RequiredAlignment &&
        UefiIsAlignedPointer(DiskReadBuffer, RequiredAlignment) != 0
    {
        return 1u8;
    }

    DiskReadBufferAlignment = RequiredAlignment;

    if DiskReadBufferRaw != core::ptr::null_mut() && DiskReadBufferFromPool != 0
    {
        if DiskReadBufferFallbackPool != 0 {
            crate::mm::MmFreeMemory(DiskReadBufferRaw);
        } else {
            (*(*GlobalSystemTable).BootServices).FreePool.unwrap()(DiskReadBufferRaw);
        }
        DiskReadBufferFallbackPool = 0u8;
        DiskReadBufferRaw = core::ptr::null_mut();
        DiskReadBuffer = core::ptr::null_mut();
        DiskReadBufferFromPool = 0u8;
    }

    Status = (*(*GlobalSystemTable).BootServices).AllocatePool.unwrap()(EfiLoaderData,
                                                           (DiskReadBufferSize + ((RequiredAlignment) as usize)) as UINTN ,
                                                           &mut DiskReadBufferRaw as *mut _ as *mut *mut core::ffi::c_void);
    if EFI_ERROR(Status) != 0 || DiskReadBufferRaw == core::ptr::null_mut()
    {
        // EFI 1.x firmware (Intel Macs among them) can refuse this; fall back to loader
        // memory instead of failing disk access outright (upstream 492aeb639c9).
        TRACE(b"AllocatePool failed for the disk read buffer, trying MmAllocateMemoryWithType (align %lu)\n\0".as_ptr() as PCHAR, (RequiredAlignment) as usize);
        DiskReadBufferRaw = MmAllocateMemoryWithType(
            (DiskReadBufferSize + (RequiredAlignment as usize)) as SIZE_T,
            LoaderFirmwareTemporary,
        );
        if DiskReadBufferRaw == core::ptr::null_mut() {
            ERR(b"Failed to allocate aligned disk read buffer (align %lu)\n\0".as_ptr() as PCHAR, (RequiredAlignment) as usize);
            DiskReadBuffer = core::ptr::null_mut();
            return 0u8;
        }
        DiskReadBufferFallbackPool = 1u8;
    }
    DiskReadBufferFromPool = 1u8;

    DiskReadBuffer = ALIGN_UP_POINTER_BY(DiskReadBufferRaw, (RequiredAlignment) as usize ) as PVOID;
    if UefiIsAlignedPointer(DiskReadBuffer, RequiredAlignment) == 0
    {
        ERR(b"Aligned disk read buffer is not properly aligned (align %lu)\n\0".as_ptr() as PCHAR, (RequiredAlignment) as usize);
        return 0u8;
    }

    return 1u8;
}

unsafe fn GetHarddiskIdentifier(DriveNumber: UCHAR) -> PCHAR {
    TRACE(b"GetHarddiskIdentifier: DriveNumber: %d\n\0".as_ptr() as *const i8, (DriveNumber) as usize);
    if DriveNumber < FIRST_BIOS_DISK {
        return core::ptr::null_mut();
    }
    PcDiskIdentifier[(DriveNumber - FIRST_BIOS_DISK) as usize].as_mut_ptr()
}

unsafe fn DiskReportError(bShowError: BOOLEAN) -> LONG {
    if bShowError != 0 {
        lReportError += 1;
    } else {
        lReportError -= 1;
    }
    lReportError
}

unsafe fn UefiReadGptHeader(
    DriveNumber: UCHAR,
    GptHeader: PGPT_TABLE_HEADER,
) -> BOOLEAN {
    let ArcDriveIndex: ULONG;
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let _Status: EFI_STATUS;
    let BlockSize: ULONG;
    let IoAlign: ULONG;
    let HeaderLba: ULONGLONG = 1;

    if DriveNumber < 0x80 {
        return 0u8;
    }

    ArcDriveIndex = (DriveNumber - 0x80) as ULONG;
    if ArcDriveIndex >= ((PcBiosDiskCount) as u32) || InternalUefiDisk == core::ptr::null_mut() {
        return 0u8;
    }

    let Status = ((*(*GlobalSystemTable).BootServices).HandleProtocol).unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        &mut BlockIo as *mut *mut EFI_BLOCK_IO as *mut *mut core::ffi::c_void,
    );

    if Status != 0 || BlockIo == core::ptr::null_mut() {
        return 0u8;
    }

    if (*(*BlockIo).Media).MediaPresent == 0 {
        return 0u8;
    }

    BlockSize = (*(*BlockIo).Media).BlockSize;
    IoAlign = (*(*BlockIo).Media).IoAlign;

    if UefiEnsureDiskReadBufferAligned(IoAlign) == 0 {
        return 0u8;
    }

    let Status = ((*BlockIo).ReadBlocks).unwrap()(
        BlockIo,
        (*(*BlockIo).Media).MediaId,
        HeaderLba,
        BlockSize as UINTN,
        DiskReadBuffer,
    );

    if Status != 0 {
        return 0u8;
    }

    RtlCopyMemory(
        GptHeader as *mut core::ffi::c_void,
        DiskReadBuffer as *const core::ffi::c_void,
        core::mem::size_of::<GPT_TABLE_HEADER>(),
    );

    if memcmp(
        (*GptHeader).Signature.as_ptr() as *const core::ffi::c_void,
        EFI_PARTITION_HEADER_SIGNATURE.as_ptr() as *const core::ffi::c_void,
        8,
    ) != 0
    {
        return 0u8;
    }

    if (*GptHeader).Revision != 0x00010000 {
        return 0u8;
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn UefiGetGptPartitionEntry(
    DriveNumber: UCHAR,
    PartitionNumber: ULONG,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
) -> BOOLEAN {

    let mut GptHeader: GPT_TABLE_HEADER = core::mem::zeroed();
    let mut GptEntry: GPT_PARTITION_ENTRY = core::mem::zeroed();
    let ArcDriveIndex: ULONG;
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let mut Status: EFI_STATUS;
    let BlockSize: ULONG;
    let EntryLba: ULONGLONG;
    let EntryOffset: ULONG;
    let EntriesPerBlock: ULONG;
    let UnusedGuid: EFI_GUID = EFI_PART_TYPE_UNUSED_GUID;

    if DriveNumber < FIRST_BIOS_DISK {
        return 0u8;
    }

    ArcDriveIndex = DriveNumber.wrapping_sub(FIRST_BIOS_DISK) as ULONG;
    if ArcDriveIndex >= ((PcBiosDiskCount) as u32) || InternalUefiDisk == core::ptr::null_mut() {
        return 0u8;
    }

    if UefiReadGptHeader(DriveNumber, &mut GptHeader) == 0 {
        return 0u8;
    }

    if PartitionNumber == 0 || PartitionNumber > GptHeader.NumberOfPartitionEntries {
        return 0u8;
    }

    let EntryIndex: ULONG = PartitionNumber - 1;

    Status = ((*(*GlobalSystemTable).BootServices).HandleProtocol).unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        &mut BlockIo as *mut *mut EFI_BLOCK_IO as *mut *mut core::ffi::c_void,
    );

    if EFI_ERROR(Status) != 0 || BlockIo == core::ptr::null_mut() {
        return 0u8;
    }

    BlockSize = (*(*BlockIo).Media).BlockSize;
    EntriesPerBlock = BlockSize / GptHeader.SizeOfPartitionEntry;
    EntryLba = GptHeader.PartitionEntryLba + (EntryIndex / EntriesPerBlock) as ULONGLONG;
    EntryOffset = (EntryIndex % EntriesPerBlock) * GptHeader.SizeOfPartitionEntry;

    Status = ((*BlockIo).ReadBlocks).unwrap()(
        BlockIo,
        (*(*BlockIo).Media).MediaId,
        EntryLba,
        BlockSize as UINTN,
        DiskReadBuffer as *mut core::ffi::c_void,
    );

    if EFI_ERROR(Status) != 0 {
        return 0u8;
    }

    RtlCopyMemory(
        &mut GptEntry as *mut GPT_PARTITION_ENTRY as *mut core::ffi::c_void,
        (DiskReadBuffer as PUCHAR).offset(EntryOffset as isize) as *const core::ffi::c_void,
        core::mem::size_of::<GPT_PARTITION_ENTRY>(),
    );

    if memcmp(
        core::ptr::addr_of!(GptEntry.PartitionTypeGuid) as *const EFI_GUID as *const core::ffi::c_void,
        &UnusedGuid as *const EFI_GUID as *const core::ffi::c_void,
        core::mem::size_of::<EFI_GUID>(),
    ) == 0
    {
        return 0u8;
    }

    RtlZeroMemory(
        PartitionTableEntry as *mut core::ffi::c_void,
        core::mem::size_of::<PARTITION_TABLE_ENTRY>(),
    );

    let StartLba: ULONGLONG = GptEntry.StartingLba;
    let EndLba: ULONGLONG = GptEntry.EndingLba;
    let SectorCount: ULONGLONG = EndLba - StartLba + 1;

    let StartSector: ULONGLONG = (StartLba * BlockSize as ULONGLONG) / 512;
    let SectorCount512: ULONGLONG = (SectorCount * BlockSize as ULONGLONG) / 512;

    (*PartitionTableEntry).SectorCountBeforePartition = StartSector as ULONG;
    (*PartitionTableEntry).PartitionSectorCount = SectorCount512 as ULONG;
    (*PartitionTableEntry).SystemIndicator = PARTITION_GPT;

    1u8
}

unsafe fn UefiGetBootPartitionEntry(
    DriveNumber: UCHAR,
    PartitionTableEntry: PPARTITION_TABLE_ENTRY,
    BootPartition: PULONG,
) -> BOOLEAN {
    let mut PartitionNum: ULONG;
    let ArcDriveIndex: ULONG;
    let mut BootBlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let Status: EFI_STATUS;
    let BootPartitionSize: ULONGLONG;
    let mut TempPartitionEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();

    TRACE(b"UefiGetBootPartitionEntry: DriveNumber: %d\n\0".as_ptr() as PCHAR, (DriveNumber - FIRST_BIOS_DISK) as usize);

    if DriveNumber < FIRST_BIOS_DISK {
        return 0u8;
    }

    ArcDriveIndex = (DriveNumber - FIRST_BIOS_DISK) as u32 ;
    if ArcDriveIndex >= ((PcBiosDiskCount) as u32) || handles == core::ptr::null_mut() || UefiBootRootIndex >= HandleCount {
        return 0u8;
    }

    Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
        *handles.offset(UefiBootRootIndex as isize),
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(BootBlockIo) as *mut *mut core::ffi::c_void,
    );

    if EFI_ERROR(Status) != 0 || BootBlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol for boot handle\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    BootPartitionSize = (*(*BootBlockIo).Media).LastBlock + 1;

    TRACE(
        b"Boot partition: Size=%llu blocks, BlockSize=%lu\n\0".as_ptr() as PCHAR, (BootPartitionSize) as usize, ((*(*BootBlockIo).Media).BlockSize) as usize,
    );

    if (*(*BootBlockIo).Media).LogicalPartition == 0 {
        TRACE(b"Boot handle is root device, using partition 0\n\0".as_ptr() as PCHAR);
        *BootPartition = 0;
        if PartitionTableEntry != core::ptr::null_mut() {
            core::ptr::write_bytes(PartitionTableEntry, 0, 1);
        }
        return 1u8;
    }

    let mut GptHeader: GPT_TABLE_HEADER = core::mem::zeroed();
    let IsGpt: BOOLEAN = UefiReadGptHeader(DriveNumber, core::ptr::addr_of_mut!(GptHeader));

    if IsGpt != 0 {
        let mut GptHeader: GPT_TABLE_HEADER = core::mem::zeroed();
        let mut GptEntry: GPT_PARTITION_ENTRY = core::mem::zeroed();
        let ArcDriveIndex: ULONG = (DriveNumber - FIRST_BIOS_DISK) as u32 ;
        let mut RootBlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
        let BlockSize: ULONG;
        let mut EntryLba: ULONGLONG;
        let mut EntryOffset: ULONG;
        let EntriesPerBlock: ULONG;
        let mut Status: EFI_STATUS;
        let UnusedGuid: EFI_GUID = EFI_PART_TYPE_UNUSED_GUID;

        if UefiReadGptHeader(DriveNumber, core::ptr::addr_of_mut!(GptHeader)) == 0 {
            ERR(b"Failed to read GPT header\n\0".as_ptr() as PCHAR);
            return 0u8;
        }

        Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
            (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
            core::ptr::addr_of_mut!(BlockIoGuid),
            core::ptr::addr_of_mut!(RootBlockIo) as *mut *mut core::ffi::c_void,
        );

        if EFI_ERROR(Status) != 0 || RootBlockIo == core::ptr::null_mut() {
            return 0u8;
        }

        BlockSize = (*(*RootBlockIo).Media).BlockSize;
        EntriesPerBlock = BlockSize / GptHeader.SizeOfPartitionEntry;

        for i in 0..GptHeader.NumberOfPartitionEntries {
            EntryLba = GptHeader.PartitionEntryLba + (i / EntriesPerBlock) as ULONGLONG;
            EntryOffset = (i % EntriesPerBlock) * GptHeader.SizeOfPartitionEntry;

            Status = (*RootBlockIo).ReadBlocks.unwrap()(
                RootBlockIo,
                (*(*RootBlockIo).Media).MediaId,
                EntryLba,
                BlockSize as UINTN,
                DiskReadBuffer as *mut core::ffi::c_void,
            );

            if EFI_ERROR(Status) != 0 {
                continue;
            }

            core::ptr::copy_nonoverlapping(
                (DiskReadBuffer as PUCHAR).offset(EntryOffset as isize),
                core::ptr::addr_of_mut!(GptEntry) as PUCHAR,
                core::mem::size_of::<GPT_PARTITION_ENTRY>(),
            );

            if memcmp(
                core::ptr::addr_of!(GptEntry.PartitionTypeGuid) as *const core::ffi::c_void,
                core::ptr::addr_of!(UnusedGuid) as *const core::ffi::c_void,
                core::mem::size_of::<EFI_GUID>(),
            ) == 0
            {
                continue;
            }

            let PartitionSizeBlocks: ULONGLONG = GptEntry.EndingLba - GptEntry.StartingLba + 1;

            TRACE(
                b"GPT Partition %lu: StartLba=%llu, EndLba=%llu, SizeBlocks=%llu\n\0".as_ptr() as PCHAR, (i + 1) as usize, (GptEntry.StartingLba) as usize, (GptEntry.EndingLba) as usize, (PartitionSizeBlocks) as usize,
            );

            if PartitionSizeBlocks == BootPartitionSize
                || (PartitionSizeBlocks > 0
                    && (PartitionSizeBlocks - 1 <= BootPartitionSize
                        && BootPartitionSize <= PartitionSizeBlocks + 1))
            {
                TRACE(
                    b"Found matching GPT partition %lu: Size matches (%llu blocks)\n\0".as_ptr() as PCHAR, (i + 1) as usize, (BootPartitionSize) as usize,
                );

                *BootPartition = i + 1;

                if PartitionTableEntry != core::ptr::null_mut() {
                    core::ptr::write_bytes(PartitionTableEntry, 0, 1);
                    let StartSector: ULONGLONG = (GptEntry.StartingLba * BlockSize as ULONGLONG) / 512;
                    let SectorCount: ULONGLONG = (PartitionSizeBlocks * BlockSize as ULONGLONG) / 512;
                    (*PartitionTableEntry).SectorCountBeforePartition = StartSector as ULONG;
                    (*PartitionTableEntry).PartitionSectorCount = SectorCount as ULONG;
                    (*PartitionTableEntry).SystemIndicator = PARTITION_GPT;
                }
                return 1u8;
            }
        }
    } else {
        PartitionNum = FIRST_PARTITION;
        while DiskGetPartitionEntry(DriveNumber, PartitionNum, core::ptr::addr_of_mut!(TempPartitionEntry)) != 0 {
            let PartitionSizeSectors: ULONGLONG = TempPartitionEntry.PartitionSectorCount as ULONGLONG;
            let PartitionSizeBlocks: ULONGLONG;

            let PartitionSizeBytes: ULONGLONG = PartitionSizeSectors * 512u64;
            PartitionSizeBlocks = PartitionSizeBytes / (*(*BootBlockIo).Media).BlockSize as ULONGLONG;

            TRACE(
                b"Partition %lu: SizeSectors=%llu, SizeBlocks=%llu\n\0".as_ptr() as PCHAR, (PartitionNum) as usize, (PartitionSizeSectors) as usize, (PartitionSizeBlocks) as usize,
            );

            if PartitionSizeBlocks == BootPartitionSize
                || (PartitionSizeBlocks > 0
                    && (PartitionSizeBlocks - 1 <= BootPartitionSize
                        && BootPartitionSize <= PartitionSizeBlocks + 1))
            {
                TRACE(
                    b"Found matching partition %lu: Size matches (%llu blocks)\n\0".as_ptr() as PCHAR, (PartitionNum) as usize, (BootPartitionSize) as usize,
                );

                *BootPartition = PartitionNum;
                if PartitionTableEntry != core::ptr::null_mut() {
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!(TempPartitionEntry) as *const u8,
                        PartitionTableEntry as *mut u8,
                        core::mem::size_of::<PARTITION_TABLE_ENTRY>(),
                    );
                }
                return 1u8;
            }

            PartitionNum += 1;
        }
    }

    if (*(*BootBlockIo).Media).RemovableMedia != 0 && (*(*BootBlockIo).Media).BlockSize == 2048 {
        TRACE(b"Boot device is CD-ROM, using partition 0xFF\n\0".as_ptr() as PCHAR);
        *BootPartition = 0xFF;
        if PartitionTableEntry != core::ptr::null_mut() {
            core::ptr::write_bytes(PartitionTableEntry, 0, 1);
        }
        return 1u8;
    }

    ERR(b"Could not determine boot partition, using partition 1 as fallback\n\0".as_ptr() as PCHAR);
    PartitionNum = FIRST_PARTITION;
    if DiskGetPartitionEntry(DriveNumber, PartitionNum, core::ptr::addr_of_mut!(TempPartitionEntry)) != 0 {
        *BootPartition = PartitionNum;
        if PartitionTableEntry != core::ptr::null_mut() {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!(TempPartitionEntry) as *const u8,
                PartitionTableEntry as *mut u8,
                core::mem::size_of::<PARTITION_TABLE_ENTRY>(),
            );
        }
        return 1u8;
    }

    0u8
}

unsafe extern "C" fn UefiDiskClose(FileId: u32) -> ARC_STATUS {
    let Context = FsGetDeviceSpecific(FileId) as *mut DISKCONTEXT;
    // The C frees with TAG_HW_DISK_CONTEXT (its own comment says so, below).
    // Translation substituted the magic literal 0x4B534944 -- ASCII "DISK",
    // which is not this tag and not any tag: the block is allocated below with
    // TAG_HW_DISK_CONTEXT ('cDwH' = 0x63447748), so every UefiDiskClose hit
    // FrLdrHeapFreeEx's tag-mismatch branch. Dead under a dead ASSERT on
    // i386/amd64; on arm64 DbgBreakPoint is a real `brk` and it wedged the boot.
    FrLdrTempFree(Context as *mut core::ffi::c_void, TAG_HW_DISK_CONTEXT);
    ESUCCESS
}

unsafe extern "C" fn UefiDiskGetFileInformation(FileId: ULONG, Information: *mut FILEINFORMATION) -> ARC_STATUS {
    
    let Context = FsGetDeviceSpecific(FileId) as *mut DISKCONTEXT;
    core::ptr::write_bytes(Information, 0, 1);

    (*Information).StartingAddress.QuadPart = ((*Context).SectorOffset * (((*Context).SectorSize) as u64)) as i64;
    (*Information).EndingAddress.QuadPart = (((*Context).SectorOffset + (*Context).SectorCount) * (((*Context).SectorSize) as u64)) as i64;
    (*Information).CurrentAddress.QuadPart = ((*Context).SectorNumber * (((*Context).SectorSize) as u64)) as i64;

    (*Information).Type = DiskPeripheral;

    ESUCCESS
}

unsafe extern "C" fn UefiDiskOpen(Path: PCHAR, _OpenMode: OPENMODE, FileId: *mut ULONG) -> ARC_STATUS {
    let Context: *mut DISKCONTEXT;
    let mut DriveNumber: UCHAR = 0;
    let mut DrivePartition: ULONG = 0;
    let SectorSize: ULONG;
    let mut SectorOffset: ULONGLONG = 0;
    let mut SectorCount: ULONGLONG = 0;
    let ArcDriveIndex: ULONG;
    let mut PartitionTableEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let Status: EFI_STATUS;

    TRACE(b"UefiDiskOpen: File ID: %p, Path: %s\n\0".as_ptr() as PCHAR, (FileId) as usize, (Path) as usize);

    if DiskReadBufferSize == 0 {
        ERR(b"DiskOpen(): DiskReadBufferSize is 0, something is wrong.\n\0".as_ptr() as PCHAR);
        ASSERT(0u8);
        return ENOMEM;
    }

    if DissectArcPath(Path, core::ptr::null_mut(), &mut DriveNumber, &mut DrivePartition) == 0 {
        return EINVAL;
    }

    TRACE(b"Opening disk: DriveNumber: %d, DrivePartition: %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize, (DrivePartition) as usize);

    if DriveNumber < FIRST_BIOS_DISK {
        return EINVAL;
    }

    ArcDriveIndex = (DriveNumber - FIRST_BIOS_DISK) as u32 ;
    if ArcDriveIndex >= ((PcBiosDiskCount) as u32) || InternalUefiDisk == core::ptr::null_mut() {
        return EINVAL;
    }

    Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        &mut BlockIo as *mut *mut EFI_BLOCK_IO as *mut *mut core::ffi::c_void
    );

    if EFI_ERROR(Status) != 0 || BlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return EINVAL;
    }

    if (*(*BlockIo).Media).MediaPresent == 0 {
        ERR(b"Media not present for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return EINVAL;
    }

    SectorSize = (*(*BlockIo).Media).BlockSize;

    if DrivePartition != 0xff && DrivePartition != 0 {
        if DiskGetPartitionEntry(DriveNumber, DrivePartition, &mut PartitionTableEntry) == 0 {
            return EINVAL;
        }

        SectorOffset = (PartitionTableEntry.SectorCountBeforePartition) as u64 ;
        SectorCount = (PartitionTableEntry.PartitionSectorCount) as u64 ;
    } else {
        let mut Geometry: GEOMETRY = core::mem::zeroed();
        if MachDiskGetDriveGeometry(DriveNumber, &mut Geometry) == 0 {
            return EINVAL;
        }

        if SectorSize != Geometry.BytesPerSector {
            ERR(b"SectorSize (%lu) != Geometry.BytesPerSector (%lu), expect problems!\n\0".as_ptr() as PCHAR, (SectorSize) as usize, (Geometry.BytesPerSector) as usize);
        }

        SectorOffset = 0;
        SectorCount = Geometry.Sectors;
    }

    Context = FrLdrTempAlloc(core::mem::size_of::<DISKCONTEXT>() as usize, TAG_HW_DISK_CONTEXT) as *mut DISKCONTEXT;
    if Context == core::ptr::null_mut() {
        return ENOMEM;
    }

    (*Context).DriveNumber = DriveNumber;
    (*Context).SectorSize = SectorSize;
    (*Context).SectorOffset = SectorOffset;
    (*Context).SectorCount = SectorCount;
    (*Context).SectorNumber = 0;
    FsSetDeviceSpecific(*FileId, Context as *mut core::ffi::c_void);
    ESUCCESS
}

unsafe extern "C" fn UefiDiskRead(FileId: ULONG, Buffer: PVOID, N: ULONG, Count: *mut ULONG) -> ARC_STATUS {
    let Context: *mut DISKCONTEXT = FsGetDeviceSpecific(FileId) as *mut DISKCONTEXT;
    let mut Ptr: *mut UCHAR = Buffer as *mut UCHAR;
    let mut Length: ULONG;
    let mut TotalSectors: ULONG;
    let MaxSectors: ULONG;
    let mut ReadSectors: ULONG;
    let mut SectorOffset: ULONGLONG;
    let mut ret: BOOLEAN;
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let _Status: EFI_STATUS;
    let ArcDriveIndex: ULONG;
    let mut N_remaining: ULONG = N;

    TotalSectors = (N + (*Context).SectorSize - 1) / (*Context).SectorSize;
    MaxSectors = (DiskReadBufferSize / (((*Context).SectorSize) as usize)) as u32 ;
    SectorOffset = (*Context).SectorOffset + (*Context).SectorNumber;

    if MaxSectors == 0 {
        ERR(b"MaxSectors is 0, cannot read\n\0".as_ptr() as PCHAR);
        *Count = 0;
        return EIO;
    }

    ArcDriveIndex = ((*Context).DriveNumber - FIRST_BIOS_DISK) as u32 ;
    if ArcDriveIndex >= ((PcBiosDiskCount) as u32) || InternalUefiDisk == core::ptr::null_mut() {
        ERR(b"Invalid drive number %d\n\0".as_ptr() as PCHAR, ((*Context).DriveNumber) as usize);
        *Count = 0;
        return EINVAL;
    }

    let Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
        (*InternalUefiDisk.add(ArcDriveIndex as usize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(BlockIo) as *mut PVOID,
    );

    if EFI_ERROR(Status) != 0 || BlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol\n\0".as_ptr() as PCHAR);
        *Count = 0;
        return EIO;
    }

    if UefiEnsureDiskReadBufferAligned((*(*BlockIo).Media).IoAlign) == 0 {
        ERR(b"Failed to align disk read buffer\n\0".as_ptr() as PCHAR);
        *Count = 0;
        return EIO;
    }

    ret = 1u8;

    while TotalSectors != 0 {
        ReadSectors = min(TotalSectors, MaxSectors);

        let Status = (*BlockIo).ReadBlocks.unwrap()(
            BlockIo,
            (*(*BlockIo).Media).MediaId,
            SectorOffset,
            (ReadSectors * (*Context).SectorSize) as UINTN,
            DiskReadBuffer as PVOID,
        );

        if EFI_ERROR(Status) != 0 {
            ERR(b"ReadBlocks failed: Status = 0x%lx\n\0".as_ptr() as PCHAR, (Status as ULONG) as usize);
            ret = 0u8;
            break;
        }

        Length = ReadSectors * (*Context).SectorSize;
        Length = min(Length, N_remaining);

        RtlCopyMemory(Ptr as PVOID, DiskReadBuffer as PVOID, Length as SIZE_T);

        Ptr = Ptr.add(Length as usize);
        N_remaining -= Length;
        SectorOffset += ReadSectors as ULONGLONG;
        TotalSectors -= ReadSectors;
    }

    *Count = (Ptr as ULONG_PTR - ((Buffer as ULONG_PTR as ULONG) as usize)) as u32 ;
    (*Context).SectorNumber = SectorOffset - (*Context).SectorOffset;

    if ret != 0 { ESUCCESS } else { EIO }
}

unsafe extern "C" fn UefiDiskSeek(FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: SEEKMODE) -> ARC_STATUS {
    
    let Context = FsGetDeviceSpecific(FileId) as *mut DISKCONTEXT;
    let mut NewPosition = *Position;

    match SeekMode {
        SeekAbsolute => {},
        SeekRelative => {
            NewPosition.QuadPart += (*Context).SectorNumber as i64 * (*Context).SectorSize as i64 ;
        },
        _ => {
            return EINVAL;
        }
    }

    if (NewPosition.QuadPart & ((*Context).SectorSize as i64 - 1)) != 0 {
        return EINVAL;
    }

    // Use unsigned arithmetic for the OOB check — C promotes mixed
    // signed/unsigned to unsigned. SectorCount sentinel u64::MAX (BIOS
    // "no LBA limit") would become -1 if cast to i64 and spuriously
    // fail every seek. See hwdisk.rs DiskSeek for the analogous fix.
    let new_sector = (NewPosition.QuadPart as u64) / (*Context).SectorSize as u64;

    if (*Context).SectorCount != 0 && new_sector >= (*Context).SectorCount {
        return EINVAL;
    }

    (*Context).SectorNumber = new_sector;
    ESUCCESS
}

unsafe fn GetHarddiskInformation(DriveNumber: UCHAR) {
    let Mbr: PMASTER_BOOT_RECORD;
    let Buffer: PULONG;
    let mut i: ULONG;
    let mut Checksum: ULONG;
    let Signature: ULONG;
    let ValidPartitionTable: BOOLEAN;
    let mut ArcName: [CHAR; 260] = [0; 260];
    let mut PartitionTableEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();
    let ArcDriveIndex: ULONG;
    let Identifier: PCHAR;

    ArcDriveIndex = DriveNumber.wrapping_sub(FIRST_BIOS_DISK) as ULONG;
    if ArcDriveIndex >= 32 {
        return;
    }

    Identifier = PcDiskIdentifier[ArcDriveIndex as usize].as_mut_ptr();

    DiskDetectPartitionType(DriveNumber);

    if MachDiskReadLogicalSectors(DriveNumber, 0u64, 1, DiskReadBuffer) == 0 {
        ERR(b"Reading MBR failed\n\0".as_ptr() as PCHAR);
        sprintf(Identifier, b"BIOSDISK%d\0".as_ptr() as PCHAR, ArcDriveIndex);
        return;
    }

    Buffer = DiskReadBuffer as PULONG;
    Mbr = DiskReadBuffer as PMASTER_BOOT_RECORD;

    Signature = (*Mbr).Signature;
    TRACE(b"Signature: %x\n\0".as_ptr() as PCHAR, (Signature) as usize);

    Checksum = 0;
    i = 0;
    while i < 512 / core::mem::size_of::<ULONG>() as ULONG {
        Checksum = Checksum.wrapping_add(*Buffer.add(i as usize));
        i += 1;
    }
    Checksum = (!Checksum).wrapping_add(1);
    TRACE(b"Checksum: %x\n\0".as_ptr() as PCHAR, (Checksum) as usize);

    ValidPartitionTable = if (*Mbr).MasterBootRecordMagic == 0xAA55 { 1 } else { 0 };

    // After the MBR fields: UefiReadGptHeader reuses DiskReadBuffer (upstream 7779fb44597).
    let mut GptHeader: GPT_TABLE_HEADER = core::mem::zeroed();
    let GptDiskGuid: *const c_void = if UefiReadGptHeader(DriveNumber, core::ptr::addr_of_mut!(GptHeader)) != 0 {
        core::ptr::addr_of!(GptHeader.DiskGuid) as *const c_void
    } else {
        core::ptr::null()
    };

    sprintf(ArcName.as_mut_ptr(), b"multi(0)disk(0)rdisk(%u)\0".as_ptr() as PCHAR, ArcDriveIndex);
    AddReactOSArcDiskInfo(ArcName.as_mut_ptr(), GptDiskGuid, Signature, Checksum, ValidPartitionTable);

    sprintf(ArcName.as_mut_ptr(), b"multi(0)disk(0)rdisk(%u)partition(0)\0".as_ptr() as PCHAR, ArcDriveIndex);
    FsRegisterDevice(ArcName.as_mut_ptr(), &UefiDiskVtbl);

    i = FIRST_PARTITION;
    DiskReportError(0u8);
    while DiskGetPartitionEntry(DriveNumber, i, &mut PartitionTableEntry) != 0 {
        if PartitionTableEntry.SystemIndicator != PARTITION_ENTRY_UNUSED {
            sprintf(ArcName.as_mut_ptr(), b"multi(0)disk(0)rdisk(%u)partition(%lu)\0".as_ptr() as PCHAR, ArcDriveIndex, i);
            FsRegisterDevice(ArcName.as_mut_ptr(), &UefiDiskVtbl);
        }
        i += 1;
    }
    DiskReportError(1u8);

    if ArcDriveIndex < ((PcBiosDiskCount) as u32) && InternalUefiDisk != core::ptr::null_mut() {
        (*InternalUefiDisk.add(ArcDriveIndex as usize)).NumOfPartitions = (i) as u8 ;
    }

    *Identifier.add(0) = (Hex[((Checksum >> 28) & 0x0F) as usize]) as i8 ;
    *Identifier.add(1) = (Hex[((Checksum >> 24) & 0x0F) as usize]) as i8 ;
    *Identifier.add(2) = (Hex[((Checksum >> 20) & 0x0F) as usize]) as i8 ;
    *Identifier.add(3) = (Hex[((Checksum >> 16) & 0x0F) as usize]) as i8 ;
    *Identifier.add(4) = (Hex[((Checksum >> 12) & 0x0F) as usize]) as i8 ;
    *Identifier.add(5) = (Hex[((Checksum >> 8) & 0x0F) as usize]) as i8 ;
    *Identifier.add(6) = (Hex[((Checksum >> 4) & 0x0F) as usize]) as i8 ;
    *Identifier.add(7) = (Hex[(Checksum & 0x0F) as usize]) as i8 ;
    *Identifier.add(8) = b'-' as CHAR;
    *Identifier.add(9) = (Hex[((Signature >> 28) & 0x0F) as usize]) as i8 ;
    *Identifier.add(10) = (Hex[((Signature >> 24) & 0x0F) as usize]) as i8 ;
    *Identifier.add(11) = (Hex[((Signature >> 20) & 0x0F) as usize]) as i8 ;
    *Identifier.add(12) = (Hex[((Signature >> 16) & 0x0F) as usize]) as i8 ;
    *Identifier.add(13) = (Hex[((Signature >> 12) & 0x0F) as usize]) as i8 ;
    *Identifier.add(14) = (Hex[((Signature >> 8) & 0x0F) as usize]) as i8 ;
    *Identifier.add(15) = (Hex[((Signature >> 4) & 0x0F) as usize]) as i8 ;
    *Identifier.add(16) = (Hex[(Signature & 0x0F) as usize]) as i8 ;
    *Identifier.add(17) = b'-' as CHAR;
    *Identifier.add(18) = if ValidPartitionTable != 0 { b'A' as CHAR } else { b'X' as CHAR };
    *Identifier.add(19) = 0;
    TRACE(b"Identifier: %s\n\0".as_ptr() as PCHAR, (Identifier) as usize);
}

unsafe fn UefiSetupBlockDevices() {
    let mut BlockDeviceIndex: ULONG;
    let SystemHandleCount: ULONG;
    let mut Status: EFI_STATUS;
    let mut i: ULONG;
    let mut HandleSize: UINTN = 0;
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();

    PcBiosDiskCount = 0;
    UefiBootRootIndex = 0;

    Status = (*(*GlobalSystemTable).BootServices).LocateHandle.unwrap()(
        ByProtocol,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(HandleSize),
        core::ptr::null_mut(),
    );

    if HandleSize == 0 {
        ERR(b"Failed to get handle buffer size: Status = 0x%lx\n\0".as_ptr() as PCHAR, (Status as ULONG) as usize);
        return;
    }

    SystemHandleCount = (HandleSize / ((core::mem::size_of::<EFI_HANDLE>()) as UINTN)) as ULONG;
    if SystemHandleCount == 0 {
        ERR(b"No block devices found\n\0".as_ptr() as PCHAR);
        return;
    }

    handles = MmAllocateMemoryWithType((HandleSize) as usize , LoaderFirmwareTemporary) as *mut EFI_HANDLE;
    if handles == core::ptr::null_mut() {
        ERR(b"Failed to allocate memory for handles\n\0".as_ptr() as PCHAR);
        return;
    }

    Status = (*(*GlobalSystemTable).BootServices).LocateHandle.unwrap()(
        ByProtocol,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(HandleSize),
        handles,
    );

    if EFI_ERROR(Status) != 0 {
        ERR(b"Failed to locate block device handles: Status = 0x%lx\n\0".as_ptr() as PCHAR, (Status as ULONG) as usize);
        return;
    }

    HandleCount = SystemHandleCount;

    InternalUefiDisk = MmAllocateMemoryWithType(
        core::mem::size_of::<INTERNAL_UEFI_DISK>() * ((SystemHandleCount) as usize),
        LoaderFirmwareTemporary,
    ) as *mut INTERNAL_UEFI_DISK;

    if InternalUefiDisk == core::ptr::null_mut() {
        ERR(b"Failed to allocate memory for internal disk structure\n\0".as_ptr() as PCHAR);
        return;
    }

    RtlZeroMemory(InternalUefiDisk as *mut VOID, core::mem::size_of::<INTERNAL_UEFI_DISK>() * ((SystemHandleCount) as usize));

    UefiBootRootIndex = 0;
    i = 0;
    while i < SystemHandleCount {
        if *handles.offset(i as isize) == PublicBootHandle {
            UefiBootRootIndex = i;
            TRACE(b"Found boot handle at index %lu\n\0".as_ptr() as PCHAR, (i) as usize);

            Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
                *handles.offset(i as isize),
                core::ptr::addr_of_mut!(BlockIoGuid),
                core::ptr::addr_of_mut!(BlockIo) as *mut *mut VOID,
            );

            if EFI_ERROR(Status) == 0 && BlockIo != core::ptr::null_mut() {
                TRACE(
                    b"Boot handle: LogicalPartition=%s, RemovableMedia=%s, BlockSize=%lu\n\0".as_ptr() as PCHAR, (if (*(*BlockIo).Media).LogicalPartition != 0 { b"TRUE\0".as_ptr() as PCHAR } else { b"FALSE\0".as_ptr() as PCHAR }) as usize, (if (*(*BlockIo).Media).RemovableMedia != 0 { b"TRUE\0".as_ptr() as PCHAR } else { b"FALSE\0".as_ptr() as PCHAR }) as usize, ((*(*BlockIo).Media).BlockSize) as usize,
                );
            }
            break;
        }
        i = i.wrapping_add(1);
    }

    BlockDeviceIndex = 0;
    i = 0;
    while i < SystemHandleCount {
        Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
            *handles.offset(i as isize),
            core::ptr::addr_of_mut!(BlockIoGuid),
            core::ptr::addr_of_mut!(BlockIo) as *mut *mut VOID,
        );

        if EFI_ERROR(Status) != 0 {
            TRACE(b"HandleProtocol failed for handle %lu: Status = 0x%lx\n\0".as_ptr() as PCHAR, (i) as usize, (Status as ULONG) as usize);
            i = i.wrapping_add(1);
            continue;
        }

        if BlockIo == core::ptr::null_mut() {
            TRACE(b"BlockIo is NULL for handle %lu\n\0".as_ptr() as PCHAR, (i) as usize);
            i = i.wrapping_add(1);
            continue;
        }

        if (*(*BlockIo).Media).MediaPresent == 0 {
            TRACE(b"Media not present for handle %lu\n\0".as_ptr() as PCHAR, (i) as usize);
            i = i.wrapping_add(1);
            continue;
        }

        if (*(*BlockIo).Media).BlockSize == 0 {
            TRACE(b"Invalid block size (0) for handle %lu\n\0".as_ptr() as PCHAR, (i) as usize);
            i = i.wrapping_add(1);
            continue;
        }

        if (*(*BlockIo).Media).BlockSize > MAX_SUPPORTED_BLOCK_SIZE {
            TRACE(
                b"Block size too large (%lu) for handle %lu, skipping\n\0".as_ptr() as PCHAR, ((*(*BlockIo).Media).BlockSize) as usize, (i) as usize,
            );
            i = i.wrapping_add(1);
            continue;
        }

        if (*(*BlockIo).Media).LogicalPartition != 0 {
            TRACE(b"Skipping logical partition handle %lu\n\0".as_ptr() as PCHAR, (i) as usize);
            i = i.wrapping_add(1);
            continue;
        }

        TRACE(
            b"Found root block device at index %lu: BlockSize=%lu, LastBlock=%llu\n\0".as_ptr() as PCHAR, (i) as usize, ((*(*BlockIo).Media).BlockSize) as usize, ((*(*BlockIo).Media).LastBlock) as usize,
        );

        (*InternalUefiDisk.offset(BlockDeviceIndex as isize)).ArcDriveNumber = (BlockDeviceIndex) as u8 ;
        (*InternalUefiDisk.offset(BlockDeviceIndex as isize)).UefiHandleIndex = i;
        (*InternalUefiDisk.offset(BlockDeviceIndex as isize)).Handle = *handles.offset(i as isize);
        (*InternalUefiDisk.offset(BlockDeviceIndex as isize)).IsThisTheBootDrive = 0u8;

        if i == UefiBootRootIndex {
            (*InternalUefiDisk.offset(BlockDeviceIndex as isize)).IsThisTheBootDrive = 1u8;
            PublicBootArcDisk = BlockDeviceIndex;
            TRACE(b"Boot device is at ARC drive index %lu (root device)\n\0".as_ptr() as PCHAR, (BlockDeviceIndex) as usize);
        }

        PcBiosDiskCount = (BlockDeviceIndex.wrapping_add(1)) as u8 ;

        TRACE(
            b"Calling GetHarddiskInformation for drive %d (BlockDeviceIndex=%lu)\n\0".as_ptr() as PCHAR, (BlockDeviceIndex.wrapping_add(FIRST_BIOS_DISK as ULONG)) as usize, (BlockDeviceIndex) as usize,
        );
        if UefiEnsureDiskReadBufferAligned((*(*BlockIo).Media).IoAlign) == 0 {
            ERR(b"Failed to align disk read buffer for drive %d\n\0".as_ptr() as PCHAR, (BlockDeviceIndex.wrapping_add(FIRST_BIOS_DISK as ULONG)) as usize);
            return;
        }

        GetHarddiskInformation(BlockDeviceIndex.wrapping_add(FIRST_BIOS_DISK as ULONG) as UCHAR);
        BlockDeviceIndex = BlockDeviceIndex.wrapping_add(1);
        i = i.wrapping_add(1);
    }

    if UefiBootRootIndex < HandleCount {
        Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
            *handles.offset(UefiBootRootIndex as isize),
            core::ptr::addr_of_mut!(BlockIoGuid),
            core::ptr::addr_of_mut!(BlockIo) as *mut *mut VOID,
        );

        if EFI_ERROR(Status) == 0 && BlockIo != core::ptr::null_mut() && (*(*BlockIo).Media).LogicalPartition != 0 {
            TRACE(b"Boot handle is a logical partition, searching for parent root device\n\0".as_ptr() as PCHAR);
            TRACE(
                b"Boot partition: BlockSize=%lu, RemovableMedia=%s\n\0".as_ptr() as PCHAR, ((*(*BlockIo).Media).BlockSize) as usize, (if (*(*BlockIo).Media).RemovableMedia != 0 { b"TRUE\0".as_ptr() as PCHAR } else { b"FALSE\0".as_ptr() as PCHAR }) as usize,
            );

            let mut FoundBootDevice: BOOLEAN = 0u8;
            i = 0;
            while i < BlockDeviceIndex {
                let mut RootBlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
                Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
                    (*InternalUefiDisk.offset(i as isize)).Handle,
                    core::ptr::addr_of_mut!(BlockIoGuid),
                    core::ptr::addr_of_mut!(RootBlockIo) as *mut *mut VOID,
                );

                if EFI_ERROR(Status) != 0 || RootBlockIo == core::ptr::null_mut() {
                    i = i.wrapping_add(1);
                    continue;
                }

                if (*(*BlockIo).Media).BlockSize == 2048 && (*(*BlockIo).Media).RemovableMedia != 0 {
                    if (*(*RootBlockIo).Media).BlockSize == 2048
                        && (*(*RootBlockIo).Media).RemovableMedia != 0
                        && (*(*RootBlockIo).Media).LogicalPartition == 0
                    {
                        PublicBootArcDisk = i;
                        (*InternalUefiDisk.offset(i as isize)).IsThisTheBootDrive = 1u8;
                        FoundBootDevice = 1u8;
                        TRACE(b"Found CD-ROM boot device at ARC drive index %lu\n\0".as_ptr() as PCHAR, (i) as usize);
                        break;
                    }
                } else if (*InternalUefiDisk.offset(i as isize)).UefiHandleIndex < UefiBootRootIndex {
                    if (*(*RootBlockIo).Media).BlockSize == (*(*BlockIo).Media).BlockSize
                        && (*(*RootBlockIo).Media).LogicalPartition == 0
                    {
                        PublicBootArcDisk = i;
                        (*InternalUefiDisk.offset(i as isize)).IsThisTheBootDrive = 1u8;
                        FoundBootDevice = 1u8;
                        TRACE(b"Found potential hard disk boot device at ARC drive index %lu\n\0".as_ptr() as PCHAR, (i) as usize);
                    }
                }
                i = i.wrapping_add(1);
            }

            if FoundBootDevice == 0 && PcBiosDiskCount > 0 {
                PublicBootArcDisk = 0;
                (*InternalUefiDisk.offset(0)).IsThisTheBootDrive = 1u8;
                TRACE(b"Could not determine boot device, assuming first drive\n\0".as_ptr() as PCHAR);
            }
        }
    }

    TRACE(b"Found %lu root block devices\n\0".as_ptr() as PCHAR, (PcBiosDiskCount) as usize);
}

unsafe fn UefiSetBootpath() -> BOOLEAN {
    let mut BootBlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let mut RootBlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let mut Status: EFI_STATUS;
    let ArcDriveIndex: ULONG;

    TRACE(b"UefiSetBootpath: Setting up boot path\n\0".as_ptr() as PCHAR);

    if UefiBootRootIndex >= HandleCount || handles == core::ptr::null_mut() {
        ERR(b"Invalid boot root index\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    ArcDriveIndex = PublicBootArcDisk;
    if ArcDriveIndex >= ((PcBiosDiskCount) as u32) || InternalUefiDisk == core::ptr::null_mut() {
        ERR(b"Invalid boot arc disk index\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
        *handles.offset(UefiBootRootIndex as isize),
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(BootBlockIo) as *mut *mut core::ffi::c_void,
    );

    if EFI_ERROR(Status) != 0 || BootBlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol for boot handle\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(RootBlockIo) as *mut *mut core::ffi::c_void,
    );

    if EFI_ERROR(Status) != 0 || RootBlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol for boot root device\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    FrldrBootDrive = (FIRST_BIOS_DISK + ((ArcDriveIndex) as u8)) as UCHAR;

    if (*(*BootBlockIo).Media).RemovableMedia == 1u8 && (*(*BootBlockIo).Media).BlockSize == 2048 {
        FrldrBootPartition = 0xFF;
        RtlStringCbPrintfA(
            FrLdrBootPath.as_mut_ptr() as PCHAR,
            core::mem::size_of_val(&FrLdrBootPath),
            b"multi(0)disk(0)cdrom(%u)\0".as_ptr() as PCHAR,
            ArcDriveIndex,
        );
        TRACE(b"Boot path set to CD-ROM: %s\n\0".as_ptr() as PCHAR, (FrLdrBootPath.as_ptr()) as usize);
        // Which ARC branch the boot handle took -- one line, and it is the first
        // thing worth knowing when a boot device behaves unexpectedly. The CD arm
        // needs RemovableMedia && BlockSize == 2048, which is a DEVICE-MODEL test
        // standing in for a filesystem question: an ISO served over usb-storage or
        // virtio-blk reports 512-byte blocks, fails this, and takes the hard-disk
        // arm, where RunLoader then finds no boot configuration.
        crate::uefildr::fldr_conout(crate::wide!("[FLDR] arc branch = CDROM\r\n\0"));
    } else {
        let mut BootPartition: ULONG = 0;
        let mut PartitionEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();

        if (*(*BootBlockIo).Media).LogicalPartition != 0 {
            // Take the number from the handle's device path (upstream 7eab8fafec4): it used to
            // be partition 1 unconditionally, so a disk/USB boot from any other partition read
            // freeldr.ini and the system from the wrong volume. Partition 1 stays the fallback.
            // Walked byte-wise; freeldr-rust has no device-path types. A node is
            // { u8 Type, u8 SubType, u16 Length }, and a HARDDRIVE_DEVICE_PATH (MEDIA_DEVICE_PATH
            // 0x04 / MEDIA_HARDDRIVE_DP 0x01) has its UINT32 PartitionNumber right after that.
            BootPartition = FIRST_PARTITION;
            let mut DevicePathGuid: EFI_GUID = EFI_DEVICE_PATH_PROTOCOL_GUID;
            let mut Node: *mut u8 = core::ptr::null_mut();
            if let Some(HandleProtocol) = (*(*GlobalSystemTable).BootServices).HandleProtocol {
                Status = HandleProtocol(
                    *handles.offset(UefiBootRootIndex as isize),
                    core::ptr::addr_of_mut!(DevicePathGuid),
                    core::ptr::addr_of_mut!(Node) as *mut *mut core::ffi::c_void,
                );
                if EFI_ERROR(Status) != 0 {
                    Node = core::ptr::null_mut();
                }
            }
            if !Node.is_null() {
                loop {
                    let (Type, SubType) = (*Node, *Node.add(1));
                    let Length = u16::from_le_bytes([*Node.add(2), *Node.add(3)]) as usize;
                    if Type == 0x7F && SubType == 0xFF {
                        break; // END_ENTIRE_DEVICE_PATH
                    }
                    if Length < 4 {
                        break; // malformed
                    }
                    if Type == 0x04 && SubType == 0x01 {
                        BootPartition = core::ptr::read_unaligned(Node.add(4) as *const u32);
                        break;
                    }
                    Node = Node.add(Length);
                }
            }
            TRACE(
                b"Boot handle is logical partition, using partition %lu\n\0".as_ptr() as PCHAR, (BootPartition) as usize,
            );
        } else {
            if UefiGetBootPartitionEntry(
                FrldrBootDrive,
                core::ptr::addr_of_mut!(PartitionEntry),
                core::ptr::addr_of_mut!(BootPartition),
            ) == 0
            {
                ERR(b"Failed to get boot partition entry\n\0".as_ptr() as PCHAR);
                crate::uefildr::fldr_conout(
                    crate::wide!("[FLDR] UefiGetBootPartitionEntry FAILED\r\n\0"),
                );
                return 0u8;
            }
        }

        RtlStringCbPrintfA(
            FrLdrBootPath.as_mut_ptr() as PCHAR,
            core::mem::size_of_val(&FrLdrBootPath),
            b"multi(0)disk(0)rdisk(%u)partition(%lu)\0".as_ptr() as PCHAR,
            ArcDriveIndex,
            BootPartition,
        );
        TRACE(
            b"Boot path set to hard disk: %s\n\0".as_ptr() as PCHAR, (FrLdrBootPath.as_ptr()) as usize,
        );
        crate::uefildr::fldr_conout(
            crate::wide!("[FLDR] arc branch = HARDDISK+partition\r\n\0"),
        );
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn UefiInitializeBootDevices() -> BOOLEAN {
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let Status: EFI_STATUS;
    let ArcDriveIndex: ULONG;
    let Mbr: PMASTER_BOOT_RECORD;
    let Buffer: PULONG;
    let mut Checksum: ULONG = 0;
    let Signature: ULONG;
    let mut i: ULONG;

    // 64KB, not EFI_PAGE_SIZE: every ReadBlocks call is a full firmware ATAPI
    // round trip (expensive under TCG). 4KB chunks meant ~9000 calls for the
    // ~35MB boot set — the per-call overhead capped mid-size files at ~4MB/s
    // (perf doc: freeldr load-phase decomposition). 64KB amortizes it; 1MB
    // hangs OVMF's ATAPI path at the first read (bisected 2026-07-06).
    DiskReadBufferSize = 64 * 1024;
    DiskReadBuffer = core::ptr::null_mut();
    DiskReadBufferRaw = core::ptr::null_mut();
    DiskReadBufferAlignment = 1;
    DiskReadBufferFromPool = 0u8;
    if UefiEnsureDiskReadBufferAligned(1) == 0 {
        ERR(b"Failed to allocate disk read buffer\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    UefiSetupBlockDevices();

    if PcBiosDiskCount == 0 {
        ERR(b"No block devices found\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    if UefiSetBootpath() == 0 {
        ERR(b"Failed to set boot path\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    ArcDriveIndex = PublicBootArcDisk;
    if ArcDriveIndex >= ((PcBiosDiskCount) as u32) || InternalUefiDisk == core::ptr::null_mut() {
        ERR(b"Invalid boot arc disk index\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(BlockIo) as *mut PVOID,
    );

    if EFI_ERROR(Status) != 0 || BlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    if (*(*BlockIo).Media).RemovableMedia == 1u8 && (*(*BlockIo).Media).BlockSize == 2048 {
        if MachDiskReadLogicalSectors(FrldrBootDrive, 16u64, 1, DiskReadBuffer) == 0 {
            ERR(b"Reading MBR from CD-ROM failed\n\0".as_ptr() as PCHAR);
            return 0u8;
        }

        Buffer = DiskReadBuffer as PULONG;
        Mbr = DiskReadBuffer as PMASTER_BOOT_RECORD;

        Signature = (*Mbr).Signature;
        TRACE(b"CD-ROM Signature: %x\n\0".as_ptr() as PCHAR, (Signature) as usize);

        i = 0;
        while i < 2048 / core::mem::size_of::<ULONG>() as u32 {
            Checksum = Checksum.wrapping_add(*Buffer.offset(i as isize));
            i += 1;
        }
        Checksum = (!Checksum).wrapping_add(1);
        TRACE(b"CD-ROM Checksum: %x\n\0".as_ptr() as PCHAR, (Checksum) as usize);

        AddReactOSArcDiskInfo(FrLdrBootPath.as_mut_ptr(), core::ptr::null(), Signature, Checksum, 1u8);

        FsRegisterDevice(FrLdrBootPath.as_ptr(), core::ptr::addr_of!(UefiDiskVtbl));
        TRACE(
            b"Registered CD-ROM boot device: 0x%02X\n\0".as_ptr() as PCHAR, (FrldrBootDrive as i32) as usize,
        );
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn UefiGetFloppyCount() -> u8 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn UefiDiskReadLogicalSectors(
    DriveNumber: UCHAR,
    SectorNumber: ULONGLONG,
    SectorCount: ULONG,
    Buffer: PVOID,
) -> BOOLEAN {
    let ArcDriveIndex: ULONG;
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let mut Status: EFI_STATUS;
    let BlockSize: ULONG;
    let IoAlign: ULONG;

    if DriveNumber < FIRST_BIOS_DISK {
        return 0u8;
    }

    ArcDriveIndex = (DriveNumber - FIRST_BIOS_DISK) as ULONG;

    if InternalUefiDisk == core::ptr::null_mut() {
        ERR(b"InternalUefiDisk not initialized\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    if ArcDriveIndex >= 32 {
        ERR(b"Drive index out of bounds: %d (ArcDriveIndex=%lu)\n\0".as_ptr() as PCHAR, (DriveNumber) as usize, (ArcDriveIndex) as usize);
        return 0u8;
    }

    if (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle == core::ptr::null_mut() {
        ERR(b"Invalid drive number: %d (ArcDriveIndex=%lu, PcBiosDiskCount=%lu, Handle=NULL)\n\0".as_ptr() as PCHAR, (DriveNumber) as usize, (ArcDriveIndex) as usize, (PcBiosDiskCount) as usize);
        return 0u8;
    }

    Status = (*(*GlobalSystemTable).BootServices).HandleProtocol.unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(BlockIo) as *mut PVOID,
    );

    if EFI_ERROR(Status) != 0 || BlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0u8;
    }

    if (*(*BlockIo).Media).MediaPresent == 0 {
        ERR(b"Media not present for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0u8;
    }

    BlockSize = (*(*BlockIo).Media).BlockSize;
    IoAlign = (*(*BlockIo).Media).IoAlign;

    if UefiEnsureDiskReadBufferAligned(IoAlign) == 0 {
        ERR(b"Failed to align disk read buffer for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0u8;
    }

    if UefiIsAlignedPointer(Buffer, if IoAlign == 0 { 1 } else { IoAlign }) == 0 {
        let mut TotalSectors: ULONG = SectorCount;
        let MaxSectors: ULONG = (DiskReadBufferSize / ((BlockSize) as usize)) as u32 ;
        let mut CurrentSector: ULONGLONG = SectorNumber;
        let mut OutPtr: PUCHAR = Buffer as PUCHAR;

        if MaxSectors == 0 {
            ERR(b"DiskReadBufferSize too small for block size %lu\n\0".as_ptr() as PCHAR, (BlockSize) as usize);
            return 0u8;
        }

        while TotalSectors != 0 {
            let ReadSectors: ULONG = min(TotalSectors, MaxSectors);
            let ReadSize: UINTN = (ReadSectors as UINTN) * (BlockSize as UINTN);

            Status = (*BlockIo).ReadBlocks.unwrap()(
                BlockIo,
                (*(*BlockIo).Media).MediaId,
                CurrentSector,
                ReadSize,
                DiskReadBuffer,
            );

            if EFI_ERROR(Status) != 0 {
                ERR(b"ReadBlocks failed: DriveNumber=%d, SectorNumber=%llu, SectorCount=%lu, Status=0x%lx\n\0".as_ptr() as PCHAR, (DriveNumber) as usize, (CurrentSector) as usize, (ReadSectors) as usize, (Status as ULONG) as usize);
                ERR(b"ReadBlocks details: BlockSize=%lu, IoAlign=%lu, Buffer=%p, DiskReadBuffer=%p, MediaId=0x%lx\n\0".as_ptr() as PCHAR, (BlockSize) as usize, (IoAlign) as usize, (Buffer) as usize, (DiskReadBuffer) as usize, ((*(*BlockIo).Media).MediaId as ULONG) as usize);
                ERR(b"ReadBlocks media: LastBlock=%llu, LogicalPartition=%s, RemovableMedia=%s\n\0".as_ptr() as PCHAR, ((*(*BlockIo).Media).LastBlock) as usize, (if (*(*BlockIo).Media).LogicalPartition != 0 { b"TRUE\0".as_ptr() as PCHAR } else { b"FALSE\0".as_ptr() as PCHAR }) as usize, (if (*(*BlockIo).Media).RemovableMedia != 0 { b"TRUE\0".as_ptr() as PCHAR } else { b"FALSE\0".as_ptr() as PCHAR }) as usize);
                return 0u8;
            }

            RtlCopyMemory(OutPtr as PVOID, DiskReadBuffer, (ReadSize) as usize );
            OutPtr = OutPtr.offset(ReadSize as isize);
            CurrentSector += ReadSectors as ULONGLONG;
            TotalSectors -= ReadSectors;
        }

        return 1u8;
    }

    Status = (*BlockIo).ReadBlocks.unwrap()(
        BlockIo,
        (*(*BlockIo).Media).MediaId,
        SectorNumber,
        (SectorCount as UINTN) * (BlockSize as UINTN),
        Buffer,
    );

    if EFI_ERROR(Status) != 0 {
        ERR(b"ReadBlocks failed: DriveNumber=%d, SectorNumber=%llu, SectorCount=%lu, Status=0x%lx\n\0".as_ptr() as PCHAR, (DriveNumber) as usize, (SectorNumber) as usize, (SectorCount) as usize, (Status as ULONG) as usize);
        ERR(b"ReadBlocks details: BlockSize=%lu, IoAlign=%lu, Buffer=%p, DiskReadBuffer=%p, MediaId=0x%lx\n\0".as_ptr() as PCHAR, (BlockSize) as usize, (IoAlign) as usize, (Buffer) as usize, (DiskReadBuffer) as usize, ((*(*BlockIo).Media).MediaId as ULONG) as usize);
        ERR(b"ReadBlocks media: LastBlock=%llu, LogicalPartition=%s, RemovableMedia=%s\n\0".as_ptr() as PCHAR, ((*(*BlockIo).Media).LastBlock) as usize, (if (*(*BlockIo).Media).LogicalPartition != 0 { b"TRUE\0".as_ptr() as PCHAR } else { b"FALSE\0".as_ptr() as PCHAR }) as usize, (if (*(*BlockIo).Media).RemovableMedia != 0 { b"TRUE\0".as_ptr() as PCHAR } else { b"FALSE\0".as_ptr() as PCHAR }) as usize);
        return 0u8;
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn UefiDiskGetDriveGeometry(DriveNumber: UCHAR, Geometry: PGEOMETRY) -> BOOLEAN {
    let ArcDriveIndex: ULONG;
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let Status: EFI_STATUS;

    if DriveNumber < FIRST_BIOS_DISK {
        return 0u8;
    }

    ArcDriveIndex = (DriveNumber - FIRST_BIOS_DISK) as u32 ;

    if InternalUefiDisk == core::ptr::null_mut() {
        ERR(b"InternalUefiDisk not initialized\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    if ArcDriveIndex >= 32 || (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle == core::ptr::null_mut() {
        ERR(b"Invalid drive number: %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0u8;
    }

    Status = ((*(*GlobalSystemTable).BootServices).HandleProtocol).unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(BlockIo) as *mut *mut core::ffi::c_void,
    );

    if EFI_ERROR(Status) != 0 || BlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0u8;
    }

    if (*(*BlockIo).Media).MediaPresent == 0 {
        ERR(b"Media not present for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0u8;
    }

    (*Geometry).Cylinders = 1;
    (*Geometry).Heads = 1;
    (*Geometry).SectorsPerTrack = ((*(*BlockIo).Media).LastBlock + 1) as ULONG;
    (*Geometry).BytesPerSector = (*(*BlockIo).Media).BlockSize;
    (*Geometry).Sectors = (*(*BlockIo).Media).LastBlock + 1;

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn UefiDiskGetCacheableBlockCount(DriveNumber: UCHAR) -> ULONG {
    let ArcDriveIndex: ULONG;
    let mut BlockIo: *mut EFI_BLOCK_IO = core::ptr::null_mut();
    let Status: EFI_STATUS;

    if DriveNumber < FIRST_BIOS_DISK {
        return 0;
    }

    ArcDriveIndex = DriveNumber as ULONG - FIRST_BIOS_DISK as ULONG;

    if InternalUefiDisk == core::ptr::null_mut() {
        ERR(b"InternalUefiDisk not initialized\n\0".as_ptr() as PCHAR);
        return 0;
    }

    if ArcDriveIndex >= 32 || (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle == core::ptr::null_mut() {
        ERR(b"Invalid drive number: %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0;
    }

    Status = ((*(*GlobalSystemTable).BootServices).HandleProtocol).unwrap()(
        (*InternalUefiDisk.offset(ArcDriveIndex as isize)).Handle,
        core::ptr::addr_of_mut!(BlockIoGuid),
        core::ptr::addr_of_mut!(BlockIo) as *mut PVOID,
    );

    if EFI_ERROR(Status) != 0 || BlockIo == core::ptr::null_mut() {
        ERR(b"Failed to get Block I/O protocol for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0;
    }

    if (*(*BlockIo).Media).MediaPresent == 0 {
        ERR(b"Media not present for drive %d\n\0".as_ptr() as PCHAR, (DriveNumber) as usize);
        return 0;
    }

    ((*(*BlockIo).Media).LastBlock + 1) as ULONG
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Disk Access Functions
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <uefildr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WARNING);
//
// #define TAG_HW_RESOURCE_LIST    'lRwH'
// #define TAG_HW_DISK_CONTEXT     'cDwH'
// #define FIRST_BIOS_DISK 0x80
// #define FIRST_PARTITION 1
//
// /* Maximum block size we support (8KB) - filters out flash devices */
// #define MAX_SUPPORTED_BLOCK_SIZE 8192
//
// /* GPT (GUID Partition Table) definitions */
// #define EFI_PARTITION_HEADER_SIGNATURE    "EFI PART"
// #define EFI_HEADER_LOCATION               1ULL
// #define EFI_TABLE_REVISION                0x00010000
// #define EFI_PARTITION_ENTRIES_BLOCK       2ULL
// #define EFI_PARTITION_ENTRY_COUNT          128
// #define EFI_PARTITION_ENTRY_SIZE          128
// #define EFI_PARTITION_NAME_LENGTH         36
//
// /* GPT Partition Type GUIDs */
// #define EFI_PART_TYPE_UNUSED_GUID \
//     {0x00000000, 0x0000, 0x0000, {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}}
//
// #define EFI_PART_TYPE_EFI_SYSTEM_PART_GUID \
//     {0xc12a7328, 0xf81f, 0x11d2, {0xba, 0x4b, 0x00, 0xa0, 0xc9, 0x3e, 0xc9, 0x3b}}
//
// #include <pshpack1.h>
//
// /* GPT Table Header */
// typedef struct _GPT_TABLE_HEADER
// {
//     CHAR8   Signature[8];              /* "EFI PART" */
//     UINT32  Revision;                   /* 0x00010000 */
//     UINT32  HeaderSize;                 /* Size of header (usually 92) */
//     UINT32  HeaderCrc32;                /* CRC32 of header */
//     UINT32  Reserved;                   /* Must be 0 */
//     UINT64  MyLba;                      /* LBA of this header */
//     UINT64  AlternateLba;               /* LBA of alternate header */
//     UINT64  FirstUsableLba;             /* First usable LBA for partitions */
//     UINT64  LastUsableLba;              /* Last usable LBA for partitions */
//     EFI_GUID DiskGuid;                  /* Disk GUID */
//     UINT64  PartitionEntryLba;          /* LBA of partition entries array */
//     UINT32  NumberOfPartitionEntries;   /* Number of partition entries */
//     UINT32  SizeOfPartitionEntry;       /* Size of each entry (usually 128) */
//     UINT32  PartitionEntryArrayCrc32;   /* CRC32 of partition entries array */
// } GPT_TABLE_HEADER, *PGPT_TABLE_HEADER;
//
// /* GPT Partition Entry */
// typedef struct _GPT_PARTITION_ENTRY
// {
//     EFI_GUID PartitionTypeGuid;         /* Partition type GUID */
//     EFI_GUID UniquePartitionGuid;       /* Unique partition GUID */
//     UINT64  StartingLba;                /* Starting LBA */
//     UINT64  EndingLba;                  /* Ending LBA */
//     UINT64  Attributes;                 /* Partition attributes */
//     CHAR16  PartitionName[EFI_PARTITION_NAME_LENGTH]; /* Partition name (UTF-16) */
// } GPT_PARTITION_ENTRY, *PGPT_PARTITION_ENTRY;
//
// #include <poppack.h>
//
// typedef struct tagDISKCONTEXT
// {
//     UCHAR DriveNumber;
//     ULONG SectorSize;
//     ULONGLONG SectorOffset;
//     ULONGLONG SectorCount;
//     ULONGLONG SectorNumber;
// } DISKCONTEXT;
//
// typedef struct _INTERNAL_UEFI_DISK
// {
//     UCHAR ArcDriveNumber;
//     UCHAR NumOfPartitions;
//     ULONG UefiHandleIndex;
//     BOOLEAN IsThisTheBootDrive;
//     EFI_HANDLE Handle;
// } INTERNAL_UEFI_DISK, *PINTERNAL_UEFI_DISK;
//
// /* GLOBALS *******************************************************************/
//
// extern EFI_SYSTEM_TABLE* GlobalSystemTable;
// extern EFI_HANDLE GlobalImageHandle;
// extern EFI_HANDLE PublicBootHandle; /* Freeldr itself */
//
// /* Made to match BIOS */
// PVOID DiskReadBuffer;
// static PVOID DiskReadBufferRaw;
// static ULONG DiskReadBufferAlignment;
// static BOOLEAN DiskReadBufferFromPool;
// UCHAR PcBiosDiskCount;
//
// UCHAR FrldrBootDrive;
// ULONG FrldrBootPartition;
// SIZE_T DiskReadBufferSize;
// PVOID Buffer;
//
// static const CHAR Hex[] = "0123456789abcdef";
// static CHAR PcDiskIdentifier[32][20];
//
// /* UEFI-specific */
// static ULONG UefiBootRootIndex = 0;
// static ULONG PublicBootArcDisk = 0;
// static INTERNAL_UEFI_DISK* InternalUefiDisk = NULL;
// static EFI_GUID BlockIoGuid = BLOCK_IO_PROTOCOL;
// static EFI_HANDLE* handles = NULL;
// static ULONG HandleCount = 0;
//
// static
// BOOLEAN
// UefiIsAlignedPointer(
//     IN PVOID Pointer,
//     IN ULONG Alignment)
// {
//     ULONG_PTR Value;
//
//     if (Alignment <= 1)
//         return TRUE;
//
//     Value = (ULONG_PTR)Pointer;
//     return ((Value & (Alignment - 1)) == 0);
// }
//
// static
// BOOLEAN
// UefiEnsureDiskReadBufferAligned(
//     IN ULONG Alignment)
// {
//     ULONG RequiredAlignment = (Alignment == 0) ? 1 : Alignment;
//     EFI_STATUS Status;
//
//     if (DiskReadBuffer != NULL && DiskReadBufferAlignment >= RequiredAlignment &&
//         UefiIsAlignedPointer(DiskReadBuffer, RequiredAlignment))
//     {
//         return TRUE;
//     }
//
//     DiskReadBufferAlignment = RequiredAlignment;
//
//     if (DiskReadBufferRaw != NULL && DiskReadBufferFromPool)
//     {
//         GlobalSystemTable->BootServices->FreePool(DiskReadBufferRaw);
//         DiskReadBufferRaw = NULL;
//         DiskReadBuffer = NULL;
//         DiskReadBufferFromPool = FALSE;
//     }
//
//     Status = GlobalSystemTable->BootServices->AllocatePool(EfiLoaderData,
//                                                            DiskReadBufferSize + RequiredAlignment,
//                                                            (void**)&DiskReadBufferRaw);
//     if (EFI_ERROR(Status) || DiskReadBufferRaw == NULL)
//     {
//         ERR("Failed to allocate aligned disk read buffer (align %lu)\n", (RequiredAlignment) as usize);
//         DiskReadBuffer = NULL;
//         return FALSE;
//     }
//     DiskReadBufferFromPool = TRUE;
//
//     DiskReadBuffer = (PVOID)ALIGN_UP_POINTER_BY(DiskReadBufferRaw, RequiredAlignment);
//     if (!UefiIsAlignedPointer(DiskReadBuffer, RequiredAlignment))
//     {
//         ERR("Aligned disk read buffer is not properly aligned (align %lu)\n", (RequiredAlignment) as usize);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* FUNCTIONS *****************************************************************/
//
// PCHAR
// GetHarddiskIdentifier(UCHAR DriveNumber)
// {
//     TRACE("GetHarddiskIdentifier: DriveNumber: %d\n", (DriveNumber) as usize);
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return NULL;
//     return PcDiskIdentifier[DriveNumber - FIRST_BIOS_DISK];
// }
//
// static LONG lReportError = 0; // >= 0: display errors; < 0: hide errors.
//
// LONG
// DiskReportError(BOOLEAN bShowError)
// {
//     /* Set the reference count */
//     if (bShowError) ++lReportError;
//     else            --lReportError;
//     return lReportError;
// }
//
//
// /* GPT Support Functions ******************************************************/
//
// static
// BOOLEAN
// UefiReadGptHeader(
//     IN UCHAR DriveNumber,
//     OUT PGPT_TABLE_HEADER GptHeader)
// {
//     ULONG ArcDriveIndex;
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//     ULONG BlockSize;
//     ULONG IoAlign;
//     ULONGLONG HeaderLba = EFI_HEADER_LOCATION;
//
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return FALSE;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//     if (ArcDriveIndex >= PcBiosDiskCount || InternalUefiDisk == NULL)
//         return FALSE;
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//         return FALSE;
//
//     if (!BlockIo->Media->MediaPresent)
//         return FALSE;
//
//     BlockSize = BlockIo->Media->BlockSize;
//     IoAlign = BlockIo->Media->IoAlign;
//
//     if (!UefiEnsureDiskReadBufferAligned(IoAlign))
//     {
//         ERR("Failed to align disk read buffer for drive %d\n", (DriveNumber) as usize);
//         return FALSE;
//     }
//
//     /* Read GPT header from LBA 1 */
//     Status = BlockIo->ReadBlocks(
//         BlockIo,
//         BlockIo->Media->MediaId,
//         HeaderLba,
//         BlockSize,
//         DiskReadBuffer);
//
//     if (EFI_ERROR(Status))
//         return FALSE;
//
//     RtlCopyMemory(GptHeader, DiskReadBuffer, sizeof(GPT_TABLE_HEADER));
//
//     /* Verify GPT signature */
//     if (memcmp(GptHeader->Signature, EFI_PARTITION_HEADER_SIGNATURE, 8) != 0)
//         return FALSE;
//
//     /* Verify revision */
//     if (GptHeader->Revision != EFI_TABLE_REVISION)
//     {
//         TRACE("GPT header has unsupported revision: 0x%x\n", (GptHeader->Revision) as usize);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// UefiGetGptPartitionEntry(
//     IN UCHAR DriveNumber,
//     IN ULONG PartitionNumber,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry)
// {
//     GPT_TABLE_HEADER GptHeader;
//     GPT_PARTITION_ENTRY GptEntry;
//     ULONG ArcDriveIndex;
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//     ULONG BlockSize;
//     ULONGLONG EntryLba;
//     ULONG EntryOffset;
//     ULONG EntriesPerBlock;
//     EFI_GUID UnusedGuid = EFI_PART_TYPE_UNUSED_GUID;
//
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return FALSE;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//     if (ArcDriveIndex >= PcBiosDiskCount || InternalUefiDisk == NULL)
//         return FALSE;
//
//     /* Read GPT header */
//     if (!UefiReadGptHeader(DriveNumber, &GptHeader))
//         return FALSE;
//
//     /* Validate partition number */
//     if (PartitionNumber == 0 || PartitionNumber > GptHeader.NumberOfPartitionEntries)
//         return FALSE;
//
//     /* Convert to 0-based index */
//     ULONG EntryIndex = PartitionNumber - 1;
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//         return FALSE;
//
//     BlockSize = BlockIo->Media->BlockSize;
//     EntriesPerBlock = BlockSize / GptHeader.SizeOfPartitionEntry;
//     EntryLba = GptHeader.PartitionEntryLba + (EntryIndex / EntriesPerBlock);
//     EntryOffset = (EntryIndex % EntriesPerBlock) * GptHeader.SizeOfPartitionEntry;
//
//     /* Read the block containing the partition entry */
//     Status = BlockIo->ReadBlocks(
//         BlockIo,
//         BlockIo->Media->MediaId,
//         EntryLba,
//         BlockSize,
//         DiskReadBuffer);
//
//     if (EFI_ERROR(Status))
//         return FALSE;
//
//     /* Extract partition entry */
//     RtlCopyMemory(&GptEntry, (PUCHAR)DiskReadBuffer + EntryOffset, sizeof(GPT_PARTITION_ENTRY));
//
//     /* Check if partition is unused */
//     if (memcmp(&GptEntry.PartitionTypeGuid, &UnusedGuid, sizeof(EFI_GUID)) == 0)
//         return FALSE;
//
//     /* Convert GPT entry to MBR-style PARTITION_TABLE_ENTRY */
//     RtlZeroMemory(PartitionTableEntry, sizeof(*PartitionTableEntry));
//
//     /* Calculate sector offset and count */
//     /* GPT uses LBA, convert to 512-byte sectors */
//     ULONGLONG StartLba = GptEntry.StartingLba;
//     ULONGLONG EndLba = GptEntry.EndingLba;
//     ULONGLONG SectorCount = (EndLba - StartLba + 1);
//
//     /* For GPT, we need to convert from device block size to 512-byte sectors */
//     ULONGLONG StartSector = (StartLba * BlockSize) / 512;
//     ULONGLONG SectorCount512 = (SectorCount * BlockSize) / 512;
//
//     PartitionTableEntry->SectorCountBeforePartition = (ULONG)StartSector;
//     PartitionTableEntry->PartitionSectorCount = (ULONG)SectorCount512;
//     PartitionTableEntry->SystemIndicator = PARTITION_GPT; /* Mark as GPT partition */
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// UefiGetBootPartitionEntry(
//     IN UCHAR DriveNumber,
//     OUT PPARTITION_TABLE_ENTRY PartitionTableEntry,
//     OUT PULONG BootPartition)
// {
//     ULONG PartitionNum;
//     ULONG ArcDriveIndex;
//     EFI_BLOCK_IO* BootBlockIo;
//     EFI_STATUS Status;
//     ULONGLONG BootPartitionSize;
//     PARTITION_TABLE_ENTRY TempPartitionEntry;
//
//     TRACE("UefiGetBootPartitionEntry: DriveNumber: %d\n", (DriveNumber - FIRST_BIOS_DISK) as usize);
//
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return FALSE;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//     if (ArcDriveIndex >= PcBiosDiskCount || handles == NULL || UefiBootRootIndex >= HandleCount)
//         return FALSE;
//
//     /* Get the boot handle's Block I/O protocol to determine partition offset/size */
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         handles[UefiBootRootIndex],
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BootBlockIo);
//
//     if (EFI_ERROR(Status) || BootBlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol for boot handle\n");
//         return FALSE;
//     }
//
//     /* For logical partitions, UEFI Block I/O protocol starts at block 0 */
//     /* We need to find which partition it corresponds to by comparing sizes */
//     BootPartitionSize = BootBlockIo->Media->LastBlock + 1;
//
//     TRACE("Boot partition: Size=%llu blocks, BlockSize=%lu\n", (//         BootPartitionSize) as usize, (BootBlockIo->Media->BlockSize) as usize);
//
//     /* If boot handle is the root device itself (not a logical partition) */
//     if (!BootBlockIo->Media->LogicalPartition)
//     {
//         TRACE("Boot handle is root device, using partition 0\n");
//         *BootPartition = 0;
//         if (PartitionTableEntry != NULL)
//         {
//             RtlZeroMemory(PartitionTableEntry, sizeof(*PartitionTableEntry));
//         }
//         return TRUE;
//     }
//
//     /* Boot handle is a logical partition - find matching partition entry */
//     /* Try to detect GPT first by reading GPT header */
//     GPT_TABLE_HEADER GptHeader;
//     BOOLEAN IsGpt = UefiReadGptHeader(DriveNumber, &GptHeader);
//
//     if (IsGpt)
//     {
//         /* For GPT, iterate through GPT partition entries */
//         GPT_TABLE_HEADER GptHeader;
//         GPT_PARTITION_ENTRY GptEntry;
//         ULONG ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//         EFI_BLOCK_IO* RootBlockIo;
//         ULONG BlockSize;
//         ULONGLONG EntryLba;
//         ULONG EntryOffset;
//         ULONG EntriesPerBlock;
//         EFI_STATUS Status;
//         EFI_GUID UnusedGuid = EFI_PART_TYPE_UNUSED_GUID;
//
//         if (!UefiReadGptHeader(DriveNumber, &GptHeader))
//         {
//             ERR("Failed to read GPT header\n");
//             return FALSE;
//         }
//
//         Status = GlobalSystemTable->BootServices->HandleProtocol(
//             InternalUefiDisk[ArcDriveIndex].Handle,
//             core::ptr::addr_of_mut!(BlockIoGuid),
//             (VOID**)&RootBlockIo);
//
//         if (EFI_ERROR(Status) || RootBlockIo == NULL)
//             return FALSE;
//
//         BlockSize = RootBlockIo->Media->BlockSize;
//         EntriesPerBlock = BlockSize / GptHeader.SizeOfPartitionEntry;
//
//         /* Iterate through GPT partition entries */
//         for (ULONG i = 0; i < GptHeader.NumberOfPartitionEntries; i++)
//         {
//             EntryLba = GptHeader.PartitionEntryLba + (i / EntriesPerBlock);
//             EntryOffset = (i % EntriesPerBlock) * GptHeader.SizeOfPartitionEntry;
//
//             /* Read the block containing the partition entry */
//             Status = RootBlockIo->ReadBlocks(
//                 RootBlockIo,
//                 RootBlockIo->Media->MediaId,
//                 EntryLba,
//                 BlockSize,
//                 DiskReadBuffer);
//
//             if (EFI_ERROR(Status))
//                 continue;
//
//             /* Extract partition entry */
//             RtlCopyMemory(&GptEntry, (PUCHAR)DiskReadBuffer + EntryOffset, sizeof(GPT_PARTITION_ENTRY));
//
//             /* Skip unused partitions */
//             if (memcmp(&GptEntry.PartitionTypeGuid, &UnusedGuid, sizeof(EFI_GUID)) == 0)
//                 continue;
//
//             /* Calculate partition size in blocks */
//             ULONGLONG PartitionSizeBlocks = GptEntry.EndingLba - GptEntry.StartingLba + 1;
//
//             TRACE("GPT Partition %lu: StartLba=%llu, EndLba=%llu, SizeBlocks=%llu\n", (//                 i + 1) as usize, (GptEntry.StartingLba) as usize, (GptEntry.EndingLba) as usize, (PartitionSizeBlocks) as usize);
//
//             /* Match partition by size (within 1 block tolerance for rounding) */
//             if (PartitionSizeBlocks == BootPartitionSize ||
//                 (PartitionSizeBlocks > 0 && 
//                  (PartitionSizeBlocks - 1 <= BootPartitionSize && 
//                   BootPartitionSize <= PartitionSizeBlocks + 1)))
//             {
//                 TRACE("Found matching GPT partition %lu: Size matches (%llu blocks)\n", (//                     i + 1) as usize, (BootPartitionSize) as usize);
//
//                 *BootPartition = i + 1; /* GPT partitions are 1-indexed */
//
//                 /* Convert GPT entry to MBR-style entry for compatibility */
//                 if (PartitionTableEntry != NULL)
//                 {
//                     RtlZeroMemory(PartitionTableEntry, sizeof(*PartitionTableEntry));
//                     ULONGLONG StartSector = (GptEntry.StartingLba * BlockSize) / 512;
//                     ULONGLONG SectorCount = (PartitionSizeBlocks * BlockSize) / 512;
//                     PartitionTableEntry->SectorCountBeforePartition = (ULONG)StartSector;
//                     PartitionTableEntry->PartitionSectorCount = (ULONG)SectorCount;
//                     PartitionTableEntry->SystemIndicator = PARTITION_GPT;
//                 }
//                 return TRUE;
//             }
//         }
//     }
//     else
//     {
//         /* MBR partition matching */
//         PartitionNum = FIRST_PARTITION;
//         while (DiskGetPartitionEntry(DriveNumber, PartitionNum, &TempPartitionEntry))
//         {
//             ULONGLONG PartitionSizeSectors = TempPartitionEntry.PartitionSectorCount;
//             ULONGLONG PartitionSizeBlocks;
//
//             /* Convert partition size from MBR sectors (always 512 bytes) to UEFI blocks */
//             /* MBR partition table always uses 512-byte sectors per specification */
//             /* UEFI Block I/O protocol reports sizes in device's BlockSize bytes */
//             /* Compare in bytes to avoid rounding issues, then convert to boot partition's block size */
//             ULONGLONG PartitionSizeBytes = PartitionSizeSectors * 512ULL;
//             PartitionSizeBlocks = PartitionSizeBytes / BootBlockIo->Media->BlockSize;
//
//             TRACE("Partition %lu: SizeSectors=%llu, SizeBlocks=%llu\n", (//                 PartitionNum) as usize, (PartitionSizeSectors) as usize, (PartitionSizeBlocks) as usize);
//
//             /* Match partition by size (within 1 block tolerance for rounding) */
//             if (PartitionSizeBlocks == BootPartitionSize ||
//                 (PartitionSizeBlocks > 0 && 
//                  (PartitionSizeBlocks - 1 <= BootPartitionSize && 
//                   BootPartitionSize <= PartitionSizeBlocks + 1)))
//             {
//                 TRACE("Found matching partition %lu: Size matches (%llu blocks)\n", (//                     PartitionNum) as usize, (BootPartitionSize) as usize);
//
//                 *BootPartition = PartitionNum;
//                 if (PartitionTableEntry != NULL)
//                 {
//                     RtlCopyMemory(PartitionTableEntry, &TempPartitionEntry, sizeof(*PartitionTableEntry));
//                 }
//                 return TRUE;
//             }
//
//             PartitionNum++;
//         }
//     }
//
//     /* If we couldn't find a match, check if it's a CD-ROM (special case) */
//     if (BootBlockIo->Media->RemovableMedia && BootBlockIo->Media->BlockSize == 2048)
//     {
//         TRACE("Boot device is CD-ROM, using partition 0xFF\n");
//         *BootPartition = 0xFF;
//         if (PartitionTableEntry != NULL)
//         {
//             RtlZeroMemory(PartitionTableEntry, sizeof(*PartitionTableEntry));
//         }
//         return TRUE;
//     }
//
//     /* Fallback: if we can't determine, use partition 1 */
//     ERR("Could not determine boot partition, using partition 1 as fallback\n");
//     PartitionNum = FIRST_PARTITION;
//     if (DiskGetPartitionEntry(DriveNumber, PartitionNum, &TempPartitionEntry))
//     {
//         *BootPartition = PartitionNum;
//         if (PartitionTableEntry != NULL)
//         {
//             RtlCopyMemory(PartitionTableEntry, &TempPartitionEntry, sizeof(*PartitionTableEntry));
//         }
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static
// ARC_STATUS
// UefiDiskClose(ULONG FileId)
// {
//     DISKCONTEXT* Context = FsGetDeviceSpecific(FileId);
//     FrLdrTempFree(Context, TAG_HW_DISK_CONTEXT);
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS
// UefiDiskGetFileInformation(ULONG FileId, FILEINFORMATION *Information)
// {
//     DISKCONTEXT* Context = FsGetDeviceSpecific(FileId);
//     RtlZeroMemory(Information, sizeof(*Information));
//
//     /*
//      * The ARC specification mentions that for partitions, StartingAddress and
//      * EndingAddress are the start and end positions of the partition in terms
//      * of byte offsets from the start of the disk.
//      * CurrentAddress is the current offset into (i.e. relative to) the partition.
//      */
//     Information->StartingAddress.QuadPart = Context->SectorOffset * Context->SectorSize;
//     Information->EndingAddress.QuadPart   = (Context->SectorOffset + Context->SectorCount) * Context->SectorSize;
//     Information->CurrentAddress.QuadPart  = Context->SectorNumber * Context->SectorSize;
//
//     Information->Type = DiskPeripheral; /* No floppy for you for now... */
//
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS
// UefiDiskOpen(CHAR *Path, OPENMODE OpenMode, ULONG *FileId)
// {
//     DISKCONTEXT* Context;
//     UCHAR DriveNumber;
//     ULONG DrivePartition, SectorSize;
//     ULONGLONG SectorOffset = 0;
//     ULONGLONG SectorCount = 0;
//     ULONG ArcDriveIndex;
//     PARTITION_TABLE_ENTRY PartitionTableEntry;
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//
//     TRACE("UefiDiskOpen: File ID: %p, Path: %s\n", (FileId) as usize, (Path) as usize);
//
//     if (DiskReadBufferSize == 0)
//     {
//         ERR("DiskOpen(): DiskReadBufferSize is 0, something is wrong.\n");
//         ASSERT(FALSE);
//         return ENOMEM;
//     }
//
//     if (!DissectArcPath(Path, NULL, &DriveNumber, &DrivePartition))
//         return EINVAL;
//
//     TRACE("Opening disk: DriveNumber: %d, DrivePartition: %d\n", (DriveNumber) as usize, (DrivePartition) as usize);
//
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return EINVAL;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//     if (ArcDriveIndex >= PcBiosDiskCount || InternalUefiDisk == NULL)
//         return EINVAL;
//
//     /* Get Block I/O protocol for this drive */
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol for drive %d\n", (DriveNumber) as usize);
//         return EINVAL;
//     }
//
//     /* Check media is present */
//     if (!BlockIo->Media->MediaPresent)
//     {
//         ERR("Media not present for drive %d\n", (DriveNumber) as usize);
//         return EINVAL;
//     }
//
//     SectorSize = BlockIo->Media->BlockSize;
//
//     if (DrivePartition != 0xff && DrivePartition != 0)
//     {
//         if (!DiskGetPartitionEntry(DriveNumber, DrivePartition, &PartitionTableEntry))
//             return EINVAL;
//
//         SectorOffset = PartitionTableEntry.SectorCountBeforePartition;
//         SectorCount = PartitionTableEntry.PartitionSectorCount;
//     }
//     else
//     {
//         GEOMETRY Geometry;
//         if (!MachDiskGetDriveGeometry(DriveNumber, &Geometry))
//             return EINVAL;
//
//         if (SectorSize != Geometry.BytesPerSector)
//         {
//             ERR("SectorSize (%lu) != Geometry.BytesPerSector (%lu), expect problems!\n", (//                 SectorSize) as usize, (Geometry.BytesPerSector) as usize);
//         }
//
//         SectorOffset = 0;
//         SectorCount = Geometry.Sectors;
//     }
//
//     Context = FrLdrTempAlloc(sizeof(DISKCONTEXT), TAG_HW_DISK_CONTEXT);
//     if (!Context)
//         return ENOMEM;
//
//     Context->DriveNumber = DriveNumber;
//     Context->SectorSize = SectorSize;
//     Context->SectorOffset = SectorOffset;
//     Context->SectorCount = SectorCount;
//     Context->SectorNumber = 0;
//     FsSetDeviceSpecific(*FileId, Context);
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS
// UefiDiskRead(ULONG FileId, VOID *Buffer, ULONG N, ULONG *Count)
// {
//     DISKCONTEXT* Context = FsGetDeviceSpecific(FileId);
//     UCHAR* Ptr = (UCHAR*)Buffer;
//     ULONG Length, TotalSectors, MaxSectors, ReadSectors;
//     ULONGLONG SectorOffset;
//     BOOLEAN ret;
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//     ULONG ArcDriveIndex;
//
//     ASSERT(DiskReadBufferSize > 0);
//
//
//     TotalSectors = (N + Context->SectorSize - 1) / Context->SectorSize;
//     MaxSectors   = DiskReadBufferSize / Context->SectorSize;
//     SectorOffset = Context->SectorOffset + Context->SectorNumber;
//
//     // If MaxSectors is 0, this will lead to infinite loop.
//     // In release builds assertions are disabled, however we also have sanity checks in DiskOpen()
//     ASSERT(MaxSectors > 0);
//
//     if (MaxSectors == 0)
//     {
//         ERR("MaxSectors is 0, cannot read\n");
//         *Count = 0;
//         return EIO;
//     }
//
//     ArcDriveIndex = Context->DriveNumber - FIRST_BIOS_DISK;
//     if (ArcDriveIndex >= PcBiosDiskCount || InternalUefiDisk == NULL)
//     {
//         ERR("Invalid drive number %d\n", (Context->DriveNumber) as usize);
//         *Count = 0;
//         return EINVAL;
//     }
//
//     /* Get Block I/O protocol */
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol\n");
//         *Count = 0;
//         return EIO;
//     }
//
//     if (!UefiEnsureDiskReadBufferAligned(BlockIo->Media->IoAlign))
//     {
//         ERR("Failed to align disk read buffer\n");
//         *Count = 0;
//         return EIO;
//     }
//
//     ret = TRUE;
//
//     while (TotalSectors)
//     {
//         ReadSectors = min(TotalSectors, MaxSectors);
//
//         Status = BlockIo->ReadBlocks(
//             BlockIo,
//             BlockIo->Media->MediaId,
//             SectorOffset,
//             ReadSectors * Context->SectorSize,
//             DiskReadBuffer);
//
//         if (EFI_ERROR(Status))
//         {
//             ERR("ReadBlocks failed: Status = 0x%lx\n", ((ULONG)Status) as usize);
//             ret = FALSE;
//             break;
//         }
//
//         Length = ReadSectors * Context->SectorSize;
//         Length = min(Length, N);
//
//         RtlCopyMemory(Ptr, DiskReadBuffer, Length);
//
//         Ptr += Length;
//         N -= Length;
//         SectorOffset += ReadSectors;
//         TotalSectors -= ReadSectors;
//     }
//
//     *Count = (ULONG)((ULONG_PTR)Ptr - (ULONG_PTR)Buffer);
//     Context->SectorNumber = SectorOffset - Context->SectorOffset;
//
//     return (ret ? ESUCCESS : EIO);
// }
//
// static
// ARC_STATUS
// UefiDiskSeek(ULONG FileId, LARGE_INTEGER *Position, SEEKMODE SeekMode)
// {
//     DISKCONTEXT* Context = FsGetDeviceSpecific(FileId);
//     LARGE_INTEGER NewPosition = *Position;
//
//     switch (SeekMode)
//     {
//         case SeekAbsolute:
//             break;
//         case SeekRelative:
//             NewPosition.QuadPart += (Context->SectorNumber * Context->SectorSize);
//             break;
//         default:
//             ASSERT(FALSE);
//             return EINVAL;
//     }
//
//     if (NewPosition.QuadPart & (Context->SectorSize - 1))
//         return EINVAL;
//
//     /* Convert in number of sectors */
//     NewPosition.QuadPart /= Context->SectorSize;
//
//     /* HACK: CDROMs may have a SectorCount of 0 */
//     if (Context->SectorCount != 0 && NewPosition.QuadPart >= Context->SectorCount)
//         return EINVAL;
//
//     Context->SectorNumber = NewPosition.QuadPart;
//     return ESUCCESS;
// }
//
// static const DEVVTBL UefiDiskVtbl =
// {
//     UefiDiskClose,
//     UefiDiskGetFileInformation,
//     UefiDiskOpen,
//     UefiDiskRead,
//     UefiDiskSeek,
// };
//
// static
// VOID
// GetHarddiskInformation(UCHAR DriveNumber)
// {
//     PMASTER_BOOT_RECORD Mbr;
//     PULONG Buffer;
//     ULONG i;
//     ULONG Checksum;
//     ULONG Signature;
//     BOOLEAN ValidPartitionTable;
//     CHAR ArcName[MAX_PATH];
//     PARTITION_TABLE_ENTRY PartitionTableEntry;
//     ULONG ArcDriveIndex;
//     PCHAR Identifier;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//     if (ArcDriveIndex >= 32)
//         return;
//
//     Identifier = PcDiskIdentifier[ArcDriveIndex];
//
//     /* Detect disk partition type */
//     DiskDetectPartitionType(DriveNumber);
//
//     /* Read the MBR */
//     if (!MachDiskReadLogicalSectors(DriveNumber, 0ULL, 1, DiskReadBuffer))
//     {
//         ERR("Reading MBR failed\n");
//         /* We failed, use a default identifier */
//         sprintf(Identifier, "BIOSDISK%d", ArcDriveIndex);
//         return;
//     }
//
//     Buffer = (ULONG*)DiskReadBuffer;
//     Mbr = (PMASTER_BOOT_RECORD)DiskReadBuffer;
//
//     Signature = Mbr->Signature;
//     TRACE("Signature: %x\n", (Signature) as usize);
//
//     /* Calculate the MBR checksum */
//     Checksum = 0;
//     for (i = 0; i < 512 / sizeof(ULONG); i++)
//     {
//         Checksum += Buffer[i];
//     }
//     Checksum = ~Checksum + 1;
//     TRACE("Checksum: %x\n", (Checksum) as usize);
//
//     ValidPartitionTable = (Mbr->MasterBootRecordMagic == 0xAA55);
//
//     /* Fill out the ARC disk block */
//     sprintf(ArcName, "multi(0)disk(0)rdisk(%u)", ArcDriveIndex);
//     AddReactOSArcDiskInfo(ArcName, Signature, Checksum, ValidPartitionTable);
//
//     sprintf(ArcName, "multi(0)disk(0)rdisk(%u)partition(0)", ArcDriveIndex);
//     FsRegisterDevice(ArcName, &UefiDiskVtbl);
//
//     /* Add partitions */
//     i = FIRST_PARTITION;
//     DiskReportError(FALSE);
//     while (DiskGetPartitionEntry(DriveNumber, i, &PartitionTableEntry))
//     {
//         if (PartitionTableEntry.SystemIndicator != PARTITION_ENTRY_UNUSED)
//         {
//             sprintf(ArcName, "multi(0)disk(0)rdisk(%u)partition(%lu)", ArcDriveIndex, i);
//             FsRegisterDevice(ArcName, &UefiDiskVtbl);
//         }
//         i++;
//     }
//     DiskReportError(TRUE);
//
//     if (ArcDriveIndex < PcBiosDiskCount && InternalUefiDisk != NULL)
//     {
//         InternalUefiDisk[ArcDriveIndex].NumOfPartitions = i;
//     }
//
//     /* Convert checksum and signature to identifier string */
//     Identifier[0] = Hex[(Checksum >> 28) & 0x0F];
//     Identifier[1] = Hex[(Checksum >> 24) & 0x0F];
//     Identifier[2] = Hex[(Checksum >> 20) & 0x0F];
//     Identifier[3] = Hex[(Checksum >> 16) & 0x0F];
//     Identifier[4] = Hex[(Checksum >> 12) & 0x0F];
//     Identifier[5] = Hex[(Checksum >> 8) & 0x0F];
//     Identifier[6] = Hex[(Checksum >> 4) & 0x0F];
//     Identifier[7] = Hex[Checksum & 0x0F];
//     Identifier[8] = '-';
//     Identifier[9] = Hex[(Signature >> 28) & 0x0F];
//     Identifier[10] = Hex[(Signature >> 24) & 0x0F];
//     Identifier[11] = Hex[(Signature >> 20) & 0x0F];
//     Identifier[12] = Hex[(Signature >> 16) & 0x0F];
//     Identifier[13] = Hex[(Signature >> 12) & 0x0F];
//     Identifier[14] = Hex[(Signature >> 8) & 0x0F];
//     Identifier[15] = Hex[(Signature >> 4) & 0x0F];
//     Identifier[16] = Hex[Signature & 0x0F];
//     Identifier[17] = '-';
//     Identifier[18] = (ValidPartitionTable ? 'A' : 'X');
//     Identifier[19] = 0;
//     TRACE("Identifier: %s\n", (Identifier) as usize);
// }
//
// static
// VOID
// UefiSetupBlockDevices(VOID)
// {
//     ULONG BlockDeviceIndex;
//     ULONG SystemHandleCount;
//     EFI_STATUS Status;
//     ULONG i;
//     UINTN HandleSize = 0;
//     EFI_BLOCK_IO* BlockIo;
//
//     PcBiosDiskCount = 0;
//     UefiBootRootIndex = 0;
//
//     /* Step 1: Get the size needed for handles buffer - no matter how it fails we're good */
//     Status = GlobalSystemTable->BootServices->LocateHandle(
//         ByProtocol,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         NULL,
//         &HandleSize,
//         NULL);
//
//     if (HandleSize == 0)
//     {
//         ERR("Failed to get handle buffer size: Status = 0x%lx\n", ((ULONG)Status) as usize);
//         return;
//     }
//
//     SystemHandleCount = HandleSize / sizeof(EFI_HANDLE);
//     if (SystemHandleCount == 0)
//     {
//         ERR("No block devices found\n");
//         return;
//     }
//
//     /* Step 2: Allocate buffer for handles */
//     handles = MmAllocateMemoryWithType(HandleSize, LoaderFirmwareTemporary);
//     if (handles == NULL)
//     {
//         ERR("Failed to allocate memory for handles\n");
//         return;
//     }
//
//     /* Step 3: Get actual handles */
//     Status = GlobalSystemTable->BootServices->LocateHandle(
//         ByProtocol,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         NULL,
//         &HandleSize,
//         handles);
//
//     if (EFI_ERROR(Status))
//     {
//         ERR("Failed to locate block device handles: Status = 0x%lx\n", ((ULONG)Status) as usize);
//         return;
//     }
//
//     HandleCount = SystemHandleCount;
//
//     /* Step 4: Allocate internal disk structure */
//     InternalUefiDisk = MmAllocateMemoryWithType(
//         sizeof(INTERNAL_UEFI_DISK) * SystemHandleCount,
//         LoaderFirmwareTemporary);
//
//     if (InternalUefiDisk == NULL)
//     {
//         ERR("Failed to allocate memory for internal disk structure\n");
//         return;
//     }
//
//     RtlZeroMemory(InternalUefiDisk, sizeof(INTERNAL_UEFI_DISK) * SystemHandleCount);
//
//     /* Step 5: Find boot handle and determine if it's a root device or partition */
//     UefiBootRootIndex = 0;
//     for (i = 0; i < SystemHandleCount; i++)
//     {
//         if (handles[i] == PublicBootHandle)
//         {
//             UefiBootRootIndex = i;
//             TRACE("Found boot handle at index %lu\n", (i) as usize);
//
//             /* Check if boot handle is a root device or partition */
//             Status = GlobalSystemTable->BootServices->HandleProtocol(
//                 handles[i],
//                 core::ptr::addr_of_mut!(BlockIoGuid),
//                 (VOID**)&BlockIo);
//
//             if (!EFI_ERROR(Status) && BlockIo != NULL)
//             {
//                 TRACE("Boot handle: LogicalPartition=%s, RemovableMedia=%s, BlockSize=%lu\n", (//                     BlockIo->Media->LogicalPartition ? "TRUE" : "FALSE") as usize, (//                     BlockIo->Media->RemovableMedia ? "TRUE" : "FALSE") as usize, (//                     BlockIo->Media->BlockSize) as usize);
//             }
//             break;
//         }
//     }
//
//     /* Step 6: Enumerate root block devices (skip logical partitions) */
//     BlockDeviceIndex = 0;
//     for (i = 0; i < SystemHandleCount; i++)
//     {
//         Status = GlobalSystemTable->BootServices->HandleProtocol(
//             handles[i],
//             core::ptr::addr_of_mut!(BlockIoGuid),
//             (VOID**)&BlockIo);
//
//         if (EFI_ERROR(Status))
//         {
//             TRACE("HandleProtocol failed for handle %lu: Status = 0x%lx\n", (i) as usize, ((ULONG)Status) as usize);
//             continue;
//         }
//
//         if (BlockIo == NULL)
//         {
//             TRACE("BlockIo is NULL for handle %lu\n", (i) as usize);
//             continue;
//         }
//
//         if (!BlockIo->Media->MediaPresent)
//         {
//             TRACE("Media not present for handle %lu\n", (i) as usize);
//             continue;
//         }
//
//         if (BlockIo->Media->BlockSize == 0)
//         {
//             TRACE("Invalid block size (0) for handle %lu\n", (i) as usize);
//             continue;
//         }
//
//         /* Filter out devices with unusually large block sizes (flash devices) */
//         if (BlockIo->Media->BlockSize > MAX_SUPPORTED_BLOCK_SIZE)
//         {
//             TRACE("Block size too large (%lu) for handle %lu, skipping\n", (//                 BlockIo->Media->BlockSize) as usize, (i) as usize);
//             continue;
//         }
//
//         /* Logical partitions are handled separately by partition scanning */
//         if (BlockIo->Media->LogicalPartition)
//         {
//             /* If boot handle is a logical partition, we need to find its parent root device */
//             /* For now, we'll handle this after enumeration by matching handles */
//             TRACE("Skipping logical partition handle %lu\n", (i) as usize);
//             continue;
//         }
//
//         /* This is a root block device */
//         TRACE("Found root block device at index %lu: BlockSize=%lu, LastBlock=%llu\n", (//             i) as usize, (BlockIo->Media->BlockSize) as usize, (BlockIo->Media->LastBlock) as usize);
//
//         InternalUefiDisk[BlockDeviceIndex].ArcDriveNumber = BlockDeviceIndex;
//         InternalUefiDisk[BlockDeviceIndex].UefiHandleIndex = i;
//         InternalUefiDisk[BlockDeviceIndex].Handle = handles[i];
//         InternalUefiDisk[BlockDeviceIndex].IsThisTheBootDrive = FALSE;
//
//         /* Check if this root device contains the boot partition */
//         /* If boot handle is a logical partition, we need to find which root device it belongs to */
//         /* For now, if boot handle index matches, mark it as boot device */
//         if (i == UefiBootRootIndex)
//         {
//             InternalUefiDisk[BlockDeviceIndex].IsThisTheBootDrive = TRUE;
//             PublicBootArcDisk = BlockDeviceIndex;
//             TRACE("Boot device is at ARC drive index %lu (root device)\n", (BlockDeviceIndex) as usize);
//         }
//
//         /* Increment PcBiosDiskCount BEFORE calling GetHarddiskInformation
//          * so that UefiDiskReadLogicalSectors can validate the drive number */
//         PcBiosDiskCount = BlockDeviceIndex + 1;
//
//         TRACE("Calling GetHarddiskInformation for drive %d (BlockDeviceIndex=%lu)\n", (//             BlockDeviceIndex + FIRST_BIOS_DISK) as usize, (BlockDeviceIndex) as usize);
//         if (!UefiEnsureDiskReadBufferAligned(BlockIo->Media->IoAlign))
//         {
//             ERR("Failed to align disk read buffer for drive %d\n", (BlockDeviceIndex + FIRST_BIOS_DISK) as usize);
//             return;
//         }
//
//         GetHarddiskInformation(BlockDeviceIndex + FIRST_BIOS_DISK);
//         BlockDeviceIndex++;
//     }
//
//     /* Step 7: If boot handle was a logical partition, find its parent root device */
//     if (UefiBootRootIndex < HandleCount)
//     {
//         Status = GlobalSystemTable->BootServices->HandleProtocol(
//             handles[UefiBootRootIndex],
//             core::ptr::addr_of_mut!(BlockIoGuid),
//             (VOID**)&BlockIo);
//
//         if (!EFI_ERROR(Status) && BlockIo != NULL && BlockIo->Media->LogicalPartition)
//         {
//             TRACE("Boot handle is a logical partition, searching for parent root device\n");
//             TRACE("Boot partition: BlockSize=%lu, RemovableMedia=%s\n", (//                 BlockIo->Media->BlockSize) as usize, (//                 BlockIo->Media->RemovableMedia ? "TRUE" : "FALSE") as usize);
//
//             /* Find the root device that matches the boot partition's characteristics */
//             /* For CD-ROMs: match BlockSize=2048 and RemovableMedia=TRUE */
//             /* For hard disks: match BlockSize and find the root device before this partition */
//             BOOLEAN FoundBootDevice = FALSE;
//             for (i = 0; i < BlockDeviceIndex; i++)
//             {
//                 EFI_BLOCK_IO* RootBlockIo;
//                 Status = GlobalSystemTable->BootServices->HandleProtocol(
//                     InternalUefiDisk[i].Handle,
//                     core::ptr::addr_of_mut!(BlockIoGuid),
//                     (VOID**)&RootBlockIo);
//
//                 if (EFI_ERROR(Status) || RootBlockIo == NULL)
//                     continue;
//
//                 /* For CD-ROM: match BlockSize=2048 and RemovableMedia */
//                 if (BlockIo->Media->BlockSize == 2048 && BlockIo->Media->RemovableMedia)
//                 {
//                     if (RootBlockIo->Media->BlockSize == 2048 && 
//                         RootBlockIo->Media->RemovableMedia &&
//                         !RootBlockIo->Media->LogicalPartition)
//                     {
//                         PublicBootArcDisk = i;
//                         InternalUefiDisk[i].IsThisTheBootDrive = TRUE;
//                         FoundBootDevice = TRUE;
//                         TRACE("Found CD-ROM boot device at ARC drive index %lu\n", (i) as usize);
//                         break;
//                     }
//                 }
//                 /* For hard disk partitions: the root device should be before the partition handle */
//                 else if (InternalUefiDisk[i].UefiHandleIndex < UefiBootRootIndex)
//                 {
//                     /* Check if this root device is likely the parent */
//                     if (RootBlockIo->Media->BlockSize == BlockIo->Media->BlockSize &&
//                         !RootBlockIo->Media->LogicalPartition)
//                     {
//                         /* This might be the parent, but we need to be more certain */
//                         /* For now, use the last root device before the boot handle */
//                         PublicBootArcDisk = i;
//                         InternalUefiDisk[i].IsThisTheBootDrive = TRUE;
//                         FoundBootDevice = TRUE;
//                         TRACE("Found potential hard disk boot device at ARC drive index %lu\n", (i) as usize);
//                     }
//                 }
//             }
//
//             if (!FoundBootDevice && PcBiosDiskCount > 0)
//             {
//                 PublicBootArcDisk = 0;
//                 InternalUefiDisk[0].IsThisTheBootDrive = TRUE;
//                 TRACE("Could not determine boot device, assuming first drive\n");
//             }
//         }
//     }
//
//     TRACE("Found %lu root block devices\n", (PcBiosDiskCount) as usize);
// }
//
// static
// BOOLEAN
// UefiSetBootpath(VOID)
// {
//     EFI_BLOCK_IO* BootBlockIo = NULL;
//     EFI_BLOCK_IO* RootBlockIo = NULL;
//     EFI_STATUS Status;
//     ULONG ArcDriveIndex;
//
//     TRACE("UefiSetBootpath: Setting up boot path\n");
//
//     if (UefiBootRootIndex >= HandleCount || handles == NULL)
//     {
//         ERR("Invalid boot root index\n");
//         return FALSE;
//     }
//
//     ArcDriveIndex = PublicBootArcDisk;
//     if (ArcDriveIndex >= PcBiosDiskCount || InternalUefiDisk == NULL)
//     {
//         ERR("Invalid boot arc disk index\n");
//         return FALSE;
//     }
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         handles[UefiBootRootIndex],
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BootBlockIo);
//
//     if (EFI_ERROR(Status) || BootBlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol for boot handle\n");
//         return FALSE;
//     }
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&RootBlockIo);
//
//     if (EFI_ERROR(Status) || RootBlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol for boot root device\n");
//         return FALSE;
//     }
//
//     FrldrBootDrive = (FIRST_BIOS_DISK + ArcDriveIndex);
//
//     /* Check if booting from CD-ROM by checking the boot handle properties */
//     /* CD-ROMs have BlockSize=2048 and RemovableMedia=TRUE */
//     if (BootBlockIo->Media->RemovableMedia == TRUE && BootBlockIo->Media->BlockSize == 2048)
//     {
//         /* Boot Partition 0xFF is the magic value that indicates booting from CD-ROM */
//         FrldrBootPartition = 0xFF;
//         RtlStringCbPrintfA(FrLdrBootPath, sizeof(FrLdrBootPath),
//                            "multi(0)disk(0)cdrom(%u)", ArcDriveIndex);
//         TRACE("Boot path set to CD-ROM: %s\n", (FrLdrBootPath) as usize);
//     }
//     else
//     {
//         ULONG BootPartition;
//         PARTITION_TABLE_ENTRY PartitionEntry;
//
//         /* This is a hard disk */
//         /* If boot handle is a logical partition, we need to determine which partition number */
//         if (BootBlockIo->Media->LogicalPartition)
//         {
//             /* For logical partitions, we need to find the partition number */
//             /* This is tricky - we'll use partition 1 as default for now */
//             /* TODO: Properly determine partition number from boot handle */
//             BootPartition = FIRST_PARTITION;
//             TRACE("Boot handle is logical partition, using partition %lu\n", (BootPartition) as usize);
//         }
//         else
//         {
//             /* Boot handle is the root device itself */
//             if (!UefiGetBootPartitionEntry(FrldrBootDrive, &PartitionEntry, &BootPartition))
//             {
//                 ERR("Failed to get boot partition entry\n");
//                 return FALSE;
//             }
//         }
//
//         RtlStringCbPrintfA(FrLdrBootPath, sizeof(FrLdrBootPath),
//                            "multi(0)disk(0)rdisk(%u)partition(%lu)",
//                            ArcDriveIndex, BootPartition);
//         TRACE("Boot path set to hard disk: %s\n", (FrLdrBootPath) as usize);
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// UefiInitializeBootDevices(VOID)
// {
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//     ULONG ArcDriveIndex;
//     PMASTER_BOOT_RECORD Mbr;
//     PULONG Buffer;
//     ULONG Checksum = 0;
//     ULONG Signature;
//     ULONG i;
//
//     DiskReadBufferSize = EFI_PAGE_SIZE;
//     DiskReadBuffer = NULL;
//     DiskReadBufferRaw = NULL;
//     DiskReadBufferAlignment = 1;
//     DiskReadBufferFromPool = FALSE;
//     if (!UefiEnsureDiskReadBufferAligned(1))
//     {
//         ERR("Failed to allocate disk read buffer\n");
//         return FALSE;
//     }
//
//     UefiSetupBlockDevices();
//
//     if (PcBiosDiskCount == 0)
//     {
//         ERR("No block devices found\n");
//         return FALSE;
//     }
//
//     if (!UefiSetBootpath())
//     {
//         ERR("Failed to set boot path\n");
//         return FALSE;
//     }
//
//     /* Handle CD-ROM boot device registration */
//     ArcDriveIndex = PublicBootArcDisk;
//     if (ArcDriveIndex >= PcBiosDiskCount || InternalUefiDisk == NULL)
//     {
//         ERR("Invalid boot arc disk index\n");
//         return FALSE;
//     }
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol\n");
//         return FALSE;
//     }
//
//     if (BlockIo->Media->RemovableMedia == TRUE && BlockIo->Media->BlockSize == 2048)
//     {
//         /* Read the MBR from CD-ROM (sector 16) */
//         if (!MachDiskReadLogicalSectors(FrldrBootDrive, 16ULL, 1, DiskReadBuffer))
//         {
//             ERR("Reading MBR from CD-ROM failed\n");
//             return FALSE;
//         }
//
//         Buffer = (ULONG*)DiskReadBuffer;
//         Mbr = (PMASTER_BOOT_RECORD)DiskReadBuffer;
//
//         Signature = Mbr->Signature;
//         TRACE("CD-ROM Signature: %x\n", (Signature) as usize);
//
//         /* Calculate the MBR checksum */
//         for (i = 0; i < 2048 / sizeof(ULONG); i++)
//         {
//             Checksum += Buffer[i];
//         }
//         Checksum = ~Checksum + 1;
//         TRACE("CD-ROM Checksum: %x\n", (Checksum) as usize);
//
//         /* Fill out the ARC disk block */
//         AddReactOSArcDiskInfo(FrLdrBootPath, Signature, Checksum, TRUE);
//
//         FsRegisterDevice(FrLdrBootPath, &UefiDiskVtbl);
//         TRACE("Registered CD-ROM boot device: 0x%02X\n", ((int)FrldrBootDrive) as usize);
//     }
//
//     return TRUE;
// }
//
// UCHAR
// UefiGetFloppyCount(VOID)
// {
//     /* No floppy support in UEFI */
//     return 0;
// }
//
// BOOLEAN
// UefiDiskReadLogicalSectors(
//     IN UCHAR DriveNumber,
//     IN ULONGLONG SectorNumber,
//     IN ULONG SectorCount,
//     OUT PVOID Buffer)
// {
//     ULONG ArcDriveIndex;
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//     ULONG BlockSize;
//     ULONG IoAlign;
//
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return FALSE;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//
//     if (InternalUefiDisk == NULL)
//     {
//         ERR("InternalUefiDisk not initialized\n");
//         return FALSE;
//     }
//
//     if (ArcDriveIndex >= 32)
//     {
//         ERR("Drive index out of bounds: %d (ArcDriveIndex=%lu)\n", (DriveNumber) as usize, (ArcDriveIndex) as usize);
//         return FALSE;
//     }
//
//     /* Allow access during initialization: check if handle is set up */
//     /* During initialization, Handle is set before GetHarddiskInformation is called */
//     if (InternalUefiDisk[ArcDriveIndex].Handle == NULL)
//     {
//         ERR("Invalid drive number: %d (ArcDriveIndex=%lu, PcBiosDiskCount=%lu, Handle=NULL)\n", (//             DriveNumber) as usize, (ArcDriveIndex) as usize, (PcBiosDiskCount) as usize);
//         return FALSE;
//     }
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol for drive %d\n", (DriveNumber) as usize);
//         return FALSE;
//     }
//
//     if (!BlockIo->Media->MediaPresent)
//     {
//         ERR("Media not present for drive %d\n", (DriveNumber) as usize);
//         return FALSE;
//     }
//
//     BlockSize = BlockIo->Media->BlockSize;
//     IoAlign = BlockIo->Media->IoAlign;
//
//     if (!UefiEnsureDiskReadBufferAligned(IoAlign))
//     {
//         ERR("Failed to align disk read buffer for drive %d\n", (DriveNumber) as usize);
//         return FALSE;
//     }
//
//     if (!UefiIsAlignedPointer(Buffer, (IoAlign == 0) ? 1 : IoAlign))
//     {
//         ULONG TotalSectors = SectorCount;
//         ULONG MaxSectors = DiskReadBufferSize / BlockSize;
//         ULONGLONG CurrentSector = SectorNumber;
//         PUCHAR OutPtr = (PUCHAR)Buffer;
//
//         if (MaxSectors == 0)
//         {
//             ERR("DiskReadBufferSize too small for block size %lu\n", (BlockSize) as usize);
//             return FALSE;
//         }
//
//         while (TotalSectors)
//         {
//             ULONG ReadSectors = min(TotalSectors, MaxSectors);
//             UINTN ReadSize = ReadSectors * BlockSize;
//
//             Status = BlockIo->ReadBlocks(
//                 BlockIo,
//                 BlockIo->Media->MediaId,
//                 CurrentSector,
//                 ReadSize,
//                 DiskReadBuffer);
//
//             if (EFI_ERROR(Status))
//             {
//                 ERR("ReadBlocks failed: DriveNumber=%d, SectorNumber=%llu, SectorCount=%lu, Status=0x%lx\n", (//                     DriveNumber) as usize, (CurrentSector) as usize, (ReadSectors) as usize, ((ULONG)Status) as usize);
//                 ERR("ReadBlocks details: BlockSize=%lu, IoAlign=%lu, Buffer=%p, DiskReadBuffer=%p, MediaId=0x%lx\n", (//                     BlockSize) as usize, (IoAlign) as usize, (Buffer) as usize, (DiskReadBuffer) as usize, ((ULONG)BlockIo->Media->MediaId) as usize);
//                 ERR("ReadBlocks media: LastBlock=%llu, LogicalPartition=%s, RemovableMedia=%s\n", (//                     BlockIo->Media->LastBlock) as usize, (//                     BlockIo->Media->LogicalPartition ? "TRUE" : "FALSE") as usize, (//                     BlockIo->Media->RemovableMedia ? "TRUE" : "FALSE") as usize);
//                 return FALSE;
//             }
//
//             RtlCopyMemory(OutPtr, DiskReadBuffer, ReadSize);
//             OutPtr += ReadSize;
//             CurrentSector += ReadSectors;
//             TotalSectors -= ReadSectors;
//         }
//
//         return TRUE;
//     }
//
//     Status = BlockIo->ReadBlocks(
//         BlockIo,
//         BlockIo->Media->MediaId,
//         SectorNumber,
//         SectorCount * BlockSize,
//         Buffer);
//
//     if (EFI_ERROR(Status))
//     {
//         ERR("ReadBlocks failed: DriveNumber=%d, SectorNumber=%llu, SectorCount=%lu, Status=0x%lx\n", (//             DriveNumber) as usize, (SectorNumber) as usize, (SectorCount) as usize, ((ULONG)Status) as usize);
//         ERR("ReadBlocks details: BlockSize=%lu, IoAlign=%lu, Buffer=%p, DiskReadBuffer=%p, MediaId=0x%lx\n", (//             BlockSize) as usize, (IoAlign) as usize, (Buffer) as usize, (DiskReadBuffer) as usize, ((ULONG)BlockIo->Media->MediaId) as usize);
//         ERR("ReadBlocks media: LastBlock=%llu, LogicalPartition=%s, RemovableMedia=%s\n", (//             BlockIo->Media->LastBlock) as usize, (//             BlockIo->Media->LogicalPartition ? "TRUE" : "FALSE") as usize, (//             BlockIo->Media->RemovableMedia ? "TRUE" : "FALSE") as usize);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// UefiDiskGetDriveGeometry(UCHAR DriveNumber, PGEOMETRY Geometry)
// {
//     ULONG ArcDriveIndex;
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return FALSE;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//
//     if (InternalUefiDisk == NULL)
//     {
//         ERR("InternalUefiDisk not initialized\n");
//         return FALSE;
//     }
//
//     if (ArcDriveIndex >= 32 || InternalUefiDisk[ArcDriveIndex].Handle == NULL)
//     {
//         ERR("Invalid drive number: %d\n", (DriveNumber) as usize);
//         return FALSE;
//     }
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol for drive %d\n", (DriveNumber) as usize);
//         return FALSE;
//     }
//
//     if (!BlockIo->Media->MediaPresent)
//     {
//         ERR("Media not present for drive %d\n", (DriveNumber) as usize);
//         return FALSE;
//     }
//
//     Geometry->Cylinders = 1; /* Not relevant for UEFI Block I/O protocol */
//     Geometry->Heads = 1;     /* Not relevant for UEFI Block I/O protocol */
//     Geometry->SectorsPerTrack = (ULONG)(BlockIo->Media->LastBlock + 1);
//     Geometry->BytesPerSector = BlockIo->Media->BlockSize;
//     Geometry->Sectors = BlockIo->Media->LastBlock + 1;
//
//     return TRUE;
// }
//
// ULONG
// UefiDiskGetCacheableBlockCount(UCHAR DriveNumber)
// {
//     ULONG ArcDriveIndex;
//     EFI_BLOCK_IO* BlockIo;
//     EFI_STATUS Status;
//
//     if (DriveNumber < FIRST_BIOS_DISK)
//         return 0;
//
//     ArcDriveIndex = DriveNumber - FIRST_BIOS_DISK;
//
//     if (InternalUefiDisk == NULL)
//     {
//         ERR("InternalUefiDisk not initialized\n");
//         return 0;
//     }
//
//     if (ArcDriveIndex >= 32 || InternalUefiDisk[ArcDriveIndex].Handle == NULL)
//     {
//         ERR("Invalid drive number: %d\n", (DriveNumber) as usize);
//         return 0;
//     }
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(
//         InternalUefiDisk[ArcDriveIndex].Handle,
//         core::ptr::addr_of_mut!(BlockIoGuid),
//         (VOID**)&BlockIo);
//
//     if (EFI_ERROR(Status) || BlockIo == NULL)
//     {
//         ERR("Failed to get Block I/O protocol for drive %d\n", (DriveNumber) as usize);
//         return 0;
//     }
//
//     if (!BlockIo->Media->MediaPresent)
//     {
//         ERR("Media not present for drive %d\n", (DriveNumber) as usize);
//         return 0;
//     }
//
//     return (ULONG)(BlockIo->Media->LastBlock + 1);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

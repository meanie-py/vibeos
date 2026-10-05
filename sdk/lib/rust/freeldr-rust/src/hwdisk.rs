use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER};
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_CdromController as CdromController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DiskPeripheral as DiskPeripheral;
pub use ntkrnl_sys::ddk::_ARC_CODES_EINVAL as EINVAL;
pub use ntkrnl_sys::ddk::_ARC_CODES_EIO as EIO;
pub use ntkrnl_sys::ddk::_ARC_CODES_ENOMEM as ENOMEM;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_FloppyDiskPeripheral as FloppyDiskPeripheral;
pub use ntkrnl_sys::ddk::_SEEKMODE_SeekAbsolute as SeekAbsolute;
pub use ntkrnl_sys::ddk::_SEEKMODE_SeekRelative as SeekRelative;

// ---- Dropped non-fn content faithfully restored (rustconv drops structs/statics) ----

// hwdisk.c #defines
pub const FIRST_BIOS_DISK: UCHAR = 0x80;
pub const FIRST_PARTITION: ULONG = 1;
// arch/pc/hardware.h: #define TAG_HW_DISK_CONTEXT 'cDwH'
// A C multi-character literal is BIG-endian by definition -- 'cDwH' is
// ('c'<<24)|('D'<<16)|('w'<<8)|'H' = 0x63447748 on every target. from_le_bytes
// here yielded 0x48774463, the bytes reversed, disagreeing with uefidisk.rs's
// spelling of the SAME constant. Self-consistent while one file both allocates
// and frees, and wrong the moment they meet.
pub const TAG_HW_DISK_CONTEXT: ULONG = u32::from_be_bytes(*b"cDwH");

// hwdisk.c: typedef struct tagDISKCONTEXT
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DISKCONTEXT {
    pub DriveNumber: UCHAR,
    pub IsFloppy: BOOLEAN,
    pub SectorSize: ULONG,
    pub SectorOffset: ULONGLONG,
    pub SectorCount: ULONGLONG,
    pub SectorNumber: ULONGLONG,
}

// hwdisk.c: static const CHAR Hex[] = "0123456789abcdef";
static Hex: [u8; 17] = *b"0123456789abcdef\0";

// hwdisk.c: UCHAR PcBiosDiskCount = 0; (external linkage; read elsewhere)
#[no_mangle]
pub static mut PcBiosDiskCount: UCHAR = 0;

// hwdisk.c: static CHAR PcDiskIdentifier[32][20];
static mut PcDiskIdentifier: [[u8; 20]; 32] = [[0; 20]; 32];

// hwdisk.c: static const DEVVTBL DiskVtbl
static DiskVtbl: DEVVTBL = DEVVTBL {
    Close: Some(DiskClose),
    GetFileInformation: Some(DiskGetFileInformation),
    Open: Some(DiskOpen),
    Read: Some(DiskRead),
    Seek: Some(DiskSeek),
    ServiceName: core::ptr::null(),
};

// pcmem.c global (defined #[no_mangle] in crate::pcmem); machpc.h extern decl.
// freeldr.c / uefidisk.c globals (external C linkage).
extern "C" {
    static mut DiskReadBufferSize: SIZE_T;
    static mut FrLdrBootPath: [CHAR; MAX_PATH];
    static FrldrBootDrive: UCHAR;
    static mut FrldrBootPartition: ULONG;
}

extern "C" {
    fn FsGetDeviceSpecific(FileId: ULONG) -> *mut DISKCONTEXT;
    fn MachDiskReadLogicalSectors( DriveNumber: ULONG, SectorOffset: ULONGLONG, ReadSectors: ULONG, Buffer: *mut VOID, ) -> BOOLEAN;
    fn DiskDetectPartitionType(DriveNumber: u8);
    fn ERR(fmt: *const i8, ...);
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn TRACE(fmt: *const i8, ...);
    fn AddReactOSArcDiskInfo(ArcName: *const i8, GptDiskGuid: *const core::ffi::c_void, Signature: u32, Checksum: u32, ValidPartitionTable: u8);
    fn FsRegisterDevice(ArcName: *const i8, Vtbl: *const core::ffi::c_void);
    fn DiskReportError(Enable: u8);
    fn DiskGetPartitionEntry(DriveNumber: u8, PartitionNumber: u32, Entry: *mut PARTITION_TABLE_ENTRY) -> u8;
    fn DiskGetBootPartitionEntry( drive: ULONG, entry: *mut PARTITION_TABLE_ENTRY, partition: *mut ULONG, ) -> BOOLEAN;
    fn RtlStringCbPrintfA(dest: *mut i8, size: usize, format: *const i8, ...) -> NTSTATUS;
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
    fn DiskGetConfigType(drive: UCHAR) -> CONFIGURATION_TYPE;
    fn FsSetDeviceSpecific(FileId: ULONG, Specific: PVOID);
    fn DissectArcPath(ArcPath: *const CHAR, Path: *mut *const CHAR, DriveNumber: *mut UCHAR, PartitionNumber: *mut ULONG) -> BOOLEAN;
}

// hwdisk.c:467 `DiskGetBootPath(PxeInit())`. PxeInit lives in pxe.c which
// fs.c gates `#if defined(_M_IX86) || defined(_M_AMD64)` + `#ifndef UEFIBOOT`
// (pxe.rs is correspondingly cfg(any(x86,x86_64)) only; the uefi feature
// mirrors UEFIBOOT). Where PXE is absent (arm64, or the x86 uefildr archive)
// PxeInit is not linkable — booting is never PXE there, so the faithful
// substitute is the constant FALSE that PxeInit would return.
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
extern "C" {
    fn PxeInit() -> BOOLEAN;
}
#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi"))))]
#[inline]
unsafe fn PxeInit() -> BOOLEAN { 0u8 /* FALSE — no PXE on this build */ }

unsafe extern "C" fn DiskClose(FileId: ULONG) -> ARC_STATUS {
    let Context = FsGetDeviceSpecific(FileId);
    FrLdrTempFree(Context as *mut core::ffi::c_void, TAG_HW_DISK_CONTEXT);
    ESUCCESS
}

unsafe extern "C" fn DiskGetFileInformation(FileId: ULONG, Information: *mut FILEINFORMATION) -> ARC_STATUS {

    let Context = FsGetDeviceSpecific(FileId);

    core::ptr::write_bytes(Information, 0, 1);

    (*Information).StartingAddress.QuadPart = ((*Context).SectorOffset * (((*Context).SectorSize) as u64)) as i64;
    (*Information).EndingAddress.QuadPart = (((*Context).SectorOffset + (*Context).SectorCount) * (((*Context).SectorSize) as u64)) as i64;
    (*Information).CurrentAddress.QuadPart = ((*Context).SectorNumber * (((*Context).SectorSize) as u64)) as i64;

    (*Information).Type = if (*Context).IsFloppy != 0 { FloppyDiskPeripheral } else { DiskPeripheral };

    ESUCCESS
}

unsafe extern "C" fn DiskOpen(Path: PCHAR, _OpenMode: OPENMODE, FileId: *mut ULONG) -> ARC_STATUS {
    let Context: *mut DISKCONTEXT;
    let DriveType: CONFIGURATION_TYPE;
    let mut DriveNumber: UCHAR = 0;
    let mut DrivePartition: ULONG = 0;
    let SectorSize: ULONG;
    let mut SectorOffset: ULONGLONG = 0;
    let mut SectorCount: ULONGLONG = 0;
    let mut PartitionTableEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();

    if DiskReadBufferSize == 0 {
        ERR(b"DiskOpen(): DiskReadBufferSize is 0, something is wrong.\n\0".as_ptr() as PCHAR);
        return ENOMEM;
    }

    if DissectArcPath(Path, core::ptr::null_mut(), &mut DriveNumber, &mut DrivePartition) == 0 {
        return EINVAL;
    }

    DriveType = DiskGetConfigType(DriveNumber);
    if DriveType == CdromController {
        SectorSize = 2048;
    } else {
        SectorSize = 512;
    }

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
            ERR(b"SectorSize (%lu) != Geometry.BytesPerSector (%lu), expect problems!\n\0".as_ptr() as PCHAR);
        }

        SectorOffset = 0;
        SectorCount = Geometry.Sectors;
    }

    Context = FrLdrTempAlloc(core::mem::size_of::<DISKCONTEXT>(), TAG_HW_DISK_CONTEXT) as *mut DISKCONTEXT;
    if Context.is_null() {
        return ENOMEM;
    }

    (*Context).DriveNumber = DriveNumber;
    (*Context).IsFloppy = if DriveType == FloppyDiskPeripheral { 1 } else { 0 };
    (*Context).SectorSize = SectorSize;
    (*Context).SectorOffset = SectorOffset;
    (*Context).SectorCount = SectorCount;
    (*Context).SectorNumber = 0;
    FsSetDeviceSpecific(*FileId, Context as *mut core::ffi::c_void);

    return ESUCCESS;
}

unsafe extern "C" fn DiskRead(FileId: ULONG, Buffer: *mut VOID, mut N: ULONG, Count: *mut ULONG) -> ARC_STATUS {
    let Context = FsGetDeviceSpecific(FileId);
    let mut Ptr = Buffer as *mut UCHAR;
    let mut Length: ULONG;
    let mut TotalSectors: ULONG;
    let MaxSectors: ULONG;
    let mut ReadSectors: ULONG;
    let mut SectorOffset: ULONGLONG;
    let mut ret: BOOLEAN;

    TotalSectors = (N + (*Context).SectorSize - 1) / (*Context).SectorSize;
    MaxSectors = (DiskReadBufferSize / (((*Context).SectorSize) as usize)) as u32 ;
    SectorOffset = (*Context).SectorOffset + (*Context).SectorNumber;

    ret = 1u8;

    while TotalSectors != 0 {
        ReadSectors = TotalSectors;
        if ReadSectors > MaxSectors {
            ReadSectors = MaxSectors;
        }

        ret = MachDiskReadLogicalSectors(
            ((*Context).DriveNumber) as u32 ,
            SectorOffset,
            ReadSectors,
            DiskReadBuffer,
        );
        if ret == 0 {
            break;
        }

        Length = ReadSectors * (*Context).SectorSize;
        if Length > N {
            Length = N;
        }

        RtlCopyMemory(Ptr as *mut VOID, DiskReadBuffer as *const VOID, Length as usize);

        Ptr = Ptr.add(Length as usize);
        N -= Length;
        SectorOffset += ReadSectors as ULONGLONG;
        TotalSectors -= ReadSectors;
    }

    *Count = (Ptr as ULONG_PTR - Buffer as ULONG_PTR) as u32 ;
    (*Context).SectorNumber = SectorOffset - (*Context).SectorOffset;

    if ret == 0 { EIO } else { ESUCCESS }
}

unsafe extern "C" fn DiskSeek(FileId: u32, Position: *mut LARGE_INTEGER, SeekMode: u32) -> ARC_STATUS {

    let Context = FsGetDeviceSpecific(FileId);
    let mut NewPosition = *Position;

    if SeekMode == SeekAbsolute {
    } else if SeekMode == SeekRelative {
        NewPosition.QuadPart += ((*Context).SectorNumber * (((*Context).SectorSize) as u64)) as i64;
    } else {
        return EINVAL;
    }

    if (NewPosition.QuadPart as u64 & ((((*Context).SectorSize - 1)) as u64)) != 0 {
        return EINVAL;
    }

    // C uses ULONGLONG arithmetic here. Casting to i64 silently turns BIOS
    // "no LBA limit" sentinel (0xFFFF_FFFF_FFFF_FFFF) into -1, making every
    // valid sector look out-of-bounds. Keep the comparison unsigned.
    let new_sector = (NewPosition.QuadPart as u64) / (*Context).SectorSize as u64;

    if (*Context).SectorCount != 0 && new_sector >= (*Context).SectorCount {
        return EINVAL;
    }

    (*Context).SectorNumber = new_sector;
    ESUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn GetHarddiskIdentifier(DriveNumber: UCHAR) -> PCHAR {
    PcDiskIdentifier[(DriveNumber - FIRST_BIOS_DISK) as usize].as_mut_ptr() as PCHAR
}

unsafe fn GetHarddiskInformation(DriveNumber: u8) {

    let Mbr: *mut MASTER_BOOT_RECORD;
    let Buffer: *mut u32;
    let mut i: u32;
    let mut Checksum: u32;
    let Signature: u32;
    let ValidPartitionTable: u8;
    let mut ArcName: [i8; 260] = [0; 260];
    let mut PartitionTableEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();
    let Identifier: *mut u8 = PcDiskIdentifier[(DriveNumber - FIRST_BIOS_DISK) as usize].as_mut_ptr();

    DiskDetectPartitionType(DriveNumber);

    if MachDiskReadLogicalSectors((DriveNumber) as u32 , 0u64, 1, DiskReadBuffer as *mut core::ffi::c_void) == 0 {
        ERR(b"Reading MBR failed\n\0".as_ptr() as *const i8);
        sprintf(Identifier as *mut i8, b"BIOSDISK%d\0".as_ptr() as *const i8, (DriveNumber - FIRST_BIOS_DISK + 1) as i32);
        return;
    }

    Buffer = DiskReadBuffer as *mut u32;
    Mbr = DiskReadBuffer as *mut MASTER_BOOT_RECORD;

    Signature = (*Mbr).Signature;
    TRACE(b"Signature: %x\n\0".as_ptr() as *const i8, (Signature) as usize);

    Checksum = 0;
    i = 0;
    while i < 512 / 4 {
        Checksum = Checksum.wrapping_add(*Buffer.offset(i as isize));
        i += 1;
    }
    Checksum = (!Checksum).wrapping_add(1);
    TRACE(b"Checksum: %x\n\0".as_ptr() as *const i8, (Checksum) as usize);

    ValidPartitionTable = if (*Mbr).MasterBootRecordMagic == 0xAA55 { 1 } else { 0 };

    sprintf(ArcName.as_mut_ptr(), b"multi(0)disk(0)rdisk(%u)\0".as_ptr() as *const i8, (DriveNumber - FIRST_BIOS_DISK) as u32);
    AddReactOSArcDiskInfo(ArcName.as_ptr(), core::ptr::null(), Signature, Checksum, ValidPartitionTable);

    sprintf(ArcName.as_mut_ptr(), b"multi(0)disk(0)rdisk(%u)partition(0)\0".as_ptr() as *const i8, (DriveNumber - FIRST_BIOS_DISK) as u32);
    FsRegisterDevice(ArcName.as_ptr(), core::ptr::addr_of!(DiskVtbl) as *const core::ffi::c_void);

    i = FIRST_PARTITION;
    DiskReportError(0);
    while DiskGetPartitionEntry(DriveNumber, i, core::ptr::addr_of_mut!(PartitionTableEntry)) != 0 {
        if PartitionTableEntry.SystemIndicator != PARTITION_ENTRY_UNUSED {
            sprintf(ArcName.as_mut_ptr(), b"multi(0)disk(0)rdisk(%u)partition(%lu)\0".as_ptr() as *const i8, (DriveNumber - FIRST_BIOS_DISK) as u32, i);
            FsRegisterDevice(ArcName.as_ptr(), core::ptr::addr_of!(DiskVtbl) as *const core::ffi::c_void);
        }
        i += 1;
    }
    DiskReportError(1);

    *Identifier.offset(0) = Hex[((Checksum >> 28) & 0x0F) as usize];
    *Identifier.offset(1) = Hex[((Checksum >> 24) & 0x0F) as usize];
    *Identifier.offset(2) = Hex[((Checksum >> 20) & 0x0F) as usize];
    *Identifier.offset(3) = Hex[((Checksum >> 16) & 0x0F) as usize];
    *Identifier.offset(4) = Hex[((Checksum >> 12) & 0x0F) as usize];
    *Identifier.offset(5) = Hex[((Checksum >> 8) & 0x0F) as usize];
    *Identifier.offset(6) = Hex[((Checksum >> 4) & 0x0F) as usize];
    *Identifier.offset(7) = Hex[(Checksum & 0x0F) as usize];
    *Identifier.offset(8) = b'-';
    *Identifier.offset(9) = Hex[((Signature >> 28) & 0x0F) as usize];
    *Identifier.offset(10) = Hex[((Signature >> 24) & 0x0F) as usize];
    *Identifier.offset(11) = Hex[((Signature >> 20) & 0x0F) as usize];
    *Identifier.offset(12) = Hex[((Signature >> 16) & 0x0F) as usize];
    *Identifier.offset(13) = Hex[((Signature >> 12) & 0x0F) as usize];
    *Identifier.offset(14) = Hex[((Signature >> 8) & 0x0F) as usize];
    *Identifier.offset(15) = Hex[((Signature >> 4) & 0x0F) as usize];
    *Identifier.offset(16) = Hex[(Signature & 0x0F) as usize];
    *Identifier.offset(17) = b'-';
    *Identifier.offset(18) = if ValidPartitionTable != 0 { b'A' } else { b'X' };
    *Identifier.offset(19) = 0;
    TRACE(b"Identifier: %s\n\0".as_ptr() as *const i8, (Identifier) as usize);
}

unsafe fn EnumerateHarddisks(BootDriveReported: PBOOLEAN) -> u8 {

    let mut DiskCount: u8;
    let mut DriveNumber: u8;
    let _i: u32;
    let _Changed: u8;

    *BootDriveReported = 0u8;

    DiskReportError(0u8);
    DiskCount = 0;
    DriveNumber = FIRST_BIOS_DISK;

    memset(DiskReadBuffer as *mut core::ffi::c_void, 0xcd, DiskReadBufferSize);
    while MachDiskReadLogicalSectors((DriveNumber) as u32 , 0u64, 1, DiskReadBuffer as *mut core::ffi::c_void) != 0 {
        let mut Changed = 0u8;
        let mut i = 0u32;
        while Changed == 0 && i < DiskReadBufferSize as u32 {
            if *((DiskReadBuffer as *const u8).add(i as usize)) != 0xcd {
                Changed = 1u8;
            }
            i += 1;
        }
        if Changed == 0 {
            TRACE(b"BIOS reports success for disk %d (0x%02X) but data didn't change\n\0".as_ptr() as *const i8, (DiskCount as i32) as usize, (DriveNumber as i32) as usize);
            break;
        }

        GetHarddiskInformation(DriveNumber);

        if FrldrBootDrive == DriveNumber {
            *BootDriveReported = 1u8;
        }

        // C `DiskCount++`: a shadowing `let` here left the count at 0, so
        // PcBiosDiskCount registered no INT13 drives in the hardware tree.
        DiskCount += 1;
        DriveNumber += 1;
        memset(DiskReadBuffer as *mut core::ffi::c_void, 0xcd, DiskReadBufferSize);
    }
    DiskReportError(1u8);

    PcBiosDiskCount = DiskCount;
    TRACE(b"BIOS reports %d harddisk%s\n\0".as_ptr() as *const i8, (DiskCount as i32) as usize, (if DiskCount == 1 { b"\0".as_ptr() } else { b"s\0".as_ptr() }) as usize);

    DiskCount
}

unsafe fn DiskGetBootPath(IsPxe: BOOLEAN) -> BOOLEAN {

    if *FrLdrBootPath.as_ptr() != 0 {
        return 1u8;
    }

    if IsPxe != 0 {
        RtlStringCbCopyA(
            FrLdrBootPath.as_mut_ptr(),
            core::mem::size_of_val(&FrLdrBootPath),
            b"net(0)\0".as_ptr() as *const i8,
        );
    } else if FrldrBootDrive == 0x49 {
        RtlStringCbCopyA(
            FrLdrBootPath.as_mut_ptr(),
            core::mem::size_of_val(&FrLdrBootPath),
            b"ramdisk(0)\0".as_ptr() as *const i8,
        );
    } else if FrldrBootDrive < FIRST_BIOS_DISK {
        RtlStringCbPrintfA(
            FrLdrBootPath.as_mut_ptr(),
            core::mem::size_of_val(&FrLdrBootPath),
            b"multi(0)disk(0)fdisk(%u)\0".as_ptr() as *const i8,
            FrldrBootDrive as u32,
        );
    } else if FrldrBootPartition == 0xFF {
        RtlStringCbPrintfA(
            FrLdrBootPath.as_mut_ptr(),
            core::mem::size_of_val(&FrLdrBootPath),
            b"multi(0)disk(0)cdrom(%u)\0".as_ptr() as *const i8,
            (FrldrBootDrive - FIRST_BIOS_DISK) as u32,
        );
    } else {
        let mut BootPartition: ULONG = 0;
        let mut PartitionEntry: PARTITION_TABLE_ENTRY = core::mem::zeroed();

        if DiskGetBootPartitionEntry((FrldrBootDrive) as u32 , &mut PartitionEntry, &mut BootPartition) == 0 {
            ERR(b"Failed to get boot partition entry\n\0".as_ptr() as *const i8);
            return 0u8;
        }

        FrldrBootPartition = BootPartition;

        RtlStringCbPrintfA(
            FrLdrBootPath.as_mut_ptr(),
            core::mem::size_of_val(&FrLdrBootPath),
            b"multi(0)disk(0)rdisk(%u)partition(%lu)\0".as_ptr() as *const i8,
            (FrldrBootDrive - FIRST_BIOS_DISK) as u32,
            FrldrBootPartition,
        );
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn PcInitializeBootDevices() -> BOOLEAN {
    crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D');

    let mut DiskCount: UCHAR;
    let mut BootDriveReported: BOOLEAN = 0u8;
    let DriveType: CONFIGURATION_TYPE;

    DiskCount = EnumerateHarddisks(&mut BootDriveReported);
    crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'2');

    DiskGetBootPath(PxeInit());
    crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'3');

    DriveType = DiskGetConfigType(FrldrBootDrive);
    crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'4');
    if (FrldrBootDrive >= FIRST_BIOS_DISK && BootDriveReported == 0u8) ||
       (DriveType == FloppyDiskPeripheral || DriveType == CdromController)
    {
        crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'5');
        let Mbr: PMASTER_BOOT_RECORD;
        let Buffer: PULONG;
        let mut Checksum: ULONG = 0;
        let Signature: ULONG;
        let mut i: ULONG;

        if MachDiskReadLogicalSectors((FrldrBootDrive) as u32 , 16u64, 1, DiskReadBuffer as *mut core::ffi::c_void) == 0u8 {
            ERR(b"Reading MBR failed\n\0".as_ptr() as *const i8);
            return 0u8;
        }
        crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'6');

        Buffer = DiskReadBuffer as PULONG;
        Mbr = DiskReadBuffer as PMASTER_BOOT_RECORD;

        Signature = (*Mbr).Signature;
        TRACE(b"Signature: %x\n\0".as_ptr() as *const i8, (Signature) as usize);

        i = 0;
        while i < 2048 / core::mem::size_of::<ULONG>() as ULONG {
            Checksum = Checksum.wrapping_add(*Buffer.offset(i as isize));
            i += 1;
        }
        Checksum = (!Checksum).wrapping_add(1);
        TRACE(b"Checksum: %x\n\0".as_ptr() as *const i8, (Checksum) as usize);
        crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'7');

        AddReactOSArcDiskInfo(FrLdrBootPath.as_ptr() as PCHAR, core::ptr::null(), Signature, Checksum, 1u8);
        crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'8');

        FsRegisterDevice(FrLdrBootPath.as_ptr() as PCHAR, &DiskVtbl as *const _ as *const core::ffi::c_void);
        crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'9');
        DiskCount = DiskCount.wrapping_add(1);
        TRACE(b"Additional boot drive detected: 0x%02X\n\0".as_ptr() as *const i8, (FrldrBootDrive as i32) as usize);
    }
    crate::frldr::dbgcon(b'B'); crate::frldr::dbgcon(b'D'); crate::frldr::dbgcon(b'Z');

    if DiskCount != 0 { 1u8 } else { 0u8 }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *
//  *  Copyright (C) 2003, 2004  Eric Kohl
//  *  Copyright (C) 2009  Hervé Poussineau
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
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(HWDETECT);
//
// /*
//  * This is the common code for harddisk for both the PC and the XBOX.
//  */
//
// #define FIRST_BIOS_DISK 0x80
// #define FIRST_PARTITION 1
//
// typedef struct tagDISKCONTEXT
// {
//     UCHAR DriveNumber;
//     BOOLEAN IsFloppy;
//     ULONG SectorSize;
//     ULONGLONG SectorOffset;
//     ULONGLONG SectorCount;
//     ULONGLONG SectorNumber;
// } DISKCONTEXT;
//
// static const CHAR Hex[] = "0123456789abcdef";
//
// /* Data cache for BIOS disks pre-enumeration */
// UCHAR PcBiosDiskCount = 0;
// static CHAR PcDiskIdentifier[32][20];
//
// PVOID DiskReadBuffer;
// SIZE_T DiskReadBufferSize;
//
//
// /* FUNCTIONS *****************************************************************/
//
// static ARC_STATUS
// DiskClose(ULONG FileId)
// {
//     DISKCONTEXT* Context = FsGetDeviceSpecific(FileId);
//     FrLdrTempFree(Context as *mut core::ffi::c_void, TAG_HW_DISK_CONTEXT);
//     return ESUCCESS;
// }
//
// static ARC_STATUS
// DiskGetFileInformation(ULONG FileId, FILEINFORMATION* Information)
// {
//     DISKCONTEXT* Context = FsGetDeviceSpecific(FileId);
//
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
//     Information->Type = (Context->IsFloppy ? FloppyDiskPeripheral : DiskPeripheral);
//
//     return ESUCCESS;
// }
//
// static ARC_STATUS
// DiskOpen(CHAR* Path, OPENMODE OpenMode, ULONG* FileId)
// {
//     DISKCONTEXT* Context;
//     CONFIGURATION_TYPE DriveType;
//     UCHAR DriveNumber;
//     ULONG DrivePartition, SectorSize;
//     ULONGLONG SectorOffset = 0;
//     ULONGLONG SectorCount = 0;
//     PARTITION_TABLE_ENTRY PartitionTableEntry;
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
//     DriveType = DiskGetConfigType(DriveNumber);
//     if (DriveType == CdromController)
//     {
//         /* This is a CD-ROM device */
//         SectorSize = 2048;
//     }
//     else
//     {
//         /* This is either a floppy disk or a hard disk device, but it doesn't
//          * matter which one because they both have 512 bytes per sector */
//         SectorSize = 512;
//     }
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
//     Context->IsFloppy = (DriveType == FloppyDiskPeripheral);
//     Context->SectorSize = SectorSize;
//     Context->SectorOffset = SectorOffset;
//     Context->SectorCount = SectorCount;
//     Context->SectorNumber = 0;
//     FsSetDeviceSpecific(*FileId, Context);
//
//     return ESUCCESS;
// }
//
// static ARC_STATUS
// DiskRead(ULONG FileId, VOID* Buffer, ULONG N, ULONG* Count)
// {
//     DISKCONTEXT* Context = FsGetDeviceSpecific(FileId);
//     UCHAR* Ptr = (UCHAR*)Buffer;
//     ULONG Length, TotalSectors, MaxSectors, ReadSectors;
//     ULONGLONG SectorOffset;
//     BOOLEAN ret;
//
//     ASSERT(DiskReadBufferSize > 0);
//
//     TotalSectors = (N + Context->SectorSize - 1) / Context->SectorSize;
//     MaxSectors   = DiskReadBufferSize / Context->SectorSize;
//     SectorOffset = Context->SectorOffset + Context->SectorNumber;
//
//     // If MaxSectors is 0, this will lead to infinite loop.
//     // In release builds assertions are disabled, however we also have sanity checks in DiskOpen()
//     ASSERT(MaxSectors > 0);
//
//     ret = TRUE;
//
//     while (TotalSectors)
//     {
//         ReadSectors = TotalSectors;
//         if (ReadSectors > MaxSectors)
//             ReadSectors = MaxSectors;
//
//         ret = MachDiskReadLogicalSectors(Context->DriveNumber,
//                                          SectorOffset,
//                                          ReadSectors,
//                                          DiskReadBuffer);
//         if (!ret)
//             break;
//
//         Length = ReadSectors * Context->SectorSize;
//         if (Length > N)
//             Length = N;
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
//     return (!ret) ? EIO : ESUCCESS;
// }
//
// static ARC_STATUS
// DiskSeek(ULONG FileId, LARGE_INTEGER* Position, SEEKMODE SeekMode)
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
// static const DEVVTBL DiskVtbl =
// {
//     DiskClose,
//     DiskGetFileInformation,
//     DiskOpen,
//     DiskRead,
//     DiskSeek,
// };
//
//
// PCHAR
// GetHarddiskIdentifier(UCHAR DriveNumber)
// {
//     return PcDiskIdentifier[DriveNumber - FIRST_BIOS_DISK];
// }
//
// static VOID
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
//     PCHAR Identifier = PcDiskIdentifier[DriveNumber - FIRST_BIOS_DISK];
//
//     /* Detect disk partition type */
//     DiskDetectPartitionType(DriveNumber);
//
//     /* Read the MBR */
//     if (!MachDiskReadLogicalSectors(DriveNumber, 0ULL, 1, DiskReadBuffer))
//     {
//         ERR("Reading MBR failed\n");
//         /* We failed, use a default identifier */
//         sprintf(Identifier, "BIOSDISK%d", DriveNumber - FIRST_BIOS_DISK + 1);
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
//     sprintf(ArcName, "multi(0)disk(0)rdisk(%u)", DriveNumber - FIRST_BIOS_DISK);
//     AddReactOSArcDiskInfo(ArcName, Signature, Checksum, ValidPartitionTable);
//
//     sprintf(ArcName, "multi(0)disk(0)rdisk(%u)partition(0)", DriveNumber - FIRST_BIOS_DISK);
//     FsRegisterDevice(ArcName, &DiskVtbl);
//
//     /* Add partitions */
//     i = FIRST_PARTITION;
//     DiskReportError(FALSE);
//     while (DiskGetPartitionEntry(DriveNumber, i, &PartitionTableEntry))
//     {
//         if (PartitionTableEntry.SystemIndicator != PARTITION_ENTRY_UNUSED)
//         {
//             sprintf(ArcName, "multi(0)disk(0)rdisk(%u)partition(%lu)", DriveNumber - FIRST_BIOS_DISK, i);
//             FsRegisterDevice(ArcName, &DiskVtbl);
//         }
//         i++;
//     }
//     DiskReportError(TRUE);
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
// static UCHAR
// EnumerateHarddisks(OUT PBOOLEAN BootDriveReported)
// {
//     UCHAR DiskCount, DriveNumber;
//     ULONG i;
//     BOOLEAN Changed;
//
//     *BootDriveReported = FALSE;
//
//     /* Count the number of visible harddisk drives */
//     DiskReportError(FALSE);
//     DiskCount = 0;
//     DriveNumber = FIRST_BIOS_DISK;
//
//     ASSERT(DiskReadBufferSize > 0);
//
//     /*
//      * There are some really broken BIOSes out there. There are even BIOSes
//      * that happily report success when you ask them to read from non-existent
//      * harddisks. So, we set the buffer to known contents first, then try to
//      * read. If the BIOS reports success but the buffer contents haven't
//      * changed then we fail anyway.
//      */
//     memset(DiskReadBuffer, 0xcd, DiskReadBufferSize);
//     while (MachDiskReadLogicalSectors(DriveNumber, 0ULL, 1, DiskReadBuffer))
//     {
//         Changed = FALSE;
//         for (i = 0; !Changed && i < DiskReadBufferSize; i++)
//         {
//             Changed = ((PUCHAR)DiskReadBuffer)[i] != 0xcd;
//         }
//         if (!Changed)
//         {
//             TRACE("BIOS reports success for disk %d (0x%02X) but data didn't change\n", (//                   (int)DiskCount) as usize, (DriveNumber) as usize);
//             break;
//         }
//
//         /* Cache the BIOS hard disk information for later use */
//         GetHarddiskInformation(DriveNumber);
//
//         /* Check if we have seen the boot drive */
//         if (FrldrBootDrive == DriveNumber)
//             *BootDriveReported = TRUE;
//
//         DiskCount++;
//         DriveNumber++;
//         memset(DiskReadBuffer, 0xcd, DiskReadBufferSize);
//     }
//     DiskReportError(TRUE);
//
//     PcBiosDiskCount = DiskCount;
//     TRACE("BIOS reports %d harddisk%s\n", (//           (int)DiskCount) as usize, ((DiskCount == 1) ? "" : "s") as usize);
//
//     return DiskCount;
// }
//
// static BOOLEAN
// DiskGetBootPath(BOOLEAN IsPxe)
// {
//     if (*FrLdrBootPath)
//         return TRUE;
//
//     // FIXME! FIXME! Do this in some drive recognition procedure!!!!
//     if (IsPxe)
//     {
//         RtlStringCbCopyA(FrLdrBootPath, sizeof(FrLdrBootPath), "net(0)");
//     }
//     else
//     /* 0x49 is our magic ramdisk drive, so try to detect it first */
//     if (FrldrBootDrive == 0x49)
//     {
//         /* This is the ramdisk. See ArmInitializeBootDevices() too... */
//         // RtlStringCbPrintfA(FrLdrBootPath, sizeof(FrLdrBootPath), "ramdisk(%u)", 0);
//         RtlStringCbCopyA(FrLdrBootPath, sizeof(FrLdrBootPath), "ramdisk(0)");
//     }
//     else if (FrldrBootDrive < FIRST_BIOS_DISK)
//     {
//         /* This is a floppy */
//         RtlStringCbPrintfA(FrLdrBootPath, sizeof(FrLdrBootPath),
//                            "multi(0)disk(0)fdisk(%u)", FrldrBootDrive);
//     }
//     else if (FrldrBootPartition == 0xFF)
//     {
//         /* Boot Partition 0xFF is the magic value that indicates booting from CD-ROM (see isoboot.S) */
//         RtlStringCbPrintfA(FrLdrBootPath, sizeof(FrLdrBootPath),
//                            "multi(0)disk(0)cdrom(%u)", FrldrBootDrive - FIRST_BIOS_DISK);
//     }
//     else
//     {
//         ULONG BootPartition;
//         PARTITION_TABLE_ENTRY PartitionEntry;
//
//         /* This is a hard disk */
//         if (!DiskGetBootPartitionEntry(FrldrBootDrive, &PartitionEntry, &BootPartition))
//         {
//             ERR("Failed to get boot partition entry\n");
//             return FALSE;
//         }
//
//         FrldrBootPartition = BootPartition;
//
//         RtlStringCbPrintfA(FrLdrBootPath, sizeof(FrLdrBootPath),
//                            "multi(0)disk(0)rdisk(%u)partition(%lu)",
//                            FrldrBootDrive - FIRST_BIOS_DISK, FrldrBootPartition);
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// PcInitializeBootDevices(VOID)
// {
//     UCHAR DiskCount;
//     BOOLEAN BootDriveReported = FALSE;
//     CONFIGURATION_TYPE DriveType;
//
//     DiskCount = EnumerateHarddisks(&BootDriveReported);
//
//     /* Initialize FrLdrBootPath, the boot path we're booting from (the "SystemPartition") */
//     DiskGetBootPath(PxeInit());
//
//     /* Add it, if it's a floppy or CD-ROM */
//     DriveType = DiskGetConfigType(FrldrBootDrive);
//     if ((FrldrBootDrive >= FIRST_BIOS_DISK && !BootDriveReported) ||
//         (DriveType == FloppyDiskPeripheral || DriveType == CdromController))
//     {
//         /* TODO: Check if it's really a CD-ROM drive */
//
//         PMASTER_BOOT_RECORD Mbr;
//         PULONG Buffer;
//         ULONG Checksum = 0;
//         ULONG Signature;
//         ULONG i;
//
//         /* Read the MBR */
//         if (!MachDiskReadLogicalSectors(FrldrBootDrive, 16ULL, 1, DiskReadBuffer))
//         {
//             ERR("Reading MBR failed\n");
//             return FALSE;
//         }
//
//         Buffer = (ULONG*)DiskReadBuffer;
//         Mbr = (PMASTER_BOOT_RECORD)DiskReadBuffer;
//
//         Signature = Mbr->Signature;
//         TRACE("Signature: %x\n", (Signature) as usize);
//
//         /* Calculate the MBR checksum */
//         for (i = 0; i < 2048 / sizeof(ULONG); i++)
//         {
//             Checksum += Buffer[i];
//         }
//         Checksum = ~Checksum + 1;
//         TRACE("Checksum: %x\n", (Checksum) as usize);
//
//         /* Fill out the ARC disk block */
//         AddReactOSArcDiskInfo(FrLdrBootPath, Signature, Checksum, TRUE);
//
//         FsRegisterDevice(FrLdrBootPath, &DiskVtbl);
//         DiskCount++; // This is not accounted for in the number of pre-enumerated BIOS drives!
//         TRACE("Additional boot drive detected: 0x%02X\n", ((int)FrldrBootDrive) as usize);
//     }
//
//     return (DiskCount != 0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_FILESYSTEM; // fat.c: DBG_DEFAULT_CHANNEL(FILESYSTEM)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER, LIST_ENTRY, PCSTR, PUINT32, UINT32, UINT8, ULARGE_INTEGER};
// ADDED 2026-07-05: patterns below were unresolved names -> silent bind-all
// catch-alls (bind-all match-arm bug class); import the real ddk consts.
use ntkrnl_sys::ddk::{_SEEKMODE_SeekAbsolute, _SEEKMODE_SeekRelative};
use core::ffi::c_void;

// ---- restored from fat.h / fat.c (rustconv drops non-fn header content) ----
pub const TAG_FAT_CHAIN: u32 = u32::from_be_bytes(*b"CtaT");
pub const TAG_FAT_FILE: u32 = u32::from_be_bytes(*b"FtaF");
pub const TAG_FAT_VOLUME: u32 = u32::from_be_bytes(*b"VtaF");
pub const TAG_FAT_BUFFER: u32 = u32::from_be_bytes(*b"BtaF");
pub const TAG_FAT_CACHE: u32 = u32::from_be_bytes(*b"HtaF");
pub const SECTORSIZE: ULONG = 512;
pub const FAT_ATTR_NORMAL: u8 = 0x00;
pub const FAT_ATTR_READONLY: u8 = 0x01;
pub const FAT_ATTR_HIDDEN: u8 = 0x02;
pub const FAT_ATTR_SYSTEM: u8 = 0x04;
pub const FAT_ATTR_VOLUMENAME: u8 = 0x08;
pub const FAT_ATTR_DIRECTORY: u8 = 0x10;
pub const FAT_ATTR_ARCHIVE: u8 = 0x20;
pub const FAT_ATTR_LONG_NAME: u8 =
    FAT_ATTR_READONLY | FAT_ATTR_HIDDEN | FAT_ATTR_SYSTEM | FAT_ATTR_VOLUMENAME;
pub const FAT12: ULONG = 1;
pub const FAT16: ULONG = 2;
pub const FAT32: ULONG = 3;
pub const FATX16: ULONG = 4;
pub const FATX32: ULONG = 5;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct _FAT_BOOTSECTOR {
    pub JumpBoot: [UCHAR; 3],
    pub OemName: [CHAR; 8],
    pub BytesPerSector: USHORT,
    pub SectorsPerCluster: UCHAR,
    pub ReservedSectors: USHORT,
    pub NumberOfFats: UCHAR,
    pub RootDirEntries: USHORT,
    pub TotalSectors: USHORT,
    pub MediaDescriptor: UCHAR,
    pub SectorsPerFat: USHORT,
    pub SectorsPerTrack: USHORT,
    pub NumberOfHeads: USHORT,
    pub HiddenSectors: ULONG,
    pub TotalSectorsBig: ULONG,
    pub DriveNumber: UCHAR,
    pub Reserved1: UCHAR,
    pub BootSignature: UCHAR,
    pub VolumeSerialNumber: ULONG,
    pub VolumeLabel: [CHAR; 11],
    pub FileSystemType: [CHAR; 8],
    pub BootCodeAndData: [UCHAR; 448],
    pub BootSectorMagic: USHORT,
}
pub type FAT_BOOTSECTOR = _FAT_BOOTSECTOR;
pub type PFAT_BOOTSECTOR = *mut _FAT_BOOTSECTOR;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct _FAT32_BOOTSECTOR {
    pub JumpBoot: [UCHAR; 3],
    pub OemName: [CHAR; 8],
    pub BytesPerSector: USHORT,
    pub SectorsPerCluster: UCHAR,
    pub ReservedSectors: USHORT,
    pub NumberOfFats: UCHAR,
    pub RootDirEntries: USHORT,
    pub TotalSectors: USHORT,
    pub MediaDescriptor: UCHAR,
    pub SectorsPerFat: USHORT,
    pub SectorsPerTrack: USHORT,
    pub NumberOfHeads: USHORT,
    pub HiddenSectors: ULONG,
    pub TotalSectorsBig: ULONG,
    pub SectorsPerFatBig: ULONG,
    pub ExtendedFlags: USHORT,
    pub FileSystemVersion: USHORT,
    pub RootDirStartCluster: ULONG,
    pub FsInfo: USHORT,
    pub BackupBootSector: USHORT,
    pub Reserved: [UCHAR; 12],
    pub DriveNumber: UCHAR,
    pub Reserved1: UCHAR,
    pub BootSignature: UCHAR,
    pub VolumeSerialNumber: ULONG,
    pub VolumeLabel: [CHAR; 11],
    pub FileSystemType: [CHAR; 8],
    pub BootCodeAndData: [UCHAR; 420],
    pub BootSectorMagic: USHORT,
}
pub type FAT32_BOOTSECTOR = _FAT32_BOOTSECTOR;
pub type PFAT32_BOOTSECTOR = *mut _FAT32_BOOTSECTOR;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct _FATX_BOOTSECTOR {
    pub FileSystemType: [CHAR; 4],
    pub VolumeSerialNumber: ULONG,
    pub SectorsPerCluster: ULONG,
    pub NumberOfFats: USHORT,
    pub Unknown: ULONG,
    pub Unused: [UCHAR; 494],
}
pub type FATX_BOOTSECTOR = _FATX_BOOTSECTOR;
pub type PFATX_BOOTSECTOR = *mut _FATX_BOOTSECTOR;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct DIRENTRY {
    pub FileName: [CHAR; 11],
    pub Attr: UCHAR,
    pub ReservedNT: UCHAR,
    pub TimeInTenths: UCHAR,
    pub CreateTime: USHORT,
    pub CreateDate: USHORT,
    pub LastAccessDate: USHORT,
    pub ClusterHigh: USHORT,
    pub Time: USHORT,
    pub Date: USHORT,
    pub ClusterLow: USHORT,
    pub Size: ULONG,
}
pub type PDIRENTRY = *mut DIRENTRY;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct LFN_DIRENTRY {
    pub SequenceNumber: UCHAR,
    pub Name0_4: [u16; 5],
    pub EntryAttributes: UCHAR,
    pub Reserved: UCHAR,
    pub AliasChecksum: UCHAR,
    pub Name5_10: [u16; 6],
    pub StartCluster: USHORT,
    pub Name11_12: [u16; 2],
}
pub type PLFN_DIRENTRY = *mut LFN_DIRENTRY;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct FATX_DIRENTRY {
    pub FileNameSize: UCHAR,
    pub Attr: UCHAR,
    pub FileName: [CHAR; 42],
    pub StartCluster: ULONG,
    pub Size: ULONG,
    pub Time: USHORT,
    pub Date: USHORT,
    pub CreateTime: USHORT,
    pub CreateDate: USHORT,
    pub LastAccessTime: USHORT,
    pub LastAccessDate: USHORT,
}
pub type PFATX_DIRENTRY = *mut FATX_DIRENTRY;

#[repr(C)]
pub struct _FAT_VOLUME_INFO {
    pub FatCache: PUCHAR,
    pub FatCacheIndex: PULONG,
    pub FatCacheSize: ULONG,
    pub FatSectorStart: ULONG,
    pub ActiveFatSectorStart: ULONG,
    pub SectorsPerFat: ULONG,
    pub RootDirSectorStart: ULONG,
    pub RootDirSectors: ULONG,
    pub RootDirStartCluster: ULONG,
    pub DataSectorStart: ULONG,
    pub DeviceId: ULONG,
    pub TotalSectors: ULONG,
    pub BytesPerSector: UINT16,
    pub SectorsPerCluster: UINT8,
    pub NumberOfFats: UINT8,
    pub FatType: UINT8,
}
pub type FAT_VOLUME_INFO = _FAT_VOLUME_INFO;
pub type PFAT_VOLUME_INFO = *mut _FAT_VOLUME_INFO;

#[repr(C)]
pub struct _FAT_FILE_INFO {
    pub Volume: PFAT_VOLUME_INFO,
    pub FileSize: ULONG,
    pub FilePointer: ULONG,
    pub CurrentCluster: ULONG,
    pub StartCluster: ULONG,
    pub FileNameLength: ULONG,
    pub Attributes: UCHAR,
    pub FileName: [CHAR; 32],
}
pub type FAT_FILE_INFO = _FAT_FILE_INFO;
pub type PFAT_FILE_INFO = *mut _FAT_FILE_INFO;

#[repr(C)]
pub struct _DIRECTORY_BUFFER {
    pub Link: LIST_ENTRY,
    pub Volume: PVOID,
    pub DirectoryStartCluster: ULONG,
    pub DirectorySize: ULONG,
    pub Data: [UCHAR; 0],
}
pub type DIRECTORY_BUFFER = _DIRECTORY_BUFFER;
pub type PDIRECTORY_BUFFER = *mut _DIRECTORY_BUFFER;

#[no_mangle]
pub static mut FatVolumes: [PFAT_VOLUME_INFO; MAX_FDS] = [core::ptr::null_mut(); MAX_FDS];
static mut DirectoryBufferListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
pub const FAT_MAX_CACHE_SIZE: ULONG = 256 * 1024;
pub const SECTOR_SIZE: ULONG = SECTORSIZE;

// fat.c: const DEVVTBL Fat{,X}FuncTable (dropped by translator).
#[no_mangle]
pub static FatFuncTable: DEVVTBL = DEVVTBL {
    Close: Some(FatClose),
    GetFileInformation: Some(FatGetFileInformation),
    Open: Some(FatOpen),
    Read: Some(FatRead),
    Seek: Some(FatSeek),
    ServiceName: crate::wide!("fastfat\0"),
};
#[no_mangle]
pub static FatXFuncTable: DEVVTBL = DEVVTBL {
    Close: Some(FatClose),
    GetFileInformation: Some(FatGetFileInformation),
    Open: Some(FatOpen),
    Read: Some(FatRead),
    Seek: Some(FatSeek),
    ServiceName: crate::wide!("vfatfs\0"),
};
// ---- end restored ----

// fat.h SW/SD/SWAPW/SWAPD are big-endian byte-swap macros. FAT on-disk is
// little-endian and the host (i386/amd64/arm64-LE) is little-endian, so they
// are no-ops here. (rustconv mis-modeled them as extern fns with a single
// concrete bootsector ptr type; really they're type-generic in-place swaps.)
#[inline(always)]
fn SW<T, V>(_obj: *mut T, _field: V) {}
#[inline(always)]
fn SD<T, V>(_obj: *mut T, _field: V) {}
#[inline(always)]
fn SWAPW(val: u16) -> u16 { val }
#[inline(always)]
fn SWAPD(x: ULONG) -> ULONG { x }

extern "C" {
    fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn FsGetDeviceSpecific(FileId: ULONG) -> PFAT_FILE_INFO;
    fn _strnicmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn _stricmp(s1: *const i8, s2: *const i8) -> i32;
    fn sprintf(s: *mut i8, format: *const i8, ...) -> i32;
    fn FileSystemError(msg: PCHAR);
    fn ArcSeek(DeviceId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: ULONG) -> ARC_STATUS;
    fn ArcRead(DeviceId: ULONG, Buffer: PVOID, Length: ULONG, Count: *mut ULONG) -> ARC_STATUS;
    fn RtlEqualMemory(Source1: *const VOID, Source2: *const VOID, Length: usize) -> BOOLEAN;
    fn ArcGetFileInformation(DeviceId: ULONG, FileInformation: *mut FILEINFORMATION) -> ARC_STATUS;
    fn FsGetNumPathParts(path: PCSTR) -> ULONG;
    fn FsGetFirstNameFromPath(dest: PCHAR, src: PCSTR);
    fn FsGetDeviceId(fileid: ULONG) -> ULONG;
    fn FsSetDeviceSpecific(fileid: ULONG, specific: *mut core::ffi::c_void);
}

// fat.h:161 `#define ISFATX(FT) ((FT)==FATX16 || (FT)==FATX32)` — pure
// value macro; rustconv emitted it as `extern { fn ISFATX }`. Local fn
// (only fat.rs uses it); removing the extern decl drops the bogus
// symbol reference entirely.
#[inline]
unsafe fn ISFATX(fat_type: ULONG) -> BOOLEAN {
    (fat_type == FATX16 || fat_type == FATX32) as BOOLEAN
}

// fat.c:25 `#define FAT_IS_END_CLUSTER(clnumber)` — a C macro that
// implicitly captures `Volume` from the enclosing scope (NOT a macro
// parameter). rustconv mistranslated it as `extern { fn
// FAT_IS_END_CLUSTER(Cluster) }`, dropping the `Volume` dependency.
// Faithful local fn taking the volume explicitly; call sites updated to
// pass their in-scope `Volume`.
#[inline]
unsafe fn FAT_IS_END_CLUSTER(Volume: PFAT_VOLUME_INFO, clnumber: UINT32) -> BOOLEAN {
    let ft = (*Volume).FatType as ULONG;
    (((ft == FAT12) && (clnumber >= 0xff8))
        || ((ft == FAT16 || ft == FATX16) && (clnumber >= 0xfff8))
        || ((ft == FAT32 || ft == FATX32) && (clnumber >= 0x0fff_fff8))) as BOOLEAN
}

#[inline(always)]

unsafe fn FatSwapFatBootSector(Obj: PFAT_BOOTSECTOR) {
    
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, BytesPerSector));
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, ReservedSectors));
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, RootDirEntries));
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, TotalSectors));
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, SectorsPerFat));
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, SectorsPerTrack));
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, NumberOfHeads));
    SD(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, HiddenSectors));
    SD(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, TotalSectorsBig));
    SD(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, VolumeSerialNumber));
    SW(Obj, core::mem::offset_of!(_FAT_BOOTSECTOR, BootSectorMagic));
}

unsafe fn FatSwapFat32BootSector(Obj: PFAT32_BOOTSECTOR) {
    
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, BytesPerSector));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, ReservedSectors));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, RootDirEntries));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, TotalSectors));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, SectorsPerFat));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, NumberOfHeads));
    SD(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, HiddenSectors));
    SD(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, TotalSectorsBig));
    SD(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, SectorsPerFatBig));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, ExtendedFlags));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, FileSystemVersion));
    SD(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, RootDirStartCluster));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, FsInfo));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, BackupBootSector));
    SD(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, VolumeSerialNumber));
    SW(Obj, core::mem::offset_of!(_FAT32_BOOTSECTOR, BootSectorMagic));
}

unsafe fn FatSwapFatXBootSector(Obj: PFATX_BOOTSECTOR) {
    
    SD(Obj, core::mem::offset_of!(_FATX_BOOTSECTOR, VolumeSerialNumber));
    SD(Obj, core::mem::offset_of!(_FATX_BOOTSECTOR, SectorsPerCluster));
    SW(Obj, core::mem::offset_of!(_FATX_BOOTSECTOR, NumberOfFats));
}

unsafe fn FatSwapDirEntry(Obj: PDIRENTRY) {
    
    SW(Obj, (*Obj).CreateTime);
    SW(Obj, (*Obj).CreateDate);
    SW(Obj, (*Obj).LastAccessDate);
    SW(Obj, (*Obj).ClusterHigh);
    SW(Obj, (*Obj).Time);
    SW(Obj, (*Obj).Date);
    SW(Obj, (*Obj).ClusterLow);
    SD(Obj, (*Obj).Size);
}

unsafe fn FatSwapLFNDirEntry(Obj: PLFN_DIRENTRY) {
    
    SW(Obj, (*Obj).StartCluster);
    for i in 0..5 {
        (*Obj).Name0_4[i] = SWAPW((*Obj).Name0_4[i]);
    }
    for i in 0..6 {
        (*Obj).Name5_10[i] = SWAPW((*Obj).Name5_10[i]);
    }
    for i in 0..2 {
        (*Obj).Name11_12[i] = SWAPW((*Obj).Name11_12[i]);
    }
}

unsafe fn FatSwapFatXDirEntry(Obj: PFATX_DIRENTRY) {
    SD(Obj, (*Obj).StartCluster);
    SD(Obj, (*Obj).Size);
    SW(Obj, (*Obj).Time);
    SW(Obj, (*Obj).Date);
    SW(Obj, (*Obj).CreateTime);
    SW(Obj, (*Obj).CreateDate);
    SW(Obj, (*Obj).LastAccessTime);
    SW(Obj, (*Obj).LastAccessDate);
}

unsafe fn FatNumberOfClusters(Volume: PFAT_VOLUME_INFO) -> ULONG {
    ((*Volume).TotalSectors - (*Volume).DataSectorStart) / (((*Volume).SectorsPerCluster) as u32)
}

unsafe fn FatOpenVolume(Volume: PFAT_VOLUME_INFO, BootSector: PFAT_BOOTSECTOR, PartitionSectorCount: ULONGLONG) -> BOOLEAN {

    let mut ErrMsg: [i8; 80] = [0; 80];
    let FatSize: ULONG;
    let mut i: ULONG;
    let FatVolumeBootSector: PFAT_BOOTSECTOR = BootSector as PFAT_BOOTSECTOR;
    let Fat32VolumeBootSector: PFAT32_BOOTSECTOR = BootSector as PFAT32_BOOTSECTOR;
    let FatXVolumeBootSector: PFATX_BOOTSECTOR = BootSector as PFATX_BOOTSECTOR;

    TRACE!("FatOpenVolume() DeviceId = %d\n", ((*Volume).DeviceId) as usize);

    (*Volume).FatType = (FatDetermineFatType(FatVolumeBootSector, PartitionSectorCount)) as u8 ;

    TRACE!("Dumping boot sector:\n");
    if ISFATX(((*Volume).FatType) as u32 ) != 0 {
        FatSwapFatXBootSector(FatXVolumeBootSector);
        TRACE!("sizeof(FATX_BOOTSECTOR) = 0x%x.\n", (core::mem::size_of::<FATX_BOOTSECTOR>()) as usize);

        TRACE!("FileSystemType: %c%c%c%c.\n", ((*FatXVolumeBootSector).FileSystemType[0]) as usize, ((*FatXVolumeBootSector).FileSystemType[1]) as usize, ((*FatXVolumeBootSector).FileSystemType[2]) as usize, ((*FatXVolumeBootSector).FileSystemType[3]) as usize);
        TRACE!("VolumeSerialNumber: 0x%x\n", ((*FatXVolumeBootSector).VolumeSerialNumber) as usize);
        TRACE!("SectorsPerCluster: %d\n", ((*FatXVolumeBootSector).SectorsPerCluster) as usize);
        TRACE!("NumberOfFats: %d\n", ((*FatXVolumeBootSector).NumberOfFats) as usize);
        TRACE!("Unknown: 0x%x\n", ((*FatXVolumeBootSector).Unknown) as usize);

        TRACE!("FatType %s\n", (if (*Volume).FatType == ((FATX16) as u8) { b"FATX16\0".as_ptr() } else { b"FATX32\0".as_ptr() }) as usize);
    } else if (*Volume).FatType == ((FAT32) as u8) {
        FatSwapFat32BootSector(Fat32VolumeBootSector);
        TRACE!("sizeof(FAT32_BOOTSECTOR) = 0x%x.\n", (core::mem::size_of::<FAT32_BOOTSECTOR>()) as usize);

        TRACE!("JumpBoot: 0x%x 0x%x 0x%x\n", ((*Fat32VolumeBootSector).JumpBoot[0]) as usize, ((*Fat32VolumeBootSector).JumpBoot[1]) as usize, ((*Fat32VolumeBootSector).JumpBoot[2]) as usize);
        TRACE!("OemName: %c%c%c%c%c%c%c%c\n", ((*Fat32VolumeBootSector).OemName[0]) as usize, ((*Fat32VolumeBootSector).OemName[1]) as usize, ((*Fat32VolumeBootSector).OemName[2]) as usize, ((*Fat32VolumeBootSector).OemName[3]) as usize, ((*Fat32VolumeBootSector).OemName[4]) as usize, ((*Fat32VolumeBootSector).OemName[5]) as usize, ((*Fat32VolumeBootSector).OemName[6]) as usize, ((*Fat32VolumeBootSector).OemName[7]) as usize);
        TRACE!("BytesPerSector: %d\n", ((*Fat32VolumeBootSector).BytesPerSector) as usize);
        TRACE!("SectorsPerCluster: %d\n", ((*Fat32VolumeBootSector).SectorsPerCluster) as usize);
        TRACE!("ReservedSectors: %d\n", ((*Fat32VolumeBootSector).ReservedSectors) as usize);
        TRACE!("NumberOfFats: %d\n", ((*Fat32VolumeBootSector).NumberOfFats) as usize);
        TRACE!("RootDirEntries: %d\n", ((*Fat32VolumeBootSector).RootDirEntries) as usize);
        TRACE!("TotalSectors: %d\n", ((*Fat32VolumeBootSector).TotalSectors) as usize);
        TRACE!("MediaDescriptor: 0x%x\n", ((*Fat32VolumeBootSector).MediaDescriptor) as usize);
        TRACE!("SectorsPerFat: %d\n", ((*Fat32VolumeBootSector).SectorsPerFat) as usize);
        TRACE!("SectorsPerTrack: %d\n", ((*Fat32VolumeBootSector).SectorsPerTrack) as usize);
        TRACE!("NumberOfHeads: %d\n", ((*Fat32VolumeBootSector).NumberOfHeads) as usize);
        TRACE!("HiddenSectors: %d\n", ((*Fat32VolumeBootSector).HiddenSectors) as usize);
        TRACE!("TotalSectorsBig: %d\n", ((*Fat32VolumeBootSector).TotalSectorsBig) as usize);
        TRACE!("SectorsPerFatBig: %d\n", ((*Fat32VolumeBootSector).SectorsPerFatBig) as usize);
        TRACE!("ExtendedFlags: 0x%x\n", ((*Fat32VolumeBootSector).ExtendedFlags) as usize);
        TRACE!("FileSystemVersion: 0x%x\n", ((*Fat32VolumeBootSector).FileSystemVersion) as usize);
        TRACE!("RootDirStartCluster: %d\n", ((*Fat32VolumeBootSector).RootDirStartCluster) as usize);
        TRACE!("FsInfo: %d\n", ((*Fat32VolumeBootSector).FsInfo) as usize);
        TRACE!("BackupBootSector: %d\n", ((*Fat32VolumeBootSector).BackupBootSector) as usize);
        TRACE!("Reserved: 0x%x\n", ((*Fat32VolumeBootSector).Reserved.as_ptr()) as usize);
        TRACE!("DriveNumber: 0x%x\n", ((*Fat32VolumeBootSector).DriveNumber) as usize);
        TRACE!("Reserved1: 0x%x\n", ((*Fat32VolumeBootSector).Reserved1) as usize);
        TRACE!("BootSignature: 0x%x\n", ((*Fat32VolumeBootSector).BootSignature) as usize);
        TRACE!("VolumeSerialNumber: 0x%x\n", ((*Fat32VolumeBootSector).VolumeSerialNumber) as usize);
        TRACE!("VolumeLabel: %c%c%c%c%c%c%c%c%c%c%c\n", ((*Fat32VolumeBootSector).VolumeLabel[0]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[1]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[2]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[3]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[4]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[5]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[6]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[7]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[8]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[9]) as usize, ((*Fat32VolumeBootSector).VolumeLabel[10]) as usize);
        TRACE!("FileSystemType: %c%c%c%c%c%c%c%c\n", ((*Fat32VolumeBootSector).FileSystemType[0]) as usize, ((*Fat32VolumeBootSector).FileSystemType[1]) as usize, ((*Fat32VolumeBootSector).FileSystemType[2]) as usize, ((*Fat32VolumeBootSector).FileSystemType[3]) as usize, ((*Fat32VolumeBootSector).FileSystemType[4]) as usize, ((*Fat32VolumeBootSector).FileSystemType[5]) as usize, ((*Fat32VolumeBootSector).FileSystemType[6]) as usize, ((*Fat32VolumeBootSector).FileSystemType[7]) as usize);
        TRACE!("BootSectorMagic: 0x%x\n", ((*Fat32VolumeBootSector).BootSectorMagic) as usize);
    } else {
        FatSwapFatBootSector(FatVolumeBootSector);
        TRACE!("sizeof(FAT_BOOTSECTOR) = 0x%x.\n", (core::mem::size_of::<FAT_BOOTSECTOR>()) as usize);

        TRACE!("JumpBoot: 0x%x 0x%x 0x%x\n", ((*FatVolumeBootSector).JumpBoot[0]) as usize, ((*FatVolumeBootSector).JumpBoot[1]) as usize, ((*FatVolumeBootSector).JumpBoot[2]) as usize);
        TRACE!("OemName: %c%c%c%c%c%c%c%c\n", ((*FatVolumeBootSector).OemName[0]) as usize, ((*FatVolumeBootSector).OemName[1]) as usize, ((*FatVolumeBootSector).OemName[2]) as usize, ((*FatVolumeBootSector).OemName[3]) as usize, ((*FatVolumeBootSector).OemName[4]) as usize, ((*FatVolumeBootSector).OemName[5]) as usize, ((*FatVolumeBootSector).OemName[6]) as usize, ((*FatVolumeBootSector).OemName[7]) as usize);
        TRACE!("BytesPerSector: %d\n", ((*FatVolumeBootSector).BytesPerSector) as usize);
        TRACE!("SectorsPerCluster: %d\n", ((*FatVolumeBootSector).SectorsPerCluster) as usize);
        TRACE!("ReservedSectors: %d\n", ((*FatVolumeBootSector).ReservedSectors) as usize);
        TRACE!("NumberOfFats: %d\n", ((*FatVolumeBootSector).NumberOfFats) as usize);
        TRACE!("RootDirEntries: %d\n", ((*FatVolumeBootSector).RootDirEntries) as usize);
        TRACE!("TotalSectors: %d\n", ((*FatVolumeBootSector).TotalSectors) as usize);
        TRACE!("MediaDescriptor: 0x%x\n", ((*FatVolumeBootSector).MediaDescriptor) as usize);
        TRACE!("SectorsPerFat: %d\n", ((*FatVolumeBootSector).SectorsPerFat) as usize);
        TRACE!("SectorsPerTrack: %d\n", ((*FatVolumeBootSector).SectorsPerTrack) as usize);
        TRACE!("NumberOfHeads: %d\n", ((*FatVolumeBootSector).NumberOfHeads) as usize);
        TRACE!("HiddenSectors: %d\n", ((*FatVolumeBootSector).HiddenSectors) as usize);
        TRACE!("TotalSectorsBig: %d\n", ((*FatVolumeBootSector).TotalSectorsBig) as usize);
        TRACE!("DriveNumber: 0x%x\n", ((*FatVolumeBootSector).DriveNumber) as usize);
        TRACE!("Reserved1: 0x%x\n", ((*FatVolumeBootSector).Reserved1) as usize);
        TRACE!("BootSignature: 0x%x\n", ((*FatVolumeBootSector).BootSignature) as usize);
        TRACE!("VolumeSerialNumber: 0x%x\n", ((*FatVolumeBootSector).VolumeSerialNumber) as usize);
        TRACE!("VolumeLabel: %c%c%c%c%c%c%c%c%c%c%c\n", ((*FatVolumeBootSector).VolumeLabel[0]) as usize, ((*FatVolumeBootSector).VolumeLabel[1]) as usize, ((*FatVolumeBootSector).VolumeLabel[2]) as usize, ((*FatVolumeBootSector).VolumeLabel[3]) as usize, ((*FatVolumeBootSector).VolumeLabel[4]) as usize, ((*FatVolumeBootSector).VolumeLabel[5]) as usize, ((*FatVolumeBootSector).VolumeLabel[6]) as usize, ((*FatVolumeBootSector).VolumeLabel[7]) as usize, ((*FatVolumeBootSector).VolumeLabel[8]) as usize, ((*FatVolumeBootSector).VolumeLabel[9]) as usize, ((*FatVolumeBootSector).VolumeLabel[10]) as usize);
        TRACE!("FileSystemType: %c%c%c%c%c%c%c%c\n", ((*FatVolumeBootSector).FileSystemType[0]) as usize, ((*FatVolumeBootSector).FileSystemType[1]) as usize, ((*FatVolumeBootSector).FileSystemType[2]) as usize, ((*FatVolumeBootSector).FileSystemType[3]) as usize, ((*FatVolumeBootSector).FileSystemType[4]) as usize, ((*FatVolumeBootSector).FileSystemType[5]) as usize, ((*FatVolumeBootSector).FileSystemType[6]) as usize, ((*FatVolumeBootSector).FileSystemType[7]) as usize);
        TRACE!("BootSectorMagic: 0x%x\n", ((*FatVolumeBootSector).BootSectorMagic) as usize);
    }

    if ISFATX(((*Volume).FatType) as u32 ) == 0 && (*FatVolumeBootSector).BootSectorMagic != 0xaa55 {
        sprintf(ErrMsg.as_mut_ptr(), b"Invalid boot sector magic (expected 0xaa55 found 0x%x)\0".as_ptr() as *const i8, (*FatVolumeBootSector).BootSectorMagic as u32 );
        FileSystemError(ErrMsg.as_mut_ptr() as PCHAR);
        return 0u8;
    }

    if (ISFATX(((*Volume).FatType) as u32 ) != 0 && 64 * 1024 < (*FatXVolumeBootSector).SectorsPerCluster * 512) ||
       (ISFATX(((*Volume).FatType) as u32 ) == 0 && 64 * 1024 < ((*FatVolumeBootSector).SectorsPerCluster as ULONG) * ((*FatVolumeBootSector).BytesPerSector as ULONG)) {
        FileSystemError(b"This file system has cluster sizes bigger than 64k.\nFreeLoader does not support this.\0".as_ptr() as PCHAR);
        return 0u8;
    }

    if ISFATX(((*Volume).FatType) as u32 ) != 0 {
        (*Volume).TotalSectors = (PartitionSectorCount) as u32 ;
    } else {
        (*Volume).TotalSectors = if (*FatVolumeBootSector).TotalSectors != 0 { ((*FatVolumeBootSector).TotalSectors as ULONGLONG) as u32  } else { ((*FatVolumeBootSector).TotalSectorsBig as ULONGLONG) as u32  };
    }

    if ISFATX(((*Volume).FatType) as u32 ) != 0 {
        (*Volume).BytesPerSector = (SECTOR_SIZE) as u16 ;
        (*Volume).SectorsPerCluster = (SWAPD((*FatXVolumeBootSector).SectorsPerCluster)) as u8 ;
        (*Volume).FatSectorStart = (4096 / (*Volume).BytesPerSector) as u32 ;
        (*Volume).ActiveFatSectorStart = (*Volume).FatSectorStart;
        (*Volume).NumberOfFats = 1;
        FatSize = (PartitionSectorCount / ((*Volume).SectorsPerCluster as ULONGLONG) * if (*Volume).FatType == ((FATX16) as u8) { 2 } else { 4 }) as ULONG;
        (*Volume).SectorsPerFat = (ROUND_UP((FatSize) as usize , 4096) / (((*Volume).BytesPerSector) as usize)) as u32 ;

        (*Volume).RootDirSectorStart = (*Volume).FatSectorStart + ((*Volume).NumberOfFats as ULONG) * (*Volume).SectorsPerFat;
        (*Volume).RootDirSectors = (*FatXVolumeBootSector).SectorsPerCluster;

        (*Volume).DataSectorStart = (*Volume).RootDirSectorStart + (*Volume).RootDirSectors;
    } else if (*Volume).FatType != ((FAT32) as u8) {
        (*Volume).BytesPerSector = (*FatVolumeBootSector).BytesPerSector;
        (*Volume).SectorsPerCluster = ((*FatVolumeBootSector).SectorsPerCluster as ULONG) as u8 ;
        (*Volume).FatSectorStart = (*FatVolumeBootSector).ReservedSectors as ULONG;
        (*Volume).ActiveFatSectorStart = (*Volume).FatSectorStart;
        (*Volume).NumberOfFats = ((*FatVolumeBootSector).NumberOfFats as ULONG) as u8 ;
        (*Volume).SectorsPerFat = (*FatVolumeBootSector).SectorsPerFat as ULONG;

        (*Volume).RootDirSectorStart = (*Volume).FatSectorStart + ((*Volume).NumberOfFats as ULONG) * (*Volume).SectorsPerFat;
        (*Volume).RootDirSectors = (((*FatVolumeBootSector).RootDirEntries as ULONG * 32) + ((((*Volume).BytesPerSector - 1)) as u32)) / (((*Volume).BytesPerSector) as u32);

        (*Volume).DataSectorStart = (*Volume).RootDirSectorStart + (*Volume).RootDirSectors;
    } else {
        (*Volume).BytesPerSector = (*Fat32VolumeBootSector).BytesPerSector;
        (*Volume).SectorsPerCluster = ((*Fat32VolumeBootSector).SectorsPerCluster as ULONG) as u8 ;
        (*Volume).FatSectorStart = (*Fat32VolumeBootSector).ReservedSectors as ULONG;
        (*Volume).ActiveFatSectorStart = (*Volume).FatSectorStart + (if (*Fat32VolumeBootSector).ExtendedFlags & 0x80 != 0 { ((*Fat32VolumeBootSector).ExtendedFlags & 0x0f) as ULONG * (*Fat32VolumeBootSector).SectorsPerFatBig  } else { 0 });
        (*Volume).NumberOfFats = ((*Fat32VolumeBootSector).NumberOfFats as ULONG) as u8 ;
        (*Volume).SectorsPerFat = (*Fat32VolumeBootSector).SectorsPerFatBig;

        (*Volume).RootDirStartCluster = (*Fat32VolumeBootSector).RootDirStartCluster;
        (*Volume).DataSectorStart = (*Volume).FatSectorStart + ((*Volume).NumberOfFats as ULONG) * (*Volume).SectorsPerFat;

        if (*Fat32VolumeBootSector).FileSystemVersion != 0 {
            FileSystemError(b"FreeLoader is too old to work with this FAT32 filesystem.\nPlease update FreeLoader.\0".as_ptr() as PCHAR);
            return 0u8;
        }
    }

    (*Volume).FatCacheSize = min((*Volume).SectorsPerFat, FAT_MAX_CACHE_SIZE / (((*Volume).BytesPerSector) as u32));
    TRACE!("FAT cache is %d sectors, %d bytes\n", ((*Volume).FatCacheSize) as usize, ((*Volume).FatCacheSize * (((*Volume).BytesPerSector) as u32)) as usize);

    (*Volume).FatCache = FrLdrTempAlloc(((*Volume).FatCacheSize * (((*Volume).BytesPerSector) as u32)) as usize, TAG_FAT_CACHE) as *mut u8;
    if (*Volume).FatCache.is_null() {
        FileSystemError(b"Cannot allocate memory for FAT cache\0".as_ptr() as PCHAR);
        return 0u8;
    }

    (*Volume).FatCacheIndex = FrLdrTempAlloc(((*Volume).FatCacheSize * core::mem::size_of::<ULONG>() as ULONG) as usize, TAG_FAT_VOLUME) as *mut ULONG;
    if (*Volume).FatCacheIndex.is_null() {
        FileSystemError(b"Cannot allocate memory for FAT cache index\0".as_ptr() as PCHAR);
        FrLdrTempFree((*Volume).FatCache as *mut core::ffi::c_void, TAG_FAT_CACHE);
        return 0u8;
    }

    if FatReadVolumeSectors(Volume, (*Volume).ActiveFatSectorStart, (*Volume).FatCacheSize, (*Volume).FatCache as *mut core::ffi::c_void) == 0 {
        FileSystemError(b"Error when reading FAT cache\0".as_ptr() as PCHAR);
        FrLdrTempFree((*Volume).FatCache as *mut core::ffi::c_void, TAG_FAT_CACHE);
        FrLdrTempFree((*Volume).FatCacheIndex as PVOID, TAG_FAT_VOLUME);
        return 0u8;
    }

    i = 0;
    while i < (*Volume).FatCacheSize {
        *(*Volume).FatCacheIndex.offset(i as isize) = (*Volume).ActiveFatSectorStart + i;
        i += 1;
    }

    1u8
}

unsafe fn FatDetermineFatType(FatBootSector: PFAT_BOOTSECTOR, PartitionSectorCount: ULONGLONG) -> ULONG {
    let RootDirSectors: ULONG;
    let DataSectorCount: ULONG;
    let SectorsPerFat: ULONG;
    let TotalSectors: ULONG;
    let CountOfClusters: ULONG;
    let Fat32BootSector: PFAT32_BOOTSECTOR = FatBootSector as PFAT32_BOOTSECTOR;
    let FatXBootSector: PFATX_BOOTSECTOR = FatBootSector as PFATX_BOOTSECTOR;

    if 0 == strncmp((*FatXBootSector).FileSystemType.as_ptr() as *const i8, b"FATX\0".as_ptr() as *const i8, 4) {
        CountOfClusters = (PartitionSectorCount / (*FatXBootSector).SectorsPerCluster as ULONGLONG) as ULONG;
        if CountOfClusters < 65525 {
            return FATX16;
        } else {
            return FATX32;
        }
    } else {
        RootDirSectors = ((SWAPW((*FatBootSector).RootDirEntries) as ULONG * 32) + (SWAPW((*FatBootSector).BytesPerSector) as ULONG - 1)) / SWAPW((*FatBootSector).BytesPerSector) as ULONG;
        SectorsPerFat = if SWAPW((*FatBootSector).SectorsPerFat) != 0 { SWAPW((*FatBootSector).SectorsPerFat) as ULONG } else { SWAPD((*Fat32BootSector).SectorsPerFatBig) };
        TotalSectors = if SWAPW((*FatBootSector).TotalSectors) != 0 { SWAPW((*FatBootSector).TotalSectors) as ULONG } else { SWAPD((*FatBootSector).TotalSectorsBig) };
        DataSectorCount = TotalSectors - (SWAPW((*FatBootSector).ReservedSectors) as ULONG + ((*FatBootSector).NumberOfFats as ULONG * SectorsPerFat) + RootDirSectors);

        if (*FatBootSector).SectorsPerCluster == 0 {
            CountOfClusters = 0;
        } else {
            CountOfClusters = DataSectorCount / (*FatBootSector).SectorsPerCluster as ULONG;
        }

        if CountOfClusters < 4085 {
            return FAT12;
        } else if CountOfClusters < 65525 {
            return FAT16;
        } else {
            return FAT32;
        }
    }
}

unsafe fn FatBufferDirectory(
    Volume: PFAT_VOLUME_INFO,
    DirectoryStartCluster: ULONG,
    DirectorySize: *mut ULONG,
    RootDirectory: BOOLEAN,
) -> PVOID {

    let mut DirectoryBuffer: PDIRECTORY_BUFFER;
    let mut Entry: PLIST_ENTRY;
    let mut DirectoryStartCluster = DirectoryStartCluster;
    let mut RootDirectory = RootDirectory;

    TRACE!("FatBufferDirectory() DirectoryStartCluster = %d RootDirectory = %s\n", (DirectoryStartCluster) as usize, (if RootDirectory != 0 {
            b"TRUE\0".as_ptr() as PCHAR
        } else {
            b"FALSE\0".as_ptr() as PCHAR
        }) as usize);

    if RootDirectory != 0 && (*Volume).FatType == ((FAT32) as u8) {
        DirectoryStartCluster = (*Volume).RootDirStartCluster;
        RootDirectory = 0u8;
    }

    // C source uses `LIST_ENTRY DirectoryBufferListHead = {&LH, &LH}` —
    // a self-referential static initializer the Rust translator can't
    // express. Without this, Flink/Blink stay NULL and the walker below
    // dereferences NULL+0x10 on the first iteration (arm64 Bug #3).
    if DirectoryBufferListHead.Flink.is_null() {
        InitializeListHead(core::ptr::addr_of_mut!(DirectoryBufferListHead));
    }

    Entry = DirectoryBufferListHead.Flink;
    loop {
        if Entry == core::ptr::addr_of_mut!(DirectoryBufferListHead) {
            break;
        }

        DirectoryBuffer = (Entry as *mut u8).offset(-(core::mem::offset_of!(DIRECTORY_BUFFER, Link) as isize)) as PDIRECTORY_BUFFER;

        if (*DirectoryBuffer).Volume == Volume as *mut core::ffi::c_void
            && (*DirectoryBuffer).DirectoryStartCluster == DirectoryStartCluster
        {
            TRACE!("Found cached buffer\n");
            *DirectorySize = (*DirectoryBuffer).DirectorySize;
            return core::ptr::addr_of_mut!((*DirectoryBuffer).Data) as PVOID;
        }

        Entry = (*Entry).Flink;
    }

    if RootDirectory != 0 {
        *DirectorySize = (*Volume).RootDirSectors * (((*Volume).BytesPerSector) as u32);
    } else {
        *DirectorySize = FatCountClustersInChain(Volume, DirectoryStartCluster)
            * (((*Volume).SectorsPerCluster) as u32)
            * (((*Volume).BytesPerSector) as u32);
    }

    TRACE!("Trying to allocate (DirectorySize) %d bytes.\n", (*DirectorySize) as usize);
    DirectoryBuffer = FrLdrTempAlloc(
        (*DirectorySize + core::mem::size_of::<DIRECTORY_BUFFER>() as ULONG) as usize,
        TAG_FAT_BUFFER,
    ) as PDIRECTORY_BUFFER;

    if DirectoryBuffer.is_null() {
        return core::ptr::null_mut();
    }

    if RootDirectory != 0 {
        if FatReadVolumeSectors(
            Volume,
            (*Volume).RootDirSectorStart,
            (*Volume).RootDirSectors,
            core::ptr::addr_of_mut!((*DirectoryBuffer).Data) as PVOID,
        ) == 0
        {
            FrLdrTempFree(DirectoryBuffer as PVOID, TAG_FAT_BUFFER);
            return core::ptr::null_mut();
        }
    } else {
        if FatReadClusterChain(
            Volume,
            DirectoryStartCluster,
            0xFFFFFFFF,
            core::ptr::addr_of_mut!((*DirectoryBuffer).Data) as PVOID,
            core::ptr::null_mut(),
        ) == 0
        {
            FrLdrTempFree(DirectoryBuffer as PVOID, TAG_FAT_BUFFER);
            return core::ptr::null_mut();
        }
    }

    (*DirectoryBuffer).Volume = Volume as *mut core::ffi::c_void;
    (*DirectoryBuffer).DirectoryStartCluster = DirectoryStartCluster;
    (*DirectoryBuffer).DirectorySize = *DirectorySize;
    InsertTailList(
        core::ptr::addr_of_mut!(DirectoryBufferListHead),
        core::ptr::addr_of_mut!((*DirectoryBuffer).Link),
    );

    return core::ptr::addr_of_mut!((*DirectoryBuffer).Data) as PVOID;
}

unsafe fn FatSearchDirectoryBufferForFile(
    _Volume: PFAT_VOLUME_INFO,
    mut DirectoryBuffer: PVOID,
    DirectorySize: ULONG,
    FileName: PCHAR,
    FatFileInfoPointer: PFAT_FILE_INFO,
) -> BOOLEAN {

    let mut LfnNameBuffer: [CHAR; 265] = [0; 265];
    let mut ShortNameBuffer: [CHAR; 20] = [0; 20];
    let StartCluster: ULONG;
    let mut OurDirEntry: DIRENTRY = core::mem::zeroed();
    let mut OurLfnDirEntry: LFN_DIRENTRY = core::mem::zeroed();
    let DirEntry: PDIRENTRY = &mut OurDirEntry;
    let LfnDirEntry: PLFN_DIRENTRY = &mut OurLfnDirEntry;

    let EntryCount: ULONG = DirectorySize / core::mem::size_of::<DIRENTRY>() as ULONG;

    TRACE!("FatSearchDirectoryBufferForFile() DirectoryBuffer = 0x%x EntryCount = %d FileName = %s\n", (DirectoryBuffer) as usize, (EntryCount) as usize, (FileName) as usize);

    RtlZeroMemory(
        ShortNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
        13 * core::mem::size_of::<CHAR>(),
    );
    RtlZeroMemory(
        LfnNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
        261 * core::mem::size_of::<CHAR>(),
    );

    let mut CurrentEntry: ULONG = 0;
    while CurrentEntry < EntryCount {
        OurLfnDirEntry = *(DirectoryBuffer as PLFN_DIRENTRY);
        FatSwapLFNDirEntry(LfnDirEntry);
        OurDirEntry = *(DirectoryBuffer as PDIRENTRY);
        FatSwapDirEntry(DirEntry);

        if (*DirEntry).FileName[0] == ((b'\0') as i8) {
            return 0u8;
        }

        if (*DirEntry).FileName[0] == ((0xE5u8) as i8) {
            RtlZeroMemory(
                ShortNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                13 * core::mem::size_of::<CHAR>(),
            );
            RtlZeroMemory(
                LfnNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                261 * core::mem::size_of::<CHAR>(),
            );
            CurrentEntry += 1;
            DirectoryBuffer = (DirectoryBuffer as PDIRENTRY).offset(1) as PVOID;
            continue;
        }

        if (*DirEntry).Attr == FAT_ATTR_LONG_NAME {
            if ((*LfnDirEntry).SequenceNumber & 0x80) != 0 {
                CurrentEntry += 1;
                DirectoryBuffer = (DirectoryBuffer as PDIRENTRY).offset(1) as PVOID;
                continue;
            }

            (*LfnDirEntry).SequenceNumber &= 0x3F;
            (*LfnDirEntry).SequenceNumber -= 1;

            if (*LfnDirEntry).Name0_4[0] != 0xFFFF {
                LfnNameBuffer[(0 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name0_4[0] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name0_4[1] != 0xFFFF {
                LfnNameBuffer[(1 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name0_4[1] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name0_4[2] != 0xFFFF {
                LfnNameBuffer[(2 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name0_4[2] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name0_4[3] != 0xFFFF {
                LfnNameBuffer[(3 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name0_4[3] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name0_4[4] != 0xFFFF {
                LfnNameBuffer[(4 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name0_4[4] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name5_10[0] != 0xFFFF {
                LfnNameBuffer[(5 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name5_10[0] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name5_10[1] != 0xFFFF {
                LfnNameBuffer[(6 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name5_10[1] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name5_10[2] != 0xFFFF {
                LfnNameBuffer[(7 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name5_10[2] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name5_10[3] != 0xFFFF {
                LfnNameBuffer[(8 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name5_10[3] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name5_10[4] != 0xFFFF {
                LfnNameBuffer[(9 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name5_10[4] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name5_10[5] != 0xFFFF {
                LfnNameBuffer[(10 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name5_10[5] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name11_12[0] != 0xFFFF {
                LfnNameBuffer[(11 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name11_12[0] as UCHAR) as CHAR;
            }
            if (*LfnDirEntry).Name11_12[1] != 0xFFFF {
                LfnNameBuffer[(12 + ((*LfnDirEntry).SequenceNumber * 13)) as usize] =
                    ((*LfnDirEntry).Name11_12[1] as UCHAR) as CHAR;
            }

            CurrentEntry += 1;
            DirectoryBuffer = (DirectoryBuffer as PDIRENTRY).offset(1) as PVOID;
            continue;
        }

        if ((*DirEntry).Attr & FAT_ATTR_VOLUMENAME) != 0 {
            RtlZeroMemory(
                ShortNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                13 * core::mem::size_of::<UCHAR>(),
            );
            RtlZeroMemory(
                LfnNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                261 * core::mem::size_of::<UCHAR>(),
            );
            CurrentEntry += 1;
            DirectoryBuffer = (DirectoryBuffer as PDIRENTRY).offset(1) as PVOID;
            continue;
        }

        FatParseShortFileName(ShortNameBuffer.as_mut_ptr(), DirEntry);

        if _stricmp(FileName, LfnNameBuffer.as_ptr()) == 0
            || _stricmp(FileName, ShortNameBuffer.as_ptr()) == 0
        {
            (*FatFileInfoPointer).Attributes = (*DirEntry).Attr;
            (*FatFileInfoPointer).FileSize = (*DirEntry).Size;
            (*FatFileInfoPointer).FilePointer = 0;
            StartCluster = (((*DirEntry).ClusterHigh as ULONG) << 16) + (*DirEntry).ClusterLow as ULONG;
            (*FatFileInfoPointer).CurrentCluster = StartCluster;
            (*FatFileInfoPointer).StartCluster = StartCluster;

            TRACE!("MSDOS Directory Entry:\n");
            TRACE!("FileName[11] = %c%c%c%c%c%c%c%c%c%c%c\n", ((*DirEntry).FileName[0]) as usize, ((*DirEntry).FileName[1]) as usize, ((*DirEntry).FileName[2]) as usize, ((*DirEntry).FileName[3]) as usize, ((*DirEntry).FileName[4]) as usize, ((*DirEntry).FileName[5]) as usize, ((*DirEntry).FileName[6]) as usize, ((*DirEntry).FileName[7]) as usize, ((*DirEntry).FileName[8]) as usize, ((*DirEntry).FileName[9]) as usize, ((*DirEntry).FileName[10]) as usize);
            TRACE!("Attr = 0x%x\n", ((*DirEntry).Attr) as usize);
            TRACE!("ReservedNT = 0x%x\n", ((*DirEntry).ReservedNT) as usize);
            TRACE!("TimeInTenths = %d\n", ((*DirEntry).TimeInTenths) as usize);
            TRACE!("CreateTime = %d\n", ((*DirEntry).CreateTime) as usize);
            TRACE!("CreateDate = %d\n", ((*DirEntry).CreateDate) as usize);
            TRACE!("LastAccessDate = %d\n", ((*DirEntry).LastAccessDate) as usize);
            TRACE!("ClusterHigh = 0x%x\n", ((*DirEntry).ClusterHigh) as usize);
            TRACE!("Time = %d\n", ((*DirEntry).Time) as usize);
            TRACE!("Date = %d\n", ((*DirEntry).Date) as usize);
            TRACE!("ClusterLow = 0x%x\n", ((*DirEntry).ClusterLow) as usize);
            TRACE!("Size = %d\n", ((*DirEntry).Size) as usize);
            TRACE!("StartCluster = 0x%x\n", (StartCluster) as usize);

            return 1u8;
        }

        RtlZeroMemory(
            ShortNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            13 * core::mem::size_of::<UCHAR>(),
        );
        RtlZeroMemory(
            LfnNameBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            261 * core::mem::size_of::<UCHAR>(),
        );

        CurrentEntry += 1;
        DirectoryBuffer = (DirectoryBuffer as PDIRENTRY).offset(1) as PVOID;
    }

    0u8
}

unsafe fn FatXSearchDirectoryBufferForFile(
    _Volume: PFAT_VOLUME_INFO,
    mut DirectoryBuffer: PVOID,
    DirectorySize: ULONG,
    FileName: PCHAR,
    FatFileInfoPointer: PFAT_FILE_INFO,
) -> BOOLEAN {
    let EntryCount: ULONG;
    let mut CurrentEntry: ULONG;
    let FileNameLen: SIZE_T;
    let mut OurDirEntry: FATX_DIRENTRY = core::mem::zeroed();
    let DirEntry: PFATX_DIRENTRY = &mut OurDirEntry;

    EntryCount = DirectorySize / core::mem::size_of::<FATX_DIRENTRY>() as ULONG;

    TRACE!("FatXSearchDirectoryBufferForFile() DirectoryBuffer = 0x%x EntryCount = %d FileName = %s\n", (DirectoryBuffer) as usize, (EntryCount) as usize, (FileName) as usize);

    FileNameLen = strlen(FileName);

    CurrentEntry = 0;
    while CurrentEntry < EntryCount {
        OurDirEntry = *(DirectoryBuffer as PFATX_DIRENTRY);
        FatSwapFatXDirEntry(&mut OurDirEntry);
        if 0xff == (*DirEntry).FileNameSize {
            break;
        }
        if 0xe5 == (*DirEntry).FileNameSize {
            CurrentEntry += 1;
            DirectoryBuffer = (DirectoryBuffer as PFATX_DIRENTRY).offset(1) as PVOID;
            continue;
        }
        if (FileNameLen == (*DirEntry).FileNameSize as SIZE_T)
            && (_strnicmp(FileName, (*DirEntry).FileName.as_ptr(), FileNameLen) == 0)
        {
            (*FatFileInfoPointer).Attributes = (*DirEntry).Attr;
            (*FatFileInfoPointer).FileSize = (*DirEntry).Size;
            (*FatFileInfoPointer).FilePointer = 0;
            (*FatFileInfoPointer).CurrentCluster = (*DirEntry).StartCluster;
            (*FatFileInfoPointer).StartCluster = (*DirEntry).StartCluster;

            TRACE!("FATX Directory Entry:\n");
            TRACE!("FileNameSize = %d\n", ((*DirEntry).FileNameSize) as usize);
            TRACE!("Attr = 0x%x\n", ((*DirEntry).Attr) as usize);
            TRACE!("StartCluster = 0x%x\n", ((*DirEntry).StartCluster) as usize);
            TRACE!("Size = %d\n", ((*DirEntry).Size) as usize);
            TRACE!("Time = %d\n", ((*DirEntry).Time) as usize);
            TRACE!("Date = %d\n", ((*DirEntry).Date) as usize);
            TRACE!("CreateTime = %d\n", ((*DirEntry).CreateTime) as usize);
            TRACE!("CreateDate = %d\n", ((*DirEntry).CreateDate) as usize);
            TRACE!("LastAccessTime = %d\n", ((*DirEntry).LastAccessTime) as usize);
            TRACE!("LastAccessDate = %d\n", ((*DirEntry).LastAccessDate) as usize);

            return 1u8;
        }
        CurrentEntry += 1;
        DirectoryBuffer = (DirectoryBuffer as PFATX_DIRENTRY).offset(1) as PVOID;
    }

    return 0u8;
}

pub unsafe extern "C" fn FatLookupFile(
    Volume: PFAT_VOLUME_INFO,
    FileName: PCSTR,
    FatFileInfoPointer: PFAT_FILE_INFO,
) -> ARC_STATUS {

    let NumberOfPathParts: ULONG;
    let mut i: ULONG;
    let mut PathPart: [CHAR; 261] = [0; 261];
    let _DirectoryBuffer: PVOID;
    let mut DirectoryStartCluster: ULONG = 0;
    let mut DirectorySize: ULONG = 0;
    let mut FatFileInfo: FAT_FILE_INFO = core::mem::zeroed();
    let mut FileName = FileName;

    TRACE!("FatLookupFile() FileName = %s\n", (FileName) as usize);

    core::ptr::write_bytes(FatFileInfoPointer, 0, 1);

    if *FileName == b'\\' as CHAR || *FileName == b'/' as CHAR {
        FileName = FileName.offset(1);
    }
    PathPart[0] = 0;

    NumberOfPathParts = FsGetNumPathParts(FileName);

    i = 0;
    while i < NumberOfPathParts {
        FsGetFirstNameFromPath(PathPart.as_mut_ptr(), FileName);

        while *FileName != b'\\' as CHAR && *FileName != b'/' as CHAR && *FileName != 0 {
            FileName = FileName.offset(1);
        }
        FileName = FileName.offset(1);

        let DirectoryBuffer = FatBufferDirectory(
            Volume,
            DirectoryStartCluster,
            core::ptr::addr_of_mut!(DirectorySize),
            if i == 0 { 1u8 } else { 0u8 },
        );
        if DirectoryBuffer == core::ptr::null_mut() {
            return ENOMEM;
        }

        if ISFATX(((*Volume).FatType) as u32 ) != 0 {
            if FatXSearchDirectoryBufferForFile(
                Volume,
                DirectoryBuffer,
                DirectorySize,
                PathPart.as_mut_ptr(),
                &mut FatFileInfo,
            ) == 0
            {
                return ENOENT;
            }
        } else {
            if FatSearchDirectoryBufferForFile(
                Volume,
                DirectoryBuffer,
                DirectorySize,
                PathPart.as_mut_ptr(),
                &mut FatFileInfo,
            ) == 0
            {
                return ENOENT;
            }
        }

        if (i + 1) < NumberOfPathParts {
            if (FatFileInfo.Attributes & FAT_ATTR_DIRECTORY) == 0 {
                return ENOTDIR;
            }
            DirectoryStartCluster = FatFileInfo.StartCluster;
        }

        i += 1;
    }

    RtlCopyMemory(
        FatFileInfoPointer as PVOID,
        &FatFileInfo as *const FAT_FILE_INFO as PVOID,
        core::mem::size_of::<FAT_FILE_INFO>(),
    );

    (*FatFileInfoPointer).Attributes = 0;
    if (FatFileInfo.Attributes & FAT_ATTR_READONLY) != 0 {
        (*FatFileInfoPointer).Attributes |= (ReadOnlyFile) as u8 ;
    }
    if (FatFileInfo.Attributes & FAT_ATTR_HIDDEN) != 0 {
        (*FatFileInfoPointer).Attributes |= (HiddenFile) as u8 ;
    }
    if (FatFileInfo.Attributes & FAT_ATTR_SYSTEM) != 0 {
        (*FatFileInfoPointer).Attributes |= (SystemFile) as u8 ;
    }
    if (FatFileInfo.Attributes & FAT_ATTR_ARCHIVE) != 0 {
        (*FatFileInfoPointer).Attributes |= (ArchiveFile) as u8 ;
    }
    if (FatFileInfo.Attributes & FAT_ATTR_DIRECTORY) != 0 {
        (*FatFileInfoPointer).Attributes |= (DirectoryFile) as u8 ;
    }

    (*FatFileInfoPointer).FileNameLength = strlen(PathPart.as_ptr()) as ULONG;
    (*FatFileInfoPointer).FileNameLength = min(
        (*FatFileInfoPointer).FileNameLength,
        (core::mem::size_of_val(&(*FatFileInfoPointer).FileName) - 1) as ULONG,
    );
    RtlCopyMemory(
        (*FatFileInfoPointer).FileName.as_mut_ptr() as PVOID,
        PathPart.as_ptr() as PVOID,
        (*FatFileInfoPointer).FileNameLength as usize,
    );

    ESUCCESS
}

unsafe fn FatParseShortFileName(Buffer: PCHAR, DirEntry: PDIRENTRY) {
    let mut Idx: ULONG;

    Idx = 0;
    RtlZeroMemory(Buffer as *mut core::ffi::c_void, 13);

    if (*DirEntry).FileName[0] == 0x05 {
        (*DirEntry).FileName[0] = 0xE5u8 as i8;
    }

    while Idx < 8 {
        if (*DirEntry).FileName[Idx as usize] == ((b' ') as i8) {
            break;
        }

        *Buffer.offset(Idx as isize) = (*DirEntry).FileName[Idx as usize];
        Idx += 1;
    }

    if (*DirEntry).FileName[8] != ((b' ') as i8) {
        *Buffer.offset(Idx as isize) = (b'.') as i8 ;
        Idx += 1;
        *Buffer.offset(Idx as isize) = if (*DirEntry).FileName[8] == ((b' ') as i8) { 0 } else { (*DirEntry).FileName[8] };
        Idx += 1;
        *Buffer.offset(Idx as isize) = if (*DirEntry).FileName[9] == ((b' ') as i8) { 0 } else { (*DirEntry).FileName[9] };
        Idx += 1;
        *Buffer.offset(Idx as isize) = if (*DirEntry).FileName[10] == ((b' ') as i8) { 0 } else { (*DirEntry).FileName[10] };
        Idx += 1;
    }
}

unsafe fn FatGetFatSector(Volume: PFAT_VOLUME_INFO, FatSectorNumber: UINT32) -> PUCHAR {

    let SectorNumAbsolute: UINT32 = (*Volume).ActiveFatSectorStart + FatSectorNumber;
    let CacheIndex: UINT32 = FatSectorNumber % (*Volume).FatCacheSize;

    if (*Volume).FatCacheIndex.add(CacheIndex as usize).read() != SectorNumAbsolute {
        let SectorsToRead: UINT32 = min(
            (*Volume).FatCacheSize - CacheIndex,
            min((*Volume).SectorsPerFat - FatSectorNumber, 4)
        );
        let mut i: UINT8;

        if FatReadVolumeSectors(
            Volume,
            SectorNumAbsolute,
            SectorsToRead,
            (*Volume).FatCache.add((CacheIndex * (((*Volume).BytesPerSector) as u32)) as usize) as PVOID
        ) == 0
        {
            return core::ptr::null_mut();
        }

        i = 0;
        while i < SectorsToRead as UINT8 {
            (*Volume).FatCacheIndex.add((CacheIndex + i as UINT32) as usize).write(SectorNumAbsolute + i as UINT32);
            i += 1;
        }

        TRACE!("FAT cache miss: read sector 0x%x from disk\n", (SectorNumAbsolute) as usize);
    } else {
        TRACE!("FAT cache hit: sector 0x%x present\n", (SectorNumAbsolute) as usize);
    }

    (*Volume).FatCache.add((CacheIndex * (((*Volume).BytesPerSector) as u32)) as usize)
}

unsafe fn FatGetFatEntry(Volume: PFAT_VOLUME_INFO, Cluster: UINT32, ClusterPointer: PUINT32) -> BOOLEAN {

    let FatOffset: UINT32;
    let ThisFatSecNum: UINT32;
    let ThisFatEntOffset: UINT32;
    let mut fat: UINT32;
    let ReadBuffer: PUCHAR;

    TRACE!("FatGetFatEntry() Retrieving FAT entry for cluster %d.\n", (Cluster) as usize);

    match (*Volume).FatType as ULONG {
        FAT12 => {
            FatOffset = Cluster + (Cluster / 2);
            ThisFatSecNum = FatOffset / (((*Volume).BytesPerSector) as u32);
            ThisFatEntOffset = FatOffset % (((*Volume).BytesPerSector) as u32);

            TRACE!("FatOffset: %d\n", (FatOffset) as usize);
            TRACE!("ThisFatSecNum: %d\n", (ThisFatSecNum) as usize);
            TRACE!("ThisFatEntOffset: %d\n", (ThisFatEntOffset) as usize);

            ReadBuffer = FatGetFatSector(Volume, ThisFatSecNum);
            if ReadBuffer.is_null() {
                return 0u8;
            }

            fat = *(ReadBuffer.add(ThisFatEntOffset as usize) as *const USHORT) as UINT32;
            fat = SWAPW(fat as UINT16) as UINT32;
            fat = if (Cluster & 0x0001) != 0 {
                fat >> 4
            } else {
                fat & 0x0FFF
            };
        }

        FAT16 | FATX16 => {
            FatOffset = Cluster * 2;
            ThisFatSecNum = FatOffset / (((*Volume).BytesPerSector) as u32);
            ThisFatEntOffset = FatOffset % (((*Volume).BytesPerSector) as u32);

            ReadBuffer = FatGetFatSector(Volume, ThisFatSecNum);
            if ReadBuffer.is_null() {
                return 0u8;
            }

            fat = *(ReadBuffer.add(ThisFatEntOffset as usize) as *const USHORT) as UINT32;
            fat = SWAPW(fat as UINT16) as UINT32;
        }

        FAT32 | FATX32 => {
            FatOffset = Cluster * 4;
            ThisFatSecNum = FatOffset / (((*Volume).BytesPerSector) as u32);
            ThisFatEntOffset = FatOffset % (((*Volume).BytesPerSector) as u32);

            ReadBuffer = FatGetFatSector(Volume, ThisFatSecNum);
            if ReadBuffer.is_null() {
                return 0u8;
            }

            fat = (*(ReadBuffer.add(ThisFatEntOffset as usize) as *const ULONG)) & 0x0FFFFFFF;
            fat = SWAPD(fat);
        }

        _ => {
            ERR!("Unknown FAT type %d\n", ((*Volume).FatType) as usize);
            return 0u8;
        }
    }

    TRACE!("FAT entry is 0x%x.\n", (fat) as usize);

    *ClusterPointer = fat;

    1u8
}

unsafe fn FatCountClustersInChain(Volume: PFAT_VOLUME_INFO, mut StartCluster: UINT32) -> ULONG {

    let mut ClusterCount: ULONG = 0;

    TRACE!("FatCountClustersInChain() StartCluster = %d\n", (StartCluster) as usize);

    loop {
        if FAT_IS_END_CLUSTER(Volume, StartCluster) != 0 {
            break;
        }

        ClusterCount += 1;

        if FatGetFatEntry(Volume, StartCluster, core::ptr::addr_of_mut!(StartCluster)) == 0 {
            return 0;
        }
    }

    TRACE!("FatCountClustersInChain() ClusterCount = %d\n", (ClusterCount) as usize);

    return ClusterCount;
}

unsafe fn FatReadAdjacentClusters(
    Volume: PFAT_VOLUME_INFO,
    StartClusterNumber: UINT32,
    MaxClusters: UINT32,
    Buffer: PVOID,
    ClustersRead: PUINT32,
    LastClusterNumber: PUINT32,
) -> BOOLEAN {

    let mut NextClusterNumber: UINT32 = 0;
    let mut ClustersToRead: UINT32 = 1;
    let mut PrevClusterNumber: UINT32 = StartClusterNumber;
    let ClusterStartSector: UINT32 = ((PrevClusterNumber - 2) * (((*Volume).SectorsPerCluster) as u32)) + (*Volume).DataSectorStart;

    *ClustersRead = 0;
    *LastClusterNumber = 0;

    if FatGetFatEntry(Volume, StartClusterNumber, &mut NextClusterNumber) == 0u8 {
        return 0u8;
    }

    while FAT_IS_END_CLUSTER(Volume, NextClusterNumber) == 0u8 && ClustersToRead < MaxClusters && (NextClusterNumber == PrevClusterNumber + 1) {
        ClustersToRead += 1;
        PrevClusterNumber = NextClusterNumber;
        if FatGetFatEntry(Volume, PrevClusterNumber, &mut NextClusterNumber) == 0u8 {
            return 0u8;
        }
    }

    if FatReadVolumeSectors(Volume, ClusterStartSector, ClustersToRead * (((*Volume).SectorsPerCluster) as u32), Buffer) == 0u8 {
        return 0u8;
    }

    *ClustersRead = ClustersToRead;
    *LastClusterNumber = NextClusterNumber;

    if FAT_IS_END_CLUSTER(Volume, NextClusterNumber) == 0u8 && ClustersToRead < MaxClusters { 1u8 } else { 0u8 }
}

unsafe fn FatReadClusterChain(
    Volume: PFAT_VOLUME_INFO,
    StartClusterNumber: UINT32,
    NumberOfClusters: UINT32,
    Buffer: PVOID,
    LastClusterNumber: PUINT32,
) -> BOOLEAN {

    let mut ClustersRead: UINT32 = 0;
    let mut NextClusterNumber: UINT32 = 0;
    let mut ClustersLeft: UINT32 = NumberOfClusters;
    let mut StartClusterNumber = StartClusterNumber;
    let mut Buffer = Buffer;

    TRACE!("FatReadClusterChain() StartClusterNumber = %d NumberOfClusters = %d Buffer = 0x%x\n", (StartClusterNumber) as usize, (NumberOfClusters) as usize, (Buffer) as usize);

    while FatReadAdjacentClusters(
        Volume,
        StartClusterNumber,
        ClustersLeft,
        Buffer,
        core::ptr::addr_of_mut!(ClustersRead),
        core::ptr::addr_of_mut!(NextClusterNumber),
    ) != 0
    {
        ClustersLeft -= ClustersRead;
        Buffer = (Buffer as ULONG_PTR + (ClustersRead * (((*Volume).SectorsPerCluster) as u32) * (((*Volume).BytesPerSector) as u32)) as ULONG_PTR) as PVOID;
        StartClusterNumber = NextClusterNumber;
    }

    if !LastClusterNumber.is_null() {
        *LastClusterNumber = NextClusterNumber;
    }

    if ClustersRead > 0 { 1u8 } else { 0u8 }
}

unsafe fn FatReadPartialCluster(
    Volume: PFAT_VOLUME_INFO,
    ClusterNumber: ULONG,
    StartingOffset: ULONG,
    Length: ULONG,
    Buffer: PVOID,
) -> BOOLEAN {
    let ClusterStartSector: ULONG;
    let SectorOffset: ULONG;
    let ReadSize: ULONG;
    let SectorCount: ULONG;
    let ReadBuffer: PUCHAR;
    let mut Success: BOOLEAN = 0u8;
    let mut StartingOffset = StartingOffset;

    ClusterStartSector = ((ClusterNumber - 2) * (((*Volume).SectorsPerCluster) as u32)) + (*Volume).DataSectorStart;

    SectorOffset = StartingOffset / (((*Volume).BytesPerSector) as u32);
    StartingOffset %= ((*Volume).BytesPerSector) as u32 ;

    SectorCount = (StartingOffset + Length + (((*Volume).BytesPerSector) as u32) - 1) / (((*Volume).BytesPerSector) as u32);

    ReadSize = SectorCount * (((*Volume).BytesPerSector) as u32);

    ReadBuffer = FrLdrTempAlloc((ReadSize) as usize , TAG_FAT_BUFFER) as PUCHAR;
    if ReadBuffer.is_null() {
        return 0u8;
    }

    if FatReadVolumeSectors(Volume, ClusterStartSector + SectorOffset, SectorCount, ReadBuffer as PVOID) != 0 {
        RtlCopyMemory(Buffer, ReadBuffer.add(StartingOffset as usize) as PVOID, Length as SIZE_T);
        Success = 1u8;
    }

    FrLdrTempFree(ReadBuffer as PVOID, TAG_FAT_BUFFER);

    return Success;
}

unsafe fn FatReadFile(FatFileInfo: PFAT_FILE_INFO, BytesToRead: ULONG, BytesRead: *mut ULONG, Buffer: PVOID) -> BOOLEAN {

    let Volume: PFAT_VOLUME_INFO = (*FatFileInfo).Volume;
    let mut NextClusterNumber: UINT32 = 0;
    let BytesPerCluster: UINT32;
    let mut BytesToRead = BytesToRead;
    let mut Buffer = Buffer;

    TRACE!("FatReadFile() BytesToRead=%u FileSize=%u FilePointer=%u CurrentCluster=0x%x\n", (BytesToRead) as usize, ((*FatFileInfo).FileSize) as usize, ((*FatFileInfo).FilePointer) as usize, ((*FatFileInfo).CurrentCluster) as usize);

    if !BytesRead.is_null() {
        *BytesRead = 0;
    }

    if (*FatFileInfo).FilePointer >= (*FatFileInfo).FileSize {
        return 1u8;
    }

    if ((*FatFileInfo).FilePointer + BytesToRead) > (*FatFileInfo).FileSize {
        BytesToRead = (*FatFileInfo).FileSize - (*FatFileInfo).FilePointer;
    }

    BytesPerCluster = ((*Volume).SectorsPerCluster as ULONG) * ((*Volume).BytesPerSector as ULONG);

    if (*FatFileInfo).FilePointer % BytesPerCluster != 0 {
        let OffsetInCluster: UINT32 = (*FatFileInfo).FilePointer % BytesPerCluster;
        let LengthInCluster: UINT32 = min(BytesToRead, BytesPerCluster - OffsetInCluster);

        if FatReadPartialCluster(Volume, (*FatFileInfo).CurrentCluster, OffsetInCluster, LengthInCluster, Buffer) == 0 {
            return 0u8;
        }
        if !BytesRead.is_null() {
            *BytesRead += LengthInCluster;
        }
        BytesToRead -= LengthInCluster;
        (*FatFileInfo).FilePointer += LengthInCluster;
        Buffer = (Buffer as ULONG_PTR + LengthInCluster as ULONG_PTR) as PVOID;

        if (LengthInCluster + OffsetInCluster) == BytesPerCluster {
            if FatGetFatEntry(Volume, (*FatFileInfo).CurrentCluster, &mut NextClusterNumber) == 0 {
                return 0u8;
            }

            (*FatFileInfo).CurrentCluster = NextClusterNumber;
            TRACE!("FatReadFile() FatFileInfo->CurrentCluster = 0x%x\n", ((*FatFileInfo).CurrentCluster) as usize);
        }
    }

    if BytesToRead > 0 {
        let NumberOfClusters: UINT32 = BytesToRead / BytesPerCluster;

        TRACE!("Going to read: %u clusters\n", (NumberOfClusters) as usize);

        if NumberOfClusters > 0 {
            let BytesReadHere: UINT32 = NumberOfClusters * BytesPerCluster;

            if FatReadClusterChain(Volume, (*FatFileInfo).CurrentCluster, NumberOfClusters, Buffer, &mut NextClusterNumber) == 0 {
                return 0u8;
            }

            if !BytesRead.is_null() {
                *BytesRead += BytesReadHere;
            }
            BytesToRead -= BytesReadHere;
            Buffer = (Buffer as ULONG_PTR + BytesReadHere as ULONG_PTR) as PVOID;

            (*FatFileInfo).FilePointer += BytesReadHere;
            (*FatFileInfo).CurrentCluster = NextClusterNumber;
            TRACE!("FatReadFile() FatFileInfo->CurrentCluster = 0x%x\n", ((*FatFileInfo).CurrentCluster) as usize);
        }
    }

    if BytesToRead > 0 {
        if FatReadPartialCluster(Volume, (*FatFileInfo).CurrentCluster, 0, BytesToRead, Buffer) == 0 {
            return 0u8;
        }
        if !BytesRead.is_null() {
            *BytesRead += BytesToRead;
        }
        (*FatFileInfo).FilePointer += BytesToRead;
    }

    return 1u8;
}

unsafe fn FatReadVolumeSectors(
    Volume: PFAT_VOLUME_INFO,
    SectorNumber: ULONG,
    SectorCount: ULONG,
    Buffer: PVOID,
) -> BOOLEAN {
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let mut Count: ULONG = 0;
    let mut Status: ARC_STATUS = ESUCCESS;

    Position.QuadPart = ((SectorNumber as ULONGLONG) * ((*Volume).BytesPerSector as ULONGLONG)) as i64 ;
    Status = ArcSeek((*Volume).DeviceId, &mut Position, SeekAbsolute);
    if Status != ESUCCESS {
        TRACE!("FatReadVolumeSectors() Failed to seek\n");
        return 0u8;
    }

    Status = ArcRead(
        (*Volume).DeviceId,
        Buffer,
        SectorCount * (((*Volume).BytesPerSector) as u32),
        &mut Count,
    );
    if Status != ESUCCESS || Count != SectorCount * (((*Volume).BytesPerSector) as u32) {
        TRACE!("FatReadVolumeSectors() Failed to read\n");
        return 0u8;
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn FatClose(FileId: ULONG) -> ARC_STATUS {

    let FileHandle = FsGetDeviceSpecific(FileId);

    FrLdrTempFree(FileHandle as PVOID, TAG_FAT_FILE);

    return ESUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn FatGetFileInformation(FileId: u32, Information: *mut FILEINFORMATION) -> ARC_STATUS {

    let FileHandle = FsGetDeviceSpecific(FileId);

    core::ptr::write_bytes(Information, 0, 1);
    (*Information).EndingAddress.u.LowPart = (*FileHandle).FileSize;
    (*Information).CurrentAddress.u.LowPart = (*FileHandle).FilePointer;

    (*Information).Attributes = (*FileHandle).Attributes;

    (*Information).FileNameLength = min((*FileHandle).FileNameLength, (core::mem::size_of_val(&(*Information).FileName) - 1) as u32 );
    core::ptr::copy_nonoverlapping(
        (*FileHandle).FileName.as_ptr(),
        (*Information).FileName.as_mut_ptr(),
        (*Information).FileNameLength as usize
    );
    (*Information).FileName[(*Information).FileNameLength as usize] = 0;

    TRACE!("FatGetFileInformation(%lu) -> FileSize = %lu, FilePointer = 0x%lx\n", (FileId) as usize, ((*Information).EndingAddress.u.LowPart) as usize, ((*Information).CurrentAddress.u.LowPart) as usize);

    0
}

#[no_mangle]
pub unsafe extern "C" fn FatOpen(Path: PCHAR, OpenMode: OPENMODE, FileId: *mut ULONG) -> ARC_STATUS {

    let FatVolume: PFAT_VOLUME_INFO;
    let mut TempFileInfo: FAT_FILE_INFO = core::mem::zeroed();
    let FileHandle: PFAT_FILE_INFO;
    let DeviceId: ULONG;
    let IsDirectory: BOOLEAN;
    let mut Status: ARC_STATUS = ESUCCESS;

    if OpenMode != OpenReadOnly && OpenMode != OpenDirectory {
        return EACCES;
    }

    DeviceId = FsGetDeviceId(*FileId);
    FatVolume = FatVolumes[DeviceId as usize];

    TRACE!("FatOpen() FileName = %s (DeviceId=%u)\n", (Path) as usize, (DeviceId) as usize);

    core::ptr::write_bytes(&mut TempFileInfo as *mut FAT_FILE_INFO, 0, 1);
    Status = FatLookupFile(FatVolume, Path, &mut TempFileInfo);
    if Status != ESUCCESS {
        return ENOENT;
    }

    IsDirectory = if TempFileInfo.Attributes & ((DirectoryFile) as u8) != 0 { 1u8 } else { 0u8 };
    if IsDirectory != 0 && OpenMode != OpenDirectory {
        return EISDIR;
    } else if IsDirectory == 0 && OpenMode != OpenReadOnly {
        return ENOTDIR;
    }

    FileHandle = FrLdrTempAlloc(core::mem::size_of::<FAT_FILE_INFO>(), TAG_FAT_FILE) as PFAT_FILE_INFO;
    if FileHandle.is_null() {
        return ENOMEM;
    }

    core::ptr::copy_nonoverlapping(&TempFileInfo as *const FAT_FILE_INFO, FileHandle, 1);
    (*FileHandle).Volume = FatVolume;

    FsSetDeviceSpecific(*FileId, FileHandle as *mut core::ffi::c_void);
    return ESUCCESS;
}

unsafe extern "C" fn FatRead(FileId: ULONG, Buffer: PVOID, N: ULONG, Count: PULONG) -> ARC_STATUS {

    let FileHandle: PFAT_FILE_INFO = FsGetDeviceSpecific(FileId);
    let Success: BOOLEAN = FatReadFile(FileHandle, N, Count, Buffer);
    
    if Success != 0 {
        ESUCCESS
    } else {
        EIO
    }
}

#[no_mangle]
pub unsafe extern "C" fn FatSeek(FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: SEEKMODE) -> ARC_STATUS {

    let FileHandle: PFAT_FILE_INFO = FsGetDeviceSpecific(FileId);
    let Volume: PFAT_VOLUME_INFO = (*FileHandle).Volume;
    let mut NewPosition: LARGE_INTEGER = *Position;

    match SeekMode {
        _SEEKMODE_SeekAbsolute => {},
        _SEEKMODE_SeekRelative => {
            NewPosition.QuadPart += ((*FileHandle).FilePointer as ULONGLONG) as i64 ;
        },
        _ => {
            return EINVAL;
        }
    }

    if NewPosition.u.HighPart != 0 {
        return EINVAL;
    }
    if NewPosition.u.LowPart >= (*FileHandle).FileSize {
        return EINVAL;
    }

    TRACE!("FatSeek() NewPosition = %u, OldPointer = %u, SeekMode = %d\n", (NewPosition.u.LowPart) as usize, ((*FileHandle).FilePointer) as usize, (SeekMode) as usize);

    {
        let OldClusterIdx: UINT32 = (*FileHandle).FilePointer / (((*Volume).SectorsPerCluster as u32) * ((*Volume).BytesPerSector as u32));
        let NewClusterIdx: UINT32 = NewPosition.u.LowPart / (((*Volume).SectorsPerCluster as u32) * ((*Volume).BytesPerSector as u32));

        TRACE!("FatSeek() OldClusterIdx: %u, NewClusterIdx: %u\n", (OldClusterIdx) as usize, (NewClusterIdx) as usize);

        if NewClusterIdx != OldClusterIdx {
            let mut CurrentClusterIdx: UINT32;
            let mut ClusterNumber: UINT32;

            if NewClusterIdx > OldClusterIdx {
                CurrentClusterIdx = OldClusterIdx;
                ClusterNumber = (*FileHandle).CurrentCluster;
            } else {
                CurrentClusterIdx = 0;
                ClusterNumber = (*FileHandle).StartCluster;
            }

            while CurrentClusterIdx < NewClusterIdx {
                if FatGetFatEntry(Volume, ClusterNumber, core::ptr::addr_of_mut!(ClusterNumber)) == 0 {
                    return EIO;
                }
                CurrentClusterIdx += 1;
            }
            (*FileHandle).CurrentCluster = ClusterNumber;
        }
    }

    (*FileHandle).FilePointer = NewPosition.u.LowPart;

    ESUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn FatGetVolumeSize(DeviceId: ULONG) -> ULONGLONG {
    let Volume = FatVolumes[DeviceId as usize];
    ((*Volume).TotalSectors as ULONGLONG) * ((*Volume).BytesPerSector as ULONGLONG)
}

#[no_mangle]
pub unsafe extern "C" fn FatMount(DeviceId: ULONG) -> *const DEVVTBL {

    TRACE!("Enter FatMount(%lu)\n", (DeviceId) as usize);

    let Volume: PFAT_VOLUME_INFO = FrLdrTempAlloc(core::mem::size_of::<FAT_VOLUME_INFO>(), TAG_FAT_VOLUME) as PFAT_VOLUME_INFO;
    if Volume.is_null() {
        return core::ptr::null();
    }
    RtlZeroMemory(Volume as PVOID, core::mem::size_of::<FAT_VOLUME_INFO>());

    let mut Buffer: [UCHAR; 512] = [0; 512];
    let BootSector: PFAT_BOOTSECTOR = Buffer.as_mut_ptr() as PFAT_BOOTSECTOR;
    let BootSector32: PFAT32_BOOTSECTOR = Buffer.as_mut_ptr() as PFAT32_BOOTSECTOR;
    let BootSectorX: PFATX_BOOTSECTOR = Buffer.as_mut_ptr() as PFATX_BOOTSECTOR;

    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    Position.QuadPart = 0;
    let mut Status: ARC_STATUS = ArcSeek(DeviceId, &mut Position, SeekAbsolute);
    if Status != ESUCCESS {
        FrLdrTempFree(Volume as PVOID, TAG_FAT_VOLUME);
        return core::ptr::null();
    }

    let mut Count: ULONG = 0;
    Status = ArcRead(DeviceId, Buffer.as_mut_ptr() as PVOID, core::mem::size_of_val(&Buffer) as ULONG, &mut Count);
    if Status != ESUCCESS || Count != core::mem::size_of_val(&Buffer) as ULONG {
        FrLdrTempFree(Volume as PVOID, TAG_FAT_VOLUME);
        return core::ptr::null();
    }

    if RtlEqualMemory((*BootSector).FileSystemType.as_ptr() as *const VOID, b"FAT12   \0".as_ptr() as *const VOID, 8) == 0u8 &&
       RtlEqualMemory((*BootSector).FileSystemType.as_ptr() as *const VOID, b"FAT16   \0".as_ptr() as *const VOID, 8) == 0u8 &&
       RtlEqualMemory((*BootSector32).FileSystemType.as_ptr() as *const VOID, b"FAT32   \0".as_ptr() as *const VOID, 8) == 0u8 &&
       RtlEqualMemory((*BootSectorX).FileSystemType.as_ptr() as *const VOID, b"FATX\0".as_ptr() as *const VOID, 4) == 0u8 {
        FrLdrTempFree(Volume as PVOID, TAG_FAT_VOLUME);
        return core::ptr::null();
    }

    let mut FileInformation: FILEINFORMATION = core::mem::zeroed();
    Status = ArcGetFileInformation(DeviceId, &mut FileInformation);
    if Status != ESUCCESS {
        FrLdrTempFree(Volume as PVOID, TAG_FAT_VOLUME);
        return core::ptr::null();
    }

    let mut SectorCount: ULARGE_INTEGER = core::mem::zeroed();
    SectorCount.QuadPart = (FileInformation.EndingAddress.QuadPart - FileInformation.StartingAddress.QuadPart) as u64 ;
    SectorCount.QuadPart /= 512;

    (*Volume).DeviceId = DeviceId;

    if FatOpenVolume(Volume, BootSector, SectorCount.QuadPart) == 0u8 {
        FrLdrTempFree(Volume as PVOID, TAG_FAT_VOLUME);
        return core::ptr::null();
    }

    FatVolumes[DeviceId as usize] = Volume;

    TRACE!("FatMount(%lu) success\n", (DeviceId) as usize);
    if ISFATX(((*Volume).FatType) as u32 ) != 0u8 {
        &FatXFuncTable
    } else {
        &FatFuncTable
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     FAT filesystem driver for FreeLoader
//  * COPYRIGHT:   Copyright 1998-2003 Brian Palmer (brianp@sginet.com)
//  *              Copyright 2009 Hervé Poussineau
//  *              Copyright 2019 Victor Perevertkin (victor.perevertkin@reactos.org)
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(FILESYSTEM);
//
// ULONG    FatDetermineFatType(PFAT_BOOTSECTOR FatBootSector, ULONGLONG PartitionSectorCount);
// PVOID    FatBufferDirectory(PFAT_VOLUME_INFO Volume, ULONG DirectoryStartCluster, ULONG* EntryCountPointer, BOOLEAN RootDirectory);
// ARC_STATUS FatLookupFile(PFAT_VOLUME_INFO Volume, PCSTR FileName, PFAT_FILE_INFO FatFileInfoPointer);
// void    FatParseShortFileName(PCHAR Buffer, PDIRENTRY DirEntry);
// static BOOLEAN FatGetFatEntry(PFAT_VOLUME_INFO Volume, UINT32 Cluster, PUINT32 ClusterPointer);
// static ULONG FatCountClustersInChain(PFAT_VOLUME_INFO Volume, UINT32 StartCluster);
// static BOOLEAN FatReadClusterChain(PFAT_VOLUME_INFO Volume, UINT32 StartClusterNumber, UINT32 NumberOfClusters, PVOID Buffer, PUINT32 LastClusterNumber);
// BOOLEAN    FatReadPartialCluster(PFAT_VOLUME_INFO Volume, ULONG ClusterNumber, ULONG StartingOffset, ULONG Length, PVOID Buffer);
// BOOLEAN    FatReadVolumeSectors(PFAT_VOLUME_INFO Volume, ULONG SectorNumber, ULONG SectorCount, PVOID Buffer);
//
// #define FAT_IS_END_CLUSTER(clnumber)  \
//     (((Volume->FatType == FAT12) && (clnumber >= 0xff8)) || \
//     ((Volume->FatType == FAT16 || Volume->FatType == FATX16) && (clnumber >= 0xfff8)) || \
//     ((Volume->FatType == FAT32 || Volume->FatType == FATX32) && (clnumber >= 0x0ffffff8)))
//
// #define TAG_FAT_CHAIN 'CtaT'
// #define TAG_FAT_FILE 'FtaF'
// #define TAG_FAT_VOLUME 'VtaF'
// #define TAG_FAT_BUFFER 'BtaF'
// #define TAG_FAT_CACHE 'HtaF'
//
// #define FAT_MAX_CACHE_SIZE (256 * 1024) // 256 KiB, note: it should fit maximum FAT12 FAT size (6144 bytes)
//
// typedef struct _FAT_VOLUME_INFO
// {
//     PUCHAR FatCache; /* A part of 1st FAT cached in memory */
//     PULONG FatCacheIndex; /* Cached sector's indexes */
//     ULONG FatCacheSize; /* Size of the cache in sectors */
//     ULONG FatSectorStart; /* Starting sector of 1st FAT table */
//     ULONG ActiveFatSectorStart; /* Starting sector of active FAT table */
//     ULONG SectorsPerFat; /* Sectors per FAT table */
//     ULONG RootDirSectorStart; /* Starting sector of the root directory (non-fat32) */
//     ULONG RootDirSectors; /* Number of sectors of the root directory (non-fat32) */
//     ULONG RootDirStartCluster; /* Starting cluster number of the root directory (fat32 only) */
//     ULONG DataSectorStart; /* Starting sector of the data area */
//     // ULONG NumberOfClusters; /* Number of clusters of the data area */
//     ULONG DeviceId;
//     ULONG TotalSectors; /* Total number of sectors on the volume */
//     UINT16 BytesPerSector; /* Number of bytes per sector */
//     UINT8 SectorsPerCluster; /* Number of sectors per cluster */
//     UINT8 NumberOfFats; /* Number of FAT tables */
//     UINT8 FatType; /* FAT12, FAT16, FAT32, FATX16 or FATX32 */
// } FAT_VOLUME_INFO;
//
// PFAT_VOLUME_INFO FatVolumes[MAX_FDS];
//
// VOID FatSwapFatBootSector(PFAT_BOOTSECTOR Obj)
// {
//     SW(Obj, BytesPerSector);
//     SW(Obj, ReservedSectors);
//     SW(Obj, RootDirEntries);
//     SW(Obj, TotalSectors);
//     SW(Obj, SectorsPerFat);
//     SW(Obj, SectorsPerTrack);
//     SW(Obj, NumberOfHeads);
//     SD(Obj, HiddenSectors);
//     SD(Obj, TotalSectorsBig);
//     SD(Obj, VolumeSerialNumber);
//     SW(Obj, BootSectorMagic);
// }
//
// VOID FatSwapFat32BootSector(PFAT32_BOOTSECTOR Obj)
// {
//     SW(Obj, BytesPerSector);
//     SW(Obj, ReservedSectors);
//     SW(Obj, RootDirEntries);
//     SW(Obj, TotalSectors);
//     SW(Obj, SectorsPerFat);
//     SW(Obj, NumberOfHeads);
//     SD(Obj, HiddenSectors);
//     SD(Obj, TotalSectorsBig);
//     SD(Obj, SectorsPerFatBig);
//     SW(Obj, ExtendedFlags);
//     SW(Obj, FileSystemVersion);
//     SD(Obj, RootDirStartCluster);
//     SW(Obj, FsInfo);
//     SW(Obj, BackupBootSector);
//     SD(Obj, VolumeSerialNumber);
//     SW(Obj, BootSectorMagic);
// }
//
// VOID FatSwapFatXBootSector(PFATX_BOOTSECTOR Obj)
// {
//     SD(Obj, VolumeSerialNumber);
//     SD(Obj, SectorsPerCluster);
//     SW(Obj, NumberOfFats);
// }
//
// VOID FatSwapDirEntry(PDIRENTRY Obj)
// {
//     SW(Obj, CreateTime);
//     SW(Obj, CreateDate);
//     SW(Obj, LastAccessDate);
//     SW(Obj, ClusterHigh);
//     SW(Obj, Time);
//     SW(Obj, Date);
//     SW(Obj, ClusterLow);
//     SD(Obj, Size);
// }
//
// VOID FatSwapLFNDirEntry(PLFN_DIRENTRY Obj)
// {
//     int i;
//     SW(Obj, StartCluster);
//     for(i = 0; i < 5; i++)
//         Obj->Name0_4[i] = SWAPW(Obj->Name0_4[i]);
//     for(i = 0; i < 6; i++)
//         Obj->Name5_10[i] = SWAPW(Obj->Name5_10[i]);
//     for(i = 0; i < 2; i++)
//         Obj->Name11_12[i] = SWAPW(Obj->Name11_12[i]);
// }
//
// VOID FatSwapFatXDirEntry(PFATX_DIRENTRY Obj)
// {
//     SD(Obj, StartCluster);
//     SD(Obj, Size);
//     SW(Obj, Time);
//     SW(Obj, Date);
//     SW(Obj, CreateTime);
//     SW(Obj, CreateDate);
//     SW(Obj, LastAccessTime);
//     SW(Obj, LastAccessDate);
// }
//
// /**
//  * @brief
//  * Returns the number of clusters of the data area of the FAT volume.
//  * This value is computed by taking the total sectors on the disk, subtracting
//  * up to the first file area sector, then dividing by the sectors per cluster count.
//  *
//  * @note
//  * Relies on the FAT_VOLUME_INFO fields already computed by FatOpenVolume().
//  *
//  * @see
//  * https://github.com/reactos/reactos/blob/435482912c6dc0aaa4371e37b7336b2b1291e65f/drivers/filesystems/fastfat/fat.h#L460
//  * https://github.com/reactos/reactos/blob/435482912c6dc0aaa4371e37b7336b2b1291e65f/drivers/filesystems/vfatfs/fsctl.c#L24
//  **/
// FORCEINLINE
// ULONG
// FatNumberOfClusters(
//     _In_ PFAT_VOLUME_INFO Volume)
// {
//     return ((Volume->TotalSectors - Volume->DataSectorStart) / Volume->SectorsPerCluster);
// }
//
// BOOLEAN FatOpenVolume(PFAT_VOLUME_INFO Volume, PFAT_BOOTSECTOR BootSector, ULONGLONG PartitionSectorCount)
// {
//     char ErrMsg[80];
//     ULONG FatSize, i;
//     PFAT_BOOTSECTOR    FatVolumeBootSector;
//     PFAT32_BOOTSECTOR Fat32VolumeBootSector;
//     PFATX_BOOTSECTOR FatXVolumeBootSector;
//
//     TRACE("FatOpenVolume() DeviceId = %d\n", (Volume->DeviceId) as usize);
//
//     //
//     // Allocate the memory to hold the boot sector
//     //
//     FatVolumeBootSector = (PFAT_BOOTSECTOR)BootSector;
//     Fat32VolumeBootSector = (PFAT32_BOOTSECTOR)BootSector;
//     FatXVolumeBootSector = (PFATX_BOOTSECTOR)BootSector;
//
//     // Get the FAT type
//     Volume->FatType = FatDetermineFatType(FatVolumeBootSector, PartitionSectorCount);
//
//     // Dump boot sector (and swap it for big endian systems)
//     TRACE("Dumping boot sector:\n");
//     if (ISFATX(Volume->FatType))
//     {
//         FatSwapFatXBootSector(FatXVolumeBootSector);
//         TRACE("sizeof(FATX_BOOTSECTOR) = 0x%x.\n", (sizeof(FATX_BOOTSECTOR)) as usize);
//
//         TRACE("FileSystemType: %c%c%c%c.\n", (FatXVolumeBootSector->FileSystemType[0]) as usize, (FatXVolumeBootSector->FileSystemType[1]) as usize, (FatXVolumeBootSector->FileSystemType[2]) as usize, (FatXVolumeBootSector->FileSystemType[3]) as usize);
//         TRACE("VolumeSerialNumber: 0x%x\n", (FatXVolumeBootSector->VolumeSerialNumber) as usize);
//         TRACE("SectorsPerCluster: %d\n", (FatXVolumeBootSector->SectorsPerCluster) as usize);
//         TRACE("NumberOfFats: %d\n", (FatXVolumeBootSector->NumberOfFats) as usize);
//         TRACE("Unknown: 0x%x\n", (FatXVolumeBootSector->Unknown) as usize);
//
//         TRACE("FatType %s\n", (Volume->FatType == FATX16 ? "FATX16" : "FATX32") as usize);
//
//     }
//     else if (Volume->FatType == FAT32)
//     {
//         FatSwapFat32BootSector(Fat32VolumeBootSector);
//         TRACE("sizeof(FAT32_BOOTSECTOR) = 0x%x.\n", (sizeof(FAT32_BOOTSECTOR)) as usize);
//
//         TRACE("JumpBoot: 0x%x 0x%x 0x%x\n", (Fat32VolumeBootSector->JumpBoot[0]) as usize, (Fat32VolumeBootSector->JumpBoot[1]) as usize, (Fat32VolumeBootSector->JumpBoot[2]) as usize);
//         TRACE("OemName: %c%c%c%c%c%c%c%c\n", (Fat32VolumeBootSector->OemName[0]) as usize, (Fat32VolumeBootSector->OemName[1]) as usize, (Fat32VolumeBootSector->OemName[2]) as usize, (Fat32VolumeBootSector->OemName[3]) as usize, (Fat32VolumeBootSector->OemName[4]) as usize, (Fat32VolumeBootSector->OemName[5]) as usize, (Fat32VolumeBootSector->OemName[6]) as usize, (Fat32VolumeBootSector->OemName[7]) as usize);
//         TRACE("BytesPerSector: %d\n", (Fat32VolumeBootSector->BytesPerSector) as usize);
//         TRACE("SectorsPerCluster: %d\n", (Fat32VolumeBootSector->SectorsPerCluster) as usize);
//         TRACE("ReservedSectors: %d\n", (Fat32VolumeBootSector->ReservedSectors) as usize);
//         TRACE("NumberOfFats: %d\n", (Fat32VolumeBootSector->NumberOfFats) as usize);
//         TRACE("RootDirEntries: %d\n", (Fat32VolumeBootSector->RootDirEntries) as usize);
//         TRACE("TotalSectors: %d\n", (Fat32VolumeBootSector->TotalSectors) as usize);
//         TRACE("MediaDescriptor: 0x%x\n", (Fat32VolumeBootSector->MediaDescriptor) as usize);
//         TRACE("SectorsPerFat: %d\n", (Fat32VolumeBootSector->SectorsPerFat) as usize);
//         TRACE("SectorsPerTrack: %d\n", (Fat32VolumeBootSector->SectorsPerTrack) as usize);
//         TRACE("NumberOfHeads: %d\n", (Fat32VolumeBootSector->NumberOfHeads) as usize);
//         TRACE("HiddenSectors: %d\n", (Fat32VolumeBootSector->HiddenSectors) as usize);
//         TRACE("TotalSectorsBig: %d\n", (Fat32VolumeBootSector->TotalSectorsBig) as usize);
//         TRACE("SectorsPerFatBig: %d\n", (Fat32VolumeBootSector->SectorsPerFatBig) as usize);
//         TRACE("ExtendedFlags: 0x%x\n", (Fat32VolumeBootSector->ExtendedFlags) as usize);
//         TRACE("FileSystemVersion: 0x%x\n", (Fat32VolumeBootSector->FileSystemVersion) as usize);
//         TRACE("RootDirStartCluster: %d\n", (Fat32VolumeBootSector->RootDirStartCluster) as usize);
//         TRACE("FsInfo: %d\n", (Fat32VolumeBootSector->FsInfo) as usize);
//         TRACE("BackupBootSector: %d\n", (Fat32VolumeBootSector->BackupBootSector) as usize);
//         TRACE("Reserved: 0x%x\n", (Fat32VolumeBootSector->Reserved) as usize);
//         TRACE("DriveNumber: 0x%x\n", (Fat32VolumeBootSector->DriveNumber) as usize);
//         TRACE("Reserved1: 0x%x\n", (Fat32VolumeBootSector->Reserved1) as usize);
//         TRACE("BootSignature: 0x%x\n", (Fat32VolumeBootSector->BootSignature) as usize);
//         TRACE("VolumeSerialNumber: 0x%x\n", (Fat32VolumeBootSector->VolumeSerialNumber) as usize);
//         TRACE("VolumeLabel: %c%c%c%c%c%c%c%c%c%c%c\n", (Fat32VolumeBootSector->VolumeLabel[0]) as usize, (Fat32VolumeBootSector->VolumeLabel[1]) as usize, (Fat32VolumeBootSector->VolumeLabel[2]) as usize, (Fat32VolumeBootSector->VolumeLabel[3]) as usize, (Fat32VolumeBootSector->VolumeLabel[4]) as usize, (Fat32VolumeBootSector->VolumeLabel[5]) as usize, (Fat32VolumeBootSector->VolumeLabel[6]) as usize, (Fat32VolumeBootSector->VolumeLabel[7]) as usize, (Fat32VolumeBootSector->VolumeLabel[8]) as usize, (Fat32VolumeBootSector->VolumeLabel[9]) as usize, (Fat32VolumeBootSector->VolumeLabel[10]) as usize);
//         TRACE("FileSystemType: %c%c%c%c%c%c%c%c\n", (Fat32VolumeBootSector->FileSystemType[0]) as usize, (Fat32VolumeBootSector->FileSystemType[1]) as usize, (Fat32VolumeBootSector->FileSystemType[2]) as usize, (Fat32VolumeBootSector->FileSystemType[3]) as usize, (Fat32VolumeBootSector->FileSystemType[4]) as usize, (Fat32VolumeBootSector->FileSystemType[5]) as usize, (Fat32VolumeBootSector->FileSystemType[6]) as usize, (Fat32VolumeBootSector->FileSystemType[7]) as usize);
//         TRACE("BootSectorMagic: 0x%x\n", (Fat32VolumeBootSector->BootSectorMagic) as usize);
//     }
//     else
//     {
//         FatSwapFatBootSector(FatVolumeBootSector);
//         TRACE("sizeof(FAT_BOOTSECTOR) = 0x%x.\n", (sizeof(FAT_BOOTSECTOR)) as usize);
//
//         TRACE("JumpBoot: 0x%x 0x%x 0x%x\n", (FatVolumeBootSector->JumpBoot[0]) as usize, (FatVolumeBootSector->JumpBoot[1]) as usize, (FatVolumeBootSector->JumpBoot[2]) as usize);
//         TRACE("OemName: %c%c%c%c%c%c%c%c\n", (FatVolumeBootSector->OemName[0]) as usize, (FatVolumeBootSector->OemName[1]) as usize, (FatVolumeBootSector->OemName[2]) as usize, (FatVolumeBootSector->OemName[3]) as usize, (FatVolumeBootSector->OemName[4]) as usize, (FatVolumeBootSector->OemName[5]) as usize, (FatVolumeBootSector->OemName[6]) as usize, (FatVolumeBootSector->OemName[7]) as usize);
//         TRACE("BytesPerSector: %d\n", (FatVolumeBootSector->BytesPerSector) as usize);
//         TRACE("SectorsPerCluster: %d\n", (FatVolumeBootSector->SectorsPerCluster) as usize);
//         TRACE("ReservedSectors: %d\n", (FatVolumeBootSector->ReservedSectors) as usize);
//         TRACE("NumberOfFats: %d\n", (FatVolumeBootSector->NumberOfFats) as usize);
//         TRACE("RootDirEntries: %d\n", (FatVolumeBootSector->RootDirEntries) as usize);
//         TRACE("TotalSectors: %d\n", (FatVolumeBootSector->TotalSectors) as usize);
//         TRACE("MediaDescriptor: 0x%x\n", (FatVolumeBootSector->MediaDescriptor) as usize);
//         TRACE("SectorsPerFat: %d\n", (FatVolumeBootSector->SectorsPerFat) as usize);
//         TRACE("SectorsPerTrack: %d\n", (FatVolumeBootSector->SectorsPerTrack) as usize);
//         TRACE("NumberOfHeads: %d\n", (FatVolumeBootSector->NumberOfHeads) as usize);
//         TRACE("HiddenSectors: %d\n", (FatVolumeBootSector->HiddenSectors) as usize);
//         TRACE("TotalSectorsBig: %d\n", (FatVolumeBootSector->TotalSectorsBig) as usize);
//         TRACE("DriveNumber: 0x%x\n", (FatVolumeBootSector->DriveNumber) as usize);
//         TRACE("Reserved1: 0x%x\n", (FatVolumeBootSector->Reserved1) as usize);
//         TRACE("BootSignature: 0x%x\n", (FatVolumeBootSector->BootSignature) as usize);
//         TRACE("VolumeSerialNumber: 0x%x\n", (FatVolumeBootSector->VolumeSerialNumber) as usize);
//         TRACE("VolumeLabel: %c%c%c%c%c%c%c%c%c%c%c\n", (FatVolumeBootSector->VolumeLabel[0]) as usize, (FatVolumeBootSector->VolumeLabel[1]) as usize, (FatVolumeBootSector->VolumeLabel[2]) as usize, (FatVolumeBootSector->VolumeLabel[3]) as usize, (FatVolumeBootSector->VolumeLabel[4]) as usize, (FatVolumeBootSector->VolumeLabel[5]) as usize, (FatVolumeBootSector->VolumeLabel[6]) as usize, (FatVolumeBootSector->VolumeLabel[7]) as usize, (FatVolumeBootSector->VolumeLabel[8]) as usize, (FatVolumeBootSector->VolumeLabel[9]) as usize, (FatVolumeBootSector->VolumeLabel[10]) as usize);
//         TRACE("FileSystemType: %c%c%c%c%c%c%c%c\n", (FatVolumeBootSector->FileSystemType[0]) as usize, (FatVolumeBootSector->FileSystemType[1]) as usize, (FatVolumeBootSector->FileSystemType[2]) as usize, (FatVolumeBootSector->FileSystemType[3]) as usize, (FatVolumeBootSector->FileSystemType[4]) as usize, (FatVolumeBootSector->FileSystemType[5]) as usize, (FatVolumeBootSector->FileSystemType[6]) as usize, (FatVolumeBootSector->FileSystemType[7]) as usize);
//         TRACE("BootSectorMagic: 0x%x\n", (FatVolumeBootSector->BootSectorMagic) as usize);
//     }
//
//     //
//     // Check the boot sector magic
//     //
//     if (! ISFATX(Volume->FatType) && FatVolumeBootSector->BootSectorMagic != 0xaa55)
//     {
//         sprintf(ErrMsg, "Invalid boot sector magic (expected 0xaa55 found 0x%x)",
//                 FatVolumeBootSector->BootSectorMagic);
//         FileSystemError(ErrMsg);
//         return FALSE;
//     }
//
//     //
//     // Check the FAT cluster size
//     // We do not support clusters bigger than 64k
//     //
//     if ((ISFATX(Volume->FatType) && 64 * 1024 < FatXVolumeBootSector->SectorsPerCluster * 512) ||
//        (! ISFATX(Volume->FatType) && 64 * 1024 < FatVolumeBootSector->SectorsPerCluster * FatVolumeBootSector->BytesPerSector))
//     {
//         FileSystemError("This file system has cluster sizes bigger than 64k.\nFreeLoader does not support this.");
//         return FALSE;
//     }
//
//     if (ISFATX(Volume->FatType))
//     {
//         Volume->TotalSectors = PartitionSectorCount;
//     }
//     else
//     {
//         /*
//          * After DOS 4.0, at least one of TotalSectors or TotalSectorsBig will be zero.
//          * In DOS version 3.2 or before, both of these might contain some value,
//          * because, before 3.2, there was no TotalSectorsBig entry. Thus some disks
//          * might have an unexpected value in the field, and we will use TotalSectorsBig
//          * only if TotalSectors equals zero.
//          */
//         C_ASSERT(FIELD_OFFSET(FAT_BOOTSECTOR, TotalSectors) ==
//                  FIELD_OFFSET(FAT32_BOOTSECTOR, TotalSectors));
//         C_ASSERT(FIELD_OFFSET(FAT_BOOTSECTOR, TotalSectorsBig) ==
//                  FIELD_OFFSET(FAT32_BOOTSECTOR, TotalSectorsBig));
//         Volume->TotalSectors = (FatVolumeBootSector->TotalSectors ? FatVolumeBootSector->TotalSectors
//                                                                   : FatVolumeBootSector->TotalSectorsBig);
//     }
//
//     /* Get the sectors per FAT, root directory sector start, and data sector start */
//     if (ISFATX(Volume->FatType))
//     {
//         Volume->BytesPerSector = SECTOR_SIZE;
//         Volume->SectorsPerCluster = SWAPD(FatXVolumeBootSector->SectorsPerCluster);
//         Volume->FatSectorStart = 4096 / Volume->BytesPerSector;
//         Volume->ActiveFatSectorStart = Volume->FatSectorStart;
//         Volume->NumberOfFats = 1; // FatXVolumeBootSector->NumberOfFats;
//         FatSize = (ULONG)(PartitionSectorCount / Volume->SectorsPerCluster *
//                   (Volume->FatType == FATX16 ? 2 : 4));
//         Volume->SectorsPerFat = ROUND_UP(FatSize, 4096) / Volume->BytesPerSector;
//
//         Volume->RootDirSectorStart = Volume->FatSectorStart + Volume->NumberOfFats * Volume->SectorsPerFat;
//         Volume->RootDirSectors = FatXVolumeBootSector->SectorsPerCluster;
//
//         Volume->DataSectorStart = Volume->RootDirSectorStart + Volume->RootDirSectors;
//     }
//     else if (Volume->FatType != FAT32)
//     {
//         Volume->BytesPerSector = FatVolumeBootSector->BytesPerSector;
//         Volume->SectorsPerCluster = FatVolumeBootSector->SectorsPerCluster;
//         Volume->FatSectorStart = FatVolumeBootSector->ReservedSectors;
//         Volume->ActiveFatSectorStart = Volume->FatSectorStart;
//         Volume->NumberOfFats = FatVolumeBootSector->NumberOfFats;
//         Volume->SectorsPerFat = FatVolumeBootSector->SectorsPerFat;
//
//         Volume->RootDirSectorStart = Volume->FatSectorStart + Volume->NumberOfFats * Volume->SectorsPerFat;
//         Volume->RootDirSectors = ((FatVolumeBootSector->RootDirEntries * 32) + (Volume->BytesPerSector - 1)) / Volume->BytesPerSector;
//         // Volume->RootDirSectors = (FatVolumeBootSector->RootDirEntries * sizeof(DIRENT) / Volume->BytesPerSector);
//
//         Volume->DataSectorStart = Volume->RootDirSectorStart + Volume->RootDirSectors;
//     }
//     else
//     {
//         Volume->BytesPerSector = Fat32VolumeBootSector->BytesPerSector;
//         Volume->SectorsPerCluster = Fat32VolumeBootSector->SectorsPerCluster;
//         Volume->FatSectorStart = Fat32VolumeBootSector->ReservedSectors;
//         Volume->ActiveFatSectorStart =
//             Volume->FatSectorStart + ((Fat32VolumeBootSector->ExtendedFlags & 0x80) ?
//                                       ((Fat32VolumeBootSector->ExtendedFlags & 0x0f) * Fat32VolumeBootSector->SectorsPerFatBig) : 0);
//         Volume->NumberOfFats = Fat32VolumeBootSector->NumberOfFats;
//         Volume->SectorsPerFat = Fat32VolumeBootSector->SectorsPerFatBig;
//         // Volume->SectorsPerFat = (Fat32VolumeBootSector->SectorsPerFat ? Fat32VolumeBootSector->SectorsPerFat : Fat32VolumeBootSector->SectorsPerFatBig);
//
//         Volume->RootDirStartCluster = Fat32VolumeBootSector->RootDirStartCluster;
//         Volume->DataSectorStart = Volume->FatSectorStart + Volume->NumberOfFats * Volume->SectorsPerFat;
//
//         //
//         // Check version
//         // we only work with version 0
//         //
//         if (Fat32VolumeBootSector->FileSystemVersion != 0)
//         {
//             FileSystemError("FreeLoader is too old to work with this FAT32 filesystem.\nPlease update FreeLoader.");
//             return FALSE;
//         }
//     }
//     // Volume->NumberOfClusters = FatNumberOfClusters(Volume);
//
//     Volume->FatCacheSize = min(Volume->SectorsPerFat, FAT_MAX_CACHE_SIZE / Volume->BytesPerSector);
//     TRACE("FAT cache is %d sectors, %d bytes\n", (Volume->FatCacheSize) as usize, (Volume->FatCacheSize * Volume->BytesPerSector) as usize);
//
//     Volume->FatCache = FrLdrTempAlloc(Volume->FatCacheSize * Volume->BytesPerSector, TAG_FAT_CACHE);
//     if (!Volume->FatCache)
//     {
//         FileSystemError("Cannot allocate memory for FAT cache");
//         return FALSE;
//     }
//
//     Volume->FatCacheIndex = FrLdrTempAlloc(Volume->FatCacheSize * sizeof(*Volume->FatCacheIndex), TAG_FAT_VOLUME);
//     if (!Volume->FatCacheIndex)
//     {
//         FileSystemError("Cannot allocate memory for FAT cache index");
//         FrLdrTempFree(Volume->FatCache, TAG_FAT_CACHE);
//         return FALSE;
//     }
//
//     // read the beginning of the FAT (or the whole one) to cache
//     if (!FatReadVolumeSectors(Volume, Volume->ActiveFatSectorStart, Volume->FatCacheSize, Volume->FatCache))
//     {
//         FileSystemError("Error when reading FAT cache");
//         FrLdrTempFree(Volume->FatCache, TAG_FAT_CACHE);
//         FrLdrTempFree(Volume->FatCacheIndex, TAG_FAT_VOLUME);
//         return FALSE;
//     }
//
//     // fill the index with sector numbers
//     for (i = 0; i < Volume->FatCacheSize; i++)
//     {
//         Volume->FatCacheIndex[i] = Volume->ActiveFatSectorStart + i;
//     }
//
//     return TRUE;
// }
//
// ULONG FatDetermineFatType(PFAT_BOOTSECTOR FatBootSector, ULONGLONG PartitionSectorCount)
// {
//     ULONG            RootDirSectors;
//     ULONG            DataSectorCount;
//     ULONG            SectorsPerFat;
//     ULONG            TotalSectors;
//     ULONG            CountOfClusters;
//     PFAT32_BOOTSECTOR    Fat32BootSector = (PFAT32_BOOTSECTOR)FatBootSector;
//     PFATX_BOOTSECTOR    FatXBootSector = (PFATX_BOOTSECTOR)FatBootSector;
//
//     if (0 == strncmp(FatXBootSector->FileSystemType, "FATX", 4))
//     {
//         CountOfClusters = (ULONG)(PartitionSectorCount / FatXBootSector->SectorsPerCluster);
//         if (CountOfClusters < 65525)
//         {
//             /* Volume is FATX16 */
//             return FATX16;
//         }
//         else
//         {
//             /* Volume is FAT32 */
//             return FATX32;
//         }
//     }
//     else
//     {
//         RootDirSectors = ((SWAPW(FatBootSector->RootDirEntries) * 32) + (SWAPW(FatBootSector->BytesPerSector) - 1)) / SWAPW(FatBootSector->BytesPerSector);
//         SectorsPerFat = SWAPW(FatBootSector->SectorsPerFat) ? SWAPW(FatBootSector->SectorsPerFat) : SWAPD(Fat32BootSector->SectorsPerFatBig);
//         TotalSectors = SWAPW(FatBootSector->TotalSectors) ? SWAPW(FatBootSector->TotalSectors) : SWAPD(FatBootSector->TotalSectorsBig);
//         DataSectorCount = TotalSectors - (SWAPW(FatBootSector->ReservedSectors) + (FatBootSector->NumberOfFats * SectorsPerFat) + RootDirSectors);
//
// //mjl
//         if (FatBootSector->SectorsPerCluster == 0)
//             CountOfClusters = 0;
//         else
//             CountOfClusters = DataSectorCount / FatBootSector->SectorsPerCluster;
//
//         if (CountOfClusters < 4085)
//         {
//             /* Volume is FAT12 */
//             return FAT12;
//         }
//         else if (CountOfClusters < 65525)
//         {
//             /* Volume is FAT16 */
//             return FAT16;
//         }
//         else
//         {
//             /* Volume is FAT32 */
//             return FAT32;
//         }
//     }
// }
//
// typedef struct _DIRECTORY_BUFFER
// {
//     LIST_ENTRY Link;
//     PVOID Volume;
//     ULONG DirectoryStartCluster;
//     ULONG DirectorySize;
//     UCHAR Data[];
// } DIRECTORY_BUFFER, *PDIRECTORY_BUFFER;
//
// LIST_ENTRY DirectoryBufferListHead = {&DirectoryBufferListHead, &DirectoryBufferListHead};
//
// PVOID FatBufferDirectory(PFAT_VOLUME_INFO Volume, ULONG DirectoryStartCluster, ULONG *DirectorySize, BOOLEAN RootDirectory)
// {
//     PDIRECTORY_BUFFER DirectoryBuffer;
//     PLIST_ENTRY Entry;
//
//     TRACE("FatBufferDirectory() DirectoryStartCluster = %d RootDirectory = %s\n", (DirectoryStartCluster) as usize, ((RootDirectory ? "TRUE" : "FALSE")) as usize);
//
//     /*
//      * For FAT32, the root directory is nothing special. We can treat it the same
//      * as a subdirectory.
//      */
//     if (RootDirectory && Volume->FatType == FAT32)
//     {
//         DirectoryStartCluster = Volume->RootDirStartCluster;
//         RootDirectory = FALSE;
//     }
//
//     /* Search the list for a match */
//     for (Entry = DirectoryBufferListHead.Flink;
//          Entry != &DirectoryBufferListHead;
//          Entry = Entry->Flink)
//     {
//         DirectoryBuffer = CONTAINING_RECORD(Entry, DIRECTORY_BUFFER, Link);
//
//         /* Check if it matches */
//         if ((DirectoryBuffer->Volume == Volume) &&
//             (DirectoryBuffer->DirectoryStartCluster == DirectoryStartCluster))
//         {
//             TRACE("Found cached buffer\n");
//             *DirectorySize = DirectoryBuffer->DirectorySize;
//             return DirectoryBuffer->Data;
//         }
//     }
//
//     //
//     // Calculate the size of the directory
//     //
//     if (RootDirectory)
//     {
//         *DirectorySize = Volume->RootDirSectors * Volume->BytesPerSector;
//     }
//     else
//     {
//         *DirectorySize = FatCountClustersInChain(Volume, DirectoryStartCluster) * Volume->SectorsPerCluster * Volume->BytesPerSector;
//     }
//
//     //
//     // Attempt to allocate memory for directory buffer
//     //
//     TRACE("Trying to allocate (DirectorySize) %d bytes.\n", (*DirectorySize) as usize);
//     DirectoryBuffer = FrLdrTempAlloc(*DirectorySize + sizeof(DIRECTORY_BUFFER),
//                                      TAG_FAT_BUFFER);
//
//     if (DirectoryBuffer == NULL)
//     {
//         return NULL;
//     }
//
//     //
//     // Now read directory contents into DirectoryBuffer
//     //
//     if (RootDirectory)
//     {
//         if (!FatReadVolumeSectors(Volume, Volume->RootDirSectorStart, Volume->RootDirSectors, DirectoryBuffer->Data))
//         {
//             FrLdrTempFree(DirectoryBuffer, TAG_FAT_BUFFER);
//             return NULL;
//         }
//     }
//     else
//     {
//         if (!FatReadClusterChain(Volume, DirectoryStartCluster, 0xFFFFFFFF, DirectoryBuffer->Data, NULL))
//         {
//             FrLdrTempFree(DirectoryBuffer, TAG_FAT_BUFFER);
//             return NULL;
//         }
//     }
//
//     /* Enqueue it in the list */
//     DirectoryBuffer->Volume = Volume;
//     DirectoryBuffer->DirectoryStartCluster = DirectoryStartCluster;
//     DirectoryBuffer->DirectorySize = *DirectorySize;
//     InsertTailList(&DirectoryBufferListHead, &DirectoryBuffer->Link);
//
//     return DirectoryBuffer->Data;
// }
//
// static BOOLEAN FatSearchDirectoryBufferForFile(PFAT_VOLUME_INFO Volume, PVOID DirectoryBuffer, ULONG DirectorySize, PCHAR FileName, PFAT_FILE_INFO FatFileInfoPointer)
// {
//     ULONG        EntryCount;
//     ULONG        CurrentEntry;
//     CHAR        LfnNameBuffer[265];
//     CHAR        ShortNameBuffer[20];
//     ULONG        StartCluster;
//     DIRENTRY        OurDirEntry;
//     LFN_DIRENTRY    OurLfnDirEntry;
//     PDIRENTRY    DirEntry = &OurDirEntry;
//     PLFN_DIRENTRY    LfnDirEntry = &OurLfnDirEntry;
//
//     EntryCount = DirectorySize / sizeof(DIRENTRY);
//
//     TRACE("FatSearchDirectoryBufferForFile() DirectoryBuffer = 0x%x EntryCount = %d FileName = %s\n", (DirectoryBuffer) as usize, (EntryCount) as usize, (FileName) as usize);
//
//     RtlZeroMemory(ShortNameBuffer, 13 * sizeof(CHAR));
//     RtlZeroMemory(LfnNameBuffer, 261 * sizeof(CHAR));
//
//     for (CurrentEntry=0; CurrentEntry<EntryCount; CurrentEntry++, DirectoryBuffer = ((PDIRENTRY)DirectoryBuffer)+1)
//     {
//         OurLfnDirEntry = *((PLFN_DIRENTRY) DirectoryBuffer);
//         FatSwapLFNDirEntry(LfnDirEntry);
//         OurDirEntry = *((PDIRENTRY) DirectoryBuffer);
//         FatSwapDirEntry(DirEntry);
//
//         //TRACE("Dumping directory entry %d:\n", (CurrentEntry) as usize);
//         //DbgDumpBuffer(DPRINT_FILESYSTEM, DirEntry, sizeof(DIRENTRY));
//
//         //
//         // Check if this is the last file in the directory
//         // If DirEntry[0] == 0x00 then that means all the
//         // entries after this one are unused. If this is the
//         // last entry then we didn't find the file in this directory.
//         //
//         if (DirEntry->FileName[0] == '\0')
//         {
//             return FALSE;
//         }
//
//         //
//         // Check if this is a deleted entry or not
//         //
//         if (DirEntry->FileName[0] == '\xE5')
//         {
//             RtlZeroMemory(ShortNameBuffer, 13 * sizeof(CHAR));
//             RtlZeroMemory(LfnNameBuffer, 261 * sizeof(CHAR));
//             continue;
//         }
//
//         //
//         // Check if this is a LFN entry
//         // If so it needs special handling
//         //
//         if (DirEntry->Attr == FAT_ATTR_LONG_NAME)
//         {
//             //
//             // Check to see if this is a deleted LFN entry, if so continue
//             //
//             if (LfnDirEntry->SequenceNumber & 0x80)
//             {
//                 continue;
//             }
//
//             //
//             // Mask off high two bits of sequence number
//             // and make the sequence number zero-based
//             //
//             LfnDirEntry->SequenceNumber &= 0x3F;
//             LfnDirEntry->SequenceNumber--;
//
//             //
//             // Get all 13 LFN entry characters
//             //
//             if (LfnDirEntry->Name0_4[0] != 0xFFFF)
//             {
//                 LfnNameBuffer[0 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name0_4[0];
//             }
//             if (LfnDirEntry->Name0_4[1] != 0xFFFF)
//             {
//                 LfnNameBuffer[1 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name0_4[1];
//             }
//             if (LfnDirEntry->Name0_4[2] != 0xFFFF)
//             {
//                 LfnNameBuffer[2 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name0_4[2];
//             }
//             if (LfnDirEntry->Name0_4[3] != 0xFFFF)
//             {
//                 LfnNameBuffer[3 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name0_4[3];
//             }
//             if (LfnDirEntry->Name0_4[4] != 0xFFFF)
//             {
//                 LfnNameBuffer[4 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name0_4[4];
//             }
//             if (LfnDirEntry->Name5_10[0] != 0xFFFF)
//             {
//                 LfnNameBuffer[5 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name5_10[0];
//             }
//             if (LfnDirEntry->Name5_10[1] != 0xFFFF)
//             {
//                 LfnNameBuffer[6 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name5_10[1];
//             }
//             if (LfnDirEntry->Name5_10[2] != 0xFFFF)
//             {
//                 LfnNameBuffer[7 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name5_10[2];
//             }
//             if (LfnDirEntry->Name5_10[3] != 0xFFFF)
//             {
//                 LfnNameBuffer[8 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name5_10[3];
//             }
//             if (LfnDirEntry->Name5_10[4] != 0xFFFF)
//             {
//                 LfnNameBuffer[9 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name5_10[4];
//             }
//             if (LfnDirEntry->Name5_10[5] != 0xFFFF)
//             {
//                 LfnNameBuffer[10 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name5_10[5];
//             }
//             if (LfnDirEntry->Name11_12[0] != 0xFFFF)
//             {
//                 LfnNameBuffer[11 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name11_12[0];
//             }
//             if (LfnDirEntry->Name11_12[1] != 0xFFFF)
//             {
//                 LfnNameBuffer[12 + (LfnDirEntry->SequenceNumber * 13)] = (UCHAR)LfnDirEntry->Name11_12[1];
//             }
//
//             //TRACE("Dumping long name buffer:\n");
//             //DbgDumpBuffer(DPRINT_FILESYSTEM, LfnNameBuffer, 260);
//
//             continue;
//         }
//
//         //
//         // Check for the volume label attribute
//         // and skip over this entry if found
//         //
//         if (DirEntry->Attr & FAT_ATTR_VOLUMENAME)
//         {
//             RtlZeroMemory(ShortNameBuffer, 13 * sizeof(UCHAR));
//             RtlZeroMemory(LfnNameBuffer, 261 * sizeof(UCHAR));
//             continue;
//         }
//
//         //
//         // If we get here then we've found a short file name
//         // entry and LfnNameBuffer contains the long file
//         // name or zeroes. All we have to do now is see if the
//         // file name matches either the short or long file name
//         // and fill in the FAT_FILE_INFO structure if it does
//         // or zero our buffers and continue looking.
//         //
//
//         //
//         // Get short file name
//         //
//         FatParseShortFileName(ShortNameBuffer, DirEntry);
//
//         //TRACE("Entry: %d LFN = %s\n", (CurrentEntry) as usize, (LfnNameBuffer) as usize);
//         //TRACE("Entry: %d DOS name = %s\n", (CurrentEntry) as usize, (ShortNameBuffer) as usize);
//
//         //
//         // See if the file name matches either the short or long name
//         //
//         if ((_stricmp(FileName, LfnNameBuffer) == 0) || (_stricmp(FileName, ShortNameBuffer) == 0))
//         {
//             //
//             // We found the entry, now fill in the FAT_FILE_INFO struct
//             //
//             FatFileInfoPointer->Attributes = DirEntry->Attr;
//             FatFileInfoPointer->FileSize = DirEntry->Size;
//             FatFileInfoPointer->FilePointer = 0;
//             StartCluster = ((ULONG)DirEntry->ClusterHigh << 16) + DirEntry->ClusterLow;
//             FatFileInfoPointer->CurrentCluster = StartCluster;
//             FatFileInfoPointer->StartCluster = StartCluster;
//
//             TRACE("MSDOS Directory Entry:\n");
//             TRACE("FileName[11] = %c%c%c%c%c%c%c%c%c%c%c\n", (//                   DirEntry->FileName[0]) as usize, (DirEntry->FileName[1]) as usize, (DirEntry->FileName[2]) as usize, (DirEntry->FileName[3]) as usize, (DirEntry->FileName[4]) as usize, (//                   DirEntry->FileName[5]) as usize, (DirEntry->FileName[6]) as usize, (DirEntry->FileName[7]) as usize, (DirEntry->FileName[8]) as usize, (DirEntry->FileName[9]) as usize, (DirEntry->FileName[10]) as usize);
//             TRACE("Attr = 0x%x\n", (DirEntry->Attr) as usize);
//             TRACE("ReservedNT = 0x%x\n", (DirEntry->ReservedNT) as usize);
//             TRACE("TimeInTenths = %d\n", (DirEntry->TimeInTenths) as usize);
//             TRACE("CreateTime = %d\n", (DirEntry->CreateTime) as usize);
//             TRACE("CreateDate = %d\n", (DirEntry->CreateDate) as usize);
//             TRACE("LastAccessDate = %d\n", (DirEntry->LastAccessDate) as usize);
//             TRACE("ClusterHigh = 0x%x\n", (DirEntry->ClusterHigh) as usize);
//             TRACE("Time = %d\n", (DirEntry->Time) as usize);
//             TRACE("Date = %d\n", (DirEntry->Date) as usize);
//             TRACE("ClusterLow = 0x%x\n", (DirEntry->ClusterLow) as usize);
//             TRACE("Size = %d\n", (DirEntry->Size) as usize);
//             TRACE("StartCluster = 0x%x\n", (StartCluster) as usize);
//
//             return TRUE;
//         }
//
//         //
//         // Nope, no match - zero buffers and continue looking
//         //
//         RtlZeroMemory(ShortNameBuffer, 13 * sizeof(UCHAR));
//         RtlZeroMemory(LfnNameBuffer, 261 * sizeof(UCHAR));
//     }
//
//     return FALSE;
// }
//
// static BOOLEAN FatXSearchDirectoryBufferForFile(PFAT_VOLUME_INFO Volume, PVOID DirectoryBuffer, ULONG DirectorySize, PCHAR FileName, PFAT_FILE_INFO FatFileInfoPointer)
// {
//     ULONG        EntryCount;
//     ULONG        CurrentEntry;
//     SIZE_T        FileNameLen;
//     FATX_DIRENTRY    OurDirEntry;
//     PFATX_DIRENTRY    DirEntry = &OurDirEntry;
//
//     EntryCount = DirectorySize / sizeof(FATX_DIRENTRY);
//
//     TRACE("FatXSearchDirectoryBufferForFile() DirectoryBuffer = 0x%x EntryCount = %d FileName = %s\n", (DirectoryBuffer) as usize, (EntryCount) as usize, (FileName) as usize);
//
//     FileNameLen = strlen(FileName);
//
//     for (CurrentEntry = 0; CurrentEntry < EntryCount; CurrentEntry++, DirectoryBuffer = ((PFATX_DIRENTRY)DirectoryBuffer)+1)
//     {
//         OurDirEntry = *(PFATX_DIRENTRY) DirectoryBuffer;
//         FatSwapFatXDirEntry(&OurDirEntry);
//         if (0xff == DirEntry->FileNameSize)
//         {
//             break;
//         }
//         if (0xe5 == DirEntry->FileNameSize)
//         {
//             continue;
//         }
//         if ((FileNameLen == DirEntry->FileNameSize) &&
//             (_strnicmp(FileName, DirEntry->FileName, FileNameLen) == 0))
//         {
//             /*
//              * We found the entry, now fill in the FAT_FILE_INFO struct
//              */
//             FatFileInfoPointer->Attributes = DirEntry->Attr;
//             FatFileInfoPointer->FileSize = DirEntry->Size;
//             FatFileInfoPointer->FilePointer = 0;
//             FatFileInfoPointer->CurrentCluster = DirEntry->StartCluster;
//             FatFileInfoPointer->StartCluster = DirEntry->StartCluster;
//
//             TRACE("FATX Directory Entry:\n");
//             TRACE("FileNameSize = %d\n", (DirEntry->FileNameSize) as usize);
//             TRACE("Attr = 0x%x\n", (DirEntry->Attr) as usize);
//             TRACE("StartCluster = 0x%x\n", (DirEntry->StartCluster) as usize);
//             TRACE("Size = %d\n", (DirEntry->Size) as usize);
//             TRACE("Time = %d\n", (DirEntry->Time) as usize);
//             TRACE("Date = %d\n", (DirEntry->Date) as usize);
//             TRACE("CreateTime = %d\n", (DirEntry->CreateTime) as usize);
//             TRACE("CreateDate = %d\n", (DirEntry->CreateDate) as usize);
//             TRACE("LastAccessTime = %d\n", (DirEntry->LastAccessTime) as usize);
//             TRACE("LastAccessDate = %d\n", (DirEntry->LastAccessDate) as usize);
//
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// /*
//  * FatLookupFile()
//  * This function searches the file system for the
//  * specified filename and fills in an FAT_FILE_INFO structure
//  * with info describing the file, etc. returns ARC error code
//  */
// ARC_STATUS FatLookupFile(PFAT_VOLUME_INFO Volume, PCSTR FileName, PFAT_FILE_INFO FatFileInfoPointer)
// {
//     ULONG        NumberOfPathParts;
//     ULONG        i;
//     CHAR        PathPart[261];
//     PVOID        DirectoryBuffer;
//     ULONG        DirectoryStartCluster = 0;
//     ULONG        DirectorySize;
//     FAT_FILE_INFO    FatFileInfo;
//
//     TRACE("FatLookupFile() FileName = %s\n", (FileName) as usize);
//
//     RtlZeroMemory(FatFileInfoPointer, sizeof(FAT_FILE_INFO));
//
//     /* Skip leading path separator, if any */
//     if (*FileName == '\\' || *FileName == '/')
//         ++FileName;
//     PathPart[0] = ANSI_NULL;
//
//     //
//     // Figure out how many sub-directories we are nested in
//     //
//     NumberOfPathParts = FsGetNumPathParts(FileName);
//
//     //
//     // Loop once for each part
//     //
//     for (i=0; i<NumberOfPathParts; i++)
//     {
//         //
//         // Get first path part
//         //
//         FsGetFirstNameFromPath(PathPart, FileName);
//
//         //
//         // Advance to the next part of the path
//         //
//         for (; (*FileName != '\\') && (*FileName != '/') && (*FileName != '\0'); FileName++)
//         {
//         }
//         FileName++;
//
//         //
//         // Buffer the directory contents
//         //
//         DirectoryBuffer = FatBufferDirectory(Volume, DirectoryStartCluster, &DirectorySize, (i == 0) );
//         if (DirectoryBuffer == NULL)
//         {
//             return ENOMEM;
//         }
//
//         //
//         // Search for file name in directory
//         //
//         if (ISFATX(Volume->FatType))
//         {
//             if (!FatXSearchDirectoryBufferForFile(Volume, DirectoryBuffer, DirectorySize, PathPart, &FatFileInfo))
//             {
//                 return ENOENT;
//             }
//         }
//         else
//         {
//             if (!FatSearchDirectoryBufferForFile(Volume, DirectoryBuffer, DirectorySize, PathPart, &FatFileInfo))
//             {
//                 return ENOENT;
//             }
//         }
//
//         //
//         // If we have another sub-directory to go then
//         // grab the start cluster and free the fat chain array
//         //
//         if ((i+1) < NumberOfPathParts)
//         {
//             //
//             // Check if current entry is a directory
//             //
//             if (!(FatFileInfo.Attributes & FAT_ATTR_DIRECTORY))
//             {
//                 return ENOTDIR;
//             }
//             DirectoryStartCluster = FatFileInfo.StartCluster;
//         }
//     }
//
//     RtlCopyMemory(FatFileInfoPointer, &FatFileInfo, sizeof(FatFileInfo));
//
//     /* Re-map the attributes to ARC file attributes */
//     FatFileInfoPointer->Attributes = 0;
//     if (FatFileInfo.Attributes & FAT_ATTR_READONLY)
//         FatFileInfoPointer->Attributes |= ReadOnlyFile;
//     if (FatFileInfo.Attributes & FAT_ATTR_HIDDEN)
//         FatFileInfoPointer->Attributes |= HiddenFile;
//     if (FatFileInfo.Attributes & FAT_ATTR_SYSTEM)
//         FatFileInfoPointer->Attributes |= SystemFile;
//     if (FatFileInfo.Attributes & FAT_ATTR_ARCHIVE)
//         FatFileInfoPointer->Attributes |= ArchiveFile;
//     if (FatFileInfo.Attributes & FAT_ATTR_DIRECTORY)
//         FatFileInfoPointer->Attributes |= DirectoryFile;
//
//     /* Copy the file name, perhaps truncated */
//     FatFileInfoPointer->FileNameLength = (ULONG)strlen(PathPart);
//     FatFileInfoPointer->FileNameLength = min(FatFileInfoPointer->FileNameLength, sizeof(FatFileInfoPointer->FileName) - 1);
//     RtlCopyMemory(FatFileInfoPointer->FileName, PathPart, FatFileInfoPointer->FileNameLength);
//
//     return ESUCCESS;
// }
//
// /*
//  * FatParseFileName()
//  * This function parses a directory entry name which
//  * is in the form of "FILE   EXT" and puts it in Buffer
//  * in the form of "file.ext"
//  */
// void FatParseShortFileName(PCHAR Buffer, PDIRENTRY DirEntry)
// {
//     ULONG        Idx;
//
//     Idx = 0;
//     RtlZeroMemory(Buffer, 13);
//
//     //
//     // Fixup first character
//     //
//     if (DirEntry->FileName[0] == 0x05)
//     {
//         DirEntry->FileName[0] = 0xE5;
//     }
//
//     //
//     // Get the file name
//     //
//     while (Idx < 8)
//     {
//         if (DirEntry->FileName[Idx] == ' ')
//         {
//             break;
//         }
//
//         Buffer[Idx] = DirEntry->FileName[Idx];
//         Idx++;
//     }
//
//     //
//     // Get extension
//     //
//     if ((DirEntry->FileName[8] != ' '))
//     {
//         Buffer[Idx++] = '.';
//         Buffer[Idx++] = (DirEntry->FileName[8] == ' ') ? '\0' : DirEntry->FileName[8];
//         Buffer[Idx++] = (DirEntry->FileName[9] == ' ') ? '\0' : DirEntry->FileName[9];
//         Buffer[Idx++] = (DirEntry->FileName[10] == ' ') ? '\0' : DirEntry->FileName[10];
//     }
//
//     //TRACE("FatParseShortFileName() ShortName = %s\n", (Buffer) as usize);
// }
//
// /**
//  * @brief Reads 1-4 sectors from FAT using the cache
//  */
// static
// PUCHAR FatGetFatSector(PFAT_VOLUME_INFO Volume, UINT32 FatSectorNumber)
// {
//     UINT32 SectorNumAbsolute = Volume->ActiveFatSectorStart + FatSectorNumber;
//     UINT32 CacheIndex = FatSectorNumber % Volume->FatCacheSize;
//
//     ASSERT(FatSectorNumber < Volume->SectorsPerFat);
//
//     // cache miss
//     if (Volume->FatCacheIndex[CacheIndex] != SectorNumAbsolute)
//     {
//         UINT32 SectorsToRead = min(Volume->FatCacheSize - CacheIndex, min(Volume->SectorsPerFat - FatSectorNumber, 4));
//         UINT8 i;
//
//         if (!FatReadVolumeSectors(Volume, SectorNumAbsolute, SectorsToRead, &Volume->FatCache[CacheIndex * Volume->BytesPerSector]))
//         {
//             return NULL;
//         }
//
//         for (i = 0; i < SectorsToRead; i++)
//         {
//             Volume->FatCacheIndex[CacheIndex + i] = SectorNumAbsolute + i;
//         }
//
//         TRACE("FAT cache miss: read sector 0x%x from disk\n", (SectorNumAbsolute) as usize);
//     }
//     else
//     {
//         TRACE("FAT cache hit: sector 0x%x present\n", (SectorNumAbsolute) as usize);
//     }
//
//     return &Volume->FatCache[CacheIndex * Volume->BytesPerSector];
// }
//
// /*
//  * FatGetFatEntry()
//  * returns the Fat entry for a given cluster number
//  */
// static
// BOOLEAN FatGetFatEntry(PFAT_VOLUME_INFO Volume, UINT32 Cluster, PUINT32 ClusterPointer)
// {
//     UINT32 FatOffset, ThisFatSecNum, ThisFatEntOffset, fat;
//     PUCHAR ReadBuffer;
//
//     TRACE("FatGetFatEntry() Retrieving FAT entry for cluster %d.\n", (Cluster) as usize);
//
//     switch(Volume->FatType)
//     {
//     case FAT12:
//
//         FatOffset = Cluster + (Cluster / 2);
//         ThisFatSecNum = FatOffset / Volume->BytesPerSector;
//         ThisFatEntOffset = (FatOffset % Volume->BytesPerSector);
//
//         TRACE("FatOffset: %d\n", (FatOffset) as usize);
//         TRACE("ThisFatSecNum: %d\n", (ThisFatSecNum) as usize);
//         TRACE("ThisFatEntOffset: %d\n", (ThisFatEntOffset) as usize);
//
//         // The cluster pointer can span within two sectors, but the FatGetFatSector function
//         // reads 4 sectors most times, except when we are at the edge of FAT cache
//         // and/or FAT region on the disk. For FAT12 the whole FAT would be cached so
//         // there will be no situation when the first sector is at the end of the cache
//         // and the next one is in the beginning
//
//         ReadBuffer = FatGetFatSector(Volume, ThisFatSecNum);
//         if (!ReadBuffer)
//         {
//             return FALSE;
//         }
//
//         fat = *((USHORT *) (ReadBuffer + ThisFatEntOffset));
//         fat = SWAPW(fat);
//         if (Cluster & 0x0001)
//             fat = fat >> 4;    /* Cluster number is ODD */
//         else
//             fat = fat & 0x0FFF;    /* Cluster number is EVEN */
//
//         break;
//
//     case FAT16:
//     case FATX16:
//
//         FatOffset = (Cluster * 2);
//         ThisFatSecNum = FatOffset / Volume->BytesPerSector;
//         ThisFatEntOffset = (FatOffset % Volume->BytesPerSector);
//
//         ReadBuffer = FatGetFatSector(Volume, ThisFatSecNum);
//         if (!ReadBuffer)
//         {
//             return FALSE;
//         }
//
//         fat = *((USHORT *) (ReadBuffer + ThisFatEntOffset));
//         fat = SWAPW(fat);
//
//         break;
//
//     case FAT32:
//     case FATX32:
//
//         FatOffset = (Cluster * 4);
//         ThisFatSecNum = FatOffset / Volume->BytesPerSector;
//         ThisFatEntOffset = (FatOffset % Volume->BytesPerSector);
//
//         ReadBuffer = FatGetFatSector(Volume, ThisFatSecNum);
//         if (!ReadBuffer)
//         {
//             return FALSE;
//         }
//
//         // Get the fat entry
//         fat = (*((ULONG *) (ReadBuffer + ThisFatEntOffset))) & 0x0FFFFFFF;
//         fat = SWAPD(fat);
//
//         break;
//
//     default:
//         ERR("Unknown FAT type %d\n", (Volume->FatType) as usize);
//         return FALSE;
//     }
//
//     TRACE("FAT entry is 0x%x.\n", (fat) as usize);
//
//     *ClusterPointer = fat;
//
//     return TRUE;
// }
//
// static
// ULONG FatCountClustersInChain(PFAT_VOLUME_INFO Volume, UINT32 StartCluster)
// {
//     ULONG    ClusterCount = 0;
//
//     TRACE("FatCountClustersInChain() StartCluster = %d\n", (StartCluster) as usize);
//
//     while (1)
//     {
//         //
//         // If end of chain then break out of our cluster counting loop
//         //
//         if (FAT_IS_END_CLUSTER(StartCluster))
//         {
//             break;
//         }
//
//         //
//         // Increment count
//         //
//         ClusterCount++;
//
//         //
//         // Get next cluster
//         //
//         if (!FatGetFatEntry(Volume, StartCluster, &StartCluster))
//         {
//             return 0;
//         }
//     }
//
//     TRACE("FatCountClustersInChain() ClusterCount = %d\n", (ClusterCount) as usize);
//
//     return ClusterCount;
// }
//
// static
// BOOLEAN FatReadAdjacentClusters(
//     PFAT_VOLUME_INFO Volume,
//     UINT32 StartClusterNumber,
//     UINT32 MaxClusters,
//     PVOID Buffer,
//     PUINT32 ClustersRead,
//     PUINT32 LastClusterNumber)
// {
//     UINT32 NextClusterNumber;
//     UINT32 ClustersToRead = 1;
//     UINT32 PrevClusterNumber = StartClusterNumber;
//     UINT32 ClusterStartSector = ((PrevClusterNumber - 2) * Volume->SectorsPerCluster) + Volume->DataSectorStart;
//
//     *ClustersRead = 0;
//     *LastClusterNumber = 0;
//
//     if (!FatGetFatEntry(Volume, StartClusterNumber, &NextClusterNumber))
//     {
//         return FALSE;
//     }
//
//     // getting the number of adjacent clusters
//     while (!FAT_IS_END_CLUSTER(NextClusterNumber) && ClustersToRead < MaxClusters && (NextClusterNumber == PrevClusterNumber + 1))
//     {
//         ClustersToRead++;
//         PrevClusterNumber = NextClusterNumber;
//         if (!FatGetFatEntry(Volume, PrevClusterNumber, &NextClusterNumber))
//         {
//             return FALSE;
//         }
//     }
//
//     if (!FatReadVolumeSectors(Volume, ClusterStartSector, ClustersToRead * Volume->SectorsPerCluster, Buffer))
//     {
//         return FALSE;
//     }
//
//     *ClustersRead = ClustersToRead;
//     *LastClusterNumber = NextClusterNumber;
//
//     return !FAT_IS_END_CLUSTER(NextClusterNumber) && ClustersToRead < MaxClusters;
// }
//
// /*
//  * FatReadClusterChain()
//  * Reads the specified clusters into memory
//  */
// static
// BOOLEAN FatReadClusterChain(PFAT_VOLUME_INFO Volume, UINT32 StartClusterNumber, UINT32 NumberOfClusters, PVOID Buffer, PUINT32 LastClusterNumber)
// {
//     UINT32 ClustersRead, NextClusterNumber, ClustersLeft = NumberOfClusters;
//
//     TRACE("FatReadClusterChain() StartClusterNumber = %d NumberOfClusters = %d Buffer = 0x%x\n", (StartClusterNumber) as usize, (NumberOfClusters) as usize, (Buffer) as usize);
//
//     ASSERT(NumberOfClusters > 0);
//
//     while (FatReadAdjacentClusters(Volume, StartClusterNumber, ClustersLeft, Buffer, &ClustersRead, &NextClusterNumber))
//     {
//         ClustersLeft -= ClustersRead;
//         Buffer = (PVOID)((ULONG_PTR)Buffer + (ClustersRead * Volume->SectorsPerCluster * Volume->BytesPerSector));
//         StartClusterNumber = NextClusterNumber;
//     }
//
//     if (LastClusterNumber)
//     {
//         *LastClusterNumber = NextClusterNumber;
//     }
//
//     return (ClustersRead > 0);
// }
//
// /*
//  * FatReadPartialCluster()
//  * Reads part of a cluster into memory
//  */
// BOOLEAN FatReadPartialCluster(PFAT_VOLUME_INFO Volume, ULONG ClusterNumber, ULONG StartingOffset, ULONG Length, PVOID Buffer)
// {
//     ULONG        ClusterStartSector;
//     ULONG SectorOffset, ReadSize, SectorCount;
//     PUCHAR ReadBuffer;
//     BOOLEAN Success = FALSE;
//
//     //TRACE("FatReadPartialCluster() ClusterNumber = %d StartingOffset = %d Length = %d Buffer = 0x%x\n", (ClusterNumber) as usize, (StartingOffset) as usize, (Length) as usize, (Buffer) as usize);
//
//     ClusterStartSector = ((ClusterNumber - 2) * Volume->SectorsPerCluster) + Volume->DataSectorStart;
//
//     // This is the offset of the data in sectors
//     SectorOffset = (StartingOffset / Volume->BytesPerSector);
//     StartingOffset %= Volume->BytesPerSector;
//
//     // Calculate how many sectors we need to read
//     SectorCount = (StartingOffset + Length + Volume->BytesPerSector - 1) / Volume->BytesPerSector;
//
//     // Calculate rounded up read size
//     ReadSize = SectorCount * Volume->BytesPerSector;
//
//     ReadBuffer = FrLdrTempAlloc(ReadSize, TAG_FAT_BUFFER);
//     if (!ReadBuffer)
//     {
//         return FALSE;
//     }
//
//     if (FatReadVolumeSectors(Volume, ClusterStartSector + SectorOffset, SectorCount, ReadBuffer))
//     {
//         RtlCopyMemory(Buffer, ReadBuffer + StartingOffset, Length);
//         Success = TRUE;
//     }
//
//     FrLdrTempFree(ReadBuffer, TAG_FAT_BUFFER);
//
//     return Success;
// }
//
// /*
//  * FatReadFile()
//  * Reads BytesToRead from open file and
//  * returns the number of bytes read in BytesRead
//  */
// static
// BOOLEAN FatReadFile(PFAT_FILE_INFO FatFileInfo, ULONG BytesToRead, ULONG* BytesRead, PVOID Buffer)
// {
//     PFAT_VOLUME_INFO Volume = FatFileInfo->Volume;
//     UINT32 NextClusterNumber, BytesPerCluster;
//
//     TRACE("FatReadFile() BytesToRead=%u FileSize=%u FilePointer=%u CurrentCluster=0x%x\n", (//         BytesToRead) as usize, (FatFileInfo->FileSize) as usize, (FatFileInfo->FilePointer) as usize, (FatFileInfo->CurrentCluster) as usize);
//
//     if (BytesRead != NULL)
//     {
//         *BytesRead = 0;
//     }
//
//     //
//     // If the user is trying to read past the end of
//     // the file then return success with BytesRead == 0.
//     //
//     if (FatFileInfo->FilePointer >= FatFileInfo->FileSize)
//     {
//         return TRUE;
//     }
//
//     //
//     // If the user is trying to read more than there is to read
//     // then adjust the amount to read.
//     //
//     if ((FatFileInfo->FilePointer + BytesToRead) > FatFileInfo->FileSize)
//     {
//         BytesToRead = (FatFileInfo->FileSize - FatFileInfo->FilePointer);
//     }
//
//     //
//     // Ok, now we have to perform at most 3 calculations
//     // I'll draw you a picture (using nifty ASCII art):
//     //
//     // CurrentFilePointer -+
//     //                     |
//     //    +----------------+
//     //    |
//     // +-----------+-----------+-----------+-----------+
//     // | Cluster 1 | Cluster 2 | Cluster 3 | Cluster 4 |
//     // +-----------+-----------+-----------+-----------+
//     //    |                                    |
//     //    +---------------+--------------------+
//     //                    |
//     // BytesToRead -------+
//     //
//     // 1 - The first calculation (and read) will align
//     //     the file pointer with the next cluster.
//     //     boundary (if we are supposed to read that much)
//     // 2 - The next calculation (and read) will read
//     //     in all the full clusters that the requested
//     //     amount of data would cover (in this case
//     //     clusters 2 & 3).
//     // 3 - The last calculation (and read) would read
//     //     in the remainder of the data requested out of
//     //     the last cluster.
//     //
//
//     BytesPerCluster = Volume->SectorsPerCluster * Volume->BytesPerSector;
//
//     //
//     // Only do the first read if we
//     // aren't aligned on a cluster boundary
//     //
//     if (FatFileInfo->FilePointer % BytesPerCluster)
//     {
//         //
//         // Do the math for our first read
//         //
//         UINT32 OffsetInCluster = FatFileInfo->FilePointer % BytesPerCluster;
//         UINT32 LengthInCluster = min(BytesToRead, BytesPerCluster - OffsetInCluster);
//
//         ASSERT(LengthInCluster <= BytesPerCluster && LengthInCluster > 0);
//
//         //
//         // Now do the read and update BytesRead, BytesToRead, FilePointer, & Buffer
//         //
//         if (!FatReadPartialCluster(Volume, FatFileInfo->CurrentCluster, OffsetInCluster, LengthInCluster, Buffer))
//         {
//             return FALSE;
//         }
//         if (BytesRead != NULL)
//         {
//             *BytesRead += LengthInCluster;
//         }
//         BytesToRead -= LengthInCluster;
//         FatFileInfo->FilePointer += LengthInCluster;
//         Buffer = (PVOID)((ULONG_PTR)Buffer + LengthInCluster);
//
//         // get the next cluster if needed
//         if ((LengthInCluster + OffsetInCluster) == BytesPerCluster)
//         {
//             if (!FatGetFatEntry(Volume, FatFileInfo->CurrentCluster, &NextClusterNumber))
//             {
//                 return FALSE;
//             }
//
//             FatFileInfo->CurrentCluster = NextClusterNumber;
//             TRACE("FatReadFile() FatFileInfo->CurrentCluster = 0x%x\n", (FatFileInfo->CurrentCluster) as usize);
//         }
//     }
//
//     //
//     // Do the math for our second read (if any data left)
//     //
//     if (BytesToRead > 0)
//     {
//         //
//         // Determine how many full clusters we need to read
//         //
//         UINT32 NumberOfClusters = BytesToRead / BytesPerCluster;
//
//         TRACE("Going to read: %u clusters\n", (NumberOfClusters) as usize);
//
//         if (NumberOfClusters > 0)
//         {
//             UINT32 BytesReadHere = NumberOfClusters * BytesPerCluster;
//
//             ASSERT(!FAT_IS_END_CLUSTER(FatFileInfo->CurrentCluster));
//
//             if (!FatReadClusterChain(Volume, FatFileInfo->CurrentCluster, NumberOfClusters, Buffer, &NextClusterNumber))
//             {
//                 return FALSE;
//             }
//
//             if (BytesRead != NULL)
//             {
//                 *BytesRead += BytesReadHere;
//             }
//             BytesToRead -= BytesReadHere;
//             Buffer = (PVOID)((ULONG_PTR)Buffer + BytesReadHere);
//
//             ASSERT(!FAT_IS_END_CLUSTER(NextClusterNumber) || BytesToRead == 0);
//
//             FatFileInfo->FilePointer += BytesReadHere;
//             FatFileInfo->CurrentCluster = NextClusterNumber;
//             TRACE("FatReadFile() FatFileInfo->CurrentCluster = 0x%x\n", (FatFileInfo->CurrentCluster) as usize);
//         }
//     }
//
//     //
//     // Do the math for our third read (if any data left)
//     //
//     if (BytesToRead > 0)
//     {
//         ASSERT(!FAT_IS_END_CLUSTER(FatFileInfo->CurrentCluster));
//
//         //
//         // Now do the read and update BytesRead & FilePointer
//         //
//         if (!FatReadPartialCluster(Volume, FatFileInfo->CurrentCluster, 0, BytesToRead, Buffer))
//         {
//             return FALSE;
//         }
//         if (BytesRead != NULL)
//         {
//             *BytesRead += BytesToRead;
//         }
//         FatFileInfo->FilePointer += BytesToRead;
//     }
//
//     return TRUE;
// }
//
// BOOLEAN FatReadVolumeSectors(PFAT_VOLUME_INFO Volume, ULONG SectorNumber, ULONG SectorCount, PVOID Buffer)
// {
//     LARGE_INTEGER Position;
//     ULONG Count;
//     ARC_STATUS Status;
//
//     //TRACE("FatReadVolumeSectors(): SectorNumber %d, SectorCount %d, Buffer %p\n", (//     //    SectorNumber) as usize, (SectorCount) as usize, (Buffer) as usize);
//
//     //
//     // Seek to right position
//     //
//     Position.QuadPart = (ULONGLONG)SectorNumber * Volume->BytesPerSector;
//     Status = ArcSeek(Volume->DeviceId, &Position, SeekAbsolute);
//     if (Status != ESUCCESS)
//     {
//         TRACE("FatReadVolumeSectors() Failed to seek\n");
//         return FALSE;
//     }
//
//     //
//     // Read data
//     //
//     Status = ArcRead(Volume->DeviceId, Buffer, SectorCount * Volume->BytesPerSector, &Count);
//     if (Status != ESUCCESS || Count != SectorCount * Volume->BytesPerSector)
//     {
//         TRACE("FatReadVolumeSectors() Failed to read\n");
//         return FALSE;
//     }
//
//     // Return success
//     return TRUE;
// }
//
// ARC_STATUS FatClose(ULONG FileId)
// {
//     PFAT_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
//
//     FrLdrTempFree(FileHandle, TAG_FAT_FILE);
//
//     return ESUCCESS;
// }
//
// ARC_STATUS FatGetFileInformation(ULONG FileId, FILEINFORMATION* Information)
// {
//     PFAT_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
//
//     RtlZeroMemory(Information, sizeof(*Information));
//     Information->EndingAddress.LowPart = FileHandle->FileSize;
//     Information->CurrentAddress.LowPart = FileHandle->FilePointer;
//
//     /* Set the ARC file attributes */
//     Information->Attributes = FileHandle->Attributes;
//
//     /* Copy the file name, perhaps truncated, and NUL-terminated */
//     Information->FileNameLength = min(FileHandle->FileNameLength, sizeof(Information->FileName) - 1);
//     RtlCopyMemory(Information->FileName, FileHandle->FileName, Information->FileNameLength);
//     Information->FileName[Information->FileNameLength] = ANSI_NULL;
//
//     TRACE("FatGetFileInformation(%lu) -> FileSize = %lu, FilePointer = 0x%lx\n", (//           FileId) as usize, (Information->EndingAddress.LowPart) as usize, (Information->CurrentAddress.LowPart) as usize);
//
//     return ESUCCESS;
// }
//
// ARC_STATUS FatOpen(CHAR* Path, OPENMODE OpenMode, ULONG* FileId)
// {
//     PFAT_VOLUME_INFO FatVolume;
//     FAT_FILE_INFO TempFileInfo;
//     PFAT_FILE_INFO FileHandle;
//     ULONG DeviceId;
//     BOOLEAN IsDirectory;
//     ARC_STATUS Status;
//
//     if (OpenMode != OpenReadOnly && OpenMode != OpenDirectory)
//         return EACCES;
//
//     DeviceId = FsGetDeviceId(*FileId);
//     FatVolume = FatVolumes[DeviceId];
//
//     TRACE("FatOpen() FileName = %s (DeviceId=%u)\n", (Path) as usize, (DeviceId) as usize);
//
//     RtlZeroMemory(&TempFileInfo, sizeof(TempFileInfo));
//     Status = FatLookupFile(FatVolume, Path, &TempFileInfo);
//     if (Status != ESUCCESS)
//         return ENOENT;
//
//     //
//     // Check if caller opened what he expected (dir vs file)
//     //
//     IsDirectory = !!(TempFileInfo.Attributes & DirectoryFile);
//     if (IsDirectory && OpenMode != OpenDirectory)
//         return EISDIR;
//     else if (!IsDirectory && OpenMode != OpenReadOnly)
//         return ENOTDIR;
//
//     FileHandle = FrLdrTempAlloc(sizeof(FAT_FILE_INFO), TAG_FAT_FILE);
//     if (!FileHandle)
//         return ENOMEM;
//
//     RtlCopyMemory(FileHandle, &TempFileInfo, sizeof(FAT_FILE_INFO));
//     FileHandle->Volume = FatVolume;
//
//     FsSetDeviceSpecific(*FileId, FileHandle);
//     return ESUCCESS;
// }
//
// ARC_STATUS FatRead(ULONG FileId, VOID* Buffer, ULONG N, ULONG* Count)
// {
//     PFAT_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
//     BOOLEAN Success;
//
//     /* Call old read method */
//     Success = FatReadFile(FileHandle, N, Count, Buffer);
//     if (Success)
//         return ESUCCESS;
//     else
//         return EIO;
// }
//
// ARC_STATUS FatSeek(ULONG FileId, LARGE_INTEGER* Position, SEEKMODE SeekMode)
// {
//     PFAT_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
//     PFAT_VOLUME_INFO Volume = FileHandle->Volume;
//     LARGE_INTEGER NewPosition = *Position;
//
//     switch (SeekMode)
//     {
//         case SeekAbsolute:
//             break;
//         case SeekRelative:
//             NewPosition.QuadPart += (ULONGLONG)FileHandle->FilePointer;
//             break;
//         default:
//             ASSERT(FALSE);
//             return EINVAL;
//     }
//
//     if (NewPosition.HighPart != 0)
//         return EINVAL;
//     if (NewPosition.LowPart >= FileHandle->FileSize)
//         return EINVAL;
//
//     TRACE("FatSeek() NewPosition = %u, OldPointer = %u, SeekMode = %d\n", (NewPosition.LowPart) as usize, (FileHandle->FilePointer) as usize, (SeekMode) as usize);
//
//     {
//         UINT32 OldClusterIdx = FileHandle->FilePointer / (Volume->SectorsPerCluster * Volume->BytesPerSector);
//         UINT32 NewClusterIdx = NewPosition.LowPart / (Volume->SectorsPerCluster * Volume->BytesPerSector);
//
//         TRACE("FatSeek() OldClusterIdx: %u, NewClusterIdx: %u\n", (OldClusterIdx) as usize, (NewClusterIdx) as usize);
//
//         if (NewClusterIdx != OldClusterIdx)
//         {
//             UINT32 CurrentClusterIdx, ClusterNumber;
//
//             if (NewClusterIdx > OldClusterIdx)
//             {
//                 CurrentClusterIdx = OldClusterIdx;
//                 ClusterNumber = FileHandle->CurrentCluster;
//             }
//             else
//             {
//                 CurrentClusterIdx = 0;
//                 ClusterNumber = FileHandle->StartCluster;
//             }
//
//             for (; CurrentClusterIdx < NewClusterIdx; CurrentClusterIdx++)
//             {
//                 if (!FatGetFatEntry(Volume, ClusterNumber, &ClusterNumber))
//                 {
//                     return EIO;
//                 }
//             }
//             FileHandle->CurrentCluster = ClusterNumber;
//         }
//     }
//
//     FileHandle->FilePointer = NewPosition.LowPart;
//
//     return ESUCCESS;
// }
//
//
// /**
//  * @brief
//  * Returns the size of the FAT volume laid on the storage media device
//  * opened via @p DeviceId.
//  **/
// ULONGLONG
// FatGetVolumeSize(
//     _In_ ULONG DeviceId)
// {
//     PFAT_VOLUME_INFO Volume = FatVolumes[DeviceId];
//     ASSERT(Volume);
//     return (ULONGLONG)Volume->TotalSectors * Volume->BytesPerSector;
// }
//
//
// const DEVVTBL FatFuncTable =
// {
//     FatClose,
//     FatGetFileInformation,
//     FatOpen,
//     FatRead,
//     FatSeek,
//     crate::wide!("fastfat"),
// };
//
// const DEVVTBL FatXFuncTable =
// {
//     FatClose,
//     FatGetFileInformation,
//     FatOpen,
//     FatRead,
//     FatSeek,
//     crate::wide!("vfatfs"),
// };
//
// const DEVVTBL* FatMount(ULONG DeviceId)
// {
//     PFAT_VOLUME_INFO Volume;
//     UCHAR Buffer[SECTOR_SIZE];
//     PFAT_BOOTSECTOR BootSector = (PFAT_BOOTSECTOR)Buffer;
//     PFAT32_BOOTSECTOR BootSector32 = (PFAT32_BOOTSECTOR)Buffer;
//     PFATX_BOOTSECTOR BootSectorX = (PFATX_BOOTSECTOR)Buffer;
//     FILEINFORMATION FileInformation;
//     LARGE_INTEGER Position;
//     ULONG Count;
//     ULARGE_INTEGER SectorCount;
//     ARC_STATUS Status;
//
//     TRACE("Enter FatMount(%lu)\n", (DeviceId) as usize);
//
//     //
//     // Allocate data for volume information
//     //
//     Volume = FrLdrTempAlloc(sizeof(FAT_VOLUME_INFO), TAG_FAT_VOLUME);
//     if (!Volume)
//         return NULL;
//     RtlZeroMemory(Volume, sizeof(FAT_VOLUME_INFO));
//
//     //
//     // Read the BootSector
//     //
//     Position.QuadPart = 0;
//     Status = ArcSeek(DeviceId, &Position, SeekAbsolute);
//     if (Status != ESUCCESS)
//     {
//         FrLdrTempFree(Volume, TAG_FAT_VOLUME);
//         return NULL;
//     }
//     Status = ArcRead(DeviceId, Buffer, sizeof(Buffer), &Count);
//     if (Status != ESUCCESS || Count != sizeof(Buffer))
//     {
//         FrLdrTempFree(Volume, TAG_FAT_VOLUME);
//         return NULL;
//     }
//
//     //
//     // Check if BootSector is valid. If no, return early
//     //
//     if (!RtlEqualMemory(BootSector->FileSystemType, "FAT12   ", 8) &&
//         !RtlEqualMemory(BootSector->FileSystemType, "FAT16   ", 8) &&
//         !RtlEqualMemory(BootSector32->FileSystemType, "FAT32   ", 8) &&
//         !RtlEqualMemory(BootSectorX->FileSystemType, "FATX", 4))
//     {
//         FrLdrTempFree(Volume, TAG_FAT_VOLUME);
//         return NULL;
//     }
//
//     //
//     // Determine sector count
//     //
//     Status = ArcGetFileInformation(DeviceId, &FileInformation);
//     if (Status != ESUCCESS)
//     {
//         FrLdrTempFree(Volume, TAG_FAT_VOLUME);
//         return NULL;
//     }
//     SectorCount.QuadPart = (FileInformation.EndingAddress.QuadPart - FileInformation.StartingAddress.QuadPart);
//     SectorCount.QuadPart /= SECTOR_SIZE;
//
//     //
//     // Keep device id
//     //
//     Volume->DeviceId = DeviceId;
//
//     //
//     // Really open the volume
//     //
//     if (!FatOpenVolume(Volume, BootSector, SectorCount.QuadPart))
//     {
//         FrLdrTempFree(Volume, TAG_FAT_VOLUME);
//         return NULL;
//     }
//
//     //
//     // Remember FAT volume information
//     //
//     FatVolumes[DeviceId] = Volume;
//
//     //
//     // Return success
//     //
//     TRACE("FatMount(%lu) success\n", (DeviceId) as usize);
//     return (ISFATX(Volume->FatType) ? &FatXFuncTable : &FatFuncTable);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

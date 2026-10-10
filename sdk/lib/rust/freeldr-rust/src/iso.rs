#[allow(unused_imports)] use crate::{ERR, TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_FILESYSTEM; // iso.c: DBG_DEFAULT_CHANNEL(FILESYSTEM)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER, PCSTR};
// ADDED 2026-07-05: patterns below were unresolved names -> silent bind-all
// catch-alls (bind-all match-arm bug class); import the real ddk consts.
use ntkrnl_sys::ddk::{_SEEKMODE_SeekAbsolute as SEEKMODE_SeekAbsolute, _SEEKMODE_SeekRelative as SEEKMODE_SeekRelative};
use core::ffi::c_void;

// ---- restored from iso.h / iso.c (rustconv drops non-fn header content) ----
pub const SECTORSIZE: ULONG = 2048;
pub const TAG_ISO_VOLUME: u32 = u32::from_be_bytes(*b"VosI");
pub const TAG_ISO_BUFFER: u32 = u32::from_be_bytes(*b"BosI");
pub const TAG_ISO_FILE: u32 = u32::from_be_bytes(*b"FosI");

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct _DIR_RECORD {
    pub RecordLength: UCHAR,
    pub ExtAttrRecordLength: UCHAR,
    pub ExtentLocationL: ULONG,
    pub ExtentLocationM: ULONG,
    pub DataLengthL: ULONG,
    pub DataLengthM: ULONG,
    pub Year: UCHAR,
    pub Month: UCHAR,
    pub Day: UCHAR,
    pub Hour: UCHAR,
    pub Minute: UCHAR,
    pub Second: UCHAR,
    pub TimeZone: UCHAR,
    pub FileFlags: UCHAR,
    pub FileUnitSize: UCHAR,
    pub InterleaveGapSize: UCHAR,
    pub VolumeSequenceNumber: ULONG,
    pub FileIdLength: UCHAR,
    pub FileId: [UCHAR; 1],
}
pub type DIR_RECORD = _DIR_RECORD;
pub type PDIR_RECORD = *mut _DIR_RECORD;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct _VD_HEADER {
    pub VdType: UCHAR,
    pub StandardId: [UCHAR; 5],
    pub VdVersion: UCHAR,
}
pub type VD_HEADER = _VD_HEADER;
pub type PVD_HEADER = *mut _VD_HEADER;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct _PVD {
    pub VdType: UCHAR,
    pub StandardId: [UCHAR; 5],
    pub VdVersion: UCHAR,
    pub unused0: UCHAR,
    pub SystemId: [UCHAR; 32],
    pub VolumeId: [UCHAR; 32],
    pub unused1: [UCHAR; 8],
    pub VolumeSpaceSizeL: ULONG,
    pub VolumeSpaceSizeM: ULONG,
    pub unused2: [UCHAR; 32],
    pub VolumeSetSizeL: USHORT,
    pub VolumeSetSizeM: USHORT,
    pub VolumeSeqNumberL: USHORT,
    pub VolumeSeqNumberM: USHORT,
    pub LogicalBlockSizeL: USHORT,
    pub LogicalBlockSizeM: USHORT,
    pub PathTableSizeL: ULONG,
    pub PathTableSizeM: ULONG,
    pub LPathTablePos: ULONG,
    pub LOptPathTablePos: ULONG,
    pub MPathTablePos: ULONG,
    pub MOptPathTablePos: ULONG,
    pub RootDirRecord: _DIR_RECORD,
    pub VolumeSetIdentifier: [UCHAR; 128],
    pub PublisherIdentifier: [UCHAR; 128],
}
pub type PVD = _PVD;
pub type PPVD = *mut _PVD;

#[repr(C)]
pub struct _ISO_VOLUME_INFO {
    pub VolumeSizeLBN: ULONG,
    pub LogicalBlockSize: ULONG,
    pub PvdDirectorySector: ULONG,
    pub PvdDirectoryLength: ULONG,
    pub DirectoryPathLength: ULONG,
    pub DirectoryLength: ULONG,
    pub DirectoryBuffer: PVOID,
    pub DirectoryPath: [UCHAR; 261],
}
pub type ISO_VOLUME_INFO = _ISO_VOLUME_INFO;
pub type PISO_VOLUME_INFO = *mut _ISO_VOLUME_INFO;

#[repr(C)]
pub struct _ISO_FILE_INFO {
    pub FileStart: ULONG,
    pub FileSize: ULONG,
    pub FilePointer: ULONG,
    pub DriveNumber: ULONG,
    pub FileNameLength: ULONG,
    pub Attributes: UCHAR,
    pub FileName: [CHAR; 32],
}
pub type ISO_FILE_INFO = _ISO_FILE_INFO;
pub type PISO_FILE_INFO = *mut _ISO_FILE_INFO;

#[no_mangle]
pub static mut IsoVolumes: [PISO_VOLUME_INFO; MAX_FDS] = [core::ptr::null_mut(); MAX_FDS];
pub const ISO_ATTR_HIDDEN: UCHAR = 0x01;
pub const ISO_ATTR_DIRECTORY: UCHAR = 0x02;
pub const SECTOR_SIZE: ULONG = SECTORSIZE;

extern "C" {
    fn ArcSeek(DeviceId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: ULONG) -> ARC_STATUS;
    fn ArcRead(DeviceId: ULONG, Buffer: PVOID, N: ULONG, Count: *mut ULONG) -> ARC_STATUS;
    fn ArcOpen(Path: PCHAR, OpenMode: OPENMODE, FileId: *mut ULONG) -> ARC_STATUS;
    fn ArcClose(FileId: ULONG) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: ULONG, Info: *mut FILEINFORMATION) -> ARC_STATUS;
    fn FsGetDeviceId(FileId: ULONG) -> ULONG;
    fn FsSetDeviceSpecific(FileId: ULONG, Specific: PVOID);
    fn _stricmp(a: *const i8, b: *const i8) -> i32;
}

// iso.c: static const DEVVTBL Iso9660FuncTable (dropped by translator).
static Iso9660FuncTable: DEVVTBL = DEVVTBL {
    Close: Some(IsoClose),
    GetFileInformation: Some(IsoGetFileInformation),
    Open: Some(IsoOpen),
    Read: Some(IsoRead),
    Seek: Some(IsoSeek),
    ServiceName: crate::wide!("cdfs\0"),
};
// ---- end restored ----
extern "C" {
}

extern "C" {
    fn FsGetDeviceSpecific(FileId: u32) -> PISO_FILE_INFO;
    fn FsGetFirstNameFromPath(Path: PCHAR, FileName: PCSTR);
    fn FsGetNumPathParts(Path: PCSTR) -> ULONG;
    fn RtlEqualMemory(Source1: PVOID, Source2: PVOID, Length: usize) -> BOOLEAN;
    fn strlen(s: PCSTR) -> ULONG;
}

unsafe fn IsoLastPathSeparator(Path: PCSTR) -> PCSTR {
    let mut Last: PCSTR = core::ptr::null();
    let mut Path = Path;

    while *Path != 0 {
        if *Path == ((b'\\') as i8) || *Path == ((b'/') as i8) {
            Last = Path;
        }
        Path = Path.offset(1);
    }

    Last
}

unsafe fn IsoSearchDirectoryBufferForFile(
    DirectoryBuffer: PVOID,
    DirectoryLength: ULONG,
    FileName: PCHAR,
    IsoFileInfoPointer: PISO_FILE_INFO,
) -> BOOLEAN {
    let mut Offset: ULONG;
    let mut Record: PDIR_RECORD;
    let mut i: ULONG;
    let mut Name: [CHAR; 32] = [0; 32];

    TRACE!("IsoSearchDirectoryBufferForFile() DirectoryBuffer = 0x%x DirectoryLength = %d FileName = %s\n", (DirectoryBuffer) as usize, (DirectoryLength) as usize, (FileName) as usize);

    Offset = 0;
    Record = DirectoryBuffer as PDIR_RECORD;
    loop {
        Offset = Offset + (*Record).RecordLength as ULONG;
        Record = (DirectoryBuffer as ULONG_PTR + Offset as ULONG_PTR) as PDIR_RECORD;

        if (*Record).RecordLength == 0 {
            Offset = (ROUND_UP(Offset, SECTORSIZE)) as u32 ;
            Record = (DirectoryBuffer as ULONG_PTR + Offset as ULONG_PTR) as PDIR_RECORD;
        }

        if Offset >= DirectoryLength {
            return 0u8;
        }

        // `FileId: [UCHAR; 1]` is a C flexible-array sentinel; the real data
        // extends past the struct. Index via raw pointer to avoid Rust's
        // bounds panic.
        let file_id_ptr = (*Record).FileId.as_ptr();
        let id0 = if (*Record).FileIdLength >= 1 { *file_id_ptr } else { 0 };
        if (*Record).FileIdLength == 1 && id0 == 0 {
            TRACE!("Name '.'\n");
        } else if (*Record).FileIdLength == 1 && id0 == 1 {
            TRACE!("Name '..'\n");
        } else {
            i = 0;
            while i < (*Record).FileIdLength as ULONG {
                let c = *file_id_ptr.add(i as usize);
                if c == b';' { break; }
                Name[i as usize] = c as i8;
                i += 1;
            }
            Name[i as usize] = 0;
            TRACE!("Name '%s'\n", (Name.as_ptr()) as usize);

            if _stricmp(FileName, Name.as_ptr()) == 0 {
                (*IsoFileInfoPointer).FileStart = (*Record).ExtentLocationL;
                (*IsoFileInfoPointer).FileSize = (*Record).DataLengthL;
                (*IsoFileInfoPointer).FilePointer = 0;
                (*IsoFileInfoPointer).Attributes = (*Record).FileFlags;
                return 1u8;
            }
        }
    }
}

unsafe fn IsoBufferDirectory(
    DeviceId: ULONG,
    DirectoryStartSector: ULONG,
    DirectoryLength: ULONG,
    pDirectoryBuffer: *mut PVOID,
) -> ARC_STATUS {
    let DirectoryBuffer: PVOID;
    let SectorCount: ULONG;
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let mut Count: ULONG = 0;
    let mut Status: ARC_STATUS = ESUCCESS;

    TRACE!("IsoBufferDirectory() DirectoryStartSector = %d DirectoryLength = %d\n", (DirectoryStartSector) as usize, (DirectoryLength) as usize);

    SectorCount = (ROUND_UP(DirectoryLength, SECTORSIZE) / SECTORSIZE) as u32 ;
    TRACE!("Trying to read (DirectoryCount) %d sectors.\n", (SectorCount) as usize);

    TRACE!("Trying to allocate (DirectoryLength) %d bytes.\n", (DirectoryLength) as usize);
    DirectoryBuffer = FrLdrTempAlloc((DirectoryLength) as usize , TAG_ISO_BUFFER);
    if DirectoryBuffer.is_null() {
        return ENOMEM;
    }

    Position.u.HighPart = 0;
    Position.u.LowPart = DirectoryStartSector * SECTORSIZE;
    Status = ArcSeek(DeviceId, &mut Position, SeekAbsolute);
    if Status != ESUCCESS {
        FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);
        return Status;
    }
    Status = ArcRead(DeviceId, DirectoryBuffer, SectorCount * SECTORSIZE, &mut Count);
    if Status != ESUCCESS || Count != SectorCount * SECTORSIZE {
        FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);
        return EIO;
    }

    *pDirectoryBuffer = DirectoryBuffer;
    ESUCCESS
}

unsafe fn IsoLookupFile(mut FileName: PCSTR, DeviceId: ULONG, IsoFileInfo: PISO_FILE_INFO) -> ARC_STATUS {
    let FullPath: PCSTR = FileName;
    let Volume: PISO_VOLUME_INFO;
    let FileNameStr: PCSTR;
    let mut i: ULONG;
    let DirectoryPathLength: ULONG;
    let mut DirectorySector: ULONG = 0;
    let mut DirectoryLength: ULONG;
    let mut DirectoryBuffer: PVOID = core::ptr::null_mut();
    let NumberOfPathParts: ULONG;
    let mut PathBuffer: [CHAR; 261] = [0; 261];
    let PathPart: PCHAR;
    let mut Status: ARC_STATUS = ESUCCESS;
    let DoFullLookup: BOOLEAN;
    let FileAttributes: UCHAR;

    TRACE!("IsoLookupFile() FileName = %s\n", (FileName) as usize);

    Volume = IsoVolumes[DeviceId as usize];

    FileNameStr = IsoLastPathSeparator(FileName).offset(1);

    DirectoryPathLength = FileNameStr.offset_from(FileName) as ULONG;

    DoFullLookup = if (DirectoryPathLength != (*Volume).DirectoryPathLength) || 
                      RtlEqualMemory(FileName as PVOID, (*Volume).DirectoryPath.as_ptr() as PVOID, DirectoryPathLength as usize) == 0 {
        1u8
    } else {
        0u8
    };

    if DoFullLookup == 0 {
        PathPart = FileNameStr as PCHAR;
        DirectoryBuffer = (*Volume).DirectoryBuffer;
        DirectoryLength = (*Volume).DirectoryLength;

        NumberOfPathParts = 1;
    } else {
        PathPart = PathBuffer.as_mut_ptr();
        DirectorySector = (*Volume).PvdDirectorySector;
        DirectoryLength = (*Volume).PvdDirectoryLength;

        if *FileName == ((b'\\') as i8) || *FileName == ((b'/') as i8) {
            FileName = FileName.offset(1);
        }
        *PathPart = 0;

        NumberOfPathParts = FsGetNumPathParts(FileName);
    }

    i = 0;
    while i < NumberOfPathParts {
        if DoFullLookup != 0 {
            FsGetFirstNameFromPath(PathPart, FileName);

            while *FileName != 0 && *FileName != ((b'\\') as i8) && *FileName != ((b'/') as i8) {
                FileName = FileName.offset(1);
            }
            FileName = FileName.offset(1);

            Status = IsoBufferDirectory(DeviceId,
                                       DirectorySector,
                                       DirectoryLength,
                                       core::ptr::addr_of_mut!(DirectoryBuffer));
            if Status != ESUCCESS {
                return Status;
            }

            if (i + 1) >= NumberOfPathParts {
                if !(*Volume).DirectoryBuffer.is_null() {
                    FrLdrTempFree((*Volume).DirectoryBuffer, TAG_ISO_BUFFER);
                }

                (*Volume).DirectoryPathLength = DirectoryPathLength;
                (*Volume).DirectoryBuffer = DirectoryBuffer;
                (*Volume).DirectoryLength = DirectoryLength;

                RtlCopyMemory((*Volume).DirectoryPath.as_mut_ptr() as PVOID, FullPath as PVOID, (DirectoryPathLength) as usize );
            }
        }

        if IsoSearchDirectoryBufferForFile(DirectoryBuffer,
                                          DirectoryLength,
                                          PathPart,
                                          IsoFileInfo) == 0 {
            if (i + 1) < NumberOfPathParts {
                FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);
            }
            return ENOENT;
        }

        if (i + 1) < NumberOfPathParts {
            FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);

            DirectorySector = (*IsoFileInfo).FileStart;
            DirectoryLength = (*IsoFileInfo).FileSize;
        }

        i += 1;
    }

    FileAttributes = (*IsoFileInfo).Attributes;
    (*IsoFileInfo).Attributes = (ReadOnlyFile) as u8 ;
    if (FileAttributes & ISO_ATTR_HIDDEN) != 0 {
        (*IsoFileInfo).Attributes |= (HiddenFile) as u8 ;
    }
    if (FileAttributes & ISO_ATTR_DIRECTORY) != 0 {
        (*IsoFileInfo).Attributes |= (DirectoryFile) as u8 ;
    }

    (*IsoFileInfo).FileNameLength = strlen(PathPart);
    (*IsoFileInfo).FileNameLength = min((*IsoFileInfo).FileNameLength, (core::mem::size_of_val(&(*IsoFileInfo).FileName) - 1) as ULONG);
    RtlCopyMemory((*IsoFileInfo).FileName.as_mut_ptr() as PVOID, PathPart as PVOID, ((*IsoFileInfo).FileNameLength) as usize );

    ESUCCESS
}

unsafe extern "C" fn IsoClose(FileId: u32) -> ARC_STATUS {
    
    let FileHandle = FsGetDeviceSpecific(FileId);
    // iso.c:323 frees with TAG_ISO_FILE, and iso.c:364 allocates with it. The
    // literal here was 0x49736F46 -- 'IsoF', TAG_ISO_FILE ('FosI', 0x466F7349)
    // with its bytes reversed -- so EVERY file close on the CD tripped
    // FrLdrHeapFreeEx's tag check. One assertion per file loaded.
    FrLdrTempFree(FileHandle as *mut core::ffi::c_void, TAG_ISO_FILE);
    0
}

unsafe extern "C" fn IsoGetFileInformation(FileId: u32, Information: *mut FILEINFORMATION) -> ARC_STATUS {

    let FileHandle = FsGetDeviceSpecific(FileId);

    RtlZeroMemory(Information as *mut core::ffi::c_void, core::mem::size_of::<FILEINFORMATION>());
    (*Information).EndingAddress.u.LowPart = (*FileHandle).FileSize;
    (*Information).CurrentAddress.u.LowPart = (*FileHandle).FilePointer;

    (*Information).Attributes = (*FileHandle).Attributes;

    (*Information).FileNameLength = min((*FileHandle).FileNameLength, (core::mem::size_of_val(&(*Information).FileName) - 1) as u32);
    RtlCopyMemory(
        (*Information).FileName.as_mut_ptr() as *mut core::ffi::c_void,
        (*FileHandle).FileName.as_ptr() as *const core::ffi::c_void,
        (*Information).FileNameLength as usize
    );
    (*Information).FileName[(*Information).FileNameLength as usize] = 0;

    TRACE!("IsoGetFileInformation(%lu) -> FileSize = %lu, FilePointer = 0x%lx\n", (FileId) as usize, ((*Information).EndingAddress.u.LowPart) as usize, ((*Information).CurrentAddress.u.LowPart) as usize);

    0
}

unsafe extern "C" fn IsoOpen(Path: PCHAR, OpenMode: OPENMODE, FileId: PULONG) -> ARC_STATUS {
    let FileHandle: PISO_FILE_INFO;
    let DeviceId: ULONG;
    let mut Status: ARC_STATUS = ESUCCESS;

    if OpenMode != OpenReadOnly {
        return EACCES;
    }

    DeviceId = FsGetDeviceId(*FileId);

    TRACE!("IsoOpen() FileName = %s\n", (Path) as usize);

    FileHandle = FrLdrTempAlloc(core::mem::size_of::<ISO_FILE_INFO>(), TAG_ISO_FILE) as PISO_FILE_INFO;
    if FileHandle.is_null() {
        return ENOMEM;
    }

    Status = IsoLookupFile(Path, DeviceId, FileHandle);
    if Status != ESUCCESS {
        FrLdrTempFree(FileHandle as PVOID, TAG_ISO_FILE);
        return ENOENT;
    }

    FsSetDeviceSpecific(*FileId, FileHandle as PVOID);
    return ESUCCESS;
}

unsafe extern "C" fn IsoRead(FileId: u32, Buffer: *mut core::ffi::c_void, N: u32, Count: *mut u32) -> ARC_STATUS {
    let FileHandle: *mut ISO_FILE_INFO = FsGetDeviceSpecific(FileId);
    let mut SectorBuffer: [u8; SECTORSIZE as usize] = core::mem::zeroed();
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let DeviceId: u32;
    let mut SectorNumber: u32;
    let OffsetInSector: u32;
    let LengthInSector: u32;
    let NumberOfSectors: u32;
    let mut BytesRead: u32 = 0;
    let mut N = N;
    let mut Buffer = Buffer;
    let mut Status: ARC_STATUS = ESUCCESS;

    TRACE!("IsoRead() Buffer = %p, N = %lu\n", (Buffer) as usize, (N) as usize);

    DeviceId = FsGetDeviceId(FileId);
    *Count = 0;

    if (*FileHandle).FilePointer >= (*FileHandle).FileSize {
        return ESUCCESS;
    }

    if (*FileHandle).FilePointer + N > (*FileHandle).FileSize {
        N = (*FileHandle).FileSize - (*FileHandle).FilePointer;
    }

    if (*FileHandle).FilePointer % SECTORSIZE != 0 {
        SectorNumber = (*FileHandle).FileStart + ((*FileHandle).FilePointer / SECTORSIZE);
        OffsetInSector = (*FileHandle).FilePointer % SECTORSIZE;
        LengthInSector = min(N, SECTORSIZE - OffsetInSector);

        Position.u.HighPart = 0;
        Position.u.LowPart = SectorNumber * SECTORSIZE;
        Status = ArcSeek(DeviceId, &mut Position, SeekAbsolute);
        if Status != ESUCCESS {
            return Status;
        }
        Status = ArcRead(DeviceId, SectorBuffer.as_mut_ptr() as *mut core::ffi::c_void, SECTORSIZE, &mut BytesRead);
        if Status != ESUCCESS || BytesRead != SECTORSIZE {
            return EIO;
        }
        RtlCopyMemory(Buffer, SectorBuffer.as_ptr().add(OffsetInSector as usize) as *const core::ffi::c_void, LengthInSector as usize);
        *Count += LengthInSector;
        N -= LengthInSector;
        (*FileHandle).FilePointer += LengthInSector;
        Buffer = (Buffer as usize + LengthInSector as usize) as *mut core::ffi::c_void;
    }

    if N > 0 {
        NumberOfSectors = N / SECTORSIZE;

        SectorNumber = (*FileHandle).FileStart + ((*FileHandle).FilePointer / SECTORSIZE);

        Position.u.HighPart = 0;
        Position.u.LowPart = SectorNumber * SECTORSIZE;
        Status = ArcSeek(DeviceId, &mut Position, SeekAbsolute);
        if Status != ESUCCESS {
            return Status;
        }
        Status = ArcRead(DeviceId, Buffer, NumberOfSectors * SECTORSIZE, &mut BytesRead);
        if Status != ESUCCESS || BytesRead != NumberOfSectors * SECTORSIZE {
            return EIO;
        }

        *Count += NumberOfSectors * SECTORSIZE;
        N -= NumberOfSectors * SECTORSIZE;
        (*FileHandle).FilePointer += NumberOfSectors * SECTORSIZE;
        Buffer = (Buffer as usize + (NumberOfSectors * SECTORSIZE) as usize) as *mut core::ffi::c_void;
    }

    if N > 0 {
        SectorNumber = (*FileHandle).FileStart + ((*FileHandle).FilePointer / SECTORSIZE);

        Position.u.HighPart = 0;
        Position.u.LowPart = SectorNumber * SECTORSIZE;
        Status = ArcSeek(DeviceId, &mut Position, SeekAbsolute);
        if Status != ESUCCESS {
            return Status;
        }
        Status = ArcRead(DeviceId, SectorBuffer.as_mut_ptr() as *mut core::ffi::c_void, SECTORSIZE, &mut BytesRead);
        if Status != ESUCCESS || BytesRead != SECTORSIZE {
            return EIO;
        }
        RtlCopyMemory(Buffer, SectorBuffer.as_ptr() as *const core::ffi::c_void, N as usize);
        *Count += N;
        (*FileHandle).FilePointer += N;
    }

    TRACE!("IsoRead() done\n");

    return ESUCCESS;
}

unsafe extern "C" fn IsoSeek(FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: SEEKMODE) -> ARC_STATUS {
    
    let FileHandle = FsGetDeviceSpecific(FileId);
    let mut NewPosition = *Position;

    match SeekMode {
        SEEKMODE_SeekAbsolute => {},
        SEEKMODE_SeekRelative => {
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

    (*FileHandle).FilePointer = NewPosition.u.LowPart;
    ESUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IsoGetVolumeSize(
    DeviceId: ULONG
) -> ULONGLONG
{
    let Volume: PISO_VOLUME_INFO = IsoVolumes[DeviceId as usize];
    let VolumeSizeLBN = (*Volume).VolumeSizeLBN;
    let LogicalBlockSize = (*Volume).LogicalBlockSize;
    (VolumeSizeLBN as ULONGLONG) * (LogicalBlockSize as ULONGLONG)
}

#[no_mangle]
pub unsafe extern "C" fn IsoMount(DeviceId: u32) -> *const DEVVTBL {
    let mut Buffer: [u8; SECTORSIZE as usize] = [0; SECTORSIZE as usize];
    let Pvd: PPVD = Buffer.as_mut_ptr() as PPVD;
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let mut Count: u32 = 0;
    let mut Status: ARC_STATUS = ESUCCESS;
    let Volume: PISO_VOLUME_INFO;

    TRACE!("Enter IsoMount(%lu)\n", (DeviceId) as usize);

    Position.QuadPart = (16 * SECTORSIZE) as i64;
    Status = ArcSeek(DeviceId, &mut Position, SeekAbsolute);
    if Status != ESUCCESS {
        return core::ptr::null();
    }
    Status = ArcRead(DeviceId, Pvd as *mut core::ffi::c_void, SECTORSIZE, &mut Count);
    if Status != ESUCCESS || Count < core::mem::size_of::<PVD>() as u32 {
        return core::ptr::null();
    }

    if !((*Pvd).VdType == 1 && RtlEqualMemory((*Pvd).StandardId.as_ptr() as *mut core::ffi::c_void, b"CD001\0".as_ptr() as *mut core::ffi::c_void, 5usize) != 0 && (*Pvd).VdVersion == 1) {
        WARN!("Unrecognized CD-ROM format\n");
        return core::ptr::null();
    }
    if (*Pvd).LogicalBlockSizeL != SECTORSIZE as u16 {
        ERR!("Unsupported LogicalBlockSize %u\n", ((*Pvd).LogicalBlockSizeL) as usize);
        return core::ptr::null();
    }

    Count = ((*Pvd).VolumeSpaceSizeL as u64 * SECTORSIZE as u64 / 1024 / 1024) as u32;
    TRACE!("Recognized ISO9660 drive, size %lu MB (%lu sectors)\n", (Count) as usize, ((*Pvd).VolumeSpaceSizeL) as usize);

    Volume = FrLdrTempAlloc(core::mem::size_of::<ISO_VOLUME_INFO>(), TAG_ISO_VOLUME) as PISO_VOLUME_INFO;
    if Volume.is_null() {
        return core::ptr::null();
    }
    RtlZeroMemory(Volume as *mut core::ffi::c_void, core::mem::size_of::<ISO_VOLUME_INFO>());

    (*Volume).VolumeSizeLBN = (*Pvd).VolumeSpaceSizeL;
    (*Volume).LogicalBlockSize = ((*Pvd).LogicalBlockSizeL) as u32 ;

    (*Volume).PvdDirectorySector = (*Pvd).RootDirRecord.ExtentLocationL;
    (*Volume).PvdDirectoryLength = (*Pvd).RootDirRecord.DataLengthL;

    *IsoVolumes.as_mut_ptr().add(DeviceId as usize) = Volume;

    &Iso9660FuncTable as *const DEVVTBL
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003  Brian Palmer  <brianp@sginet.com>
//  *  Copyright (C) 2009       Hervé Poussineau  <hpoussin@reactos.org>
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
// #ifndef _M_ARM
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(FILESYSTEM);
//
// #define SECTORSIZE 2048
// #define TAG_ISO_VOLUME 'VosI'
// #define TAG_ISO_BUFFER 'BosI'
// #define TAG_ISO_FILE 'FosI'
//
// typedef struct _ISO_VOLUME_INFO
// {
//     ULONG VolumeSizeLBN;    ///< In Logical Block Numbers (LBN)
//     ULONG LogicalBlockSize; ///< Should be SECTORSIZE
//     ULONG PvdDirectorySector;
//     ULONG PvdDirectoryLength;
//     ULONG DirectoryPathLength;
//     ULONG DirectoryLength;
//     PVOID DirectoryBuffer;
//     UCHAR DirectoryPath[261];
// } ISO_VOLUME_INFO, *PISO_VOLUME_INFO;
//
// static PISO_VOLUME_INFO IsoVolumes[MAX_FDS];
//
// static
// PCSTR
// IsoLastPathSeparator(
//     _In_ PCSTR Path)
// {
//     PCSTR Last = NULL;
//
//     ASSERT(Path != NULL);
//
//     while (*Path != ANSI_NULL)
//     {
//         if (*Path == '\\' || *Path == '/')
//             Last = Path;
//
//         ++Path;
//     }
//
//     return Last;
// }
//
// static BOOLEAN IsoSearchDirectoryBufferForFile(PVOID DirectoryBuffer, ULONG DirectoryLength, PCHAR FileName, PISO_FILE_INFO IsoFileInfoPointer)
// {
//     PDIR_RECORD    Record;
//     ULONG        Offset;
//     ULONG i;
//     CHAR Name[32];
//
//     TRACE("IsoSearchDirectoryBufferForFile() DirectoryBuffer = 0x%x DirectoryLength = %d FileName = %s\n", (DirectoryBuffer) as usize, (DirectoryLength) as usize, (FileName) as usize);
//
//     Offset = 0;
//     Record = (PDIR_RECORD)DirectoryBuffer;
//     while (TRUE)
//     {
//         Offset = Offset + Record->RecordLength;
//         Record = (PDIR_RECORD)((ULONG_PTR)DirectoryBuffer + Offset);
//
//         if (Record->RecordLength == 0)
//         {
//             Offset = ROUND_UP(Offset, SECTORSIZE);
//             Record = (PDIR_RECORD)((ULONG_PTR)DirectoryBuffer + Offset);
//         }
//
//         if (Offset >= DirectoryLength)
//             return FALSE;
//
//         if (Record->FileIdLength == 1 && Record->FileId[0] == 0)
//         {
//             TRACE("Name '.'\n");
//         }
//         else if (Record->FileIdLength == 1 && Record->FileId[0] == 1)
//         {
//             TRACE("Name '..'\n");
//         }
//         else
//         {
//             for (i = 0; i < Record->FileIdLength && Record->FileId[i] != ';'; i++)
//                 Name[i] = Record->FileId[i];
//             Name[i] = ANSI_NULL;
//             TRACE("Name '%s'\n", (Name) as usize);
//
//             if (_stricmp(FileName, Name) == 0)
//             {
//                 IsoFileInfoPointer->FileStart = Record->ExtentLocationL;
//                 IsoFileInfoPointer->FileSize = Record->DataLengthL;
//                 IsoFileInfoPointer->FilePointer = 0;
//                 IsoFileInfoPointer->Attributes = Record->FileFlags;
//                 return TRUE;
//             }
//         }
//     }
//
//     return FALSE;
// }
//
//
// /*
//  * IsoBufferDirectory()
//  * This function allocates a buffer, reads the specified directory
//  * and returns a pointer to that buffer into pDirectoryBuffer. The
//  * function returns an ARC error code. The directory is specified
//  * by its starting sector and length.
//  */
// static ARC_STATUS IsoBufferDirectory(ULONG DeviceId, ULONG DirectoryStartSector, ULONG DirectoryLength,
//     PVOID* pDirectoryBuffer)
// {
//     PVOID DirectoryBuffer;
//     ULONG SectorCount;
//     LARGE_INTEGER Position;
//     ULONG Count;
//     ARC_STATUS Status;
//
//     TRACE("IsoBufferDirectory() DirectoryStartSector = %d DirectoryLength = %d\n", (DirectoryStartSector) as usize, (DirectoryLength) as usize);
//
//     SectorCount = ROUND_UP(DirectoryLength, SECTORSIZE) / SECTORSIZE;
//     TRACE("Trying to read (DirectoryCount) %d sectors.\n", (SectorCount) as usize);
//
//     //
//     // Attempt to allocate memory for directory buffer
//     //
//     TRACE("Trying to allocate (DirectoryLength) %d bytes.\n", (DirectoryLength) as usize);
//     DirectoryBuffer = FrLdrTempAlloc(DirectoryLength, TAG_ISO_BUFFER);
//     if (!DirectoryBuffer)
//         return ENOMEM;
//
//     //
//     // Now read directory contents into DirectoryBuffer
//     //
//     Position.HighPart = 0;
//     Position.LowPart = DirectoryStartSector * SECTORSIZE;
//     Status = ArcSeek(DeviceId, &Position, SeekAbsolute);
//     if (Status != ESUCCESS)
//     {
//         FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);
//         return Status;
//     }
//     Status = ArcRead(DeviceId, DirectoryBuffer, SectorCount * SECTORSIZE, &Count);
//     if (Status != ESUCCESS || Count != SectorCount * SECTORSIZE)
//     {
//         FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);
//         return EIO;
//     }
//
//     *pDirectoryBuffer = DirectoryBuffer;
//     return ESUCCESS;
// }
//
// /*
//  * IsoLookupFile()
//  * This function searches the file system for the
//  * specified filename and fills in an ISO_FILE_INFO structure
//  * with info describing the file, etc. returns ARC error code
//  */
// static ARC_STATUS IsoLookupFile(PCSTR FileName, ULONG DeviceId, PISO_FILE_INFO IsoFileInfo)
// {
//     PCSTR FullPath = FileName;
//     PISO_VOLUME_INFO Volume;
//     PCSTR FileNameStr;
//     ULONG i, DirectoryPathLength, DirectorySector, DirectoryLength;
//     PVOID DirectoryBuffer;
//     ULONG NumberOfPathParts;
//     CHAR PathBuffer[261];
//     CHAR* PathPart;
//     ARC_STATUS Status;
//     BOOLEAN DoFullLookup;
//     UCHAR FileAttributes;
//
//     TRACE("IsoLookupFile() FileName = %s\n", (FileName) as usize);
//
//     Volume = IsoVolumes[DeviceId];
//
//     /*
//      * Extract the file name
//      * '\test.ini' --> 'test.ini'
//      * '\folder\app.exe' --> 'app.exe'
//      */
//     FileNameStr = IsoLastPathSeparator(FileName) + 1;
//
//     /*
//      * Extract the directory path, including trailing slash
//      * '\test.ini' --> '\'
//      * '\folder\app.exe' --> '\folder\'
//      */
//     DirectoryPathLength = FileNameStr - FileName;
//
//     /* See if this directory has been buffered before */
//     DoFullLookup = (DirectoryPathLength != Volume->DirectoryPathLength) ||
//                    !RtlEqualMemory(FileName, Volume->DirectoryPath, DirectoryPathLength);
//     if (!DoFullLookup)
//     {
//         PathPart = (CHAR*)FileNameStr;
//         DirectoryBuffer = Volume->DirectoryBuffer;
//         DirectoryLength = Volume->DirectoryLength;
//
//         NumberOfPathParts = 1;
//     }
//     else
//     {
//         PathPart = PathBuffer;
//         DirectorySector = Volume->PvdDirectorySector;
//         DirectoryLength = Volume->PvdDirectoryLength;
//
//         /* Skip leading path separator, if any */
//         if (*FileName == '\\' || *FileName == '/')
//             ++FileName;
//         PathPart[0] = ANSI_NULL;
//
//         /* Figure out how many sub-directories we are nested in */
//         NumberOfPathParts = FsGetNumPathParts(FileName);
//     }
//
//     //
//     // Loop once for each part
//     //
//     for (i=0; i<NumberOfPathParts; i++)
//     {
//         if (DoFullLookup)
//         {
//             /* Get first path part */
//             FsGetFirstNameFromPath(PathPart, FileName);
//
//             /* Advance to the next part of the path */
//             while ((*FileName != ANSI_NULL) && (*FileName != '\\') && (*FileName != '/'))
//             {
//                 FileName++;
//             }
//             FileName++;
//
//             /* Buffer the directory contents */
//             Status = IsoBufferDirectory(DeviceId,
//                                         DirectorySector,
//                                         DirectoryLength,
//                                         &DirectoryBuffer);
//             if (Status != ESUCCESS)
//                 return Status;
//
//             /* Save the directory buffer */
//             if ((i + 1) >= NumberOfPathParts)
//             {
//                 if (Volume->DirectoryBuffer)
//                     FrLdrTempFree(Volume->DirectoryBuffer, TAG_ISO_BUFFER);
//
//                 Volume->DirectoryPathLength = DirectoryPathLength;
//                 Volume->DirectoryBuffer = DirectoryBuffer;
//                 Volume->DirectoryLength = DirectoryLength;
//
//                 RtlCopyMemory(Volume->DirectoryPath, FullPath, DirectoryPathLength);
//             }
//         }
//
//         /* Search for file name in directory */
//         if (!IsoSearchDirectoryBufferForFile(DirectoryBuffer,
//                                              DirectoryLength,
//                                              PathPart,
//                                              IsoFileInfo))
//         {
//             /* Free the directory buffer that wasn't cached */
//             if ((i + 1) < NumberOfPathParts)
//             {
//                 ASSERT(DirectoryBuffer != Volume->DirectoryBuffer);
//                 FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);
//             }
//             return ENOENT;
//         }
//
//         //
//         // If we have another sub-directory to go then
//         // grab the start sector and file size
//         //
//         if ((i+1) < NumberOfPathParts)
//         {
//             FrLdrTempFree(DirectoryBuffer, TAG_ISO_BUFFER);
//
//             DirectorySector = IsoFileInfo->FileStart;
//             DirectoryLength = IsoFileInfo->FileSize;
//         }
//     }
//
//     /* Re-map the attributes to ARC file attributes */
//     FileAttributes = IsoFileInfo->Attributes;
//     IsoFileInfo->Attributes = ReadOnlyFile;
//     if (FileAttributes & ISO_ATTR_HIDDEN)
//         IsoFileInfo->Attributes |= HiddenFile;
//     if (FileAttributes & ISO_ATTR_DIRECTORY)
//         IsoFileInfo->Attributes |= DirectoryFile;
//
//     /* Copy the file name, perhaps truncated */
//     IsoFileInfo->FileNameLength = (ULONG)strlen(PathPart);
//     IsoFileInfo->FileNameLength = min(IsoFileInfo->FileNameLength, sizeof(IsoFileInfo->FileName) - 1);
//     RtlCopyMemory(IsoFileInfo->FileName, PathPart, IsoFileInfo->FileNameLength);
//
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS IsoClose(ULONG FileId)
// {
//     PISO_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
//     FrLdrTempFree(FileHandle, TAG_ISO_FILE);
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS IsoGetFileInformation(ULONG FileId, FILEINFORMATION* Information)
// {
//     PISO_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
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
//     TRACE("IsoGetFileInformation(%lu) -> FileSize = %lu, FilePointer = 0x%lx\n", (//           FileId) as usize, (Information->EndingAddress.LowPart) as usize, (Information->CurrentAddress.LowPart) as usize);
//
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS IsoOpen(CHAR* Path, OPENMODE OpenMode, ULONG* FileId)
// {
//     PISO_FILE_INFO FileHandle;
//     ULONG DeviceId;
//     ARC_STATUS Status;
//
//     if (OpenMode != OpenReadOnly)
//         return EACCES;
//
//     DeviceId = FsGetDeviceId(*FileId);
//
//     TRACE("IsoOpen() FileName = %s\n", (Path) as usize);
//
//     FileHandle = FrLdrTempAlloc(sizeof(*FileHandle), TAG_ISO_FILE);
//     if (!FileHandle)
//         return ENOMEM;
//
//     Status = IsoLookupFile(Path, DeviceId, FileHandle);
//     if (Status != ESUCCESS)
//     {
//         FrLdrTempFree(FileHandle, TAG_ISO_FILE);
//         return ENOENT;
//     }
//
//     FsSetDeviceSpecific(*FileId, FileHandle);
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS IsoRead(ULONG FileId, VOID* Buffer, ULONG N, ULONG* Count)
// {
//     ARC_STATUS Status;
//     PISO_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
//     UCHAR SectorBuffer[SECTORSIZE];
//     LARGE_INTEGER Position;
//     ULONG DeviceId;
//     ULONG SectorNumber;
//     ULONG OffsetInSector;
//     ULONG LengthInSector;
//     ULONG NumberOfSectors;
//     ULONG BytesRead;
//
//     TRACE("IsoRead() Buffer = %p, N = %lu\n", (Buffer) as usize, (N) as usize);
//
//     DeviceId = FsGetDeviceId(FileId);
//     *Count = 0;
//
//     //
//     // If the user is trying to read past the end of
//     // the file then return success with Count == 0.
//     //
//     if (FileHandle->FilePointer >= FileHandle->FileSize)
//     {
//         return ESUCCESS;
//     }
//
//     //
//     // If the user is trying to read more than there is to read
//     // then adjust the amount to read.
//     //
//     if (FileHandle->FilePointer + N > FileHandle->FileSize)
//     {
//         N = FileHandle->FileSize - FileHandle->FilePointer;
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
//     // | Sector  1 | Sector  2 | Sector  3 | Sector  4 |
//     // +-----------+-----------+-----------+-----------+
//     //    |                                    |
//     //    +---------------+--------------------+
//     //                    |
//     // N -----------------+
//     //
//     // 1 - The first calculation (and read) will align
//     //     the file pointer with the next sector
//     //     boundary (if we are supposed to read that much)
//     // 2 - The next calculation (and read) will read
//     //     in all the full sectors that the requested
//     //     amount of data would cover (in this case
//     //     sectors 2 & 3).
//     // 3 - The last calculation (and read) would read
//     //     in the remainder of the data requested out of
//     //     the last sector.
//     //
//
//
//     //
//     // Only do the first read if we
//     // aren't aligned on a cluster boundary
//     //
//     if (FileHandle->FilePointer % SECTORSIZE)
//     {
//         //
//         // Do the math for our first read
//         //
//         SectorNumber = FileHandle->FileStart + (FileHandle->FilePointer / SECTORSIZE);
//         OffsetInSector = FileHandle->FilePointer % SECTORSIZE;
//         LengthInSector = min(N, SECTORSIZE - OffsetInSector);
//
//         //
//         // Now do the read and update Count, N, FilePointer, & Buffer
//         //
//         Position.HighPart = 0;
//         Position.LowPart = SectorNumber * SECTORSIZE;
//         Status = ArcSeek(DeviceId, &Position, SeekAbsolute);
//         if (Status != ESUCCESS)
//         {
//             return Status;
//         }
//         Status = ArcRead(DeviceId, SectorBuffer, SECTORSIZE, &BytesRead);
//         if (Status != ESUCCESS || BytesRead != SECTORSIZE)
//         {
//             return EIO;
//         }
//         RtlCopyMemory(Buffer, SectorBuffer + OffsetInSector, LengthInSector);
//         *Count += LengthInSector;
//         N -= LengthInSector;
//         FileHandle->FilePointer += LengthInSector;
//         Buffer = (PVOID)((ULONG_PTR)Buffer + LengthInSector);
//     }
//
//     //
//     // Do the math for our second read (if any data left)
//     //
//     if (N > 0)
//     {
//         //
//         // Determine how many full clusters we need to read
//         //
//         NumberOfSectors = (N / SECTORSIZE);
//
//         SectorNumber = FileHandle->FileStart + (FileHandle->FilePointer / SECTORSIZE);
//
//         //
//         // Now do the read and update Count, N, FilePointer, & Buffer
//         //
//         Position.HighPart = 0;
//         Position.LowPart = SectorNumber * SECTORSIZE;
//         Status = ArcSeek(DeviceId, &Position, SeekAbsolute);
//         if (Status != ESUCCESS)
//         {
//             return Status;
//         }
//         Status = ArcRead(DeviceId, Buffer, NumberOfSectors * SECTORSIZE, &BytesRead);
//         if (Status != ESUCCESS || BytesRead != NumberOfSectors * SECTORSIZE)
//         {
//             return EIO;
//         }
//
//         *Count += NumberOfSectors * SECTORSIZE;
//         N -= NumberOfSectors * SECTORSIZE;
//         FileHandle->FilePointer += NumberOfSectors * SECTORSIZE;
//         Buffer = (PVOID)((ULONG_PTR)Buffer + NumberOfSectors * SECTORSIZE);
//     }
//
//     //
//     // Do the math for our third read (if any data left)
//     //
//     if (N > 0)
//     {
//         SectorNumber = FileHandle->FileStart + (FileHandle->FilePointer / SECTORSIZE);
//
//         //
//         // Now do the read and update Count, N, FilePointer, & Buffer
//         //
//         Position.HighPart = 0;
//         Position.LowPart = SectorNumber * SECTORSIZE;
//         Status = ArcSeek(DeviceId, &Position, SeekAbsolute);
//         if (Status != ESUCCESS)
//         {
//             return Status;
//         }
//         Status = ArcRead(DeviceId, SectorBuffer, SECTORSIZE, &BytesRead);
//         if (Status != ESUCCESS || BytesRead != SECTORSIZE)
//         {
//             return EIO;
//         }
//         RtlCopyMemory(Buffer, SectorBuffer, N);
//         *Count += N;
//         FileHandle->FilePointer += N;
//     }
//
//     TRACE("IsoRead() done\n");
//
//     return ESUCCESS;
// }
//
// static
// ARC_STATUS IsoSeek(ULONG FileId, LARGE_INTEGER* Position, SEEKMODE SeekMode)
// {
//     PISO_FILE_INFO FileHandle = FsGetDeviceSpecific(FileId);
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
//     FileHandle->FilePointer = NewPosition.LowPart;
//     return ESUCCESS;
// }
//
//
// /**
//  * @brief
//  * Returns the size of the ISO-9660 volume laid on the storage media device
//  * opened via @p DeviceId.
//  *
//  * This routine is in particular used to determine the useful (volume) size
//  * of a CD-ROM media.
//  **/
// ULONGLONG
// IsoGetVolumeSize(
//     _In_ ULONG DeviceId)
// {
//     PISO_VOLUME_INFO Volume = IsoVolumes[DeviceId];
//     ASSERT(Volume);
//     return (ULONGLONG)Volume->VolumeSizeLBN * Volume->LogicalBlockSize;
// }
//
//
// static const DEVVTBL Iso9660FuncTable =
// {
//     IsoClose,
//     IsoGetFileInformation,
//     IsoOpen,
//     IsoRead,
//     IsoSeek,
//     crate::wide!("cdfs"),
// };
//
// const DEVVTBL* IsoMount(ULONG DeviceId)
// {
//     UCHAR Buffer[SECTORSIZE];
//     PPVD Pvd = (PPVD)Buffer;
//     LARGE_INTEGER Position;
//     ULONG Count;
//     ARC_STATUS Status;
//     PISO_VOLUME_INFO Volume;
//
//     TRACE("Enter IsoMount(%lu)\n", (DeviceId) as usize);
//
//     /* Read the ISO Primary Volume Descriptor */
//     Position.QuadPart = 16 * SECTORSIZE;
//     Status = ArcSeek(DeviceId, &Position, SeekAbsolute);
//     if (Status != ESUCCESS)
//         return NULL;
//     Status = ArcRead(DeviceId, Pvd, SECTORSIZE, &Count);
//     if (Status != ESUCCESS || Count < sizeof(PVD))
//         return NULL;
//
//     /* Check whether the PVD is valid */
//     if (!(Pvd->VdType == 1 && RtlEqualMemory(Pvd->StandardId, "CD001", 5) && Pvd->VdVersion == 1))
//     {
//         WARN("Unrecognized CD-ROM format\n");
//         return NULL;
//     }
//     if (Pvd->LogicalBlockSizeL != SECTORSIZE)
//     {
//         ERR("Unsupported LogicalBlockSize %u\n", (Pvd->LogicalBlockSizeL) as usize);
//         return NULL;
//     }
//
//     Count = (ULONG)((ULONGLONG)Pvd->VolumeSpaceSizeL * SECTORSIZE / 1024 / 1024);
//     TRACE("Recognized ISO9660 drive, size %lu MB (%lu sectors)\n", (//           Count) as usize, (Pvd->VolumeSpaceSizeL) as usize);
//
//     Volume = FrLdrTempAlloc(sizeof(*Volume), TAG_ISO_VOLUME);
//     if (!Volume)
//         return NULL;
//     RtlZeroMemory(Volume, sizeof(*Volume));
//
//     /*
//      * This is equivalent to using the RAW_ISO_VD::VolSpaceI field:
//      * https://github.com/reactos/reactos/blob/435482912c6dc0aaa4371e37b7336b2b1291e65f/drivers/filesystems/cdfs/cd.h#L126
//      * and also what MS CDFS does with the CdRvdVolSz() macro,
//      * in cdfs/strucsup.c!CdUpdateVcbFromVolDescriptor().
//      */
//     Volume->VolumeSizeLBN = Pvd->VolumeSpaceSizeL;
//     Volume->LogicalBlockSize = Pvd->LogicalBlockSizeL;
//
//     /* Cache some useful PVD information */
//     Volume->PvdDirectorySector = Pvd->RootDirRecord.ExtentLocationL;
//     Volume->PvdDirectoryLength = Pvd->RootDirRecord.DataLengthL;
//
//     IsoVolumes[DeviceId] = Volume;
//
//     /* Everything OK, return the ISO9660 function table */
//     return &Iso9660FuncTable;
// }
//
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

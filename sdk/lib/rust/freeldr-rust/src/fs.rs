#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_FILESYSTEM; // fs.c: DBG_DEFAULT_CHANNEL(FILESYSTEM)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER, LIST_ENTRY, PCCH, PCSTR, PSTR, PULONGLONG};
use core::ffi::c_void;

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


extern "C" {
    fn IsoGetVolumeSize(DeviceId: ULONG) -> ULONGLONG;
    fn FatGetVolumeSize(DeviceId: ULONG) -> ULONGLONG;
    fn strncmp(a: *const i8, b: *const i8, n: usize) -> i32;
}

// fs.c FileSystems[] — PRUNED to iso+fat+pxe (ntfs/ext/btrfs dropped:
// livecd boots ISO, ramdisk is FAT; their .c are commented out of
// FREELDR_BOOTLIB_SOURCE as a consequence of this omission).
//
// fs.c:69 gates PxeMount with `#if defined(_M_IX86) || defined(_M_AMD64)`
// then `#ifndef UEFIBOOT`. rustconv dropped both guards. Restore faithfully:
// PxeMount is in the table ONLY on x86/amd64 AND only in the non-UEFI
// (BIOS freeldr_pe) build. arm64 (UEFI-only) and the x86 uefildr archive
// exclude it — pxe.rs is correspondingly cfg(any(x86,x86_64)) only and the
// uefi feature mirrors the UEFIBOOT define.
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
extern "C" {
    fn IsoMount(DeviceId: ULONG) -> *const DEVVTBL;
    fn FatMount(DeviceId: ULONG) -> *const DEVVTBL;
    fn PxeMount(DeviceId: ULONG) -> *const DEVVTBL;
}
#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi"))))]
extern "C" {
    fn IsoMount(DeviceId: ULONG) -> *const DEVVTBL;
    fn FatMount(DeviceId: ULONG) -> *const DEVVTBL;
}
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
static FileSystems: [PFS_MOUNT; 3] = [IsoMount, FatMount, PxeMount];
#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi"))))]
static FileSystems: [PFS_MOUNT; 2] = [IsoMount, FatMount];

extern "C" {
    fn DbgPrint(fmt: *const i8, ...) -> NTSTATUS;
    fn UiMessageBox(msg: PCSTR);
    fn strlen(s: *const i8) -> usize;
}

// fs.c:54 `#define IS_VALID_FILEID(FileId) \
//   ((ULONG)(FileId) < _countof(FileData) && FileData[(ULONG)(FileId)].FuncTable)`
// — file-scoped macro over fs.c's static FileData[]; rustconv emitted it
// as `extern { fn IS_VALID_FILEID }` + call sites. Real cdecl wrapper
// here (fs.rs owns the FileData translation). UNIQUE (C macro).
#[no_mangle]
pub unsafe extern "C" fn IS_VALID_FILEID(FileId: ULONG) -> BOOLEAN {
    ((FileId < MAX_FDS as ULONG)
        && !FileData[FileId as usize].FuncTable.is_null()) as BOOLEAN
}

// fs.c: 'NDsF'/'vDsF' multichar literals (big-endian). AI emitted the wrong
// value (0x4E564544 "NVED"); corrected to from_be_bytes.
const TAG_DEVICE_NAME: u32 = u32::from_be_bytes(*b"NDsF");
const TAG_DEVICE: u32 = u32::from_be_bytes(*b"vDsF");

// ---- restored from fs.c (rustconv drops file-local structs/statics) ----
#[repr(C)]
pub struct DEVICE {
    pub ListEntry: LIST_ENTRY,
    pub FuncTable: *const DEVVTBL,
    pub FileFuncTable: *const DEVVTBL,
    pub DeviceName: PSTR,
    pub DeviceId: ULONG,
    pub ReferenceCount: ULONG,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILEDATA {
    pub DeviceId: ULONG,
    pub ReferenceCount: ULONG,
    pub FuncTable: *const DEVVTBL,
    pub Specific: PVOID,
}
static mut FileData: [FILEDATA; MAX_FDS] = [FILEDATA {
    DeviceId: 0,
    ReferenceCount: 0,
    FuncTable: core::ptr::null(),
    Specific: core::ptr::null_mut(),
}; MAX_FDS];
static mut DeviceListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
// ---- end restored ----

#[no_mangle]
pub unsafe extern "C" fn FsGetVolumeSize(
    DeviceId: ULONG,
    VolumeSize: PULONGLONG,
) -> ARC_STATUS {
    let pDevice: *mut DEVICE;
    let FuncTable: *const DEVVTBL;
    let ServiceName: PCWSTR;
    let mut pFsGetVolumeSize: Option<unsafe extern "C" fn(ULONG) -> ULONGLONG> = None;

    pDevice = FsGetDeviceById(DeviceId);
    if pDevice.is_null() {
        return EBADF;
    }

    *VolumeSize = 0;

    if (*pDevice).FileFuncTable.is_null() && (*pDevice).ReferenceCount <= 1 {
        let mut fs: ULONG = 0;
        while fs < FileSystems.len() as ULONG {
            (*pDevice).FileFuncTable = FileSystems[fs as usize](DeviceId);
            if !(*pDevice).FileFuncTable.is_null() {
                break;
            }
            fs += 1;
        }
    }

    FuncTable = (*pDevice).FileFuncTable;
    ServiceName = if !FuncTable.is_null() && !(*FuncTable).ServiceName.is_null() {
        (*FuncTable).ServiceName
    } else {
        core::ptr::null()
    };
    if ServiceName.is_null() {
        ERR!("Non-mounted device %lu\n", (DeviceId) as usize);
        return ENXIO;
    }

unsafe fn DumpDeviceList() {

    let mut pEntry: PLIST_ENTRY;

    DbgPrint(b"\n== Dumping ARC devices ==\n\0".as_ptr() as *const i8);

    pEntry = DeviceListHead.Flink;
    while pEntry != core::ptr::addr_of_mut!(DeviceListHead) {
        let pDevice = (pEntry as *mut u8).offset(-(core::mem::offset_of!(DEVICE, ListEntry) as isize)) as *mut DEVICE;
        let DeviceId = (*pDevice).DeviceId;
        let mut FuncTable: *const DEVVTBL;

        DbgPrint(
            b"\n\0".as_ptr() as *const i8,
        );
        DbgPrint(
            b"DeviceId %lu\n\0".as_ptr() as *const i8, (DeviceId) as usize,
        );
        DbgPrint(
            b"  RefCount: %lu\n\0".as_ptr() as *const i8, ((*pDevice).ReferenceCount) as usize,
        );
        DbgPrint(
            b"  DeviceName  : '%s'\n\0".as_ptr() as *const i8, ((*pDevice).DeviceName) as usize,
        );

        FuncTable = (*pDevice).FuncTable;
        DbgPrint(
            b"  DevFuncTable: 0x%p (In array: 0x%p) ; Name: '%S'\n\0".as_ptr() as *const i8, (FuncTable) as usize, (if DeviceId != INVALID_FILE_ID { FileData[DeviceId as usize].FuncTable } else { core::ptr::null() }) as usize, (if !FuncTable.is_null() && !(*FuncTable).ServiceName.is_null() {
                (*FuncTable).ServiceName
            } else {
                crate::wide!("n/a\0")
            }) as usize,
        );

        FuncTable = (*pDevice).FileFuncTable;
        DbgPrint(
            b"  FileFuncTable: 0x%p ; Name: '%S'\n\0".as_ptr() as *const i8, (FuncTable) as usize, (if !FuncTable.is_null() && !(*FuncTable).ServiceName.is_null() {
                (*FuncTable).ServiceName
            } else {
                crate::wide!("n/a\0")
            }) as usize,
        );

        pEntry = (*pEntry).Flink;
    }

    DbgPrint(b"\n== END Dumping ARC devices ==\n\n\0".as_ptr() as *const i8);
}

unsafe fn DumpFileTable() {
    let mut i: ULONG;

    DbgPrint(b"\n== Dumping ARC files table ==\n\0".as_ptr() as PCHAR);

    i = 0;
    while i < (core::mem::size_of_val(&FileData) / core::mem::size_of::<FILEDATA>()) as ULONG {
        let pFileData: *mut FILEDATA = &mut FileData[i as usize];
        let FuncTable: *const DEVVTBL;

        if (*pFileData).FuncTable.is_null() {
            i += 1;
            continue;
        }

        DbgPrint(
            b"\n\
              FileId %lu\n\
              \x20\x20RefCount: %lu\n\
              \x20\x20ParentDeviceId: %lu\n\
              \x20\x20Specific: 0x%p\n\0".as_ptr() as PCHAR, (i) as usize, ((*pFileData).ReferenceCount) as usize, ((*pFileData).DeviceId) as usize, ((*pFileData).Specific) as usize,
        );

        FuncTable = (*pFileData).FuncTable;
        DbgPrint(
            b"\x20\x20FuncTable: 0x%p ; Name: '%S'\n\0".as_ptr() as PCHAR, (FuncTable) as usize, (if !FuncTable.is_null() && !(*FuncTable).ServiceName.is_null() {
                (*FuncTable).ServiceName
            } else {
                crate::wide!("n/a\0") as PWSTR
            }) as usize,
        );

        i += 1;
    }

    DbgPrint(b"\n== END Dumping ARC files table ==\n\n\0".as_ptr() as PCHAR);
}

unsafe fn NormalizeArcDeviceName(
    DeviceName: PCCH,
    Length: PSIZE_T
) -> PCHAR
{
    let mut Count: u32;
    let NameLength: usize;
    let mut p: PCCH;
    let End: PCCH;
    let mut q: PCHAR;
    let NormName: PCHAR;

    NameLength = *Length;
    End = DeviceName.add(NameLength);

    Count = 0;
    p = DeviceName;
    while p < End {
        if p.add(1) < End && *p == b'(' as i8 && *p.add(1) == b')' as i8 {
            Count += 1;
        }
        p = p.add(1);
    }

    if Count == 0 {
        return DeviceName as PCHAR;
    }

    *Length = NameLength + Count as usize;

    NormName = FrLdrTempAlloc(*Length, TAG_DEVICE_NAME) as PCHAR;
    if NormName.is_null() {
        return core::ptr::null_mut();
    }

    p = DeviceName;
    q = NormName;
    while p < End {
        *q = *p;
        q = q.add(1);
        if p.add(1) < End && *p == b'(' as i8 && *p.add(1) == b')' as i8 {
            *q = b'0' as i8;
            q = q.add(1);
        }
        p = p.add(1);
    }

    NormName
}

#[no_mangle]
pub unsafe extern "C" fn ArcOpen(Path: PCHAR, OpenMode: OPENMODE, FileId: PULONG) -> ARC_STATUS
{
    let mut Status: ARC_STATUS = ESUCCESS;
    let mut i: u32;
    let mut pEntry: PLIST_ENTRY;
    let mut pDevice: *mut DEVICE;
    let mut DeviceName: PCHAR;
    let mut FileName: PSTR;
    let mut Length: usize;
    let DeviceOpenMode: OPENMODE;
    let mut DeviceId: u32;

    TRACE!("Opening file '%s'...\n", (Path) as usize);

    *FileId = INVALID_FILE_ID;

    FileName = strrchr(Path, ')' as i32);
    if FileName.is_null() {
        return EINVAL;
    }
    FileName = FileName.offset(1);

    Length = FileName.offset_from(Path) as usize;
    DeviceName = NormalizeArcDeviceName(Path, core::ptr::addr_of_mut!(Length));
    if DeviceName.is_null() {
        return ENOMEM;
    }

    if OpenMode == OpenReadOnly || OpenMode == OpenWriteOnly {
        DeviceOpenMode = OpenMode;
    } else {
        DeviceOpenMode = OpenReadWrite;
    }

    pDevice = core::ptr::null_mut();
    pEntry = DeviceListHead.Flink;
    while pEntry != core::ptr::addr_of_mut!(DeviceListHead) {
        pDevice = (pEntry as *mut u8).offset(-(core::mem::offset_of!(DEVICE, ListEntry) as isize)) as *mut DEVICE;
        if (strlen((*pDevice).DeviceName) == Length) && (strncmp((*pDevice).DeviceName, DeviceName, Length) == 0) {
            break;
        }
        pEntry = (*pEntry).Flink;
    }

    if DeviceName != Path {
        FrLdrTempFree(DeviceName as *mut core::ffi::c_void, TAG_DEVICE_NAME);
    }
    DeviceName = core::ptr::null_mut();

    if pEntry == core::ptr::addr_of_mut!(DeviceListHead) {
        return ENODEV;
    }

    if (*pDevice).ReferenceCount == 0 {
        DeviceId = 0;
        loop {
            if DeviceId >= _countof(core::ptr::addr_of!(FileData)) {
                return EMFILE;
            }
            if FileData[DeviceId as usize].FuncTable.is_null() {
                break;
            }
            DeviceId += 1;
        }

        FileData[DeviceId as usize].ReferenceCount = 0;
        FileData[DeviceId as usize].FuncTable = (*pDevice).FuncTable;
        Status = ((*(*pDevice).FuncTable).Open.unwrap())((*pDevice).DeviceName, DeviceOpenMode, core::ptr::addr_of_mut!(DeviceId));
        if Status != ESUCCESS {
            FileData[DeviceId as usize].FuncTable = core::ptr::null_mut();
            return Status;
        }
        (*pDevice).DeviceId = DeviceId;
    } else {
        DeviceId = (*pDevice).DeviceId;
    }

    (*pDevice).ReferenceCount += 1;
    FileData[DeviceId as usize].ReferenceCount += 1;

    if *FileName == 0 {
        *FileId = DeviceId;
        return ESUCCESS;
    }

    i = 0;
    'find_file_slot: loop {
        if i >= _countof(core::ptr::addr_of!(FileData)) {
            Status = EMFILE;
            break 'find_file_slot;
        }
        if FileData[i as usize].FuncTable.is_null() {
            break;
        }
        i += 1;
    }

    if i < _countof(core::ptr::addr_of!(FileData)) {
        if (*pDevice).FileFuncTable.is_null() && ((*pDevice).ReferenceCount <= 1) {
            let mut fs: u32 = 0;
            while fs < _countof(core::ptr::addr_of!(FileSystems)) {
                (*pDevice).FileFuncTable = FileSystems[fs as usize](DeviceId);
                if !(*pDevice).FileFuncTable.is_null() {
                    break;
                }
                fs += 1;
            }
        }
        if (*pDevice).FileFuncTable.is_null() {
            Status = ENOENT;
        } else {
            FileData[i as usize].DeviceId = DeviceId;
            FileData[i as usize].ReferenceCount = 0;
            FileData[i as usize].FuncTable = (*pDevice).FileFuncTable;
            *FileId = i;
            Status = ((*FileData[i as usize].FuncTable).Open.unwrap())(FileName, OpenMode, FileId);
            if Status != ESUCCESS {
                FileData[i as usize].DeviceId = INVALID_FILE_ID;
                FileData[i as usize].FuncTable = core::ptr::null_mut();
                FileData[i as usize].Specific = core::ptr::null_mut();
                *FileId = INVALID_FILE_ID;
            } else {
                FileData[i as usize].ReferenceCount += 1;
            }
        }
    }

    if Status != ESUCCESS {
        FileData[DeviceId as usize].ReferenceCount -= 1;
        if FileData[DeviceId as usize].ReferenceCount == 0 {
            ((*FileData[DeviceId as usize].FuncTable).Close.unwrap())(DeviceId);
            FileData[DeviceId as usize].DeviceId = INVALID_FILE_ID;
            FileData[DeviceId as usize].FuncTable = core::ptr::null_mut();
            FileData[DeviceId as usize].Specific = core::ptr::null_mut();
        }
        (*pDevice).ReferenceCount -= 1;
        if (*pDevice).ReferenceCount == 0 {
            (*pDevice).DeviceId = INVALID_FILE_ID;
        }
    }

    return Status;
}

unsafe fn FsGetDeviceById(DeviceId: ULONG) -> *mut DEVICE {
    let mut pEntry: PLIST_ENTRY = DeviceListHead.Flink;
    
    while pEntry != &DeviceListHead as *const _ as *mut _ {
        let pDevice = (pEntry as *mut u8).offset(-(core::mem::offset_of!(DEVICE, ListEntry) as isize)) as *mut DEVICE;
        if (*pDevice).DeviceId == DeviceId {
            return pDevice;
        }
        pEntry = (*pEntry).Flink;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn ArcClose(FileId: ULONG) -> ARC_STATUS {

    let DeviceId: ULONG;
    let pDevice: *mut DEVICE;

    if IS_VALID_FILEID(FileId) == 0 {
        return EBADF;
    }

    DeviceId = FileData[FileId as usize].DeviceId;

    if FileData[FileId as usize].ReferenceCount > 0 {
        FileData[FileId as usize].ReferenceCount -= 1;
        if FileData[FileId as usize].ReferenceCount == 0 {
            ((*FileData[FileId as usize].FuncTable).Close.unwrap())(FileId);
            FileData[FileId as usize].DeviceId = INVALID_FILE_ID;
            FileData[FileId as usize].FuncTable = core::ptr::null_mut();
            FileData[FileId as usize].Specific = core::ptr::null_mut();
        }
    }

    pDevice = FsGetDeviceById(FileId);
    if !pDevice.is_null() {
        if (*pDevice).ReferenceCount > 0 {
            (*pDevice).ReferenceCount -= 1;
            if (*pDevice).ReferenceCount == 0 {
                (*pDevice).DeviceId = INVALID_FILE_ID;
            }
        }
    }

    if IS_VALID_FILEID(DeviceId) != 0 {
        ArcClose(DeviceId);
    }

    ESUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ArcRead(
    FileId: ULONG,
    Buffer: *mut VOID,
    N: ULONG,
    Count: *mut ULONG,
) -> ARC_STATUS {
    
    if IS_VALID_FILEID(FileId) == 0 {
        return EBADF;
    }
    ((*(*FileData.as_ptr().add(FileId as usize)).FuncTable).Read.unwrap())(FileId, Buffer, N, Count)
}

#[no_mangle]
pub unsafe extern "C" fn ArcSeek(FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: SEEKMODE) -> ARC_STATUS {
    if IS_VALID_FILEID(FileId) == 0 {
        return EBADF;
    }
    ((*(*FileData.as_mut_ptr().add(FileId as usize)).FuncTable).Seek.unwrap())(FileId, Position, SeekMode)
}

#[no_mangle]
pub unsafe extern "C" fn ArcGetFileInformation(
    FileId: ULONG,
    Information: *mut FILEINFORMATION,
) -> ARC_STATUS {

    if IS_VALID_FILEID(FileId) == 0 {
        return EBADF;
    }
    ((*(*FileData.as_ptr().add(FileId as usize)).FuncTable).GetFileInformation.unwrap())(
        FileId,
        Information,
    )
}

#[no_mangle]
pub unsafe extern "C" fn FileSystemError(ErrorString: PCSTR) {
    
    ERR!("%s\n", (ErrorString) as usize);
    UiMessageBox(ErrorString);
}

#[no_mangle]
pub unsafe extern "C" fn FsOpenFile(
    mut FileName: PCSTR,
    DefaultPath: PCSTR,
    OpenMode: OPENMODE,
    FileId: PULONG,
) -> ARC_STATUS
{
    let mut Status: NTSTATUS;
    let cchPathLen: usize;
    let mut FullPath: [u8; MAX_PATH] = [0; MAX_PATH];

    if strrchr(FileName as *const i8, ')' as i32).is_null() {
        if !DefaultPath.is_null() {
            Status = RtlStringCbCopyA(FullPath.as_mut_ptr() as *mut i8, core::mem::size_of_val(&FullPath), DefaultPath as *const i8);
            if !NT_SUCCESS(Status) {
                return ENAMETOOLONG;
            }
        }

        cchPathLen = min(
            core::mem::size_of_val(&FullPath) / core::mem::size_of::<u8>(),
            strlen(FullPath.as_ptr() as *const i8) as usize,
        );
        if cchPathLen >= core::mem::size_of_val(&FullPath) / core::mem::size_of::<u8>() {
            return ENAMETOOLONG;
        }

        if (*FileName != ((b'\\') as i8) && *FileName != ((b'/') as i8)) &&
            cchPathLen > 0 && (FullPath[cchPathLen - 1] != b'\\' && FullPath[cchPathLen - 1] != b'/') {
            Status = RtlStringCbCatA(FullPath.as_mut_ptr() as *mut i8, core::mem::size_of_val(&FullPath), b"\\\0".as_ptr() as *const i8);
            if !NT_SUCCESS(Status) {
                return ENAMETOOLONG;
            }
        } else if (*FileName == ((b'\\') as i8) || *FileName == ((b'/') as i8)) &&
                  cchPathLen > 0 && (FullPath[cchPathLen - 1] == b'\\' || FullPath[cchPathLen - 1] == b'/') {
            while *FileName == ((b'\\') as i8) || *FileName == ((b'/') as i8) {
                FileName = FileName.offset(1);
            }
        }
    }

    Status = RtlStringCbCatA(FullPath.as_mut_ptr() as *mut i8, core::mem::size_of_val(&FullPath), FileName as *const i8);
    if !NT_SUCCESS(Status) {
        return ENAMETOOLONG;
    }

    return ArcOpen(FullPath.as_mut_ptr() as PCHAR, OpenMode, FileId);
}

#[cfg(not(target_arch = "arm"))]
    {
        if _wcsicmp(ServiceName, w!("cdfs").as_ptr() as PCWSTR) == 0 {
            pFsGetVolumeSize = Some(IsoGetVolumeSize);
        }
    }
    
    if pFsGetVolumeSize.is_none() &&
        (_wcsicmp(ServiceName, w!("fastfat").as_ptr() as PCWSTR) == 0 ||
         _wcsicmp(ServiceName, w!("vfatfs").as_ptr() as PCWSTR) == 0)
    {
        pFsGetVolumeSize = Some(FatGetVolumeSize);
    }

    if pFsGetVolumeSize.is_none() {
        ERR!("Unsupported file system '%S' on storage device %lu\n", (ServiceName) as usize, (DeviceId) as usize);
        return EIO;
    }

    *VolumeSize = pFsGetVolumeSize.unwrap()(DeviceId);
    ESUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn FsGetNumPathParts(Path: PCSTR) -> ULONG {
    let mut i: usize;
    let len: usize;
    let mut num: ULONG;

    len = strlen(Path as *const i8);

    i = 0;
    num = 0;
    while i < len {
        if (*Path.add(i) == b'\\' as i8) || (*Path.add(i) == b'/' as i8) {
            num += 1;
        }
        i += 1;
    }
    num += 1;

    TRACE!("FsGetNumPathParts() Path = %s NumPathParts = %d\n", (Path) as usize, (num) as usize);

    return num;
}

#[no_mangle]
pub unsafe extern "C" fn FsGetFirstNameFromPath(Buffer: PCHAR, Path: PCSTR) {

    let len = strlen(Path);

    let mut i: usize = 0;
    while i < len {
        let c = *Path.add(i);
        if c == b'\\' as i8 || c == b'/' as i8 {
            break;
        } else {
            *Buffer.add(i) = c;
        }
        i += 1;
    }

    *Buffer.add(i) = 0;

    TRACE!("FsGetFirstNameFromPath() Path = %s FirstName = %s\n", (Path) as usize, (Buffer) as usize);
}

#[no_mangle]
pub unsafe extern "C" fn FsRegisterDevice(
    DeviceName: PCSTR,
    FuncTable: *const DEVVTBL,
) {

    let pNewEntry: *mut DEVICE;
    let Length: usize;

    TRACE!("FsRegisterDevice(%s)\n", (DeviceName) as usize);

    Length = strlen(DeviceName as *const i8) + 1;
    pNewEntry = FrLdrTempAlloc(
        core::mem::size_of::<DEVICE>() + Length,
        TAG_DEVICE,
    ) as *mut DEVICE;
    if pNewEntry.is_null() {
        return;
    }
    // Zero the header first (upstream 56481e1b8d4): FrLdrTempAlloc memory is
    // not cleared, and FileFuncTable -- which ArcOpen tests to decide whether
    // the device still needs a filesystem mounted -- was otherwise left as
    // heap garbage.
    core::ptr::write_bytes(pNewEntry, 0, 1);
    (*pNewEntry).FuncTable = FuncTable;
    (*pNewEntry).DeviceId = INVALID_FILE_ID;
    (*pNewEntry).ReferenceCount = 0;
    (*pNewEntry).DeviceName = pNewEntry.add(1) as PSTR;
    core::ptr::copy_nonoverlapping(
        DeviceName as *const u8,
        (*pNewEntry).DeviceName as *mut u8,
        Length,
    );

    InsertHeadList(
        core::ptr::addr_of_mut!(DeviceListHead),
        core::ptr::addr_of_mut!((*pNewEntry).ListEntry),
    );
}

#[no_mangle]
pub unsafe extern "C" fn FsGetServiceName(FileId: u32) -> PCWSTR {
    if IS_VALID_FILEID(FileId) == 0 {
        return core::ptr::null();
    }
    (*(*FileData.as_ptr().add(FileId as usize)).FuncTable).ServiceName
}

#[no_mangle]
pub unsafe extern "C" fn FsSetDeviceSpecific(FileId: ULONG, Specific: PVOID) {
    if IS_VALID_FILEID(FileId) == 0 {
        return;
    }
    FileData[FileId as usize].Specific = Specific;
}

#[no_mangle]
pub unsafe extern "C" fn FsGetDeviceSpecific(FileId: ULONG) -> PVOID {
    if IS_VALID_FILEID(FileId) == 0 {
        return core::ptr::null_mut();
    }
    FileData[FileId as usize].Specific
}

#[no_mangle]
pub unsafe extern "C" fn FsGetDeviceId(FileId: ULONG) -> ULONG {
    if FileId >= (core::mem::size_of_val(&FileData) / core::mem::size_of::<FILEDATA>()) as ULONG {
        return INVALID_FILE_ID;
    }
    FileData[FileId as usize].DeviceId
}

#[no_mangle]
pub unsafe extern "C" fn FsInit() {
    crate::frldr::dbgcon(b'F'); crate::frldr::dbgcon(b's');
    let mut i: u32;

    // rustconv translated C `memset(FileData, 0, sizeof(FileData))` as
    // `write_bytes(FileData.as_mut_ptr(), 0, sizeof_val)`. That's a UB-class
    // bug: write_bytes::<T>(ptr, val, count) writes count * sizeof::<T>()
    // bytes, so this was clobbering `count * sizeof(FILEDATA)` = 60 * 16 *
    // 16 = ~15KB of BSS PAST FileData, including FrLdrDefaultHeap,
    // FrLdrTempHeap, and any other static-mut in adjacent BSS — causing
    // FrLdrHeapAlloc to read back NULL and hang the freelist walker.
    // See feedback_write_bytes_element_count.md.
    core::ptr::write_bytes(
        FileData.as_mut_ptr() as *mut u8,
        0,
        core::mem::size_of_val(&FileData),
    );
    i = 0;
    while i < FileData.len() as u32 {
        FileData[i as usize].DeviceId = INVALID_FILE_ID;
        i += 1;
    }

    InitializeListHead(&mut DeviceListHead);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003  Brian Palmer  <brianp@sginet.com>
//  *  Copyright (C) 2008-2009  Hervé Poussineau  <hpoussin@reactos.org>
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
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(FILESYSTEM);
//
// /* GLOBALS ********************************************************************/
//
// #define TAG_DEVICE_NAME 'NDsF'
// #define TAG_DEVICE 'vDsF'
//
// typedef struct tagDEVICE
// {
//     LIST_ENTRY ListEntry;
//     const DEVVTBL* FuncTable;     ///< Driver function table.
//     const DEVVTBL* FileFuncTable; ///< Used only by mounted storage devices.
//     PSTR DeviceName;
//     ULONG DeviceId; ///< Entry ID in FileData when the device gets referenced.
//     ULONG ReferenceCount;
// } DEVICE;
//
// typedef struct tagFILEDATA
// {
//     ULONG DeviceId; ///< Parent device's ID.
//     ULONG ReferenceCount;
//     const DEVVTBL* FuncTable;
//     PVOID Specific;
// } FILEDATA;
//
// static FILEDATA FileData[MAX_FDS];
// static LIST_ENTRY DeviceListHead;
//
// #define IS_VALID_FILEID(FileId) \
//     ((ULONG)(FileId) < _countof(FileData) && FileData[(ULONG)(FileId)].FuncTable)
//
// typedef const DEVVTBL* (*PFS_MOUNT)(ULONG DeviceId);
//
// PFS_MOUNT FileSystems[] =
// {
// #ifndef _M_ARM
//     IsoMount,
// #endif
//     FatMount,
//     BtrFsMount,
// #ifndef _M_ARM
//     NtfsMount,
//     ExtMount,
// #endif
// #if defined(_M_IX86) || defined(_M_AMD64)
// #ifndef UEFIBOOT
//     PxeMount,
// #endif
// #endif
// };
//
//
// /* DEBUGGING HELPERS **********************************************************/
//
// #if DBG && 0
// static VOID
// DumpDeviceList(VOID)
// {
//     PLIST_ENTRY pEntry;
//
//     DbgPrint("\n== Dumping ARC devices ==\n");
//
//     for (pEntry = DeviceListHead.Flink;
//          pEntry != &DeviceListHead;
//          pEntry = pEntry->Flink)
//     {
//         DEVICE* pDevice = CONTAINING_RECORD(pEntry, DEVICE, ListEntry);
//         ULONG DeviceId = pDevice->DeviceId;
//         const DEVVTBL* FuncTable;
//
//         DbgPrint("\n"
//                  "DeviceId %lu\n"
//                  "  RefCount: %lu\n"
//                  "  DeviceName  : '%s'\n", (//                  DeviceId) as usize, (pDevice->ReferenceCount) as usize, (pDevice->DeviceName) as usize);
//
//         FuncTable = pDevice->FuncTable;
//         DbgPrint("  DevFuncTable: 0x%p (In array: 0x%p) ; Name: '%S'\n", (//                  FuncTable) as usize, (//                  (DeviceId != INVALID_FILE_ID ? FileData[DeviceId].FuncTable : NULL)) as usize, (//                  (FuncTable && FuncTable->ServiceName)
//                     ? FuncTable->ServiceName : crate::wide!("n/a")) as usize);
//
//         FuncTable = pDevice->FileFuncTable;
//         DbgPrint("  FileFuncTable: 0x%p ; Name: '%S'\n", (//                  FuncTable) as usize, (//                  (FuncTable && FuncTable->ServiceName)
//                     ? FuncTable->ServiceName : crate::wide!("n/a")) as usize);
//     }
//
//     DbgPrint("\n== END Dumping ARC devices ==\n\n");
// }
//
// static VOID
// DumpFileTable(VOID)
// {
//     ULONG i;
//
//     DbgPrint("\n== Dumping ARC files table ==\n");
//
//     for (i = 0; i < _countof(FileData); ++i)
//     {
//         FILEDATA* pFileData = &FileData[i];
//         const DEVVTBL* FuncTable;
//
//         /* Show only occupied slots */
//         if (!pFileData->FuncTable)
//             continue;
//
//         DbgPrint("\n"
//                  "FileId %lu\n"
//                  "  RefCount: %lu\n"
//                  "  ParentDeviceId: %lu\n"
//                  /*"  FileName  : '%s'\n"*/
//                  "  Specific: 0x%p\n", (//                  i) as usize, (pFileData->ReferenceCount) as usize, (//                  pFileData->DeviceId) as usize, (pFileData->Specific) as usize);
//
//         FuncTable = pFileData->FuncTable;
//         DbgPrint("  FuncTable: 0x%p ; Name: '%S'\n", (//                  FuncTable) as usize, (//                  (FuncTable && FuncTable->ServiceName)
//                     ? FuncTable->ServiceName : crate::wide!("n/a")) as usize);
//     }
//
//     DbgPrint("\n== END Dumping ARC files table ==\n\n");
// }
// #endif // DBG
//
//
// /* ARC FUNCTIONS **************************************************************/
//
// /**
//  * @brief
//  * Replace "()" by "(0)" in the given ARC device name, if necessary.
//  *
//  * @param[in]   DeviceName
//  * The original ARC device name string to normalize.
//  *
//  * @param[in,out]   Length
//  * Points to a SIZE_T variable that:
//  * - on input, specifies the length of the ARC device name string pointed
//  *   by DeviceName;
//  * - on output, receives the length of the normalized ARC device name,
//  *   returned by value by this function.
//  *
//  * @return
//  * - On success, returns a string pointer to the normalized ARC device name.
//  *   If the device name string had to be normalized, this returned string is
//  *   allocated from the temporary heap, with TAG_DEVICE_NAME tag.
//  *   Otherwise, the function returns the original device name string pointer.
//  * - On failure, the function returns NULL.
//  **/
// PCHAR
// NormalizeArcDeviceName(
//     _In_ PCCH DeviceName,
//     _Inout_ PSIZE_T Length)
// {
//     ULONG Count;
//     SIZE_T NameLength;
//     PCCH p, End;
//     PCHAR q, NormName;
//
//     NameLength = *Length;
//     End = DeviceName + NameLength;
//
//     /* Count the number of "()", which needs to be replaced by "(0)" */
//     Count = 0;
//     for (p = DeviceName; p < End; ++p)
//     {
//         if ((p + 1) < End && *p == '(' && *(p + 1) == ')')
//             ++Count; //, ++p;
//     }
//
//     if (Count == 0) /* No need to duplicate the device name */
//         return (PCHAR)DeviceName;
//
//     /* Return the updated length */
//     *Length = NameLength + Count;
//
//     /* Duplicate the device name and replace "()" by "(0)" */
//     NormName = FrLdrTempAlloc(*Length, TAG_DEVICE_NAME);
//     if (!NormName)
//         return NULL;
//     for (p = DeviceName, q = NormName; p < End; ++p)
//     {
//         *q++ = *p;
//         if ((p + 1) < End && *p == '(' && *(p + 1) == ')')
//             *q++ = '0'; //, *q++ = *++p;
//     }
//     return NormName;
// }
//
// ARC_STATUS ArcOpen(CHAR* Path, OPENMODE OpenMode, ULONG* FileId)
// {
//     ARC_STATUS Status;
//     ULONG i;
//     PLIST_ENTRY pEntry;
//     DEVICE* pDevice;
//     PCHAR DeviceName;
//     PSTR FileName;
//     SIZE_T Length;
//     OPENMODE DeviceOpenMode;
//     ULONG DeviceId;
//
//     /* Print status message */
//     TRACE("Opening file '%s'...\n", (Path) as usize);
//
//     *FileId = INVALID_FILE_ID;
//
//     /* Search last ')', which delimits device and path */
//     FileName = strrchr(Path, ')');
//     if (!FileName)
//         return EINVAL;
//     ++FileName;
//
//     /* Normalize the device name, replacing "()" by "(0)" if necessary */
//     Length = FileName - Path;
//     DeviceName = NormalizeArcDeviceName(Path, &Length);
//     if (!DeviceName)
//         return ENOMEM;
//
//     if (OpenMode == OpenReadOnly || OpenMode == OpenWriteOnly)
//         DeviceOpenMode = OpenMode;
//     else
//         DeviceOpenMode = OpenReadWrite;
//
//     /* Search for the registered device */
//     pDevice = NULL;
//     for (pEntry = DeviceListHead.Flink;
//          pEntry != &DeviceListHead;
//          pEntry = pEntry->Flink)
//     {
//         pDevice = CONTAINING_RECORD(pEntry, DEVICE, ListEntry);
//         if ((strlen(pDevice->DeviceName) == Length) && (strncmp(pDevice->DeviceName, DeviceName, Length) == 0))
//             break;
//     }
//
//     /* Cleanup */
//     if (DeviceName != Path)
//         FrLdrTempFree(DeviceName, TAG_DEVICE_NAME);
//     DeviceName = NULL;
//
//     if (pEntry == &DeviceListHead)
//         return ENODEV;
//
//     /* OK, device found. Is it already opened? */
//     if (pDevice->ReferenceCount == 0)
//     {
//         /* Find some room for the device */
//         for (DeviceId = 0; ; ++DeviceId)
//         {
//             if (DeviceId >= _countof(FileData))
//                 return EMFILE;
//             if (!FileData[DeviceId].FuncTable)
//                 break;
//         }
//
//         /* Try to open the device */
//         FileData[DeviceId].ReferenceCount = 0;
//         FileData[DeviceId].FuncTable = pDevice->FuncTable;
//         Status = pDevice->FuncTable->Open(pDevice->DeviceName, DeviceOpenMode, &DeviceId);
//         if (Status != ESUCCESS)
//         {
//             FileData[DeviceId].FuncTable = NULL;
//             return Status;
//         }
//         pDevice->DeviceId = DeviceId;
//     }
//     else
//     {
//         /* Reuse the existing entry */
//         DeviceId = pDevice->DeviceId;
//         ASSERT(FileData[DeviceId].FuncTable == pDevice->FuncTable);
//     }
//
//     /* Done, increase the device reference count */
//     pDevice->ReferenceCount++;
//     FileData[DeviceId].ReferenceCount++;
//
//     if (!*FileName)
//     {
//         /* The caller wanted to open the raw device: return its ID */
//         *FileId = DeviceId;
//         return ESUCCESS;
//     }
//
//
//     /*
//      * We are opening a file.
//      */
//
//     /* Find some room for the file */
//     for (i = 0; ; ++i)
//     {
//         if (i >= _countof(FileData))
//         {
//             Status = EMFILE;
//             goto Done;
//         }
//         if (!FileData[i].FuncTable)
//             break;
//     }
//
//     /* The device is accessed using filesystem semantics.
//      * Try to detect the file system if not already done. */
//     if (!pDevice->FileFuncTable && (pDevice->ReferenceCount <= 1))
//     {
//         for (ULONG fs = 0; fs < _countof(FileSystems); ++fs)
//         {
//             pDevice->FileFuncTable = FileSystems[fs](DeviceId);
//             if (pDevice->FileFuncTable)
//                 break;
//         }
//     }
//     if (!pDevice->FileFuncTable)
//     {
//         /* Error, unable to detect the file system */
//         Status = ENOENT; // ENXIO;
//         goto Done;
//     }
//
//     /*
//      * At this point, the device is found and opened. Its file ID is stored
//      * in DeviceId, and pDevice->FileFuncTable contains what needs to be
//      * called to manipulate the file.
//      */
//
//     /* Open the file */
//     FileData[i].DeviceId = DeviceId;
//     FileData[i].ReferenceCount = 0;
//     FileData[i].FuncTable = pDevice->FileFuncTable;
//     *FileId = i;
//     Status = FileData[i].FuncTable->Open(FileName, OpenMode, FileId);
//     if (Status != ESUCCESS)
//     {
//         FileData[i].DeviceId = INVALID_FILE_ID;
//         FileData[i].FuncTable = NULL;
//         FileData[i].Specific = NULL;
//         *FileId = INVALID_FILE_ID;
//     }
//     else
//     {
//         /* Reference the file */
//         FileData[i].ReferenceCount++;
//     }
//
// Done:
//     /* If we failed somewhere, dereference the device as well */
//     if (Status != ESUCCESS)
//     {
//         // ArcClose(DeviceId);
//         if (--FileData[DeviceId].ReferenceCount == 0)
//         {
//             (void)FileData[DeviceId].FuncTable->Close(DeviceId);
//             FileData[DeviceId].DeviceId = INVALID_FILE_ID;
//             FileData[DeviceId].FuncTable = NULL;
//             FileData[DeviceId].Specific = NULL;
//         }
//         if (--pDevice->ReferenceCount == 0)
//             pDevice->DeviceId = INVALID_FILE_ID;
//     }
//
//     return Status;
// }
//
// static DEVICE*
// FsGetDeviceById(ULONG DeviceId)
// {
//     PLIST_ENTRY pEntry;
//
//     for (pEntry = DeviceListHead.Flink;
//          pEntry != &DeviceListHead;
//          pEntry = pEntry->Flink)
//     {
//         DEVICE* pDevice = CONTAINING_RECORD(pEntry, DEVICE, ListEntry);
//         if (pDevice->DeviceId == DeviceId)
//             return pDevice;
//     }
//     return NULL;
// }
//
// ARC_STATUS
// ArcClose(
//     _In_ ULONG FileId)
// {
//     ULONG DeviceId;
//     DEVICE* pDevice;
//
//     if (!IS_VALID_FILEID(FileId))
//         return EBADF;
//
//     /* Retrieve the parent device's ID if any, for later */
//     DeviceId = FileData[FileId].DeviceId;
//
//     /* Dereference the file and close it if needed */
//     ASSERT(FileData[FileId].ReferenceCount > 0);
//     if (--FileData[FileId].ReferenceCount == 0)
//     {
//         (void)FileData[FileId].FuncTable->Close(FileId);
//         FileData[FileId].DeviceId = INVALID_FILE_ID;
//         FileData[FileId].FuncTable = NULL;
//         FileData[FileId].Specific = NULL;
//     }
//
//     /* Check whether this file actually references a device */
//     pDevice = FsGetDeviceById(FileId);
//     if (pDevice)
//     {
//         /* It does, dereference it */
//         ASSERT(pDevice->ReferenceCount > 0);
//         if (--pDevice->ReferenceCount == 0)
//             pDevice->DeviceId = INVALID_FILE_ID;
//     }
//
//     /* And dereference the parent device too, if there is one */
//     if (IS_VALID_FILEID(DeviceId))
//         ArcClose(DeviceId);
//
//     return ESUCCESS;
// }
//
// ARC_STATUS ArcRead(ULONG FileId, VOID* Buffer, ULONG N, ULONG* Count)
// {
//     if (!IS_VALID_FILEID(FileId))
//         return EBADF;
//     return FileData[FileId].FuncTable->Read(FileId, Buffer, N, Count);
// }
//
// ARC_STATUS ArcSeek(ULONG FileId, LARGE_INTEGER* Position, SEEKMODE SeekMode)
// {
//     if (!IS_VALID_FILEID(FileId))
//         return EBADF;
//     return FileData[FileId].FuncTable->Seek(FileId, Position, SeekMode);
// }
//
// ARC_STATUS ArcGetFileInformation(ULONG FileId, FILEINFORMATION* Information)
// {
//     if (!IS_VALID_FILEID(FileId))
//         return EBADF;
//     return FileData[FileId].FuncTable->GetFileInformation(FileId, Information);
// }
//
// /* FUNCTIONS ******************************************************************/
//
// VOID FileSystemError(PCSTR ErrorString)
// {
//     ERR("%s\n", (ErrorString) as usize);
//     UiMessageBox(ErrorString);
// }
//
// ARC_STATUS
// FsOpenFile(
//     IN PCSTR FileName,
//     IN PCSTR DefaultPath OPTIONAL,
//     IN OPENMODE OpenMode,
//     OUT PULONG FileId)
// {
//     NTSTATUS Status;
//     SIZE_T cchPathLen;
//     CHAR FullPath[MAX_PATH] = "";
//
//     /*
//      * Check whether FileName is a full path and if not, create a full
//      * file name using the user-provided default path (if present).
//      *
//      * See ArcOpen(): Search last ')', which delimits device and path.
//      */
//     if (strrchr(FileName, ')') == NULL)
//     {
//         /* This is not a full path: prepend the user-provided default path */
//         if (DefaultPath)
//         {
//             Status = RtlStringCbCopyA(FullPath, sizeof(FullPath), DefaultPath);
//             if (!NT_SUCCESS(Status))
//                 return ENAMETOOLONG;
//         }
//
//         /* Append a path separator if needed */
//
//         cchPathLen = min(sizeof(FullPath)/sizeof(CHAR), strlen(FullPath));
//         if (cchPathLen >= sizeof(FullPath)/sizeof(CHAR))
//             return ENAMETOOLONG;
//
//         if ((*FileName != '\\' && *FileName != '/') &&
//             cchPathLen > 0 && (FullPath[cchPathLen-1] != '\\' && FullPath[cchPathLen-1] != '/'))
//         {
//             /* FileName does not start with '\' and FullPath does not end with '\' */
//             Status = RtlStringCbCatA(FullPath, sizeof(FullPath), "\\");
//             if (!NT_SUCCESS(Status))
//                 return ENAMETOOLONG;
//         }
//         else if ((*FileName == '\\' || *FileName == '/') &&
//                  cchPathLen > 0 && (FullPath[cchPathLen-1] == '\\' || FullPath[cchPathLen-1] == '/'))
//         {
//             /* FileName starts with '\' and FullPath ends with '\' */
//             while (*FileName == '\\' || *FileName == '/')
//                 ++FileName; // Skip any backslash
//         }
//     }
//     /* Append (or just copy) the remaining file name */
//     Status = RtlStringCbCatA(FullPath, sizeof(FullPath), FileName);
//     if (!NT_SUCCESS(Status))
//         return ENAMETOOLONG;
//
//     /* Open the file */
//     return ArcOpen(FullPath, OpenMode, FileId);
// }
//
// /**
//  * @brief
//  * Returns the (useful) size of a file-system volume laid on the
//  * storage media device opened via @p DeviceId.
//  **/
// ARC_STATUS
// FsGetVolumeSize(
//     _In_ ULONG DeviceId,
//     _Out_ PULONGLONG VolumeSize)
// {
//     DEVICE* pDevice;
//     const DEVVTBL* FuncTable;
//     PCWSTR ServiceName;
//     ULONGLONG (*pFsGetVolumeSize)(_In_ ULONG DeviceId);
//
//     /* Check whether this file actually references a device */
//     pDevice = FsGetDeviceById(DeviceId);
//     if (!pDevice)
//         return EBADF; /* It doesn't, fail */
//
//     *VolumeSize = 0;
//
//     // TODO: Mount the device, if not already done?
// #if 1 // FIXME: TEMP HACK!
//     if (!pDevice->FileFuncTable && (pDevice->ReferenceCount <= 1))
//     {
//         for (ULONG fs = 0; fs < _countof(FileSystems); ++fs)
//         {
//             pDevice->FileFuncTable = FileSystems[fs](DeviceId);
//             if (pDevice->FileFuncTable)
//                 break;
//         }
//     }
// #endif
//     // TODO: Use a virtual table function member for easily handling other file systems.
//
//     /* Check that the device is mounted (non-NULL FileFuncTable)
//      * and there is an associated "service" (file-system) name. */
//     FuncTable = pDevice->FileFuncTable;
//     ServiceName = (FuncTable && FuncTable->ServiceName)
//                     ? FuncTable->ServiceName : NULL;
//     if (!ServiceName)
//     {
//         ERR("Non-mounted device %lu\n", (DeviceId) as usize);
//         return ENXIO;
//     }
//
//     /* HACK: Since we don't currently use a virtual table function member,
//      * we instead do a manual lookup over the file-system name.
//      * The ordering follows that of the FileSystems[] table. */
// #ifndef _M_ARM
//     if (!_wcsicmp(ServiceName, crate::wide!("cdfs")))
//         pFsGetVolumeSize = IsoGetVolumeSize;
//     else
// #endif
//     if (!_wcsicmp(ServiceName, crate::wide!("fastfat")) || !_wcsicmp(ServiceName, crate::wide!("vfatfs")))
//         pFsGetVolumeSize = FatGetVolumeSize;
//     else
//     if (!_wcsicmp(ServiceName, crate::wide!("btrfs")))
//         pFsGetVolumeSize = BtrFsGetVolumeSize;
//     else
// #ifndef _M_ARM
//     if (!_wcsicmp(ServiceName, crate::wide!("ntfs")))
//         pFsGetVolumeSize = NtfsGetVolumeSize;
//     else
//     if (!_wcsicmp(ServiceName, crate::wide!("ext2fs")))
//         pFsGetVolumeSize = ExtGetVolumeSize;
//     else
// #endif
//     {
//         ERR("Unsupported file system '%S' on storage device %lu\n", (ServiceName) as usize, (DeviceId) as usize);
//         return EIO; // ENXIO;
//     }
//     *VolumeSize = pFsGetVolumeSize(DeviceId);
//     return ESUCCESS;
// }
//
// /*
//  * FsGetNumPathParts()
//  * This function parses a path in the form of dir1\dir2\file1.ext
//  * and returns the number of parts it has (i.e. 3 - dir1,dir2,file1.ext).
//  */
// ULONG FsGetNumPathParts(PCSTR Path)
// {
//     size_t i;
//     size_t len;
//     ULONG  num;
//
//     len = strlen(Path);
//
//     for (i = 0, num = 0; i < len; i++)
//     {
//         if ((Path[i] == '\\') || (Path[i] == '/'))
//         {
//             num++;
//         }
//     }
//     num++;
//
//     TRACE("FsGetNumPathParts() Path = %s NumPathParts = %d\n", (Path) as usize, (num) as usize);
//
//     return num;
// }
//
// /*
//  * FsGetFirstNameFromPath()
//  * This function parses a path in the form of dir1\dir2\file1.ext
//  * and puts the first name of the path (e.g. "dir1") in buffer
//  * compatible with the MSDOS directory structure.
//  */
// VOID FsGetFirstNameFromPath(PCHAR Buffer, PCSTR Path)
// {
//     size_t i;
//     size_t len;
//
//     len = strlen(Path);
//
//     // Copy all the characters up to the end of the
//     // string or until we hit a '\' character
//     // and put them in Buffer
//     for (i = 0; i < len; i++)
//     {
//         if ((Path[i] == '\\') || (Path[i] == '/'))
//         {
//             break;
//         }
//         else
//         {
//             Buffer[i] = Path[i];
//         }
//     }
//
//     Buffer[i] = 0;
//
//     TRACE("FsGetFirstNameFromPath() Path = %s FirstName = %s\n", (Path) as usize, (Buffer) as usize);
// }
//
// VOID
// FsRegisterDevice(
//     _In_ PCSTR DeviceName,
//     _In_ const DEVVTBL* FuncTable)
// {
//     DEVICE* pNewEntry;
//     SIZE_T Length;
//
//     TRACE("FsRegisterDevice(%s)\n", (DeviceName) as usize);
//
//     Length = strlen(DeviceName) + 1;
//     pNewEntry = FrLdrTempAlloc(sizeof(DEVICE) + Length, TAG_DEVICE);
//     if (!pNewEntry)
//         return;
//     pNewEntry->FuncTable = FuncTable;
//     pNewEntry->DeviceId = INVALID_FILE_ID;
//     pNewEntry->ReferenceCount = 0;
//     pNewEntry->DeviceName = (PSTR)(pNewEntry + 1);
//     RtlCopyMemory(pNewEntry->DeviceName, DeviceName, Length);
//
//     InsertHeadList(&DeviceListHead, &pNewEntry->ListEntry);
// }
//
// PCWSTR FsGetServiceName(ULONG FileId)
// {
//     if (!IS_VALID_FILEID(FileId))
//         return NULL;
//     return FileData[FileId].FuncTable->ServiceName;
// }
//
// VOID FsSetDeviceSpecific(ULONG FileId, PVOID Specific)
// {
//     if (!IS_VALID_FILEID(FileId))
//         return;
//     FileData[FileId].Specific = Specific;
// }
//
// PVOID FsGetDeviceSpecific(ULONG FileId)
// {
//     if (!IS_VALID_FILEID(FileId))
//         return NULL;
//     return FileData[FileId].Specific;
// }
//
// ULONG FsGetDeviceId(ULONG FileId)
// {
//     if (FileId >= _countof(FileData)) // !IS_VALID_FILEID(FileId)
//         return INVALID_FILE_ID;
//     return FileData[FileId].DeviceId;
// }
//
// VOID FsInit(VOID)
// {
//     ULONG i;
//
//     RtlZeroMemory(FileData, sizeof(FileData));
//     for (i = 0; i < _countof(FileData); ++i)
//         FileData[i].DeviceId = INVALID_FILE_ID;
//
//     InitializeListHead(&DeviceListHead);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

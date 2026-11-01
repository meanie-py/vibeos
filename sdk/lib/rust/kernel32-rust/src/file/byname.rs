//! GetFileInformationByName (Windows 11 24H2): stat a file by path without the caller holding
//! a handle. Windows answers it with NtQueryInformationByName, which our file systems do not
//! implement (FileStatInformation & co. are NTFS-era classes), so this opens the file for
//! attributes only and assembles the stat records from the classes every file system here
//! answers. CPython 3.12+ takes its os.stat() fast path through this and falls back on the
//! error codes, so those follow Windows exactly.
//!
//! Measured on Windows 11 (winref 20261031-223113-3h38le85): class 0 FileStatByNameInfo (72
//! bytes), 1 FileStatLxByNameInfo (96), 2 FileCaseSensitiveByNameInfo (4), 3
//! FileStatBasicByNameInfo (104); a class >= 4 is ERROR_INVALID_PARAMETER even for a missing
//! file, so it is checked first; a buffer shorter than the record is ERROR_BAD_LENGTH; a
//! missing file ERROR_FILE_NOT_FOUND. A reparse point is described, not followed.
use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::*;
use crate::string::nls_subst::NT_SUCCESS;
use core::ffi::c_void;
use core::mem::{size_of, zeroed};

const ERROR_INVALID_PARAMETER_: u32 = 87;
const ERROR_BAD_LENGTH_: u32 = 24;

const FILE_STAT_BY_NAME_INFO: i32 = 0;
const FILE_STAT_LX_BY_NAME_INFO: i32 = 1;
const FILE_CASE_SENSITIVE_BY_NAME_INFO: i32 = 2;
const FILE_STAT_BASIC_BY_NAME_INFO: i32 = 3;

/// FILE_INFORMATION_CLASS FileCaseSensitiveInformation.
const FILE_CASE_SENSITIVE_INFORMATION_CLASS: i32 = 71;

#[repr(C)]
struct FileStatInformation {
    FileId: i64,
    CreationTime: i64,
    LastAccessTime: i64,
    LastWriteTime: i64,
    ChangeTime: i64,
    AllocationSize: i64,
    EndOfFile: i64,
    FileAttributes: u32,
    ReparseTag: u32,
    NumberOfLinks: u32,
    EffectiveAccess: u32,
}

#[repr(C)]
struct FileStatLxInformation {
    Stat: FileStatInformation,
    LxFlags: u32,
    LxUid: u32,
    LxGid: u32,
    LxMode: u32,
    LxDeviceIdMajor: u32,
    LxDeviceIdMinor: u32,
}

#[repr(C)]
struct FileStatBasicInformation {
    FileId: i64,
    CreationTime: i64,
    LastAccessTime: i64,
    LastWriteTime: i64,
    ChangeTime: i64,
    AllocationSize: i64,
    EndOfFile: i64,
    FileAttributes: u32,
    ReparseTag: u32,
    NumberOfLinks: u32,
    DeviceType: u32,
    DeviceCharacteristics: u32,
    Reserved: u32,
    VolumeSerialNumber: i64,
    FileId128: [u8; 16],
}

const _: () = assert!(size_of::<FileStatInformation>() == 72);
const _: () = assert!(size_of::<FileStatLxInformation>() == 96);
const _: () = assert!(size_of::<FileStatBasicInformation>() == 104);

/// FILE_FS_VOLUME_INFORMATION with room for its trailing label: the header alone is too
/// short for any file system that has one, and a short buffer is a stack overrun.
#[repr(C)]
struct VolumeInfo {
    Volume: _FILE_FS_VOLUME_INFORMATION,
    Label: [u16; 64],
}

unsafe fn query<T>(h: HANDLE, class: _FILE_INFORMATION_CLASS, out: &mut T) -> NTSTATUS {
    let mut iosb: _IO_STATUS_BLOCK = zeroed();
    NtQueryInformationFile(h, &mut iosb, out as *mut T as PVOID, size_of::<T>() as ULONG, class)
}

/// Open `path` the way NtQueryInformationByName does: for attributes, sharing everything,
/// not following a reparse point. The handle is opened MAXIMUM_ALLOWED first so its granted
/// access can stand for EffectiveAccess; a refusal of that (sharing, or rights the caller
/// lacks) falls back to the attributes-only open Windows itself needs, whose access then is
/// all the caller has been shown to hold.
unsafe fn open_for_stat(path: LPCWSTR, handle: &mut HANDLE, granted: &mut u32) -> NTSTATUS {
    let mut nt_path: _UNICODE_STRING = zeroed();
    let mut relative: _RTL_RELATIVE_NAME_U = zeroed();
    if RtlDosPathNameToNtPathName_U(path, &mut nt_path, core::ptr::null_mut(), &mut relative) == 0 {
        return STATUS_OBJECT_NAME_INVALID;
    }
    let mut oa: _OBJECT_ATTRIBUTES = zeroed();
    oa.Length = size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    oa.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    if relative.RelativeName.Length != 0 {
        oa.ObjectName = &mut relative.RelativeName as *mut _ as PUNICODE_STRING;
        oa.RootDirectory = relative.ContainingDirectory;
    } else {
        oa.ObjectName = &mut nt_path as *mut _ as PUNICODE_STRING;
    }
    let share = (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG;
    let options = (FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT) as ULONG;
    let mut iosb: _IO_STATUS_BLOCK = zeroed();
    let mut status = NtOpenFile(handle, MAXIMUM_ALLOWED as ACCESS_MASK, &mut oa, &mut iosb, share, options);
    if NT_SUCCESS(status) {
        let mut basic: _OBJECT_BASIC_INFORMATION = zeroed();
        let mut len: ULONG = 0;
        *granted = if NT_SUCCESS(NtQueryObject(
            *handle,
            ObjectBasicInformation,
            &mut basic as *mut _ as PVOID,
            size_of::<_OBJECT_BASIC_INFORMATION>() as ULONG,
            &mut len,
        )) {
            basic.GrantedAccess
        } else {
            0
        };
    } else if status == STATUS_SHARING_VIOLATION || status == STATUS_ACCESS_DENIED {
        status = NtOpenFile(handle, (FILE_READ_ATTRIBUTES | SYNCHRONIZE) as ACCESS_MASK, &mut oa, &mut iosb, share, options);
        *granted = (FILE_READ_ATTRIBUTES | SYNCHRONIZE) as u32;
    }
    RtlReleaseRelativeName(&mut relative);
    RtlFreeHeap(RtlGetProcessHeap(), 0, nt_path.Buffer as PVOID);
    status
}

/// The fields every stat record shares, from the classes our file systems answer. Basic and
/// standard are required; the file id and reparse tag are not answered by FAT or CDFS (they
/// fail FileAttributeTagInformation with STATUS_INVALID_PARAMETER, as Windows' FAT does) and
/// read 0 there, which is what a file with no id and no reparse point would report.
unsafe fn stat_common(h: HANDLE, granted: u32) -> Result<FileStatInformation, NTSTATUS> {
    let mut basic: _FILE_BASIC_INFORMATION = zeroed();
    let s = query(h, FileBasicInformation, &mut basic);
    if !NT_SUCCESS(s) {
        return Err(s);
    }
    let mut standard: _FILE_STANDARD_INFORMATION = zeroed();
    let s = query(h, FileStandardInformation, &mut standard);
    if !NT_SUCCESS(s) {
        return Err(s);
    }
    let mut internal: _FILE_INTERNAL_INFORMATION = zeroed();
    let file_id = if NT_SUCCESS(query(h, FileInternalInformation, &mut internal)) {
        internal.IndexNumber.QuadPart
    } else {
        0
    };
    let mut tag: _FILE_ATTRIBUTE_TAG_INFORMATION = zeroed();
    let reparse_tag = if NT_SUCCESS(query(h, FileAttributeTagInformation, &mut tag)) {
        tag.ReparseTag
    } else {
        0
    };
    Ok(FileStatInformation {
        FileId: file_id,
        CreationTime: basic.CreationTime.QuadPart,
        LastAccessTime: basic.LastAccessTime.QuadPart,
        LastWriteTime: basic.LastWriteTime.QuadPart,
        ChangeTime: basic.ChangeTime.QuadPart,
        AllocationSize: standard.AllocationSize.QuadPart,
        EndOfFile: standard.EndOfFile.QuadPart,
        FileAttributes: basic.FileAttributes,
        ReparseTag: reparse_tag,
        NumberOfLinks: standard.NumberOfLinks,
        EffectiveAccess: granted,
    })
}

unsafe fn fill(h: HANDLE, class: i32, buffer: PVOID, granted: u32) -> NTSTATUS {
    if class == FILE_CASE_SENSITIVE_BY_NAME_INFO {
        let mut iosb: _IO_STATUS_BLOCK = zeroed();
        return NtQueryInformationFile(h, &mut iosb, buffer, size_of::<u32>() as ULONG, FILE_CASE_SENSITIVE_INFORMATION_CLASS as _);
    }
    let stat = match stat_common(h, granted) {
        Ok(s) => s,
        Err(s) => return s,
    };
    match class {
        FILE_STAT_BY_NAME_INFO => *(buffer as *mut FileStatInformation) = stat,
        FILE_STAT_LX_BY_NAME_INFO => {
            *(buffer as *mut FileStatLxInformation) = FileStatLxInformation {
                Stat: stat,
                LxFlags: 0,
                LxUid: 0,
                LxGid: 0,
                LxMode: 0,
                LxDeviceIdMajor: 0,
                LxDeviceIdMinor: 0,
            }
        }
        _ => {
            let mut iosb: _IO_STATUS_BLOCK = zeroed();
            let mut device: _FILE_FS_DEVICE_INFORMATION = zeroed();
            let s = NtQueryVolumeInformationFile(h, &mut iosb, &mut device as *mut _ as PVOID, size_of::<_FILE_FS_DEVICE_INFORMATION>() as ULONG, FileFsDeviceInformation);
            if !NT_SUCCESS(s) {
                return s;
            }
            let mut volume: VolumeInfo = zeroed();
            // A label longer than the buffer is STATUS_BUFFER_OVERFLOW with the fixed part,
            // serial included, filled.
            let s = NtQueryVolumeInformationFile(h, &mut iosb, &mut volume as *mut _ as PVOID, size_of::<VolumeInfo>() as ULONG, FileFsVolumeInformation);
            let serial = if NT_SUCCESS(s) || s == STATUS_BUFFER_OVERFLOW {
                volume.Volume.VolumeSerialNumber as i64
            } else {
                0
            };
            let mut id128 = [0u8; 16];
            id128[..8].copy_from_slice(&stat.FileId.to_le_bytes());
            *(buffer as *mut FileStatBasicInformation) = FileStatBasicInformation {
                FileId: stat.FileId,
                CreationTime: stat.CreationTime,
                LastAccessTime: stat.LastAccessTime,
                LastWriteTime: stat.LastWriteTime,
                ChangeTime: stat.ChangeTime,
                AllocationSize: stat.AllocationSize,
                EndOfFile: stat.EndOfFile,
                FileAttributes: stat.FileAttributes,
                ReparseTag: stat.ReparseTag,
                NumberOfLinks: stat.NumberOfLinks,
                DeviceType: device.DeviceType as u32,
                DeviceCharacteristics: device.Characteristics as u32,
                Reserved: 0,
                VolumeSerialNumber: serial,
                FileId128: id128,
            };
        }
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn GetFileInformationByName(
    FileName: LPCWSTR,
    FileInformationClass: i32,
    FileInfoBuffer: PVOID,
    FileInfoBufferSize: ULONG,
) -> BOOL {
    let need = match FileInformationClass {
        FILE_STAT_BY_NAME_INFO => size_of::<FileStatInformation>(),
        FILE_STAT_LX_BY_NAME_INFO => size_of::<FileStatLxInformation>(),
        FILE_CASE_SENSITIVE_BY_NAME_INFO => size_of::<u32>(),
        FILE_STAT_BASIC_BY_NAME_INFO => size_of::<FileStatBasicInformation>(),
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER_);
            return 0;
        }
    };
    if FileInfoBuffer.is_null() || (FileInfoBufferSize as usize) < need {
        SetLastError(ERROR_BAD_LENGTH_);
        return 0;
    }
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut granted = 0u32;
    let mut status = open_for_stat(FileName, &mut handle, &mut granted);
    if NT_SUCCESS(status) {
        status = fill(handle, FileInformationClass, FileInfoBuffer as *mut c_void, granted);
        NtClose(handle);
    }
    if !NT_SUCCESS(status) {
        BaseSetLastNTError(status);
        return 0;
    }
    1
}

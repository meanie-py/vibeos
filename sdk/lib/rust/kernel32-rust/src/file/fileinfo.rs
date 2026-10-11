use winedebug_rust::{TRACE, WARN};
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 200 as ULONG;

// kernel32-internal helpers (defined in the OTHER kernel32 .c files / sibling
// modules that stay/already translated) + cross-file Win32 exports this file calls.
use crate::k32_externs::*;
use crate::string::nls_subst::NT_SUCCESS;

// DEBUG_CHANNEL TRACE/WARN are no-ops here (NDEBUG), matching wine/res.rs.

// winbase.h sentinels not in the ntdll FFI surface.
pub const INVALID_SET_FILE_POINTER: DWORD = 0xFFFFFFFF;
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF;

extern "C" {
    fn TranslateStdHandle(hHandle: HANDLE) -> HANDLE;
    // FilenameA2W is cdecl in C (fileutils.c / kernel32.h: no WINAPI).
    fn FilenameA2W(NameA: LPCSTR, alloc: BOOL) -> PWCHAR;
}
extern "system" {
    fn FlushConsoleInputBuffer(hConsoleInput: HANDLE) -> BOOL;
    fn VerifyConsoleIoHandle(Handle: HANDLE) -> BOOL;
}

// value-type aliases (the underlying `_FILE_*` structs come from ntdll_sys::nt).
pub type FILE_BASIC_INFORMATION = _FILE_BASIC_INFORMATION;
pub type FILE_STANDARD_INFORMATION = _FILE_STANDARD_INFORMATION;
pub type FILE_INTERNAL_INFORMATION = _FILE_INTERNAL_INFORMATION;
pub type FILE_POSITION_INFORMATION = _FILE_POSITION_INFORMATION;
pub type FILE_END_OF_FILE_INFORMATION = _FILE_END_OF_FILE_INFORMATION;
pub type FILE_ALLOCATION_INFORMATION = _FILE_ALLOCATION_INFORMATION;
pub type FILE_VALID_DATA_LENGTH_INFORMATION = _FILE_VALID_DATA_LENGTH_INFORMATION;
pub type FILE_COMPRESSION_INFORMATION = _FILE_COMPRESSION_INFORMATION;
pub type FILE_NETWORK_OPEN_INFORMATION = _FILE_NETWORK_OPEN_INFORMATION;
pub type FILE_FS_VOLUME_INFORMATION = _FILE_FS_VOLUME_INFORMATION;
pub type FILE_FS_DEVICE_INFORMATION = _FILE_FS_DEVICE_INFORMATION;

#[no_mangle]
pub unsafe extern "system" fn FlushFileBuffers(mut hFile: HANDLE) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    hFile = TranslateStdHandle(hFile);

    if IsConsoleHandle(hFile) {
        return FlushConsoleInputBuffer(hFile);
    }

    Status = NtFlushBuffersFile(hFile, &mut IoStatusBlock);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0; // FALSE
    }
    return 1; // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SetFilePointer(
    hFile: HANDLE,
    lDistanceToMove: LONG,
    lpDistanceToMoveHigh: PLONG,
    dwMoveMethod: DWORD,
) -> DWORD {
    let mut FilePosition: FILE_POSITION_INFORMATION = core::mem::zeroed();
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut errCode: NTSTATUS = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Distance: LARGE_INTEGER = core::mem::zeroed();

    TRACE!(
        b"SetFilePointer(hFile %p, lDistanceToMove %d, dwMoveMethod %lu)\n\0".as_ptr() as *const i8,
        hFile,
        lDistanceToMove,
        dwMoveMethod,
    );

    if IsConsoleHandle(hFile) {
        SetLastError(ERROR_INVALID_HANDLE);
        return INVALID_SET_FILE_POINTER;
    }

    if !lpDistanceToMoveHigh.is_null() {
        Distance.u.HighPart = *lpDistanceToMoveHigh;
        Distance.u.LowPart = lDistanceToMove as DWORD;
    } else {
        Distance.QuadPart = lDistanceToMove as LONGLONG;
    }

    match dwMoveMethod {
        FILE_CURRENT => {
            errCode = NtQueryInformationFile(
                hFile,
                &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
                &mut FilePosition as *mut FILE_POSITION_INFORMATION as PVOID,
                core::mem::size_of::<FILE_POSITION_INFORMATION>() as ULONG,
                FilePositionInformation,
            );
            FilePosition.CurrentByteOffset.QuadPart += Distance.QuadPart;
            if !NT_SUCCESS(errCode) {
                if !lpDistanceToMoveHigh.is_null() {
                    *lpDistanceToMoveHigh = -1;
                }
                BaseSetLastNTError(errCode);
                return INVALID_SET_FILE_POINTER;
            }
        }
        FILE_END => {
            errCode = NtQueryInformationFile(
                hFile,
                &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
                &mut FileStandard as *mut FILE_STANDARD_INFORMATION as PVOID,
                core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
                FileStandardInformation,
            );
            FilePosition.CurrentByteOffset.QuadPart =
                FileStandard.EndOfFile.QuadPart + Distance.QuadPart;
            if !NT_SUCCESS(errCode) {
                if !lpDistanceToMoveHigh.is_null() {
                    *lpDistanceToMoveHigh = -1;
                }
                BaseSetLastNTError(errCode);
                return INVALID_SET_FILE_POINTER;
            }
        }
        FILE_BEGIN => {
            FilePosition.CurrentByteOffset.QuadPart = Distance.QuadPart;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return INVALID_SET_FILE_POINTER;
        }
    }

    if FilePosition.CurrentByteOffset.QuadPart < 0 {
        SetLastError(ERROR_NEGATIVE_SEEK);
        return INVALID_SET_FILE_POINTER;
    }

    if lpDistanceToMoveHigh.is_null() && FilePosition.CurrentByteOffset.u.HighPart != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_SET_FILE_POINTER;
    }

    errCode = NtSetInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FilePosition as *mut FILE_POSITION_INFORMATION as PVOID,
        core::mem::size_of::<FILE_POSITION_INFORMATION>() as ULONG,
        FilePositionInformation,
    );
    if !NT_SUCCESS(errCode) {
        if !lpDistanceToMoveHigh.is_null() {
            *lpDistanceToMoveHigh = -1;
        }
        BaseSetLastNTError(errCode);
        return INVALID_SET_FILE_POINTER;
    }

    if !lpDistanceToMoveHigh.is_null() {
        *lpDistanceToMoveHigh = FilePosition.CurrentByteOffset.u.HighPart;
    }

    if FilePosition.CurrentByteOffset.u.LowPart == MAXDWORD {
        SetLastError(ERROR_SUCCESS);
    }

    FilePosition.CurrentByteOffset.u.LowPart
}

#[no_mangle]
pub unsafe extern "system" fn SetFilePointerEx(
    hFile: HANDLE,
    liDistanceToMove: LARGE_INTEGER,
    lpNewFilePointer: PLARGE_INTEGER,
    dwMoveMethod: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FilePosition: FILE_POSITION_INFORMATION = core::mem::zeroed();
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();

    if IsConsoleHandle(hFile) {
        BaseSetLastNTError(STATUS_INVALID_HANDLE);
        return 0;
    }

    match dwMoveMethod {
        FILE_CURRENT => {
            Status = NtQueryInformationFile(
                hFile,
                &mut IoStatusBlock,
                &mut FilePosition as *mut FILE_POSITION_INFORMATION as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_POSITION_INFORMATION>() as ULONG,
                FilePositionInformation,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                return 0;
            }
            FilePosition.CurrentByteOffset.QuadPart += liDistanceToMove.QuadPart;
        }
        FILE_END => {
            Status = NtQueryInformationFile(
                hFile,
                &mut IoStatusBlock,
                &mut FileStandard as *mut FILE_STANDARD_INFORMATION as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
                FileStandardInformation,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                return 0;
            }
            FilePosition.CurrentByteOffset.QuadPart =
                FileStandard.EndOfFile.QuadPart + liDistanceToMove.QuadPart;
        }
        FILE_BEGIN => {
            FilePosition.CurrentByteOffset.QuadPart = liDistanceToMove.QuadPart;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    if FilePosition.CurrentByteOffset.QuadPart < 0 {
        SetLastError(ERROR_NEGATIVE_SEEK);
        return 0;
    }

    Status = NtSetInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FilePosition as *mut FILE_POSITION_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_POSITION_INFORMATION>() as ULONG,
        FilePositionInformation,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if !lpNewFilePointer.is_null() {
        *lpNewFilePointer = FilePosition.CurrentByteOffset;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileType(mut hFile: HANDLE) -> DWORD {
    let mut DeviceInfo: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();
    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    hFile = TranslateStdHandle(hFile);

    if IsConsoleHandle(hFile) {
        if VerifyConsoleIoHandle(hFile) != 0 {
            return FILE_TYPE_CHAR as DWORD;
        }
    }

    Status = NtQueryVolumeInformationFile(
        hFile,
        &mut StatusBlock as *mut _IO_STATUS_BLOCK,
        &mut DeviceInfo as *mut FILE_FS_DEVICE_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
        FileFsDeviceInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FILE_TYPE_UNKNOWN as DWORD;
    }

    match DeviceInfo.DeviceType {
        2 | 3 | 4 | 5 | 6 | 7 | 8 | 36 => return FILE_TYPE_DISK as DWORD,
        11 | 15 | 21 | 22 | 24 | 27 | 28 | 29 | 43 => return FILE_TYPE_CHAR as DWORD,
        17 => return FILE_TYPE_PIPE as DWORD,
        _ => {}
    }

    return FILE_TYPE_UNKNOWN as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileSize(
    hFile: HANDLE,
    lpFileSizeHigh: LPDWORD,
) -> DWORD {
    let mut errCode: NTSTATUS;
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    errCode = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileStandard as *mut FILE_STANDARD_INFORMATION as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        FileStandardInformation,
    );
    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        if lpFileSizeHigh.is_null() {
            return 0xFFFFFFFF; // -1 as DWORD
        } else {
            return 0;
        }
    }
    if !lpFileSizeHigh.is_null() {
        *lpFileSizeHigh = FileStandard.EndOfFile.u.HighPart as DWORD;
    }
    return FileStandard.EndOfFile.u.LowPart;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileSizeEx(
    hFile: HANDLE,
    lpFileSize: PLARGE_INTEGER,
) -> BOOL {
    let mut errCode: NTSTATUS;
    let mut FileStandard: FILE_STANDARD_INFORMATION;
    let mut IoStatusBlock: _IO_STATUS_BLOCK;

    FileStandard = core::mem::zeroed();
    IoStatusBlock = core::mem::zeroed();

    errCode = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileStandard as *mut FILE_STANDARD_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
        FileStandardInformation,
    );
    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        return 0;
    }
    if !lpFileSize.is_null() {
        *lpFileSize = FileStandard.EndOfFile;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetCompressedFileSizeA(
    lpFileName: LPCSTR,
    lpFileSizeHigh: LPDWORD,
) -> DWORD {
    let FileNameW: PWCHAR = FilenameA2W(lpFileName, 0);
    if FileNameW.is_null() {
        return INVALID_FILE_SIZE;
    }
    return GetCompressedFileSizeW(FileNameW as LPCWSTR, lpFileSizeHigh);
}

#[no_mangle]
pub unsafe extern "system" fn GetCompressedFileSizeW(
    lpFileName: LPCWSTR,
    lpFileSizeHigh: LPDWORD,
) -> DWORD {
    let mut FileCompression: FILE_COMPRESSION_INFORMATION = core::mem::zeroed();
    let mut errCode: NTSTATUS = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut hFile: HANDLE;

    hFile = CreateFileW(
        lpFileName,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE {
        return INVALID_FILE_SIZE;
    }

    errCode = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileCompression as *mut _ as PVOID,
        core::mem::size_of::<FILE_COMPRESSION_INFORMATION>() as ULONG,
        FileCompressionInformation,
    );

    CloseHandle(hFile);

    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        return INVALID_FILE_SIZE;
    }

    if !lpFileSizeHigh.is_null() {
        *lpFileSizeHigh = FileCompression.CompressedFileSize.u.HighPart as DWORD;
    }

    SetLastError(NO_ERROR);
    FileCompression.CompressedFileSize.u.LowPart
}

#[no_mangle]
pub unsafe extern "system" fn GetFileInformationByHandle(
    hFile: HANDLE,
    lpFileInformation: LPBY_HANDLE_FILE_INFORMATION,
) -> BOOL {
    let mut FileFsVolume: FILE_FS_VOLUME_INFORMATION = core::mem::zeroed();
    let mut FileFsVolumeName: [WCHAR; 255] = [0; 255];
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut FileInternal: FILE_INTERNAL_INFORMATION = core::mem::zeroed();
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut errCode: NTSTATUS = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if IsConsoleHandle(hFile) {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }

    errCode = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileBasic as *mut _ as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );
    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        return FALSE as BOOL;
    }

    (*lpFileInformation).dwFileAttributes = FileBasic.FileAttributes as DWORD;

    (*lpFileInformation).ftCreationTime.dwHighDateTime = FileBasic.CreationTime.u.HighPart as DWORD;
    (*lpFileInformation).ftCreationTime.dwLowDateTime = FileBasic.CreationTime.u.LowPart;

    (*lpFileInformation).ftLastAccessTime.dwHighDateTime = FileBasic.LastAccessTime.u.HighPart as DWORD;
    (*lpFileInformation).ftLastAccessTime.dwLowDateTime = FileBasic.LastAccessTime.u.LowPart;

    (*lpFileInformation).ftLastWriteTime.dwHighDateTime = FileBasic.LastWriteTime.u.HighPart as DWORD;
    (*lpFileInformation).ftLastWriteTime.dwLowDateTime = FileBasic.LastWriteTime.u.LowPart;

    errCode = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileInternal as *mut _ as PVOID,
        core::mem::size_of::<FILE_INTERNAL_INFORMATION>() as ULONG,
        FileInternalInformation,
    );
    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        return FALSE as BOOL;
    }

    (*lpFileInformation).nFileIndexHigh = FileInternal.IndexNumber.u.HighPart as DWORD;
    (*lpFileInformation).nFileIndexLow = FileInternal.IndexNumber.u.LowPart;

    errCode = NtQueryVolumeInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileFsVolume as *mut _ as PVOID,
        (core::mem::size_of::<FILE_FS_VOLUME_INFORMATION>() + core::mem::size_of::<[WCHAR; 255]>()) as ULONG,
        FileFsVolumeInformation,
    );
    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        return FALSE as BOOL;
    }

    (*lpFileInformation).dwVolumeSerialNumber = FileFsVolume.VolumeSerialNumber as DWORD;

    errCode = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileStandard as *mut _ as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        FileStandardInformation,
    );
    if !NT_SUCCESS(errCode) {
        BaseSetLastNTError(errCode);
        return FALSE as BOOL;
    }

    (*lpFileInformation).nNumberOfLinks = FileStandard.NumberOfLinks as DWORD;
    (*lpFileInformation).nFileSizeHigh = FileStandard.EndOfFile.u.HighPart as DWORD;
    (*lpFileInformation).nFileSizeLow = FileStandard.EndOfFile.u.LowPart;

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileAttributesExW(
    lpFileName: LPCWSTR,
    fInfoLevelId: GET_FILEEX_INFO_LEVELS,
    lpFileInformation: LPVOID,
) -> BOOL {
    let mut FileInformation: FILE_NETWORK_OPEN_INFORMATION = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;
    let mut FileAttributeData: *mut WIN32_FILE_ATTRIBUTE_DATA = core::ptr::null_mut();

    TRACE!(b"GetFileAttributesExW(%S) called\n\0".as_ptr() as *const i8, lpFileName);

    if fInfoLevelId != GetFileExInfoStandard || lpFileInformation.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut FileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        WARN!(
            b"Invalid path '%S'\n\0".as_ptr() as *const i8,
            lpFileName,
        );
        SetLastError(ERROR_BAD_PATHNAME);
        return FALSE as BOOL;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtQueryFullAttributesFile(&mut ObjectAttributes, &mut FileInformation);

    RtlFreeUnicodeString(&mut FileName);

    if !NT_SUCCESS(Status) {
        WARN!(
            b"NtQueryFullAttributesFile() failed (Status %lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    FileAttributeData = lpFileInformation as *mut WIN32_FILE_ATTRIBUTE_DATA;
    (*FileAttributeData).dwFileAttributes = FileInformation.FileAttributes;
    (*FileAttributeData).ftCreationTime.dwLowDateTime = FileInformation.CreationTime.u.LowPart;
    (*FileAttributeData).ftCreationTime.dwHighDateTime = FileInformation.CreationTime.u.HighPart as DWORD;
    (*FileAttributeData).ftLastAccessTime.dwLowDateTime = FileInformation.LastAccessTime.u.LowPart;
    (*FileAttributeData).ftLastAccessTime.dwHighDateTime = FileInformation.LastAccessTime.u.HighPart as DWORD;
    (*FileAttributeData).ftLastWriteTime.dwLowDateTime = FileInformation.LastWriteTime.u.LowPart;
    (*FileAttributeData).ftLastWriteTime.dwHighDateTime = FileInformation.LastWriteTime.u.HighPart as DWORD;
    (*FileAttributeData).nFileSizeLow = FileInformation.EndOfFile.u.LowPart;
    (*FileAttributeData).nFileSizeHigh = FileInformation.EndOfFile.u.HighPart as DWORD;

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileAttributesExA(
    lpFileName: LPCSTR,
    fInfoLevelId: GET_FILEEX_INFO_LEVELS,
    lpFileInformation: LPVOID,
) -> BOOL {
    let FileNameW: PWCHAR = FilenameA2W(lpFileName, FALSE as BOOL);
    if FileNameW.is_null() {
        return FALSE as BOOL;
    }
    return GetFileAttributesExW(FileNameW as LPCWSTR, fInfoLevelId, lpFileInformation);
}

#[no_mangle]
pub unsafe extern "system" fn GetFileAttributesA(lpFileName: LPCSTR) -> DWORD {
    let mut FileNameW: PWSTR = core::ptr::null_mut();
    if lpFileName.is_null()
        || {
            FileNameW = FilenameA2W(lpFileName, 0) as PWSTR;
            FileNameW.is_null()
        }
    {
        return INVALID_FILE_ATTRIBUTES;
    }
    return GetFileAttributesW(FileNameW as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD {
    let mut Status: NTSTATUS = 0;
    let mut FileName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut FileInformation: FILE_BASIC_INFORMATION = core::mem::zeroed();

    /* Get the NT path name */
    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut FileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return INVALID_FILE_ATTRIBUTES;
    }

    /* Prepare for querying attributes */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Simply query attributes */
    Status = NtQueryAttributesFile(&mut ObjectAttributes, &mut FileInformation);
    // RtlDosPathNameToNtPathName_U heap-allocated FileName.Buffer; every call leaked it
    // until upstream ef8bcd3fbe7 ("Fix leaks after calling RtlDosPathNameToNtPathName_U").
    RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
    if !NT_SUCCESS(Status) {
        /* It failed? Is it a DOS device? */
        if RtlIsDosDeviceName_U(lpFileName) != 0 {
            return FILE_ATTRIBUTE_ARCHIVE;
        }

        /* Set the error otherwise */
        BaseSetLastNTError(Status);
        return INVALID_FILE_ATTRIBUTES;
    }

    /* Return the file attributes */
    return FileInformation.FileAttributes;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileAttributesByHandle(
    hFile: HANDLE,
    dwFileAttributes: LPDWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Status: NTSTATUS;

    // UNREFERENCED_PARAMETER(dwFlags);
    let _ = dwFlags;

    if IsConsoleHandle(hFile) {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    Status = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );
    if NT_SUCCESS(Status) {
        *dwFileAttributes = FileBasic.FileAttributes;
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetFileAttributesByHandle(
    hFile: HANDLE,
    dwFileAttributes: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    // UNREFERENCED_PARAMETER(dwFlags);
    let _ = dwFlags;

    if IsConsoleHandle(hFile) {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return 0; // FALSE
    }

    Status = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );
    if NT_SUCCESS(Status) {
        FileBasic.FileAttributes = dwFileAttributes;

        Status = NtSetInformationFile(
            hFile,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
            FileBasicInformation,
        );
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0; // FALSE
    }

    return 1; // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SetFileAttributesA(
    lpFileName: LPCSTR,
    dwFileAttributes: DWORD,
) -> BOOL {
    let mut FileNameW: PWCHAR = core::ptr::null_mut();
    FileNameW = FilenameA2W(lpFileName, 0);
    if FileNameW.is_null() {
        return 0;
    }
    return SetFileAttributesW(FileNameW as LPCWSTR, dwFileAttributes);
}

#[no_mangle]
pub unsafe extern "system" fn SetFileAttributesW(
    lpFileName: LPCWSTR,
    dwFileAttributes: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut PathUBuffer: PWSTR;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut NtPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileInformation: FILE_BASIC_INFORMATION = core::mem::zeroed();

    /* Get relative name */
    if RtlDosPathNameToRelativeNtPathName_U(
        lpFileName,
        &mut NtPathU,
        core::ptr::null_mut(),
        &mut RelativeName,
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    /* Save buffer to allow later freeing */
    PathUBuffer = NtPathU.Buffer;

    /* If we have relative name (and root dir), use them instead */
    if RelativeName.RelativeName.Length != 0 {
        NtPathU.Length = RelativeName.RelativeName.Length;
        NtPathU.MaximumLength = RelativeName.RelativeName.MaximumLength;
        NtPathU.Buffer = RelativeName.RelativeName.Buffer;
    } else {
        RelativeName.ContainingDirectory = core::ptr::null_mut();
    }

    /* Prepare the object attribute for opening the file */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut NtPathU;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Attempt to open the file, while supporting reparse point */
    Status = NtOpenFile(
        &mut FileHandle,
        (FILE_WRITE_ATTRIBUTES | SYNCHRONIZE) as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
        (FILE_OPEN_REPARSE_POINT | FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT) as ULONG,
    );

    /* If opening failed, check whether it was because of reparse point support */
    if !NT_SUCCESS(Status) {
        /* Nope, just quit */
        if Status != STATUS_INVALID_PARAMETER {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* Yes, retry without */
        Status = NtOpenFile(
            &mut FileHandle,
            (FILE_WRITE_ATTRIBUTES | SYNCHRONIZE) as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
            (FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT) as ULONG,
        );

        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }
    }

    /* We don't need strings anylonger */
    RtlReleaseRelativeName(&mut RelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);

    /* Zero our structure, we'll only set file attributes */
    core::ptr::write_bytes(&mut FileInformation, 0, 1);

    /* Set the attributes, filtering only allowed attributes, and forcing normal attribute */
    FileInformation.FileAttributes = (dwFileAttributes & FILE_ATTRIBUTE_VALID_SET_FLAGS) | FILE_ATTRIBUTE_NORMAL;

    /* Finally, set the attributes */
    Status = NtSetInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        &mut FileInformation as *mut _ as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );

    /* Close the file */
    NtClose(FileHandle);

    /* If it failed, set the error and fail */
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileTime(
    hFile: HANDLE,
    lpCreationTime: LPFILETIME,
    lpLastAccessTime: LPFILETIME,
    lpLastWriteTime: LPFILETIME,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();

    if IsConsoleHandle(hFile) {
        BaseSetLastNTError(STATUS_INVALID_HANDLE);
        return 0;
    }

    Status = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if !lpCreationTime.is_null() {
        (*lpCreationTime).dwLowDateTime = FileBasic.CreationTime.u.LowPart;
        (*lpCreationTime).dwHighDateTime = FileBasic.CreationTime.u.HighPart as DWORD;
    }

    if !lpLastAccessTime.is_null() {
        (*lpLastAccessTime).dwLowDateTime = FileBasic.LastAccessTime.u.LowPart;
        (*lpLastAccessTime).dwHighDateTime = FileBasic.LastAccessTime.u.HighPart as DWORD;
    }

    if !lpLastWriteTime.is_null() {
        (*lpLastWriteTime).dwLowDateTime = FileBasic.LastWriteTime.u.LowPart;
        (*lpLastWriteTime).dwHighDateTime = FileBasic.LastWriteTime.u.HighPart as DWORD;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetFileTime(
    hFile: HANDLE,
    lpCreationTime: *const FILETIME,
    lpLastAccessTime: *const FILETIME,
    lpLastWriteTime: *const FILETIME,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();

    if IsConsoleHandle(hFile) {
        BaseSetLastNTError(STATUS_INVALID_HANDLE);
        return 0;
    }

    core::ptr::write_bytes(&mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut u8, 0, core::mem::size_of::<FILE_BASIC_INFORMATION>());

    if !lpCreationTime.is_null() {
        FileBasic.CreationTime.u.LowPart = (*lpCreationTime).dwLowDateTime;
        FileBasic.CreationTime.u.HighPart = (*lpCreationTime).dwHighDateTime as i32;
    }

    if !lpLastAccessTime.is_null() {
        FileBasic.LastAccessTime.u.LowPart = (*lpLastAccessTime).dwLowDateTime;
        FileBasic.LastAccessTime.u.HighPart = (*lpLastAccessTime).dwHighDateTime as i32;
    }

    if !lpLastWriteTime.is_null() {
        FileBasic.LastWriteTime.u.LowPart = (*lpLastWriteTime).dwLowDateTime;
        FileBasic.LastWriteTime.u.HighPart = (*lpLastWriteTime).dwHighDateTime as i32;
    }

    Status = NtSetInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetEndOfFile(hFile: HANDLE) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut EndOfFileInfo: FILE_END_OF_FILE_INFORMATION = core::mem::zeroed();
    let mut FileAllocationInfo_local: FILE_ALLOCATION_INFORMATION = core::mem::zeroed();
    let mut FilePosInfo: FILE_POSITION_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if IsConsoleHandle(hFile) {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }

    Status = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FilePosInfo as *mut FILE_POSITION_INFORMATION as PVOID,
        core::mem::size_of::<FILE_POSITION_INFORMATION>() as ULONG,
        FilePositionInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    EndOfFileInfo.EndOfFile.QuadPart = FilePosInfo.CurrentByteOffset.QuadPart;

    Status = NtSetInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut EndOfFileInfo as *mut FILE_END_OF_FILE_INFORMATION as PVOID,
        core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as ULONG,
        FileEndOfFileInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    FileAllocationInfo_local.AllocationSize.QuadPart = FilePosInfo.CurrentByteOffset.QuadPart;

    Status = NtSetInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileAllocationInfo_local as *mut FILE_ALLOCATION_INFORMATION as PVOID,
        core::mem::size_of::<FILE_ALLOCATION_INFORMATION>() as ULONG,
        FileAllocationInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetFileValidData(
    hFile: HANDLE,
    ValidDataLength: LONGLONG,
) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ValidDataLengthInformation: FILE_VALID_DATA_LENGTH_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS;

    ValidDataLengthInformation.ValidDataLength.QuadPart = ValidDataLength;

    Status = NtSetInformationFile(
        hFile,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut ValidDataLengthInformation as *mut FILE_VALID_DATA_LENGTH_INFORMATION as PVOID,
        core::mem::size_of::<FILE_VALID_DATA_LENGTH_INFORMATION>() as ULONG,
        FileValidDataLengthInformation,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/fileinfo.c
//  * PURPOSE:         Directory functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(kernel32file);
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// FlushFileBuffers(IN HANDLE hFile)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     hFile = TranslateStdHandle(hFile);
//
//     if (IsConsoleHandle(hFile))
//     {
//         return FlushConsoleInputBuffer(hFile);
//     }
//
//     Status = NtFlushBuffersFile(hFile,
//                                 &IoStatusBlock);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// SetFilePointer(HANDLE hFile,
//            LONG lDistanceToMove,
//            PLONG lpDistanceToMoveHigh,
//            DWORD dwMoveMethod)
// {
//    FILE_POSITION_INFORMATION FilePosition;
//    FILE_STANDARD_INFORMATION FileStandard;
//    NTSTATUS errCode;
//    IO_STATUS_BLOCK IoStatusBlock;
//    LARGE_INTEGER Distance;
//
//    TRACE!("SetFilePointer(hFile %p, lDistanceToMove %d, dwMoveMethod %lu)\n",
//       hFile,lDistanceToMove,dwMoveMethod);
//
//    if(IsConsoleHandle(hFile))
//    {
//      SetLastError(ERROR_INVALID_HANDLE);
//      return INVALID_SET_FILE_POINTER;
//    }
//
//    if (lpDistanceToMoveHigh)
//    {
//       Distance.u.HighPart = *lpDistanceToMoveHigh;
//       Distance.u.LowPart = lDistanceToMove;
//    }
//    else
//    {
//       Distance.QuadPart = lDistanceToMove;
//    }
//
//    switch(dwMoveMethod)
//    {
//      case FILE_CURRENT:
//     errCode = NtQueryInformationFile(hFile,
//                    &IoStatusBlock,
//                    &FilePosition,
//                    sizeof(FILE_POSITION_INFORMATION),
//                    FilePositionInformation);
//     FilePosition.CurrentByteOffset.QuadPart += Distance.QuadPart;
//     if (!NT_SUCCESS(errCode))
//     {
//       if (lpDistanceToMoveHigh != NULL)
//           *lpDistanceToMoveHigh = -1;
//       BaseSetLastNTError(errCode);
//       return INVALID_SET_FILE_POINTER;
//     }
//     break;
//      case FILE_END:
//     errCode = NtQueryInformationFile(hFile,
//                                &IoStatusBlock,
//                                &FileStandard,
//                                sizeof(FILE_STANDARD_INFORMATION),
//                                FileStandardInformation);
//     FilePosition.CurrentByteOffset.QuadPart =
//                   FileStandard.EndOfFile.QuadPart + Distance.QuadPart;
//     if (!NT_SUCCESS(errCode))
//     {
//       if (lpDistanceToMoveHigh != NULL)
//           *lpDistanceToMoveHigh = -1;
//       BaseSetLastNTError(errCode);
//       return INVALID_SET_FILE_POINTER;
//     }
//     break;
//      case FILE_BEGIN:
//         FilePosition.CurrentByteOffset.QuadPart = Distance.QuadPart;
//     break;
//      default:
//         SetLastError(ERROR_INVALID_PARAMETER);
//     return INVALID_SET_FILE_POINTER;
//    }
//
//    if(FilePosition.CurrentByteOffset.QuadPart < 0)
//    {
//      SetLastError(ERROR_NEGATIVE_SEEK);
//      return INVALID_SET_FILE_POINTER;
//    }
//
//    if (lpDistanceToMoveHigh == NULL && FilePosition.CurrentByteOffset.HighPart != 0)
//    {
//      /* If we're moving the pointer outside of the 32 bit boundaries but
//         the application only passed a 32 bit value we need to bail out! */
//      SetLastError(ERROR_INVALID_PARAMETER);
//      return INVALID_SET_FILE_POINTER;
//    }
//
//    errCode = NtSetInformationFile(hFile,
//                   &IoStatusBlock,
//                   &FilePosition,
//                   sizeof(FILE_POSITION_INFORMATION),
//                   FilePositionInformation);
//    if (!NT_SUCCESS(errCode))
//      {
//        if (lpDistanceToMoveHigh != NULL)
//            *lpDistanceToMoveHigh = -1;
//
//        BaseSetLastNTError(errCode);
//        return INVALID_SET_FILE_POINTER;
//      }
//
//    if (lpDistanceToMoveHigh != NULL)
//      {
//         *lpDistanceToMoveHigh = FilePosition.CurrentByteOffset.u.HighPart;
//      }
//
//    if (FilePosition.CurrentByteOffset.u.LowPart == MAXDWORD)
//      {
//        /* The value of -1 is valid here, especially when the new
//           file position is greater than 4 GB. Since NtSetInformationFile
//           succeeded we never set an error code and we explicitly need
//           to clear a previously set error code in this case, which
//           an application will check if INVALID_SET_FILE_POINTER is returned! */
//        SetLastError(ERROR_SUCCESS);
//      }
//
//    return FilePosition.CurrentByteOffset.u.LowPart;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetFilePointerEx(HANDLE hFile,
//                  LARGE_INTEGER liDistanceToMove,
//                  PLARGE_INTEGER lpNewFilePointer,
//                  DWORD dwMoveMethod)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_POSITION_INFORMATION FilePosition;
//     FILE_STANDARD_INFORMATION FileStandard;
//
//     if (IsConsoleHandle(hFile))
//     {
//         BaseSetLastNTError(STATUS_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     switch (dwMoveMethod)
//     {
//         case FILE_CURRENT:
//         {
//             Status = NtQueryInformationFile(hFile, &IoStatusBlock,
// 			                                &FilePosition,
// 			                                sizeof(FILE_POSITION_INFORMATION),
// 			                                FilePositionInformation);
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//
// 	        FilePosition.CurrentByteOffset.QuadPart += liDistanceToMove.QuadPart;
//             break;
//         }
//
//         case FILE_END:
//         {
//             Status = NtQueryInformationFile(hFile, &IoStatusBlock,
//                                             &FileStandard,
//                                             sizeof(FILE_STANDARD_INFORMATION),
//                                             FileStandardInformation);
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//
//             FilePosition.CurrentByteOffset.QuadPart = FileStandard.EndOfFile.QuadPart +
//                                                       liDistanceToMove.QuadPart;
//             break;
//         }
//
//         case FILE_BEGIN:
//         {
//             FilePosition.CurrentByteOffset.QuadPart = liDistanceToMove.QuadPart;
//             break;
//         }
//
//         default:
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//         }
//     }
//
//     if (FilePosition.CurrentByteOffset.QuadPart < 0)
//     {
//         SetLastError(ERROR_NEGATIVE_SEEK);
//         return FALSE;
//     }
//
//     Status = NtSetInformationFile(hFile, &IoStatusBlock, &FilePosition,
//                                   sizeof(FILE_POSITION_INFORMATION),
//                                   FilePositionInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     if (lpNewFilePointer != NULL)
//     {
//        *lpNewFilePointer = FilePosition.CurrentByteOffset;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD WINAPI
// GetFileType(HANDLE hFile)
// {
//   FILE_FS_DEVICE_INFORMATION DeviceInfo;
//   IO_STATUS_BLOCK StatusBlock;
//   NTSTATUS Status;
//
//   /* Get real handle */
//   hFile = TranslateStdHandle(hFile);
//
//   /* Check for console handle */
//   if (IsConsoleHandle(hFile))
//     {
//       if (VerifyConsoleIoHandle(hFile))
// 	return FILE_TYPE_CHAR;
//     }
//
//   Status = NtQueryVolumeInformationFile(hFile,
// 					&StatusBlock,
// 					&DeviceInfo,
// 					sizeof(FILE_FS_DEVICE_INFORMATION),
// 					FileFsDeviceInformation);
//   if (!NT_SUCCESS(Status))
//     {
//       BaseSetLastNTError(Status);
//       return FILE_TYPE_UNKNOWN;
//     }
//
//   switch (DeviceInfo.DeviceType)
//     {
//       case FILE_DEVICE_CD_ROM:
//       case FILE_DEVICE_CD_ROM_FILE_SYSTEM:
//       case FILE_DEVICE_CONTROLLER:
//       case FILE_DEVICE_DATALINK:
//       case FILE_DEVICE_DFS:
//       case FILE_DEVICE_DISK:
//       case FILE_DEVICE_DISK_FILE_SYSTEM:
//       case FILE_DEVICE_VIRTUAL_DISK:
// 	return FILE_TYPE_DISK;
//
//       case FILE_DEVICE_KEYBOARD:
//       case FILE_DEVICE_MOUSE:
//       case FILE_DEVICE_NULL:
//       case FILE_DEVICE_PARALLEL_PORT:
//       case FILE_DEVICE_PRINTER:
//       case FILE_DEVICE_SERIAL_PORT:
//       case FILE_DEVICE_SCREEN:
//       case FILE_DEVICE_SOUND:
//       case FILE_DEVICE_MODEM:
// 	return FILE_TYPE_CHAR;
//
//       case FILE_DEVICE_NAMED_PIPE:
// 	return FILE_TYPE_PIPE;
//     }
//
//   return FILE_TYPE_UNKNOWN;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD WINAPI
// GetFileSize(HANDLE hFile,
// 	    LPDWORD lpFileSizeHigh)
// {
//    NTSTATUS errCode;
//    FILE_STANDARD_INFORMATION FileStandard;
//    IO_STATUS_BLOCK IoStatusBlock;
//
//    errCode = NtQueryInformationFile(hFile,
// 				    &IoStatusBlock,
// 				    &FileStandard,
// 				    sizeof(FILE_STANDARD_INFORMATION),
// 				    FileStandardInformation);
//    if (!NT_SUCCESS(errCode))
//      {
// 	BaseSetLastNTError(errCode);
// 	if ( lpFileSizeHigh == NULL )
// 	  {
// 	     return -1;
// 	  }
// 	else
// 	  {
// 	     return 0;
// 	  }
//      }
//    if ( lpFileSizeHigh != NULL )
//      *lpFileSizeHigh = FileStandard.EndOfFile.u.HighPart;
//
//    return FileStandard.EndOfFile.u.LowPart;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetFileSizeEx(
//     HANDLE hFile,
//     PLARGE_INTEGER lpFileSize
//     )
// {
//    NTSTATUS errCode;
//    FILE_STANDARD_INFORMATION FileStandard;
//    IO_STATUS_BLOCK IoStatusBlock;
//
//    errCode = NtQueryInformationFile(hFile,
// 				    &IoStatusBlock,
// 				    &FileStandard,
// 				    sizeof(FILE_STANDARD_INFORMATION),
// 				    FileStandardInformation);
//    if (!NT_SUCCESS(errCode))
//      {
// 	BaseSetLastNTError(errCode);
// 	return FALSE;
//      }
//    if (lpFileSize)
//      *lpFileSize = FileStandard.EndOfFile;
//
//    return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD WINAPI
// GetCompressedFileSizeA(LPCSTR lpFileName,
// 		       LPDWORD lpFileSizeHigh)
// {
//    PWCHAR FileNameW;
//
//    if (!(FileNameW = FilenameA2W(lpFileName, FALSE)))
//       return INVALID_FILE_SIZE;
//
//    return GetCompressedFileSizeW(FileNameW, lpFileSizeHigh);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD WINAPI
// GetCompressedFileSizeW(LPCWSTR lpFileName,
// 		       LPDWORD lpFileSizeHigh)
// {
//    FILE_COMPRESSION_INFORMATION FileCompression;
//    NTSTATUS errCode;
//    IO_STATUS_BLOCK IoStatusBlock;
//    HANDLE hFile;
//
//    hFile = CreateFileW(lpFileName,
// 		       GENERIC_READ,
// 		       FILE_SHARE_READ,
// 		       NULL,
// 		       OPEN_EXISTING,
// 		       FILE_ATTRIBUTE_NORMAL,
// 		       NULL);
//
//    if (hFile == INVALID_HANDLE_VALUE)
//       return INVALID_FILE_SIZE;
//
//    errCode = NtQueryInformationFile(hFile,
// 				    &IoStatusBlock,
// 				    &FileCompression,
// 				    sizeof(FILE_COMPRESSION_INFORMATION),
// 				    FileCompressionInformation);
//
//    CloseHandle(hFile);
//
//    if (!NT_SUCCESS(errCode))
//      {
// 	BaseSetLastNTError(errCode);
// 	return INVALID_FILE_SIZE;
//      }
//
//    if(lpFileSizeHigh)
//     *lpFileSizeHigh = FileCompression.CompressedFileSize.u.HighPart;
//
//    SetLastError(NO_ERROR);
//    return FileCompression.CompressedFileSize.u.LowPart;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// GetFileInformationByHandle(HANDLE hFile,
// 			   LPBY_HANDLE_FILE_INFORMATION lpFileInformation)
// {
//    struct
//    {
//         FILE_FS_VOLUME_INFORMATION FileFsVolume;
//         WCHAR Name[255];
//    }
//    FileFsVolume;
//
//    FILE_BASIC_INFORMATION FileBasic;
//    FILE_INTERNAL_INFORMATION FileInternal;
//    FILE_STANDARD_INFORMATION FileStandard;
//    NTSTATUS errCode;
//    IO_STATUS_BLOCK IoStatusBlock;
//
//    if(IsConsoleHandle(hFile))
//    {
//      SetLastError(ERROR_INVALID_HANDLE);
//      return FALSE;
//    }
//
//    errCode = NtQueryInformationFile(hFile,
// 				    &IoStatusBlock,
// 				    &FileBasic,
// 				    sizeof(FILE_BASIC_INFORMATION),
// 				    FileBasicInformation);
//    if (!NT_SUCCESS(errCode))
//      {
// 	BaseSetLastNTError(errCode);
// 	return FALSE;
//      }
//
//    lpFileInformation->dwFileAttributes = (DWORD)FileBasic.FileAttributes;
//
//    lpFileInformation->ftCreationTime.dwHighDateTime = FileBasic.CreationTime.u.HighPart;
//    lpFileInformation->ftCreationTime.dwLowDateTime = FileBasic.CreationTime.u.LowPart;
//
//    lpFileInformation->ftLastAccessTime.dwHighDateTime = FileBasic.LastAccessTime.u.HighPart;
//    lpFileInformation->ftLastAccessTime.dwLowDateTime = FileBasic.LastAccessTime.u.LowPart;
//
//    lpFileInformation->ftLastWriteTime.dwHighDateTime = FileBasic.LastWriteTime.u.HighPart;
//    lpFileInformation->ftLastWriteTime.dwLowDateTime = FileBasic.LastWriteTime.u.LowPart;
//
//    errCode = NtQueryInformationFile(hFile,
// 				    &IoStatusBlock,
// 				    &FileInternal,
// 				    sizeof(FILE_INTERNAL_INFORMATION),
// 				    FileInternalInformation);
//    if (!NT_SUCCESS(errCode))
//      {
// 	BaseSetLastNTError(errCode);
// 	return FALSE;
//      }
//
//    lpFileInformation->nFileIndexHigh = FileInternal.IndexNumber.u.HighPart;
//    lpFileInformation->nFileIndexLow = FileInternal.IndexNumber.u.LowPart;
//
//    errCode = NtQueryVolumeInformationFile(hFile,
// 					  &IoStatusBlock,
// 					  &FileFsVolume,
// 					  sizeof(FileFsVolume),
// 					  FileFsVolumeInformation);
//    if (!NT_SUCCESS(errCode))
//      {
// 	BaseSetLastNTError(errCode);
// 	return FALSE;
//      }
//
//    lpFileInformation->dwVolumeSerialNumber = FileFsVolume.FileFsVolume.VolumeSerialNumber;
//
//    errCode = NtQueryInformationFile(hFile,
// 				    &IoStatusBlock,
// 				    &FileStandard,
// 				    sizeof(FILE_STANDARD_INFORMATION),
// 				    FileStandardInformation);
//    if (!NT_SUCCESS(errCode))
//      {
// 	BaseSetLastNTError(errCode);
// 	return FALSE;
//      }
//
//    lpFileInformation->nNumberOfLinks = FileStandard.NumberOfLinks;
//    lpFileInformation->nFileSizeHigh = FileStandard.EndOfFile.u.HighPart;
//    lpFileInformation->nFileSizeLow = FileStandard.EndOfFile.u.LowPart;
//
//    return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// GetFileAttributesExW(LPCWSTR lpFileName,
// 		     GET_FILEEX_INFO_LEVELS fInfoLevelId,
// 		     LPVOID lpFileInformation)
// {
//   FILE_NETWORK_OPEN_INFORMATION FileInformation;
//   OBJECT_ATTRIBUTES ObjectAttributes;
//   UNICODE_STRING FileName;
//   NTSTATUS Status;
//   WIN32_FILE_ATTRIBUTE_DATA* FileAttributeData;
//
//   TRACE!("GetFileAttributesExW(%S) called\n", lpFileName);
//
//
//   if (fInfoLevelId != GetFileExInfoStandard || lpFileInformation == NULL)
//   {
//      SetLastError(ERROR_INVALID_PARAMETER);
//      return FALSE;
//   }
//
//   /* Validate and translate the filename */
//   if (!RtlDosPathNameToNtPathName_U (lpFileName,
// 				     &FileName,
// 				     NULL,
// 				     NULL))
//     {
//       WARN ("Invalid path '%S'\n", lpFileName);
//       SetLastError (ERROR_BAD_PATHNAME);
//       return FALSE;
//     }
//
//   /* build the object attributes */
//   InitializeObjectAttributes (&ObjectAttributes,
// 			      &FileName,
// 			      OBJ_CASE_INSENSITIVE,
// 			      NULL,
// 			      NULL);
//
//   /* Get file attributes */
//   Status = NtQueryFullAttributesFile(&ObjectAttributes,
//                                      &FileInformation);
//
//   RtlFreeUnicodeString (&FileName);
//   if (!NT_SUCCESS (Status))
//     {
//       WARN ("NtQueryFullAttributesFile() failed (Status %lx)\n", Status);
//       BaseSetLastNTError (Status);
//       return FALSE;
//     }
//
//   FileAttributeData = (WIN32_FILE_ATTRIBUTE_DATA*)lpFileInformation;
//   FileAttributeData->dwFileAttributes = FileInformation.FileAttributes;
//   FileAttributeData->ftCreationTime.dwLowDateTime = FileInformation.CreationTime.u.LowPart;
//   FileAttributeData->ftCreationTime.dwHighDateTime = FileInformation.CreationTime.u.HighPart;
//   FileAttributeData->ftLastAccessTime.dwLowDateTime = FileInformation.LastAccessTime.u.LowPart;
//   FileAttributeData->ftLastAccessTime.dwHighDateTime = FileInformation.LastAccessTime.u.HighPart;
//   FileAttributeData->ftLastWriteTime.dwLowDateTime = FileInformation.LastWriteTime.u.LowPart;
//   FileAttributeData->ftLastWriteTime.dwHighDateTime = FileInformation.LastWriteTime.u.HighPart;
//   FileAttributeData->nFileSizeLow = FileInformation.EndOfFile.u.LowPart;
//   FileAttributeData->nFileSizeHigh = FileInformation.EndOfFile.u.HighPart;
//
//   return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// GetFileAttributesExA(LPCSTR lpFileName,
// 		     GET_FILEEX_INFO_LEVELS fInfoLevelId,
// 		     LPVOID lpFileInformation)
// {
//    PWCHAR FileNameW;
//
//    if (!(FileNameW = FilenameA2W(lpFileName, FALSE)))
//       return FALSE;
//
//    return GetFileAttributesExW(FileNameW, fInfoLevelId, lpFileInformation);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD WINAPI
// GetFileAttributesA(LPCSTR lpFileName)
// {
//    PWSTR FileNameW;
//
//    if (!lpFileName || !(FileNameW = FilenameA2W(lpFileName, FALSE)))
//       return INVALID_FILE_ATTRIBUTES;
//
//    return GetFileAttributesW(FileNameW);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetFileAttributesW(LPCWSTR lpFileName)
// {
//     NTSTATUS Status;
//     UNICODE_STRING FileName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     FILE_BASIC_INFORMATION FileInformation;
//
//     /* Get the NT path name */
//     if (!RtlDosPathNameToNtPathName_U(lpFileName, &FileName, NULL, NULL))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return INVALID_FILE_ATTRIBUTES;
//     }
//
//     /* Prepare for querying attributes */
//     InitializeObjectAttributes(&ObjectAttributes, &FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL, NULL);
//     /* Simply query attributes */
//     Status = NtQueryAttributesFile(&ObjectAttributes, &FileInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         /* It failed? Is it a DOS device? */
//         if (RtlIsDosDeviceName_U(lpFileName))
//         {
//             return FILE_ATTRIBUTE_ARCHIVE;
//         }
//
//         /* Set the error otherwise */
//         BaseSetLastNTError(Status);
//         return INVALID_FILE_ATTRIBUTES;
//     }
//
//     /* Return the file attributes */
//     return FileInformation.FileAttributes;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// GetFileAttributesByHandle(IN HANDLE hFile,
//                           OUT LPDWORD dwFileAttributes,
//                           IN DWORD dwFlags)
// {
//     FILE_BASIC_INFORMATION FileBasic;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER(dwFlags);
//
//     if (IsConsoleHandle(hFile))
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     Status = NtQueryInformationFile(hFile,
//                                     &IoStatusBlock,
//                                     &FileBasic,
//                                     sizeof(FileBasic),
//                                     FileBasicInformation);
//     if (NT_SUCCESS(Status))
//     {
//         *dwFileAttributes = FileBasic.FileAttributes;
//         return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// SetFileAttributesByHandle(IN HANDLE hFile,
//                           IN DWORD dwFileAttributes,
//                           IN DWORD dwFlags)
// {
//     FILE_BASIC_INFORMATION FileBasic;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER(dwFlags);
//
//     if (IsConsoleHandle(hFile))
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     Status = NtQueryInformationFile(hFile,
//                                     &IoStatusBlock,
//                                     &FileBasic,
//                                     sizeof(FileBasic),
//                                     FileBasicInformation);
//     if (NT_SUCCESS(Status))
//     {
//         FileBasic.FileAttributes = dwFileAttributes;
//
//         Status = NtSetInformationFile(hFile,
//                                       &IoStatusBlock,
//                                       &FileBasic,
//                                       sizeof(FileBasic),
//                                       FileBasicInformation);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// SetFileAttributesA(
//    LPCSTR lpFileName,
// 	DWORD dwFileAttributes)
// {
//    PWCHAR FileNameW;
//
//    if (!(FileNameW = FilenameA2W(lpFileName, FALSE)))
//       return FALSE;
//
//    return SetFileAttributesW(FileNameW, dwFileAttributes);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetFileAttributesW(LPCWSTR lpFileName,
//                    DWORD dwFileAttributes)
// {
//     NTSTATUS Status;
//     PWSTR PathUBuffer;
//     HANDLE FileHandle;
//     UNICODE_STRING NtPathU;
//     IO_STATUS_BLOCK IoStatusBlock;
//     RTL_RELATIVE_NAME_U RelativeName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     FILE_BASIC_INFORMATION FileInformation;
//
//     /* Get relative name */
//     if (!RtlDosPathNameToRelativeNtPathName_U(lpFileName, &NtPathU, NULL, &RelativeName))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* Save buffer to allow later freeing */
//     PathUBuffer = NtPathU.Buffer;
//
//     /* If we have relative name (and root dir), use them instead */
//     if (RelativeName.RelativeName.Length != 0)
//     {
//         NtPathU.Length = RelativeName.RelativeName.Length;
//         NtPathU.MaximumLength = RelativeName.RelativeName.MaximumLength;
//         NtPathU.Buffer = RelativeName.RelativeName.Buffer;
//     }
//     else
//     {
//         RelativeName.ContainingDirectory = NULL;
//     }
//
//     /* Prepare the object attribute for opening the file */
//     InitializeObjectAttributes(&ObjectAttributes, &NtPathU,
//                                 OBJ_CASE_INSENSITIVE,
//                                 RelativeName.ContainingDirectory, NULL);
//
//     /* Attempt to open the file, while supporting reparse point */
//     Status = NtOpenFile(&FileHandle, FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
//                         &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_OPEN_REPARSE_POINT | FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT);
//     /* If opening failed, check whether it was because of reparse point support */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Nope, just quit */
//         if (Status != STATUS_INVALID_PARAMETER)
//         {
//             RtlReleaseRelativeName(&RelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//             BaseSetLastNTError(Status);
//
//             return FALSE;
//         }
//
//         /* Yes, retry without */
//         Status = NtOpenFile(&FileHandle, FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
//                             &ObjectAttributes, &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                             FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT);
//         if (!NT_SUCCESS(Status))
//         {
//             RtlReleaseRelativeName(&RelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//             BaseSetLastNTError(Status);
//
//             return FALSE;
//         }
//     }
//
//     /* We don't need strings anylonger */
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//
//     /* Zero our structure, we'll only set file attributes */
//     ZeroMemory(&FileInformation, sizeof(FileInformation));
//     /* Set the attributes, filtering only allowed attributes, and forcing normal attribute */
//     FileInformation.FileAttributes = (dwFileAttributes & FILE_ATTRIBUTE_VALID_SET_FLAGS) | FILE_ATTRIBUTE_NORMAL;
//
//     /* Finally, set the attributes */
//     Status = NtSetInformationFile(FileHandle, &IoStatusBlock, &FileInformation,
//                                   sizeof(FILE_BASIC_INFORMATION), FileBasicInformation);
//     /* Close the file */
//     NtClose(FileHandle);
//
//     /* If it failed, set the error and fail */
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// GetFileTime(IN HANDLE hFile,
//             OUT LPFILETIME lpCreationTime OPTIONAL,
//             OUT LPFILETIME lpLastAccessTime OPTIONAL,
//             OUT LPFILETIME lpLastWriteTime OPTIONAL)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION FileBasic;
//
//     if(IsConsoleHandle(hFile))
//     {
//         BaseSetLastNTError(STATUS_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     Status = NtQueryInformationFile(hFile,
//                                     &IoStatusBlock,
//                                     &FileBasic,
//                                     sizeof(FILE_BASIC_INFORMATION),
//                                     FileBasicInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     if (lpCreationTime)
//     {
//         lpCreationTime->dwLowDateTime = FileBasic.CreationTime.LowPart;
//         lpCreationTime->dwHighDateTime = FileBasic.CreationTime.HighPart;
//     }
//
//     if (lpLastAccessTime)
//     {
//         lpLastAccessTime->dwLowDateTime = FileBasic.LastAccessTime.LowPart;
//         lpLastAccessTime->dwHighDateTime = FileBasic.LastAccessTime.HighPart;
//     }
//
//     if (lpLastWriteTime)
//     {
//         lpLastWriteTime->dwLowDateTime = FileBasic.LastWriteTime.LowPart;
//         lpLastWriteTime->dwHighDateTime = FileBasic.LastWriteTime.HighPart;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// SetFileTime(IN HANDLE hFile,
//             CONST FILETIME *lpCreationTime OPTIONAL,
//             CONST FILETIME *lpLastAccessTime OPTIONAL,
//             CONST FILETIME *lpLastWriteTime OPTIONAL)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION FileBasic;
//
//     if(IsConsoleHandle(hFile))
//     {
//         BaseSetLastNTError(STATUS_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     memset(&FileBasic, 0, sizeof(FILE_BASIC_INFORMATION));
//
//     if (lpCreationTime)
//     {
//         FileBasic.CreationTime.LowPart = lpCreationTime->dwLowDateTime;
//         FileBasic.CreationTime.HighPart = lpCreationTime->dwHighDateTime;
//     }
//
//     if (lpLastAccessTime)
//     {
//         FileBasic.LastAccessTime.LowPart = lpLastAccessTime->dwLowDateTime;
//         FileBasic.LastAccessTime.HighPart = lpLastAccessTime->dwHighDateTime;
//     }
//
//     if (lpLastWriteTime)
//     {
//         FileBasic.LastWriteTime.LowPart = lpLastWriteTime->dwLowDateTime;
//         FileBasic.LastWriteTime.HighPart = lpLastWriteTime->dwHighDateTime;
//     }
//
//     Status = NtSetInformationFile(hFile,
//                                   &IoStatusBlock,
//                                   &FileBasic,
//                                   sizeof(FILE_BASIC_INFORMATION),
//                                   FileBasicInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * The caller must have opened the file with the DesiredAccess FILE_WRITE_DATA flag set.
//  *
//  * @implemented
//  */
// BOOL WINAPI
// SetEndOfFile(HANDLE hFile)
// {
// 	IO_STATUS_BLOCK  IoStatusBlock;
// 	FILE_END_OF_FILE_INFORMATION	EndOfFileInfo;
// 	FILE_ALLOCATION_INFORMATION		FileAllocationInfo_local;
// 	FILE_POSITION_INFORMATION		 FilePosInfo;
// 	NTSTATUS Status;
//
// 	if(IsConsoleHandle(hFile))
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
//
// 	//get current position
// 	Status = NtQueryInformationFile(
// 					hFile,
// 					&IoStatusBlock,
// 					&FilePosInfo,
// 					sizeof(FILE_POSITION_INFORMATION),
// 					FilePositionInformation
// 					);
//
// 	if (!NT_SUCCESS(Status)){
// 		BaseSetLastNTError(Status);
// 		return FALSE;
// 	}
//
// 	EndOfFileInfo.EndOfFile.QuadPart = FilePosInfo.CurrentByteOffset.QuadPart;
//
// 	/*
// 	NOTE:
// 	This call is not supposed to free up any space after the eof marker
// 	if the file gets truncated. We have to deallocate the space explicitly afterwards.
// 	But...most file systems dispatch both FileEndOfFileInformation
// 	and FileAllocationInformation as they were the same	command.
//
// 	*/
// 	Status = NtSetInformationFile(
// 						hFile,
// 						&IoStatusBlock,	 //out
// 						&EndOfFileInfo,
// 						sizeof(FILE_END_OF_FILE_INFORMATION),
// 						FileEndOfFileInformation
// 						);
//
// 	if (!NT_SUCCESS(Status)){
// 		BaseSetLastNTError(Status);
// 		return FALSE;
// 	}
//
// 	FileAllocationInfo_local.AllocationSize.QuadPart = FilePosInfo.CurrentByteOffset.QuadPart;
//
//
// 	Status = NtSetInformationFile(
// 						hFile,
// 						&IoStatusBlock,	 //out
// 						&FileAllocationInfo_local,
// 						sizeof(FILE_ALLOCATION_INFORMATION),
// 						FileAllocationInformation
// 						);
//
// 	if (!NT_SUCCESS(Status)){
// 		BaseSetLastNTError(Status);
// 		return FALSE;
// 	}
//
// 	return TRUE;
//
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetFileValidData(
//     HANDLE hFile,
//     LONGLONG ValidDataLength
//     )
// {
// 	IO_STATUS_BLOCK IoStatusBlock;
// 	FILE_VALID_DATA_LENGTH_INFORMATION ValidDataLengthInformation;
// 	NTSTATUS Status;
//
// 	ValidDataLengthInformation.ValidDataLength.QuadPart = ValidDataLength;
//
// 	Status = NtSetInformationFile(
// 						hFile,
// 						&IoStatusBlock,	 //out
// 						&ValidDataLengthInformation,
// 						sizeof(FILE_VALID_DATA_LENGTH_INFORMATION),
// 						FileValidDataLengthInformation
// 						);
//
// 	if (!NT_SUCCESS(Status)){
// 		BaseSetLastNTError(Status);
// 		return FALSE;
// 	}
//
// 	return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

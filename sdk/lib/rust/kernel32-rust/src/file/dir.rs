use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
use crate::client::utils::Basep8BitStringToDynamicUnicodeString;
use crate::file::deviceio::DeviceIoControl;
use crate::prelude::*;
use core::ffi::c_void;

// kernel32-internal helpers + cross-file Win32 exports this file calls.
use crate::k32_externs::{
    GetFullPathNameW, CloseHandle, InitializeObjectAttributes, RtlCopyMemory,
};

extern "system" {
    fn Basep8BitStringToStaticUnicodeString(LpStr: LPCSTR) -> PUNICODE_STRING;
    fn GetProcessHeap() -> HANDLE;
    fn SetVolumeMountPointW(lpszVolumeMountPoint: LPCWSTR, lpszVolumeName: LPCWSTR) -> BOOL;
    fn DeleteVolumeMountPointW(lpszVolumeMountPoint: LPCWSTR) -> BOOL;
    fn BaseMarkFileForDelete(FileHandle: HANDLE, FileAttributes: ULONG);
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();
pub const SFN_LENGTH: u32 = 12;

// ntstatus.h
pub const STATUS_NOT_A_DIRECTORY: NTSTATUS = 0xC0000103u32 as NTSTATUS;
pub const STATUS_IO_REPARSE_TAG_NOT_HANDLED: NTSTATUS = 0xC0000279u32 as NTSTATUS;

// winioctl.h: CTL_CODE(FILE_DEVICE_FILE_SYSTEM=0x9, fn, METHOD_BUFFERED=0, FILE_ANY_ACCESS=0)
pub const FSCTL_GET_REPARSE_POINT: ULONG = 0x000900A8; // fn 42
pub const FSCTL_SET_REPARSE_POINT: ULONG = 0x000900A4; // fn 41

// ntifs.h IS_VOLUME_NAME(s, l): s is a *const WCHAR, l a USHORT byte length.
macro_rules! IS_VOLUME_NAME {
    ($s:expr, $l:expr $(,)?) => {{
        let s = $s;
        let l = $l;
        (l == 96 || (l == 98 && *s.offset(48) == '\\' as u16))
            && *s.offset(0) == '\\' as u16
            && (*s.offset(1) == '?' as u16 || *s.offset(1) == '\\' as u16)
            && *s.offset(2) == '?' as u16
            && *s.offset(3) == '\\' as u16
            && *s.offset(4) == 'V' as u16
            && *s.offset(5) == 'o' as u16
            && *s.offset(6) == 'l' as u16
            && *s.offset(7) == 'u' as u16
            && *s.offset(8) == 'm' as u16
            && *s.offset(9) == 'e' as u16
            && *s.offset(10) == '{' as u16
            && *s.offset(19) == '-' as u16
            && *s.offset(24) == '-' as u16
            && *s.offset(29) == '-' as u16
            && *s.offset(34) == '-' as u16
            && *s.offset(47) == '}' as u16
    }};
}

#[no_mangle]
pub unsafe extern "system" fn CreateDirectoryA(
    lpPathName: LPCSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> BOOL {
    let PathNameW: PUNICODE_STRING;
    PathNameW = Basep8BitStringToStaticUnicodeString(lpPathName);
    if PathNameW.is_null() {
        return 0i32;
    }
    return CreateDirectoryW((*PathNameW).Buffer, lpSecurityAttributes);
}

#[no_mangle]
pub unsafe extern "system" fn CreateDirectoryExA(
    lpTemplateDirectory: LPCSTR,
    lpNewDirectory: LPCSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> BOOL {
    let mut TemplateDirectoryW: PUNICODE_STRING = core::ptr::null_mut();
    let mut NewDirectoryW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ret: BOOL = 0;

    TemplateDirectoryW = Basep8BitStringToStaticUnicodeString(lpTemplateDirectory);
    if TemplateDirectoryW.is_null() {
        return 0;
    }

    if Basep8BitStringToDynamicUnicodeString(&mut NewDirectoryW, lpNewDirectory) == 0 {
        return 0;
    }

    ret = CreateDirectoryExW(
        (*TemplateDirectoryW).Buffer,
        NewDirectoryW.Buffer,
        lpSecurityAttributes,
    );

    RtlFreeUnicodeString(&mut NewDirectoryW);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CreateDirectoryW(
    lpPathName: LPCWSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> BOOL {
    let mut Length: DWORD;
    let mut Status: NTSTATUS;
    let mut DirectoryHandle: HANDLE = core::mem::zeroed();
    let mut NtPathU: _UNICODE_STRING = core::mem::zeroed();
    let PathUBuffer: PWSTR;
    let mut FilePart: PWSTR = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    /* Get relative name */
    if RtlDosPathNameToRelativeNtPathName_U(
        lpPathName,
        &mut NtPathU,
        core::ptr::null_mut(),
        &mut RelativeName,
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND as DWORD);
        return 0;
    }

    /* Check if path length is < MAX_PATH (with space for file name).
     * If not, prefix is required.
     */
    if NtPathU.Length as usize
        > ((MAX_PATH - SFN_LENGTH) as usize).wrapping_mul(core::mem::size_of::<WCHAR>())
        && *lpPathName.offset(0) as i32 != '\\' as i32
        && *lpPathName.offset(1) as i32 != '\\' as i32
        && *lpPathName.offset(2) as i32 != '?' as i32
        && *lpPathName.offset(3) as i32 != '\\' as i32
    {
        /* Get file name position and full path length */
        Length = GetFullPathNameW(lpPathName, 0, core::ptr::null_mut(), &mut FilePart);
        if Length == 0 {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathU.Buffer as PVOID);
            SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
            return 0;
        }

        /* Keep place for 8.3 file name */
        Length = (Length as DWORD).wrapping_add(SFN_LENGTH as DWORD);
        /* No prefix, so, must be smaller than MAX_PATH */
        if Length > MAX_PATH as DWORD {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(GetProcessHeap(), 0, NtPathU.Buffer as PVOID);
            SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
            return 0;
        }
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

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.ObjectName = &mut NtPathU as *mut _UNICODE_STRING as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = if !lpSecurityAttributes.is_null() {
        (*lpSecurityAttributes).lpSecurityDescriptor
    } else {
        core::ptr::null_mut()
    };
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        &mut DirectoryHandle,
        (FILE_LIST_DIRECTORY | SYNCHRONIZE) as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL as ULONG,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        FILE_CREATE as ULONG,
        (FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
        core::ptr::null_mut(),
        0,
    );

    RtlReleaseRelativeName(&mut RelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);

    if NT_SUCCESS(Status) {
        NtClose(DirectoryHandle);
        return 1;
    }

    if RtlIsDosDeviceName_U(lpPathName) != 0 {
        Status = STATUS_NOT_A_DIRECTORY;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CreateDirectoryExW(
    lpTemplateDirectory: LPCWSTR,
    lpNewDirectory: LPCWSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> BOOL {
    let mut Length: DWORD = 0;
    let mut Status: NTSTATUS = 0;
    let mut EaBuffer: PVOID = core::ptr::null_mut();
    let mut ReparsePoint: BOOL = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileEaInfo: FILE_EA_INFORMATION = core::mem::zeroed();
    let mut EaLength: ULONG = 0;
    let mut StreamSize: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut BasicInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut ReparseDataBuffer: PREPARSE_DATA_BUFFER = core::ptr::null_mut();
    let mut TemplateHandle: HANDLE = core::ptr::null_mut();
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();
    let mut StreamInfo: PFILE_STREAM_INFORMATION = core::ptr::null_mut();
    let mut FileTagInfo: FILE_ATTRIBUTE_TAG_INFORMATION = core::mem::zeroed();
    let mut NtPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut NtTemplatePathU: _UNICODE_STRING = core::mem::zeroed();
    let mut NewDirectory: _UNICODE_STRING = core::mem::zeroed();
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut TemplateRelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut TemplateBuffer: PWSTR = core::ptr::null_mut();
    let mut PathUBuffer: PWSTR = core::ptr::null_mut();
    let mut FilePart: PWSTR = core::ptr::null_mut();
    let mut SubstituteName: PWSTR = core::ptr::null_mut();

    /* Get relative name of the template */
    if RtlDosPathNameToRelativeNtPathName_U(
        lpTemplateDirectory,
        &mut NtTemplatePathU,
        core::ptr::null_mut(),
        &mut TemplateRelativeName,
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    /* Save buffer for further freeing */
    TemplateBuffer = NtTemplatePathU.Buffer;

    /* If we have relative name (and root dir), use them instead */
    if TemplateRelativeName.RelativeName.Length != 0 {
        NtTemplatePathU.Length = TemplateRelativeName.RelativeName.Length;
        NtTemplatePathU.MaximumLength = TemplateRelativeName.RelativeName.MaximumLength;
        NtTemplatePathU.Buffer = TemplateRelativeName.RelativeName.Buffer;
    } else {
        TemplateRelativeName.ContainingDirectory = core::ptr::null_mut();
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtTemplatePathU,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    /* Open template directory */
    Status = NtOpenFile(
        &mut TemplateHandle,
        (FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | FILE_READ_EA) as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        (FILE_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
    );

    if !NT_SUCCESS(Status) {
        if Status != STATUS_INVALID_PARAMETER {
            RtlReleaseRelativeName(&mut TemplateRelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* Opening failed due to lacking reparse points support in the FSD, try without */
        Status = NtOpenFile(
            &mut TemplateHandle,
            (FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | FILE_READ_EA) as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
            (FILE_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
        );

        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut TemplateRelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* Request file attributes */
        BasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
        Status = NtQueryInformationFile(
            TemplateHandle,
            &mut IoStatusBlock,
            &mut BasicInfo as *mut FILE_BASIC_INFORMATION as PVOID,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
            FileBasicInformation,
        );
        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut TemplateRelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
            CloseHandle(TemplateHandle);
            BaseSetLastNTError(Status);
            return 0;
        }
    } else {
        /* Request file attributes */
        BasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
        Status = NtQueryInformationFile(
            TemplateHandle,
            &mut IoStatusBlock,
            &mut BasicInfo as *mut FILE_BASIC_INFORMATION as PVOID,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
            FileBasicInformation,
        );
        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut TemplateRelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
            CloseHandle(TemplateHandle);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* If it is a reparse point, then get information about it */
        if (BasicInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
            Status = NtQueryInformationFile(
                TemplateHandle,
                &mut IoStatusBlock,
                &mut FileTagInfo as *mut FILE_ATTRIBUTE_TAG_INFORMATION as PVOID,
                core::mem::size_of::<FILE_ATTRIBUTE_TAG_INFORMATION>() as ULONG,
                FileAttributeTagInformation,
            );
            if !NT_SUCCESS(Status) {
                RtlReleaseRelativeName(&mut TemplateRelativeName);
                RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
                CloseHandle(TemplateHandle);
                BaseSetLastNTError(Status);
                return 0;
            }

            /* Only mount points are supported, retry without if anything different */
            if FileTagInfo.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT {
                CloseHandle(TemplateHandle);
                /* goto OpenWithoutReparseSupport */
                Status = NtOpenFile(
                    &mut TemplateHandle,
                    (FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | FILE_READ_EA) as ACCESS_MASK,
                    &mut ObjectAttributes,
                    &mut IoStatusBlock,
                    (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
                    (FILE_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
                );

                if !NT_SUCCESS(Status) {
                    RtlReleaseRelativeName(&mut TemplateRelativeName);
                    RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
                    BaseSetLastNTError(Status);
                    return 0;
                }

                /* Request file attributes */
                BasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
                Status = NtQueryInformationFile(
                    TemplateHandle,
                    &mut IoStatusBlock,
                    &mut BasicInfo as *mut FILE_BASIC_INFORMATION as PVOID,
                    core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
                    FileBasicInformation,
                );
                if !NT_SUCCESS(Status) {
                    RtlReleaseRelativeName(&mut TemplateRelativeName);
                    RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
                    CloseHandle(TemplateHandle);
                    BaseSetLastNTError(Status);
                    return 0;
                }
            } else {
                /* Mark we are playing with a reparse point */
                ReparsePoint = 1;
            }
        }
    }

    /* Get relative name of the directory */
    if RtlDosPathNameToRelativeNtPathName_U(
        lpNewDirectory,
        &mut NtPathU,
        core::ptr::null_mut(),
        &mut RelativeName,
    ) == 0
    {
        RtlReleaseRelativeName(&mut TemplateRelativeName);
        RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
        NtClose(TemplateHandle);
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    /* Save its buffer for further freeing */
    PathUBuffer = NtPathU.Buffer;

    /* Template & directory can't be the same */
    if RtlEqualUnicodeString(&NtPathU, &NtTemplatePathU, 1) != 0 {
        RtlReleaseRelativeName(&mut RelativeName);
        RtlReleaseRelativeName(&mut TemplateRelativeName);
        RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
        NtClose(TemplateHandle);
        SetLastError(ERROR_INVALID_NAME);
        return 0;
    }

    RtlReleaseRelativeName(&mut TemplateRelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer as PVOID);

    /* Check if path length is < MAX_PATH (with space for file name).
     * If not, prefix is required.
     */
    if NtPathU.Length > ((MAX_PATH - SFN_LENGTH) as usize * core::mem::size_of::<WCHAR>()) as u16
        && *lpNewDirectory.offset(0) as i32 != '\\' as i32
        && *lpNewDirectory.offset(1) as i32 != '\\' as i32
        && *lpNewDirectory.offset(2) as i32 != '?' as i32
        && *lpNewDirectory.offset(3) as i32 != '\\' as i32
    {
        /* Get file name position and full path length */
        Length = GetFullPathNameW(lpNewDirectory, 0, core::ptr::null_mut(), &mut FilePart);
        if Length == 0 {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            CloseHandle(TemplateHandle);
            SetLastError(ERROR_FILENAME_EXCED_RANGE);
            return 0;
        }

        /* Keep place for 8.3 file name */
        Length = Length.wrapping_add(SFN_LENGTH as DWORD);
        /* No prefix, so, must be smaller than MAX_PATH */
        if Length > MAX_PATH as DWORD {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            CloseHandle(TemplateHandle);
            SetLastError(ERROR_FILENAME_EXCED_RANGE);
            return 0;
        }
    }

    /* If we have relative name (and root dir), use them instead */
    if RelativeName.RelativeName.Length != 0 {
        NtPathU.Length = RelativeName.RelativeName.Length;
        NtPathU.MaximumLength = RelativeName.RelativeName.MaximumLength;
        NtPathU.Buffer = RelativeName.RelativeName.Buffer;
    } else {
        RelativeName.ContainingDirectory = core::ptr::null_mut();
    }

    /* Get extended attributes */
    Status = NtQueryInformationFile(
        TemplateHandle,
        &mut IoStatusBlock,
        &mut FileEaInfo as *mut FILE_EA_INFORMATION as PVOID,
        core::mem::size_of::<FILE_EA_INFORMATION>() as ULONG,
        FileEaInformation,
    );
    if !NT_SUCCESS(Status) {
        RtlReleaseRelativeName(&mut RelativeName);
        RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
        CloseHandle(TemplateHandle);
        BaseSetLastNTError(Status);
        return 0;
    }

    /* Start reading extended attributes */
    if FileEaInfo.EaSize != 0 {
        EaLength = FileEaInfo.EaSize.wrapping_mul(2);
        loop {
            /* Allocate buffer for reading */
            EaBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, EaLength as SIZE_T);
            if EaBuffer.is_null() {
                RtlReleaseRelativeName(&mut RelativeName);
                RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
                CloseHandle(TemplateHandle);
                BaseSetLastNTError(STATUS_NO_MEMORY);
                return 0;
            }

            /* Query EAs */
            Status = NtQueryEaFile(
                TemplateHandle,
                &mut IoStatusBlock,
                EaBuffer,
                EaLength,
                0,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                1,
            );
            if !NT_SUCCESS(Status) {
                RtlFreeHeap(RtlGetProcessHeap(), 0, EaBuffer);
                IoStatusBlock.Information = 0;
            }

            /* If we don't fail because of too small buffer, stop here */
            if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
                EaLength = IoStatusBlock.Information as ULONG;
                break;
            }

            EaLength = EaLength.wrapping_mul(2);
        }
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtPathU,
        OBJ_CASE_INSENSITIVE,
        RelativeName.ContainingDirectory,
        if !lpSecurityAttributes.is_null() {
            (*lpSecurityAttributes).lpSecurityDescriptor
        } else {
            core::ptr::null_mut()
        },
    );

    /* Ensure attributes are valid */
    BasicInfo.FileAttributes &= FILE_ATTRIBUTE_VALID_FLAGS;

    /* Create the new directory */
    Status = NtCreateFile(
        &mut DirectoryHandle,
        (FILE_LIST_DIRECTORY
            | SYNCHRONIZE
            | FILE_WRITE_ATTRIBUTES
            | FILE_READ_ATTRIBUTES
            | (if (BasicInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                FILE_ADD_FILE
            } else {
                0
            })) as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        BasicInfo.FileAttributes,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        FILE_CREATE as ULONG,
        (FILE_DIRECTORY_FILE
            | FILE_SYNCHRONOUS_IO_NONALERT
            | FILE_OPEN_FOR_BACKUP_INTENT
            | FILE_OPEN_REPARSE_POINT) as ULONG,
        EaBuffer,
        EaLength,
    );
    if !NT_SUCCESS(Status) {
        if Status == STATUS_INVALID_PARAMETER || Status == STATUS_ACCESS_DENIED {
            /* If creation failed, it might be because FSD doesn't support reparse points
             * Retry without asking for such support in case template is not a reparse point
             */
            if ReparsePoint == 0 {
                Status = NtCreateFile(
                    &mut DirectoryHandle,
                    (FILE_LIST_DIRECTORY | SYNCHRONIZE | FILE_WRITE_ATTRIBUTES | FILE_READ_ATTRIBUTES)
                        as ACCESS_MASK,
                    &mut ObjectAttributes,
                    &mut IoStatusBlock,
                    core::ptr::null_mut(),
                    BasicInfo.FileAttributes,
                    (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
                    FILE_CREATE as ULONG,
                    (FILE_DIRECTORY_FILE
                        | FILE_SYNCHRONOUS_IO_NONALERT
                        | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
                    EaBuffer,
                    EaLength,
                );
            } else {
                RtlReleaseRelativeName(&mut RelativeName);
                RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
                if !EaBuffer.is_null() {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, EaBuffer);
                }
                CloseHandle(TemplateHandle);
                BaseSetLastNTError(Status);
                return 0;
            }
        }
    }

    RtlReleaseRelativeName(&mut RelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
    if !EaBuffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, EaBuffer);
    }

    if !NT_SUCCESS(Status) {
        NtClose(TemplateHandle);
        if RtlIsDosDeviceName_U(lpNewDirectory) != 0 {
            Status = STATUS_NOT_A_DIRECTORY;
        }
        BaseSetLastNTError(Status);
        return 0;
    }

    /* If template is a reparse point, copy reparse data */
    if ReparsePoint != 0 {
        ReparseDataBuffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            MAXIMUM_REPARSE_DATA_BUFFER_SIZE as SIZE_T,
        ) as PREPARSE_DATA_BUFFER;
        if ReparseDataBuffer.is_null() {
            NtClose(TemplateHandle);
            NtClose(DirectoryHandle);
            SetLastError(STATUS_NO_MEMORY as DWORD);
            return 0;
        }

        /* First query data */
        Status = NtFsControlFile(
            TemplateHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            FSCTL_GET_REPARSE_POINT as ULONG,
            core::ptr::null_mut(),
            0,
            ReparseDataBuffer as PVOID,
            MAXIMUM_REPARSE_DATA_BUFFER_SIZE as ULONG,
        );
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
            NtClose(TemplateHandle);
            NtClose(DirectoryHandle);
            SetLastError(Status as DWORD);
            return 0;
        }

        /* Once again, ensure it is a mount point */
        if (*ReparseDataBuffer).ReparseTag != IO_REPARSE_TAG_MOUNT_POINT {
            RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
            NtClose(TemplateHandle);
            NtClose(DirectoryHandle);
            SetLastError(STATUS_OBJECT_NAME_INVALID as DWORD);
            return 0;
        }

        /* Get volume name */
        SubstituteName = ((&raw mut (*ReparseDataBuffer).__bindgen_anon_1.MountPointReparseBuffer.PathBuffer
            as *mut WCHAR) as usize)
            .wrapping_add(
                (*ReparseDataBuffer)
                    .__bindgen_anon_1.MountPointReparseBuffer
                    .SubstituteNameOffset as usize,
            ) as PWSTR;

        if IS_VOLUME_NAME!(
            SubstituteName,
            (*ReparseDataBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength,
        ) {
            /* Prepare to define a new mount point for that volume */
            RtlInitUnicodeString(&mut NewDirectory, lpNewDirectory);
            NewDirectory.Buffer = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                (NewDirectory.Length as usize)
                    .wrapping_add(2usize.wrapping_mul(core::mem::size_of::<WCHAR>()))
                    as SIZE_T,
            ) as PWSTR;
            if NewDirectory.Buffer.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
                NtClose(TemplateHandle);
                NtClose(DirectoryHandle);
                return 0;
            }

            RtlCopyMemory(
                NewDirectory.Buffer as PVOID,
                lpNewDirectory as PVOID,
                NewDirectory.Length as SIZE_T,
            );
            if *NewDirectory
                .Buffer
                .offset((NewDirectory.Length as usize / core::mem::size_of::<WCHAR>()) as isize)
                as i32
                != '\\' as i32
            {
                *NewDirectory
                    .Buffer
                    .offset((NewDirectory.Length as usize / core::mem::size_of::<WCHAR>()) as isize) =
                    '\\' as u16;
                *NewDirectory.Buffer.offset(
                    (NewDirectory.Length as usize / core::mem::size_of::<WCHAR>())
                        .wrapping_add(1) as isize,
                ) = 0;
            }

            /* Define a new mount point for that volume */
            SetVolumeMountPointW(NewDirectory.Buffer, SubstituteName);

            RtlFreeHeap(RtlGetProcessHeap(), 0, NewDirectory.Buffer as PVOID);
            RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
            NtClose(TemplateHandle);
            NtClose(DirectoryHandle);
            return 1;
        }

        /* Otherwise copy data raw */
        Status = NtFsControlFile(
            DirectoryHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            FSCTL_SET_REPARSE_POINT as ULONG,
            ReparseDataBuffer as PVOID,
            ((*ReparseDataBuffer).ReparseDataLength as ULONG)
                .wrapping_add(core::mem::offset_of!(REPARSE_DATA_BUFFER, __bindgen_anon_1.MountPointReparseBuffer.PathBuffer) as ULONG),
            core::ptr::null_mut(),
            0,
        );

        RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
        NtClose(TemplateHandle);
        NtClose(DirectoryHandle);

        if NT_SUCCESS(Status) {
            return 1;
        }

        BaseSetLastNTError(Status);
        return 0;
    }
    /* In case it's not a reparse point, handle streams on the file */
    else {
        StreamSize = 0x1000;
        loop {
            StreamInfo = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                StreamSize as SIZE_T,
            ) as PFILE_STREAM_INFORMATION;
            if StreamInfo.is_null() {
                BaseMarkFileForDelete(DirectoryHandle, BasicInfo.FileAttributes);
                SetLastError(STATUS_NO_MEMORY as DWORD);
                break;
            }

            /* Query stream information */
            Status = NtQueryInformationFile(
                TemplateHandle,
                &mut IoStatusBlock,
                StreamInfo as PVOID,
                StreamSize,
                FileStreamInformation,
            );
            if NT_SUCCESS(Status) {
                break;
            }

            RtlFreeHeap(RtlGetProcessHeap(), 0, StreamInfo as PVOID);
            StreamInfo = core::ptr::null_mut();

            /* If it failed, ensure that's not because of too small buffer */
            if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
                break;
            }

            StreamSize = StreamSize.wrapping_mul(2);
        }

        if !NT_SUCCESS(Status) || IoStatusBlock.Information == 0 {
            if !StreamInfo.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, StreamInfo as PVOID);
            }

            NtClose(TemplateHandle);
            NtClose(DirectoryHandle);
            return 1;
        }

        /* FIXME: TODO */
        // DPRINT removed
        RtlFreeHeap(RtlGetProcessHeap(), 0, StreamInfo as PVOID);
        NtClose(TemplateHandle);
        NtClose(DirectoryHandle);
        return 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RemoveDirectoryA(lpPathName: LPCSTR) -> BOOL {
    let PathNameW: PUNICODE_STRING = Basep8BitStringToStaticUnicodeString(lpPathName);
    if PathNameW.is_null() {
        return 0;
    }
    return RemoveDirectoryW((*PathNameW).Buffer);
}

#[no_mangle]
pub unsafe extern "system" fn RemoveDirectoryW(lpPathName: LPCWSTR) -> BOOL {
    let mut Status: NTSTATUS;
    let mut BytesReturned: DWORD = 0;
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut NtPathU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut PathName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut RelativeName: RTL_RELATIVE_NAME_U = RTL_RELATIVE_NAME_U {
        RelativeName: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        },
        ContainingDirectory: core::ptr::null_mut(),
        CurDirRef: core::ptr::null_mut(),
    };
    let mut PathUBuffer: PWSTR = core::ptr::null_mut();
    let mut SubstituteName: PWSTR = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ReparseDataBuffer: PREPARSE_DATA_BUFFER = core::ptr::null_mut();
    let mut FileDispInfo: FILE_DISPOSITION_INFORMATION = FILE_DISPOSITION_INFORMATION {
        DeleteFileA: 0,
    };
    let mut FileTagInfo: FILE_ATTRIBUTE_TAG_INFORMATION = FILE_ATTRIBUTE_TAG_INFORMATION {
        FileAttributes: 0,
        ReparseTag: 0,
    };

    /* Get relative name */
    if RtlDosPathNameToRelativeNtPathName_U(
        lpPathName,
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

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut NtPathU;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    'MarkFileForDelete: {
    /* Try to open directory */
    Status = NtOpenFile(
        &mut DirectoryHandle,
        (DELETE | SYNCHRONIZE | FAILED_ACCESS_ACE_FLAG) as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
        (FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        /* We only accept failure for reparse points not being supported */
        if Status != STATUS_INVALID_PARAMETER {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* Try to open, with reparse points support */
        Status = NtOpenFile(
            &mut DirectoryHandle,
            (DELETE | SYNCHRONIZE) as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
            (FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* Success, mark directory */
        break 'MarkFileForDelete;
    }

    /* Get information about file (and reparse point) */
    Status = NtQueryInformationFile(
        DirectoryHandle,
        &mut IoStatusBlock,
        &mut FileTagInfo as *mut _ as PVOID,
        core::mem::size_of::<FILE_ATTRIBUTE_TAG_INFORMATION>() as ULONG,
        FileAttributeTagInformation,
    );
    if !NT_SUCCESS(Status) {
        /* FSD might not support querying reparse points information */
        if Status != STATUS_NOT_IMPLEMENTED && Status != STATUS_INVALID_PARAMETER {
            NtClose(DirectoryHandle);
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* If that's the case, then just delete directory */
        break 'MarkFileForDelete;
    }

    /* If that's not a reparse point, nothing more to do than just delete */
    if FileTagInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT == 0 {
        break 'MarkFileForDelete;
    }

    /* Check if that's a mount point */
    if FileTagInfo.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT {
        /* It's not */
        NtClose(DirectoryHandle);

        /* So, try to reopen directory, ignoring mount point */
        Status = NtOpenFile(
            &mut DirectoryHandle,
            (DELETE | SYNCHRONIZE) as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
            (FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
        );
        if NT_SUCCESS(Status) {
            /* It succeed, we can safely delete directory (and ignore reparse point) */
            break 'MarkFileForDelete;
        }

        /* If it failed, only allow case where IO mount point was ignored */
        if Status != STATUS_IO_REPARSE_TAG_NOT_HANDLED {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        /* Reopen with reparse point support */
        Status = NtOpenFile(
            &mut DirectoryHandle,
            (DELETE | SYNCHRONIZE) as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
            (FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT) as ULONG,
        );
        if NT_SUCCESS(Status) {
            /* And mark for delete */
            break 'MarkFileForDelete;
        }

        RtlReleaseRelativeName(&mut RelativeName);
        RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
        BaseSetLastNTError(Status);
        return 0;
    }

    /* Here, we have a mount point, prepare to query information about it */
    ReparseDataBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        MAXIMUM_REPARSE_DATA_BUFFER_SIZE as SIZE_T,
    ) as PREPARSE_DATA_BUFFER;
    if ReparseDataBuffer.is_null() {
        RtlReleaseRelativeName(&mut RelativeName);
        RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);
        NtClose(DirectoryHandle);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* Query */
    if DeviceIoControl(
        DirectoryHandle,
        FSCTL_GET_REPARSE_POINT,
        core::ptr::null_mut(),
        0,
        ReparseDataBuffer as LPVOID,
        MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
        &mut BytesReturned,
        core::ptr::null_mut(),
    ) == 0
    {
        RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
        break 'MarkFileForDelete;
    }

    /* Get volume name */
    SubstituteName = ((&raw mut (*ReparseDataBuffer).__bindgen_anon_1.MountPointReparseBuffer.PathBuffer) as *mut WCHAR as ULONG_PTR)
        .wrapping_add((*ReparseDataBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameOffset as ULONG_PTR) as PWSTR;
    if !IS_VOLUME_NAME!(
        SubstituteName,
        (*ReparseDataBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength,
    ) {
        /* This is not a volume, we can safely delete */
        RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
        break 'MarkFileForDelete;
    }

    /* Prepare to delete mount point */
    RtlInitUnicodeString(&mut PathName, lpPathName);
    PathName.Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (PathName.Length as usize).wrapping_add(2 * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PWSTR;
    if PathName.Buffer.is_null() {
        RtlReleaseRelativeName(&mut RelativeName);
        RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
        NtClose(DirectoryHandle);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    RtlCopyMemory(
        PathName.Buffer as PVOID,
        lpPathName as PVOID,
        PathName.Length as SIZE_T,
    );
    if *PathName.Buffer.offset((PathName.Length as usize / core::mem::size_of::<WCHAR>()).wrapping_sub(1) as isize) != '\\' as u16
    {
        *PathName.Buffer.offset((PathName.Length as usize / core::mem::size_of::<WCHAR>()) as isize) = '\\' as u16;
        *PathName.Buffer.offset((PathName.Length as usize / core::mem::size_of::<WCHAR>()).wrapping_add(1) as isize) = UNICODE_NULL;
    }

    /* Delete mount point for that volume */
    DeleteVolumeMountPointW(PathName.Buffer);
    RtlFreeHeap(RtlGetProcessHeap(), 0, PathName.Buffer as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer as PVOID);
    } // 'MarkFileForDelete

    /* And mark directory for delete */
    RtlReleaseRelativeName(&mut RelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer as PVOID);

    /* Mark & set */
    FileDispInfo.DeleteFileA = 1;
    Status = NtSetInformationFile(
        DirectoryHandle,
        &mut IoStatusBlock,
        &mut FileDispInfo as *mut _ as PVOID,
        core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as ULONG,
        FileDispositionInformation,
    );
    NtClose(DirectoryHandle);

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
//  * FILE:            dll/win32/kernel32/client/file/dir.c
//  * PURPOSE:         Directory functions
//  * PROGRAMMER:      Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Short File Name length in chars (8.3) */
// #define SFN_LENGTH 12
//
// /* Match a volume name like:
//  * \\?\Volume{GUID}
//  */
// #define IS_VOLUME_NAME(s, l)                       \
//   ((l == 96 || (l == 98 && s[48] == '\\')) &&      \
//    s[0] == '\\'&& (s[1] == '?' || s[1] == '\\') && \
//    s[2] == '?' && s[3] == '\\' && s[4] == 'V' &&   \
//    s[5] == 'o' && s[6] == 'l' && s[7] == 'u' &&    \
//    s[8] == 'm' && s[9] == 'e' && s[10] == '{' &&   \
//    s[19] == '-' && s[24] == '-' && s[29] == '-' && \
//    s[34] == '-' && s[47] == '}')
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateDirectoryA(IN LPCSTR lpPathName,
//                  IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     PUNICODE_STRING PathNameW;
//
//     PathNameW = Basep8BitStringToStaticUnicodeString(lpPathName);
//     if (!PathNameW)
//     {
//         return FALSE;
//     }
//
//     return CreateDirectoryW(PathNameW->Buffer,
//                             lpSecurityAttributes);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateDirectoryExA(IN LPCSTR lpTemplateDirectory,
//                    IN LPCSTR lpNewDirectory,
//                    IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     PUNICODE_STRING TemplateDirectoryW;
//     UNICODE_STRING NewDirectoryW;
//     BOOL ret;
//
//     TemplateDirectoryW = Basep8BitStringToStaticUnicodeString(lpTemplateDirectory);
//     if (!TemplateDirectoryW)
//     {
//         return FALSE;
//     }
//
//     if (!Basep8BitStringToDynamicUnicodeString(&NewDirectoryW, lpNewDirectory))
//     {
//         return FALSE;
//     }
//
//     ret = CreateDirectoryExW(TemplateDirectoryW->Buffer,
//                              NewDirectoryW.Buffer,
//                              lpSecurityAttributes);
//
//     RtlFreeUnicodeString(&NewDirectoryW);
//
//     return ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateDirectoryW(IN LPCWSTR lpPathName,
//                  IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     DWORD Length;
//     NTSTATUS Status;
//     HANDLE DirectoryHandle;
//     UNICODE_STRING NtPathU;
//     PWSTR PathUBuffer, FilePart;
//     IO_STATUS_BLOCK IoStatusBlock;
//     RTL_RELATIVE_NAME_U RelativeName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Get relative name */
//     if (!RtlDosPathNameToRelativeNtPathName_U(lpPathName, &NtPathU, NULL, &RelativeName))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* Check if path length is < MAX_PATH (with space for file name).
//      * If not, prefix is required.
//      */
//     if (NtPathU.Length > (MAX_PATH - SFN_LENGTH) * sizeof(WCHAR) && lpPathName[0] != L'\\' &&
//         lpPathName[1] != L'\\' && lpPathName[2] != L'?' && lpPathName[3] != L'\\')
//     {
//         /* Get file name position and full path length */
//         Length = GetFullPathNameW(lpPathName, 0, NULL, &FilePart);
//         if (Length == 0)
//         {
//             RtlReleaseRelativeName(&RelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathU.Buffer);
//             SetLastError(ERROR_FILENAME_EXCED_RANGE);
//             return FALSE;
//         }
//
//         /* Keep place for 8.3 file name */
//         Length += SFN_LENGTH;
//         /* No prefix, so, must be smaller than MAX_PATH */
//         if (Length > MAX_PATH)
//         {
//             RtlReleaseRelativeName(&RelativeName);
//             RtlFreeHeap(GetProcessHeap(), 0, NtPathU.Buffer);
//             SetLastError(ERROR_FILENAME_EXCED_RANGE);
//             return FALSE;
//         }
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
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtPathU,
//                                OBJ_CASE_INSENSITIVE,
//                                RelativeName.ContainingDirectory,
//                                (lpSecurityAttributes ? lpSecurityAttributes->lpSecurityDescriptor : NULL));
//
//     Status = NtCreateFile(&DirectoryHandle,
//                           FILE_LIST_DIRECTORY | SYNCHRONIZE,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FILE_ATTRIBUTE_NORMAL,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           FILE_CREATE,
//                           FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT,
//                           NULL,
//                           0);
//
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//
//     if (NT_SUCCESS(Status))
//     {
//         NtClose(DirectoryHandle);
//         return TRUE;
//     }
//
//     if (RtlIsDosDeviceName_U(lpPathName))
//     {
//         Status = STATUS_NOT_A_DIRECTORY;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateDirectoryExW(IN LPCWSTR lpTemplateDirectory,
//                    IN LPCWSTR lpNewDirectory,
//                    IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     DWORD Length;
//     NTSTATUS Status;
//     PVOID EaBuffer = NULL;
//     BOOL ReparsePoint = FALSE;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_EA_INFORMATION FileEaInfo;
//     ULONG EaLength = 0, StreamSize;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     FILE_BASIC_INFORMATION FileBasicInfo;
//     PREPARSE_DATA_BUFFER ReparseDataBuffer;
//     HANDLE TemplateHandle, DirectoryHandle;
//     PFILE_STREAM_INFORMATION FileStreamInfo;
//     FILE_ATTRIBUTE_TAG_INFORMATION FileTagInfo;
//     UNICODE_STRING NtPathU, NtTemplatePathU, NewDirectory;
//     RTL_RELATIVE_NAME_U RelativeName, TemplateRelativeName;
//     PWSTR TemplateBuffer, PathUBuffer, FilePart, SubstituteName;
//
//     /* Get relative name of the template */
//     if (!RtlDosPathNameToRelativeNtPathName_U(lpTemplateDirectory, &NtTemplatePathU, NULL, &TemplateRelativeName))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* Save buffer for further freeing */
//     TemplateBuffer = NtTemplatePathU.Buffer;
//
//     /* If we have relative name (and root dir), use them instead */
//     if (TemplateRelativeName.RelativeName.Length != 0)
//     {
//         NtTemplatePathU.Length = TemplateRelativeName.RelativeName.Length;
//         NtTemplatePathU.MaximumLength = TemplateRelativeName.RelativeName.MaximumLength;
//         NtTemplatePathU.Buffer = TemplateRelativeName.RelativeName.Buffer;
//     }
//     else
//     {
//         TemplateRelativeName.ContainingDirectory = NULL;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtTemplatePathU,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open template directory */
//     Status = NtOpenFile(&TemplateHandle,
//                         FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | FILE_READ_EA,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_OPEN_FOR_BACKUP_INTENT);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status != STATUS_INVALID_PARAMETER)
//         {
//             RtlReleaseRelativeName(&TemplateRelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
// OpenWithoutReparseSupport:
//         /* Opening failed due to lacking reparse points support in the FSD, try without */
//         Status = NtOpenFile(&TemplateHandle,
//                             FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | FILE_READ_EA,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             FILE_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT);
//
//         if (!NT_SUCCESS(Status))
//         {
//             RtlReleaseRelativeName(&TemplateRelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         /* Request file attributes */
//         FileBasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
//         Status = NtQueryInformationFile(TemplateHandle,
//                                         &IoStatusBlock,
//                                         &FileBasicInfo,
//                                         sizeof(FileBasicInfo),
//                                         FileBasicInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             RtlReleaseRelativeName(&TemplateRelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//             CloseHandle(TemplateHandle);
//             BaseSetLastNTError(Status);
//             return FALSE;
//
//         }
//     }
//     else
//     {
//         /* Request file attributes */
//         FileBasicInfo.FileAttributes = FILE_ATTRIBUTE_NORMAL;
//         Status = NtQueryInformationFile(TemplateHandle,
//                                         &IoStatusBlock,
//                                         &FileBasicInfo,
//                                         sizeof(FileBasicInfo),
//                                         FileBasicInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             RtlReleaseRelativeName(&TemplateRelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//             CloseHandle(TemplateHandle);
//             BaseSetLastNTError(Status);
//             return FALSE;
//
//         }
//
//         /* If it is a reparse point, then get information about it */
//         if (FileBasicInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT)
//         {
//             Status = NtQueryInformationFile(TemplateHandle,
//                                             &IoStatusBlock,
//                                             &FileTagInfo,
//                                             sizeof(FileTagInfo),
//                                             FileAttributeTagInformation);
//             if (!NT_SUCCESS(Status))
//             {
//                 RtlReleaseRelativeName(&TemplateRelativeName);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//                 CloseHandle(TemplateHandle);
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//
//             /* Only mount points are supported, retry without if anything different */
//             if (FileTagInfo.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT)
//             {
//                 CloseHandle(TemplateHandle);
//                 goto OpenWithoutReparseSupport;
//             }
//
//             /* Mark we are playing with a reparse point */
//             ReparsePoint = TRUE;
//         }
//     }
//
//     /* Get relative name of the directory */
//     if (!RtlDosPathNameToRelativeNtPathName_U(lpNewDirectory, &NtPathU, NULL, &RelativeName))
//     {
//         RtlReleaseRelativeName(&TemplateRelativeName);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//         NtClose(TemplateHandle);
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* Save its buffer for further freeing */
//     PathUBuffer = NtPathU.Buffer;
//
//     /* Template & directory can't be the same */
//     if (RtlEqualUnicodeString(&NtPathU,
//                               &NtTemplatePathU,
//                               TRUE))
//     {
//         RtlReleaseRelativeName(&RelativeName);
//         RtlReleaseRelativeName(&TemplateRelativeName);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//         NtClose(TemplateHandle);
//         SetLastError(ERROR_INVALID_NAME);
//         return FALSE;
//     }
//
//     RtlReleaseRelativeName(&TemplateRelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, TemplateBuffer);
//
//     /* Check if path length is < MAX_PATH (with space for file name).
//      * If not, prefix is required.
//      */
//     if (NtPathU.Length > (MAX_PATH - SFN_LENGTH) * sizeof(WCHAR) && lpNewDirectory[0] != L'\\' &&
//         lpNewDirectory[1] != L'\\' && lpNewDirectory[2] != L'?' && lpNewDirectory[3] != L'\\')
//     {
//         /* Get file name position and full path length */
//         Length = GetFullPathNameW(lpNewDirectory, 0, NULL, &FilePart);
//         if (Length == 0)
//         {
//             RtlReleaseRelativeName(&RelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//             CloseHandle(TemplateHandle);
//             SetLastError(ERROR_FILENAME_EXCED_RANGE);
//             return FALSE;
//         }
//
//         /* Keep place for 8.3 file name */
//         Length += SFN_LENGTH;
//         /* No prefix, so, must be smaller than MAX_PATH */
//         if (Length > MAX_PATH)
//         {
//             RtlReleaseRelativeName(&RelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//             CloseHandle(TemplateHandle);
//             SetLastError(ERROR_FILENAME_EXCED_RANGE);
//             return FALSE;
//         }
//     }
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
//     /* Get extended attributes */
//     Status = NtQueryInformationFile(TemplateHandle,
//                                     &IoStatusBlock,
//                                     &FileEaInfo,
//                                     sizeof(FileEaInfo),
//                                     FileEaInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlReleaseRelativeName(&RelativeName);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//         CloseHandle(TemplateHandle);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Start reading extended attributes */
//     if (FileEaInfo.EaSize != 0)
//     {
//         for (EaLength = FileEaInfo.EaSize * 2; ; EaLength = EaLength * 2)
//         {
//             /* Allocate buffer for reading */
//             EaBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, EaLength);
//             if (!EaBuffer)
//             {
//                 RtlReleaseRelativeName(&RelativeName);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//                 CloseHandle(TemplateHandle);
//                 BaseSetLastNTError(STATUS_NO_MEMORY);
//                 return FALSE;
//             }
//
//             /* Query EAs */
//             Status = NtQueryEaFile(TemplateHandle,
//                                    &IoStatusBlock,
//                                    EaBuffer,
//                                    EaLength,
//                                    FALSE,
//                                    NULL,
//                                    0,
//                                    NULL,
//                                    TRUE);
//             if (!NT_SUCCESS(Status))
//             {
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, EaBuffer);
//                 IoStatusBlock.Information = 0;
//             }
//
//             /* If we don't fail because of too small buffer, stop here */
//             if (Status != STATUS_BUFFER_OVERFLOW &&
//                 Status != STATUS_BUFFER_TOO_SMALL)
//             {
//                 EaLength = IoStatusBlock.Information;
//                 break;
//             }
//         }
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtPathU,
//                                OBJ_CASE_INSENSITIVE,
//                                RelativeName.ContainingDirectory,
//                                (lpSecurityAttributes ? lpSecurityAttributes->lpSecurityDescriptor : NULL));
//
//     /* Ensure attributes are valid */
//     FileBasicInfo.FileAttributes &= FILE_ATTRIBUTE_VALID_FLAGS;
//
//     /* Create the new directory */
//     Status = NtCreateFile(&DirectoryHandle,
//                           FILE_LIST_DIRECTORY | SYNCHRONIZE | FILE_WRITE_ATTRIBUTES |
//                           FILE_READ_ATTRIBUTES | (FileBasicInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT ? FILE_ADD_FILE : 0),
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FileBasicInfo.FileAttributes,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           FILE_CREATE,
//                           FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
//                           FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT,
//                           EaBuffer,
//                           EaLength);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_INVALID_PARAMETER || Status == STATUS_ACCESS_DENIED)
//         {
//             /* If creation failed, it might be because FSD doesn't support reparse points
//              * Retry without asking for such support in case template is not a reparse point
//              */
//             if (!ReparsePoint)
//             {
//                 Status = NtCreateFile(&DirectoryHandle,
//                                       FILE_LIST_DIRECTORY | SYNCHRONIZE |
//                                       FILE_WRITE_ATTRIBUTES | FILE_READ_ATTRIBUTES,
//                                       &ObjectAttributes,
//                                       &IoStatusBlock,
//                                       NULL,
//                                       FileBasicInfo.FileAttributes,
//                                       FILE_SHARE_READ | FILE_SHARE_WRITE,
//                                       FILE_CREATE,
//                                       FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
//                                       FILE_OPEN_FOR_BACKUP_INTENT,
//                                       EaBuffer,
//                                       EaLength);
//             }
//             else
//             {
//                 RtlReleaseRelativeName(&RelativeName);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//                 if (EaBuffer)
//                 {
//                     RtlFreeHeap(RtlGetProcessHeap(), 0, EaBuffer);
//                 }
//                 CloseHandle(TemplateHandle);
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//         }
//     }
//
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//     if (EaBuffer)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, EaBuffer);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         NtClose(TemplateHandle);
//         if (RtlIsDosDeviceName_U(lpNewDirectory))
//         {
//             Status = STATUS_NOT_A_DIRECTORY;
//         }
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* If template is a reparse point, copy reparse data */
//     if (ReparsePoint)
//     {
//         ReparseDataBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0,
//                                             MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//         if (!ReparseDataBuffer)
//         {
//             NtClose(TemplateHandle);
//             NtClose(DirectoryHandle);
//             SetLastError(STATUS_NO_MEMORY);
//             return FALSE;
//         }
//
//         /* First query data */
//         Status = NtFsControlFile(TemplateHandle,
//                                  NULL,
//                                  NULL,
//                                  NULL,
//                                  &IoStatusBlock,
//                                  FSCTL_GET_REPARSE_POINT,
//                                  NULL,
//                                  0,
//                                  ReparseDataBuffer,
//                                  MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//         if (!NT_SUCCESS(Status))
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//             NtClose(TemplateHandle);
//             NtClose(DirectoryHandle);
//             SetLastError(Status);
//             return FALSE;
//         }
//
//         /* Once again, ensure it is a mount point */
//         if (ReparseDataBuffer->ReparseTag != IO_REPARSE_TAG_MOUNT_POINT)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//             NtClose(TemplateHandle);
//             NtClose(DirectoryHandle);
//             SetLastError(STATUS_OBJECT_NAME_INVALID);
//             return FALSE;
//         }
//
//         /* Get volume name */
//         SubstituteName = (PWSTR)((ULONG_PTR)ReparseDataBuffer->MountPointReparseBuffer.PathBuffer +
//                                  ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameOffset);
//         if (IS_VOLUME_NAME(SubstituteName, ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameLength))
//         {
//             /* Prepare to define a new mount point for that volume */
//             RtlInitUnicodeString(&NewDirectory, lpNewDirectory);
//             NewDirectory.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, NewDirectory.Length + 2 * sizeof(WCHAR));
//             if (!NewDirectory.Buffer)
//             {
//                 SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//                 NtClose(TemplateHandle);
//                 NtClose(DirectoryHandle);
//                 return FALSE;
//             }
//
//             RtlCopyMemory(&NewDirectory.Buffer, lpNewDirectory, NewDirectory.Length);
//             if (NewDirectory.Buffer[NewDirectory.Length / sizeof(WCHAR)] != L'\\')
//             {
//                 NewDirectory.Buffer[NewDirectory.Length / sizeof(WCHAR)] = L'\\';
//                 NewDirectory.Buffer[(NewDirectory.Length / sizeof(WCHAR)) + 1] = UNICODE_NULL;
//             }
//
//             /* Define a new mount point for that volume */
//             SetVolumeMountPointW(NewDirectory.Buffer, SubstituteName);
//
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NewDirectory.Buffer);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//             NtClose(TemplateHandle);
//             NtClose(DirectoryHandle);
//             return TRUE;
//         }
//
//         /* Otherwise copy data raw */
//         Status = NtFsControlFile(DirectoryHandle,
//                                  NULL,
//                                  NULL,
//                                  NULL,
//                                  &IoStatusBlock,
//                                  FSCTL_SET_REPARSE_POINT,
//                                  ReparseDataBuffer,
//                                  ReparseDataBuffer->ReparseDataLength +
//                                  FIELD_OFFSET(REPARSE_DATA_BUFFER, MountPointReparseBuffer.PathBuffer),
//                                  NULL,
//                                  0);
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//         NtClose(TemplateHandle);
//         NtClose(DirectoryHandle);
//
//         if (NT_SUCCESS(Status))
//         {
//             return TRUE;
//         }
//
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//     /* In case it's not a reparse point, handle streams on the file */
//     else
//     {
//         for (StreamSize = 0x1000; ; StreamSize = StreamSize * 2)
//         {
//             FileStreamInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, StreamSize);
//             if (!FileStreamInfo)
//             {
//                 BaseMarkFileForDelete(DirectoryHandle, FileBasicInfo.FileAttributes);
//                 SetLastError(STATUS_NO_MEMORY);
//                 break;
//             }
//
//             /* Query stream information */
//             Status = NtQueryInformationFile(TemplateHandle,
//                                             &IoStatusBlock,
//                                             FileStreamInfo,
//                                             StreamSize,
//                                             FileStreamInformation);
//             if (NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             RtlFreeHeap(RtlGetProcessHeap(), 0, FileStreamInfo);
//             FileStreamInfo = NULL;
//
//             /* If it failed, ensure that's not because of too small buffer */
//             if (Status != STATUS_BUFFER_OVERFLOW &&
//                 Status != STATUS_BUFFER_TOO_SMALL)
//             {
//                 break;
//             }
//         }
//
//         if (!NT_SUCCESS(Status) || IoStatusBlock.Information == 0)
//         {
//             if (FileStreamInfo)
//             {
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, FileStreamInfo);
//             }
//
//             NtClose(TemplateHandle);
//             NtClose(DirectoryHandle);
//             return TRUE;
//         }
//
//         /* FIXME: TODO */
//         DPRINT1("Warning: streams copying is unimplemented!\n");
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FileStreamInfo);
//         NtClose(TemplateHandle);
//         NtClose(DirectoryHandle);
//         return TRUE;
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// RemoveDirectoryA(IN LPCSTR lpPathName)
// {
//     PUNICODE_STRING PathNameW;
//
//     PathNameW = Basep8BitStringToStaticUnicodeString(lpPathName);
//     if (!PathNameW)
//     {
//         return FALSE;
//     }
//
//     return RemoveDirectoryW(PathNameW->Buffer);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// RemoveDirectoryW(IN LPCWSTR lpPathName)
// {
//     NTSTATUS Status;
//     DWORD BytesReturned;
//     HANDLE DirectoryHandle;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING NtPathU, PathName;
//     RTL_RELATIVE_NAME_U RelativeName;
//     PWSTR PathUBuffer, SubstituteName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PREPARSE_DATA_BUFFER ReparseDataBuffer;
//     FILE_DISPOSITION_INFORMATION FileDispInfo;
//     FILE_ATTRIBUTE_TAG_INFORMATION FileTagInfo;
//
//     /* Get relative name */
//     if (!RtlDosPathNameToRelativeNtPathName_U(lpPathName, &NtPathU, NULL, &RelativeName))
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
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtPathU,
//                                OBJ_CASE_INSENSITIVE,
//                                RelativeName.ContainingDirectory,
//                                NULL);
//
//     /* Try to open directory */
//     Status = NtOpenFile(&DirectoryHandle,
//                         DELETE | SYNCHRONIZE | FAILED_ACCESS_ACE_FLAG,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
//                         FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We only accept failure for reparse points not being supported */
//         if (Status != STATUS_INVALID_PARAMETER)
//         {
//             goto Cleanup;
//         }
//
//         /* Try to open, with reparse points support */
//         Status = NtOpenFile(&DirectoryHandle,
//                             DELETE | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                             FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
//                             FILE_OPEN_FOR_BACKUP_INTENT);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//
//         /* Success, mark directory */
//         goto MarkFileForDelete;
//     }
//
//     /* Get information about file (and reparse point) */
//     Status = NtQueryInformationFile(DirectoryHandle,
//                                     &IoStatusBlock,
//                                     &FileTagInfo,
//                                     sizeof(FileTagInfo),
//                                     FileAttributeTagInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         /* FSD might not support querying reparse points information */
//         if (Status != STATUS_NOT_IMPLEMENTED &&
//             Status != STATUS_INVALID_PARAMETER)
//         {
//             goto CleanupHandle;
//         }
//
//         /* If that's the case, then just delete directory */
//         goto MarkFileForDelete;
//     }
//
//     /* If that's not a reparse point, nothing more to do than just delete */
//     if (!(FileTagInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT))
//     {
//         goto MarkFileForDelete;
//     }
//
//     /* Check if that's a mount point */
//     if (FileTagInfo.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT)
//     {
//         /* It's not */
//         NtClose(DirectoryHandle);
//
//         /* So, try to reopen directory, ignoring mount point */
//         Status = NtOpenFile(&DirectoryHandle,
//                             DELETE | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                             FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
//                             FILE_OPEN_FOR_BACKUP_INTENT);
//         if (NT_SUCCESS(Status))
//         {
//             /* It succeed, we can safely delete directory (and ignore reparse point) */
//             goto MarkFileForDelete;
//         }
//
//         /* If it failed, only allow case where IO mount point was ignored */
//         if (Status != STATUS_IO_REPARSE_TAG_NOT_HANDLED)
//         {
//             goto Cleanup;
//         }
//
//         /* Reopen with reparse point support */
//         Status = NtOpenFile(&DirectoryHandle,
//                             DELETE | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                             FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
//                             FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT);
//         if (NT_SUCCESS(Status))
//         {
//             /* And mark for delete */
//             goto MarkFileForDelete;
//         }
//
//         goto Cleanup;
//     }
//
//     /* Here, we have a mount point, prepare to query information about it */
//     ReparseDataBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0,
//                                         MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//     if (!ReparseDataBuffer)
//     {
//         RtlReleaseRelativeName(&RelativeName);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//         NtClose(DirectoryHandle);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Query */
//     if (!DeviceIoControl(DirectoryHandle,
//                          FSCTL_GET_REPARSE_POINT,
//                          NULL, 0,
//                          ReparseDataBuffer,
//                          MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
//                          &BytesReturned,
//                          NULL))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//         goto MarkFileForDelete;
//     }
//
//     /* Get volume name */
//     SubstituteName = (PWSTR)((ULONG_PTR)ReparseDataBuffer->MountPointReparseBuffer.PathBuffer +
//                              ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameOffset);
//     if (!IS_VOLUME_NAME(SubstituteName, ReparseDataBuffer->MountPointReparseBuffer.SubstituteNameLength))
//     {
//         /* This is not a volume, we can safely delete */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//         goto MarkFileForDelete;
//     }
//
//     /* Prepare to delete mount point */
//     RtlInitUnicodeString(&PathName, lpPathName);
//     PathName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, PathName.Length + 2 * sizeof(WCHAR));
//     if (!PathName.Buffer)
//     {
//         RtlReleaseRelativeName(&RelativeName);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//         NtClose(DirectoryHandle);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     RtlCopyMemory(PathName.Buffer, lpPathName, PathName.Length);
//     if (PathName.Buffer[(PathName.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         PathName.Buffer[PathName.Length / sizeof(WCHAR)] = L'\\';
//         PathName.Buffer[(PathName.Length / sizeof(WCHAR)) + 1] = UNICODE_NULL;
//     }
//
//     /* Delete mount point for that volume */
//     DeleteVolumeMountPointW(PathName.Buffer);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathName.Buffer);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseDataBuffer);
//
//     /* And mark directory for delete */
// MarkFileForDelete:
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//
//     /* Mark & set */
//     FileDispInfo.DeleteFile = TRUE;
//     Status = NtSetInformationFile(DirectoryHandle,
//                                   &IoStatusBlock,
//                                   &FileDispInfo,
//                                   sizeof(FILE_DISPOSITION_INFORMATION),
//                                   FileDispositionInformation);
//     NtClose(DirectoryHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
//
// CleanupHandle:
//     NtClose(DirectoryHandle);
//
// Cleanup:
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathUBuffer);
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

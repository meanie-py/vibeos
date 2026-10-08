use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

/// `RtlCopyMemory(Destination, Source, Length)` — memcpy (non-overlapping).
#[inline]
unsafe fn RtlCopyMemory(Destination: PVOID, Source: *const core::ffi::c_void, Length: usize) {
    memcpy(Destination, Source, Length);
}
/// `RtlZeroMemory(Destination, Length)` — memset 0.
#[inline]
unsafe fn RtlZeroMemory(Destination: PVOID, Length: usize) {
    memset(Destination, 0, Length);
}
/// `RtlMoveMemory(Destination, Source, Length)` — memmove (overlapping-safe).
#[inline]
unsafe fn RtlMoveMemory(Destination: PVOID, Source: *const core::ffi::c_void, Length: usize) {
    memmove(Destination, Source, Length);
}

extern "system" {
    fn BaseSetLastNTError(Status: NTSTATUS);
    fn CloseHandle(hObject: HANDLE) -> u32;
    fn CreateFileW( lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: *mut core::ffi::c_void, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE, ) -> HANDLE;
    fn DeviceIoControl( hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: *mut core::ffi::c_void, nInBufferSize: DWORD, lpOutBuffer: *mut core::ffi::c_void, nOutBufferSize: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: *mut core::ffi::c_void, ) -> BOOL;
    fn GetLastError() -> DWORD;
    fn MOUNTMGR_IS_DOS_VOLUME_NAME_WB(VolumeName: PUNICODE_STRING) -> BOOLEAN;
    fn SetLastError(dwErrCode: DWORD);
    fn Basep8BitStringToStaticUnicodeString(lpString: LPCSTR) -> PUNICODE_STRING;
    fn Basep8BitStringToDynamicUnicodeString( DestinationString: PUNICODE_STRING, lpString: LPCSTR, ) -> u8;
    fn GetVolumeNameForVolumeMountPointW(lpszVolumeMountPoint: LPCWSTR, lpszVolumeName: LPWSTR, cchBufferLength: DWORD) -> BOOL;
    fn WideCharToMultiByte(CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: c_int, lpMultiByteStr: LPSTR, cbMultiByte: c_int, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut BOOL) -> c_int;
    fn GetDriveTypeA(lpRootPathName: LPCSTR) -> UINT;
    fn GetFullPathNameW(lpFileName: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR) -> DWORD;
}
extern "C" {
    fn BasepGetVolumeNameForVolumeMountPoint(VolumeMountPoint: LPCWSTR, VolumeName: LPWSTR, VolumeNameLength: ULONG, MountPoint: PBOOL) -> BOOL;
}

// ---- MOUNTMGR types/consts/IOCTLs (sdk/include/ddk/mountmgr.h) ----
pub const MOUNTMGR_DOS_DEVICE_NAME: &[u16] = &[
    0x5C, 0x5C, 0x2E, 0x5C, b'M' as u16, b'o' as u16, b'u' as u16, b'n' as u16, b't' as u16,
    b'P' as u16, b'o' as u16, b'i' as u16, b'n' as u16, b't' as u16, b'M' as u16, b'a' as u16,
    b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, 0,
]; // L"\\\\.\\MountPointManager"
// CTL_CODE(MOUNTMGRCONTROLTYPE='m', func, METHOD_BUFFERED, FILE_ANY_ACCESS)
//   = ('m' << 16) | (access << 14) | (func << 2) | method
pub const IOCTL_MOUNTMGR_QUERY_POINTS: DWORD = (0x6D << 16) | (2 << 2);
pub const IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS: DWORD = (0x6D << 16) | (13 << 2);

#[repr(C)]
pub struct MOUNTMGR_MOUNT_POINT {
    pub SymbolicLinkNameOffset: ULONG,
    pub SymbolicLinkNameLength: USHORT,
    pub UniqueIdOffset: ULONG,
    pub UniqueIdLength: USHORT,
    pub DeviceNameOffset: ULONG,
    pub DeviceNameLength: USHORT,
}
#[repr(C)]
pub struct MOUNTMGR_MOUNT_POINTS {
    pub Size: ULONG,
    pub NumberOfMountPoints: ULONG,
    pub MountPoints: [MOUNTMGR_MOUNT_POINT; 1],
}
#[repr(C)]
pub struct MOUNTMGR_TARGET_NAME {
    pub DeviceNameLength: USHORT,
    pub DeviceName: [WCHAR; 1],
}
#[repr(C)]
pub struct MOUNTMGR_VOLUME_PATHS {
    pub MultiSzLength: ULONG,
    pub MultiSz: [WCHAR; 1],
}

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn GetVolumeInformationA(
    lpRootPathName: LPCSTR,
    lpVolumeNameBuffer: LPSTR,
    nVolumeNameSize: DWORD,
    lpVolumeSerialNumber: LPDWORD,
    lpMaximumComponentLength: LPDWORD,
    lpFileSystemFlags: LPDWORD,
    lpFileSystemNameBuffer: LPSTR,
    nFileSystemNameSize: DWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let mut Status: NTSTATUS;
    let RootPathNameU: PUNICODE_STRING;
    let mut VolumeName: ANSI_STRING = core::mem::zeroed();
    let mut FileSystemName: ANSI_STRING = core::mem::zeroed();
    let mut VolumeNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut FileSystemNameU: _UNICODE_STRING = core::mem::zeroed();

    /* If no root path provided, default to \ */
    let lpRootPathName: LPCSTR = if lpRootPathName.is_null() {
        b"\\\0".as_ptr() as LPCSTR
    } else {
        lpRootPathName
    };

    /* Convert root path to unicode */
    RootPathNameU = Basep8BitStringToStaticUnicodeString(lpRootPathName);
    if RootPathNameU.is_null() {
        return 0;
    }

    /* Init all our STRINGS (U/A) */
    VolumeNameU.Buffer = core::ptr::null_mut();
    VolumeNameU.MaximumLength = 0;
    FileSystemNameU.Buffer = core::ptr::null_mut();
    FileSystemNameU.MaximumLength = 0;

    VolumeName.Buffer = lpVolumeNameBuffer;
    VolumeName.MaximumLength = (nVolumeNameSize + 1) as u16;
    FileSystemName.Buffer = lpFileSystemNameBuffer;
    FileSystemName.MaximumLength = (nFileSystemNameSize + 1) as u16;

    /* Assume failure for now */
    Ret = 0;

    'CleanAndQuit: {
        /* If caller wants volume name, allocate a buffer to receive it */
        if !lpVolumeNameBuffer.is_null() {
            VolumeNameU.MaximumLength = ((core::mem::size_of::<WCHAR>() as DWORD) * (nVolumeNameSize + 1)) as u16;
            VolumeNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumeNameU.MaximumLength as SIZE_T) as PWSTR;
            if VolumeNameU.Buffer.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'CleanAndQuit;
            }
        }

        /* If caller wants file system name, allocate a buffer to receive it */
        if !lpFileSystemNameBuffer.is_null() {
            FileSystemNameU.MaximumLength = ((core::mem::size_of::<WCHAR>() as DWORD) * (nFileSystemNameSize + 1)) as u16;
            FileSystemNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, FileSystemNameU.MaximumLength as SIZE_T) as PWSTR;
            if FileSystemNameU.Buffer.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'CleanAndQuit;
            }
        }

        /* Call W */
        Ret = GetVolumeInformationW(
            (*RootPathNameU).Buffer,
            VolumeNameU.Buffer,
            nVolumeNameSize,
            lpVolumeSerialNumber,
            lpMaximumComponentLength,
            lpFileSystemFlags,
            FileSystemNameU.Buffer,
            nFileSystemNameSize,
        );

        /* If it succeed, convert back to ANSI */
        if Ret != 0 {
            if !lpVolumeNameBuffer.is_null() {
                RtlInitUnicodeString(&mut VolumeNameU, VolumeNameU.Buffer);
                Status = RtlUnicodeStringToAnsiString(&mut VolumeName, &VolumeNameU, 0);
                if !NT_SUCCESS(Status) {
                    BaseSetLastNTError(Status);
                    Ret = 0;
                    break 'CleanAndQuit;
                }
            }

            if !lpFileSystemNameBuffer.is_null() {
                RtlInitUnicodeString(&mut FileSystemNameU, FileSystemNameU.Buffer);
                Status = RtlUnicodeStringToAnsiString(&mut FileSystemName, &FileSystemNameU, 0);
                if !NT_SUCCESS(Status) {
                    BaseSetLastNTError(Status);
                    Ret = 0;
                    break 'CleanAndQuit;
                }
            }
        }
    }

    /* Clean and quit */
    if !VolumeNameU.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameU.Buffer as PVOID);
    }

    if !FileSystemNameU.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileSystemNameU.Buffer as PVOID);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn IsThisARootDirectory(
    VolumeHandle: HANDLE,
    NtPathName: PUNICODE_STRING,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileNameInfoData: FileNameInfoBuffer = core::mem::zeroed();

    /* If we have a handle, query the name */
    if !VolumeHandle.is_null() {
        Status = NtQueryInformationFile(
            VolumeHandle,
            &mut IoStatusBlock,
            &mut FileNameInfoData as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FileNameInfoBuffer>() as u32,
            FileNameInformation,
        );
        if !NT_SUCCESS(Status) {
            return 0;
        }

        /* Check we properly end with a \ */
        if FileNameInfoData.FileName[FileNameInfoData.FileNameLength as usize / core::mem::size_of::<WCHAR>() - 1] != '\\' as u16
        {
            return 0;
        }
    }

    /* If we have a path */
    if !NtPathName.is_null() {
        let mut LinkHandle: HANDLE = core::ptr::null_mut();
        let mut Buffer: [WCHAR; 512] = [0; 512];
        let mut ReturnedLength: ULONG = 0;
        let mut LinkTarget: _UNICODE_STRING = core::mem::zeroed();
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

        (*NtPathName).Length -= core::mem::size_of::<WCHAR>() as u16;

        InitializeObjectAttributes(
            &mut ObjectAttributes,
            NtPathName,
            OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        /* Try to see whether that's a symbolic name */
        Status = NtOpenSymbolicLinkObject(
            &mut LinkHandle,
            SYMBOLIC_LINK_QUERY,
            &mut ObjectAttributes,
        );
        (*NtPathName).Length += core::mem::size_of::<WCHAR>() as u16;
        if !NT_SUCCESS(Status) {
            return 0;
        }

        /* If so, query the target */
        LinkTarget.Buffer = Buffer.as_mut_ptr();
        LinkTarget.Length = 0;
        LinkTarget.MaximumLength = core::mem::size_of::<[WCHAR; 512]>() as u16;

        Status = NtQuerySymbolicLinkObject(LinkHandle, &mut LinkTarget, &mut ReturnedLength);
        NtClose(LinkHandle);
        /* A root directory (NtName) is a symbolic link */
        if !NT_SUCCESS(Status) {
            return 0;
        }
    }

    return 1;
}

// Helper struct for the anonymous struct in C:
//   struct { FILE_NAME_INFORMATION; WCHAR Buffer[MAX_PATH]; }
// FILE_NAME_INFORMATION = { ULONG FileNameLength; WCHAR FileName[1]; }, so the
// trailing Buffer just extends FileName. Flatten so .FileNameLength/.FileName work.
#[repr(C)]
struct FileNameInfoBuffer {
    FileNameLength: ULONG,
    FileName: [WCHAR; MAX_PATH as usize + 1],
}

// Helper function to expand InitializeObjectAttributes macro
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: PUNICODE_STRING,
    a: u32,
    r: HANDLE,
    s: *mut core::ffi::c_void,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn GetVolumeInformationW(
    lpRootPathName: LPCWSTR,
    lpVolumeNameBuffer: LPWSTR,
    mut nVolumeNameSize: DWORD,
    lpVolumeSerialNumber: LPDWORD,
    lpMaximumComponentLength: LPDWORD,
    lpFileSystemFlags: LPDWORD,
    lpFileSystemNameBuffer: LPWSTR,
    mut nFileSystemNameSize: DWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let mut Status: NTSTATUS;
    let mut VolumeHandle: HANDLE = core::ptr::null_mut();
    let mut RootPathName: LPCWSTR;
    let mut NtPathName: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut VolumeInfo: PFILE_FS_VOLUME_INFORMATION;
    let mut VolumeAttr: PFILE_FS_ATTRIBUTE_INFORMATION;
    let mut OldMode: ULONG = 0;
    let mut VolumeInfoSize: ULONG;
    let mut VolumeAttrSize: ULONG;

    /* If no root path provided, default to \ */
    if lpRootPathName.is_null() {
        RootPathName = w!("\\\0").as_ptr() as LPCWSTR;
    } else {
        RootPathName = lpRootPathName;
    }

    /* Convert length to bytes */
    nVolumeNameSize *= core::mem::size_of::<WCHAR>() as DWORD;
    nFileSystemNameSize *= core::mem::size_of::<WCHAR>() as DWORD;

    /* Convert to NT name */
    if RtlDosPathNameToNtPathName_U(RootPathName, &mut NtPathName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    /* Check we really end with a backslash */
    if *(NtPathName.Buffer.offset((NtPathName.Length as usize / core::mem::size_of::<WCHAR>()) as isize - 1)) != '\\' as u16 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
        return 0;
    }

    /* Try to open the received path */
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtPathName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    /* No errors to the user */
    RtlSetThreadErrorMode(RTL_SEM_FAILCRITICALERRORS, &mut OldMode);
    Status = NtOpenFile(
        &mut VolumeHandle,
        SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        0,
        FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT,
    );
    RtlSetThreadErrorMode(OldMode, core::ptr::null_mut());
    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        BaseSetLastNTError(Status);
        return 0;
    }

    /* Check whether that's a root directory */
    if IsThisARootDirectory(VolumeHandle, &mut NtPathName) == 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        NtClose(VolumeHandle);
        SetLastError(ERROR_DIR_NOT_ROOT);
        return 0;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);

    /* Assume we don't need to query FileFsVolumeInformation */
    VolumeInfo = core::ptr::null_mut();
    /* If user wants volume name, allocate a buffer to query it */
    if !lpVolumeNameBuffer.is_null() {
        VolumeInfoSize = nVolumeNameSize + core::mem::size_of::<FILE_FS_VOLUME_INFORMATION>() as ULONG;
    }
    /* If user just wants the serial number, allocate a dummy buffer */
    else if !lpVolumeSerialNumber.is_null() {
        VolumeInfoSize = (MAX_PATH as ULONG) * core::mem::size_of::<WCHAR>() as ULONG + core::mem::size_of::<FILE_FS_VOLUME_INFORMATION>() as ULONG;
    }
    /* Otherwise, nothing to query */
    else {
        VolumeInfoSize = 0;
    }

    /* If we're to query, allocate a big enough buffer */
    if VolumeInfoSize != 0 {
        VolumeInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, (VolumeInfoSize) as SIZE_T) as PFILE_FS_VOLUME_INFORMATION;
        if VolumeInfo.is_null() {
            NtClose(VolumeHandle);
            BaseSetLastNTError(STATUS_NO_MEMORY);
            return 0;
        }
    }

    /* Assume we don't need to query FileFsAttributeInformation */
    VolumeAttr = core::ptr::null_mut();
    /* If user wants filesystem name, allocate a buffer to query it */
    if !lpFileSystemNameBuffer.is_null() {
        VolumeAttrSize = nFileSystemNameSize + core::mem::size_of::<FILE_FS_ATTRIBUTE_INFORMATION>() as ULONG;
    }
    /* If user just wants max compo len or flags, allocate a dummy buffer */
    else if !lpMaximumComponentLength.is_null() || !lpFileSystemFlags.is_null() {
        VolumeAttrSize = (MAX_PATH as ULONG) * core::mem::size_of::<WCHAR>() as ULONG + core::mem::size_of::<FILE_FS_ATTRIBUTE_INFORMATION>() as ULONG;
    } else {
        VolumeAttrSize = 0;
    }

    /* If we're to query, allocate a big enough buffer */
    if VolumeAttrSize != 0 {
        VolumeAttr = RtlAllocateHeap(RtlGetProcessHeap(), 0, (VolumeAttrSize) as SIZE_T) as PFILE_FS_ATTRIBUTE_INFORMATION;
        if VolumeAttr.is_null() {
            if !VolumeInfo.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeInfo as *mut core::ffi::c_void);
            }

            NtClose(VolumeHandle);
            BaseSetLastNTError(STATUS_NO_MEMORY);
            return 0;
        }
    }

    /* Assume we'll fail */
    Ret = 0;

    'CleanAndQuit: {
        /* If we're to query FileFsVolumeInformation, do it now! */
        if !VolumeInfo.is_null() {
            Status = NtQueryVolumeInformationFile(
                VolumeHandle,
                &mut IoStatusBlock,
                VolumeInfo as *mut core::ffi::c_void,
                VolumeInfoSize,
                FileFsVolumeInformation,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                break 'CleanAndQuit;
            }
        }

        /* If we're to query FileFsAttributeInformation, do it now! */
        if !VolumeAttr.is_null() {
            Status = NtQueryVolumeInformationFile(
                VolumeHandle,
                &mut IoStatusBlock,
                VolumeAttr as *mut core::ffi::c_void,
                VolumeAttrSize,
                FileFsAttributeInformation,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                break 'CleanAndQuit;
            }
        }

        /* If user wants volume name */
        if !lpVolumeNameBuffer.is_null() {
            /* Check its buffer can hold it (+ 0) */
            if (*VolumeInfo).VolumeLabelLength >= nVolumeNameSize {
                SetLastError(ERROR_BAD_LENGTH);
                break 'CleanAndQuit;
            }

            /* Copy and zero */
            RtlCopyMemory(
                lpVolumeNameBuffer as *mut core::ffi::c_void,
                (*VolumeInfo).VolumeLabel.as_ptr() as *const core::ffi::c_void,
                (*VolumeInfo).VolumeLabelLength as usize,
            );
            *lpVolumeNameBuffer.offset((*VolumeInfo).VolumeLabelLength as isize / core::mem::size_of::<WCHAR>() as isize) = 0;
        }

        /* If user wants wants serial number, return it */
        if !lpVolumeSerialNumber.is_null() {
            *lpVolumeSerialNumber = (*VolumeInfo).VolumeSerialNumber;
        }

        /* If user wants filesystem name */
        if !lpFileSystemNameBuffer.is_null() {
            /* Check its buffer can hold it (+ 0) */
            if (*VolumeAttr).FileSystemNameLength >= nFileSystemNameSize {
                SetLastError(ERROR_BAD_LENGTH);
                break 'CleanAndQuit;
            }

            /* Copy and zero */
            RtlCopyMemory(
                lpFileSystemNameBuffer as *mut core::ffi::c_void,
                (*VolumeAttr).FileSystemName.as_ptr() as *const core::ffi::c_void,
                (*VolumeAttr).FileSystemNameLength as usize,
            );
            *lpFileSystemNameBuffer.offset((*VolumeAttr).FileSystemNameLength as isize / core::mem::size_of::<WCHAR>() as isize) = 0;
        }

        /* If user wants wants max compo len, return it */
        if !lpMaximumComponentLength.is_null() {
            *lpMaximumComponentLength = (*VolumeAttr).MaximumComponentNameLength;
        }

        /* If user wants wants FS flags, return them */
        if !lpFileSystemFlags.is_null() {
            *lpFileSystemFlags = (*VolumeAttr).FileSystemAttributes;
        }

        /* We did it! */
        Ret = 1;
    }

    NtClose(VolumeHandle);

    if !VolumeInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeInfo as *mut core::ffi::c_void);
    }

    if !VolumeAttr.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeAttr as *mut core::ffi::c_void);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn SetVolumeLabelA(
    lpRootPathName: LPCSTR,
    lpVolumeName: LPCSTR,
) -> BOOL {
    let mut Ret: BOOL;
    let mut VolumeNameU: _UNICODE_STRING = core::mem::zeroed();
    let RootPathNameU: PUNICODE_STRING;

    let lpRootPathName: LPCSTR = if lpRootPathName.is_null() {
        b"\\\0".as_ptr() as LPCSTR
    } else {
        lpRootPathName
    };

    RootPathNameU = Basep8BitStringToStaticUnicodeString(lpRootPathName);
    if RootPathNameU.is_null() {
        return 0;
    }

    if !lpVolumeName.is_null() {
        if Basep8BitStringToDynamicUnicodeString(&mut VolumeNameU, lpVolumeName) == 0 {
            return 0;
        }
    } else {
        VolumeNameU.Buffer = core::ptr::null_mut();
    }

    Ret = SetVolumeLabelW((*RootPathNameU).Buffer, VolumeNameU.Buffer);
    RtlFreeUnicodeString(&mut VolumeNameU);
    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn SetVolumeLabelW(
    lpRootPathName: LPCWSTR,
    lpVolumeName: LPCWSTR,
) -> BOOL {
    let mut Ret: BOOL;
    let mut Status: NTSTATUS;
    let mut VolumeRoot: PWSTR;
    let mut VolumeHandle: HANDLE = core::ptr::null_mut();
    let mut VolumeGuid: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FsLabelInfo: PFILE_FS_LABEL_INFORMATION;
    let mut VolumeName: _UNICODE_STRING = core::mem::zeroed();
    let mut NtVolumeName: _UNICODE_STRING = core::mem::zeroed();

    VolumeRoot = w!("\\").as_ptr() as PWSTR;

    if !lpVolumeName.is_null() {
        RtlInitUnicodeString(&mut VolumeName, lpVolumeName);
    } else {
        VolumeName.Length = 0;
        VolumeName.MaximumLength = 0;
        VolumeName.Buffer = core::ptr::null_mut();
    }

    if !lpRootPathName.is_null() {
        Ret = GetVolumeNameForVolumeMountPointW(lpRootPathName, VolumeGuid.as_mut_ptr(), MAX_PATH);
    } else {
        Ret = 0;
    }

    if Ret != 0 {
        VolumeRoot = VolumeGuid.as_mut_ptr();
    } else {
        if !lpRootPathName.is_null() {
            VolumeRoot = lpRootPathName as PWSTR;
        }
    }

    if RtlDosPathNameToNtPathName_U(VolumeRoot, &mut NtVolumeName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    if *NtVolumeName.Buffer.offset((NtVolumeName.Length / core::mem::size_of::<WCHAR>() as u16).wrapping_sub(1) as isize) != 0x5C_u16 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer as PVOID);
        BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
        return 0;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtVolumeName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtOpenFile(
        &mut VolumeHandle,
        SYNCHRONIZE | FILE_WRITE_DATA,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );

    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer as PVOID);
        BaseSetLastNTError(Status);
        return 0;
    }

    if IsThisARootDirectory(VolumeHandle, core::ptr::null_mut()) == 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer as PVOID);
        NtClose(VolumeHandle);
        SetLastError(ERROR_DIR_NOT_ROOT);
        return 0;
    }

    NtClose(VolumeHandle);

    NtVolumeName.Length = NtVolumeName.Length.wrapping_sub(core::mem::size_of::<WCHAR>() as u16);

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtVolumeName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtOpenFile(
        &mut VolumeHandle,
        SYNCHRONIZE | FILE_WRITE_DATA,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer as PVOID);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    Ret = 1;

    FsLabelInfo = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (core::mem::size_of::<FILE_FS_LABEL_INFORMATION>() + VolumeName.Length as usize) as SIZE_T,
    ) as PFILE_FS_LABEL_INFORMATION;

    if !FsLabelInfo.is_null() {
        RtlCopyMemory(
            (*FsLabelInfo).VolumeLabel.as_mut_ptr() as PVOID,
            VolumeName.Buffer as PVOID,
            VolumeName.Length as usize,
        );
        (*FsLabelInfo).VolumeLabelLength = VolumeName.Length as u32;

        Status = NtSetVolumeInformationFile(
            VolumeHandle,
            &mut IoStatusBlock,
            FsLabelInfo as PVOID,
            (core::mem::size_of::<FILE_FS_LABEL_INFORMATION>() + VolumeName.Length as usize) as u32,
            FileFsLabelInformation,
        );
    } else {
        Status = STATUS_NO_MEMORY;
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        Ret = 0;
    }

    NtClose(VolumeHandle);

    if !FsLabelInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FsLabelInfo as PVOID);
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstVolumeW(
    volume: LPWSTR,
    len: DWORD,
) -> HANDLE {
    let mut size: DWORD = 1024;
    let mut br: DWORD = 0;
    let mgr: HANDLE = CreateFileW(
        MOUNTMGR_DOS_DEVICE_NAME.as_ptr(),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        INVALID_HANDLE_VALUE,
    );
    if mgr == INVALID_HANDLE_VALUE {
        return INVALID_HANDLE_VALUE;
    }

    loop {
        let mut input: MOUNTMGR_MOUNT_POINT = core::mem::zeroed();
        let output: *mut MOUNTMGR_MOUNT_POINTS;

        output = RtlAllocateHeap(RtlGetProcessHeap(), 0, size as SIZE_T) as *mut MOUNTMGR_MOUNT_POINTS;
        if output.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break;
        }
        core::ptr::write_bytes(&mut input as *mut MOUNTMGR_MOUNT_POINT, 0, 1);

        if DeviceIoControl(
            mgr,
            IOCTL_MOUNTMGR_QUERY_POINTS,
            &mut input as *mut MOUNTMGR_MOUNT_POINT as *mut core::ffi::c_void,
            core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as DWORD,
            output as *mut core::ffi::c_void,
            size,
            &mut br as *mut DWORD,
            core::ptr::null_mut(),
        ) == 0
        {
            if GetLastError() != ERROR_MORE_DATA {
                break;
            }
            size = (*output).Size;
            RtlFreeHeap(RtlGetProcessHeap(), 0, output as *mut core::ffi::c_void);
            continue;
        }
        CloseHandle(mgr);
        /* abuse the Size field to store the current index */
        (*output).Size = 0;
        if FindNextVolumeW(output as HANDLE, volume, len) == 0 {
            RtlFreeHeap(RtlGetProcessHeap(), 0, output as *mut core::ffi::c_void);
            return INVALID_HANDLE_VALUE;
        }
        return output as HANDLE;
    }
    CloseHandle(mgr);
    INVALID_HANDLE_VALUE
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstVolumeA(
    volume: LPSTR,
    len: DWORD,
) -> HANDLE {
    let buffer: *mut WCHAR;
    let mut handle: HANDLE;

    buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;

    if buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return INVALID_HANDLE_VALUE;
    }

    handle = FindFirstVolumeW(buffer, len);

    if handle != INVALID_HANDLE_VALUE {
        if WideCharToMultiByte(
            CP_ACP,
            0,
            buffer,
            -1,
            volume,
            len as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            FindVolumeClose(handle);
            handle = INVALID_HANDLE_VALUE;
        }
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, buffer as *mut core::ffi::c_void);
    return handle;
}
#[no_mangle]
pub unsafe extern "system" fn FindVolumeClose(hFindVolume: HANDLE) -> BOOL {
    return RtlFreeHeap(RtlGetProcessHeap(), 0, hFindVolume) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetVolumePathNameA(
    lpszFileName: LPCSTR,
    lpszVolumePathName: LPSTR,
    cchBufferLength: DWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let FileNameU: PUNICODE_STRING;
    let mut VolumePathName: ANSI_STRING = core::mem::zeroed();
    let mut VolumePathNameU: _UNICODE_STRING = core::mem::zeroed();

    /* Convert file name to unicode */
    FileNameU = Basep8BitStringToStaticUnicodeString(lpszFileName);
    if FileNameU.is_null() {
        return 0;
    }

    /* Initialize all the strings we'll need */
    VolumePathName.Buffer = lpszVolumePathName;
    VolumePathName.Length = 0;
    VolumePathName.MaximumLength = (cchBufferLength - 1) as u16;

    VolumePathNameU.Length = 0;
    VolumePathNameU.MaximumLength = ((cchBufferLength - 1) as usize * core::mem::size_of::<WCHAR>() + core::mem::size_of::<WCHAR>()) as u16;
    /* Allocate a buffer for calling the -W */
    VolumePathNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumePathNameU.MaximumLength as SIZE_T) as PWSTR;
    if VolumePathNameU.Buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* Call the -W implementation */
    Ret = GetVolumePathNameW((*FileNameU).Buffer, VolumePathNameU.Buffer, cchBufferLength);
    /* If it succeed */
    if Ret != 0 {
        let Status: NTSTATUS;

        /* Convert back to ANSI */
        RtlInitUnicodeString(&mut VolumePathNameU, VolumePathNameU.Buffer);
        Status = RtlUnicodeStringToAnsiString(&mut VolumePathName, &VolumePathNameU, 0);
        /* If conversion failed, just set error code and fail the rest */
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Ret = 0;
        }
        /* Otherwise, null terminate the string (it's OK, we computed -1) */
        else {
            *VolumePathName.Buffer.offset(VolumePathName.Length as isize) = 0i8;
        }
    }

    /* Free the buffer allocated for -W call */
    RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNameU.Buffer as PVOID);
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetVolumePathNameW(
    lpszFileName: LPCWSTR,
    lpszVolumePathName: LPWSTR,
    cchBufferLength: DWORD,
) -> BOOL {
    let mut MountPoint: BOOL = 0;
    let mut FullPathLen: DWORD;
    let mut OldFilePart: WCHAR;
    let mut FullPath: _UNICODE_STRING = core::mem::zeroed();
    let mut FullPathBuf: PWSTR;
    let mut FilePart: PWSTR = core::ptr::null_mut();
    let mut VolumeNameBuf: PWSTR;

    /* Probe for full path len */
    FullPathLen = GetFullPathNameW(lpszFileName, 0, core::ptr::null_mut(), core::ptr::null_mut());
    if FullPathLen == 0 {
        return 0;
    }

    /* Allocate a big enough buffer to receive it */
    FullPathBuf = RtlAllocateHeap(RtlGetProcessHeap(), 0, ((FullPathLen as usize + 10) * core::mem::size_of::<WCHAR>()) as SIZE_T) as PWSTR;
    if FullPathBuf.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* And get full path name */
    if GetFullPathNameW(lpszFileName, FullPathLen + 10, FullPathBuf, &mut FilePart) == 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf as PVOID);
        return 0;
    }

    /* Make a string out of it */
    RtlInitUnicodeString(&mut FullPath, FullPathBuf);
    /* We will finish our string with '\', for ease of the parsing after */
    if *FullPath.Buffer.offset(((FullPath.Length as usize / core::mem::size_of::<WCHAR>()) - 1) as isize) != '\\' as u16 {
        FullPath.Length += core::mem::size_of::<WCHAR>() as u16;
        *FullPath.Buffer.offset(((FullPath.Length as usize / core::mem::size_of::<WCHAR>()) - 1) as isize) = '\\' as u16;
        *FullPath.Buffer.offset((FullPath.Length as usize / core::mem::size_of::<WCHAR>()) as isize) = 0;
    }

    /* Allocate a buffer big enough to receive our volume name */
    VolumeNameBuf = RtlAllocateHeap(RtlGetProcessHeap(), 0, (0x2000 * core::mem::size_of::<WCHAR>()) as SIZE_T) as PWSTR;
    if VolumeNameBuf.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf as PVOID);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* We don't care about file part: we added an extra backslash, so there's no
     * file, we're back at the dir level.
     * We'll recompute file part afterwards
     */
    FilePart = core::ptr::null_mut();
    /* Keep track of the letter we could drop to shorten the string */
    OldFilePart = 0;
    /* As long as querying volume name fails, keep looping */
    while BasepGetVolumeNameForVolumeMountPoint(FullPath.Buffer, VolumeNameBuf, 0x2000u32, &mut MountPoint) == 0 {
        let mut LastSlash: USHORT;

        /* Not a mount point, but opening returning access denied? Assume it's one, just not
         * a reparse backed one (classic mount point, a device)!
         */
        if MountPoint == 0 && GetLastError() == ERROR_ACCESS_DENIED {
            MountPoint = 1;
        }

        /* BasepGetVolumeNameForVolumeMountPoint failed, but returned a volume name.
         * This can happen when we are given a reparse point where MountMgr could find associated
         * volume name which is not a valid DOS volume
         * A valid DOS name always starts with \\
         */
        if *VolumeNameBuf.offset(0) != 0 && (*FullPath.Buffer.offset(0) != '\\' as u16 || *FullPath.Buffer.offset(1) != '\\' as u16) {
            let mut RootPathName: [CHAR; 4] = [0; 4];

            /* Construct a simple <letter>:\ string to get drive type */
            RootPathName[0] = *FullPath.Buffer.offset(0) as CHAR;
            RootPathName[1] = ':' as CHAR;
            RootPathName[2] = '\\' as CHAR;
            RootPathName[3] = 0;

            /* If we weren't given a drive letter actually, or if that's not a remote drive
             * Note: in this code path, we're recursive and stop fail loop
             */
            if *FullPath.Buffer.offset(1) != ':' as u16 || GetDriveTypeA(RootPathName.as_ptr()) != DRIVE_REMOTE {
                let Ret: BOOL;

                /* We won't need the full path, we'll now work with the returned volume name */
                RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf as PVOID);
                /* If it wasn't an NT name which was returned */
                if *VolumeNameBuf.offset(0) != '\\' as u16 || *VolumeNameBuf.offset(1) != '?' as u16 ||
                    *VolumeNameBuf.offset(2) != '?' as u16 || *VolumeNameBuf.offset(3) != '\\' as u16 {
                    let GlobalPath: PWSTR;
                    let mut GlobalRoot: _UNICODE_STRING = core::mem::zeroed();

                    /* Create a new name in the NT namespace (from Win32) */
                    RtlInitUnicodeString(&mut FullPath, VolumeNameBuf);
                    RtlInitUnicodeString(&mut GlobalRoot, w!("\\\\?\\GLOBALROOT\0").as_ptr() as LPCWSTR);

                    /* We allocate a buffer than can contain both the namespace and the volume name */
                    GlobalPath = RtlAllocateHeap(RtlGetProcessHeap(), 0, (FullPath.Length + GlobalRoot.Length) as SIZE_T) as PWSTR;
                    if GlobalPath.is_null() {
                        RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameBuf as PVOID);
                        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                        return 0;
                    }

                    /* Fill in the new query name */
                    RtlCopyMemory(GlobalPath as PVOID, GlobalRoot.Buffer as PVOID, GlobalRoot.Length as usize);
                    RtlCopyMemory((GlobalPath as ULONG_PTR + GlobalRoot.Length as ULONG_PTR) as PVOID, FullPath.Buffer as PVOID, FullPath.Length as usize);
                    *GlobalPath.offset((FullPath.Length + GlobalRoot.Length) as isize / core::mem::size_of::<WCHAR>() as isize) = 0;

                    /* Give it another try */
                    Ret = GetVolumePathNameW(GlobalPath, lpszVolumePathName, cchBufferLength);

                    RtlFreeHeap(RtlGetProcessHeap(), 0, GlobalPath as PVOID);
                } else {
                    /* If we don't have a drive letter in the Win32 name space \\.\<letter>: */
                    if *VolumeNameBuf.offset(4) != 0 && *VolumeNameBuf.offset(5) != ':' as u16 {
                        /* Shit our starting \\ */
                        RtlInitUnicodeString(&mut FullPath, VolumeNameBuf);
                        RtlMoveMemory(
                            VolumeNameBuf as PVOID,
                            (VolumeNameBuf as ULONG_PTR + (2 * core::mem::size_of::<WCHAR>()) as ULONG_PTR) as PVOID,
                            (FullPath.Length - (3 * core::mem::size_of::<WCHAR>()) as u16) as usize,
                        );
                    }
                    /* Otherwise, just make sure we're double \ at the being to query again with the
                     * proper namespace
                     */
                    else {
                        *VolumeNameBuf.offset(1) = '\\' as u16;
                    }

                    /* Give it another try */
                    Ret = GetVolumePathNameW(VolumeNameBuf, lpszVolumePathName, cchBufferLength);
                }

                /* And done! */
                RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameBuf as PVOID);
                return Ret;
            }
        }

        /* No mount point but with a file part? Restore filepart and exit */
        if MountPoint == 0 && !FilePart.is_null() {
            *FilePart.offset(0) = OldFilePart;
            RtlInitUnicodeString(&mut FullPath, FullPathBuf);
            break;
        }

        /* We cannot go down the path any longer, too small */
        if FullPath.Length <= core::mem::size_of::<WCHAR>() as u16 {
            break;
        }

        /* Prepare the next split */
        LastSlash = (FullPath.Length / core::mem::size_of::<WCHAR>() as u16) - 2;
        if FullPath.Length / core::mem::size_of::<WCHAR>() as u16 != 2 {
            loop {
                if *FullPath.Buffer.offset(LastSlash as isize) == '\\' as u16 {
                    break;
                }

                LastSlash -= 1;
                if LastSlash == 0 {
                    break;
                }
            }
        }

        /* We couldn't split path, quit */
        if LastSlash == 0 {
            break;
        }

        /* If that's a mount point, keep track of the directory name */
        if MountPoint != 0 {
            FilePart = FullPath.Buffer.offset(LastSlash as isize + 1);
            OldFilePart = *FilePart.offset(0);
            /* And null terminate the string */
            *FilePart.offset(0) = 0;
        }
        /* Otherwise, just null terminate the string */
        else {
            *FullPath.Buffer.offset(LastSlash as isize + 1) = 0;
        }

        /* We went down a bit in the path, fix the string and retry */
        RtlInitUnicodeString(&mut FullPath, FullPathBuf);
    }

    /* Once here, we'll return something from the full path buffer, so release
     * output buffer
     */
    RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameBuf as PVOID);

    /* Not a mount point, bail out */
    if MountPoint == 0 && FilePart.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf as PVOID);
        return 0;
    }

    /* Make sure we have enough room to copy our volume */
    if (cchBufferLength as usize * core::mem::size_of::<WCHAR>()) < FullPath.Length as usize + core::mem::size_of::<u16>() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf as PVOID);
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
        return 0;
    }

    /* Copy and null terminate */
    RtlCopyMemory(lpszVolumePathName as PVOID, FullPath.Buffer as PVOID, FullPath.Length as usize);
    *lpszVolumePathName.offset(FullPath.Length as isize / core::mem::size_of::<WCHAR>() as isize) = 0;

    RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf as PVOID);

    /* Done! */
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn FindNextVolumeA(
    handle: HANDLE,
    volume: LPSTR,
    len: DWORD,
) -> BOOL {
    let buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;
    let mut ret: BOOL;

    if buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    ret = FindNextVolumeW(handle, buffer, len);
    if ret != 0 {
        if WideCharToMultiByte(
            CP_ACP,
            0,
            buffer,
            -1,
            volume,
            len as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            ret = 0;
        }
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, buffer as *mut _);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn FindNextVolumeW(
    handle: HANDLE,
    volume: LPWSTR,
    len: DWORD,
) -> BOOL {
    let data: *mut MOUNTMGR_MOUNT_POINTS = handle as *mut MOUNTMGR_MOUNT_POINTS;

    while (*data).Size < (*data).NumberOfMountPoints {
        static volumeW: [WCHAR; 11] = [
            b'\\' as WCHAR,
            b'?' as WCHAR,
            b'?' as WCHAR,
            b'\\' as WCHAR,
            b'V' as WCHAR,
            b'o' as WCHAR,
            b'l' as WCHAR,
            b'u' as WCHAR,
            b'm' as WCHAR,
            b'e' as WCHAR,
            b'{' as WCHAR,
        ];
        let link: *mut WCHAR = (data as *mut u8)
            .offset((*core::ptr::addr_of!((*data).MountPoints[0]).add((*data).Size as usize)).SymbolicLinkNameOffset as isize)
            as *mut WCHAR;
        let size: DWORD = (*core::ptr::addr_of!((*data).MountPoints[0]).add((*data).Size as usize)).SymbolicLinkNameLength as DWORD;
        (*data).Size += 1;
        /* skip non-volumes */
        if size < core::mem::size_of::<[WCHAR; 11]>() as DWORD
            || memcmp(
                link as *const core::ffi::c_void,
                volumeW.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of::<[WCHAR; 11]>(),
            ) != 0
        {
            continue;
        }
        if size + core::mem::size_of::<WCHAR>() as DWORD >= len * core::mem::size_of::<WCHAR>() as DWORD
        {
            SetLastError(ERROR_FILENAME_EXCED_RANGE);
            return 0;
        }
        memcpy(
            volume as *mut core::ffi::c_void,
            link as *const core::ffi::c_void,
            size as usize,
        );
        *volume.offset(1) = b'\\' as WCHAR; /* map \??\ to \\?\ */
        *volume.offset((size / core::mem::size_of::<WCHAR>() as DWORD) as isize) = b'\\' as WCHAR; /* Windows appends a backslash */
        *volume.offset((size / core::mem::size_of::<WCHAR>() as DWORD + 1) as isize) = 0;
        return 1;
    }
    SetLastError(ERROR_NO_MORE_FILES);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetVolumePathNamesForVolumeNameA(
    lpszVolumeName: LPCSTR,
    lpszVolumePathNames: LPSTR,
    cchBufferLength: DWORD,
    lpcchReturnLength: PDWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let Status: NTSTATUS;
    let mut cchReturnLength: DWORD = 0;
    let mut VolumePathName: ANSI_STRING = core::mem::zeroed();
    let VolumeNameU: PUNICODE_STRING;
    let mut VolumePathNamesU: _UNICODE_STRING = core::mem::zeroed();

    /* Convert volume name to unicode */
    VolumeNameU = Basep8BitStringToStaticUnicodeString(lpszVolumeName);
    if VolumeNameU.is_null() {
        return 0;
    }

    /* Initialize the strings we'll use later on */
    VolumePathName.Length = 0;
    VolumePathName.MaximumLength = cchBufferLength as u16;
    VolumePathName.Buffer = lpszVolumePathNames;

    VolumePathNamesU.Length = 0;
    VolumePathNamesU.MaximumLength = (core::mem::size_of::<WCHAR>() as DWORD).wrapping_mul(cchBufferLength) as u16;
    /* If caller provided a non 0 sized string, allocate a buffer for our unicode string */
    if VolumePathNamesU.MaximumLength != 0 {
        VolumePathNamesU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.MaximumLength as SIZE_T) as *mut WCHAR;
        if VolumePathNamesU.Buffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    } else {
        VolumePathNamesU.Buffer = core::ptr::null_mut();
    }

    /* Call the -W implementation */
    Ret = GetVolumePathNamesForVolumeNameW(
        (*VolumeNameU).Buffer,
        VolumePathNamesU.Buffer,
        cchBufferLength,
        &mut cchReturnLength,
    );
    /* Call succeed, we'll return the total length */
    if Ret != 0 {
        VolumePathNamesU.Length = (core::mem::size_of::<WCHAR>() as DWORD).wrapping_mul(cchReturnLength) as u16;
    } else {
        /* Else, if we fail for anything else than too small buffer, quit */
        if GetLastError() != ERROR_MORE_DATA {
            if !VolumePathNamesU.Buffer.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.Buffer as *mut core::ffi::c_void);
            }

            return 0;
        }

        /* Otherwise, we'll just copy as much as we can */
        VolumePathNamesU.Length = (core::mem::size_of::<WCHAR>() as DWORD).wrapping_mul(cchBufferLength) as u16;
    }

    /* Convert our output string back to ANSI */
    Status = RtlUnicodeStringToAnsiString(&mut VolumePathName, &VolumePathNamesU, 0);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);

        if !VolumePathNamesU.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.Buffer as *mut core::ffi::c_void);
        }

        return 0;
    }

    /* If caller wants return length, two cases... */
    if !lpcchReturnLength.is_null() {
        /* We succeed: return the copied length */
        if Ret != 0 {
            *lpcchReturnLength = VolumePathName.Length as DWORD;
        }
        /* We failed, return the size we would have loved having! */
        else {
            *lpcchReturnLength = (core::mem::size_of::<WCHAR>() as DWORD).wrapping_mul(cchReturnLength);
        }
    }

    /* Release our buffer if allocated */
    if !VolumePathNamesU.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.Buffer as *mut core::ffi::c_void);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetVolumePathNamesForVolumeNameW(
    lpszVolumeName: LPCWSTR,
    lpszVolumePathNames: LPWSTR,
    cchBufferLength: DWORD,
    lpcchReturnLength: PDWORD,
) -> BOOL {

    let mut Ret: BOOL;
    let mut MultiSz: PWSTR;
    let mut BytesReturned: DWORD = 0;
    let mut MountMgrHandle: HANDLE;
    let mut VolumeName: _UNICODE_STRING = core::mem::zeroed();
    let mut TargetName: *mut MOUNTMGR_TARGET_NAME;
    let mut VolumePaths: *mut MOUNTMGR_VOLUME_PATHS;
    let mut BufferSize: ULONG;
    let mut CharsInMgr: ULONG;
    let mut CharsInOutput: ULONG;
    let mut Paths: ULONG;

    RtlInitUnicodeString(&mut VolumeName, lpszVolumeName);
    if *(VolumeName.Buffer.offset((VolumeName.Length as usize / core::mem::size_of::<WCHAR>()) as isize - 1)) != 0x5C_u16 {
        BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
        return 0;
    }

    if MOUNTMGR_IS_DOS_VOLUME_NAME_WB(&mut VolumeName) == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    TargetName = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        ((MAX_PATH as ULONG) * core::mem::size_of::<WCHAR>() as ULONG + core::mem::size_of::<USHORT>() as ULONG) as SIZE_T,
    ) as *mut MOUNTMGR_TARGET_NAME;
    if TargetName.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    RtlZeroMemory(
        TargetName as *mut core::ffi::c_void,
        ((MAX_PATH as ULONG) * core::mem::size_of::<WCHAR>() as ULONG + core::mem::size_of::<USHORT>() as ULONG) as usize,
    );
    (*TargetName).DeviceNameLength = VolumeName.Length - core::mem::size_of::<WCHAR>() as u16;
    RtlCopyMemory(
        (*TargetName).DeviceName.as_mut_ptr() as *mut core::ffi::c_void,
        VolumeName.Buffer as *const core::ffi::c_void,
        (*TargetName).DeviceNameLength as usize,
    );
    *((*TargetName).DeviceName.as_mut_ptr()).offset(1) = 0x3F_u16;

    MountMgrHandle = CreateFileW(
        MOUNTMGR_DOS_DEVICE_NAME.as_ptr(),
        0,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        INVALID_HANDLE_VALUE,
    );
    if MountMgrHandle == INVALID_HANDLE_VALUE {
        RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName as *mut core::ffi::c_void);
        return 0;
    }

    VolumePaths = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<MOUNTMGR_VOLUME_PATHS>() as SIZE_T,
    ) as *mut MOUNTMGR_VOLUME_PATHS;
    if VolumePaths.is_null() {
        CloseHandle(MountMgrHandle);
        RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName as *mut core::ffi::c_void);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    Ret = DeviceIoControl(
        MountMgrHandle,
        IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS,
        TargetName as *mut core::ffi::c_void,
        (MAX_PATH as ULONG) * core::mem::size_of::<WCHAR>() as ULONG + core::mem::size_of::<USHORT>() as ULONG,
        VolumePaths as *mut core::ffi::c_void,
        core::mem::size_of::<MOUNTMGR_VOLUME_PATHS>() as ULONG,
        &mut BytesReturned,
        core::ptr::null_mut(),
    );
    while Ret == 0 {
        if GetLastError() != ERROR_MORE_DATA {
            CloseHandle(MountMgrHandle);
            RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName as *mut core::ffi::c_void);
            RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePaths as *mut core::ffi::c_void);
            return 0;
        }

        BufferSize = (*VolumePaths).MultiSzLength + core::mem::size_of::<MOUNTMGR_VOLUME_PATHS>() as ULONG;

        RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePaths as *mut core::ffi::c_void);
        VolumePaths = RtlAllocateHeap(RtlGetProcessHeap(), 0, (BufferSize) as SIZE_T) as *mut MOUNTMGR_VOLUME_PATHS;
        if VolumePaths.is_null() {
            CloseHandle(MountMgrHandle);
            RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName as *mut core::ffi::c_void);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }

        Ret = DeviceIoControl(
            MountMgrHandle,
            IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS,
            TargetName as *mut core::ffi::c_void,
            (MAX_PATH as ULONG) * core::mem::size_of::<WCHAR>() as ULONG + core::mem::size_of::<USHORT>() as ULONG,
            VolumePaths as *mut core::ffi::c_void,
            BufferSize,
            &mut BytesReturned,
            core::ptr::null_mut(),
        );
    }

    CloseHandle(MountMgrHandle);
    RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName as *mut core::ffi::c_void);

    Paths = 0;
    CharsInMgr = 0;
    CharsInOutput = 0;
    MultiSz = (*VolumePaths).MultiSz.as_mut_ptr();

    if cchBufferLength != 0 {
        while (CharsInMgr < (*VolumePaths).MultiSzLength / core::mem::size_of::<WCHAR>() as ULONG - 1)
            && (CharsInOutput < cchBufferLength)
        {
            if *((MultiSz as *const u16).offset(CharsInMgr as isize)) as u16 == UNICODE_NULL as u16 {
                Paths += 1;
                *((lpszVolumePathNames as *mut u16).offset(CharsInOutput as isize)) = 0x5C_u16;
                CharsInOutput += 1;
                if CharsInOutput == cchBufferLength {
                    break;
                }
            }

            *((lpszVolumePathNames as *mut u16).offset(CharsInOutput as isize)) =
                *((MultiSz as *const u16).offset(CharsInMgr as isize));
            CharsInMgr += 1;
            CharsInOutput += 1;
        }
    }

    if CharsInMgr < (*VolumePaths).MultiSzLength / core::mem::size_of::<WCHAR>() as ULONG - 1 {
        while CharsInMgr < (*VolumePaths).MultiSzLength / core::mem::size_of::<WCHAR>() as ULONG - 1 {
            if *((MultiSz as *const u16).offset(CharsInMgr as isize)) as u16 == UNICODE_NULL as u16 {
                Paths += 1;
            }
            CharsInMgr += 1;
        }
    }

    if CharsInOutput >= cchBufferLength {
        Ret = 0;
        SetLastError(ERROR_MORE_DATA);
        if !lpcchReturnLength.is_null() {
            *lpcchReturnLength = Paths + (*VolumePaths).MultiSzLength / core::mem::size_of::<WCHAR>() as ULONG;
        }
    } else {
        *((lpszVolumePathNames as *mut u16).offset(CharsInOutput as isize)) = UNICODE_NULL as u16;
        Ret = 1;
        if !lpcchReturnLength.is_null() {
            *lpcchReturnLength = CharsInOutput + 1;
        }
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePaths as *mut core::ffi::c_void);

    return Ret;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/volume.c
//  * PURPOSE:         File volume functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  *                  Erik Bos, Alexandre Julliard :
//  *                      GetLogicalDriveStringsA,
//  *                      GetLogicalDriveStringsW, GetLogicalDrives
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
// //WINE copyright notice:
// /*
//  * DOS drives handling functions
//  *
//  * Copyright 1993 Erik Bos
//  * Copyright 1996 Alexandre Julliard
//  */
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVolumeInformationA(IN LPCSTR lpRootPathName,
//                       IN LPSTR lpVolumeNameBuffer,
//                       IN DWORD nVolumeNameSize,
//                       OUT LPDWORD lpVolumeSerialNumber OPTIONAL,
//                       OUT LPDWORD lpMaximumComponentLength OPTIONAL,
//                       OUT LPDWORD lpFileSystemFlags OPTIONAL,
//                       OUT LPSTR lpFileSystemNameBuffer OPTIONAL,
//                       IN DWORD nFileSystemNameSize)
// {
//     BOOL Ret;
//     NTSTATUS Status;
//     PUNICODE_STRING RootPathNameU;
//     ANSI_STRING VolumeName, FileSystemName;
//     UNICODE_STRING VolumeNameU, FileSystemNameU;
//
//     /* If no root path provided, default to \ */
//     if (lpRootPathName == NULL)
//     {
//         lpRootPathName = "\\";
//     }
//
//     /* Convert root path to unicode */
//     RootPathNameU = Basep8BitStringToStaticUnicodeString(lpRootPathName);
//     if (RootPathNameU == NULL)
//     {
//         return FALSE;
//     }
//
//     /* Init all our STRINGS (U/A) */
//     VolumeNameU.Buffer = NULL;
//     VolumeNameU.MaximumLength = 0;
//     FileSystemNameU.Buffer = NULL;
//     FileSystemNameU.MaximumLength = 0;
//
//     VolumeName.Buffer = lpVolumeNameBuffer;
//     VolumeName.MaximumLength = nVolumeNameSize + 1;
//     FileSystemName.Buffer = lpFileSystemNameBuffer;
//     FileSystemName.MaximumLength = nFileSystemNameSize + 1;
//
//     /* Assume failure for now */
//     Ret = FALSE;
//
//     /* If caller wants volume name, allocate a buffer to receive it */
//     if (lpVolumeNameBuffer != NULL)
//     {
//         VolumeNameU.MaximumLength = sizeof(WCHAR) * (nVolumeNameSize + 1);
//         VolumeNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0,
// 	                                         VolumeNameU.MaximumLength);
//         if (VolumeNameU.Buffer == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto CleanAndQuit;
//         }
//     }
//
//     /* If caller wants file system name, allocate a buffer to receive it */
//     if (lpFileSystemNameBuffer != NULL)
//     {
//         FileSystemNameU.MaximumLength = sizeof(WCHAR) * (nFileSystemNameSize + 1);
//         FileSystemNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0,
// 	                                             FileSystemNameU.MaximumLength);
//         if (FileSystemNameU.Buffer == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto CleanAndQuit;
//         }
//     }
//
//     /* Call W */
//     Ret = GetVolumeInformationW(RootPathNameU->Buffer,  VolumeNameU.Buffer,
//                                 nVolumeNameSize, lpVolumeSerialNumber,
//                                 lpMaximumComponentLength, lpFileSystemFlags,
//                                 FileSystemNameU.Buffer, nFileSystemNameSize);
//     /* If it succeed, convert back to ANSI */
//     if (Ret)
//     {
//         if (lpVolumeNameBuffer != NULL)
//         {
//             RtlInitUnicodeString(&VolumeNameU, VolumeNameU.Buffer);
//             Status = RtlUnicodeStringToAnsiString(&VolumeName, &VolumeNameU, FALSE);
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 Ret = FALSE;
//
//                 goto CleanAndQuit;
//             }
//         }
//
//         if (lpFileSystemNameBuffer != NULL)
//         {
//             RtlInitUnicodeString(&FileSystemNameU, FileSystemNameU.Buffer);
//             Status = RtlUnicodeStringToAnsiString(&FileSystemName, &FileSystemNameU, FALSE);
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 Ret = FALSE;
//
//                 goto CleanAndQuit;
//             }
//         }
//     }
//
//     /* Clean and quit */
// CleanAndQuit:
//     if (VolumeNameU.Buffer != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameU.Buffer);
//     }
//
//     if (FileSystemNameU.Buffer != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FileSystemNameU.Buffer);
//     }
//
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// IsThisARootDirectory(IN HANDLE VolumeHandle,
//                      IN PUNICODE_STRING NtPathName)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     struct
//     {
//         FILE_NAME_INFORMATION;
//         WCHAR Buffer[MAX_PATH];
//     } FileNameInfo;
//
//     /* If we have a handle, query the name */
//     if (VolumeHandle)
//     {
//         Status = NtQueryInformationFile(VolumeHandle, &IoStatusBlock, &FileNameInfo, sizeof(FileNameInfo), FileNameInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             return FALSE;
//         }
//
//         /* Check we properly end with a \ */
//         if (FileNameInfo.FileName[FileNameInfo.FileNameLength / sizeof(WCHAR) - 1] != L'\\')
//         {
//             return FALSE;
//         }
//     }
//
//     /* If we have a path */
//     if (NtPathName != NULL)
//     {
//         HANDLE LinkHandle;
//         WCHAR Buffer[512];
//         ULONG ReturnedLength;
//         UNICODE_STRING LinkTarget;
//         OBJECT_ATTRIBUTES ObjectAttributes;
//
//         NtPathName->Length -= sizeof(WCHAR);
//
//         InitializeObjectAttributes(&ObjectAttributes, NtPathName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL, NULL);
//
//         /* Try to see whether that's a symbolic name */
//         Status = NtOpenSymbolicLinkObject(&LinkHandle, SYMBOLIC_LINK_QUERY, &ObjectAttributes);
//         NtPathName->Length += sizeof(WCHAR);
//         if (!NT_SUCCESS(Status))
//         {
//             return FALSE;
//         }
//
//         /* If so, query the target */
//         LinkTarget.Buffer = Buffer;
//         LinkTarget.Length = 0;
//         LinkTarget.MaximumLength = sizeof(Buffer);
//
//         Status = NtQuerySymbolicLinkObject(LinkHandle, &LinkTarget, &ReturnedLength);
//         NtClose(LinkHandle);
//         /* A root directory (NtName) is a symbolic link */
//         if (!NT_SUCCESS(Status))
//         {
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVolumeInformationW(IN LPCWSTR lpRootPathName,
//                       IN LPWSTR lpVolumeNameBuffer,
//                       IN DWORD nVolumeNameSize,
//                       OUT LPDWORD lpVolumeSerialNumber OPTIONAL,
//                       OUT LPDWORD lpMaximumComponentLength OPTIONAL,
//                       OUT LPDWORD lpFileSystemFlags OPTIONAL,
//                       OUT LPWSTR lpFileSystemNameBuffer OPTIONAL,
//                       IN DWORD nFileSystemNameSize)
// {
//     BOOL Ret;
//     NTSTATUS Status;
//     HANDLE VolumeHandle;
//     LPCWSTR RootPathName;
//     UNICODE_STRING NtPathName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PFILE_FS_VOLUME_INFORMATION VolumeInfo;
//     PFILE_FS_ATTRIBUTE_INFORMATION VolumeAttr;
//     ULONG OldMode, VolumeInfoSize, VolumeAttrSize;
//
//     /* If no root path provided, default to \ */
//     if (lpRootPathName == NULL)
//     {
//         RootPathName = L"\\";
//     }
//     else
//     {
//         RootPathName = lpRootPathName;
//     }
//
//     /* Convert length to bytes */
//     nVolumeNameSize *= sizeof(WCHAR);
//     nFileSystemNameSize *= sizeof(WCHAR);
//
//     /* Convert to NT name */
//     if (!RtlDosPathNameToNtPathName_U(RootPathName, &NtPathName, NULL, NULL))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* Check we really end with a backslash */
//     if (NtPathName.Buffer[(NtPathName.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
//         return FALSE;
//     }
//
//     /* Try to open the received path */
//     InitializeObjectAttributes(&ObjectAttributes, &NtPathName,
//                                 OBJ_CASE_INSENSITIVE,
//                                 NULL, NULL);
//
//     /* No errors to the user */
//     RtlSetThreadErrorMode(RTL_SEM_FAILCRITICALERRORS, &OldMode);
//     Status = NtOpenFile(&VolumeHandle, SYNCHRONIZE, &ObjectAttributes, &IoStatusBlock, 0, FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT);
//     RtlSetThreadErrorMode(OldMode, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Check whether that's a root directory */
//     if (!IsThisARootDirectory(VolumeHandle, &NtPathName))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         NtClose(VolumeHandle);
//         SetLastError(ERROR_DIR_NOT_ROOT);
//         return FALSE;
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//
//     /* Assume we don't need to query FileFsVolumeInformation */
//     VolumeInfo = NULL;
//     /* If user wants volume name, allocate a buffer to query it */
//     if (lpVolumeNameBuffer != NULL)
//     {
//         VolumeInfoSize = nVolumeNameSize + sizeof(FILE_FS_VOLUME_INFORMATION);
//     }
//     /* If user just wants the serial number, allocate a dummy buffer */
//     else if (lpVolumeSerialNumber != NULL)
//     {
//         VolumeInfoSize = MAX_PATH * sizeof(WCHAR) + sizeof(FILE_FS_VOLUME_INFORMATION);
//     }
//     /* Otherwise, nothing to query */
//     else
//     {
//         VolumeInfoSize = 0;
//     }
//
//     /* If we're to query, allocate a big enough buffer */
//     if (VolumeInfoSize != 0)
//     {
//         VolumeInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumeInfoSize);
//         if (VolumeInfo == NULL)
//         {
//             NtClose(VolumeHandle);
//             BaseSetLastNTError(STATUS_NO_MEMORY);
//             return FALSE;
//         }
//     }
//
//     /* Assume we don't need to query FileFsAttributeInformation */
//     VolumeAttr = NULL;
//     /* If user wants filesystem name, allocate a buffer to query it */
//     if (lpFileSystemNameBuffer != NULL)
//     {
//         VolumeAttrSize = nFileSystemNameSize + sizeof(FILE_FS_ATTRIBUTE_INFORMATION);
//     }
//     /* If user just wants max compo len or flags, allocate a dummy buffer */
//     else if (lpMaximumComponentLength != NULL || lpFileSystemFlags != NULL)
//     {
//         VolumeAttrSize = MAX_PATH * sizeof(WCHAR) + sizeof(FILE_FS_ATTRIBUTE_INFORMATION);
//     }
//     else
//     {
//         VolumeAttrSize = 0;
//     }
//
//     /* If we're to query, allocate a big enough buffer */
//     if (VolumeAttrSize != 0)
//     {
//         VolumeAttr = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumeAttrSize);
//         if (VolumeAttr == NULL)
//         {
//             if (VolumeInfo != NULL)
//             {
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeInfo);
//             }
//
//             NtClose(VolumeHandle);
//             BaseSetLastNTError(STATUS_NO_MEMORY);
//             return FALSE;
//         }
//     }
//
//     /* Assume we'll fail */
//     Ret = FALSE;
//
//     /* If we're to query FileFsVolumeInformation, do it now! */
//     if (VolumeInfo != NULL)
//     {
//         Status = NtQueryVolumeInformationFile(VolumeHandle, &IoStatusBlock, VolumeInfo, VolumeInfoSize, FileFsVolumeInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             goto CleanAndQuit;
//         }
//     }
//
//     /* If we're to query FileFsAttributeInformation, do it now! */
//     if (VolumeAttr != NULL)
//     {
//         Status = NtQueryVolumeInformationFile(VolumeHandle, &IoStatusBlock, VolumeAttr, VolumeAttrSize, FileFsAttributeInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             goto CleanAndQuit;
//         }
//     }
//
//     /* If user wants volume name */
//     if (lpVolumeNameBuffer != NULL)
//     {
//         /* Check its buffer can hold it (+ 0) */
//         if (VolumeInfo->VolumeLabelLength >= nVolumeNameSize)
//         {
//             SetLastError(ERROR_BAD_LENGTH);
//             goto CleanAndQuit;
//         }
//
//         /* Copy and zero */
//         RtlCopyMemory(lpVolumeNameBuffer, VolumeInfo->VolumeLabel, VolumeInfo->VolumeLabelLength);
//         lpVolumeNameBuffer[VolumeInfo->VolumeLabelLength / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     /* If user wants wants serial number, return it */
//     if (lpVolumeSerialNumber != NULL)
//     {
//         *lpVolumeSerialNumber = VolumeInfo->VolumeSerialNumber;
//     }
//
//     /* If user wants filesystem name */
//     if (lpFileSystemNameBuffer != NULL)
//     {
//         /* Check its buffer can hold it (+ 0) */
//         if (VolumeAttr->FileSystemNameLength >= nFileSystemNameSize)
//         {
//             SetLastError(ERROR_BAD_LENGTH);
//             goto CleanAndQuit;
//         }
//
//         /* Copy and zero */
//         RtlCopyMemory(lpFileSystemNameBuffer, VolumeAttr->FileSystemName, VolumeAttr->FileSystemNameLength);
//         lpFileSystemNameBuffer[VolumeAttr->FileSystemNameLength / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     /* If user wants wants max compo len, return it */
//     if (lpMaximumComponentLength != NULL)
//     {
//         *lpMaximumComponentLength = VolumeAttr->MaximumComponentNameLength;
//     }
//
//     /* If user wants wants FS flags, return them */
//     if (lpFileSystemFlags != NULL)
//     {
//         *lpFileSystemFlags = VolumeAttr->FileSystemAttributes;
//     }
//
//     /* We did it! */
//     Ret = TRUE;
//
// CleanAndQuit:
//     NtClose(VolumeHandle);
//
//     if (VolumeInfo != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeInfo);
//     }
//
//     if (VolumeAttr != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeAttr);
//     }
//
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetVolumeLabelA(IN LPCSTR lpRootPathName,
//                 IN LPCSTR lpVolumeName OPTIONAL) /* NULL if deleting label */
// {
//     BOOL Ret;
//     UNICODE_STRING VolumeNameU;
//     PUNICODE_STRING RootPathNameU;
//
//     if (lpRootPathName == NULL)
//     {
//         lpRootPathName = "\\";
//     }
//
//     RootPathNameU = Basep8BitStringToStaticUnicodeString(lpRootPathName);
//     if (RootPathNameU == NULL)
//     {
//         return FALSE;
//     }
//
//     if (lpVolumeName != NULL)
//     {
//         if (!Basep8BitStringToDynamicUnicodeString(&VolumeNameU, lpVolumeName))
//         {
//             return FALSE;
//         }
//     }
//     else
//     {
//         VolumeNameU.Buffer = NULL;
//     }
//
//     Ret = SetVolumeLabelW(RootPathNameU->Buffer, VolumeNameU.Buffer);
//     RtlFreeUnicodeString(&VolumeNameU);
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetVolumeLabelW(IN LPCWSTR lpRootPathName,
//                 IN LPCWSTR lpVolumeName OPTIONAL) /* NULL if deleting label */
// {
//     BOOL Ret;
//     NTSTATUS Status;
//     PWSTR VolumeRoot;
//     HANDLE VolumeHandle;
//     WCHAR VolumeGuid[MAX_PATH];
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PFILE_FS_LABEL_INFORMATION FsLabelInfo;
//     UNICODE_STRING VolumeName, NtVolumeName;
//
//     /* If no root path provided, default to \ */
//     VolumeRoot = L"\\";
//
//     /* If user wants to set a label, make it a string */
//     if (lpVolumeName != NULL)
//     {
//         RtlInitUnicodeString(&VolumeName, lpVolumeName);
//     }
//     else
//     {
//         VolumeName.Length = 0;
//         VolumeName.MaximumLength = 0;
//         VolumeName.Buffer = NULL;
//     }
//
//     /* If we received a volume, try to get its GUID name */
//     if (lpRootPathName != NULL)
//     {
//         Ret = GetVolumeNameForVolumeMountPointW(lpRootPathName, VolumeGuid, MAX_PATH);
//     }
//     else
//     {
//         Ret = FALSE;
//     }
//
//     /* If we got the GUID name, use it */
//     if (Ret)
//     {
//         VolumeRoot = VolumeGuid;
//     }
//     else
//     {
//         /* Otherwise, use the name provided by the caller */
//         if (lpRootPathName != NULL)
//         {
//             VolumeRoot = (PWSTR)lpRootPathName;
//         }
//     }
//
//     /* Convert to a NT path */
//     if (!RtlDosPathNameToNtPathName_U(VolumeRoot, &NtVolumeName, NULL, NULL))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//
//     /* Check we really end with a backslash */
//     if (NtVolumeName.Buffer[(NtVolumeName.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer);
//         BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
//         return FALSE;
//     }
//
//     /* Try to open the root directory */
//     InitializeObjectAttributes(&ObjectAttributes, &NtVolumeName,
//                                OBJ_CASE_INSENSITIVE, NULL, NULL);
//
//     Status = NtOpenFile(&VolumeHandle, SYNCHRONIZE | FILE_WRITE_DATA,
//                         &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Validate it's really a root path */
//     if (!IsThisARootDirectory(VolumeHandle, NULL))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer);
//         NtClose(VolumeHandle);
//         SetLastError(ERROR_DIR_NOT_ROOT);
//         return FALSE;
//     }
//
//     /* Done */
//     NtClose(VolumeHandle);
//
//     /* Now, open the volume to perform the label change */
//     NtVolumeName.Length -= sizeof(WCHAR);
//     InitializeObjectAttributes(&ObjectAttributes, &NtVolumeName,
//                                OBJ_CASE_INSENSITIVE, NULL, NULL);
//
//     Status = NtOpenFile(&VolumeHandle, SYNCHRONIZE | FILE_WRITE_DATA,
//                         &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NtVolumeName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Assume success */
//     Ret = TRUE;
//
//     /* Allocate a buffer that can hold new label and its size */
//     FsLabelInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, sizeof(FILE_FS_LABEL_INFORMATION) + VolumeName.Length);
//     if (FsLabelInfo != NULL)
//     {
//         /* Copy name and set its size */
//         RtlCopyMemory(FsLabelInfo->VolumeLabel, VolumeName.Buffer, VolumeName.Length);
//         FsLabelInfo->VolumeLabelLength = VolumeName.Length;
//
//         /* And finally, set new label */
//         Status = NtSetVolumeInformationFile(VolumeHandle, &IoStatusBlock, FsLabelInfo, sizeof(FILE_FS_LABEL_INFORMATION) + VolumeName.Length, FileFsLabelInformation);
//     }
//     else
//     {
//         /* Allocation failed */
//         Status = STATUS_NO_MEMORY;
//     }
//
//     /* In case of failure, set status and mark failure */
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         Ret = FALSE;
//     }
//
//     /* We're done */
//     NtClose(VolumeHandle);
//
//     /* Free buffer if required */
//     if (FsLabelInfo != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FsLabelInfo);
//     }
//
//     return Ret;
// }
//
// /*
//  * @implemented (Wine 13 sep 2008)
//  */
// HANDLE
// WINAPI
// FindFirstVolumeW(IN LPWSTR volume,
//                  IN DWORD len)
// {
//     DWORD size = 1024;
//     DWORD br;
//     HANDLE mgr = CreateFileW( MOUNTMGR_DOS_DEVICE_NAME, 0, FILE_SHARE_READ|FILE_SHARE_WRITE,
//                               NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, INVALID_HANDLE_VALUE );
//     if (mgr == INVALID_HANDLE_VALUE) return INVALID_HANDLE_VALUE;
//
//     for (;;)
//     {
//         MOUNTMGR_MOUNT_POINT input;
//         MOUNTMGR_MOUNT_POINTS *output;
//
//         if (!(output = RtlAllocateHeap( RtlGetProcessHeap(), 0, size )))
//         {
//             SetLastError( ERROR_NOT_ENOUGH_MEMORY );
//             break;
//         }
//         memset( &input, 0, sizeof(input) );
//
//         if (!DeviceIoControl( mgr, IOCTL_MOUNTMGR_QUERY_POINTS, &input, sizeof(input),
//                               output, size, &br, NULL ))
//         {
//             if (GetLastError() != ERROR_MORE_DATA) break;
//             size = output->Size;
//             RtlFreeHeap( RtlGetProcessHeap(), 0, output );
//             continue;
//         }
//         CloseHandle( mgr );
//         /* abuse the Size field to store the current index */
//         output->Size = 0;
//         if (!FindNextVolumeW( output, volume, len ))
//         {
//             RtlFreeHeap( RtlGetProcessHeap(), 0, output );
//             return INVALID_HANDLE_VALUE;
//         }
//         return (HANDLE)output;
//     }
//     CloseHandle( mgr );
//     return INVALID_HANDLE_VALUE;
// }
//
// /*
//  * @implemented (Wine 13 sep 2008)
//  */
// HANDLE
// WINAPI
// FindFirstVolumeA(IN LPSTR volume,
//                  IN DWORD len)
// {
//     WCHAR *buffer = NULL;
//     HANDLE handle;
//
//     buffer = RtlAllocateHeap( RtlGetProcessHeap(), 0, len * sizeof(WCHAR) );
//
//     if (!buffer)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     handle = FindFirstVolumeW( buffer, len );
//
//     if (handle != INVALID_HANDLE_VALUE)
//     {
//         if (!WideCharToMultiByte( CP_ACP, 0, buffer, -1, volume, len, NULL, NULL ))
//         {
//             FindVolumeClose( handle );
//             handle = INVALID_HANDLE_VALUE;
//         }
//     }
//     RtlFreeHeap( RtlGetProcessHeap(), 0, buffer );
//     return handle;
// }
//
// /*
//  * @implemented (Wine 13 sep 2008)
//  */
// BOOL
// WINAPI
// FindVolumeClose(IN HANDLE hFindVolume)
// {
//     return RtlFreeHeap(RtlGetProcessHeap(), 0, hFindVolume);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVolumePathNameA(IN LPCSTR lpszFileName,
//                    IN LPSTR lpszVolumePathName,
//                    IN DWORD cchBufferLength)
// {
//     BOOL Ret;
//     PUNICODE_STRING FileNameU;
//     ANSI_STRING VolumePathName;
//     UNICODE_STRING VolumePathNameU;
//
//     /* Convert file name to unicode */
//     FileNameU = Basep8BitStringToStaticUnicodeString(lpszFileName);
//     if (FileNameU == NULL)
//     {
//         return FALSE;
//     }
//
//     /* Initialize all the strings we'll need */
//     VolumePathName.Buffer = lpszVolumePathName;
//     VolumePathName.Length = 0;
//     VolumePathName.MaximumLength = cchBufferLength - 1;
//
//     VolumePathNameU.Length = 0;
//     VolumePathNameU.MaximumLength = (cchBufferLength - 1) * sizeof(WCHAR) + sizeof(UNICODE_NULL);
//     /* Allocate a buffer for calling the -W */
//     VolumePathNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumePathNameU.MaximumLength);
//     if (VolumePathNameU.Buffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Call the -W implementation */
//     Ret = GetVolumePathNameW(FileNameU->Buffer, VolumePathNameU.Buffer, cchBufferLength);
//     /* If it succeed */
//     if (Ret)
//     {
//         NTSTATUS Status;
//
//         /* Convert back to ANSI */
//         RtlInitUnicodeString(&VolumePathNameU, VolumePathNameU.Buffer);
//         Status = RtlUnicodeStringToAnsiString(&VolumePathName, &VolumePathNameU, FALSE);
//         /* If conversion failed, just set error code and fail the rest */
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(Status);
//             Ret = FALSE;
//         }
//         /* Otherwise, null terminate the string (it's OK, we computed -1) */
//         else
//         {
//             VolumePathName.Buffer[VolumePathName.Length] = ANSI_NULL;
//         }
//     }
//
//     /* Free the buffer allocated for -W call */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNameU.Buffer);
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVolumePathNameW(IN LPCWSTR lpszFileName,
//                    IN LPWSTR lpszVolumePathName,
//                    IN DWORD cchBufferLength)
// {
//     BOOL MountPoint;
//     DWORD FullPathLen;
//     WCHAR OldFilePart;
//     UNICODE_STRING FullPath;
//     PWSTR FullPathBuf, FilePart, VolumeNameBuf;
//
//     /* Probe for full path len */
//     FullPathLen = GetFullPathNameW(lpszFileName, 0, NULL, NULL);
//     if (FullPathLen == 0)
//     {
//         return FALSE;
//     }
//
//     /* Allocate a big enough buffer to receive it */
//     FullPathBuf = RtlAllocateHeap(RtlGetProcessHeap(), 0, (FullPathLen + 10) * sizeof(WCHAR));
//     if (FullPathBuf == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* And get full path name */
//     if (GetFullPathNameW(lpszFileName, FullPathLen + 10, FullPathBuf, &FilePart) == 0)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf);
//         return FALSE;
//     }
//
//     /* Make a string out of it */
//     RtlInitUnicodeString(&FullPath, FullPathBuf);
//     /* We will finish our string with '\', for ease of the parsing after */
//     if (FullPath.Buffer[(FullPath.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         FullPath.Length += sizeof(WCHAR);
//         FullPath.Buffer[(FullPath.Length / sizeof(WCHAR)) - 1] = L'\\';
//         FullPath.Buffer[FullPath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     /* Allocate a buffer big enough to receive our volume name */
//     VolumeNameBuf = RtlAllocateHeap(RtlGetProcessHeap(), 0, 0x2000 * sizeof(WCHAR));
//     if (VolumeNameBuf == NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* We don't care about file part: we added an extra backslash, so there's no
//      * file, we're back at the dir level.
//      * We'll recompute file part afterwards
//      */
//     FilePart = NULL;
//     /* Keep track of the letter we could drop to shorten the string */
//     OldFilePart = UNICODE_NULL;
//     /* As long as querying volume name fails, keep looping */
//     while (!BasepGetVolumeNameForVolumeMountPoint(FullPath.Buffer, VolumeNameBuf, 0x2000u, &MountPoint))
//     {
//         USHORT LastSlash;
//
//         /* Not a mount point, but opening returning access denied? Assume it's one, just not
//          * a reparse backed one (classic mount point, a device)!
//          */
//         if (!MountPoint && GetLastError() == ERROR_ACCESS_DENIED)
//         {
//             MountPoint = TRUE;
//         }
//
//         /* BasepGetVolumeNameForVolumeMountPoint failed, but returned a volume name.
//          * This can happen when we are given a reparse point where MountMgr could find associated
//          * volume name which is not a valid DOS volume
//          * A valid DOS name always starts with \\
//          */
//         if (VolumeNameBuf[0] != UNICODE_NULL && (FullPath.Buffer[0] != L'\\' || FullPath.Buffer[1] != L'\\'))
//         {
//             CHAR RootPathName[4];
//
//             /* Construct a simple <letter>:\ string to get drive type */
//             RootPathName[0] = FullPath.Buffer[0];
//             RootPathName[1] = ':';
//             RootPathName[2] = '\\';
//             RootPathName[3] = ANSI_NULL;
//
//             /* If we weren't given a drive letter actually, or if that's not a remote drive
//              * Note: in this code path, we're recursive and stop fail loop
//              */
//             if (FullPath.Buffer[1] != L':' || GetDriveTypeA(RootPathName) != DRIVE_REMOTE)
//             {
//                 BOOL Ret;
//
//                 /* We won't need the full path, we'll now work with the returned volume name */
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf);
//                 /* If it wasn't an NT name which was returned */
//                 if ((VolumeNameBuf[0] != L'\\') || (VolumeNameBuf[1] != L'?') ||
//                     (VolumeNameBuf[2] != L'?') || (VolumeNameBuf[3] != L'\\'))
//                 {
//                     PWSTR GlobalPath;
//                     UNICODE_STRING GlobalRoot;
//
//                     /* Create a new name in the NT namespace (from Win32) */
//                     RtlInitUnicodeString(&FullPath, VolumeNameBuf);
//                     RtlInitUnicodeString(&GlobalRoot, L"\\\\?\\GLOBALROOT");
//
//                     /* We allocate a buffer than can contain both the namespace and the volume name */
//                     GlobalPath = RtlAllocateHeap(RtlGetProcessHeap(), 0, FullPath.Length + GlobalRoot.Length);
//                     if (GlobalPath == NULL)
//                     {
//                         RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameBuf);
//                         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//                         return FALSE;
//                     }
//
//                     /* Fill in the new query name */
//                     RtlCopyMemory(GlobalPath, GlobalRoot.Buffer, GlobalRoot.Length);
//                     RtlCopyMemory((PVOID)((ULONG_PTR)GlobalPath + GlobalRoot.Length), FullPath.Buffer, FullPath.Length);
//                     GlobalPath[(FullPath.Length + GlobalRoot.Length) / sizeof(WCHAR)] = UNICODE_NULL;
//
//                     /* Give it another try */
//                     Ret = GetVolumePathNameW(GlobalPath, lpszVolumePathName, cchBufferLength);
//
//                     RtlFreeHeap(RtlGetProcessHeap(), 0, GlobalPath);
//                 }
//                 else
//                 {
//                     /* If we don't have a drive letter in the Win32 name space \\.\<letter>: */
//                     if ((VolumeNameBuf[4] != UNICODE_NULL) && (VolumeNameBuf[5] != L':'))
//                     {
//                         /* Shit our starting \\ */
//                         RtlInitUnicodeString(&FullPath, VolumeNameBuf);
//                         RtlMoveMemory(VolumeNameBuf, (PVOID)((ULONG_PTR)VolumeNameBuf + (2 * sizeof(WCHAR))), FullPath.Length - (3 * sizeof(WCHAR)));
//                     }
//                     /* Otherwise, just make sure we're double \ at the being to query again with the
//                      * proper namespace
//                      */
//                     else
//                     {
//                         VolumeNameBuf[1] = L'\\';
//                     }
//
//                     /* Give it another try */
//                     Ret = GetVolumePathNameW(VolumeNameBuf, lpszVolumePathName, cchBufferLength);
//                 }
//
//                 /* And done! */
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameBuf);
//                 return Ret;
//             }
//         }
//
//         /* No mount point but with a file part? Restore filepart and exit */
//         if (!MountPoint && FilePart != NULL)
//         {
//             FilePart[0] = OldFilePart;
//             RtlInitUnicodeString(&FullPath, FullPathBuf);
//             break;
//         }
//
//         /* We cannot go down the path any longer, too small */
//         if (FullPath.Length <= sizeof(WCHAR))
//         {
//             break;
//         }
//
//         /* Prepare the next split */
//         LastSlash = (FullPath.Length / sizeof(WCHAR)) - 2;
//         if (FullPath.Length / sizeof(WCHAR) != 2)
//         {
//             do
//             {
//                 if (FullPath.Buffer[LastSlash] == L'\\')
//                 {
//                     break;
//                 }
//
//                 --LastSlash;
//             } while (LastSlash != 0);
//         }
//
//         /* We couldn't split path, quit */
//         if (LastSlash == 0)
//         {
//             break;
//         }
//
//         /* If that's a mount point, keep track of the directory name */
//         if (MountPoint)
//         {
//             FilePart = &FullPath.Buffer[LastSlash + 1];
//             OldFilePart = FilePart[0];
//             /* And null terminate the string */
//             FilePart[0] = UNICODE_NULL;
//         }
//         /* Otherwise, just null terminate the string */
//         else
//         {
//             FullPath.Buffer[LastSlash + 1] = UNICODE_NULL;
//         }
//
//         /* We went down a bit in the path, fix the string and retry */
//         RtlInitUnicodeString(&FullPath, FullPathBuf);
//     }
//
//     /* Once here, we'll return something from the full path buffer, so release
//      * output buffer
//      */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameBuf);
//
//     /* Not a mount point, bail out */
//     if (!MountPoint && FilePart == NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf);
//         return FALSE;
//     }
//
//     /* Make sure we have enough room to copy our volume */
//     if ((cchBufferLength * sizeof(WCHAR)) < FullPath.Length + sizeof(UNICODE_NULL))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf);
//         SetLastError(ERROR_FILENAME_EXCED_RANGE);
//         return FALSE;
//     }
//
//     /* Copy and null terminate */
//     RtlCopyMemory(lpszVolumePathName, FullPath.Buffer, FullPath.Length);
//     lpszVolumePathName[FullPath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, FullPathBuf);
//
//     /* Done! */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindNextVolumeA(IN HANDLE handle,
//                 IN LPSTR volume,
//                 IN DWORD len)
// {
//     WCHAR *buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, len * sizeof(WCHAR));
//     BOOL ret;
//
//     if (!buffer)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if ((ret = FindNextVolumeW( handle, buffer, len )))
//     {
//         if (!WideCharToMultiByte( CP_ACP, 0, buffer, -1, volume, len, NULL, NULL )) ret = FALSE;
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, buffer);
//     return ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindNextVolumeW(IN HANDLE handle,
//                 IN LPWSTR volume,
//                 IN DWORD len)
// {
//     MOUNTMGR_MOUNT_POINTS *data = handle;
//
//     while (data->Size < data->NumberOfMountPoints)
//     {
//         static const WCHAR volumeW[] = {'\\','?','?','\\','V','o','l','u','m','e','{',};
//         WCHAR *link = (WCHAR *)((char *)data + data->MountPoints[data->Size].SymbolicLinkNameOffset);
//         DWORD size = data->MountPoints[data->Size].SymbolicLinkNameLength;
//         data->Size++;
//         /* skip non-volumes */
//         if (size < sizeof(volumeW) || memcmp( link, volumeW, sizeof(volumeW) )) continue;
//         if (size + sizeof(WCHAR) >= len * sizeof(WCHAR))
//         {
//             SetLastError( ERROR_FILENAME_EXCED_RANGE );
//             return FALSE;
//         }
//         memcpy( volume, link, size );
//         volume[1] = '\\';  /* map \??\ to \\?\ */
//         volume[size / sizeof(WCHAR)] = '\\';  /* Windows appends a backslash */
//         volume[size / sizeof(WCHAR) + 1] = 0;
//         DPRINT( "returning entry %u %s\n", data->Size - 1, volume );
//         return TRUE;
//     }
//     SetLastError( ERROR_NO_MORE_FILES );
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVolumePathNamesForVolumeNameA(IN LPCSTR lpszVolumeName,
//                                  IN LPSTR lpszVolumePathNames,
//                                  IN DWORD cchBufferLength,
//                                  OUT PDWORD lpcchReturnLength)
// {
//     BOOL Ret;
//     NTSTATUS Status;
//     DWORD cchReturnLength;
//     ANSI_STRING VolumePathName;
//     PUNICODE_STRING VolumeNameU;
//     UNICODE_STRING VolumePathNamesU;
//
//     /* Convert volume name to unicode */
//     VolumeNameU = Basep8BitStringToStaticUnicodeString(lpszVolumeName);
//     if (VolumeNameU == NULL)
//     {
//         return FALSE;
//     }
//
//     /* Initialize the strings we'll use later on */
//     VolumePathName.Length = 0;
//     VolumePathName.MaximumLength = cchBufferLength;
//     VolumePathName.Buffer = lpszVolumePathNames;
//
//     VolumePathNamesU.Length = 0;
//     VolumePathNamesU.MaximumLength = sizeof(WCHAR) * cchBufferLength;
//     /* If caller provided a non 0 sized string, allocate a buffer for our unicode string */
//     if (VolumePathNamesU.MaximumLength != 0)
//     {
//         VolumePathNamesU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.MaximumLength);
//         if (VolumePathNamesU.Buffer == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//     else
//     {
//         VolumePathNamesU.Buffer = NULL;
//     }
//
//     /* Call the -W implementation */
//     Ret = GetVolumePathNamesForVolumeNameW(VolumeNameU->Buffer, VolumePathNamesU.Buffer,
//                                            cchBufferLength, &cchReturnLength);
//     /* Call succeed, we'll return the total length */
//     if (Ret)
//     {
//         VolumePathNamesU.Length = sizeof(WCHAR) * cchReturnLength;
//     }
//     else
//     {
//         /* Else, if we fail for anything else than too small buffer, quit */
//         if (GetLastError() != ERROR_MORE_DATA)
//         {
//             if (VolumePathNamesU.Buffer != NULL)
//             {
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.Buffer);
//             }
//
//             return FALSE;
//         }
//
//         /* Otherwise, we'll just copy as much as we can */
//         VolumePathNamesU.Length = sizeof(WCHAR) * cchBufferLength;
//     }
//
//     /* Convert our output string back to ANSI */
//     Status = RtlUnicodeStringToAnsiString(&VolumePathName, &VolumePathNamesU, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//
//         if (VolumePathNamesU.Buffer != NULL)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.Buffer);
//         }
//
//         return FALSE;
//     }
//
//     /* If caller wants return length, two cases... */
//     if (lpcchReturnLength != NULL)
//     {
//         /* We succeed: return the copied length */
//         if (Ret)
//         {
//             *lpcchReturnLength = VolumePathName.Length;
//         }
//         /* We failed, return the size we would have loved having! */
//         else
//         {
//             *lpcchReturnLength = sizeof(WCHAR) * cchReturnLength;
//         }
//     }
//
//     /* Release our buffer if allocated */
//     if (VolumePathNamesU.Buffer != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePathNamesU.Buffer);
//     }
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVolumePathNamesForVolumeNameW(IN LPCWSTR lpszVolumeName,
//                                  IN LPWSTR lpszVolumePathNames,
//                                  IN DWORD cchBufferLength,
//                                  OUT PDWORD lpcchReturnLength)
// {
//     BOOL Ret;
//     PWSTR MultiSz;
//     DWORD BytesReturned;
//     HANDLE MountMgrHandle;
//     UNICODE_STRING VolumeName;
//     PMOUNTMGR_TARGET_NAME TargetName;
//     PMOUNTMGR_VOLUME_PATHS VolumePaths;
//     ULONG BufferSize, CharsInMgr, CharsInOutput, Paths;
//
//     /* First look that our volume name looks somehow correct */
//     RtlInitUnicodeString(&VolumeName, lpszVolumeName);
//     if (VolumeName.Buffer[(VolumeName.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
//         return FALSE;
//     }
//
//     /* Validate it's a DOS volume name finishing with a backslash */
//     if (!MOUNTMGR_IS_DOS_VOLUME_NAME_WB(&VolumeName))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Allocate an input MOUNTMGR_TARGET_NAME */
//     TargetName = RtlAllocateHeap(RtlGetProcessHeap(), 0, MAX_PATH * sizeof(WCHAR) + sizeof(USHORT));
//     if (TargetName == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* And fill it */
//     RtlZeroMemory(TargetName, MAX_PATH * sizeof(WCHAR) + sizeof(USHORT));
//     TargetName->DeviceNameLength = VolumeName.Length - sizeof(WCHAR);
//     RtlCopyMemory(TargetName->DeviceName, VolumeName.Buffer, TargetName->DeviceNameLength);
//     TargetName->DeviceName[1] = L'?';
//
//     /* Open the mount manager */
//     MountMgrHandle = CreateFileW(MOUNTMGR_DOS_DEVICE_NAME, 0,
//                                  FILE_SHARE_READ | FILE_SHARE_WRITE, NULL,
//                                  OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
//                                  INVALID_HANDLE_VALUE);
//     if (MountMgrHandle == INVALID_HANDLE_VALUE)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName);
//         return FALSE;
//     }
//
//     /* Allocate an initial output buffer, just to get length */
//     VolumePaths = RtlAllocateHeap(RtlGetProcessHeap(), 0, sizeof(MOUNTMGR_VOLUME_PATHS));
//     if (VolumePaths == NULL)
//     {
//         CloseHandle(MountMgrHandle);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Query the paths */
//     Ret = DeviceIoControl(MountMgrHandle, IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS,
//                           TargetName, MAX_PATH * sizeof(WCHAR) + sizeof(USHORT),
//                           VolumePaths, sizeof(MOUNTMGR_VOLUME_PATHS), &BytesReturned,
//                           NULL);
//     /* Loop until we can query everything */
//     while (!Ret)
//     {
//         /* If failed for another reason than too small buffer, fail */
//         if (GetLastError() != ERROR_MORE_DATA)
//         {
//             CloseHandle(MountMgrHandle);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePaths);
//             return FALSE;
//         }
//
//         /* Get the required length */
//         BufferSize = VolumePaths->MultiSzLength + sizeof(MOUNTMGR_VOLUME_PATHS);
//
//         /* And reallocate our output buffer (big enough this time) */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePaths);
//         VolumePaths = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize);
//         if (VolumePaths == NULL)
//         {
//             CloseHandle(MountMgrHandle);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName);
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//
//         /* Query again the mount mgr */
//         Ret = DeviceIoControl(MountMgrHandle, IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATHS,
//                               TargetName, MAX_PATH * sizeof(WCHAR) + sizeof(USHORT),
//                               VolumePaths, BufferSize, &BytesReturned, NULL);
//     }
//
//     /* We're done, no need for input nor mount mgr any longer */
//     CloseHandle(MountMgrHandle);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, TargetName);
//
//     /* Initialize:
//      - Number of paths we saw (useful to count extra \)
//      - Progress in mount mgr output
//      - Progress in output buffer
//      - Direct buffer to returned MultiSz
//     */
//     Paths = 0;
//     CharsInMgr = 0;
//     CharsInOutput = 0;
//     MultiSz = VolumePaths->MultiSz;
//
//     /* If we have an output buffer */
//     if (cchBufferLength != 0)
//     {
//         /* Loop on the output to recopy it back to the caller
//          * Note that we loop until -1 not to handle last 0 (will be done later on)
//          */
//         for (; (CharsInMgr < VolumePaths->MultiSzLength / sizeof(WCHAR) - 1) && (CharsInOutput < cchBufferLength);
//              ++CharsInMgr, ++CharsInOutput)
//         {
//             /* When we reach the end of a path */
//             if (MultiSz[CharsInMgr] == UNICODE_NULL)
//             {
//                 /* On path done (count), add an extra \ at the end */
//                 ++Paths;
//                 lpszVolumePathNames[CharsInOutput] = L'\\';
//                 ++CharsInOutput;
//                 /* Make sure we don't overflow */
//                 if (CharsInOutput == cchBufferLength)
//                 {
//                     break;
//                 }
//             }
//
//             /* Copy the char to the caller
//              * So, in case we're in the end of a path, we wrote two chars to
//              * the output buffer: \\ and \0
//              */
//             lpszVolumePathNames[CharsInOutput] = MultiSz[CharsInMgr];
//         }
//     }
//
//     /* If output buffer was too small (ie, we couldn't parse all the input buffer) */
//     if (CharsInMgr < VolumePaths->MultiSzLength / sizeof(WCHAR) - 1)
//     {
//         /* Keep looping on it, to count the number of extra \ that will be required
//          * So that on the next call, caller can allocate enough space
//          */
//         for (; CharsInMgr < VolumePaths->MultiSzLength / sizeof(WCHAR) - 1; ++CharsInMgr)
//         {
//             if (MultiSz[CharsInMgr] == UNICODE_NULL)
//             {
//                 ++Paths;
//             }
//         }
//     }
//
//     /* If we couldn't write as much as we wanted to the output buffer
//      * This handles the case where we could write everything excepted the
//      * terminating \0 for multi SZ
//      */
//     if (CharsInOutput >= cchBufferLength)
//     {
//         /* Fail and set appropriate error code */
//         Ret = FALSE;
//         SetLastError(ERROR_MORE_DATA);
//         /* If caller wants to know how many chars to allocate, return it */
//         if (lpcchReturnLength != NULL)
//         {
//             /* It's amount of extra \ + number of chars in MultiSz (including double \0) */
//             *lpcchReturnLength = Paths + (VolumePaths->MultiSzLength / sizeof(WCHAR));
//         }
//     }
//     else
//     {
//         /* It succeed so terminate the multi SZ (second \0) */
//         lpszVolumePathNames[CharsInOutput] = UNICODE_NULL;
//         Ret = TRUE;
//
//         /* If caller wants the amount of chars written, return it */
//         if (lpcchReturnLength != NULL)
//         {
//             /* Including the terminating \0 we just added */
//             *lpcchReturnLength = CharsInOutput + 1;
//         }
//     }
//
//     /* Free last bits */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, VolumePaths);
//
//     /* And return */
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

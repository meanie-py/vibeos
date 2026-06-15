use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;
pub const MAX_DOS_DRIVES: u32 = 26;
const GetDiskFreeSpace2GB: ULONG = 0x8;
const STATUS_FILE_IS_A_DIRECTORY: NTSTATUS = 0xC00000BAu32 as NTSTATUS;

extern "system" {
    fn Basep8BitStringToStaticUnicodeString(AnsiString: LPCSTR) -> PUNICODE_STRING;
    fn BasepGetVolumeNameFromReparsePoint(
        lpszMountPoint: LPCWSTR,
        lpszVolumeName: LPWSTR,
        cchBufferLength: DWORD,
        IsAMountPoint: LPBOOL,
    ) -> BOOL;
    fn IsThisARootDirectory(VolumeHandle: HANDLE, NtPathName: PUNICODE_STRING) -> BOOL;
}

// c2rust named the anonymous Query struct of PROCESS_DEVICEMAP_INFORMATION
// `C2RustUnnamed_11`; map it to the bindgen inner type.
type C2RustUnnamed_11 = _PROCESS_DEVICEMAP_INFORMATION__bindgen_ty_1__bindgen_ty_2;

#[no_mangle]
pub unsafe extern "system" fn GetLogicalDriveStringsA(
    nBufferLength: DWORD,
    lpBuffer: LPSTR,
) -> DWORD {
    let mut drive: DWORD;
    let mut count: DWORD;
    let mut dwDriveMap: DWORD;
    let mut p: LPSTR;

    dwDriveMap = GetLogicalDrives();

    drive = 0;
    count = 0;
    while drive < MAX_DOS_DRIVES as DWORD {
        if dwDriveMap & ((1 as DWORD) << drive) != 0 {
            count += 1;
        }
        drive += 1;
    }

    if (count * 4) + 1 > nBufferLength {
        return (count * 4) + 1;
    }

    p = lpBuffer;

    drive = 0;
    while drive < MAX_DOS_DRIVES as DWORD {
        if dwDriveMap & ((1 as DWORD) << drive) != 0 {
            *p = (b'A' + drive as u8) as CHAR;
            p = p.offset(1);
            *p = b':' as CHAR;
            p = p.offset(1);
            *p = b'\\' as CHAR;
            p = p.offset(1);
            *p = 0i8 as CHAR;
            p = p.offset(1);
        }
        drive += 1;
    }
    *p = 0i8 as CHAR;

    return count * 4;
}

#[no_mangle]
pub unsafe extern "system" fn GetLogicalDriveStringsW(
    nBufferLength: DWORD,
    lpBuffer: LPWSTR,
) -> DWORD {
    let mut drive: DWORD;
    let mut count: DWORD;
    let mut dwDriveMap: DWORD;
    let mut p: LPWSTR;

    dwDriveMap = GetLogicalDrives();

    drive = 0;
    count = 0;
    while drive < MAX_DOS_DRIVES as DWORD {
        if dwDriveMap & ((1 as DWORD) << drive) != 0 {
            count += 1;
        }
        drive += 1;
    }

    if (count * 4 + 1) > nBufferLength {
        return count * 4 + 1;
    }

    p = lpBuffer;
    drive = 0;
    while drive < MAX_DOS_DRIVES as DWORD {
        if dwDriveMap & ((1 as DWORD) << drive) != 0 {
            let fresh4 = p;
            p = p.offset(1);
            *fresh4 = (b'A' as DWORD + drive) as WCHAR;
            let fresh5 = p;
            p = p.offset(1);
            *fresh5 = b':' as WCHAR;
            let fresh6 = p;
            p = p.offset(1);
            *fresh6 = b'\\' as WCHAR;
            let fresh7 = p;
            p = p.offset(1);
            *fresh7 = b'\0' as WCHAR;
        }
        drive += 1;
    }
    *p = b'\0' as WCHAR;

    return count * 4;
}

#[no_mangle]
pub unsafe extern "system" fn GetLogicalDrives() -> u32 {
    let mut status: i32;
    let mut process_device_map_info: PROCESS_DEVICEMAP_INFORMATION = core::mem::zeroed();

    status = NtQueryInformationProcess(
        -1isize as *mut core::ffi::c_void,
        ProcessDeviceMap,
        &mut process_device_map_info.__bindgen_anon_1.Query as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<C2RustUnnamed_11>() as u32,
        core::ptr::null_mut(),
    );

    if !(status >= 0) {
        BaseSetLastNTError(status);
        return 0;
    }

    if process_device_map_info.__bindgen_anon_1.Query.DriveMap == 0 {
        SetLastError(0);
    }

    return process_device_map_info.__bindgen_anon_1.Query.DriveMap;
}

#[no_mangle]
pub unsafe extern "system" fn GetDiskFreeSpaceA(
    lpRootPathName: LPCSTR,
    lpSectorsPerCluster: LPDWORD,
    lpBytesPerSector: LPDWORD,
    lpNumberOfFreeClusters: LPDWORD,
    lpTotalNumberOfClusters: LPDWORD,
) -> BOOL {
    let mut RootPath: PCSTR = core::ptr::null();
    let mut RootPathU: PUNICODE_STRING = core::ptr::null_mut();

    RootPath = lpRootPathName;
    if RootPath.is_null() {
        RootPath = b"\\\0".as_ptr() as PCSTR;
    }

    RootPathU = Basep8BitStringToStaticUnicodeString(RootPath);
    if RootPathU.is_null() {
        return FALSE as BOOL;
    }

    return GetDiskFreeSpaceW(
        (*RootPathU).Buffer,
        lpSectorsPerCluster,
        lpBytesPerSector,
        lpNumberOfFreeClusters,
        lpTotalNumberOfClusters,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetDiskFreeSpaceW(
    lpRootPathName: LPCWSTR,
    lpSectorsPerCluster: LPDWORD,
    lpBytesPerSector: LPDWORD,
    lpNumberOfFreeClusters: LPDWORD,
    lpTotalNumberOfClusters: LPDWORD,
) -> BOOL {
    let mut Below2GB: BOOL;
    let mut RootPath: PCWSTR;
    let mut Status: NTSTATUS;
    let mut RootHandle: HANDLE = core::ptr::null_mut();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileFsSize: FILE_FS_SIZE_INFORMATION = core::mem::zeroed();

    /* If no path provided, get root path */
    RootPath = lpRootPathName;
    if lpRootPathName.is_null() {
        RootPath = w!("\\").as_ptr();
    }

    /* Convert the path to NT path */
    if RtlDosPathNameToNtPathName_U(RootPath, &mut FileName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return FALSE as BOOL;
    }

    /* Open it for disk space query! */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenFile(
        &mut RootHandle,
        SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_FREE_SPACE_QUERY,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
        if !lpBytesPerSector.is_null() {
            *lpBytesPerSector = 0;
        }
        return FALSE as BOOL;
    }

    /* We don't need the name any longer */
    RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);

    /* Query disk space! */
    Status = NtQueryVolumeInformationFile(
        RootHandle,
        &mut IoStatusBlock,
        &mut FileFsSize as *mut FILE_FS_SIZE_INFORMATION as PVOID,
        core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as ULONG,
        FileFsSizeInformation,
    );
    NtClose(RootHandle);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    /* Are we in some compatibility mode where size must be below 2GB? */
    Below2GB = (((*NtCurrentPeb()).AppCompatFlags.__bindgen_anon_1.LowPart & GetDiskFreeSpace2GB) == GetDiskFreeSpace2GB) as BOOL;

    /* If we're to overflow output, make sure we return the maximum */
    if FileFsSize.TotalAllocationUnits.__bindgen_anon_1.HighPart != 0 {
        FileFsSize.TotalAllocationUnits.__bindgen_anon_1.LowPart = -1i32 as DWORD;
    }

    if FileFsSize.AvailableAllocationUnits.__bindgen_anon_1.HighPart != 0 {
        FileFsSize.AvailableAllocationUnits.__bindgen_anon_1.LowPart = -1i32 as DWORD;
    }

    /* Return what user asked for */
    if !lpSectorsPerCluster.is_null() {
        *lpSectorsPerCluster = FileFsSize.SectorsPerAllocationUnit;
    }

    if !lpBytesPerSector.is_null() {
        *lpBytesPerSector = FileFsSize.BytesPerSector;
    }

    if !lpNumberOfFreeClusters.is_null() {
        if Below2GB == 0 {
            *lpNumberOfFreeClusters = FileFsSize.AvailableAllocationUnits.__bindgen_anon_1.LowPart;
        }
        /* If we have to remain below 2GB... */
        else {
            let mut FreeClusters: DWORD;

            /* Compute how many clusters there are in less than 2GB: 2 * 1024 * 1024 * 1024- 1 */
            FreeClusters = (0x7FFFFFFF as ULONG).wrapping_div(
                FileFsSize.SectorsPerAllocationUnit.wrapping_mul(FileFsSize.BytesPerSector),
            ) as DWORD;
            /* If that's higher than what was queried, then return the queried value, it's OK! */
            if FreeClusters > FileFsSize.AvailableAllocationUnits.__bindgen_anon_1.LowPart {
                FreeClusters = FileFsSize.AvailableAllocationUnits.__bindgen_anon_1.LowPart;
            }

            *lpNumberOfFreeClusters = FreeClusters;
        }
    }

    if !lpTotalNumberOfClusters.is_null() {
        if Below2GB == 0 {
            *lpTotalNumberOfClusters = FileFsSize.TotalAllocationUnits.__bindgen_anon_1.LowPart;
        }
        /* If we have to remain below 2GB... */
        else {
            let mut TotalClusters: DWORD;

            /* Compute how many clusters there are in less than 2GB: 2 * 1024 * 1024 * 1024- 1 */
            TotalClusters = (0x7FFFFFFF as ULONG).wrapping_div(
                FileFsSize.SectorsPerAllocationUnit.wrapping_mul(FileFsSize.BytesPerSector),
            ) as DWORD;
            /* If that's higher than what was queried, then return the queried value, it's OK! */
            if TotalClusters > FileFsSize.TotalAllocationUnits.__bindgen_anon_1.LowPart {
                TotalClusters = FileFsSize.TotalAllocationUnits.__bindgen_anon_1.LowPart;
            }

            *lpTotalNumberOfClusters = TotalClusters;
        }
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetDiskFreeSpaceExA(
    lpDirectoryName: LPCSTR,
    lpFreeBytesAvailableToCaller: PULARGE_INTEGER,
    lpTotalNumberOfBytes: PULARGE_INTEGER,
    lpTotalNumberOfFreeBytes: PULARGE_INTEGER,
) -> BOOL {
    let mut RootPath: PCSTR = core::ptr::null();
    let mut RootPathU: PUNICODE_STRING = core::ptr::null_mut();

    RootPath = lpDirectoryName;
    if RootPath.is_null() {
        RootPath = b"\\\0".as_ptr() as PCSTR;
    }

    RootPathU = Basep8BitStringToStaticUnicodeString(RootPath);
    if RootPathU.is_null() {
        return 0;
    }

    return GetDiskFreeSpaceExW(
        (*RootPathU).Buffer,
        lpFreeBytesAvailableToCaller,
        lpTotalNumberOfBytes,
        lpTotalNumberOfFreeBytes,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetDiskFreeSpaceExW(
    lpDirectoryName: LPCWSTR,
    lpFreeBytesAvailableToCaller: PULARGE_INTEGER,
    lpTotalNumberOfBytes: PULARGE_INTEGER,
    lpTotalNumberOfFreeBytes: PULARGE_INTEGER,
) -> BOOL {
    let mut RootPath: PCWSTR;
    let mut Status: NTSTATUS;
    let mut RootHandle: HANDLE = core::ptr::null_mut();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut BytesPerAllocationUnit: DWORD;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileFsSize: FILE_FS_SIZE_INFORMATION = core::mem::zeroed();

    RootPath = lpDirectoryName;
    if lpDirectoryName.is_null() {
        RootPath = b"\\\0".as_ptr() as PCWSTR;
    }

    if RtlDosPathNameToNtPathName_U(
        RootPath,
        &mut FileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut RootHandle,
        SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_FREE_SPACE_QUERY,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        if GetLastError() == ERROR_FILE_NOT_FOUND {
            SetLastError(ERROR_PATH_NOT_FOUND);
        }
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
        return 0;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);

    if !lpTotalNumberOfFreeBytes.is_null() {
        let mut FileFsFullSize: FILE_FS_FULL_SIZE_INFORMATION = core::mem::zeroed();

        Status = NtQueryVolumeInformationFile(
            RootHandle,
            &mut IoStatusBlock,
            &mut FileFsFullSize as *mut _ as PVOID,
            core::mem::size_of::<FILE_FS_FULL_SIZE_INFORMATION>() as ULONG,
            FileFsFullSizeInformation,
        );

        if NT_SUCCESS(Status) {
            NtClose(RootHandle);

            BytesPerAllocationUnit = FileFsFullSize.SectorsPerAllocationUnit
                * FileFsFullSize.BytesPerSector;

            if !lpFreeBytesAvailableToCaller.is_null() {
                (*lpFreeBytesAvailableToCaller).QuadPart = (FileFsFullSize
                    .CallerAvailableAllocationUnits
                    .QuadPart as i64
                    * BytesPerAllocationUnit as i64) as u64;
            }

            if !lpTotalNumberOfBytes.is_null() {
                (*lpTotalNumberOfBytes).QuadPart = (FileFsFullSize.TotalAllocationUnits.QuadPart as i64
                    * BytesPerAllocationUnit as i64) as u64;
            }

            (*lpTotalNumberOfFreeBytes).QuadPart = (FileFsFullSize
                .ActualAvailableAllocationUnits
                .QuadPart as i64
                * BytesPerAllocationUnit as i64) as u64;

            return 1;
        }
    }

    Status = NtQueryVolumeInformationFile(
        RootHandle,
        &mut IoStatusBlock,
        &mut FileFsSize as *mut _ as PVOID,
        core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as ULONG,
        FileFsSizeInformation,
    );

    NtClose(RootHandle);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    BytesPerAllocationUnit = FileFsSize.SectorsPerAllocationUnit * FileFsSize.BytesPerSector;

    if !lpFreeBytesAvailableToCaller.is_null() {
        (*lpFreeBytesAvailableToCaller).QuadPart =
            (FileFsSize.AvailableAllocationUnits.QuadPart as i64 * BytesPerAllocationUnit as i64) as u64;
    }

    if !lpTotalNumberOfBytes.is_null() {
        (*lpTotalNumberOfBytes).QuadPart =
            (FileFsSize.TotalAllocationUnits.QuadPart as i64 * BytesPerAllocationUnit as i64) as u64;
    }

    if !lpTotalNumberOfFreeBytes.is_null() {
        (*lpTotalNumberOfFreeBytes).QuadPart =
            (FileFsSize.AvailableAllocationUnits.QuadPart as i64 * BytesPerAllocationUnit as i64) as u64;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetDriveTypeA(lpRootPathName: LPCSTR) -> UINT {
    let mut RootPathU: PWSTR = core::ptr::null_mut();
    if !lpRootPathName.is_null() {
        let mut RootPathUStr: PUNICODE_STRING = core::ptr::null_mut();
        RootPathUStr = Basep8BitStringToStaticUnicodeString(lpRootPathName);
        if RootPathUStr.is_null() {
            return DRIVE_NO_ROOT_DIR as UINT;
        }
        RootPathU = (*RootPathUStr).Buffer;
    } else {
        RootPathU = core::ptr::null_mut();
    }
    return GetDriveTypeW(RootPathU as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> UINT {
    let mut RetryOpen: BOOL = 0;
    let mut RootPath: PCWSTR = core::ptr::null();
    let mut Status: NTSTATUS = 0;
    let mut DriveLetter: WCHAR = 0;
    let mut RootHandle: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut PathName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut VolumeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut FileFsDevice: FILE_FS_DEVICE_INFORMATION = FILE_FS_DEVICE_INFORMATION {
        DeviceType: 0,
        Characteristics: 0,
    };
    let mut Buffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut VolumeName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if lpRootPathName.is_null() {
        RootPath = Buffer.as_mut_ptr() as PCWSTR;
        if RtlGetCurrentDirectory_U(
            core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as ULONG,
            Buffer.as_mut_ptr() as PWSTR,
        ) as usize > (3 as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
        {
            Buffer[3] = UNICODE_NULL;
        }
    } else {
        if lpRootPathName as isize == -1 {
            return DRIVE_UNKNOWN as UINT;
        }

        RootPath = lpRootPathName as PCWSTR;
        if wcslen(lpRootPathName as *const wchar_t) == 2 {
            DriveLetter = RtlUpcaseUnicodeChar(*lpRootPathName.offset(0));
            if DriveLetter as i32 >= 'A' as i32
                && DriveLetter as i32 <= 'Z' as i32
                && *lpRootPathName.offset(1) as i32 == ':' as i32
            {
                Buffer[0] = DriveLetter;
                Buffer[1] = ':' as WCHAR;
                Buffer[2] = '\\' as WCHAR;
                Buffer[3] = UNICODE_NULL;
                RootPath = Buffer.as_mut_ptr() as PCWSTR;
            }
        }
    }

    DriveLetter = RtlUpcaseUnicodeChar(*RootPath.offset(0));
    if DriveLetter as i32 >= 'A' as i32
        && DriveLetter as i32 <= 'Z' as i32
        && *RootPath.offset(1) as i32 == ':' as i32
        && *RootPath.offset(2) as i32 == '\\' as i32
        && *RootPath.offset(3) as i32 == UNICODE_NULL as i32
    {
        let mut Index: USHORT = 0;
        let mut DeviceMap: PROCESS_DEVICEMAP_INFORMATION = core::mem::zeroed();
        DeviceMap.__bindgen_anon_1.Query = C2RustUnnamed_11 {
            DriveMap: 0,
            DriveType: [0; 32],
        };

        Status = NtQueryInformationProcess(
            -1 as LONG_PTR as HANDLE,
            ProcessDeviceMap,
            &mut DeviceMap.__bindgen_anon_1.Query as *mut _ as PVOID,
            core::mem::size_of::<C2RustUnnamed_11>() as ULONG,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            core::ptr::write_bytes(
                &mut DeviceMap as *mut _ as PVOID,
                0,
                core::mem::size_of::<PROCESS_DEVICEMAP_INFORMATION>(),
            );
        }

        Index = (DriveLetter as i32 - 'A' as i32) as USHORT;
        if ((1 as ULONG) << Index as i32) & DeviceMap.__bindgen_anon_1.Query.DriveMap != 0 {
            if DeviceMap.__bindgen_anon_1.Query.DriveType[Index as usize] as i32 >= DRIVE_REMOVABLE as i32
                && DeviceMap.__bindgen_anon_1.Query.DriveType[Index as usize] as i32 <= DRIVE_RAMDISK as i32
            {
                return DeviceMap.__bindgen_anon_1.Query.DriveType[Index as usize] as UINT;
            } else {
                return DRIVE_UNKNOWN as UINT;
            }
        }
    }

    if lpRootPathName.is_null() {
        static BACKSLASH: [WCHAR; 2] = ['\\' as WCHAR, UNICODE_NULL];
        RootPath = BACKSLASH.as_ptr() as PCWSTR;
    }

    if RtlDosPathNameToNtPathName_U(
        RootPath,
        &mut PathName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        return DRIVE_NO_ROOT_DIR as UINT;
    }

    if *PathName.Buffer.offset(
        (PathName.Length as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_sub(1) as isize,
    ) as i32 != '\\' as i32
    {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0 as ULONG,
            PathName.Buffer as PVOID,
        );
        return DRIVE_NO_ROOT_DIR as UINT;
    }

    PathName.Length = (PathName.Length as usize)
        .wrapping_sub(core::mem::size_of::<WCHAR>() as usize) as USHORT;

    RetryOpen = 1;
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.ObjectName = &mut PathName as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut RootHandle,
        (SYNCHRONIZE | FILE_READ_ATTRIBUTES) as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        (FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE) as ULONG,
    );

    if Status == STATUS_FILE_IS_A_DIRECTORY {
        if BasepGetVolumeNameFromReparsePoint(
            lpRootPathName,
            VolumeName.as_mut_ptr() as LPWSTR,
            MAX_PATH as DWORD,
            core::ptr::null_mut(),
        ) != 0
        {
            RtlInitUnicodeString(
                &mut VolumeString,
                VolumeName.as_mut_ptr() as PCWSTR,
            );
            VolumeName[1] = '?' as WCHAR;
            VolumeString.Length = (VolumeString.Length as usize)
                .wrapping_sub(core::mem::size_of::<WCHAR>() as usize) as USHORT;
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
            ObjectAttributes.ObjectName = &mut VolumeString as PUNICODE_STRING;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
        }
    } else {
        PathName.Length = (PathName.Length as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>() as usize) as USHORT;
        if IsThisARootDirectory(
            core::ptr::null_mut(),
            &mut PathName,
        ) != 0
        {
            RetryOpen = 0;
        } else {
            PathName.Length = (PathName.Length as usize)
                .wrapping_sub(core::mem::size_of::<WCHAR>() as usize) as USHORT;
            if NT_SUCCESS(Status) {
                NtClose(RootHandle);
            }
        }
    }

    if RetryOpen != 0 {
        Status = NtOpenFile(
            &mut RootHandle,
            (SYNCHRONIZE | FILE_READ_ATTRIBUTES) as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
            FILE_SYNCHRONOUS_IO_NONALERT as ULONG,
        );
    }

    RtlFreeHeap(
        RtlGetProcessHeap(),
        0 as ULONG,
        PathName.Buffer as PVOID,
    );

    if !NT_SUCCESS(Status) {
        return DRIVE_NO_ROOT_DIR as UINT;
    }

    Status = NtQueryVolumeInformationFile(
        RootHandle,
        &mut IoStatusBlock,
        &mut FileFsDevice as *mut _ as PVOID,
        core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
        FileFsDeviceInformation,
    );

    NtClose(RootHandle);

    if !NT_SUCCESS(Status) {
        return DRIVE_UNKNOWN as UINT;
    }

    if (FileFsDevice.Characteristics as ULONG & FILE_REMOTE_DEVICE as ULONG)
        == FILE_REMOTE_DEVICE as ULONG
    {
        return DRIVE_REMOTE as UINT;
    }

    match FileFsDevice.DeviceType {
        2 | 3 => return DRIVE_CDROM as UINT,
        7 | 8 => {
            if (FileFsDevice.Characteristics as ULONG & FILE_REMOVABLE_MEDIA as ULONG)
                == FILE_REMOVABLE_MEDIA as ULONG
                || (FileFsDevice.Characteristics as ULONG & FILE_FLOPPY_DISKETTE as ULONG)
                    == FILE_FLOPPY_DISKETTE as ULONG
            {
                return DRIVE_REMOVABLE as UINT;
            } else {
                return DRIVE_FIXED as UINT;
            }
        }
        18 | 20 => return DRIVE_REMOTE as UINT,
        36 => return DRIVE_RAMDISK as UINT,
        _ => {}
    }

    return DRIVE_UNKNOWN as UINT;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/disk.c
//  * PURPOSE:         Disk and Drive functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  *                  Erik Bos, Alexandre Julliard :
//  *                      GetLogicalDriveStringsA,
//  *                      GetLogicalDriveStringsW, GetLogicalDrives
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
//
// #define NDEBUG
// #include <debug.h>
//
// #define MAX_DOS_DRIVES 26
//
// /*
//  * @implemented
//  */
// /* Synced to Wine-2008/12/28 */
// DWORD
// WINAPI
// GetLogicalDriveStringsA(IN DWORD nBufferLength,
//                         IN LPSTR lpBuffer)
// {
//     DWORD drive, count;
//     DWORD dwDriveMap;
//     LPSTR p;
//
//     dwDriveMap = GetLogicalDrives();
//
//     for (drive = count = 0; drive < MAX_DOS_DRIVES; drive++)
//     {
//         if (dwDriveMap & (1<<drive))
//             count++;
//     }
//
//
//     if ((count * 4) + 1 > nBufferLength) return ((count * 4) + 1);
//
//     p = lpBuffer;
//
//     for (drive = 0; drive < MAX_DOS_DRIVES; drive++)
//         if (dwDriveMap & (1<<drive))
//         {
//             *p++ = 'A' + (UCHAR)drive;
//             *p++ = ':';
//             *p++ = '\\';
//             *p++ = '\0';
//         }
//     *p = '\0';
//
//     return (count * 4);
// }
//
// /*
//  * @implemented
//  */
// /* Synced to Wine-2008/12/28 */
// DWORD
// WINAPI
// GetLogicalDriveStringsW(IN DWORD nBufferLength,
//                         IN LPWSTR lpBuffer)
// {
//     DWORD drive, count;
//     DWORD dwDriveMap;
//     LPWSTR p;
//
//     dwDriveMap = GetLogicalDrives();
//
//     for (drive = count = 0; drive < MAX_DOS_DRIVES; drive++)
//     {
//         if (dwDriveMap & (1<<drive))
//             count++;
//     }
//
//     if ((count * 4) + 1 > nBufferLength) return ((count * 4) + 1);
//
//     p = lpBuffer;
//     for (drive = 0; drive < MAX_DOS_DRIVES; drive++)
//         if (dwDriveMap & (1<<drive))
//         {
//             *p++ = (WCHAR)('A' + drive);
//             *p++ = (WCHAR)':';
//             *p++ = (WCHAR)'\\';
//             *p++ = (WCHAR)'\0';
//         }
//     *p = (WCHAR)'\0';
//
//     return (count * 4);
// }
//
// /*
//  * @implemented
//  */
// /* Synced to Wine-? */
// DWORD
// WINAPI
// GetLogicalDrives(VOID)
// {
//     NTSTATUS Status;
//     PROCESS_DEVICEMAP_INFORMATION ProcessDeviceMapInfo;
//
//     /* Get the Device Map for this Process */
//     Status = NtQueryInformationProcess(NtCurrentProcess(),
//                                        ProcessDeviceMap,
//                                        &ProcessDeviceMapInfo.__bindgen_anon_1.Query,
//                                        sizeof(ProcessDeviceMapInfo.__bindgen_anon_1.Query),
//                                        NULL);
//
//     /* Return the Drive Map */
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     if (ProcessDeviceMapInfo.__bindgen_anon_1.Query.DriveMap == 0)
//     {
//         SetLastError(ERROR_SUCCESS);
//     }
//
//     return ProcessDeviceMapInfo.__bindgen_anon_1.Query.DriveMap;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetDiskFreeSpaceA(IN LPCSTR lpRootPathName,
//                   OUT LPDWORD lpSectorsPerCluster,
//                   OUT LPDWORD lpBytesPerSector,
//                   OUT LPDWORD lpNumberOfFreeClusters,
//                   OUT LPDWORD lpTotalNumberOfClusters)
// {
//     PCSTR RootPath;
//     PUNICODE_STRING RootPathU;
//
//     RootPath = lpRootPathName;
//     if (RootPath == NULL)
//     {
//         RootPath = "\\";
//     }
//
//     RootPathU = Basep8BitStringToStaticUnicodeString(RootPath);
//     if (RootPathU == NULL)
//     {
//         return FALSE as BOOL;
//     }
//
//     return GetDiskFreeSpaceW(RootPathU->Buffer, lpSectorsPerCluster,
//                              lpBytesPerSector, lpNumberOfFreeClusters,
//                              lpTotalNumberOfClusters);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetDiskFreeSpaceW(IN LPCWSTR lpRootPathName,
//                   OUT LPDWORD lpSectorsPerCluster,
//                   OUT LPDWORD lpBytesPerSector,
//                   OUT LPDWORD lpNumberOfFreeClusters,
//                   OUT LPDWORD lpTotalNumberOfClusters)
// {
//     BOOL Below2GB;
//     PCWSTR RootPath;
//     NTSTATUS Status;
//     HANDLE RootHandle;
//     UNICODE_STRING FileName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     FILE_FS_SIZE_INFORMATION FileFsSize;
//
//     /* If no path provided, get root path */
//     RootPath = lpRootPathName;
//     if (lpRootPathName == NULL)
//     {
//         RootPath = L"\\";
//     }
//
//     /* Convert the path to NT path */
//     if (!RtlDosPathNameToNtPathName_U(RootPath, &FileName, NULL, NULL))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE as BOOL;
//     }
//
//     /* Open it for disk space query! */
//     InitializeObjectAttributes(&ObjectAttributes, &FileName,
//                                OBJ_CASE_INSENSITIVE, NULL, NULL);
//     Status = NtOpenFile(&RootHandle, SYNCHRONIZE, &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_FREE_SPACE_QUERY);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//         if (lpBytesPerSector != NULL)
//         {
//             *lpBytesPerSector = 0;
//         }
//
//         return FALSE as BOOL;
//     }
//
//     /* We don't need the name any longer */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//
//     /* Query disk space! */
//     Status = NtQueryVolumeInformationFile(RootHandle, &IoStatusBlock, &FileFsSize,
//                                           sizeof(FILE_FS_SIZE_INFORMATION),
//                                           FileFsSizeInformation);
//     NtClose(RootHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE as BOOL;
//     }
//
//     /* Are we in some compatibility mode where size must be below 2GB? */
//     Below2GB = ((NtCurrentPeb()->AppCompatFlags.LowPart & GetDiskFreeSpace2GB) == GetDiskFreeSpace2GB);
//
//     /* If we're to overflow output, make sure we return the maximum */
//     if (FileFsSize.TotalAllocationUnits.HighPart != 0)
//     {
//         FileFsSize.TotalAllocationUnits.LowPart = -1;
//     }
//
//     if (FileFsSize.AvailableAllocationUnits.HighPart != 0)
//     {
//         FileFsSize.AvailableAllocationUnits.LowPart = -1;
//     }
//
//     /* Return what user asked for */
//     if (lpSectorsPerCluster != NULL)
//     {
//         *lpSectorsPerCluster = FileFsSize.SectorsPerAllocationUnit;
//     }
//
//     if (lpBytesPerSector != NULL)
//     {
//         *lpBytesPerSector = FileFsSize.BytesPerSector;
//     }
//
//     if (lpNumberOfFreeClusters != NULL)
//     {
//         if (!Below2GB)
//         {
//             *lpNumberOfFreeClusters = FileFsSize.AvailableAllocationUnits.LowPart;
//         }
//         /* If we have to remain below 2GB... */
//         else
//         {
//             DWORD FreeClusters;
//
//             /* Compute how many clusters there are in less than 2GB: 2 * 1024 * 1024 * 1024- 1 */
//             FreeClusters = 0x7FFFFFFF / (FileFsSize.SectorsPerAllocationUnit * FileFsSize.BytesPerSector);
//             /* If that's higher than what was queried, then return the queried value, it's OK! */
//             if (FreeClusters > FileFsSize.AvailableAllocationUnits.LowPart)
//             {
//                 FreeClusters = FileFsSize.AvailableAllocationUnits.LowPart;
//             }
//
//             *lpNumberOfFreeClusters = FreeClusters;
//         }
//     }
//
//     if (lpTotalNumberOfClusters != NULL)
//     {
//         if (!Below2GB)
//         {
//             *lpTotalNumberOfClusters = FileFsSize.TotalAllocationUnits.LowPart;
//         }
//         /* If we have to remain below 2GB... */
//         else
//         {
//             DWORD TotalClusters;
//
//             /* Compute how many clusters there are in less than 2GB: 2 * 1024 * 1024 * 1024- 1 */
//             TotalClusters = 0x7FFFFFFF / (FileFsSize.SectorsPerAllocationUnit * FileFsSize.BytesPerSector);
//             /* If that's higher than what was queried, then return the queried value, it's OK! */
//             if (TotalClusters > FileFsSize.TotalAllocationUnits.LowPart)
//             {
//                 TotalClusters = FileFsSize.TotalAllocationUnits.LowPart;
//             }
//
//             *lpTotalNumberOfClusters = TotalClusters;
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
// GetDiskFreeSpaceExA(IN LPCSTR lpDirectoryName OPTIONAL,
//                     OUT PULARGE_INTEGER lpFreeBytesAvailableToCaller,
//                     OUT PULARGE_INTEGER lpTotalNumberOfBytes,
//                     OUT PULARGE_INTEGER lpTotalNumberOfFreeBytes)
// {
//     PCSTR RootPath;
//     PUNICODE_STRING RootPathU;
//
//     RootPath = lpDirectoryName;
//     if (RootPath == NULL)
//     {
//         RootPath = "\\";
//     }
//
//     RootPathU = Basep8BitStringToStaticUnicodeString(RootPath);
//     if (RootPathU == NULL)
//     {
//         return FALSE as BOOL;
//     }
//
//     return GetDiskFreeSpaceExW(RootPathU->Buffer, lpFreeBytesAvailableToCaller,
//                               lpTotalNumberOfBytes, lpTotalNumberOfFreeBytes);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetDiskFreeSpaceExW(IN LPCWSTR lpDirectoryName OPTIONAL,
//                     OUT PULARGE_INTEGER lpFreeBytesAvailableToCaller,
//                     OUT PULARGE_INTEGER lpTotalNumberOfBytes,
//                     OUT PULARGE_INTEGER lpTotalNumberOfFreeBytes)
// {
//     PCWSTR RootPath;
//     NTSTATUS Status;
//     HANDLE RootHandle;
//     UNICODE_STRING FileName;
//     DWORD BytesPerAllocationUnit;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     FILE_FS_SIZE_INFORMATION FileFsSize;
//
//     /* If no path provided, get root path */
//     RootPath = lpDirectoryName;
//     if (lpDirectoryName == NULL)
//     {
//         RootPath = L"\\";
//     }
//
//     /* Convert the path to NT path */
//     if (!RtlDosPathNameToNtPathName_U(RootPath, &FileName, NULL, NULL))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE as BOOL;
//     }
//
//     /* Open it for disk space query! */
//     InitializeObjectAttributes(&ObjectAttributes, &FileName,
//                                OBJ_CASE_INSENSITIVE, NULL, NULL);
//     Status = NtOpenFile(&RootHandle, SYNCHRONIZE, &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_FREE_SPACE_QUERY);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         /* If error conversion lead to file not found, override to use path not found
//          * which is more accurate
//          */
//         if (GetLastError() == ERROR_FILE_NOT_FOUND)
//         {
//             SetLastError(ERROR_PATH_NOT_FOUND);
//         }
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//
//         return FALSE as BOOL;
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer);
//
//     /* If user asks for lpTotalNumberOfFreeBytes, try to use full size information */
//     if (lpTotalNumberOfFreeBytes != NULL)
//     {
//         FILE_FS_FULL_SIZE_INFORMATION FileFsFullSize;
//
//         /* Issue the full fs size request */
//         Status = NtQueryVolumeInformationFile(RootHandle, &IoStatusBlock, &FileFsFullSize,
//                                               sizeof(FILE_FS_FULL_SIZE_INFORMATION),
//                                               FileFsFullSizeInformation);
//         /* If it succeed, complete out buffers */
//         if (NT_SUCCESS(Status))
//         {
//             /* We can close here, we'll return */
//             NtClose(RootHandle);
//
//             /* Compute the size of an AU */
//             BytesPerAllocationUnit = FileFsFullSize.SectorsPerAllocationUnit * FileFsFullSize.BytesPerSector;
//
//             /* And then return what was asked */
//             if (lpFreeBytesAvailableToCaller != NULL)
//             {
//                 lpFreeBytesAvailableToCaller->QuadPart = FileFsFullSize.CallerAvailableAllocationUnits.QuadPart * BytesPerAllocationUnit;
//             }
//
//             if (lpTotalNumberOfBytes != NULL)
//             {
//                 lpTotalNumberOfBytes->QuadPart = FileFsFullSize.TotalAllocationUnits.QuadPart * BytesPerAllocationUnit;
//             }
//
//             /* No need to check for nullness ;-) */
//             lpTotalNumberOfFreeBytes->QuadPart = FileFsFullSize.ActualAvailableAllocationUnits.QuadPart * BytesPerAllocationUnit;
//
//             return TRUE;
//         }
//     }
//
//     /* Otherwise, fallback to normal size information */
//     Status = NtQueryVolumeInformationFile(RootHandle, &IoStatusBlock,
//                                           &FileFsSize, sizeof(FILE_FS_SIZE_INFORMATION),
//                                           FileFsSizeInformation);
//     NtClose(RootHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE as BOOL;
//     }
//
//     /* Compute the size of an AU */
//     BytesPerAllocationUnit = FileFsSize.SectorsPerAllocationUnit * FileFsSize.BytesPerSector;
//
//     /* And then return what was asked, available is free, the same! */
//     if (lpFreeBytesAvailableToCaller != NULL)
//     {
//         lpFreeBytesAvailableToCaller->QuadPart = FileFsSize.AvailableAllocationUnits.QuadPart * BytesPerAllocationUnit;
//     }
//
//     if (lpTotalNumberOfBytes != NULL)
//     {
//         lpTotalNumberOfBytes->QuadPart = FileFsSize.TotalAllocationUnits.QuadPart * BytesPerAllocationUnit;
//     }
//
//     if (lpTotalNumberOfFreeBytes != NULL)
//     {
//         lpTotalNumberOfFreeBytes->QuadPart = FileFsSize.AvailableAllocationUnits.QuadPart * BytesPerAllocationUnit;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// GetDriveTypeA(IN LPCSTR lpRootPathName)
// {
//     PWSTR RootPathU;
//
//     if (lpRootPathName != NULL)
//     {
//         PUNICODE_STRING RootPathUStr;
//
//         RootPathUStr = Basep8BitStringToStaticUnicodeString(lpRootPathName);
//         if (RootPathUStr == NULL)
//         {
//             return DRIVE_NO_ROOT_DIR;
//         }
//
//         RootPathU = RootPathUStr->Buffer;
//     }
//     else
//     {
//         RootPathU = NULL;
//     }
//
//     return GetDriveTypeW(RootPathU);
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// GetDriveTypeW(IN LPCWSTR lpRootPathName)
// {
//     BOOL RetryOpen;
//     PCWSTR RootPath;
//     NTSTATUS Status;
//     WCHAR DriveLetter;
//     HANDLE RootHandle;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING PathName, VolumeString;
//     FILE_FS_DEVICE_INFORMATION FileFsDevice;
//     WCHAR Buffer[MAX_PATH], VolumeName[MAX_PATH];
//
//     /* If no path, get one */
//     if (lpRootPathName == NULL)
//     {
//         RootPath = Buffer;
//         /* This will be current drive (<letter>:\ - drop the rest)*/
//         if (RtlGetCurrentDirectory_U(sizeof(Buffer), Buffer) > 3 * sizeof(WCHAR))
//         {
//             Buffer[3] = UNICODE_NULL;
//         }
//     }
//     else
//     {
//         /* Handle broken value */
//         if (lpRootPathName == (PVOID)-1)
//         {
//             return DRIVE_UNKNOWN;
//         }
//
//         RootPath = lpRootPathName;
//         /* If provided path is 2-len, it might be a drive letter... */
//         if (wcslen(lpRootPathName) == 2)
//         {
//             /* Check it! */
//             DriveLetter = RtlUpcaseUnicodeChar(lpRootPathName[0]);
//             /* That's a drive letter! */
//             if (DriveLetter >= L'A' && DriveLetter <= L'Z' && lpRootPathName[1] == L':')
//             {
//                 /* Make it a volume */
//                 Buffer[0] = DriveLetter;
//                 Buffer[1] = L':';
//                 Buffer[2] = L'\\';
//                 Buffer[3] = UNICODE_NULL;
//                 RootPath = Buffer;
//             }
//         }
//     }
//
//     /* If the provided looks like a DOS device... Like <letter>:\<0> */
//     DriveLetter = RtlUpcaseUnicodeChar(RootPath[0]);
//     /* We'll take the quick path!
//      * We'll find the device type looking at the device map (and types ;-))
//      * associated with the current process
//      */
//     if (DriveLetter >= L'A' && DriveLetter <= L'Z' && RootPath[1] == L':' &&
//         RootPath[2] == L'\\' && RootPath[3] == UNICODE_NULL)
//     {
//         USHORT Index;
//         PROCESS_DEVICEMAP_INFORMATION DeviceMap;
//
//         /* Query the device map */
//         Status = NtQueryInformationProcess(NtCurrentProcess(),
//                                            ProcessDeviceMap,
//                                            &DeviceMap.__bindgen_anon_1.Query,
//                                            sizeof(DeviceMap.__bindgen_anon_1.Query),
//                                            NULL);
//         /* Zero output if we failed */
//         if (!NT_SUCCESS(Status))
//         {
//             RtlZeroMemory(&DeviceMap, sizeof(PROCESS_DEVICEMAP_INFORMATION));
//         }
//
//         /* Get our index in the device map */
//         Index = DriveLetter - L'A';
//         /* Check we're in the device map (bit set) */
//         if (((1 << Index) & DeviceMap.__bindgen_anon_1.Query.DriveMap) != 0)
//         {
//             /* Validate device type and return it */
//             if (DeviceMap.__bindgen_anon_1.Query.DriveType[Index] >= DRIVE_REMOVABLE &&
//                 DeviceMap.__bindgen_anon_1.Query.DriveType[Index] <= DRIVE_RAMDISK)
//             {
//                 return DeviceMap.__bindgen_anon_1.Query.DriveType[Index];
//             }
//             /* Otherwise, return we don't know the type */
//             else
//             {
//                 return DRIVE_UNKNOWN;
//             }
//         }
//
//         /* We couldn't find ourselves, do it the slow way */
//     }
//
//     /* No path provided, use root */
//     if (lpRootPathName == NULL)
//     {
//         RootPath = L"\\";
//     }
//
//     /* Convert to NT path */
//     if (!RtlDosPathNameToNtPathName_U(RootPath, &PathName, NULL, NULL))
//     {
//         return DRIVE_NO_ROOT_DIR;
//     }
//
//     /* If not a directory, fail, we need a volume */
//     if (PathName.Buffer[(PathName.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PathName.Buffer);
//         return DRIVE_NO_ROOT_DIR;
//     }
//
//     /* We will work with a device object, so trim the trailing backslash now */
//     PathName.Length -= sizeof(WCHAR);
//
//     /* Let's probe for it, by forcing open failure! */
//     RetryOpen = TRUE;
//     InitializeObjectAttributes(&ObjectAttributes, &PathName,
//                                OBJ_CASE_INSENSITIVE, NULL, NULL);
//     Status = NtOpenFile(&RootHandle, SYNCHRONIZE | FILE_READ_ATTRIBUTES,
//                         &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE);
//     /* It properly failed! */
//     if (Status == STATUS_FILE_IS_A_DIRECTORY)
//     {
//         /* It might be a mount point, then, query for target */
//         if (BasepGetVolumeNameFromReparsePoint(lpRootPathName, VolumeName, MAX_PATH, NULL))
//         {
//             /* We'll reopen the target */
//             RtlInitUnicodeString(&VolumeString, VolumeName);
//             VolumeName[1] = L'?';
//             VolumeString.Length -= sizeof(WCHAR);
//             InitializeObjectAttributes(&ObjectAttributes, &VolumeString,
//                                        OBJ_CASE_INSENSITIVE, NULL, NULL);
//         }
//     }
//     else
//     {
//         /* heh. It worked? Or failed for whatever other reason?
//          * Check we have a directory if we get farther in path
//          */
//         PathName.Length += sizeof(WCHAR);
//         if (IsThisARootDirectory(0, &PathName))
//         {
//             /* Yes? Heh, then it's fine, keep our current handle */
//             RetryOpen = FALSE;
//         }
//         else
//         {
//             /* Then, retry to open without forcing non directory type */
//             PathName.Length -= sizeof(WCHAR);
//             if (NT_SUCCESS(Status))
//             {
//                 NtClose(RootHandle);
//             }
//         }
//     }
//
//     /* Now, we retry without forcing file type - should work now */
//     if (RetryOpen)
//     {
//         Status = NtOpenFile(&RootHandle, SYNCHRONIZE | FILE_READ_ATTRIBUTES,
//                             &ObjectAttributes, &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             FILE_SYNCHRONOUS_IO_NONALERT);
//     }
//
//     /* We don't need path any longer */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathName.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         return DRIVE_NO_ROOT_DIR;
//     }
//
//     /* Query the device for its type */
//     Status = NtQueryVolumeInformationFile(RootHandle,
//                                           &IoStatusBlock,
//                                           &FileFsDevice,
//                                           sizeof(FILE_FS_DEVICE_INFORMATION),
//                                           FileFsDeviceInformation);
//     /* No longer required */
//     NtClose(RootHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         return DRIVE_UNKNOWN;
//     }
//
//     /* Do we have a remote device? Return so! */
//     if ((FileFsDevice.Characteristics & FILE_REMOTE_DEVICE) == FILE_REMOTE_DEVICE)
//     {
//         return DRIVE_REMOTE;
//     }
//
//     /* Check the device type */
//     switch (FileFsDevice.DeviceType)
//     {
//         /* CDROM, easy */
//         case FILE_DEVICE_CD_ROM:
//         case FILE_DEVICE_CD_ROM_FILE_SYSTEM:
//             return DRIVE_CDROM;
//
//         /* Disk... */
//         case FILE_DEVICE_DISK:
//         case FILE_DEVICE_DISK_FILE_SYSTEM:
//             /* Removable media? Floppy is one */
//             if ((FileFsDevice.Characteristics & FILE_REMOVABLE_MEDIA) == FILE_REMOVABLE_MEDIA ||
//                 (FileFsDevice.Characteristics & FILE_FLOPPY_DISKETTE) == FILE_FLOPPY_DISKETTE)
//             {
//                 return DRIVE_REMOVABLE;
//             }
//             else
//             {
//                 return DRIVE_FIXED;
//             }
//
//         /* Easy cases */
//         case FILE_DEVICE_NETWORK:
//         case FILE_DEVICE_NETWORK_FILE_SYSTEM:
//             return DRIVE_REMOTE;
//
//         case FILE_DEVICE_VIRTUAL_DISK:
//             return DRIVE_RAMDISK;
//     }
//
//     /* Nothing matching, just fail */
//     return DRIVE_UNKNOWN;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

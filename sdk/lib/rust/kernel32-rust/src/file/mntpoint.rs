use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

// kernel32-internal helpers (InitializeObjectAttributes, Base*, etc.).
use crate::k32_externs::*;
use crate::string::nls_subst::NT_SUCCESS;

// MOUNTMGR DDK surface (sdk/include/ddk/mountmgr.h) — not in the ntdll FFI.
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
pub type PMOUNTMGR_MOUNT_POINT = *mut MOUNTMGR_MOUNT_POINT;
pub type PMOUNTMGR_MOUNT_POINTS = *mut MOUNTMGR_MOUNT_POINTS;

// CTL_CODE(t,f,m,a) = (t<<16)|(a<<14)|(f<<2)|m ; METHOD_BUFFERED=0, FILE_ANY_ACCESS=0.
pub const IOCTL_MOUNTMGR_QUERY_POINTS: DWORD = 0x006D_0008; // 'm', func 2
pub const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: DWORD = 0x004D_0008; // 'M', func 2
// FSCTL_GET_REPARSE_POINT: FILE_DEVICE_FILE_SYSTEM=9, func 42.
pub const FSCTL_GET_REPARSE_POINT: DWORD = 0x0009_00A8;

// MOUNTMGR_DOS_DEVICE_NAME = L"\\.\MountPointManager"
pub static MOUNTMGR_DOS_DEVICE_NAME_BUF: [u16; 22] = crate::to_w("\\\\.\\MountPointManager");
#[allow(non_upper_case_globals)]
pub const MOUNTMGR_DOS_DEVICE_NAME: LPCWSTR = MOUNTMGR_DOS_DEVICE_NAME_BUF.as_ptr();

extern "system" {
    fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> DWORD;
    fn CreateFileW( lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: *mut core::ffi::c_void, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE, ) -> HANDLE;
    fn DeviceIoControl( hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: *mut core::ffi::c_void, nInBufferSize: DWORD, lpOutBuffer: *mut core::ffi::c_void, nOutBufferSize: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: *mut core::ffi::c_void, ) -> BOOL;
    fn GetLastError() -> DWORD;
    fn MOUNTMGR_IS_NT_VOLUME_NAME(VolumeName: PUNICODE_STRING) -> BOOLEAN;
    fn MOUNTMGR_IS_DOS_VOLUME_NAME_WB(VolumeName: PUNICODE_STRING) -> BOOLEAN;
    fn Basep8BitStringToStaticUnicodeString(lpString: LPCSTR) -> PUNICODE_STRING;
}
extern "C" {
    fn towupper(c: WCHAR) -> WCHAR;
}

// Local anonymous struct from GetVolumeNameForRoot:
//   struct { MOUNTDEV_NAME; WCHAR Buffer[MAX_PATH]; } MountDevName;
// MOUNTDEV_NAME is { USHORT NameLength; WCHAR Name[1]; }; the trailing
// Buffer[MAX_PATH] extends the inline Name array. Modelled flat.
#[repr(C)]
struct MountDevName {
    NameLength: USHORT,
    Name: [WCHAR; MAX_PATH as usize],
}

unsafe fn GetVolumeNameForRoot(
    lpszRootPath: LPCWSTR,
    lpszVolumeName: LPWSTR,
    cchBufferLength: DWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let mut Status: NTSTATUS;
    let mut FoundVolume: PWSTR = core::ptr::null_mut();
    let mut BytesReturned: DWORD = 0;
    let mut NtPathName: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut MountPoint: PMOUNTMGR_MOUNT_POINT;
    let mut CurrentMntPt: ULONG;
    let mut FoundVolumeLen: ULONG = 0;
    let mut MountPoints: PMOUNTMGR_MOUNT_POINTS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut VolumeHandle: HANDLE = core::ptr::null_mut();
    let mut MountMgrHandle: HANDLE;
    let mut MountDevName: MountDevName = core::mem::zeroed();

    /* It makes no sense on a non-local drive */
    if GetDriveTypeW(lpszRootPath) == DRIVE_REMOTE {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    /* Get the NT path */
    if RtlDosPathNameToNtPathName_U(lpszRootPath, &mut NtPathName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return 0;
    }

    /* If it's a root path - likely - drop backslash to open volume */
    if *NtPathName.Buffer.offset((NtPathName.Length / core::mem::size_of::<WCHAR>() as u16).wrapping_sub(1) as isize) == '\\' as u16 {
        *NtPathName.Buffer.offset((NtPathName.Length / core::mem::size_of::<WCHAR>() as u16).wrapping_sub(1) as isize) = 0u16;
        NtPathName.Length = NtPathName.Length.wrapping_sub(core::mem::size_of::<WCHAR>() as u16);
    }

    /* If that's a DOS volume, upper case the letter */
    if NtPathName.Length >= (2 * core::mem::size_of::<WCHAR>()) as u16 {
        if *NtPathName.Buffer.offset((NtPathName.Length / core::mem::size_of::<WCHAR>() as u16).wrapping_sub(1) as isize) == ':' as u16 {
            *NtPathName.Buffer.offset((NtPathName.Length / core::mem::size_of::<WCHAR>() as u16).wrapping_sub(2) as isize) = towupper(*NtPathName.Buffer.offset((NtPathName.Length / core::mem::size_of::<WCHAR>() as u16).wrapping_sub(2) as isize));
        }
    }

    /* Attempt to open the volume */
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtPathName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenFile(
        &mut VolumeHandle,
        SYNCHRONIZE | FILE_READ_ATTRIBUTES,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_ALERT,
    );
    if !NT_SUCCESS(Status) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        BaseSetLastNTError(Status);
        return 0;
    }

    /* Query the device name - that's what we'll translate */
    if DeviceIoControl(
        VolumeHandle,
        IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
        core::ptr::null_mut(),
        0,
        &mut MountDevName as *mut MountDevName as *mut core::ffi::c_void,
        core::mem::size_of::<MountDevName>() as DWORD,
        &mut BytesReturned,
        core::ptr::null_mut(),
    ) == 0
    {
        NtClose(VolumeHandle);
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        return 0;
    }

    /* No longer need the volume */
    NtClose(VolumeHandle);
    RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);

    /* We'll keep the device name for later usage */
    NtPathName.Length = MountDevName.NameLength;
    NtPathName.MaximumLength = MountDevName.NameLength + core::mem::size_of::<WCHAR>() as u16;
    NtPathName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, NtPathName.MaximumLength as SIZE_T) as PWSTR;
    if NtPathName.Buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    RtlCopyMemory(
        NtPathName.Buffer as *mut core::ffi::c_void,
        &MountDevName.Name as *const _ as *const core::ffi::c_void,
        NtPathName.Length as SIZE_T,
    );
    *NtPathName.Buffer.offset((NtPathName.Length / core::mem::size_of::<WCHAR>() as u16) as isize) = 0u16;

    /* Allocate the structure for querying the mount mgr */
    MountPoint = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (NtPathName.Length as usize) as SIZE_T + core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as SIZE_T,
    ) as PMOUNTMGR_MOUNT_POINT;
    if MountPoint.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* 0 everything, we provide a device name */
    RtlZeroMemory(MountPoint as *mut core::ffi::c_void, core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as SIZE_T);
    (*MountPoint).DeviceNameOffset = core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as u32;
    (*MountPoint).DeviceNameLength = NtPathName.Length;
    RtlCopyMemory(
        (MountPoint as usize + core::mem::size_of::<MOUNTMGR_MOUNT_POINT>()) as *mut core::ffi::c_void,
        NtPathName.Buffer as *const core::ffi::c_void,
        NtPathName.Length as SIZE_T,
    );

    /* Allocate a dummy output buffer to probe for size */
    MountPoints = RtlAllocateHeap(RtlGetProcessHeap(), 0, core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as SIZE_T) as PMOUNTMGR_MOUNT_POINTS;
    if MountPoints.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint as *mut core::ffi::c_void);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* Open a handle to the mount manager */
    MountMgrHandle = CreateFileW(
        MOUNTMGR_DOS_DEVICE_NAME,
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        INVALID_HANDLE_VALUE,
    );
    if MountMgrHandle == INVALID_HANDLE_VALUE {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints as *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint as *mut core::ffi::c_void);
        return 0;
    }

    /* Query the names associated to our device name */
    Ret = DeviceIoControl(
        MountMgrHandle,
        IOCTL_MOUNTMGR_QUERY_POINTS,
        MountPoint as *mut core::ffi::c_void,
        (NtPathName.Length as u32) + core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as u32,
        MountPoints as *mut core::ffi::c_void,
        core::mem::size_of::<MOUNTMGR_MOUNT_POINTS>() as DWORD,
        &mut BytesReturned,
        core::ptr::null_mut(),
    );
    /* As long as the buffer is too small, keep looping */
    while Ret == 0 && GetLastError() == ERROR_MORE_DATA {
        let mut BufferSize: ULONG;

        /* Get the size we've to allocate */
        BufferSize = (*MountPoints).Size;
        /* Reallocate the buffer with enough room */
        RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints as *mut core::ffi::c_void);
        MountPoints = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize as SIZE_T) as PMOUNTMGR_MOUNT_POINTS;
        if MountPoints.is_null() {
            CloseHandle(MountMgrHandle);
            RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
            RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint as *mut core::ffi::c_void);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }

        /* Reissue the request, it should work now! */
        Ret = DeviceIoControl(
            MountMgrHandle,
            IOCTL_MOUNTMGR_QUERY_POINTS,
            MountPoint as *mut core::ffi::c_void,
            (NtPathName.Length as u32) + core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as u32,
            MountPoints as *mut core::ffi::c_void,
            BufferSize,
            &mut BytesReturned,
            core::ptr::null_mut(),
        );
    }

    /* We're done, no longer need the mount manager */
    CloseHandle(MountMgrHandle);
    /* Nor our input buffer */
    RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint as *mut core::ffi::c_void);

    /* If the mount manager failed, just quit */
    if Ret == 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints as *mut core::ffi::c_void);
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    CurrentMntPt = 0;
    /* If there were no associated mount points, we'll return the device name */
    if (*MountPoints).NumberOfMountPoints == 0 {
        FoundVolume = NtPathName.Buffer;
        FoundVolumeLen = NtPathName.Length as ULONG;
    }
    /* Otherwise, find one which is matching */
    else {
        while CurrentMntPt < (*MountPoints).NumberOfMountPoints {
            let mut SymbolicLink: _UNICODE_STRING = core::mem::zeroed();

            /* Make a string of it, to easy the checks */
            // MOUNTMGR_MOUNT_POINTS.MountPoints is `MountPoints[1]` bounded by
            // NumberOfMountPoints -- a flexible array. bindgen's `[T; 1]` panicked past
            // the first mount point, so FindFirstVolume/FindNextVolume faulted on any
            // machine with more than one volume.
            let mp = (&raw const (*MountPoints).MountPoints[0]).add(CurrentMntPt as usize);
            SymbolicLink.Length = (*mp).SymbolicLinkNameLength;
            SymbolicLink.MaximumLength = SymbolicLink.Length;
            SymbolicLink.Buffer = (MountPoints as usize + (*mp).SymbolicLinkNameOffset as usize) as PWSTR;
            /* If that's a NT volume name (GUID form), keep it! */
            if MOUNTMGR_IS_NT_VOLUME_NAME(&mut SymbolicLink) != 0 {
                FoundVolume = SymbolicLink.Buffer;
                FoundVolumeLen = SymbolicLink.Length as ULONG;

                break;
            }

            CurrentMntPt = CurrentMntPt.wrapping_add(1);
        }
    }

    /* We couldn't find anything matching, return an error */
    if CurrentMntPt == (*MountPoints).NumberOfMountPoints {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints as *mut core::ffi::c_void);
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* We found a matching volume, have we enough memory to return it? */
    if (cchBufferLength as usize) * core::mem::size_of::<WCHAR>() < (FoundVolumeLen as usize) + 2 * core::mem::size_of::<WCHAR>() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints as *mut core::ffi::c_void);
        SetLastError(ERROR_FILENAME_EXCED_RANGE);
        return 0;
    }

    /* Copy it back! */
    RtlCopyMemory(
        lpszVolumeName as *mut core::ffi::c_void,
        FoundVolume as *const core::ffi::c_void,
        FoundVolumeLen as SIZE_T,
    );
    /* Make it compliant */
    *lpszVolumeName.offset(1) = '\\' as u16;
    /* And transform it as root path */
    *lpszVolumeName.offset((FoundVolumeLen as usize / core::mem::size_of::<WCHAR>()) as isize) = '\\' as u16;
    *lpszVolumeName.offset((FoundVolumeLen as usize / core::mem::size_of::<WCHAR>()).wrapping_add(1) as isize) = 0u16;

    /* We're done! */
    RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer as *mut core::ffi::c_void);
    RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints as *mut core::ffi::c_void);
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn BasepGetVolumeNameFromReparsePoint(
    lpszMountPoint: LPCWSTR,
    lpszVolumeName: LPWSTR,
    cchBufferLength: DWORD,
    IsAMountPoint: LPBOOL,
) -> BOOL {
    let mut Old: WCHAR;
    let mut BytesReturned: DWORD = 0;
    let mut ReparseHandle: HANDLE;
    let mut SubstituteName: _UNICODE_STRING = core::mem::zeroed();
    let mut ReparseBuffer: PREPARSE_DATA_BUFFER;

    /* Try to open the reparse point */
    ReparseHandle = CreateFileW(
        lpszMountPoint,
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS | FILE_ATTRIBUTE_NORMAL,
        INVALID_HANDLE_VALUE,
    );
    /* It failed! */
    if ReparseHandle == INVALID_HANDLE_VALUE {
        /* Report it's not a mount point (it's not a reparse point) */
        if !IsAMountPoint.is_null() {
            *IsAMountPoint = 0;
        }

        /* And zero output */
        if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
            *lpszVolumeName.offset(0) = 0u16;
        }

        return 0;
    }

    /* This is a mount point! */
    if !IsAMountPoint.is_null() {
        *IsAMountPoint = 1;
    }

    /* Prepare a buffer big enough to read its data */
    ReparseBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        MAXIMUM_REPARSE_DATA_BUFFER_SIZE as SIZE_T,
    ) as PREPARSE_DATA_BUFFER;
    if ReparseBuffer.is_null() {
        CloseHandle(ReparseHandle);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);

        /* Zero output */
        if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
            *lpszVolumeName.offset(0) = 0u16;
        }

        return 0;
    }

    /* Dump the reparse point data */
    if DeviceIoControl(
        ReparseHandle,
        FSCTL_GET_REPARSE_POINT,
        core::ptr::null_mut(),
        0,
        ReparseBuffer as LPVOID,
        MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
        &mut BytesReturned,
        core::ptr::null_mut(),
    ) == 0
    {
        RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer as PVOID);
        CloseHandle(ReparseHandle);

        /* Zero output */
        if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
            *lpszVolumeName.offset(0) = 0u16;
        }

        return 0;
    }

    /* We no longer need the reparse point */
    CloseHandle(ReparseHandle);

    /* We only handle mount points */
    if (*ReparseBuffer).ReparseTag != IO_REPARSE_TAG_MOUNT_POINT {
        RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer as PVOID);

        /* Zero output */
        if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
            *lpszVolumeName.offset(0) = 0u16;
        }

        return 0;
    }

    /* Do we have enough room for copying substitue name? */
    if ((*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength
        + core::mem::size_of::<WCHAR>() as u16)
        > cchBufferLength as u16 * core::mem::size_of::<WCHAR>() as u16
    {
        RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer as PVOID);
        SetLastError(ERROR_FILENAME_EXCED_RANGE);

        /* Zero output */
        if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
            *lpszVolumeName.offset(0) = 0u16;
        }

        return 0;
    }

    /* Copy the link target */
    RtlCopyMemory(
        lpszVolumeName as PVOID,
        // PathBuffer is a C flexible array (`WCHAR PathBuffer[1]`, ndk/iotypes.h:1260)
        // that bindgen renders as [WCHAR; 1], and the buffer above is allocated
        // MAXIMUM_REPARSE_DATA_BUFFER_SIZE. Indexing it with the runtime
        // SubstituteNameOffset bounds-checks against 1 and panics for any offset
        // past the first character. Constant index 0 is checked at compile time,
        // so `&raw const ...[0]` yields the element pointer with no panic branch.
        (&raw const (*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.PathBuffer[0])
            .add(
                (*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameOffset
                    as usize
                    / core::mem::size_of::<WCHAR>(),
            ) as PVOID,
        (*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength as SIZE_T,
    );
    /* Make it DOS valid */
    Old = *lpszVolumeName.offset(1);
    /* We want a root path */
    *lpszVolumeName.offset(1) = '\\' as u16;
    /* And null terminate obviously */
    *lpszVolumeName.offset(
        (*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength as isize
            / core::mem::size_of::<WCHAR>() as isize,
    ) = 0u16;

    /* Make it a string to easily check it */
    SubstituteName.Length = (*ReparseBuffer).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength;
    SubstituteName.MaximumLength = SubstituteName.Length;
    SubstituteName.Buffer = lpszVolumeName;

    /* No longer need the data? */
    RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer as PVOID);

    /* Is that a dos volume name with backslash? */
    if MOUNTMGR_IS_DOS_VOLUME_NAME_WB(&mut SubstituteName) != 0 {
        return 1;
    }

    /* No, so restore previous name and return to the caller */
    *lpszVolumeName.offset(1) = Old;
    SetLastError(ERROR_INVALID_PARAMETER);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn BasepGetVolumeNameForVolumeMountPoint(
    lpszMountPoint: LPCWSTR,
    lpszVolumeName: LPWSTR,
    cchBufferLength: DWORD,
    IsAMountPoint: LPBOOL,
) -> BOOL {
    let mut Ret: BOOL;
    let mut MountPoint: _UNICODE_STRING = core::mem::zeroed();

    /* Assume it's a mount point (likely for non reparse points) */
    if !IsAMountPoint.is_null() {
        *IsAMountPoint = 1;
    }

    /* Make a string with the mount point name */
    RtlInitUnicodeString(&mut MountPoint, lpszMountPoint);
    /* Not a root path? */
    if *MountPoint.Buffer.offset((MountPoint.Length / core::mem::size_of::<WCHAR>() as u16) as isize - 1) != '\\' as u16
    {
        BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
        /* Zero output */
        if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
            *lpszVolumeName.offset(0) = 0u16;
        }

        return 0;
    }

    /* Does it look like <letter>:\? */
    if MountPoint.Length == 3 * core::mem::size_of::<WCHAR>() as u16 {
        /* Try to get volume name for root path */
        Ret = GetVolumeNameForRoot(lpszMountPoint, lpszVolumeName, cchBufferLength);
        /* It failed? */
        if Ret == 0 {
            /* If wasn't a drive letter, so maybe a reparse point? */
            if *MountPoint.Buffer.offset(1) != ':' as u16 {
                Ret = BasepGetVolumeNameFromReparsePoint(
                    lpszMountPoint,
                    lpszVolumeName,
                    cchBufferLength,
                    IsAMountPoint,
                );
            }
            /* It was, so zero output */
            else if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
                *lpszVolumeName.offset(0) = 0u16;
            }
        }
    } else {
        /* Try to get volume name for root path */
        Ret = GetVolumeNameForRoot(lpszMountPoint, lpszVolumeName, cchBufferLength);
        /* It failed? */
        if Ret == 0 {
            /* It was a DOS volume as UNC name, so fail and zero output */
            if MountPoint.Length == 14
                && *MountPoint.Buffer.offset(0) == '\\' as u16
                && *MountPoint.Buffer.offset(1) == '\\' as u16
                && (*MountPoint.Buffer.offset(2) == '.' as u16 || *MountPoint.Buffer.offset(2) == '?' as u16)
                && *MountPoint.Buffer.offset(3) == '\\' as u16
                && *MountPoint.Buffer.offset(5) == ':' as u16
            {
                if !lpszVolumeName.is_null() && cchBufferLength >= 1 {
                    *lpszVolumeName.offset(0) = 0u16;
                }
            }
            /* Maybe it's a reparse point? */
            else {
                Ret = BasepGetVolumeNameFromReparsePoint(
                    lpszMountPoint,
                    lpszVolumeName,
                    cchBufferLength,
                    IsAMountPoint,
                );
            }
        }
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetVolumeNameForVolumeMountPointW(
    VolumeMountPoint: LPCWSTR,
    VolumeName: LPWSTR,
    VolumeNameLength: DWORD,
) -> BOOL {
    let Ret: BOOL;

    /* Just query our internal function */
    Ret = BasepGetVolumeNameForVolumeMountPoint(VolumeMountPoint, VolumeName, VolumeNameLength, core::ptr::null_mut());
    if Ret == 0 && !VolumeName.is_null() && VolumeNameLength >= 1 {
        *VolumeName.offset(0) = 0u16;
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetVolumeNameForVolumeMountPointA(
    lpszVolumeMountPoint: LPCSTR,
    lpszVolumeName: LPSTR,
    cchBufferLength: DWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let mut VolumeName: ANSI_STRING = core::mem::zeroed();
    let mut VolumeNameU: _UNICODE_STRING = core::mem::zeroed();
    let VolumeMountPointU: PUNICODE_STRING;

    /* Convert mount point to unicode */
    VolumeMountPointU = Basep8BitStringToStaticUnicodeString(lpszVolumeMountPoint);
    if VolumeMountPointU.is_null() {
        return 0;
    }

    /* Initialize the strings we'll use for convention */
    VolumeName.Buffer = lpszVolumeName;
    VolumeName.Length = 0;
    VolumeName.MaximumLength = (cchBufferLength - 1) as u16;

    VolumeNameU.Length = 0;
    VolumeNameU.MaximumLength = ((cchBufferLength - 1) as usize * core::mem::size_of::<WCHAR>() + core::mem::size_of::<WCHAR>()) as u16;
    /* Allocate a buffer big enough to contain the returned name */
    VolumeNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumeNameU.MaximumLength as SIZE_T) as PWSTR;
    if VolumeNameU.Buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    /* Query -W */
    Ret = GetVolumeNameForVolumeMountPointW(
        (*VolumeMountPointU).Buffer,
        VolumeNameU.Buffer,
        cchBufferLength,
    );
    /* If it succeed, perform -A conversion */
    if Ret != 0 {
        let Status: NTSTATUS;

        /* Reinit our string for length */
        RtlInitUnicodeString(&mut VolumeNameU, VolumeNameU.Buffer);
        /* Convert to ANSI */
        Status = RtlUnicodeStringToAnsiString(&mut VolumeName, &VolumeNameU, 0);
        /* If conversion failed, force failure, otherwise, just null terminate */
        if !NT_SUCCESS(Status) {
            Ret = 0;
            BaseSetLastNTError(Status);
        } else {
            *VolumeName.Buffer.offset(VolumeName.Length as isize) = 0i8;
        }
    }

    /* Internal buffer no longer needed */
    RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameU.Buffer as *mut core::ffi::c_void);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetVolumeMountPointW(
    lpszVolumeMountPoint: LPCWSTR,
    lpszVolumeName: LPCWSTR,
) -> BOOL {
    STUB!();
    0
}

#[no_mangle]
pub unsafe extern "system" fn SetVolumeMountPointA(
    lpszVolumeMountPoint: LPCSTR,
    lpszVolumeName: LPCSTR,
) -> BOOL {
    STUB!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteVolumeMountPointA(
    lpszVolumeMountPoint: LPCSTR,
) -> BOOL {
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteVolumeMountPointW(
    lpszVolumeMountPoint: LPCWSTR,
) -> BOOL {
    STUB!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstVolumeMountPointW(
    lpszRootPathName: LPCWSTR,
    lpszVolumeMountPoint: LPWSTR,
    cchBufferLength: DWORD,
) -> HANDLE {
    STUB!();
    return 0 as HANDLE;
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstVolumeMountPointA(
    lpszRootPathName: LPCSTR,
    lpszVolumeMountPoint: LPSTR,
    cchBufferLength: DWORD,
) -> HANDLE {
    STUB!();
    return 0 as HANDLE;
}

#[no_mangle]
pub unsafe extern "system" fn FindNextVolumeMountPointA(
    hFindVolumeMountPoint: HANDLE,
    lpszVolumeMountPoint: LPSTR,
    cchBufferLength: DWORD,
) -> BOOL {
    STUB!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FindNextVolumeMountPointW(
    hFindVolumeMountPoint: HANDLE,
    lpszVolumeMountPoint: LPWSTR,
    cchBufferLength: DWORD,
) -> BOOL {
    STUB!();
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FindVolumeMountPointClose(
    hFindVolumeMountPoint: HANDLE,
) -> BOOL {
    STUB!();
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/mntpoint.c
//  * PURPOSE:         File volume mount point functions
//  * PROGRAMMER:      Pierre Schweitzer (pierre@reactos.org)
//  */
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// static BOOL
// GetVolumeNameForRoot(IN LPCWSTR lpszRootPath,
//                      OUT LPWSTR lpszVolumeName,
//                      IN DWORD cchBufferLength)
// {
//     BOOL Ret;
//     NTSTATUS Status;
//     PWSTR FoundVolume;
//     DWORD BytesReturned;
//     UNICODE_STRING NtPathName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PMOUNTMGR_MOUNT_POINT MountPoint;
//     ULONG CurrentMntPt, FoundVolumeLen;
//     PMOUNTMGR_MOUNT_POINTS MountPoints;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE VolumeHandle, MountMgrHandle;
//     struct
//     {
//         MOUNTDEV_NAME;
//         WCHAR Buffer[MAX_PATH];
//     } MountDevName;
//
//     /* It makes no sense on a non-local drive */
//     if (GetDriveTypeW(lpszRootPath) == DRIVE_REMOTE)
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* Get the NT path */
//     if (!RtlDosPathNameToNtPathName_U(lpszRootPath, &NtPathName, NULL, NULL))
//     {
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* If it's a root path - likely - drop backslash to open volume */
//     if (NtPathName.Buffer[(NtPathName.Length / sizeof(WCHAR)) - 1] == L'\\')
//     {
//         NtPathName.Buffer[(NtPathName.Length / sizeof(WCHAR)) - 1] = UNICODE_NULL;
//         NtPathName.Length -= sizeof(WCHAR);
//     }
//
//     /* If that's a DOS volume, upper case the letter */
//     if (NtPathName.Length >= 2 * sizeof(WCHAR))
//     {
//         if (NtPathName.Buffer[(NtPathName.Length / sizeof(WCHAR)) - 1] == L':')
//         {
//             NtPathName.Buffer[(NtPathName.Length / sizeof(WCHAR)) - 2] = towupper(NtPathName.Buffer[(NtPathName.Length / sizeof(WCHAR)) - 2]);
//         }
//     }
//
//     /* Attempt to open the volume */
//     InitializeObjectAttributes(&ObjectAttributes, &NtPathName,
//                                OBJ_CASE_INSENSITIVE, NULL, NULL);
//     Status = NtOpenFile(&VolumeHandle, SYNCHRONIZE | FILE_READ_ATTRIBUTES,
//                         &ObjectAttributes, &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_ALERT);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Query the device name - that's what we'll translate */
//     if (!DeviceIoControl(VolumeHandle, IOCTL_MOUNTDEV_QUERY_DEVICE_NAME, NULL,
//                          0, &MountDevName, sizeof(MountDevName), &BytesReturned,
//                          NULL))
//     {
//         NtClose(VolumeHandle);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         return FALSE;
//     }
//
//     /* No longer need the volume */
//     NtClose(VolumeHandle);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//
//     /* We'll keep the device name for later usage */
//     NtPathName.Length = MountDevName.NameLength;
//     NtPathName.MaximumLength = MountDevName.NameLength + sizeof(UNICODE_NULL);
//     NtPathName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, NtPathName.MaximumLength);
//     if (NtPathName.Buffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     RtlCopyMemory(NtPathName.Buffer, MountDevName.Name, NtPathName.Length);
//     NtPathName.Buffer[NtPathName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Allocate the structure for querying the mount mgr */
//     MountPoint = RtlAllocateHeap(RtlGetProcessHeap(), 0,
//                                  NtPathName.Length + sizeof(MOUNTMGR_MOUNT_POINT));
//     if (MountPoint == NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* 0 everything, we provide a device name */
//     RtlZeroMemory(MountPoint, sizeof(MOUNTMGR_MOUNT_POINT));
//     MountPoint->DeviceNameOffset = sizeof(MOUNTMGR_MOUNT_POINT);
//     MountPoint->DeviceNameLength = NtPathName.Length;
//     RtlCopyMemory((PVOID)((ULONG_PTR)MountPoint + sizeof(MOUNTMGR_MOUNT_POINT)), NtPathName.Buffer, NtPathName.Length);
//
//     /* Allocate a dummy output buffer to probe for size */
//     MountPoints = RtlAllocateHeap(RtlGetProcessHeap(), 0, sizeof(MOUNTMGR_MOUNT_POINTS));
//     if (MountPoints == NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Open a handle to the mount manager */
//     MountMgrHandle = CreateFileW(MOUNTMGR_DOS_DEVICE_NAME, 0,
//                                  FILE_SHARE_READ | FILE_SHARE_WRITE, NULL,
//                                  OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
//                                  INVALID_HANDLE_VALUE);
//     if (MountMgrHandle == INVALID_HANDLE_VALUE)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint);
//         return FALSE;
//     }
//
//     /* Query the names associated to our device name */
//     Ret = DeviceIoControl(MountMgrHandle, IOCTL_MOUNTMGR_QUERY_POINTS,
//                           MountPoint, NtPathName.Length + sizeof(MOUNTMGR_MOUNT_POINT),
//                           MountPoints, sizeof(MOUNTMGR_MOUNT_POINTS), &BytesReturned,
//                           NULL);
//     /* As long as the buffer is too small, keep looping */
//     while (!Ret && GetLastError() == ERROR_MORE_DATA)
//     {
//         ULONG BufferSize;
//
//         /* Get the size we've to allocate */
//         BufferSize = MountPoints->Size;
//         /* Reallocate the buffer with enough room */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints);
//         MountPoints = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize);
//         if (MountPoints == NULL)
//         {
//             CloseHandle(MountMgrHandle);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint);
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//
//         /* Reissue the request, it should work now! */
//         Ret = DeviceIoControl(MountMgrHandle, IOCTL_MOUNTMGR_QUERY_POINTS,
//                               MountPoint, NtPathName.Length + sizeof(MOUNTMGR_MOUNT_POINT),
//                               MountPoints, BufferSize, &BytesReturned, NULL);
//     }
//
//     /* We're done, no longer need the mount manager */
//     CloseHandle(MountMgrHandle);
//     /* Nor our input buffer */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoint);
//
//     /* If the mount manager failed, just quit */
//     if (!Ret)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints);
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     CurrentMntPt = 0;
//     /* If there were no associated mount points, we'll return the device name */
//     if (MountPoints->NumberOfMountPoints == 0)
//     {
//         FoundVolume = NtPathName.Buffer;
//         FoundVolumeLen = NtPathName.Length;
//     }
//     /* Otherwise, find one which is matching */
//     else
//     {
//         for (; CurrentMntPt < MountPoints->NumberOfMountPoints; ++CurrentMntPt)
//         {
//             UNICODE_STRING SymbolicLink;
//
//             /* Make a string of it, to easy the checks */
//             SymbolicLink.Length = MountPoints->MountPoints[CurrentMntPt].SymbolicLinkNameLength;
//             SymbolicLink.MaximumLength = SymbolicLink.Length;
//             SymbolicLink.Buffer = (PVOID)((ULONG_PTR)MountPoints + MountPoints->MountPoints[CurrentMntPt].SymbolicLinkNameOffset);
//             /* If that's a NT volume name (GUID form), keep it! */
//             if (MOUNTMGR_IS_NT_VOLUME_NAME(&SymbolicLink))
//             {
//                 FoundVolume = SymbolicLink.Buffer;
//                 FoundVolumeLen = SymbolicLink.Length;
//
//                 break;
//             }
//         }
//     }
//
//     /* We couldn't find anything matching, return an error */
//     if (CurrentMntPt == MountPoints->NumberOfMountPoints)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints);
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* We found a matching volume, have we enough memory to return it? */
//     if (cchBufferLength * sizeof(WCHAR) < FoundVolumeLen + 2 * sizeof(WCHAR))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints);
//         SetLastError(ERROR_FILENAME_EXCED_RANGE);
//         return FALSE;
//     }
//
//     /* Copy it back! */
//     RtlCopyMemory(lpszVolumeName, FoundVolume, FoundVolumeLen);
//     /* Make it compliant */
//     lpszVolumeName[1] = L'\\';
//     /* And transform it as root path */
//     lpszVolumeName[FoundVolumeLen / sizeof(WCHAR)] = L'\\';
//     lpszVolumeName[FoundVolumeLen / sizeof(WCHAR) + 1] = UNICODE_NULL;
//
//     /* We're done! */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NtPathName.Buffer);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, MountPoints);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// BasepGetVolumeNameFromReparsePoint(IN LPCWSTR lpszMountPoint,
//                                    OUT LPWSTR lpszVolumeName,
//                                    IN DWORD cchBufferLength,
//                                    OUT LPBOOL IsAMountPoint)
// {
//     WCHAR Old;
//     DWORD BytesReturned;
//     HANDLE ReparseHandle;
//     UNICODE_STRING SubstituteName;
//     PREPARSE_DATA_BUFFER ReparseBuffer;
//
//     /* Try to open the reparse point */
//     ReparseHandle = CreateFileW(lpszMountPoint, 0,
//                                 FILE_SHARE_READ | FILE_SHARE_WRITE, NULL,
//                                 OPEN_EXISTING,
//                                 FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS | FILE_ATTRIBUTE_NORMAL,
//                                 INVALID_HANDLE_VALUE);
//     /* It failed! */
//     if (ReparseHandle == INVALID_HANDLE_VALUE)
//     {
//         /* Report it's not a mount point (it's not a reparse point) */
//         if (IsAMountPoint != NULL)
//         {
//             *IsAMountPoint = FALSE;
//         }
//
//         /* And zero output */
//         if (lpszVolumeName != NULL && cchBufferLength >= 1)
//         {
//             lpszVolumeName[0] = UNICODE_NULL;
//         }
//
//         return FALSE;
//     }
//
//     /* This is a mount point! */
//     if (IsAMountPoint != NULL)
//     {
//         *IsAMountPoint = TRUE;
//     }
//
//     /* Prepare a buffer big enough to read its data */
//     ReparseBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, MAXIMUM_REPARSE_DATA_BUFFER_SIZE);
//     if (ReparseBuffer == NULL)
//     {
//         CloseHandle(ReparseHandle);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//
//         /* Zero output */
//         if (lpszVolumeName != NULL && cchBufferLength >= 1)
//         {
//             lpszVolumeName[0] = UNICODE_NULL;
//         }
//
//         return FALSE;
//     }
//
//     /* Dump the reparse point data */
//     if (!DeviceIoControl(ReparseHandle, FSCTL_GET_REPARSE_POINT, NULL, 0,
//                          ReparseBuffer, MAXIMUM_REPARSE_DATA_BUFFER_SIZE, &BytesReturned,
//                          NULL))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer);
//         CloseHandle(ReparseHandle);
//
//         /* Zero output */
//         if (lpszVolumeName != NULL && cchBufferLength >= 1)
//         {
//             lpszVolumeName[0] = UNICODE_NULL;
//         }
//
//         return FALSE;
//     }
//
//     /* We no longer need the reparse point */
//     CloseHandle(ReparseHandle);
//
//     /* We only handle mount points */
//     if (ReparseBuffer->ReparseTag != IO_REPARSE_TAG_MOUNT_POINT)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer);
//
//         /* Zero output */
//         if (lpszVolumeName != NULL && cchBufferLength >= 1)
//         {
//             lpszVolumeName[0] = UNICODE_NULL;
//         }
//
//         return FALSE;
//     }
//
//     /* Do we have enough room for copying substitue name? */
//     if ((ReparseBuffer->MountPointReparseBuffer.SubstituteNameLength + sizeof(UNICODE_NULL)) > cchBufferLength * sizeof(WCHAR))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer);
//         SetLastError(ERROR_FILENAME_EXCED_RANGE);
//
//         /* Zero output */
//         if (lpszVolumeName != NULL && cchBufferLength >= 1)
//         {
//             lpszVolumeName[0] = UNICODE_NULL;
//         }
//
//         return FALSE;
//     }
//
//     /* Copy the link target */
//     RtlCopyMemory(lpszVolumeName,
//                   &ReparseBuffer->MountPointReparseBuffer.PathBuffer[ReparseBuffer->MountPointReparseBuffer.SubstituteNameOffset / sizeof(WCHAR)],
//                   ReparseBuffer->MountPointReparseBuffer.SubstituteNameLength);
//     /* Make it DOS valid */
//     Old = lpszVolumeName[1];
//     /* We want a root path */
//     lpszVolumeName[1] = L'\\';
//     /* And null terminate obviously */
//     lpszVolumeName[ReparseBuffer->MountPointReparseBuffer.SubstituteNameLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Make it a string to easily check it */
//     SubstituteName.Length = ReparseBuffer->MountPointReparseBuffer.SubstituteNameLength;
//     SubstituteName.MaximumLength = SubstituteName.Length;
//     SubstituteName.Buffer = lpszVolumeName;
//
//     /* No longer need the data? */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ReparseBuffer);
//
//     /* Is that a dos volume name with backslash? */
//     if (MOUNTMGR_IS_DOS_VOLUME_NAME_WB(&SubstituteName))
//     {
//         return TRUE;
//     }
//
//     /* No, so restore previous name and return to the caller */
//     lpszVolumeName[1] = Old;
//     SetLastError(ERROR_INVALID_PARAMETER);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// BasepGetVolumeNameForVolumeMountPoint(IN LPCWSTR lpszMountPoint,
//                                       OUT LPWSTR lpszVolumeName,
//                                       IN DWORD cchBufferLength,
//                                       OUT LPBOOL IsAMountPoint)
// {
//     BOOL Ret;
//     UNICODE_STRING MountPoint;
//
//     /* Assume it's a mount point (likely for non reparse points) */
//     if (IsAMountPoint != NULL)
//     {
//         *IsAMountPoint = 1;
//     }
//
//     /* Make a string with the mount point name */
//     RtlInitUnicodeString(&MountPoint, lpszMountPoint);
//     /* Not a root path? */
//     if (MountPoint.Buffer[(MountPoint.Length / sizeof(WCHAR)) - 1] != L'\\')
//     {
//         BaseSetLastNTError(STATUS_OBJECT_NAME_INVALID);
//         /* Zero output */
//         if (lpszVolumeName != NULL && cchBufferLength >= 1)
//         {
//             lpszVolumeName[0] = UNICODE_NULL;
//         }
//
//         return FALSE;
//     }
//
//     /* Does it look like <letter>:\? */
//     if (MountPoint.Length == 3 * sizeof(WCHAR))
//     {
//         /* Try to get volume name for root path */
//         Ret = GetVolumeNameForRoot(lpszMountPoint, lpszVolumeName, cchBufferLength);
//         /* It failed? */
//         if (!Ret)
//         {
//             /* If wasn't a drive letter, so maybe a reparse point? */
//             if (MountPoint.Buffer[1] != ':')
//             {
//                 Ret = BasepGetVolumeNameFromReparsePoint(lpszMountPoint, lpszVolumeName, cchBufferLength, IsAMountPoint);
//             }
//             /* It was, so zero output */
//             else if (lpszVolumeName != NULL && cchBufferLength >= 1)
//             {
//                 lpszVolumeName[0] = UNICODE_NULL;
//             }
//         }
//     }
//     else
//     {
//         /* Try to get volume name for root path */
//         Ret = GetVolumeNameForRoot(lpszMountPoint, lpszVolumeName, cchBufferLength);
//         /* It failed? */
//         if (!Ret)
//         {
//             /* It was a DOS volume as UNC name, so fail and zero output */
//             if (MountPoint.Length == 14 && MountPoint.Buffer[0] == '\\' && MountPoint.Buffer[1] == '\\' &&
//                 (MountPoint.Buffer[2] == '.' || MountPoint.Buffer[2] == '?') && MountPoint.Buffer[3] == L'\\' &&
//                 MountPoint.Buffer[5] == ':')
//             {
//                 if (lpszVolumeName != NULL && cchBufferLength >= 1)
//                 {
//                     lpszVolumeName[0] = UNICODE_NULL;
//                 }
//             }
//             /* Maybe it's a reparse point? */
//             else
//             {
//                 Ret = BasepGetVolumeNameFromReparsePoint(lpszMountPoint, lpszVolumeName, cchBufferLength, IsAMountPoint);
//             }
//         }
//     }
//
//     return Ret;
// }
//
// /**
//  * @name GetVolumeNameForVolumeMountPointW
//  * @implemented
//  *
//  * Return an unique volume name for a drive root or mount point.
//  *
//  * @param VolumeMountPoint
//  *        Pointer to string that contains either root drive name or
//  *        mount point name.
//  * @param VolumeName
//  *        Pointer to buffer that is filled with resulting unique
//  *        volume name on success.
//  * @param VolumeNameLength
//  *        Size of VolumeName buffer in TCHARs.
//  *
//  * @return
//  *     TRUE when the function succeeds and the VolumeName buffer is filled,
//  *     FALSE otherwise.
//  */
// BOOL
// WINAPI
// GetVolumeNameForVolumeMountPointW(IN LPCWSTR VolumeMountPoint,
//                                   OUT LPWSTR VolumeName,
//                                   IN DWORD VolumeNameLength)
// {
//     BOOL Ret;
//
//     /* Just query our internal function */
//     Ret = BasepGetVolumeNameForVolumeMountPoint(VolumeMountPoint, VolumeName,
//                                                 VolumeNameLength, NULL);
//     if (!Ret && VolumeName != NULL && VolumeNameLength >= 1)
//     {
//         VolumeName[0] = UNICODE_NULL;
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
// GetVolumeNameForVolumeMountPointA(IN LPCSTR lpszVolumeMountPoint,
//                                   IN LPSTR lpszVolumeName,
//                                   IN DWORD cchBufferLength)
// {
//     BOOL Ret;
//     ANSI_STRING VolumeName;
//     UNICODE_STRING VolumeNameU;
//     PUNICODE_STRING VolumeMountPointU;
//
//     /* Convert mount point to unicode */
//     VolumeMountPointU = Basep8BitStringToStaticUnicodeString(lpszVolumeMountPoint);
//     if (VolumeMountPointU == NULL)
//     {
//         return FALSE;
//     }
//
//     /* Initialize the strings we'll use for convention */
//     VolumeName.Buffer = lpszVolumeName;
//     VolumeName.Length = 0;
//     VolumeName.MaximumLength = cchBufferLength - 1;
//
//     VolumeNameU.Length = 0;
//     VolumeNameU.MaximumLength = (cchBufferLength - 1) * sizeof(WCHAR) + sizeof(UNICODE_NULL);
//     /* Allocate a buffer big enough to contain the returned name */
//     VolumeNameU.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, VolumeNameU.MaximumLength);
//     if (VolumeNameU.Buffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Query -W */
//     Ret = GetVolumeNameForVolumeMountPointW(VolumeMountPointU->Buffer, VolumeNameU.Buffer, cchBufferLength);
//     /* If it succeed, perform -A conversion */
//     if (Ret)
//     {
//         NTSTATUS Status;
//
//         /* Reinit our string for length */
//         RtlInitUnicodeString(&VolumeNameU, VolumeNameU.Buffer);
//         /* Convert to ANSI */
//         Status = RtlUnicodeStringToAnsiString(&VolumeName, &VolumeNameU, FALSE);
//         /* If conversion failed, force failure, otherwise, just null terminate */
//         if (!NT_SUCCESS(Status))
//         {
//             Ret = FALSE;
//             BaseSetLastNTError(Status);
//         }
//         else
//         {
//             VolumeName.Buffer[VolumeName.Length] = ANSI_NULL;
//         }
//     }
//
//     /* Internal buffer no longer needed */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, VolumeNameU.Buffer);
//
//     return Ret;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SetVolumeMountPointW(IN LPCWSTR lpszVolumeMountPoint,
//                      IN LPCWSTR lpszVolumeName)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SetVolumeMountPointA(IN LPCSTR lpszVolumeMountPoint,
//                      IN LPCSTR lpszVolumeName)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// DeleteVolumeMountPointA(IN LPCSTR lpszVolumeMountPoint)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// DeleteVolumeMountPointW(IN LPCWSTR lpszVolumeMountPoint)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// HANDLE
// WINAPI
// FindFirstVolumeMountPointW(IN LPCWSTR lpszRootPathName,
//                            IN LPWSTR lpszVolumeMountPoint,
//                            IN DWORD cchBufferLength)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// HANDLE
// WINAPI
// FindFirstVolumeMountPointA(IN LPCSTR lpszRootPathName,
//                            IN LPSTR lpszVolumeMountPoint,
//                            IN DWORD cchBufferLength)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// FindNextVolumeMountPointA(IN HANDLE hFindVolumeMountPoint,
//                           IN LPSTR lpszVolumeMountPoint,
//                           DWORD cchBufferLength)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// FindNextVolumeMountPointW(IN HANDLE hFindVolumeMountPoint,
//                           IN LPWSTR lpszVolumeMountPoint,
//                           DWORD cchBufferLength)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// FindVolumeMountPointClose(IN HANDLE hFindVolumeMountPoint)
// {
//     STUB;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

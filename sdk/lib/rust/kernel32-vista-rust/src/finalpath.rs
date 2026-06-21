//! GetFinalPathNameByHandle.c — GetFinalPathNameByHandleW + the MountMgr helper.

use crate::common::*;
use core::ffi::c_void;

const MAX_PATH: usize = 260;

// C: struct { union { OBJECT_NAME_INFORMATION Object; FILE_NAME_INFORMATION File; }; WCHAR String[MAX_PATH]; }
// Both heads overlap at offset 0 and are 8 bytes; use OBJECT_NAME_INFORMATION as the
// representative head and cast the base pointer to PFILE_NAME_INFORMATION as needed.
#[repr(C)]
struct NameInfoBuf {
    Object: OBJECT_NAME_INFORMATION,
    String: [WCHAR; MAX_PATH],
}

// FILE_NAME_INFORMATION head (overlaps Object at offset 0). Local def to avoid a
// glob-import resolution quirk with the bindgen PFILE_NAME_INFORMATION alias.
#[repr(C)]
struct FileNameInfoHdr {
    FileNameLength: ULONG,
    FileName: [WCHAR; 1],
}

// rtlfuncs.h RtlInitEmptyUnicodeString — FORCEINLINE (no exported symbol; the bindgen
// extern decl has no backing). Local def (shadows the glob import) so the call inlines
// instead of referencing a nonexistent `_RtlInitEmptyUnicodeString`.
#[inline]
unsafe fn RtlInitEmptyUnicodeString(
    UnicodeString: PUNICODE_STRING,
    Buffer: PWSTR,
    BufferSize: USHORT,
) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

unsafe fn QueryDosVolumeNameForNtDeviceName(
    DeviceName: PCUNICODE_STRING,
    DosVolumeName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut MupDevice: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut MupDevice, w!("\\Device\\Mup").as_ptr());
    let mut LanManDevice: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut LanManDevice, w!("\\Device\\LanmanRedirector").as_ptr());
    let mut MountMgrDevice: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut MountMgrDevice, w!("\\Device\\MountPointManager").as_ptr());

    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.ObjectName = &mut MountMgrDevice;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;

    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut MountMgrHandle: HANDLE = core::ptr::null_mut();
    let mut Buffer: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let TargetName = Buffer.as_mut_ptr() as *mut MOUNTMGR_TARGET_NAME;
    let VolumePaths = Buffer.as_mut_ptr() as *mut MOUNTMGR_VOLUME_PATHS;

    /* Validate parameters */
    if DeviceName.is_null()
        || (*DeviceName).Buffer.is_null()
        || (*DeviceName).Length == 0
        || DosVolumeName.is_null()
        || (*DosVolumeName).Buffer.is_null()
        || (*DosVolumeName).MaximumLength == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* Reset the output name */
    (*DosVolumeName).Length = 0;

    /* Check for network shares */
    if RtlEqualUnicodeString(DeviceName, &MupDevice, TRUE as BOOLEAN) != 0
        || RtlEqualUnicodeString(DeviceName, &LanManDevice, TRUE as BOOLEAN) != 0
    {
        return RtlAppendUnicodeToString(DosVolumeName, w!("\\\\?\\UNC\\").as_ptr());
    }

    /* Open a handle to the mount manager */
    let mut Status = NtCreateFile(
        &mut MountMgrHandle,
        STANDARD_RIGHTS_READ,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN_IF,
        0,
        core::ptr::null_mut(),
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Use the MountMgr to query the volume name */
    (*TargetName).DeviceNameLength = (*DeviceName).Length;
    RtlCopyMemory(
        (*TargetName).DeviceName.as_mut_ptr() as PVOID,
        (*DeviceName).Buffer as *const c_void,
        (*DeviceName).Length as SIZE_T,
    );
    Status = NtDeviceIoControlFile(
        MountMgrHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        IOCTL_MOUNTMGR_QUERY_DOS_VOLUME_PATH,
        Buffer.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&Buffer) as ULONG,
        Buffer.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&Buffer) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        NtClose(MountMgrHandle);
        return Status;
    }

    let info = IoStatusBlock.Information as usize;
    if (info < core::mem::size_of::<MOUNTMGR_VOLUME_PATHS>())
        || (info < ((*VolumePaths).MultiSzLength as usize + core::mem::size_of::<ULONG>()))
    {
        NtClose(MountMgrHandle);
        return STATUS_UNSUCCESSFUL;
    }

    /* Construct the full volume path from the first returned DOS volume.
       (NOTE: the C original has a misplaced-paren bug in this condition; the
       success path appends all three parts, which is faithfully preserved here.) */
    if !NT_SUCCESS(RtlAppendUnicodeToString(DosVolumeName, w!("\\\\?\\").as_ptr()))
        || !NT_SUCCESS(RtlAppendUnicodeToString(DosVolumeName, (*VolumePaths).MultiSz.as_ptr()))
        || !NT_SUCCESS(RtlAppendUnicodeToString(DosVolumeName, w!("\\").as_ptr()))
    {
        NtClose(MountMgrHandle);
        return Status;
    }

    /* This is based on Windows behavior */
    SetLastError(ERROR_SUCCESS);

    NtClose(MountMgrHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn GetFinalPathNameByHandleW(
    hFile: HANDLE,
    lpszFilePath: LPWSTR,
    cchFilePath: DWORD,
    dwFlags: DWORD,
) -> DWORD {
    let mut NameInfoBuffer: NameInfoBuf = core::mem::zeroed();
    let pFileNameInfo = &mut NameInfoBuffer as *mut NameInfoBuf as *mut FileNameInfoHdr;
    let ObjectName: PUNICODE_STRING = &mut NameInfoBuffer.Object.Name;
    let mut VolumeBuffer: [WCHAR; 53] = [0; 53]; // sizeof("\\Device\\Volume{<guid>}")
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceName: UNICODE_STRING = core::mem::zeroed();
    let mut RelativePath: UNICODE_STRING = core::mem::zeroed();
    let mut VolumeName: UNICODE_STRING = core::mem::zeroed();

    /* Extract the requested volume type */
    let VolumeType = dwFlags & (VOLUME_NAME_GUID | VOLUME_NAME_NONE | VOLUME_NAME_NT);

    /* Check if more than one volume type flag is set */
    if VolumeType & (VolumeType.wrapping_sub(1)) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Query the file name (relative path without volume; validates the handle) */
    let mut Status = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut NameInfoBuffer as *mut NameInfoBuf as PVOID,
        core::mem::size_of::<NameInfoBuf>() as ULONG,
        FileNameInformation,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    /* If no volume was requested, return the relative path */
    if VolumeType == VOLUME_NAME_NONE {
        let final_len = (*pFileNameInfo).FileNameLength / core::mem::size_of::<WCHAR>() as ULONG;
        if cchFilePath < final_len + 1 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return final_len + 1;
        }
        RtlCopyMemory(
            lpszFilePath as PVOID,
            (*pFileNameInfo).FileName.as_ptr() as *const c_void,
            (*pFileNameInfo).FileNameLength as SIZE_T,
        );
        *lpszFilePath.add(final_len as usize) = UNICODE_NULL;
        return final_len;
    }

    /* Query the NT object name */
    Status = NtQueryObject(
        hFile,
        ObjectNameInformation,
        &mut NameInfoBuffer as *mut NameInfoBuf as PVOID,
        (core::mem::size_of::<NameInfoBuf>() - core::mem::size_of::<WCHAR>()) as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    /* NT path requested → return as is */
    if VolumeType == VOLUME_NAME_NT {
        let final_len = (*ObjectName).Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG;
        if cchFilePath < final_len + 1 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return final_len + 1;
        }
        RtlCopyMemory(
            lpszFilePath as PVOID,
            (*ObjectName).Buffer as *const c_void,
            (*ObjectName).Length as SIZE_T,
        );
        *lpszFilePath.add((*ObjectName).Length as usize / core::mem::size_of::<WCHAR>()) =
            UNICODE_NULL;
        return final_len;
    }

    /* For everything else we need to split the path */
    let PathSplit = wcschr(
        (*ObjectName).Buffer.add("\\Device\\".len()),
        '\\' as WCHAR,
    );
    if PathSplit.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    DeviceName.Buffer = (*ObjectName).Buffer;
    DeviceName.Length =
        (PathSplit.offset_from((*ObjectName).Buffer) as USHORT) * core::mem::size_of::<WCHAR>() as USHORT;
    DeviceName.MaximumLength = DeviceName.Length;

    RelativePath.Buffer = PathSplit.add(1);
    RelativePath.Length =
        (*ObjectName).Length - DeviceName.Length - core::mem::size_of::<WCHAR>() as USHORT;
    RelativePath.MaximumLength = RelativePath.Length;

    /* Query the DOS volume name */
    RtlInitEmptyUnicodeString(
        &mut VolumeName,
        VolumeBuffer.as_mut_ptr(),
        core::mem::size_of_val(&VolumeBuffer) as USHORT,
    );
    Status = QueryDosVolumeNameForNtDeviceName(&DeviceName, &mut VolumeName);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if VolumeType == VOLUME_NAME_DOS {
        let final_len = (VolumeName.Length as ULONG + RelativePath.Length as ULONG)
            / core::mem::size_of::<WCHAR>() as ULONG;
        if cchFilePath < final_len + 1 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return final_len + 1;
        }
        RtlCopyMemory(
            lpszFilePath as PVOID,
            VolumeName.Buffer as *const c_void,
            VolumeName.Length as SIZE_T,
        );
        RtlCopyMemory(
            lpszFilePath.add(VolumeName.Length as usize / core::mem::size_of::<WCHAR>()) as PVOID,
            RelativePath.Buffer as *const c_void,
            RelativePath.Length as SIZE_T,
        );
        *lpszFilePath.add(final_len as usize) = UNICODE_NULL;
        return final_len;
    } else if VolumeType == VOLUME_NAME_GUID {
        let Success = GetVolumeNameForVolumeMountPointW(
            VolumeBuffer.as_ptr(),
            VolumeBuffer.as_mut_ptr(),
            VolumeBuffer.len() as DWORD,
        );
        if Success == 0 {
            SetLastError(ERROR_PATH_NOT_FOUND);
            return 0;
        }

        let VolumeLength = wcslen(VolumeBuffer.as_ptr());
        let final_len =
            VolumeLength as ULONG + RelativePath.Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG;
        if cchFilePath < final_len + 1 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return final_len + 1;
        }
        RtlCopyMemory(
            lpszFilePath as PVOID,
            VolumeBuffer.as_ptr() as *const c_void,
            (VolumeLength * core::mem::size_of::<WCHAR>()) as SIZE_T,
        );
        RtlCopyMemory(
            lpszFilePath.add(VolumeLength) as PVOID,
            RelativePath.Buffer as *const c_void,
            RelativePath.Length as SIZE_T,
        );
        *lpszFilePath.add(final_len as usize) = UNICODE_NULL;

        SetLastError(ERROR_SUCCESS);
        return final_len;
    }

    SetLastError(ERROR_INVALID_PARAMETER);
    0
}

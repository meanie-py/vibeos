// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn QueryAvailableFileSystemFormat(
    mut Index: DWORD,
    mut FileSystem: PWCHAR,
    mut Major: *mut UCHAR,
    mut Minor: *mut UCHAR,
    mut LatestVersion: *mut BOOLEAN,
) -> BOOLEAN {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Provider: PIFS_PROVIDER = core::ptr::null_mut();

    if FileSystem.is_null() || Major.is_null() || Minor.is_null() || LatestVersion.is_null() {
        return FALSE as BOOLEAN;
    }

    ListEntry = (*core::ptr::addr_of_mut!(ProviderListHead)).Flink as PLIST_ENTRY;
    loop {
        if ListEntry == core::ptr::addr_of_mut!(ProviderListHead) {
            return FALSE as BOOLEAN;
        }
        if Index == 0 as DWORD {
            break;
        }
        ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
        Index = Index.wrapping_sub(1);
    }

    Provider = CONTAINING_RECORD!(ListEntry, IFS_PROVIDER, ListEntry) as PIFS_PROVIDER;
    wcscpy(FileSystem, (*Provider).Name as PCWSTR);
    *Major = 0 as UCHAR; /* FIXME */
    *Minor = 0 as UCHAR; /* FIXME */
    *LatestVersion = TRUE as BOOLEAN; /* FIXME */

    return TRUE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn QueryDeviceInformation(
    DriveRoot: PCWSTR,
    DeviceInformation: PVOID,
    BufferSize: ULONG,
) -> BOOL {
    let DeviceInfo: PDEVICE_INFORMATION = DeviceInformation as PDEVICE_INFORMATION;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DiskGeometry: DISK_GEOMETRY = core::mem::zeroed();
    let mut HotplugInfo: STORAGE_HOTPLUG_INFO = core::mem::zeroed();
    let mut LengthInformation: GET_LENGTH_INFORMATION = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut DiskDevice: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut DriveName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Buffer should be able to at least hold DeviceFlags */
    if BufferSize < RTL_SIZEOF_THROUGH_FIELD!(DEVICE_INFORMATION, DeviceFlags) as ULONG
        || !NT_SUCCESS!(RtlStringCchCopyW(
            addr_of_mut!(DriveName) as *mut WCHAR,
            (ARRAYSIZE!(DriveName)) as _,
            DriveRoot,
        ))
    {
        return (FALSE) as BOOL;
    }

    if DriveName[wcslen(addr_of_mut!(DriveName) as *mut WCHAR) - 1]
        != 0x005Cu16
    {
        /* Append the trailing backslash for GetVolumeNameForVolumeMountPointW */
        if !NT_SUCCESS!(RtlStringCchCatW(
            addr_of_mut!(DriveName) as *mut WCHAR,
            (ARRAYSIZE!(DriveName)) as _,
            RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
        )) {
            return (FALSE) as BOOL;
        }
    }

    if GetVolumeNameForVolumeMountPointW(
        addr_of_mut!(DriveName) as *mut WCHAR,
        addr_of_mut!(DiskDevice) as *mut WCHAR,
        ARRAYSIZE!(DiskDevice),
    ) == 0
        || RtlDosPathNameToNtPathName_U(
            addr_of_mut!(DiskDevice) as *mut WCHAR as PCWSTR,
            addr_of_mut!(DeviceName),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
    {
        /* Disk has no volume GUID, fallback to QueryDosDevice */
        DriveName[wcslen(addr_of_mut!(DriveName) as *mut WCHAR) - 1] =
            UNICODE_NULL;
        if QueryDosDeviceW(
            addr_of_mut!(DriveName) as *mut WCHAR as LPCWSTR,
            addr_of_mut!(DiskDevice) as *mut WCHAR as LPWSTR,
            ARRAYSIZE!(DiskDevice) as DWORD,
        ) == 0
        {
            return (FALSE) as BOOL;
        }
        RtlInitUnicodeString(
            addr_of_mut!(DeviceName),
            addr_of_mut!(DiskDevice) as *mut WCHAR as PCWSTR,
        );
    } else {
        /* Trim the trailing backslash since we will work with a device object */
        DeviceName.Length -= core::mem::size_of::<WCHAR>() as USHORT;
    }

    /* InitializeObjectAttributes, expanded inline */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = addr_of_mut!(DeviceName);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        addr_of_mut!(FileHandle),
        (FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE) as ACCESS_MASK,
        addr_of_mut!(ObjectAttributes),
        addr_of_mut!(Iosb),
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
        FILE_SYNCHRONOUS_IO_NONALERT as ULONG,
    );
    if DeviceName.Buffer != addr_of_mut!(DiskDevice) as *mut WCHAR {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0 as ULONG,
            DeviceName.Buffer as PVOID,
        );
    }
    if !NT_SUCCESS!(Status) {
        return (FALSE) as BOOL;
    }

    Status = NtDeviceIoControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        addr_of_mut!(Iosb),
        IOCTL_STORAGE_GET_HOTPLUG_INFO as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        addr_of_mut!(HotplugInfo) as PVOID,
        core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as ULONG,
    );

    'quit: {
        if !NT_SUCCESS!(Status) {
            break 'quit;
        }

        (*DeviceInfo).DeviceFlags = 0 as ULONG;
        if HotplugInfo.MediaHotplug != 0 || HotplugInfo.DeviceHotplug != 0 {
            /* This is a hotplug device */
            (*DeviceInfo).DeviceFlags |= DEVICE_HOTPLUG as ULONG;
        }

        /* UNIMPLEMENTED: Other flags that would be set here are related to Sony "Memory Stick"
         * type of devices, which we do not have any special support for */

        if BufferSize >= RTL_SIZEOF_THROUGH_FIELD!(DEVICE_INFORMATION, SectorCount) as ULONG {
            /* This is the Vista+ version of the structure.
             * We need to also provide disk sector size and volume length in sectors. */
            Status = NtDeviceIoControlFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                addr_of_mut!(Iosb),
                IOCTL_DISK_GET_DRIVE_GEOMETRY as ULONG,
                core::ptr::null_mut(),
                0 as ULONG,
                addr_of_mut!(DiskGeometry) as PVOID,
                core::mem::size_of::<DISK_GEOMETRY>() as ULONG,
            );
            if !NT_SUCCESS!(Status) {
                break 'quit;
            }

            Status = NtDeviceIoControlFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                addr_of_mut!(Iosb),
                IOCTL_DISK_GET_LENGTH_INFO as ULONG,
                core::ptr::null_mut(),
                0 as ULONG,
                addr_of_mut!(LengthInformation) as PVOID,
                core::mem::size_of::<GET_LENGTH_INFORMATION>() as ULONG,
            );
            if !NT_SUCCESS!(Status) {
                break 'quit;
            }

            LengthInformation.Length.QuadPart /= DiskGeometry.BytesPerSector as LONGLONG;
            (*DeviceInfo).SectorSize = DiskGeometry.BytesPerSector as ULONG;
            (*DeviceInfo).SectorCount = LengthInformation.Length;
        }

        Status = STATUS_SUCCESS;
    }

    /* Quit: */
    NtClose(FileHandle);
    return NT_SUCCESS!(Status) as BOOL;
}

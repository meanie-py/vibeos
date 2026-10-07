// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1D04A375054D: [u16; 5] = [0x0045u16, 0x0058, 0x0054, 0x0033, 0x0000];
pub static RUSTCONV_WSTR_7B07D8ECA2BA: [u16; 5] = [0x004Eu16, 0x0054, 0x0046, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_89762610BDB9: [u16; 5] = [0x004Eu16, 0x006F, 0x006E, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_903B52F937BA: [u16; 4] = [0x0052u16, 0x0041, 0x0057, 0x0000];
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_BB53EE05898F: [u16; 6] = [0x0046u16, 0x0041, 0x0054, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_BE0DEB469A7B: [u16; 6] = [0x0042u16, 0x0054, 0x0052, 0x0046, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_C8184B8768AD: [u16; 5] = [0x0045u16, 0x0058, 0x0054, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_C8D83D0D9E56: [u16; 5] = [0x0045u16, 0x0058, 0x0054, 0x0032, 0x0000];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> u32;
}

unsafe extern "C" fn GetFileSystemNameWorker(
    mut PartitionHandle: HANDLE,
    mut IoStatusBlock: PIO_STATUS_BLOCK,
    mut FileSystemName: PWSTR,
    mut FileSystemNameSize: SIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Buffer: [UCHAR; core::mem::size_of::<FILE_FS_ATTRIBUTE_INFORMATION>() + (MAX_PATH as usize) * core::mem::size_of::<WCHAR>()] = [0; core::mem::size_of::<FILE_FS_ATTRIBUTE_INFORMATION>() + (MAX_PATH as usize) * core::mem::size_of::<WCHAR>()];
    let mut FileFsAttribute: PFILE_FS_ATTRIBUTE_INFORMATION = &raw mut Buffer as *mut UCHAR as PFILE_FS_ATTRIBUTE_INFORMATION;
    Status = NtQueryVolumeInformationFile(
        PartitionHandle,
        IoStatusBlock,
        FileFsAttribute as PVOID,
        core::mem::size_of_val(&Buffer) as u32,
        FileFsAttributeInformation,
    );
    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) NtQueryVolumeInformationFile() failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
            b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const core::ffi::c_char,
            42 as core::ffi::c_int,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const core::ffi::c_char,
                42 as core::ffi::c_int,
            );
        }
        return Status;
    }
    if (FileSystemNameSize as usize) < ((*FileFsAttribute).FileSystemNameLength as usize) + core::mem::size_of::<WCHAR>() {
        return STATUS_BUFFER_TOO_SMALL;
    }
    return RtlStringCbCopyNW(
        FileSystemName,
        FileSystemNameSize as usize,
        &raw mut (*FileFsAttribute).FileSystemName as *mut WCHAR as *const u16,
        (*FileFsAttribute).FileSystemNameLength as usize,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetFileSystemName_UStr(
    mut PartitionPath: PUNICODE_STRING,
    mut PartitionHandle: HANDLE,
    mut FileSystemName: PWSTR,
    mut FileSystemNameSize: SIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if !PartitionPath.is_null() && !PartitionHandle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !PartitionPath.is_null() {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = PartitionPath;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as u32;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            core::ptr::addr_of_mut!(PartitionHandle),
            FILE_GENERIC_READ as u32,
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(IoStatusBlock),
            (FILE_SHARE_READ | FILE_SHARE_WRITE) as u32,
            0 as u32,
        );

        if ((NT_SUCCESS!(Status)) == 0) {
            if DbgPrint(
                b"(%s:%d) Failed to open partition '%wZ', Status 0x%08lx\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
                86 as i32,
                PartitionPath,
                Status,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
                    86 as i32,
                );
            }
            return Status;
        }
    }

    Status = GetFileSystemNameWorker(
        PartitionHandle,
        core::ptr::addr_of_mut!(IoStatusBlock),
        FileSystemName,
        FileSystemNameSize,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        if DbgPrint(
            b"(%s:%d) GetFileSystemName() failed for partition '%wZ' (0x%p), Status 0x%08lx\n\0".as_ptr() as *const i8,
            b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
            99 as i32,
            PartitionPath,
            PartitionHandle,
            Status,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
                99 as i32,
            );
        }
    }

    if !PartitionPath.is_null() {
        NtClose(PartitionHandle);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn GetFileSystemName(
    PartitionPath: PCWSTR,
    PartitionHandle: HANDLE,
    FileSystemName: PWSTR,
    FileSystemNameSize: SIZE_T,
) -> NTSTATUS {
    let mut PartitionPathU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if !PartitionPath.is_null() && !PartitionHandle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !PartitionPath.is_null() {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(PartitionPathU),
            PartitionPath,
        );
    }

    return GetFileSystemName_UStr(
        if !PartitionPath.is_null() {
            core::ptr::addr_of_mut!(PartitionPathU)
        } else {
            core::ptr::null_mut()
        },
        if !PartitionPath.is_null() {
            core::ptr::null_mut()
        } else {
            PartitionHandle
        },
        FileSystemName,
        FileSystemNameSize,
    );
}

unsafe extern "C" fn InferFileSystemWorker(
    PartitionHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileSystemName: PWSTR,
    FileSystemNameSize: SIZE_T,
) -> NTSTATUS {
    union PartInfoUnion {
        InfoEx: PARTITION_INFORMATION_EX,
        Info: PARTITION_INFORMATION,
    }

    let mut Status: NTSTATUS = 0;
    let mut Status2: NTSTATUS = 0;
    let mut PartInfo: PartInfoUnion = core::mem::zeroed();
    let mut PartitionType: u8 = 0;

    'quit: {
        if (FileSystemNameSize as usize) < core::mem::size_of::<u16>() {
            return STATUS_BUFFER_TOO_SMALL;
        }

        *FileSystemName = 0x0000u16;

        Status = GetFileSystemName_UStr(
            core::ptr::null_mut::<_UNICODE_STRING>(),
            PartitionHandle,
            FileSystemName,
            FileSystemNameSize,
        );
        if (Status >= 0) && (*FileSystemName != 0) {
            break 'quit;
        }

        Status2 = NtDeviceIoControlFile(
            PartitionHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            IoStatusBlock,
            IOCTL_DISK_GET_PARTITION_INFO_EX as u32,
            core::ptr::null_mut(),
            0 as u32,
            core::ptr::addr_of_mut!(PartInfo.InfoEx) as *mut core::ffi::c_void,
            core::mem::size_of::<PARTITION_INFORMATION_EX>() as u32,
        );
        if !(Status2 >= 0) {
            if DbgPrint(
                b"(%s:%d) IOCTL_DISK_GET_PARTITION_INFO_EX failed (Status %lx)\n\0".as_ptr() as *const i8,
                b"fsrec.c\0".as_ptr() as *const i8,
                178 as i32,
                Status2,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"fsrec.c\0".as_ptr() as *const i8,
                    178 as i32,
                );
            }

            if Status2 != STATUS_INVALID_DEVICE_REQUEST as i32 {
                break 'quit;
            }

            Status2 = NtDeviceIoControlFile(
                PartitionHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                IoStatusBlock,
                IOCTL_DISK_GET_PARTITION_INFO as u32,
                core::ptr::null_mut(),
                0 as u32,
                core::ptr::addr_of_mut!(PartInfo.Info) as *mut core::ffi::c_void,
                core::mem::size_of::<PARTITION_INFORMATION>() as u32,
            );
            if !(Status2 >= 0) {
                if DbgPrint(
                    b"(%s:%d) IOCTL_DISK_GET_PARTITION_INFO failed (Status %lx)\n\0".as_ptr() as *const i8,
                    b"fsrec.c\0".as_ptr() as *const i8,
                    200 as i32,
                    Status2,
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"fsrec.c\0".as_ptr() as *const i8,
                        200 as i32,
                    );
                }

                break 'quit;
            }

            PartitionType = PartInfo.Info.PartitionType;
        } else {
            if PartInfo.InfoEx.PartitionStyle as i32 != PARTITION_STYLE_MBR as i32 {
                break 'quit;
            }
            PartitionType = PartInfo.InfoEx.__bindgen_anon_1.Mbr.PartitionType;
        }

        if (PartitionType as i32 == PARTITION_FAT_12 as i32)
            || (PartitionType as i32 == PARTITION_FAT_16 as i32)
            || (PartitionType as i32 == PARTITION_HUGE as i32)
            || (PartitionType as i32 == PARTITION_XINT13 as i32)
        {
            Status = RtlStringCbCopyW(FileSystemName, (FileSystemNameSize) as _, RUSTCONV_WSTR_A953DDDD78B1.as_ptr());
        } else if (PartitionType as i32 == PARTITION_FAT32 as i32)
            || (PartitionType as i32 == PARTITION_FAT32_XINT13 as i32)
        {
            Status = RtlStringCbCopyW(FileSystemName, (FileSystemNameSize) as _, RUSTCONV_WSTR_BB53EE05898F.as_ptr());
        } else if PartitionType as i32 == PARTITION_LINUX as i32 {
            Status = RtlStringCbCopyW(FileSystemName, (FileSystemNameSize) as _, RUSTCONV_WSTR_BE0DEB469A7B.as_ptr());
        } else if PartitionType as i32 == PARTITION_IFS as i32 {
            Status = RtlStringCbCopyW(FileSystemName, (FileSystemNameSize) as _, RUSTCONV_WSTR_7B07D8ECA2BA.as_ptr());
        }
    }

    if (*FileSystemName != 0)
        && _wcsicmp(FileSystemName as *const u16, RUSTCONV_WSTR_7B07D8ECA2BA.as_ptr())
            == 0 as i32
    {
        if DbgPrint(
            b"(%s:%d) Recognized file system '%S' that doesn't have write support yet!\n\0".as_ptr() as *const i8,
            b"fsrec.c\0".as_ptr() as *const i8,
            262 as i32,
            FileSystemName,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"fsrec.c\0".as_ptr() as *const i8,
                262 as i32,
            );
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn InferFileSystem(
    mut PartitionPath: PCWSTR,
    mut PartitionHandle: HANDLE,
    mut FileSystemName: PWSTR,
    mut FileSystemNameSize: SIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut PartitionPathU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };

    if !PartitionPath.is_null() && !PartitionHandle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !PartitionPath.is_null() {
        RtlInitUnicodeString(&raw mut PartitionPathU, PartitionPath);
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &raw mut PartitionPathU;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

        Status = NtOpenFile(
            &raw mut PartitionHandle,
            FILE_GENERIC_READ,
            &raw mut ObjectAttributes,
            &raw mut IoStatusBlock,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            0,
        );

        if ((NT_SUCCESS!(Status)) == 0) {
            if DbgPrint(
                b"(%s:%d) Failed to open partition '%S', Status 0x%08lx\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
                302i32,
                PartitionPath,
                Status,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
                    302i32,
                );
            }
            return Status;
        }
    }

    Status = InferFileSystemWorker(
        PartitionHandle,
        &raw mut IoStatusBlock,
        FileSystemName,
        FileSystemNameSize,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        if DbgPrint(
            b"(%s:%d) InferFileSystem() failed for partition '%S' (0x%p), Status 0x%08lx\n\0".as_ptr() as *const i8,
            b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
            315i32,
            PartitionPath,
            PartitionHandle,
            Status,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
                315i32,
            );
        }
    } else if DbgPrint(
        b"(%s:%d) InferFileSystem(): FileSystem (guessed): %S\n\0".as_ptr() as *const i8,
        b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
        320i32,
        if *FileSystemName as i32 != 0 {
            FileSystemName as *const u16
        } else {
            RUSTCONV_WSTR_89762610BDB9.as_ptr()
        },
    ) != 0 {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"base/setup/lib/utils/fsrec.c\0".as_ptr() as *const i8,
            320i32,
        );
    }

    if !PartitionPath.is_null() {
        NtClose(PartitionHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn FileSystemToMBRPartitionType(
    FileSystem: PCWSTR,
    StartSector: ULONGLONG,
    SectorCount: ULONGLONG,
) -> UCHAR {
    if SectorCount == 0 {
        return PARTITION_ENTRY_UNUSED as UCHAR;
    }

    if _wcsicmp(FileSystem, RUSTCONV_WSTR_A953DDDD78B1.as_ptr()) == 0
        || _wcsicmp(FileSystem, RUSTCONV_WSTR_BB53EE05898F.as_ptr()) == 0
        || _wcsicmp(FileSystem, RUSTCONV_WSTR_903B52F937BA.as_ptr()) == 0
    {
        if SectorCount < 8192 {
            /* FAT12 CHS partition (disk is smaller than 4.1MB) */
            return PARTITION_FAT_12 as UCHAR;
        } else if StartSector < 1450560 {
            /* Partition starts below the 8.4GB boundary ==> CHS partition */

            if SectorCount < 65536 {
                /* FAT16 CHS partition (partition size < 32MB) */
                return PARTITION_FAT_16 as UCHAR;
            } else if SectorCount < 1048576 {
                /* FAT16 CHS partition (partition size < 512MB) */
                return PARTITION_HUGE as UCHAR;
            } else {
                /* FAT32 CHS partition (partition size >= 512MB) */
                return PARTITION_FAT32 as UCHAR;
            }
        } else {
            /* Partition starts above the 8.4GB boundary ==> LBA partition */

            if SectorCount < 1048576 {
                /* FAT16 LBA partition (partition size < 512MB) */
                return PARTITION_XINT13 as UCHAR;
            } else {
                /* FAT32 LBA partition (partition size >= 512MB) */
                return PARTITION_FAT32_XINT13 as UCHAR;
            }
        }
    } else if _wcsicmp(FileSystem, RUSTCONV_WSTR_7B07D8ECA2BA.as_ptr()) == 0 {
        return PARTITION_IFS as UCHAR;
    } else if _wcsicmp(FileSystem, RUSTCONV_WSTR_BE0DEB469A7B.as_ptr()) == 0
        || _wcsicmp(FileSystem, RUSTCONV_WSTR_C8D83D0D9E56.as_ptr()) == 0
        || _wcsicmp(FileSystem, RUSTCONV_WSTR_1D04A375054D.as_ptr()) == 0
        || _wcsicmp(FileSystem, RUSTCONV_WSTR_C8184B8768AD.as_ptr()) == 0
    {
        return PARTITION_LINUX as UCHAR;
    } else {
        /* Unknown file system */
        DPRINT1!(b"Unknown file system '%S'\n\0".as_ptr() as *const i8, FileSystem);
        return PARTITION_ENTRY_UNUSED as UCHAR;
    }
}

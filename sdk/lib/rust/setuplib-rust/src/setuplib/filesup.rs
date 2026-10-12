// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1F163068ACB2: [u16; 11] = [
    0x005Cu16, 0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_E82559BA02DE: [u16; 17] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0000,
];
pub static RUSTCONV_WSTR_F13BC6011403: [u16; 9] = [0x006Bu16, 0x0065, 0x0072, 0x006E, 0x0065, 0x006C, 0x0033, 0x0032, 0x0000];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> u32;
}

unsafe extern "C" fn SetupCreateSingleDirectory(DirectoryName: PCUNICODE_STRING) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut PathName: _UNICODE_STRING = *DirectoryName;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DirectoryHandle: *mut core::ffi::c_void = core::mem::zeroed();

    if PathName.Length as usize >= 2 * core::mem::size_of::<u16>()
        && *PathName.Buffer.offset(
            (PathName.Length as usize / core::mem::size_of::<u16>()) as isize
                - 1 as isize,
        ) == (OBJ_NAME_PATH_SEPARATOR) as _
    {
        PathName.Length -= core::mem::size_of::<u16>() as u16;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut PathName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        &mut DirectoryHandle,
        FILE_LIST_DIRECTORY | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_DIRECTORY,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN_IF,
        FILE_OPEN_FOR_BACKUP_INTENT | FILE_DIRECTORY_FILE,
        core::ptr::null_mut(),
        0,
    );
    if NT_SUCCESS!(Status) != 0 {
        NtClose(DirectoryHandle);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn SetupCreateDirectory(PathName: PCWSTR) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut PathNameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Buffer: PCWSTR;
    let mut Ptr: PCWCH;
    let mut End: PCWCH;

    RtlInitUnicodeString(&mut PathNameU, PathName);
    Buffer = PathNameU.Buffer;
    End = Buffer.add((PathNameU.Length as usize) / core::mem::size_of::<u16>());

    Ptr = End;
    while Ptr > Buffer {
        let mut bExists: i32;

        if (Ptr < End) && (*Ptr == (OBJ_NAME_PATH_SEPARATOR) as _) {
            PathNameU.Length = ((Ptr.add(1) as usize) - (Buffer as usize)) as u16;
        }

        DPRINT!(b"PathName: %wZ\n\0".as_ptr() as *const i8, &PathNameU);
        bExists = (DoesPathExist_UStr(core::ptr::null_mut(), &PathNameU, (TRUE) as BOOLEAN)) as _;
        if bExists != 0 {
            break;
        }

        while (Ptr > Buffer) && (*Ptr == (OBJ_NAME_PATH_SEPARATOR) as _) {
            Ptr = Ptr.sub(1);
        }
        while (Ptr > Buffer) && (*Ptr != (OBJ_NAME_PATH_SEPARATOR) as _) {
            Ptr = Ptr.sub(1);
        }
    }

    while (Ptr < End) && (*Ptr == (OBJ_NAME_PATH_SEPARATOR) as _) {
        Ptr = Ptr.add(1);
    }

    while Ptr < End {
        while (Ptr < End) && (*Ptr != (OBJ_NAME_PATH_SEPARATOR) as _) {
            Ptr = Ptr.add(1);
        }
        PathNameU.Length = ((Ptr as usize) - (Buffer as usize)) as u16;

        DPRINT!(b"Create: %wZ\n\0".as_ptr() as *const i8, &PathNameU);
        Status = SetupCreateSingleDirectory(&PathNameU);
        if ((NT_SUCCESS!(Status)) == 0) {
            break;
        }
        Ptr = Ptr.add(1);
    }

    DPRINT!(b"Done.\n\0".as_ptr() as *const i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn SetupDeleteFile(
    FileName: PCWSTR,
    ForceDelete: u8,
) -> i32 {
    let mut Status: i32;
    let mut NtPathU: _UNICODE_STRING = _UNICODE_STRING {
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
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut FileDispInfo: FILE_DISPOSITION_INFORMATION = FILE_DISPOSITION_INFORMATION {
        DeleteFileA: 0,
    };
    let mut RetryOnce: u8 = (FALSE) as u8;

    RtlInitUnicodeString(&mut NtPathU, FileName);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut NtPathU;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    loop {
        Status = NtOpenFile(
            &mut FileHandle,
            DELETE | FILE_READ_ATTRIBUTES |
                (if RetryOnce != 0 { FILE_WRITE_ATTRIBUTES } else { 0 }),
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_NON_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("NtOpenFile failed with Status 0x%08lx\n", Status);
            return Status;
        }

        if RetryOnce != 0 {
            let mut FileInformation: FILE_BASIC_INFORMATION = FILE_BASIC_INFORMATION {
                CreationTime: { let mut __lit: LARGE_INTEGER = core::mem::zeroed(); __lit.__bindgen_anon_1.LowPart = 0; __lit.__bindgen_anon_1.HighPart = 0; __lit },
                LastAccessTime: { let mut __lit: LARGE_INTEGER = core::mem::zeroed(); __lit.__bindgen_anon_1.LowPart = 0; __lit.__bindgen_anon_1.HighPart = 0; __lit },
                LastWriteTime: { let mut __lit: LARGE_INTEGER = core::mem::zeroed(); __lit.__bindgen_anon_1.LowPart = 0; __lit.__bindgen_anon_1.HighPart = 0; __lit },
                ChangeTime: { let mut __lit: LARGE_INTEGER = core::mem::zeroed(); __lit.__bindgen_anon_1.LowPart = 0; __lit.__bindgen_anon_1.HighPart = 0; __lit },
                FileAttributes: 0,
            };

            Status = NtQueryInformationFile(
                FileHandle,
                &mut IoStatusBlock,
                &mut FileInformation as *mut _ as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
                FileBasicInformation,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!("NtQueryInformationFile failed with Status 0x%08lx\n", Status);
                NtClose(FileHandle);
                return Status;
            }

            FileInformation.FileAttributes = FILE_ATTRIBUTE_NORMAL;
            Status = NtSetInformationFile(
                FileHandle,
                &mut IoStatusBlock,
                &mut FileInformation as *mut _ as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
                FileBasicInformation,
            );
            NtClose(FileHandle);
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!("NtSetInformationFile failed with Status 0x%08lx\n", Status);
                return Status;
            }
        }

        FileDispInfo.DeleteFileA = (TRUE) as u8;
        Status = NtSetInformationFile(
            FileHandle,
            &mut IoStatusBlock,
            &mut FileDispInfo as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as u32,
            FileDispositionInformation,
        );
        NtClose(FileHandle);

        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("Deletion of file '%S' failed, Status 0x%08lx\n", FileName, Status);
        }

        if ((NT_SUCCESS!(Status)) == 0) && ForceDelete != 0 && RetryOnce == 0 {
            RetryOnce = (TRUE) as u8;
            continue;
        }
        break;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn SetupCopyFile(
    SourceFileName: PCWSTR,
    DestinationFileName: PCWSTR,
    FailIfExists: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileHandleSource: HANDLE = core::ptr::null_mut();
    let mut FileHandleDest: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut RegionSize: ULONG;
    let mut SourceFileSection: HANDLE = core::ptr::null_mut();
    let mut SourceFileMap: PVOID = core::ptr::null_mut();
    let mut SourceSectionSize: SIZE_T = 0;
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();

    RtlInitUnicodeString(&mut FileName, SourceFileName);
    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut FileName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtOpenFile(
        &mut FileHandleSource,
        GENERIC_READ,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ,
        FILE_SEQUENTIAL_ONLY,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtOpenFile failed: %x, %wZ\n", Status, &FileName);
        return Status;
    }

    Status = NtQueryInformationFile(
        FileHandleSource,
        &mut IoStatusBlock,
        &mut FileStandard as *mut _ as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        FileStandardInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtQueryInformationFile failed: %x\n", Status);
        NtClose(FileHandleSource);
        return Status;
    }

    Status = NtQueryInformationFile(
        FileHandleSource,
        &mut IoStatusBlock,
        &mut FileBasic as *mut _ as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtQueryInformationFile failed: %x\n", Status);
        NtClose(FileHandleSource);
        return Status;
    }

    Status = NtCreateSection(
        &mut SourceFileSection,
        SECTION_MAP_READ,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_READONLY,
        SEC_COMMIT,
        FileHandleSource,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtCreateSection failed: %x, %S\n", Status, SourceFileName);
        NtClose(FileHandleSource);
        return Status;
    }

    Status = NtMapViewOfSection(
        SourceFileSection,
        NtCurrentProcess(),
        &mut SourceFileMap,
        0,
        0,
        core::ptr::null_mut(),
        &mut SourceSectionSize,
        ViewUnmap,
        0,
        PAGE_READONLY,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtMapViewOfSection failed: %x, %S\n", Status, SourceFileName);
        NtClose(SourceFileSection);
        NtClose(FileHandleSource);
        return Status;
    }

    RtlInitUnicodeString(&mut FileName, DestinationFileName);
    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut FileName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtCreateFile(
        &mut FileHandleDest,
        GENERIC_WRITE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FileBasic.FileAttributes,
        0,
        if FailIfExists != 0 {
            FILE_CREATE
        } else {
            FILE_OVERWRITE_IF
        },
        FILE_NO_INTERMEDIATE_BUFFERING | FILE_SEQUENTIAL_ONLY | FILE_SYNCHRONOUS_IO_NONALERT,
        core::ptr::null_mut(),
        0,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        if Status == STATUS_ACCESS_DENIED {
            let mut FileBasicInfo_local: FILE_BASIC_INFORMATION = core::mem::zeroed();

            Status = NtCreateFile(
                &mut FileHandleDest,
                FILE_WRITE_ATTRIBUTES | SYNCHRONIZE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                core::ptr::null_mut(),
                FILE_ATTRIBUTE_NORMAL,
                0,
                FILE_OPEN,
                FILE_NO_INTERMEDIATE_BUFFERING | FILE_SEQUENTIAL_ONLY | FILE_SYNCHRONOUS_IO_NONALERT,
                core::ptr::null_mut(),
                0,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!("NtCreateFile failed: %x, %wZ\n", Status, &FileName);
                NtUnmapViewOfSection(NtCurrentProcess(), SourceFileMap);
                NtClose(SourceFileSection);
                NtClose(FileHandleSource);
                return Status;
            }

            RtlZeroMemory!(
                &mut FileBasicInfo_local as *mut _ as PVOID,
                core::mem::size_of::<FILE_BASIC_INFORMATION>(),
            );
            FileBasicInfo_local.FileAttributes = FILE_ATTRIBUTE_NORMAL;
            NtSetInformationFile(
                FileHandleDest,
                &mut IoStatusBlock,
                &mut FileBasicInfo_local as *mut _ as PVOID,
                core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
                FileBasicInformation,
            );

            NtClose(FileHandleDest);

            Status = NtCreateFile(
                &mut FileHandleDest,
                GENERIC_WRITE | SYNCHRONIZE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                core::ptr::null_mut(),
                FILE_ATTRIBUTE_NORMAL,
                0,
                FILE_OVERWRITE_IF,
                FILE_NO_INTERMEDIATE_BUFFERING | FILE_SEQUENTIAL_ONLY | FILE_SYNCHRONOUS_IO_NONALERT,
                core::ptr::null_mut(),
                0,
            );
        }

        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!("NtCreateFile failed: %x, %wZ\n", Status, &FileName);
            NtUnmapViewOfSection(NtCurrentProcess(), SourceFileMap);
            NtClose(SourceFileSection);
            NtClose(FileHandleSource);
            return Status;
        }
    }

    RegionSize = PAGE_ROUND_UP!(FileStandard.EndOfFile.u.LowPart) as ULONG;
    IoStatusBlock.__bindgen_anon_1.Status = 0;
    ByteOffset.QuadPart = 0;
    Status = NtWriteFile(
        FileHandleDest,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        SourceFileMap,
        RegionSize,
        &mut ByteOffset,
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            "NtWriteFile failed: %x:%x, iosb: %p src: %p, size: %x\n",
            Status,
            IoStatusBlock.__bindgen_anon_1.Status,
            &IoStatusBlock,
            SourceFileMap,
            RegionSize,
        );
        NtClose(FileHandleDest);
        NtUnmapViewOfSection(NtCurrentProcess(), SourceFileMap);
        NtClose(SourceFileSection);
        NtClose(FileHandleSource);
        return Status;
    }

    Status = NtSetInformationFile(
        FileHandleDest,
        &mut IoStatusBlock,
        &mut FileBasic as *mut _ as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtSetInformationFile failed: %x\n", Status);
        NtClose(FileHandleDest);
        NtUnmapViewOfSection(NtCurrentProcess(), SourceFileMap);
        NtClose(SourceFileSection);
        NtClose(FileHandleSource);
        return Status;
    }

    Status = NtSetInformationFile(
        FileHandleDest,
        &mut IoStatusBlock,
        &mut FileStandard.EndOfFile as *mut _ as PVOID,
        core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as ULONG,
        FileEndOfFileInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtSetInformationFile failed: %x\n", Status);
    }

    NtClose(FileHandleDest);
    NtUnmapViewOfSection(NtCurrentProcess(), SourceFileMap);
    NtClose(SourceFileSection);
    NtClose(FileHandleSource);

    Status
}

#[no_mangle]
pub unsafe extern "C" fn SetupMoveFile(
    ExistingFileName: PCWSTR,
    NewFileName: PCWSTR,
    Flags: u32,
) -> i32 {
    let mut Status: i32 = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RenameInfo: PFILE_RENAME_INFORMATION = core::ptr::null_mut();
    let mut NewPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut ExistingPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut SourceHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ReplaceIfExists: u8 = 0;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ExistingPathU), ExistingFileName);
    RtlInitUnicodeString(core::ptr::addr_of_mut!(NewPathU), NewFileName);

    '_SEH2_TRY: {
        ReplaceIfExists = ((Flags & MOVEFILE_REPLACE_EXISTING) != 0) as u8;

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as u32;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(ExistingPathU);
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            core::ptr::addr_of_mut!(SourceHandle),
            FILE_READ_ATTRIBUTES | DELETE | SYNCHRONIZE,
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(IoStatusBlock),
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_OPEN_FOR_BACKUP_INTENT
                | if (Flags & MOVEFILE_WRITE_THROUGH) != 0 {
                    FILE_WRITE_THROUGH
                } else {
                    0
                },
        );

        if ((NT_SUCCESS!(Status)) == 0) {
            if Status != STATUS_INVALID_PARAMETER {
                break '_SEH2_TRY;
            }
        }

        if SourceHandle.is_null() {
            RtlAssert(
                b"SourceHandle\0".as_ptr() as *mut core::ffi::c_void,
                b"base/setup/lib/utils/filesup.c\0"
                    .as_ptr() as *mut core::ffi::c_void,
                506u32,
                core::ptr::null_mut::<i8>(),
            );
        }

        RenameInfo = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            0 as u32,
            (NewPathU.Length as usize + core::mem::size_of::<FILE_RENAME_INFORMATION>()) as SIZE_T,
        ) as PFILE_RENAME_INFORMATION;

        if RenameInfo.is_null() {
            Status = STATUS_NO_MEMORY;
            break '_SEH2_TRY;
        }

        RtlCopyMemory!(
            core::ptr::addr_of_mut!((*RenameInfo).FileName) as *mut core::ffi::c_void,
            NewPathU.Buffer as *const core::ffi::c_void,
            NewPathU.Length as usize,
        );
        (*RenameInfo).ReplaceIfExists = ReplaceIfExists;
        (*RenameInfo).RootDirectory = core::ptr::null_mut();
        (*RenameInfo).FileNameLength = NewPathU.Length as u32;

        Status = NtSetInformationFile(
            SourceHandle,
            core::ptr::addr_of_mut!(IoStatusBlock),
            RenameInfo as *mut core::ffi::c_void,
            (NewPathU.Length as usize + core::mem::size_of::<FILE_RENAME_INFORMATION>()) as u32,
            FileRenameInformation,
        );

        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0 as u32,
            RenameInfo as *mut core::ffi::c_void,
        );

        if NT_SUCCESS!(Status) != 0 {
            break '_SEH2_TRY;
        }

        if Status != STATUS_NOT_SAME_DEVICE || (Flags & MOVEFILE_COPY_ALLOWED) == 0 {
            if Status == STATUS_NOT_IMPLEMENTED {
                if DbgPrint(
                    b"(%s:%d) Forcing copy, renaming not supported by FSD\n\0".as_ptr()
                        as *const i8,
                    b"base/setup/lib/utils/filesup.c\0"
                        .as_ptr() as *const i8,
                    545i32,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"base/setup/lib/utils/filesup.c\0"
                            .as_ptr() as *const i8,
                        545i32,
                    );
                }
            } else {
                break '_SEH2_TRY;
            }
        }

        NtClose(SourceHandle);
        SourceHandle = core::ptr::null_mut();

        Status = SetupCopyFile(
            ExistingFileName,
            NewFileName,
            (ReplaceIfExists == 0) as u8,
        );

        if NT_SUCCESS!(Status) != 0 {
            SetupDeleteFile(ExistingFileName, (TRUE) as u8);
        }
    }

    '_SEH2_FINALLY: {
        if !SourceHandle.is_null() {
            NtClose(SourceHandle);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn ConcatPathsV(
    PathBuffer: PWSTR,
    cchPathSize: usize,
    NumberOfPathComponents: u32,
    mut PathComponentsList: *mut core::ffi::c_char, // va_list: `char *` on our three targets
) -> i32 {
    let mut Status: i32 = STATUS_SUCCESS;
    let mut cchPathLen: usize;
    let mut PathComponent: PCWSTR;

    if cchPathSize < 1 {
        return STATUS_SUCCESS;
    }

    let mut NumberOfPathComponents = NumberOfPathComponents;
    // C's va_arg: varargs-rust pulls through the module's shim and advances this copy.
    let mut Args = varargs_rust::VaArgs::from_ptr(core::ptr::addr_of_mut!(PathComponentsList).cast());
    while NumberOfPathComponents != 0 {
        NumberOfPathComponents -= 1;
        PathComponent = Args.arg::<PCWSTR>();
        if PathComponent.is_null() {
            continue;
        }

        cchPathLen = min(cchPathSize, wcslen(PathBuffer));
        if cchPathLen >= cchPathSize {
            return STATUS_BUFFER_OVERFLOW;
        }

        if *PathComponent.offset(0 as isize) as i32 != (OBJ_NAME_PATH_SEPARATOR) as _
            && cchPathLen > 0
            && *PathBuffer.offset((cchPathLen - 1) as isize) as i32
                != (OBJ_NAME_PATH_SEPARATOR) as _
        {
            Status = RtlStringCchCatW(PathBuffer, cchPathSize, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
            if ((NT_SUCCESS!(Status)) == 0) {
                return Status;
            }
        } else if *PathComponent.offset(0 as isize) as i32
            == (OBJ_NAME_PATH_SEPARATOR) as _
            && cchPathLen > 0
            && *PathBuffer.offset((cchPathLen - 1) as isize) as i32
                == (OBJ_NAME_PATH_SEPARATOR) as _
        {
            while *PathComponent as i32 == (OBJ_NAME_PATH_SEPARATOR) as _ {
                PathComponent = PathComponent.offset(1);
            }
        }
        Status = RtlStringCchCatW(PathBuffer, cchPathSize, PathComponent);
        if ((NT_SUCCESS!(Status)) == 0) {
            return Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn CombinePathsV(
    PathBuffer: PWSTR,
    cchPathSize: usize,
    NumberOfPathComponents: u32,
    PathComponentsList: *mut core::ffi::c_char, // va_list: `char *` on our three targets
) -> i32 {
    if cchPathSize < 1 {
        return STATUS_SUCCESS;
    }
    *PathBuffer = UNICODE_NULL;
    return ConcatPathsV(PathBuffer, cchPathSize, NumberOfPathComponents, PathComponentsList);
}

extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn ConcatPaths(PathBuffer: PWSTR, cchPathSize: usize, NumberOfPathComponents: u32, ...) -> i32;
}

/// `ConcatPaths(PathBuffer, cchPathSize, NumberOfPathComponents, ...)`, defined through
/// varargs-rust's naked thunk: `a.get(i)` is fixed argument `i`, `a.va(3)` is the `va_list`
/// C's `va_start(.., NumberOfPathComponents)` would have produced.
mod ConcatPaths_export {
    use super::*;
    varargs_rust::va_entry!(ConcatPaths => |a| -> i32 {
        let PathBuffer = a.get(0) as PWSTR;
        let cchPathSize = a.get(1) as usize;
        let NumberOfPathComponents = a.get(2) as u32;
        if cchPathSize < 1 {
            return STATUS_SUCCESS;
        }
        ConcatPathsV(PathBuffer, cchPathSize, NumberOfPathComponents, a.va(3) as *mut core::ffi::c_char)
    });
}

extern "C" {
    /// C-variadic: DEFINED below by `va_entry!` (the symbol), declared here so the crate's own
    /// callers can pass it varargs.
    pub fn CombinePaths(PathBuffer: PWSTR, cchPathSize: usize, NumberOfPathComponents: u32, ...) -> i32;
}

/// `CombinePaths(PathBuffer, cchPathSize, NumberOfPathComponents, ...)`, defined through
/// varargs-rust's naked thunk: `a.get(i)` is fixed argument `i`, `a.va(3)` is the `va_list`
/// C's `va_start(.., NumberOfPathComponents)` would have produced.
mod CombinePaths_export {
    use super::*;
    varargs_rust::va_entry!(CombinePaths => |a| -> i32 {
        let PathBuffer = a.get(0) as PWSTR;
        let cchPathSize = a.get(1) as usize;
        let NumberOfPathComponents = a.get(2) as u32;
        if cchPathSize < 1 {
            return STATUS_SUCCESS;
        }
        *PathBuffer = UNICODE_NULL;
        CombinePathsV(PathBuffer, cchPathSize, NumberOfPathComponents, a.va(3) as *mut core::ffi::c_char)
    });
}
#[no_mangle]
pub unsafe extern "C" fn DoesPathExist_UStr(
    RootDirectory: HANDLE,
    PathName: PCUNICODE_STRING,
    IsDirectory: BOOLEAN,
) -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = RootDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = PathName as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        if IsDirectory != 0 {
            FILE_LIST_DIRECTORY | SYNCHRONIZE
        } else {
            FILE_GENERIC_READ
        },
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT
            | if IsDirectory != 0 {
                FILE_DIRECTORY_FILE
            } else {
                FILE_NON_DIRECTORY_FILE
            },
    );

    if NT_SUCCESS!(Status) != 0 {
        NtClose(FileHandle);
    } else {
        DPRINT!(
            b"Failed to open %s '%wZ', Status 0x%08lx\n\0".as_ptr() as *const i8,
            if IsDirectory != 0 {
                b"directory\0".as_ptr() as *const i8
            } else {
                b"file\0".as_ptr() as *const i8
            },
            PathName,
            Status,
        );
    }

    return NT_SUCCESS!(Status) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn DoesPathExist(
    RootDirectory: HANDLE,
    PathName: PCWSTR,
    IsDirectory: BOOLEAN,
) -> BOOLEAN {
    let mut PathNameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(&mut PathNameU, PathName);
    return DoesPathExist_UStr(RootDirectory, &PathNameU, IsDirectory);
}
#[no_mangle]
pub unsafe extern "C" fn DoesFileExist_2(
    PathName: PCWSTR,
    FileName: PCWSTR,
) -> i32 {
    let mut FullName: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    CombinePaths(
        FullName.as_mut_ptr(),
        (MAX_PATH) as _,
        2,
        PathName,
        FileName,
    );
    return DoesFileExist!(core::ptr::null_mut(), FullName.as_ptr()) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn NtPathToDiskPartComponents(
    NtPath: PCWSTR,
    pDiskNumber: PULONG,
    pPartNumber: PULONG,
    PathComponent: *mut PCWSTR,
) -> BOOLEAN {
    let mut DiskNumber: ULONG;
    let mut PartNumber: ULONG;
    let mut Path: PCWSTR;

    *pDiskNumber = 0;
    *pPartNumber = 0;
    if !PathComponent.is_null() {
        *PathComponent = core::ptr::null_mut();
    }

    Path = NtPath;

    if _wcsnicmp(Path, RUSTCONV_WSTR_E82559BA02DE.as_ptr(), 16) != 0 {
        DPRINT1!(b"'%S' : Not a possible hard disk device.\n\0".as_ptr() as *const i8, NtPath);
        return (FALSE) as u8;
    }

    Path = Path.add(16 as usize);

    if iswdigit(*Path) == 0 {
        DPRINT1!(b"'%S' : expected a number! Not a regular hard disk device.\n\0".as_ptr() as *const i8, Path);
        return (FALSE) as u8;
    }
    DiskNumber = wcstoul(Path, &mut Path as *mut PCWSTR as *mut PWSTR, 10);

    if *Path != 0 && *Path != (OBJ_NAME_PATH_SEPARATOR) as _ {
        DPRINT1!(b"'%S' : expected a path separator!\n\0".as_ptr() as *const i8, Path);
        return (FALSE) as u8;
    }

    if *Path == 0 {
        DPRINT1!(b"The path only specified a hard disk (and nothing else, like a partition...), so we stop there.\n\0".as_ptr() as *const i8);
    } else {
        if _wcsnicmp(Path, RUSTCONV_WSTR_1F163068ACB2.as_ptr(), 10) != 0 {
            DPRINT1!(b"'%S' : unexpected format!\n\0".as_ptr() as *const i8, NtPath);
        } else {
            Path = Path.add(10 as usize);

            if iswdigit(*Path) == 0 {
                DPRINT1!(b"'%S' : expected a number!\n\0".as_ptr() as *const i8, Path);
            } else {
                PartNumber = wcstoul(Path, &mut Path as *mut PCWSTR as *mut PWSTR, 10);

                if *Path != 0 && *Path != (OBJ_NAME_PATH_SEPARATOR) as _ {
                    DPRINT1!(b"'%S' : expected a path separator!\n\0".as_ptr() as *const i8, Path);
                } else {
                    *pPartNumber = PartNumber;
                }
            }
        }
    }

    *pDiskNumber = DiskNumber;

    if !PathComponent.is_null() {
        *PathComponent = Path;
    }

    return (TRUE) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn OpenAndMapFile(
    mut RootDirectory: HANDLE,
    mut PathNameToFile: PCWSTR,
    mut FileHandle: PHANDLE,
    mut FileSize: PULONG,
    mut SectionHandle: PHANDLE,
    mut BaseAddress: *mut PVOID,
    mut ReadWriteAccess: BOOLEAN,
) -> NTSTATUS {
    let mut current_block: u64 = 0;
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
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut LocalFileHandle: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(&raw mut FileName, PathNameToFile);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = RootDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as u32;
    ObjectAttributes.ObjectName = &raw mut FileName as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if !FileHandle.is_null() {
        *FileHandle = core::ptr::null_mut();
    }

    Status = NtOpenFile(
        &raw mut LocalFileHandle,
        (FILE_GENERIC_READ | (if ReadWriteAccess != 0 { FILE_GENERIC_WRITE } else { 0 })) as u32,
        &raw mut ObjectAttributes,
        &raw mut IoStatusBlock,
        FILE_SHARE_READ as u32,
        (FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE) as u32,
    );

    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) Failed to open file '%wZ' (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
            913 as i32,
            &raw mut FileName,
            Status,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                913 as i32,
            );
        }
        return Status;
    }

    if !FileSize.is_null() {
        let mut FileInfo: FILE_STANDARD_INFORMATION = FILE_STANDARD_INFORMATION {
            AllocationSize: { let mut __lit: LARGE_INTEGER = core::mem::zeroed(); __lit.__bindgen_anon_1.LowPart = 0; __lit.__bindgen_anon_1.HighPart = 0; __lit },
            EndOfFile: { let mut __lit: LARGE_INTEGER = core::mem::zeroed(); __lit.__bindgen_anon_1.LowPart = 0; __lit.__bindgen_anon_1.HighPart = 0; __lit },
            NumberOfLinks: 0,
            DeletePending: 0,
            Directory: 0,
        };
        Status = NtQueryInformationFile(
            LocalFileHandle,
            &raw mut IoStatusBlock,
            &raw mut FileInfo as *mut _ as PVOID,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
            FileStandardInformation,
        );
        if !(Status >= 0) {
            if DbgPrint(
                b"(%s:%d) NtQueryInformationFile() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                930 as i32,
                Status,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                    930 as i32,
                );
            }
            current_block = 7235355898396663828;
        } else {
            if FileInfo.EndOfFile.__bindgen_anon_1.HighPart != 0 {
                if DbgPrint(
                    b"(%s:%d) WARNING!! The file '%wZ' is too large!\n\0".as_ptr() as *const i8,
                    b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                    933 as i32,
                    &raw mut FileName,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                        933 as i32,
                    );
                }
            }
            *FileSize = FileInfo.EndOfFile.__bindgen_anon_1.LowPart as u32;
            if DbgPrint(
                b"(%s:%d) File size: %lu\n\0".as_ptr() as *const i8,
                b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                936 as i32,
                *FileSize,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                    936 as i32,
                );
            }
            current_block = 11913429853522160501;
        }
    } else {
        current_block = 11913429853522160501;
    }

    match current_block {
        11913429853522160501 => {
            Status = MapFile(
                LocalFileHandle,
                SectionHandle,
                BaseAddress,
                ReadWriteAccess,
            );
            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) Failed to map file '%wZ' (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                    946 as i32,
                    &raw mut FileName,
                    Status,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"base/setup/lib/utils/filesup.c\0".as_ptr() as *const i8,
                        946 as i32,
                    );
                }
            }
        }
        _ => {}
    }

    if Status >= 0 && !FileHandle.is_null() {
        *FileHandle = LocalFileHandle;
    } else {
        NtClose(LocalFileHandle);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn MapFile(
    FileHandle: HANDLE,
    SectionHandle: PHANDLE,
    BaseAddress: *mut PVOID,
    ReadWriteAccess: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SectionPageProtection: ULONG;
    let mut ViewSize: SIZE_T;
    let mut ViewBase: PVOID;

    SectionPageProtection = if ReadWriteAccess != 0 {
        PAGE_READWRITE
    } else {
        PAGE_READONLY
    };

    *SectionHandle = core::ptr::null_mut();
    Status = NtCreateSection(
        SectionHandle,
        STANDARD_RIGHTS_REQUIRED | SECTION_QUERY | SECTION_MAP_READ
            | if ReadWriteAccess != 0 {
                SECTION_MAP_WRITE
            } else {
                0
            },
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SectionPageProtection,
        SEC_COMMIT,
        FileHandle,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"Failed to create a memory section for file 0x%p (Status 0x%08lx)\n\0".as_ptr()
                as *const i8,
            FileHandle,
            Status,
        );
        return Status;
    }

    ViewSize = 0;
    ViewBase = core::ptr::null_mut();
    Status = NtMapViewOfSection(
        *SectionHandle,
        NtCurrentProcess(),
        &mut ViewBase,
        0,
        0,
        core::ptr::null_mut(),
        &mut ViewSize,
        ViewShare,
        0,
        SectionPageProtection,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"Failed to map a view for file 0x%p (Status 0x%08lx)\n\0".as_ptr()
                as *const i8,
            FileHandle,
            Status,
        );
        NtClose(*SectionHandle);
        *SectionHandle = core::ptr::null_mut();
        return Status;
    }

    *BaseAddress = ViewBase;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn UnMapFile(
    SectionHandle: HANDLE,
    BaseAddress: PVOID,
) -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut Success: BOOLEAN = (TRUE) as u8;

    Status = NtUnmapViewOfSection(NtCurrentProcess(), BaseAddress);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtUnmapViewOfSection(0x%p) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            BaseAddress,
            Status,
        );
        Success = (FALSE) as u8;
    }
    Status = NtClose(SectionHandle);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtClose(0x%p) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            SectionHandle,
            Status,
        );
        Success = (FALSE) as u8;
    }

    return Success;
}

unsafe extern "C" fn GetWin32DriveTypeOfDriveNumber(mut DriveNumber: USHORT) -> UCHAR {
    let mut DeviceMap: PROCESS_DEVICEMAP_INFORMATION = { let mut __lit: PROCESS_DEVICEMAP_INFORMATION = core::mem::zeroed(); __lit.__bindgen_anon_1.Query = core::mem::zeroed(); __lit };
    let mut Status: NTSTATUS = 0;
    Status = NtQueryInformationProcess(
        NtCurrentProcess(),
        ProcessDeviceMap,
        &mut DeviceMap.__bindgen_anon_1.Query as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of_val(&DeviceMap.__bindgen_anon_1.Query) as u32,
        core::ptr::null_mut(),
    );
    if Status >= 0
        && (DeviceMap.__bindgen_anon_1.Query.DriveMap & ((1 as u32) << DriveNumber as u32))
            != 0
    {
        let mut Type: UCHAR = DeviceMap.__bindgen_anon_1.Query.DriveType[DriveNumber as usize];
        if ((Type as i32)) as u32 <= DRIVE_RAMDISK {
            return Type;
        }
    }
    return (DRIVE_UNKNOWN) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn GetNtDevicePathOfDriveNumber(
    mut DriveNumber: USHORT,
    mut DevicePath: PUNICODE_STRING,
) -> UCHAR {
    let mut szDosDevPath: [WCHAR; 7] = [
        '\\' as WCHAR,
        '?' as WCHAR,
        '?' as WCHAR,
        '\\' as WCHAR,
        ('A' as i32 + DriveNumber as i32) as WCHAR,
        ':' as WCHAR,
        UNICODE_NULL as WCHAR,
    ];
    let mut Type: UCHAR = 0;
    // `= NULL` in the C: a bare `fn` cannot be null, and zeroing one panicked on entry.
    let mut pfnQueryDosDeviceW: Option<unsafe extern "system" fn(PCWSTR, PWSTR, ULONG) -> ULONG> =
        None;
    let mut hKernel32: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut DeviceHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut String: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    Type = GetWin32DriveTypeOfDriveNumber(DriveNumber);
    if (Type as i32) <= DRIVE_NO_ROOT_DIR as i32 {
        return Type;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(String),
        RUSTCONV_WSTR_F13BC6011403.as_ptr(),
    );

    Status = LdrGetDllHandle(
        core::ptr::null_mut::<WCHAR>(),
        core::ptr::null_mut::<ULONG>(),
        core::ptr::addr_of_mut!(String),
        core::ptr::addr_of_mut!(hKernel32),
    );

    if NT_SUCCESS!(Status) != 0 {
        let mut AnsiString: ANSI_STRING = ANSI_STRING {
            Length: (b"QueryDosDeviceW\0".len() - 1) as USHORT,
            MaximumLength: b"QueryDosDeviceW\0".len() as USHORT,
            Buffer: b"QueryDosDeviceW\0".as_ptr() as *mut i8,
        };
        Status = LdrGetProcedureAddress(
            hKernel32 as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(AnsiString),
            0 as ULONG,
            core::ptr::addr_of_mut!(pfnQueryDosDeviceW) as *mut *mut core::ffi::c_void,
        );
    }

    if NT_SUCCESS!(Status) != 0 {
        let cch: ULONG = pfnQueryDosDeviceW.unwrap_unchecked()(
            (core::ptr::addr_of_mut!(szDosDevPath) as *mut WCHAR)
                .add(4 as usize) as PCWSTR,
            (*DevicePath).Buffer,
            ((*DevicePath).MaximumLength as usize / core::mem::size_of::<WCHAR>()) as ULONG,
        );
        if cch != 0 {
            (*DevicePath).Length =
                ((cch.wrapping_sub(1 as ULONG) as usize)
                    * core::mem::size_of::<WCHAR>()) as USHORT;
            return Type;
        }
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(String),
        core::ptr::addr_of!(szDosDevPath) as *const WCHAR,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(String);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenSymbolicLinkObject(
        core::ptr::addr_of_mut!(DeviceHandle),
        SYMBOLIC_LINK_QUERY as ULONG,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );

    if NT_SUCCESS!(Status) != 0 {
        let mut ReturnLength: ULONG = 0;
        Status = NtQuerySymbolicLinkObject(
            DeviceHandle,
            DevicePath,
            core::ptr::addr_of_mut!(ReturnLength),
        );
        NtClose(DeviceHandle);
    }

    return if NT_SUCCESS!(Status) != 0 {
        Type
    } else {
        DRIVE_UNKNOWN as UCHAR
    };
}

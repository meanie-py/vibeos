use crate::setuplib::filesup::CombinePaths;
use crate::setuplib::filesup::SetupMoveFile;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0C8C471E3DBF: [u16; 16] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0063,
    0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_11BF324B77A7: [u16; 6] = [0x0025u16, 0x0073, 0x002E, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_391152FBD200: [u16; 4] = [0x006Fu16, 0x006C, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_A24FD9ADB425: [u16; 4] = [0x0062u16, 0x0072, 0x006B, 0x0000];
pub static RUSTCONV_WSTR_CFB64F0A0EE0: [u16; 36] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0055, 0x0053,
    0x0065, 0x0074, 0x0075, 0x0070, 0x005F, 0x0056, 0x0065, 0x0072, 0x0069, 0x0066,
    0x0079, 0x0048, 0x0069, 0x0076, 0x0065, 0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(format: *const core::ffi::c_char, ...) -> core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_0BA09314F12F: [u16; 18] = [
    0x0053u16, 0x0079, 0x006D, 0x0062, 0x006F, 0x006C, 0x0069, 0x0063, 0x004C, 0x0069,
    0x006E, 0x006B, 0x0056, 0x0061, 0x006C, 0x0075, 0x0065, 0x0000,
];

static mut SymbolicLinkValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 18]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 18]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_0BA09314F12F.as_ptr() as PWSTR,
    };

#[no_mangle]
pub unsafe extern "C" fn CreateNestedKey(
    KeyHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    CreateOptions: ULONG,
) -> NTSTATUS {
    let mut LocalObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut LocalKeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Disposition: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    let mut FullNameLength: USHORT = 0;
    let mut Ptr: PWCHAR = core::ptr::null_mut();
    let mut LocalKeyHandle: HANDLE = core::ptr::null_mut();

    Status = NtCreateKey(
        KeyHandle,
        KEY_ALL_ACCESS,
        ObjectAttributes,
        0,
        core::ptr::null_mut(),
        CreateOptions,
        core::ptr::addr_of_mut!(Disposition),
    );
    DPRINT!(
        b"NtCreateKey(%wZ) called (Status %lx)\n\0".as_ptr() as *const i8,
        (*ObjectAttributes).ObjectName,
        Status,
    );
    if Status != STATUS_OBJECT_NAME_NOT_FOUND {
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"CreateNestedKey: NtCreateKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8,
                (*ObjectAttributes).ObjectName,
                Status,
            );
        }
        return Status;
    }

    core::ptr::copy_nonoverlapping(
        ObjectAttributes as *const u8,
        core::ptr::addr_of_mut!(LocalObjectAttributes) as *mut u8,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
    );
    RtlCreateUnicodeString(
        core::ptr::addr_of_mut!(LocalKeyName),
        (*(*ObjectAttributes).ObjectName).Buffer as *const u16,
    );
    LocalObjectAttributes.ObjectName = core::ptr::addr_of_mut!(LocalKeyName);
    FullNameLength = LocalKeyName.Length;

    while Status == STATUS_OBJECT_NAME_NOT_FOUND {
        Ptr = wcsrchr(
            LocalKeyName.Buffer as *const u16,
            '\\' as u16,
        );
        if Ptr.is_null() || Ptr == LocalKeyName.Buffer {
            Status = STATUS_UNSUCCESSFUL;
            break;
        }
        *Ptr = 0 as u16;
        LocalKeyName.Length = (Ptr.offset_from(LocalKeyName.Buffer) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()) as u16;

        Status = NtCreateKey(
            core::ptr::addr_of_mut!(LocalKeyHandle),
            KEY_CREATE_SUB_KEY,
            core::ptr::addr_of_mut!(LocalObjectAttributes),
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            core::ptr::addr_of_mut!(Disposition),
        );
        DPRINT!(
            b"NtCreateKey(%wZ) called (Status %lx)\n\0".as_ptr() as *const i8,
            core::ptr::addr_of_mut!(LocalKeyName),
            Status,
        );
        if ((NT_SUCCESS!(Status)) == 0) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            DPRINT1!(
                b"CreateNestedKey: NtCreateKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8,
                LocalObjectAttributes.ObjectName,
                Status,
            );
        }
    }

    if ((NT_SUCCESS!(Status)) == 0) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(LocalKeyName));
        return Status;
    }

    loop {
        if LocalKeyName.Length == FullNameLength {
            Status = STATUS_SUCCESS;
            *KeyHandle = LocalKeyHandle;
            break;
        }
        NtClose(LocalKeyHandle);

        *LocalKeyName.Buffer.add(
            LocalKeyName.Length as usize / core::mem::size_of::<u16>()
        ) = 0x005Cu16;
        LocalKeyName.Length = ((wcslen(
            LocalKeyName.Buffer as *const u16
        ) as u16) as usize)
            .wrapping_mul(core::mem::size_of::<u16>()) as u16;

        Status = NtCreateKey(
            core::ptr::addr_of_mut!(LocalKeyHandle),
            KEY_ALL_ACCESS,
            core::ptr::addr_of_mut!(LocalObjectAttributes),
            0,
            core::ptr::null_mut(),
            CreateOptions,
            core::ptr::addr_of_mut!(Disposition),
        );
        DPRINT!(
            b"NtCreateKey(%wZ) called (Status %lx)\n\0".as_ptr() as *const i8,
            core::ptr::addr_of_mut!(LocalKeyName),
            Status,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"CreateNestedKey: NtCreateKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8,
                LocalObjectAttributes.ObjectName,
                Status,
            );
            break;
        }
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(LocalKeyName));

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn CreateRegistryFile(
    NtSystemRoot: PUNICODE_STRING,
    RegistryKey: PCWSTR,
    IsHiveNew: BOOLEAN,
    ProtoKeyHandle: HANDLE,
) -> NTSTATUS {
    static mut Extensions: [PCWSTR; 2] = [
        RUSTCONV_WSTR_391152FBD200.as_ptr(),
        RUSTCONV_WSTR_A24FD9ADB425.as_ptr(),
    ];

    let mut Status: NTSTATUS = 0;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Extension: PCWSTR = core::ptr::null();
    let mut PathBuffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut PathBuffer2: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    CombinePaths(
        PathBuffer.as_mut_ptr(),
        (ARRAYSIZE!(PathBuffer)) as _,
        3,
        (*NtSystemRoot).Buffer as PCWSTR,
        RUSTCONV_WSTR_0C8C471E3DBF.as_ptr(),
        RegistryKey,
    );

    Extension = unsafe {
        Extensions[if IsHiveNew != 0 {
            0
        } else {
            1
        }]
    };

    if DoesFileExist!(core::ptr::null_mut(), PathBuffer.as_mut_ptr() as PCWSTR) != 0 {
        DPRINT1!(
            b"Registry hive '%S' already exists, rename it\n\0".as_ptr() as *const i8,
            PathBuffer.as_mut_ptr() as PCWSTR,
        );

        RtlStringCchPrintfW!(
            PathBuffer2.as_mut_ptr(),
            ARRAYSIZE!(PathBuffer2),
            RUSTCONV_WSTR_11BF324B77A7.as_ptr(),
            PathBuffer.as_mut_ptr() as PCWSTR,
            Extension,
        );

        Status = SetupMoveFile(
            PathBuffer.as_mut_ptr() as PCWSTR,
            PathBuffer2.as_mut_ptr() as PCWSTR,
            MOVEFILE_REPLACE_EXISTING,
        );
    }

    RtlInitUnicodeString(
        &raw mut FileName,
        PathBuffer.as_mut_ptr() as PCWSTR,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &raw mut FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        &raw mut FileHandle,
        FILE_GENERIC_WRITE,
        &raw mut ObjectAttributes,
        &raw mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        0,
        FILE_OVERWRITE_IF,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
        core::ptr::null_mut(),
        0,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtCreateFile(%wZ) failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
            &raw mut FileName,
            Status,
        );
        return Status;
    }

    Status = NtSaveKeyEx(ProtoKeyHandle, FileHandle, REG_LATEST_FORMAT);

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtSaveKeyEx(%wZ) failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
            &raw mut FileName,
            Status,
        );
    }

    NtClose(FileHandle);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CreateSymLinkKey(
    RootKey: HANDLE,
    LinkKeyName: PCWSTR,
    TargetKeyName: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LinkKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Disposition: ULONG = 0;

    RtlInitUnicodeString(&raw mut KeyName, LinkKeyName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RootKey;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateKey(
        &raw mut LinkKeyHandle,
        (KEY_SET_VALUE as ACCESS_MASK | KEY_CREATE_LINK as ACCESS_MASK),
        &raw mut ObjectAttributes,
        0 as ULONG,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        (REG_OPTION_VOLATILE as ULONG | REG_OPTION_CREATE_LINK as ULONG),
        &raw mut Disposition,
    );

    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) CreateSymLinkKey: couldn't create '%S', Status = 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char,
            b"base/setup/lib/utils/regutil.c\0".as_ptr() as *const core::ffi::c_char,
            284 as core::ffi::c_int,
            LinkKeyName,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                b"base/setup/lib/utils/regutil.c\0".as_ptr() as *const core::ffi::c_char,
                284 as core::ffi::c_int,
            );
        }
        return Status;
    }

    if Disposition != REG_CREATED_NEW_KEY as ULONG {
        if DbgPrint(
            b"(%s:%d) CreateSymLinkKey: %S already exists!\n\0".as_ptr() as *const core::ffi::c_char,
            b"base/setup/lib/utils/regutil.c\0".as_ptr() as *const core::ffi::c_char,
            291 as core::ffi::c_int,
            LinkKeyName,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                b"base/setup/lib/utils/regutil.c\0".as_ptr() as *const core::ffi::c_char,
                291 as core::ffi::c_int,
            );
        }
        NtClose(LinkKeyHandle);
        return STATUS_OBJECT_NAME_EXISTS;
    }

    RtlInitUnicodeString(&raw mut KeyName, TargetKeyName);
    Status = NtSetValueKey(
        LinkKeyHandle,
        &raw mut SymbolicLinkValueName,
        0 as ULONG,
        REG_LINK as ULONG,
        KeyName.Buffer as *mut core::ffi::c_void,
        KeyName.Length as ULONG,
    );

    NtClose(LinkKeyHandle);

    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) CreateSymLinkKey: couldn't create symbolic link '%S' for '%S', Status = 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char,
            b"base/setup/lib/utils/regutil.c\0".as_ptr() as *const core::ffi::c_char,
            311 as core::ffi::c_int,
            LinkKeyName,
            TargetKeyName,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                b"base/setup/lib/utils/regutil.c\0".as_ptr() as *const core::ffi::c_char,
                311 as core::ffi::c_int,
            );
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn DeleteSymLinkKey(
    RootKey: HANDLE,
    LinkKeyName: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut LinkKeyHandle: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyName), LinkKeyName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = RootKey;
    ObjectAttributes.Attributes =
        (OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_OPENLINK) as u32;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(
        core::ptr::addr_of_mut!(LinkKeyHandle),
        (DELETE | KEY_SET_VALUE | KEY_CREATE_LINK) as u32,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtOpenKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8,
            core::ptr::addr_of_mut!(KeyName),
            Status,
        );
        return Status;
    }

    Status = NtDeleteValueKey(
        LinkKeyHandle,
        core::ptr::addr_of_mut!(SymbolicLinkValueName),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtDeleteValueKey(%wZ) failed (Status %lx)\n\0".as_ptr() as *const i8,
            core::ptr::addr_of_mut!(KeyName),
            Status,
        );
        NtClose(LinkKeyHandle);
        return Status;
    }

    Status = NtDeleteKey(LinkKeyHandle);
    NtClose(LinkKeyHandle);

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"DeleteSymLinkKey: couldn't delete symbolic link '%S', Status = 0x%08lx\n\0".as_ptr()
                as *const i8,
            LinkKeyName,
            Status,
        );
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn ConnectRegistry(
    RootKey: HANDLE,
    RegMountPoint: PCWSTR,
    NtSystemRoot: PUNICODE_STRING,
    RegistryKey: PCWSTR,
) -> NTSTATUS {
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut FileName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut KeyObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut FileObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut PathBuffer: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];

    RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyName), RegMountPoint);

    KeyObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    KeyObjectAttributes.RootDirectory = RootKey;
    KeyObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    KeyObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    KeyObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    KeyObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    CombinePaths(
        PathBuffer.as_mut_ptr(),
        (ARRAYSIZE!(PathBuffer)) as _,
        3,
        (*NtSystemRoot).Buffer,
        RUSTCONV_WSTR_0C8C471E3DBF.as_ptr(),
        RegistryKey,
    );

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(FileName),
        PathBuffer.as_ptr(),
    );

    FileObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    FileObjectAttributes.RootDirectory = core::ptr::null_mut();
    FileObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    FileObjectAttributes.ObjectName = core::ptr::addr_of_mut!(FileName);
    FileObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    FileObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    return NtLoadKey(
        core::ptr::addr_of_mut!(KeyObjectAttributes),
        core::ptr::addr_of_mut!(FileObjectAttributes),
    );
}
#[no_mangle]
pub unsafe extern "C" fn DisconnectRegistry(
    RootKey: HANDLE,
    RegMountPoint: PCWSTR,
    Flags: ULONG,
) -> NTSTATUS {
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
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
    RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyName), RegMountPoint);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RootKey;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName) as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    return NtUnloadKey2(core::ptr::addr_of_mut!(ObjectAttributes), Flags);
}
#[no_mangle]
pub unsafe extern "C" fn VerifyRegistryHive(
    NtSystemRoot: PUNICODE_STRING,
    RegistryKey: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    /* Try to mount the specified registry hive */
    Status = ConnectRegistry(
        core::ptr::null_mut(),
        RUSTCONV_WSTR_CFB64F0A0EE0.as_ptr(),
        NtSystemRoot,
        RegistryKey
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"ConnectRegistry(%S) failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
            RegistryKey,
            Status,
        );
    }

    DPRINT1!(
        b"VerifyRegistryHive: ConnectRegistry(%S) returns Status 0x%08lx\n\0".as_ptr() as *const i8,
        RegistryKey,
        Status,
    );

    if Status == STATUS_REGISTRY_HIVE_RECOVERED {
        DPRINT1!(
            b"VerifyRegistryHive: Registry hive %S was recovered but some data may be lost (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            RegistryKey,
            Status,
        );
    }

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"VerifyRegistryHive: Registry hive %S is corrupted (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            RegistryKey,
            Status,
        );
        return Status;
    }

    if Status == STATUS_REGISTRY_RECOVERED {
        DPRINT1!(
            b"VerifyRegistryHive: Registry hive %S succeeded recovered (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            RegistryKey,
            Status,
        );
    }

    /* Unmount the hive */
    Status = DisconnectRegistry(
        core::ptr::null_mut(),
        RUSTCONV_WSTR_CFB64F0A0EE0.as_ptr(),
        0,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"DisconnectRegistry(%S) failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
            RegistryKey,
            Status,
        );
    }

    return Status;
}

// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1F41958ED29A: [u16; 8] = [0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0025, 0x0063, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_42DDBCFAE9C2: [u16; 16] = [
    0x0053u16, 0x0065, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x004D, 0x0061,
    0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_43C66C260828: [u16; 2] = [0x000Du16, 0x0000];
pub static RUSTCONV_WSTR_8CB72516B703: [u16; 23] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x004B, 0x0065,
    0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0043, 0x006C, 0x0061, 0x0073,
    0x0073, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_A8C8153F947C: [u16; 15] = [
    0x0041u16, 0x0075, 0x0074, 0x006F, 0x0043, 0x0068, 0x006B, 0x0054, 0x0069, 0x006D,
    0x0065, 0x004F, 0x0075, 0x0074, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut FileSystems: [FILESYSTEM_CHKDSK; 4] = unsafe {
    [
        _FILESYSTEM_CHKDSK {
            Name: ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(
                *b"F\0A\0T\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            ChkdskFunc: Some(
                VfatChkdsk,
            ),
        },
        _FILESYSTEM_CHKDSK {
            Name: ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(
                *b"F\0A\0T\x003\x002\0\0\0\0\0\0\0\0\0\0\0",
            ),
            ChkdskFunc: Some(
                VfatChkdsk,
            ),
        },
        _FILESYSTEM_CHKDSK {
            Name: ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(
                *b"F\0A\0T\0X\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            ChkdskFunc: Some(
                VfatxChkdsk,
            ),
        },
        _FILESYSTEM_CHKDSK {
            Name: ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(
                *b"C\0D\0F\0S\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            ChkdskFunc: Some(
                CdfsChkdsk,
            ),
        },
    ]
};

#[no_mangle]
pub static mut KeyboardHandle: HANDLE = NULL;

#[repr(C)]
pub struct _FILESYSTEM_CHKDSK {
    pub Name: [WCHAR; 10],
    pub ChkdskFunc: PULIB_CHKDSK,
}
pub type FILESYSTEM_CHKDSK = _FILESYSTEM_CHKDSK;
pub type PFILESYSTEM_CHKDSK = *mut FILESYSTEM_CHKDSK;

#[no_mangle]
unsafe extern "C" fn PrintString_v(fmt: *mut ::core::ffi::c_char, ap_list: *mut ::core::ffi::c_void) -> INT {
    let mut Len: INT = 0;
    let mut buffer: [::core::ffi::c_char; 512] = [0; 512];
    let mut ap: va_list = *(ap_list as *mut va_list);
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();


    Len = vsprintf((buffer.as_mut_ptr()).cast(), fmt.cast(), (ap).cast()) as INT;


    RtlInitAnsiString(
        core::ptr::addr_of_mut!(AnsiString),
        buffer.as_mut_ptr() as *const ::core::ffi::c_char,
    );
    RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(UnicodeString),
        core::ptr::addr_of_mut!(AnsiString),
        TRUE as BOOLEAN,
    );
    NtDisplayString(core::ptr::addr_of_mut!(UnicodeString));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(UnicodeString));

    return Len;
}



unsafe extern "C" fn EraseLine(mut LineLength: INT) {
    let mut Len: INT;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut buffer: [WCHAR; 512] = [0; 512];

    if LineLength <= 0 {
        return;
    }

    /* Go to the beginning of the line */
    RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeString), RUSTCONV_WSTR_43C66C260828.as_ptr());
    NtDisplayString(core::ptr::addr_of_mut!(UnicodeString));

    /* Fill the buffer chunk with spaces */
    Len = min!(LineLength, (ARRAYSIZE!(buffer)) as i32) as INT;
    while Len > 0 {
        Len -= 1;
        buffer[Len as usize] = 0x0020u16;
    }

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(UnicodeString),
        buffer.as_mut_ptr(),
        core::mem::size_of::<[WCHAR; 512]>() as USHORT,
    );
    while LineLength > 0 {
        /* Display the buffer */
        Len = min!(LineLength, (ARRAYSIZE!(buffer)) as i32) as INT;
        LineLength -= Len;
        UnicodeString.Length = (Len as usize * core::mem::size_of::<WCHAR>()) as USHORT;
        NtDisplayString(core::ptr::addr_of_mut!(UnicodeString));
    }

    /* Go to the beginning of the line */
    RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeString), RUSTCONV_WSTR_43C66C260828.as_ptr());
    NtDisplayString(core::ptr::addr_of_mut!(UnicodeString));
}

unsafe extern "C" fn OpenKeyboard(KeyboardHandle_local: PHANDLE) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut KeyboardName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_8CB72516B703.as_ptr());

    /* Just open the class driver */
    // InitializeObjectAttributes(&ObjectAttributes, &KeyboardName, COMMENT_TOKEN_ECHO____, NULL, NULL);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyboardName);
    ObjectAttributes.Attributes = (0) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    return NtOpenFile(
        KeyboardHandle_local,
        (FILE_ALL_ACCESS),
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        (FILE_OPEN),
        (0),
    );
}

unsafe extern "C" fn WaitForKeyboard(KeyboardHandle_local: HANDLE, mut TimeOut: LONG) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut InputData: KEYBOARD_INPUT_DATA = core::mem::zeroed();
    let mut Len: INT = 0;

    /* Skip the wait if there is no timeout */
    if TimeOut <= 0 {
        return STATUS_TIMEOUT;
    }

    /* Attempt to read a down key-press from the keyboard */
    loop {
        Offset.QuadPart = 0 as LONGLONG;
        Status = NtReadFile(
            KeyboardHandle_local,
            NULL,
            None,
            NULL,
            core::ptr::addr_of_mut!(IoStatusBlock),
            core::ptr::addr_of_mut!(InputData) as PVOID,
            core::mem::size_of::<KEYBOARD_INPUT_DATA>() as ULONG,
            core::ptr::addr_of_mut!(Offset),
            core::ptr::null_mut::<ULONG>(),
        );
        if !NT_SUCCESS!(Status) {
            /* Something failed, bail out */
            return Status;
        }
        if Status != STATUS_PENDING
            && (InputData.Flags as ::core::ffi::c_int & (KEY_BREAK as i32)) == 0
        {
            /* We have a down key-press, return */
            return Status;
        }
        if !(Status != STATUS_PENDING) {
            break;
        }
    }

    /* Perform the countdown of TimeOut seconds */
    Status = STATUS_TIMEOUT;
    while TimeOut > 0 {
        /*
         * Display the countdown string.
         * Note that we only do a carriage return to go back to the
         * beginning of the line to possibly erase it. Note also that
         * we display a trailing space to erase any last character
         * when the string length decreases.
         */
        Len = PrintString(
            b"Press any key within %d second(s) to cancel and resume startup. \r\0".as_ptr()
                as *mut i8,
            TimeOut,
        );

        /* Decrease the countdown */
        TimeOut -= 1;

        /* Wait one second for a key press */
        Timeout.QuadPart = -(10000000 as LONGLONG);
        Status = NtWaitForSingleObject(
            KeyboardHandle_local,
            (FALSE) as BOOLEAN,
            core::ptr::addr_of_mut!(Timeout),
        );
        if Status != STATUS_TIMEOUT {
            break;
        }
    }

    /* Erase the countdown string */
    EraseLine(Len);

    if Status == STATUS_TIMEOUT {
        /* The user did not press any key, cancel the read */
        NtCancelIoFile(KeyboardHandle_local, core::ptr::addr_of_mut!(IoStatusBlock));
    } else {
        /* Otherwise, return some status */
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    return Status;
}

unsafe extern "C" fn GetFileSystem(
    VolumePathU: PUNICODE_STRING,
    FileSystemName: PWSTR,
    FileSystemNameSize: SIZE_T,
) -> NTSTATUS {
    const BUFFER_SIZE: usize = core::mem::size_of::<FILE_FS_ATTRIBUTE_INFORMATION>()
        + MAX_PATH as usize * core::mem::size_of::<WCHAR>();

    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileFsAttribute: PFILE_FS_ATTRIBUTE_INFORMATION;
    let mut Buffer: [UCHAR; BUFFER_SIZE] = [0; BUFFER_SIZE];

    FileFsAttribute = core::ptr::addr_of_mut!(Buffer) as *mut UCHAR as PFILE_FS_ATTRIBUTE_INFORMATION;

    // InitializeObjectAttributes(&ObjectAttributes, VolumePathU, OBJ_CASE_INSENSITIVE, NULL, NULL)
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = VolumePathU;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        core::ptr::addr_of_mut!(FileHandle),
        FILE_GENERIC_READ,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN,
        0,
        core::ptr::null_mut(),
        0,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            "Could not open volume '%wZ' to obtain its file system, Status 0x%08lx\n",
            VolumePathU,
            Status
        );
        return Status;
    }

    Status = NtQueryVolumeInformationFile(
        FileHandle,
        core::ptr::addr_of_mut!(IoStatusBlock),
        (FileFsAttribute).cast(),
        BUFFER_SIZE as ULONG,
        FileFsAttributeInformation,
    );
    NtClose(FileHandle);

    if !NT_SUCCESS!(Status) {
        DPRINT1!("NtQueryVolumeInformationFile() failed, Status 0x%08lx\n", Status);
        return Status;
    }

    if FileSystemNameSize
        >= ((*FileFsAttribute).FileSystemNameLength as SIZE_T)
            + core::mem::size_of::<WCHAR>() as SIZE_T
    {
        RtlCopyMemory!(
            FileSystemName as PVOID,
            core::ptr::addr_of!((*FileFsAttribute).FileSystemName) as PVOID,
            (*FileFsAttribute).FileSystemNameLength as SIZE_T,
        );
        *FileSystemName.add(
            ((*FileFsAttribute).FileSystemNameLength as usize)
                / core::mem::size_of::<WCHAR>(),
        ) = UNICODE_NULL;
    } else {
        return STATUS_BUFFER_TOO_SMALL;
    }

    return STATUS_SUCCESS;
}
unsafe extern "system" fn ChkdskCallback(
    mut Command: CALLBACKCOMMAND,
    mut Modifier: ULONG,
    mut Argument: PVOID,
) -> BOOLEAN {
    let mut Percent: PDWORD = ::core::ptr::null_mut::<DWORD>();
    let mut Status: PBOOLEAN = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut Output: PTEXTOUTPUT = ::core::ptr::null_mut();
    match Command as ::core::ffi::c_uint {
        UNKNOWN2 => {
            DPRINT!("UNKNOWN2\n");
        }
        UNKNOWN3 => {
            DPRINT!("UNKNOWN3\n");
        }
        UNKNOWN4 => {
            DPRINT!("UNKNOWN4\n");
        }
        UNKNOWN5 => {
            DPRINT!("UNKNOWN5\n");
        }
        UNKNOWN9 => {
            DPRINT!("UNKNOWN9\n");
        }
        UNKNOWNA => {
            DPRINT!("UNKNOWNA\n");
        }
        UNKNOWNC => {
            DPRINT!("UNKNOWNC\n");
        }
        UNKNOWND => {
            DPRINT!("UNKNOWND\n");
        }
        INSUFFICIENTRIGHTS => {
            DPRINT!("INSUFFICIENTRIGHTS\n");
        }
        FSNOTSUPPORTED => {
            DPRINT!("FSNOTSUPPORTED\n");
        }
        VOLUMEINUSE => {
            DPRINT!("VOLUMEINUSE\n");
        }
        STRUCTUREPROGRESS => {
            DPRINT!("STRUCTUREPROGRESS\n");
        }
        DONEWITHSTRUCTURE => {
            DPRINT!("DONEWITHSTRUCTURE\n");
        }
        CLUSTERSIZETOOSMALL => {
            DPRINT!("CLUSTERSIZETOOSMALL\n");
        }
        PROGRESS => {
            Percent = Argument as PDWORD;
            PrintString(
                b"%d percent completed.\r\0" as *const u8 as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                *Percent,
            );
        }
        OUTPUT => {
            Output = Argument as PTEXTOUTPUT;
            PrintString(
                b"%s\0" as *const u8 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                (*Output).Output,
            );
        }
        DONE => {
            Status = Argument as PBOOLEAN;
            if *Status as ::core::ffi::c_int == (FALSE as i32) {
                PrintString(
                    b"The file system check was unable to complete successfully.\r\n\r\n\0"
                        as *const u8 as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
                // Error = TRUE;
            }
        }
        _ => {}
    }
    return TRUE as BOOLEAN;
}

unsafe extern "C" fn CheckVolume(
    VolumePath: PCWSTR,
    TimeOut: LONG,
    CheckOnlyIfDirty: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Success: BOOLEAN = 0;
    let mut DisplayName: PCWSTR = core::ptr::null();
    let mut VolumePathU: _UNICODE_STRING = core::mem::zeroed();
    let mut Count: ULONG = 0;
    let mut FileSystem: [WCHAR; 128] = [0; 128];

    RtlInitUnicodeString(core::ptr::addr_of_mut!(VolumePathU), VolumePath);

    /* Get a drive string for display purposes only */
    if wcslen(VolumePath) == 6
        && *VolumePath.offset(0 as isize) == 0x005Cu16
        && *VolumePath.offset(1 as isize) == 0x003Fu16
        && *VolumePath.offset(2 as isize) == 0x003Fu16
        && *VolumePath.offset(3 as isize) == 0x005Cu16
        && *VolumePath.offset(5 as isize) == 0x003Au16
    {
        /* DOS drive */
        DisplayName = VolumePath.offset(4 as isize);
    } else {
        DisplayName = VolumePath;
    }

    DPRINT1!("AUTOCHK: Checking %wZ\n", core::ptr::addr_of_mut!(VolumePathU));
    PrintString(
        b"Verifying the file system on %S\r\n\0".as_ptr() as *mut i8,
        DisplayName,
    );

    /* Get the file system */
    Status = GetFileSystem(
        core::ptr::addr_of_mut!(VolumePathU),
        core::ptr::addr_of_mut!(FileSystem) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; 128]>() as SIZE_T,
    );
    'quit: {
        if !NT_SUCCESS!(Status) {
            DPRINT1!("GetFileSystem() failed, Status 0x%08lx\n", Status);
            PrintString(
                b"  Unable to detect the file system of volume %S\r\n\0".as_ptr() as *mut i8,
                DisplayName,
            );
            break 'quit;
        }

        PrintString(
            b"  The file system type is %S.\r\n\r\n\0".as_ptr() as *mut i8,
            core::ptr::addr_of_mut!(FileSystem) as *mut WCHAR,
        );

        /* Find a suitable file system provider */
        Count = 0 as ULONG;
        while (Count as usize) < (RTL_NUMBER_OF!(FileSystems) as usize) {
            if wcscmp(
                core::ptr::addr_of_mut!(FileSystem) as *mut WCHAR,
                core::ptr::addr_of_mut!(FileSystems[Count as usize].Name) as *mut WCHAR,
            ) == 0
            {
                break;
            }
            Count = Count.wrapping_add(1);
        }
        if (Count as usize) >= (RTL_NUMBER_OF!(FileSystems) as usize) {
            DPRINT1!(
                "File system %S not supported\n",
                core::ptr::addr_of_mut!(FileSystem) as *mut WCHAR
            );
            PrintString(
                b"  Unable to verify the volume. The %S file system is not supported.\r\n\0"
                    .as_ptr() as *mut i8,
                core::ptr::addr_of_mut!(FileSystem) as *mut WCHAR,
            );
            Status = STATUS_DLL_NOT_FOUND;
            break 'quit;
        }

        /* Check whether the volume is dirty */
        Status = STATUS_SUCCESS;
        Success = FileSystems[Count as usize]
            .ChkdskFunc
            .unwrap_unchecked()(
            core::ptr::addr_of_mut!(VolumePathU),
            Some(ChkdskCallback),
            FALSE as BOOLEAN, // FixErrors
            TRUE as BOOLEAN, // Verbose
            TRUE as BOOLEAN, // CheckOnlyIfDirty
            FALSE as BOOLEAN, // ScanDrive
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Status) as PULONG,
        );

        /* Perform the check either when the volume is dirty or a check is forced */
        if Success != 0 && ((Status == STATUS_DISK_CORRUPT_ERROR) || CheckOnlyIfDirty == 0) {
            /* Let the user decide whether to repair */
            if Status == STATUS_DISK_CORRUPT_ERROR {
                PrintString(
                    b"The file system on volume %S needs to be checked for problems.\r\n\0"
                        .as_ptr() as *mut i8,
                    DisplayName,
                );
                PrintString(
                    b"You may cancel this check, but it is recommended that you continue.\r\n\0"
                        .as_ptr() as *mut i8,
                );
            } else {
                PrintString(
                    b"A volume check has been scheduled.\r\n\0".as_ptr() as *mut i8,
                );
            }

            if KeyboardHandle.is_null()
                || WaitForKeyboard(KeyboardHandle, TimeOut) == STATUS_TIMEOUT
            {
                PrintString(
                    b"The system will now check the file system.\r\n\r\n\0".as_ptr() as *mut i8,
                );
                Status = STATUS_SUCCESS;
                Success = FileSystems[Count as usize]
                    .ChkdskFunc
                    .unwrap_unchecked()(
                    core::ptr::addr_of_mut!(VolumePathU),
                    Some(
                        ChkdskCallback,
                    ),
                    TRUE as BOOLEAN, // FixErrors
                    TRUE as BOOLEAN, // Verbose
                    CheckOnlyIfDirty,
                    FALSE as BOOLEAN, // ScanDrive
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(Status) as PULONG,
                );
            } else {
                PrintString(
                    b"The file system check has been skipped.\r\n\0".as_ptr() as *mut i8,
                );
            }
        }
    }

    PrintString(b"\r\n\r\n\0".as_ptr() as *mut i8);
    return Status;
}

unsafe extern "C" fn QueryTimeout(TimeOut: PLONG) {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(QueryTable) as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>(),
    );
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
    QueryTable[0].Name = RUSTCONV_WSTR_A8C8153F947C.as_ptr();
    QueryTable[0].EntryContext = TimeOut as PVOID;

    RtlQueryRegistryValues(
        RTL_REGISTRY_CONTROL,
        RUSTCONV_WSTR_42DDBCFAE9C2.as_ptr(),
        core::ptr::addr_of_mut!(QueryTable) as PRTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    /* See: https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/autochk */
    *TimeOut = min!(max!(*TimeOut, 0), 259200);
}

#[no_mangle]
pub unsafe extern "C" fn _main(
    argc: INT,
    argv: *mut PCHAR,
    envp: *mut PCHAR,
    DebugFlag: ULONG,
) -> INT {
    let mut Status: NTSTATUS = 0;
    let mut TimeOut: LONG = 0;
    let mut i: ULONG = 0;
    let mut CheckAllVolumes: BOOLEAN = FALSE as BOOLEAN;
    let mut CheckOnlyIfDirty: BOOLEAN = TRUE as BOOLEAN;
    let mut DeviceMap: PROCESS_DEVICEMAP_INFORMATION = core::mem::zeroed();
    let mut SkipDrives: PCHAR = core::ptr::null_mut();
    let mut VolumePathA: ANSI_STRING = core::mem::zeroed();
    let mut VolumePathU: _UNICODE_STRING = core::mem::zeroed();
    let mut VolumePath: [WCHAR; 128] = [0; 128];

    /*
     * Parse the command-line: optional command switches,
     * then the NT volume name (or drive letter) to be analysed.
     * If "*" is passed this means that we check all the volumes.
     */
    if argc <= 1 {
        /* Only one parameter (the program name), bail out */
        return 1;
    }
    i = 1 as ULONG;
    while i < argc as ULONG {
        if *(*argv.offset(i as isize)).offset(0 as isize) as i32 == '/' as i32
            || *(*argv.offset(i as isize)).offset(0 as isize) as i32 == '-' as i32
        {
            DPRINT!("Parameter %d: %s\n", i, *argv.offset(i as isize));
            let mut goto_default: bool = false;
            match toupper(
                *(*argv.offset(i as isize)).offset(1 as isize) as i32,
            ) {
                75 => {
                    /* 'K': List of drive letters to skip */
                    /* Check for the separator */
                    if *(*argv.offset(i as isize)).offset(2 as isize) as i32
                        != ':' as i32
                    {
                        goto_default = true;
                    } else {
                        SkipDrives = (*argv.offset(i as isize))
                            .offset(3 as isize);
                    }
                }
                82 | 80 => {
                    /* 'R': Repair the errors, implies /P ; 'P': Check even if not dirty */
                    if *(*argv.offset(i as isize)).offset(2 as isize) as i32
                        != ANSI_NULL as i32
                    {
                        goto_default = true;
                    } else {
                        CheckOnlyIfDirty = FALSE as BOOLEAN;
                    }
                }
                _ => {
                    goto_default = true;
                }
            }
            if goto_default {
                DPRINT1!("Unknown switch %d: %s\n", i, *argv.offset(i as isize));
            }
        } else {
            DPRINT!(
                "Parameter %d - Volume specification: %s\n",
                i,
                *argv.offset(i as isize)
            );
            if strcmp(
                *argv.offset(i as isize) as *const i8,
                b"*\0".as_ptr() as *const i8,
            ) == 0
            {
                CheckAllVolumes = TRUE as BOOLEAN;
            } else {
                RtlInitEmptyUnicodeString(
                    core::ptr::addr_of_mut!(VolumePathU),
                    core::ptr::addr_of_mut!(VolumePath) as *mut WCHAR as PWSTR,
                    (128 * core::mem::size_of::<WCHAR>()) as USHORT,
                );
                RtlInitAnsiString(
                    core::ptr::addr_of_mut!(VolumePathA),
                    *argv.offset(i as isize) as *const i8,
                );
                Status = RtlAnsiStringToUnicodeString(
                    core::ptr::addr_of_mut!(VolumePathU),
                    core::ptr::addr_of_mut!(VolumePathA),
                    FALSE as BOOLEAN,
                );
            }

            /* Stop the parsing now */
            break;
        }
        i = i.wrapping_add(1);
    }

    /*
     * FIXME: We should probably use here the mount manager to be
     * able to check volumes which don't have a drive letter.
     */
    Status = NtQueryInformationProcess(
        NtCurrentProcess(),
        ProcessDeviceMap,
        core::ptr::addr_of_mut!(DeviceMap.__bindgen_anon_1.Query) as PVOID,
        core::mem::size_of_val(&DeviceMap.__bindgen_anon_1.Query) as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("NtQueryInformationProcess() failed, Status 0x%08lx\n", Status);
        return 1;
    }

    /* Filter out the skipped drives from the map */
    if !SkipDrives.is_null() && *SkipDrives != 0 {
        DPRINT1!("Skipping drives:");
        while *SkipDrives != 0 {
            DbgPrint(b" %c:\0".as_ptr() as *const i8, *SkipDrives as i32);
            /* Retrieve the index and filter the drive out */
            i = (toupper(*SkipDrives as i32) - 'A' as i32) as ULONG;
            if (0 as ULONG) <= i && i <= ('Z' as i32 - 'A' as i32) as ULONG {
                DeviceMap.__bindgen_anon_1.Query.DriveMap &= !((1 as ULONG) << i);
            }

            /* Go to the next drive letter */
            SkipDrives = SkipDrives.offset(1);
        }
        DbgPrint(b"\n\0".as_ptr() as *const i8);
    }

    /* Query the timeout */
    TimeOut = 3 as LONG;
    QueryTimeout(core::ptr::addr_of_mut!(TimeOut));

    /* Open the keyboard */
    Status = OpenKeyboard(core::ptr::addr_of_mut!(KeyboardHandle));
    if !NT_SUCCESS!(Status) {
        DPRINT1!("OpenKeyboard() failed, Status 0x%08lx\n", Status);
        /* Ignore keyboard interaction */
    }

    if CheckAllVolumes != 0 {
        /* Enumerate and check each of the available volumes */
        i = 0 as ULONG;
        while i <= ('Z' as i32 - 'A' as i32) as ULONG {
            if (DeviceMap.__bindgen_anon_1.Query.DriveMap & ((1 as ULONG) << i)) != 0
                && (DeviceMap.__bindgen_anon_1.Query.DriveType[i as usize] as i32 == (DOSDEVICE_DRIVE_FIXED as i32))
            {
                _swprintf(
                    addr_of_mut!(VolumePath) as *mut WCHAR,
                    RUSTCONV_WSTR_1F41958ED29A.as_ptr(),
                    (0x0041u16 as ULONG).wrapping_add(i),
                );
                CheckVolume(
                    addr_of_mut!(VolumePath) as *mut WCHAR as PCWSTR,
                    TimeOut,
                    CheckOnlyIfDirty,
                );
            }
            i = i.wrapping_add(1);
        }
    } else {
        /* Retrieve our index and analyse the volume */
        if wcslen(addr_of_mut!(VolumePath) as *mut WCHAR as *const WCHAR)
            == 6
            && VolumePath[0 as usize] == 0x005Cu16
            && VolumePath[1 as usize] == 0x003Fu16
            && VolumePath[2 as usize] == 0x003Fu16
            && VolumePath[3 as usize] == 0x005Cu16
            && VolumePath[5 as usize] == 0x003Au16
        {
            i = (toupper(VolumePath[4 as usize] as i32)
                - 'A' as i32) as ULONG;
            if (DeviceMap.__bindgen_anon_1.Query.DriveMap & ((1 as ULONG) << i)) != 0
                && (DeviceMap.__bindgen_anon_1.Query.DriveType[i as usize] as i32 == (DOSDEVICE_DRIVE_FIXED as i32))
            {
                CheckVolume(
                    addr_of_mut!(VolumePath) as *mut WCHAR as PCWSTR,
                    TimeOut,
                    CheckOnlyIfDirty,
                );
            }
        } else {
            /* Just perform the check on the specified volume */
            // TODO: Check volume type using NtQueryVolumeInformationFile(FileFsDeviceInformation)
            CheckVolume(
                addr_of_mut!(VolumePath) as *mut WCHAR as PCWSTR,
                TimeOut,
                CheckOnlyIfDirty,
            );
        }
    }

    /* Close the keyboard */
    if !KeyboardHandle.is_null() {
        NtClose(KeyboardHandle);
    }

    // PrintString("Done\r\n\r\n");
    return 0;
}

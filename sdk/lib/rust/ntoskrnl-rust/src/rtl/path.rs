use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{FILE_BASIC_INFORMATION, FILE_FS_DEVICE_INFORMATION, FileFsDeviceInformation, _IO_STATUS_BLOCK, NtClose, NtOpenFile, NtQueryVolumeInformationFile, _OBJECT_ATTRIBUTES, PCUNICODE_STRING, PCURDIR, PCWCH, PRTLP_CURDIR_REF, PRTL_RELATIVE_NAME_U, PRTL_UNICODE_STRING_BUFFER, PUNICODE_STRING, RTLP_CURDIR_REF, RTL_RELATIVE_NAME_U, RTL_UNICODE_STRING_BUFFER, RtlAcquirePebLock, RtlAllocateHeap, RtlAppendUnicodeStringToString, RtlDosSearchPath_Ustr, RtlEqualUnicodeString, RtlFindCharInUnicodeString, RtlFreeHeap, RtlGetCurrentPeb, RtlInitEmptyUnicodeString, RtlInitUnicodeStringEx, RtlPathTypeDriveAbsolute, RtlPathTypeDriveRelative, RtlPathTypeLocalDevice, RtlPathTypeRelative, RtlPathTypeRootLocalDevice, RtlPathTypeRooted, RtlPathTypeUncAbsolute, RtlPathTypeUnknown, RtlPrefixUnicodeString, RtlQueryEnvironmentVariable_U, RtlReleasePebLock, Type, Unknown, ZwQueryAttributesFile};
use core::ffi::c_void;

extern "system" {
    fn RtlpAllocateStringMemory(Length: ULONG, Tag: ULONG) -> PWSTR;
    fn RtlpFreeStringMemory(Buffer: *mut core::ffi::c_void, Tag: ULONG);
}

// ---- file-scope preamble (translated) ----
pub const MAX_PFX_SIZE: u32 = 16;

pub const RTL_CURDIR_IS_REMOVABLE: u32 = 0x1;

pub const RTL_CURDIR_DROP_OLD_HANDLE: u32 = 0x2;

pub const RTL_CURDIR_ALL_FLAGS: u32 = RTL_CURDIR_DROP_OLD_HANDLE | RTL_CURDIR_IS_REMOVABLE;

#[no_mangle]
pub static DeviceRootString: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: "\\\\.\\\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosDevicesUncPrefix: _UNICODE_STRING = _UNICODE_STRING {
    Length: 16,
    MaximumLength: 18,
    Buffer: "\\??\\UNC\\\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpWin32NtRootSlash: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: "\\\\?\\\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosSlashCONDevice: _UNICODE_STRING = _UNICODE_STRING {
    Length: 14,
    MaximumLength: 16,
    Buffer: "\\\\.\\CON\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosDevicesPrefix: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: "\\??\\\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosLPTDevice: _UNICODE_STRING = _UNICODE_STRING {
    Length: 6,
    MaximumLength: 8,
    Buffer: "LPT\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosCOMDevice: _UNICODE_STRING = _UNICODE_STRING {
    Length: 6,
    MaximumLength: 8,
    Buffer: "COM\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosPRNDevice: _UNICODE_STRING = _UNICODE_STRING {
    Length: 6,
    MaximumLength: 8,
    Buffer: "PRN\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosAUXDevice: _UNICODE_STRING = _UNICODE_STRING {
    Length: 6,
    MaximumLength: 8,
    Buffer: "AUX\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosCONDevice: _UNICODE_STRING = _UNICODE_STRING {
    Length: 6,
    MaximumLength: 8,
    Buffer: "CON\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDosNULDevice: _UNICODE_STRING = _UNICODE_STRING {
    Length: 6,
    MaximumLength: 8,
    Buffer: "NUL\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static RtlpDoubleSlashPrefix: _UNICODE_STRING = _UNICODE_STRING {
    Length: 4,
    MaximumLength: 6,
    Buffer: "\\\\\0".as_ptr() as PWSTR,
};

pub static RtlpDefaultExtension: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 10,
    Buffer: ".DLL\0".as_ptr() as PWSTR,
};

pub static RtlpDotLocal: _UNICODE_STRING = _UNICODE_STRING {
    Length: 14,
    MaximumLength: 16,
    Buffer: ".Local\\\0".as_ptr() as PWSTR,
};

pub static RtlpPathDividers: _UNICODE_STRING = _UNICODE_STRING {
    Length: 4,
    MaximumLength: 6,
    Buffer: "\\/\0".as_ptr() as PWSTR,
};

#[no_mangle]
pub static mut RtlpCurDirRef: PRTLP_CURDIR_REF = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn RtlDetermineDosPathNameType_Ustr(PathString: PCUNICODE_STRING) -> RTL_PATH_TYPE {
    let is_path_separator = |x: u16| x == b'\\' as u16 || x == b'/' as u16;

    let Path = (*PathString).Buffer;
    let Chars = ((*PathString).Length as ULONG) / (core::mem::size_of::<u16>() as ULONG);

    if Chars == 0 {
        return RtlPathTypeRelative;
    }

    if is_path_separator(*Path.offset(0)) {
        if Chars < 2 || !is_path_separator(*Path.offset(1)) {
            return RtlPathTypeRooted;
        }
        if Chars < 3 || (*Path.offset(2) != b'.' as u16 && *Path.offset(2) != b'?' as u16) {
            return RtlPathTypeUncAbsolute;
        }
        if Chars >= 4 && is_path_separator(*Path.offset(3)) {
            return RtlPathTypeLocalDevice;
        }
        if Chars != 3 {
            return RtlPathTypeUncAbsolute;
        }
        RtlPathTypeRootLocalDevice
    } else {
        if Chars < 2 || *Path.offset(1) != b':' as u16 {
            return RtlPathTypeRelative;
        }
        if Chars < 3 || !is_path_separator(*Path.offset(2)) {
            return RtlPathTypeDriveRelative;
        }
        RtlPathTypeDriveAbsolute
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsDosDeviceName_Ustr(PathString: PCUNICODE_STRING) -> ULONG {
    let is_path_separator = |x: u16| x == b'\\' as u16 || x == b'/' as u16;
    let make_long = |low: u16, high: u16| -> ULONG {
        ((low as ULONG) & 0xffff) | (((high as ULONG) & 0xffff) << 16)
    };

    if PathString.is_null() {
        return 0;
    }

    match RtlDetermineDosPathNameType_Ustr(PathString) {
        x if x == RtlPathTypeUnknown || x == RtlPathTypeUncAbsolute => {
            return 0;
        }
        x if x == RtlPathTypeLocalDevice => {
            if RtlEqualUnicodeString(PathString, &RtlpDosSlashCONDevice, 1u8) != 0 {
                return make_long(RtlpDosCONDevice.Length, DeviceRootString.Length);
            }
            return 0;
        }
        _ => {}
    }

    let mut PathCopy: _UNICODE_STRING = *PathString;
    let OriginalLength: USHORT = (*PathString).Length;

    let mut PathChars: USHORT = PathCopy.Length / (core::mem::size_of::<u16>() as USHORT);
    if PathChars == 0 {
        return 0;
    }

    let mut ColonCount: USHORT = 0;

    if *PathCopy.Buffer.offset((PathChars - 1) as isize) == b':' as u16 {
        PathCopy.Length -= core::mem::size_of::<u16>() as USHORT;
        PathChars -= 1;
        if PathChars == 0 {
            return 0;
        }
        ColonCount = 1;
    }

    loop {
        let c = *PathCopy.Buffer.offset((PathChars - 1) as isize);
        if c != b'.' as u16 && c != b' ' as u16 {
            break;
        }
        PathCopy.Length -= core::mem::size_of::<u16>() as USHORT;
        ColonCount += 1;
        PathChars -= 1;
        if PathChars == 0 {
            break;
        }
    }

    let mut ReturnOffset: USHORT = 0;

    if PathChars != 0 {
        let mut End = PathCopy.Buffer.offset((PathChars - 1) as isize);
        while End >= PathCopy.Buffer {
            let mut c = *End;
            if is_path_separator(c) || (c == b':' as u16 && End == PathCopy.Buffer.offset(1)) {
                End = End.offset(1);
                c = RtlpDowncaseUnicodeChar(*End);

                if (End < PathCopy.Buffer.offset((OriginalLength as isize) / (core::mem::size_of::<u16>() as isize)))
                    && (c == b'l' as u16 || c == b'c' as u16 || c == b'p' as u16 || c == b'a' as u16 || c == b'n' as u16)
                {
                    ReturnOffset = ((End as usize - PathCopy.Buffer as usize)) as USHORT;

                    PathCopy.Length = OriginalLength - ReturnOffset - (ColonCount * (core::mem::size_of::<u16>() as USHORT));
                    PathCopy.Buffer = End;

                    PathChars = PathCopy.Length / (core::mem::size_of::<u16>() as USHORT);

                    break;
                } else {
                    return 0;
                }
            }
            End = End.offset(-1);
        }

        let c = RtlpDowncaseUnicodeChar(*PathCopy.Buffer);
        if c != b'l' as u16 && c != b'c' as u16 && c != b'p' as u16 && c != b'a' as u16 && c != b'n' as u16 {
            return 0;
        }
    }

    let mut Start = PathCopy.Buffer;
    let End = Start.offset(PathChars as isize);
    while Start < End {
        let c = *Start;
        if c == b'.' as u16 || c == b':' as u16 {
            break;
        }
        Start = Start.offset(1);
    }

    while Start > PathCopy.Buffer && *Start.offset(-1) == b' ' as u16 {
        Start = Start.offset(-1);
    }

    PathChars = ((Start as usize - PathCopy.Buffer as usize) / core::mem::size_of::<u16>()) as USHORT;
    PathCopy.Length = PathChars * (core::mem::size_of::<u16>() as USHORT);

    if PathChars == 4
        && iswdigit(*PathCopy.Buffer.offset(3) as i32) != 0
        && *PathCopy.Buffer.offset(3) != b'0' as u16
    {
        PathCopy.Length -= core::mem::size_of::<u16>() as USHORT;
        if RtlEqualUnicodeString(&PathCopy, &RtlpDosLPTDevice, 1u8) != 0
            || RtlEqualUnicodeString(&PathCopy, &RtlpDosCOMDevice, 1u8) != 0
        {
            let ReturnLength: USHORT = (5 * core::mem::size_of::<u16>() - core::mem::size_of::<u16>()) as USHORT;
            return make_long(ReturnLength, ReturnOffset);
        }
    } else if PathChars == 3
        && (RtlEqualUnicodeString(&PathCopy, &RtlpDosPRNDevice, 1u8) != 0
            || RtlEqualUnicodeString(&PathCopy, &RtlpDosAUXDevice, 1u8) != 0
            || RtlEqualUnicodeString(&PathCopy, &RtlpDosNULDevice, 1u8) != 0
            || RtlEqualUnicodeString(&PathCopy, &RtlpDosCONDevice, 1u8) != 0)
    {
        let ReturnLength: USHORT = (4 * core::mem::size_of::<u16>() - core::mem::size_of::<u16>()) as USHORT;
        return make_long(ReturnLength, ReturnOffset);
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCheckDeviceName(
    FileName: PUNICODE_STRING,
    Length: ULONG,
    NameInvalid: PBOOLEAN,
) -> NTSTATUS {
    let Buffer: PWCHAR = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (*FileName).Length as usize,
    ) as PWCHAR;
    let Status: NTSTATUS;

    if !Buffer.is_null() {
        *NameInvalid = 1u8;

        RtlCopyMemory(
            Buffer as *mut core::ffi::c_void,
            (*FileName).Buffer as *const core::ffi::c_void,
            (*FileName).Length as usize,
        );

        *Buffer.offset((Length as usize / core::mem::size_of::<u16>()) as isize) = '.' as u16;
        *Buffer.offset((Length as usize / core::mem::size_of::<u16>() + 1) as isize) = 0u16;

        *NameInvalid = if RtlDoesFileExists_U(Buffer) != 0 { 0u8 } else { 1u8 };

        Status = RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as *mut core::ffi::c_void) as NTSTATUS;
    } else {
        *NameInvalid = 0u8;
        Status = STATUS_NO_MEMORY;
    }

    Status
}

unsafe fn RtlpCollapsePath(Path: PWSTR, mark: ULONG, SkipTrailingPathSeparators: BOOLEAN) -> ULONG {
    let mut p: PWSTR;
    let mut next: PWSTR;

    let mut PathLength: usize = wcslen(Path);
    let EndBuffer: PWSTR = Path.add(PathLength);
    let mut EndPath: PWSTR;

    /* Convert slashes into backslashes */
    p = Path;
    while *p != 0 {
        if *p == b'/' as u16 {
            *p = b'\\' as u16;
        }
        p = p.add(1);
    }

    /* Collapse duplicate backslashes */
    let mark_max = core::cmp::max(1u32, mark) as usize;
    next = Path.add(mark_max);
    p = next;
    while *p != 0 {
        if *p != b'\\' as u16 || *next.offset(-1) != b'\\' as u16 {
            *next = *p;
            next = next.add(1);
        }
        p = p.add(1);
    }
    *next = 0;
    EndPath = next;

    p = Path.add(mark as usize);
    'outer: while *p != 0 {
        if *p == b'.' as u16 {
            let c1 = *p.add(1);
            if c1 == 0 {
                /* final . */
                if p > Path.add(mark as usize) {
                    p = p.offset(-1);
                }
                *p = 0;
                EndPath = p;
                continue 'outer;
            } else if c1 == b'\\' as u16 {
                /* .\ component */
                next = p.add(2);
                core::ptr::copy(next, p, (EndPath.offset_from(next) + 1) as usize);
                EndPath = EndPath.offset(-(next.offset_from(p)));
                continue 'outer;
            } else if c1 == b'.' as u16 {
                let c2 = *p.add(2);
                if c2 == b'\\' as u16 {
                    /* ..\ component */
                    next = p.add(3);
                    if p > Path.add(mark as usize) {
                        p = p.offset(-1);
                        while p > Path.add(mark as usize) && *p.offset(-1) != b'\\' as u16 {
                            p = p.offset(-1);
                        }
                    }
                    core::ptr::copy(next, p, (EndPath.offset_from(next) + 1) as usize);
                    EndPath = EndPath.offset(-(next.offset_from(p)));
                    continue 'outer;
                } else if c2 == 0 {
                    /* final .. */
                    if p > Path.add(mark as usize) {
                        p = p.offset(-1);
                        while p > Path.add(mark as usize) && *p.offset(-1) != b'\\' as u16 {
                            p = p.offset(-1);
                        }
                        if p > Path.add(mark as usize) {
                            p = p.offset(-1);
                        }
                    }
                    *p = 0;
                    EndPath = p;
                    continue 'outer;
                }
            }
        }

        /* Skip to the next component */
        while *p != 0 && *p != b'\\' as u16 {
            p = p.add(1);
        }
        if *p == b'\\' as u16 {
            /* Remove last dot in previous dir name */
            if p > Path.add(mark as usize) && *p.offset(-1) == b'.' as u16 {
                core::ptr::copy(p, p.offset(-1), (EndPath.offset_from(p) + 1) as usize);
                EndPath = EndPath.offset(-1);
            } else {
                p = p.add(1);
            }
        }
    }

    /* Remove trailing backslashes if needed */
    if SkipTrailingPathSeparators != 0 {
        while p > Path.add(mark as usize)
            && (*p.offset(-1) == b'\\' as u16 || *p.offset(-1) == b'/' as u16)
        {
            p = p.offset(-1);
        }
    }

    /* Remove trailing spaces and dots */
    while p > Path && (*p.offset(-1) == b' ' as u16 || *p.offset(-1) == b'.' as u16) {
        p = p.offset(-1);
    }

    RtlZeroMemory(p as *mut _, ((EndBuffer.offset_from(p) + 1) as usize) * core::mem::size_of::<u16>());

    PathLength = p.offset_from(Path) as usize;
    (PathLength * core::mem::size_of::<u16>()) as ULONG
}

unsafe fn RtlpSkipUNCPrefix(FileNameBuffer: PCWSTR) -> SIZE_T {
    let mut UncPath: PCWSTR = FileNameBuffer.add(2);

    while *UncPath != 0 && !(*UncPath == b'\\' as u16 || *UncPath == b'/' as u16) {
        UncPath = UncPath.add(1);
    }
    while *UncPath == b'\\' as u16 || *UncPath == b'/' as u16 {
        UncPath = UncPath.add(1);
    }
    while *UncPath != 0 && !(*UncPath == b'\\' as u16 || *UncPath == b'/' as u16) {
        UncPath = UncPath.add(1);
    }

    (UncPath.offset_from(FileNameBuffer)) as SIZE_T
}
#[no_mangle]
pub unsafe extern "system" fn RtlpApplyLengthFunction(
    Flags: ULONG,
    Type: ULONG,
    UnicodeStringOrUnicodeStringBuffer: PVOID,
    LengthFunction: Option<unsafe extern "system" fn(ULONG, PUNICODE_STRING, PULONG) -> NTSTATUS>,
) -> NTSTATUS {
    let String: PUNICODE_STRING;
    let mut Length: ULONG;

    if Flags != 0 || UnicodeStringOrUnicodeStringBuffer.is_null() || LengthFunction.is_none() {
        return STATUS_INVALID_PARAMETER;
    }

    if Type as usize == core::mem::size_of::<_UNICODE_STRING>() {
        String = UnicodeStringOrUnicodeStringBuffer as PUNICODE_STRING;
    } else if Type as usize == core::mem::size_of::<RTL_UNICODE_STRING_BUFFER>() {
        String = core::ptr::addr_of_mut!((*(UnicodeStringOrUnicodeStringBuffer as PRTL_UNICODE_STRING_BUFFER)).String);
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    Length = 0;
    let Status = (LengthFunction.unwrap())(0, String, &mut Length);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if Length > UNICODE_STRING_MAX_CHARS {
        return STATUS_NAME_TOO_LONG;
    }

    (*String).Length = (Length * core::mem::size_of::<u16>() as ULONG) as u16;

    if Type as usize == core::mem::size_of::<RTL_UNICODE_STRING_BUFFER>() {
        *(*String).Buffer.offset(Length as isize) = 0u16;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetLengthWithoutLastFullDosOrNtPathElement(Flags: ULONG, Path: PCUNICODE_STRING, LengthOut: *mut ULONG) -> NTSTATUS {
    let is_path_separator = |x: u16| x == b'\\' as u16 || x == b'/' as u16;

    let mut PathDividersBuffer: [u16; 2] = [b'\\' as u16, b'/' as u16];
    let mut PathDividers: _UNICODE_STRING = core::mem::zeroed();
    PathDividers.Length = (2 * core::mem::size_of::<u16>()) as u16;
    PathDividers.MaximumLength = (2 * core::mem::size_of::<u16>()) as u16;
    PathDividers.Buffer = PathDividersBuffer.as_mut_ptr();

    let mut Position: u16 = 0;

    if !LengthOut.is_null() {
        *LengthOut = 0;
    }

    if Flags != 0 || Path.is_null() || LengthOut.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*Path).Length as usize) / core::mem::size_of::<u16>() == 0 {
        return STATUS_SUCCESS;
    }

    let PathType = RtlDetermineDosPathNameType_Ustr(Path);

    if PathType == RtlPathTypeLocalDevice {
        if ((*Path).Length as usize) / core::mem::size_of::<u16>() < 7
            || *(*Path).Buffer.offset(5) != b':' as u16
            || !is_path_separator(*(*Path).Buffer.offset(6))
        {
            return STATUS_INVALID_PARAMETER;
        }
    } else if PathType == RtlPathTypeRooted {
    } else if PathType == RtlPathTypeUncAbsolute {
    } else if PathType == RtlPathTypeDriveAbsolute {
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    if !NT_SUCCESS(RtlFindCharInUnicodeString(
        1,
        Path,
        &PathDividers,
        &mut Position,
    )) {
        Position = 0;
    }

    if Position != 0 && (Position as usize + core::mem::size_of::<u16>()) == (*Path).Length as usize {
        let mut Tmp: _UNICODE_STRING = *Path;
        Tmp.Length = Position;

        while (Tmp.Length as usize) > core::mem::size_of::<u16>()
            && is_path_separator(*Tmp.Buffer.offset((Tmp.Length as usize / core::mem::size_of::<u16>()) as isize))
        {
            Tmp.Length -= core::mem::size_of::<u16>() as u16;
        }

        if !NT_SUCCESS(RtlFindCharInUnicodeString(
            1,
            &Tmp,
            &PathDividers,
            &mut Position,
        )) {
            Position = 0;
        }
    }

    if Position != 0 {
        Position /= core::mem::size_of::<u16>() as u16;
    }

    if Position != 0 {
        while Position > 1 && is_path_separator(*(*Path).Buffer.offset((Position - 1) as isize)) {
            Position -= 1;
        }
    }

    if Position > 0 {
        *LengthOut = (Position + 1) as ULONG;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlComputePrivatizedDllName_U(
    DllName: PUNICODE_STRING,
    RealName: PUNICODE_STRING,
    LocalName: PUNICODE_STRING,
) -> NTSTATUS {
    const RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END: ULONG = 0x00000001;
    const RTL_USER_PROCESS_PARAMETERS_NORMALIZED: ULONG = 0x00000001;
    const TAG_USTR: ULONG = 0x52545355; // placeholder tag
    const UNICODE_STRING_MAX_BYTES: ULONG = 65534;
    const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC0000106u32 as NTSTATUS;

    let ExtensionChar: _UNICODE_STRING = _UNICODE_STRING {
        Length: 2,
        MaximumLength: 4,
        Buffer: {
            static DOT: [u16; 2] = [0x2E, 0x0000];
            DOT.as_ptr() as PWSTR
        },
    };

    let mut Position: u16;
    let mut ImagePathName: _UNICODE_STRING;
    let mut DllNameOnly: _UNICODE_STRING = core::mem::zeroed();
    let mut CopyRealName: _UNICODE_STRING;
    let mut CopyLocalName: _UNICODE_STRING;
    let HasExtension: u8;
    let mut RequiredSize: ULONG;
    let mut Status: NTSTATUS;

    CopyRealName = *RealName;
    CopyLocalName = *LocalName;

    let Peb = RtlGetCurrentPeb();
    let ProcessParameters = (*Peb).ProcessParameters;
    ImagePathName = (*ProcessParameters).ImagePathName;

    if ((*ProcessParameters).Flags & RTL_USER_PROCESS_PARAMETERS_NORMALIZED) == 0 {
        ImagePathName.Buffer =
            (ImagePathName.Buffer as usize + ProcessParameters as usize) as PWSTR;
    }

    let mut Pos: u16 = 0;
    if !NT_SUCCESS(RtlFindCharInUnicodeString(
        RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
        DllName,
        &RtlpPathDividers,
        &mut Pos,
    )) {
        DllNameOnly = *DllName;
    } else {
        Pos += core::mem::size_of::<u16>() as u16;
        DllNameOnly.Buffer = (*DllName).Buffer.add(Pos as usize / core::mem::size_of::<u16>());
        DllNameOnly.Length = (*DllName).Length - Pos;
        DllNameOnly.MaximumLength = (*DllName).MaximumLength - Pos;
    }

    Position = 0;
    if !NT_SUCCESS(RtlFindCharInUnicodeString(
        RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
        &mut DllNameOnly,
        &ExtensionChar,
        &mut Position,
    )) {
        Position = 0;
    }

    HasExtension = if Position > 1 { 1 } else { 0 };

    RequiredSize = ImagePathName.Length as ULONG
        + RtlpDotLocal.Length as ULONG
        + DllNameOnly.Length as ULONG
        + (if HasExtension != 0 { 0 } else { RtlpDefaultExtension.Length as ULONG })
        + core::mem::size_of::<u16>() as ULONG;

    if RequiredSize > UNICODE_STRING_MAX_BYTES {
        return STATUS_NAME_TOO_LONG;
    }

    if RequiredSize > CopyLocalName.MaximumLength as ULONG {
        CopyLocalName.Buffer = RtlpAllocateStringMemory(RequiredSize, TAG_USTR);
        if CopyLocalName.Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }
        CopyLocalName.MaximumLength = RequiredSize as u16;
    }

    CopyLocalName.Length = 0;
    Status = RtlAppendUnicodeStringToString(&mut CopyLocalName, &mut ImagePathName);
    if NT_SUCCESS(Status) {
        Status = RtlAppendUnicodeStringToString(
            &mut CopyLocalName,
            &RtlpDotLocal as *const _ as PUNICODE_STRING,
        );
    }
    if NT_SUCCESS(Status) {
        Status = RtlAppendUnicodeStringToString(&mut CopyLocalName, &mut DllNameOnly);
    }
    if NT_SUCCESS(Status) && HasExtension == 0 {
        Status = RtlAppendUnicodeStringToString(
            &mut CopyLocalName,
            &RtlpDefaultExtension as *const _ as PUNICODE_STRING,
        );
    }

    if NT_SUCCESS(Status) {
        let mut Pos2: u16 = 0;
        if NT_SUCCESS(RtlFindCharInUnicodeString(
            RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
            &mut ImagePathName,
            &RtlpPathDividers,
            &mut Pos2,
        )) {
            ImagePathName.Length = Pos2 + core::mem::size_of::<u16>() as u16;
        }

        RequiredSize = ImagePathName.Length as ULONG
            + DllNameOnly.Length as ULONG
            + (if HasExtension != 0 { 0 } else { RtlpDefaultExtension.Length as ULONG })
            + core::mem::size_of::<u16>() as ULONG;

        if RequiredSize >= UNICODE_STRING_MAX_BYTES {
            Status = STATUS_NAME_TOO_LONG;
        } else {
            if RequiredSize > CopyRealName.MaximumLength as ULONG {
                CopyRealName.Buffer = RtlpAllocateStringMemory(RequiredSize, TAG_USTR);
                if CopyRealName.Buffer.is_null() {
                    Status = STATUS_NO_MEMORY;
                }
                CopyRealName.MaximumLength = RequiredSize as u16;
            }
            CopyRealName.Length = 0;
            if NT_SUCCESS(Status) {
                Status = RtlAppendUnicodeStringToString(&mut CopyRealName, &mut ImagePathName);
            }
            if NT_SUCCESS(Status) {
                Status = RtlAppendUnicodeStringToString(&mut CopyRealName, &mut DllNameOnly);
            }
            if NT_SUCCESS(Status) && HasExtension == 0 {
                Status = RtlAppendUnicodeStringToString(
                    &mut CopyRealName,
                    &RtlpDefaultExtension as *const _ as PUNICODE_STRING,
                );
            }
        }
    }

    if !NT_SUCCESS(Status) {
        if !CopyRealName.Buffer.is_null() && CopyRealName.Buffer != (*RealName).Buffer {
            RtlpFreeStringMemory(CopyRealName.Buffer as *mut core::ffi::c_void, TAG_USTR);
        }
        if !CopyLocalName.Buffer.is_null() && CopyLocalName.Buffer != (*LocalName).Buffer {
            RtlpFreeStringMemory(CopyLocalName.Buffer as *mut core::ffi::c_void, TAG_USTR);
        }
        return Status;
    }

    *RealName = CopyRealName;
    *LocalName = CopyLocalName;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetFullPathName_Ustr(
    FileName: PUNICODE_STRING,
    Size: ULONG,
    Buffer: PWSTR,
    ShortName: *mut PCWSTR,
    InvalidName: PBOOLEAN,
    PathType: *mut RTL_PATH_TYPE,
) -> ULONG {
    let is_path_separator = |x: u16| x == b'\\' as u16 || x == b'/' as u16;
    let hiword = |l: ULONG| -> u16 { ((l >> 16) & 0xffff) as u16 };
    let loword = |l: ULONG| -> u16 { (l & 0xffff) as u16 };

    let Status: NTSTATUS;
    let FileNameBuffer: PWCHAR;
    let mut FileNameLength: ULONG;
    let FileNameChars: ULONG;
    let mut DosLength: ULONG;
    let DosLengthOffset: ULONG;
    let FullLength: ULONG;
    let SkipTrailingPathSeparators: BOOLEAN;
    let mut c: WCHAR;

    let mut reqsize: ULONG = 0;
    let mut ptr: PCWSTR;

    let CurDirName: PCUNICODE_STRING;
    let mut EnvVarName: _UNICODE_STRING = core::mem::zeroed();
    let mut EnvVarValue: _UNICODE_STRING = core::mem::zeroed();
    let mut EnvVarNameBuffer: [WCHAR; 4] = [0; 4];

    let mut PrefixCut: ULONG = 0;
    let mut Prefix: PWCHAR = core::ptr::null_mut();
    let mut PrefixLength: ULONG = 0;
    let mut Source: PWCHAR;
    let mut SourceLength: ULONG;

    if !InvalidName.is_null() {
        *InvalidName = 0u8;
    }

    *PathType = RtlPathTypeUnknown;
    if (*FileName).Length == 0 || *(*FileName).Buffer.offset(0) == 0 {
        return 0;
    }

    FileNameBuffer = (*FileName).Buffer;
    FileNameLength = (*FileName).Length as ULONG;
    FileNameChars = FileNameLength / core::mem::size_of::<WCHAR>() as ULONG;

    c = *FileNameBuffer.offset((FileNameChars - 1) as isize);
    while FileNameLength != 0 && c == b' ' as u16 {
        FileNameLength -= core::mem::size_of::<WCHAR>() as ULONG;
        if FileNameLength != 0 {
            c = *FileNameBuffer.offset(
                (FileNameLength / core::mem::size_of::<WCHAR>() as ULONG - 1) as isize,
            );
        }
    }

    if FileNameLength == 0 {
        return 0;
    }

    SkipTrailingPathSeparators =
        if is_path_separator(*FileNameBuffer.offset((FileNameChars - 1) as isize)) {
            0u8
        } else {
            1u8
        };

    DosLength = RtlIsDosDeviceName_Ustr(FileName);
    if DosLength != 0 {
        if !ShortName.is_null() {
            *ShortName = core::ptr::null_mut();
        }

        DosLengthOffset = hiword(DosLength) as ULONG;
        DosLength = loword(DosLength) as ULONG;

        if !InvalidName.is_null() && DosLengthOffset != 0 {
            Status = RtlpCheckDeviceName(FileName, DosLengthOffset, InvalidName);

            if !NT_SUCCESS(Status) {
                return 0;
            }
            if *InvalidName != 0 {
                return 0;
            }
        }

        FullLength = DosLength + DeviceRootString.Length as ULONG;
        if FullLength < Size {
            core::ptr::copy(
                DeviceRootString.Buffer as *const u8,
                Buffer as *mut u8,
                DeviceRootString.Length as usize,
            );

            core::ptr::copy(
                (FileNameBuffer as *const u8).add(DosLengthOffset as usize),
                (Buffer as *mut u8).add(DeviceRootString.Length as usize),
                DosLength as usize,
            );

            *((Buffer as usize + FullLength as usize) as *mut WCHAR) = 0;
            return FullLength;
        }

        if (FullLength + core::mem::size_of::<WCHAR>() as ULONG) > 0xfffe {
            return 0;
        }
        return FullLength + core::mem::size_of::<WCHAR>() as ULONG;
    }

    RtlZeroMemory(Buffer as *mut _, Size as usize);

    *PathType = RtlDetermineDosPathNameType_U(FileNameBuffer);

    Source = FileNameBuffer;
    SourceLength = FileNameLength;
    EnvVarValue.Buffer = core::ptr::null_mut();

    RtlAcquirePebLock();

    let peb = {
        #[cfg(target_arch = "x86_64")]
        {
            let teb: *mut u8;
            core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb);
            *((teb as usize + 0x60) as *const *mut _NT_PEB)
        }
        #[cfg(target_arch = "x86")]
        {
            let teb: *mut u8;
            core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb);
            *((teb as usize + 0x30) as *const *mut _NT_PEB)
        }
    };
    CurDirName =
        core::ptr::addr_of!((*(*peb).ProcessParameters).CurrentDirectory.DosPath) as PCUNICODE_STRING;

    'quit: {
        loop {
            match *PathType {
                x if x == RtlPathTypeUncAbsolute => {
                    PrefixCut = RtlpSkipUNCPrefix(FileNameBuffer) as ULONG;
                    break;
                }
                x if x == RtlPathTypeLocalDevice => {
                    PrefixCut = 4;
                    break;
                }
                x if x == RtlPathTypeDriveAbsolute => {
                    Prefix = FileNameBuffer;
                    PrefixLength = 3 * core::mem::size_of::<WCHAR>() as ULONG;
                    Source = Source.offset(3);
                    SourceLength -= 3 * core::mem::size_of::<WCHAR>() as ULONG;
                    PrefixCut = 3;
                    break;
                }
                x if x == RtlPathTypeDriveRelative => {
                    Source = Source.offset(2);
                    SourceLength -= 2 * core::mem::size_of::<WCHAR>() as ULONG;

                    let CurDrive = RtlpUpcaseUnicodeChar(*(*CurDirName).Buffer.offset(0));
                    let NewDrive = RtlpUpcaseUnicodeChar(*FileNameBuffer.offset(0));

                    if NewDrive != CurDrive || *(*CurDirName).Buffer.offset(1) != b':' as u16 {
                        EnvVarNameBuffer[0] = b'=' as u16;
                        EnvVarNameBuffer[1] = NewDrive;
                        EnvVarNameBuffer[2] = b':' as u16;
                        EnvVarNameBuffer[3] = 0;

                        EnvVarName.Length = (3 * core::mem::size_of::<WCHAR>()) as u16;
                        EnvVarName.MaximumLength =
                            EnvVarName.Length + core::mem::size_of::<WCHAR>() as u16;
                        EnvVarName.Buffer = EnvVarNameBuffer.as_mut_ptr();

                        EnvVarValue.Length = 0;
                        EnvVarValue.MaximumLength = Size as u16;
                        EnvVarValue.Buffer =
                            RtlAllocateHeap(RtlGetProcessHeap(), 0, Size as usize) as PWSTR;
                        if EnvVarValue.Buffer.is_null() {
                            Prefix = core::ptr::null_mut();
                            PrefixLength = 0;
                            break 'quit;
                        }

                        let qStatus = RtlQueryEnvironmentVariable_U(
                            core::ptr::null_mut(),
                            &mut EnvVarName,
                            &mut EnvVarValue,
                        );
                        if qStatus == STATUS_SUCCESS {
                            *EnvVarValue.Buffer.offset(
                                (EnvVarValue.Length as usize
                                    / core::mem::size_of::<WCHAR>())
                                    as isize,
                            ) = b'\\' as u16;
                            Prefix = EnvVarValue.Buffer;
                            PrefixLength = EnvVarValue.Length as ULONG
                                + core::mem::size_of::<WCHAR>() as ULONG;
                        } else if qStatus == 0xC0000023u32 as NTSTATUS {
                            reqsize = EnvVarValue.Length as ULONG
                                + SourceLength
                                + core::mem::size_of::<WCHAR>() as ULONG;
                            break 'quit;
                        } else {
                            EnvVarNameBuffer[0] = NewDrive;
                            EnvVarNameBuffer[1] = b':' as u16;
                            EnvVarNameBuffer[2] = b'\\' as u16;
                            EnvVarNameBuffer[3] = 0;
                            Prefix = EnvVarNameBuffer.as_mut_ptr();
                            PrefixLength = 3 * core::mem::size_of::<WCHAR>() as ULONG;

                            RtlFreeHeap(RtlGetProcessHeap(), 0, EnvVarValue.Buffer as *mut _);
                            EnvVarValue.Buffer = core::ptr::null_mut();
                        }
                        PrefixCut = 3;
                        break;
                    }
                    // Fall through to RtlPathTypeRelative
                    Prefix = (*CurDirName).Buffer;
                    PrefixLength = (*CurDirName).Length as ULONG;
                    if *(*CurDirName).Buffer.offset(1) != b':' as u16 {
                        PrefixCut = RtlpSkipUNCPrefix((*CurDirName).Buffer) as ULONG;
                    } else {
                        PrefixCut = 3;
                    }
                    break;
                }
                x if x == RtlPathTypeRelative => {
                    Prefix = (*CurDirName).Buffer;
                    PrefixLength = (*CurDirName).Length as ULONG;
                    if *(*CurDirName).Buffer.offset(1) != b':' as u16 {
                        PrefixCut = RtlpSkipUNCPrefix((*CurDirName).Buffer) as ULONG;
                    } else {
                        PrefixCut = 3;
                    }
                    break;
                }
                x if x == RtlPathTypeRooted => {
                    if *(*CurDirName).Buffer.offset(1) == b':' as u16 {
                        Prefix = (*CurDirName).Buffer;
                        PrefixLength = 3 * core::mem::size_of::<WCHAR>() as ULONG;
                        PrefixCut = 3;
                    } else {
                        PrefixCut = RtlpSkipUNCPrefix((*CurDirName).Buffer) as ULONG;
                        PrefixLength = PrefixCut * core::mem::size_of::<WCHAR>() as ULONG;
                        Prefix = (*CurDirName).Buffer;
                    }
                    break;
                }
                x if x == RtlPathTypeRootLocalDevice => {
                    Prefix = DeviceRootString.Buffer;
                    PrefixLength = DeviceRootString.Length as ULONG;
                    Source = Source.offset(3);
                    SourceLength -= 3 * core::mem::size_of::<WCHAR>() as ULONG;
                    PrefixCut = 4;
                    break;
                }
                _ => {
                    break 'quit;
                }
            }
        }

        reqsize = PrefixLength;
        if reqsize + SourceLength + core::mem::size_of::<WCHAR>() as ULONG > Size {
            reqsize += SourceLength + core::mem::size_of::<WCHAR>() as ULONG;
            break 'quit;
        }

        if PrefixLength != 0 {
            core::ptr::copy(Prefix, Buffer, PrefixLength as usize / core::mem::size_of::<WCHAR>());
        }
        core::ptr::copy(
            Source,
            Buffer.offset((PrefixLength / core::mem::size_of::<WCHAR>() as ULONG) as isize),
            (SourceLength as usize + core::mem::size_of::<WCHAR>()) / core::mem::size_of::<WCHAR>(),
        );

        Prefix = core::ptr::null_mut();
        if !EnvVarValue.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, EnvVarValue.Buffer as *mut _);
            EnvVarValue.Buffer = core::ptr::null_mut();
        }

        reqsize = RtlpCollapsePath(Buffer, PrefixCut, SkipTrailingPathSeparators);

        if !ShortName.is_null() {
            ptr = wcsrchr(Buffer, b'\\' as u16);
            if !ptr.is_null() {
                ptr = ptr.offset(1);
            }

            PrefixCut = if *PathType == RtlPathTypeUncAbsolute {
                PrefixCut
            } else {
                3
            };

            if !ptr.is_null()
                && *ptr != 0
                && ptr as usize >= Buffer.offset(PrefixCut as isize) as usize
            {
                *ShortName = ptr;
            } else {
                *ShortName = core::ptr::null_mut();
            }
        }
    }

    RtlReleasePebLock();

    reqsize
}

#[no_mangle]
pub unsafe extern "system" fn RtlpWin32NTNameToNtPathName_U(
    DosPath: PUNICODE_STRING,
    NtPath: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> NTSTATUS {
    let DosLength: ULONG;
    let NewBuffer: PWSTR;
    let mut p: PWSTR;

    if DosPath.is_null() {
        return STATUS_OBJECT_NAME_INVALID;
    }

    DosLength = (*DosPath).Length as ULONG;
    if DosLength >= UNICODE_STRING_MAX_BYTES as ULONG {
        return STATUS_NAME_TOO_LONG;
    }

    NewBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (DosLength + core::mem::size_of::<WCHAR>() as ULONG) as usize,
    ) as PWSTR;
    if NewBuffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlCopyMemory(
        NewBuffer as PVOID,
        RtlpDosDevicesPrefix.Buffer as PVOID,
        RtlpDosDevicesPrefix.Length as usize,
    );
    RtlCopyMemory(
        (NewBuffer as PCHAR).add(RtlpDosDevicesPrefix.Length as usize) as PVOID,
        (*DosPath)
            .Buffer
            .add(RtlpDosDevicesPrefix.Length as usize / core::mem::size_of::<WCHAR>()) as PVOID,
        ((*DosPath).Length - RtlpDosDevicesPrefix.Length) as usize,
    );
    *NewBuffer.add(DosLength as usize / core::mem::size_of::<WCHAR>()) = 0;

    if !RelativeName.is_null() {
        RtlInitEmptyUnicodeString(
            core::ptr::addr_of_mut!((*RelativeName).RelativeName),
            core::ptr::null_mut(),
            0,
        );
        (*RelativeName).ContainingDirectory = core::ptr::null_mut();
        (*RelativeName).CurDirRef = core::ptr::null_mut();
    }

    if !PartName.is_null() {
        p = NewBuffer.add(DosLength as usize / core::mem::size_of::<WCHAR>());
        loop {
            p = p.sub(1);
            if !(p > NewBuffer) {
                break;
            }
            if *p == '\\' as WCHAR {
                p = p.add(1);
                break;
            }
        }

        if (p > NewBuffer) && (*p != 0) {
            *PartName = p;
        } else {
            *PartName = core::ptr::null_mut();
        }
    }

    (*NtPath).Buffer = NewBuffer;
    (*NtPath).Length = DosLength as USHORT;
    (*NtPath).MaximumLength = DosLength as USHORT + core::mem::size_of::<WCHAR>() as USHORT;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDosPathNameToRelativeNtPathName_Ustr(
    HaveRelative: BOOLEAN,
    DosName: PCUNICODE_STRING,
    NtName: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> NTSTATUS {
    let mut BigBuffer: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let PrefixBuffer: PWCHAR;
    let NewBuffer: PWCHAR;
    let Buffer: PWCHAR;
    let mut MaxLength: ULONG;
    let PathLength: ULONG;
    let PrefixLength: ULONG;
    let PrefixCut: ULONG;
    let LengthChars: ULONG;
    let Length: ULONG;
    let mut CapturedDosName: _UNICODE_STRING;
    let mut PartNameString: _UNICODE_STRING = core::mem::zeroed();
    let mut FullPath: _UNICODE_STRING = core::mem::zeroed();
    let QuickPath: BOOLEAN;
    let mut InputPathType: RTL_PATH_TYPE = 0;
    let BufferPathType: RTL_PATH_TYPE;
    let mut Status: NTSTATUS;
    let mut NameInvalid: BOOLEAN = 0;
    let CurrentDirectory: PCURDIR;

    MaxLength = core::mem::size_of::<[u16; MAX_PATH as usize + 1]>() as ULONG;

    if DosName.is_null() {
        return STATUS_OBJECT_NAME_INVALID;
    }

    CapturedDosName = *DosName;

    if (CapturedDosName.Length <= RtlpWin32NtRootSlash.Length)
        || (*CapturedDosName.Buffer.offset(0) != *RtlpWin32NtRootSlash.Buffer.offset(0))
        || (*CapturedDosName.Buffer.offset(1) != *RtlpWin32NtRootSlash.Buffer.offset(1))
        || (*CapturedDosName.Buffer.offset(2) != *RtlpWin32NtRootSlash.Buffer.offset(2))
        || (*CapturedDosName.Buffer.offset(3) != *RtlpWin32NtRootSlash.Buffer.offset(3))
    {
        QuickPath = 0;

        Buffer = BigBuffer.as_mut_ptr();
        MaxLength += RtlpDosDevicesUncPrefix.Length as ULONG;

        NewBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, MaxLength as usize) as PWCHAR;
        if NewBuffer.is_null() {
            return STATUS_NO_MEMORY;
        }
    } else {
        QuickPath = 1;
        NewBuffer = core::ptr::null_mut();
        Buffer = BigBuffer.as_mut_ptr();
    }

    RtlAcquirePebLock();
    if QuickPath != 0 {
        Status = RtlpWin32NTNameToNtPathName_U(
            core::ptr::addr_of_mut!(CapturedDosName),
            NtName,
            PartName,
            RelativeName,
        );
        RtlReleasePebLock();
        return Status;
    }

    PathLength = RtlGetFullPathName_Ustr(
        core::ptr::addr_of_mut!(CapturedDosName),
        MAX_PATH * core::mem::size_of::<u16>() as ULONG,
        Buffer,
        PartName as *mut PCWSTR,
        core::ptr::addr_of_mut!(NameInvalid),
        core::ptr::addr_of_mut!(InputPathType),
    );
    if (NameInvalid != 0) || (PathLength == 0) || (PathLength > MAX_PATH * core::mem::size_of::<u16>() as ULONG) {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer as PVOID);
        RtlReleasePebLock();
        return STATUS_OBJECT_NAME_INVALID;
    }

    PrefixLength = RtlpDosDevicesPrefix.Length as ULONG;
    PrefixBuffer = RtlpDosDevicesPrefix.Buffer;
    PrefixCut = 0;

    BufferPathType = RtlDetermineDosPathNameType_U(Buffer);

    let (PrefixLength, PrefixBuffer, PrefixCut) = match BufferPathType {
        x if x == RtlPathTypeUncAbsolute => (
            RtlpDosDevicesUncPrefix.Length as ULONG,
            RtlpDosDevicesUncPrefix.Buffer,
            2u32,
        ),
        x if x == RtlPathTypeLocalDevice => (PrefixLength, PrefixBuffer, 4u32),
        _ => (PrefixLength, PrefixBuffer, PrefixCut),
    };

    core::ptr::copy_nonoverlapping(PrefixBuffer as *const u8, NewBuffer as *mut u8, PrefixLength as usize);
    core::ptr::copy_nonoverlapping(
        Buffer.offset(PrefixCut as isize) as *const u8,
        (NewBuffer as *mut u8).offset(PrefixLength as isize),
        (PathLength - PrefixCut * core::mem::size_of::<u16>() as ULONG) as usize,
    );

    Length = PathLength + PrefixLength - PrefixCut * core::mem::size_of::<u16>() as ULONG;
    LengthChars = Length / core::mem::size_of::<u16>() as ULONG;

    (*NtName).Buffer = NewBuffer;
    (*NtName).Length = Length as u16;
    (*NtName).MaximumLength = MaxLength as u16;
    *NewBuffer.offset(LengthChars as isize) = 0;

    if !PartName.is_null() && !(*PartName).is_null() {
        Status = RtlInitUnicodeStringEx(core::ptr::addr_of_mut!(PartNameString), *PartName);
        if NT_SUCCESS(Status) {
            *PartName = NewBuffer.offset((LengthChars - (PartNameString.Length / core::mem::size_of::<u16>() as u16) as ULONG) as isize) as PCWSTR;
        } else {
            RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer as PVOID);
            RtlReleasePebLock();
            return Status;
        }
    }

    if !RelativeName.is_null() {
        RtlInitEmptyUnicodeString(
            core::ptr::addr_of_mut!((*RelativeName).RelativeName),
            core::ptr::null_mut(),
            0,
        );
        (*RelativeName).ContainingDirectory = core::ptr::null_mut();
        (*RelativeName).CurDirRef = core::ptr::null_mut();

        if InputPathType == RtlPathTypeRelative {
            let Peb = (*(*NtCurrentTeb()).ProcessEnvironmentBlock).clone_marker_unused();
            let _ = Peb;
            CurrentDirectory = core::ptr::addr_of_mut!(
                (*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessParameters).CurrentDirectory
            );
            if !(*CurrentDirectory).Handle.is_null() {
                Status = RtlInitUnicodeStringEx(core::ptr::addr_of_mut!(FullPath), Buffer);
                if !NT_SUCCESS(Status) {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer as PVOID);
                    RtlReleasePebLock();
                    return Status;
                }

                if (*CurrentDirectory).DosPath.Length > FullPath.Length {
                    RtlReleasePebLock();
                    return Status;
                }

                if RtlEqualUnicodeString(
                    core::ptr::addr_of_mut!(FullPath),
                    core::ptr::addr_of_mut!((*CurrentDirectory).DosPath),
                    1,
                ) != 0
                {
                    (*RelativeName).RelativeName.Buffer = ((NewBuffer as usize)
                        + PrefixLength as usize
                        + FullPath.Length as usize
                        - PrefixCut as usize * core::mem::size_of::<u16>())
                        as PWSTR;
                    (*RelativeName).RelativeName.Length = (PathLength - FullPath.Length as ULONG) as u16;
                    if *(*RelativeName).RelativeName.Buffer.offset(0) == b'\\' as u16 {
                        (*RelativeName).RelativeName.Buffer = (*RelativeName).RelativeName.Buffer.offset(1);
                        (*RelativeName).RelativeName.Length -= core::mem::size_of::<u16>() as u16;
                    }
                    (*RelativeName).RelativeName.MaximumLength = (*RelativeName).RelativeName.Length;

                    if HaveRelative == 0 {
                        (*RelativeName).ContainingDirectory = (*CurrentDirectory).Handle;
                        return Status;
                    }

                    (*RelativeName).CurDirRef = RtlpCurDirRef;
                    if !(*RelativeName).CurDirRef.is_null() {
                        InterlockedIncrement(core::ptr::addr_of_mut!((*RtlpCurDirRef).RefCount));
                    }

                    (*RelativeName).ContainingDirectory = (*CurrentDirectory).Handle;
                }
            }
        }
    }

    RtlReleasePebLock();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDosPathNameToRelativeNtPathName_U(
    HaveRelative: BOOLEAN,
    DosName: PCWSTR,
    NtName: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> NTSTATUS {
    let mut NameString: _UNICODE_STRING = core::mem::zeroed();

    let mut Status = RtlInitUnicodeStringEx(&mut NameString, DosName);
    if NT_SUCCESS(Status) {
        Status = RtlpDosPathNameToRelativeNtPathName_Ustr(
            HaveRelative,
            &NameString,
            NtName,
            PartName,
            RelativeName,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlDosPathNameToRelativeNtPathName_Ustr(
    DosName: PCUNICODE_STRING,
    NtName: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> BOOLEAN {
    if NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_Ustr(
        1u8,
        DosName,
        NtName,
        PartName,
        RelativeName,
    )) {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlDoesFileExists_UstrEx(
    FileName: PCUNICODE_STRING,
    SucceedIfBusy: BOOLEAN,
) -> BOOLEAN {
    let mut Result: BOOLEAN;
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut NtPathName: _UNICODE_STRING = core::mem::zeroed();
    let Buffer: PVOID;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut BasicInformation: FILE_BASIC_INFORMATION = core::mem::zeroed();

    /* Get the NT Path */
    Result = RtlDosPathNameToRelativeNtPathName_Ustr(
        FileName,
        &mut NtPathName,
        core::ptr::null_mut(),
        &mut RelativeName,
    );
    if Result == 0 {
        return 0u8;
    }

    /* Save the buffer */
    Buffer = NtPathName.Buffer as PVOID;

    /* Check if we have a relative name */
    if RelativeName.RelativeName.Length != 0 {
        /* Use it */
        NtPathName = RelativeName.RelativeName;
    } else {
        /* Otherwise ignore it */
        RelativeName.ContainingDirectory = core::ptr::null_mut();
    }

    /* Initialize the object attributes */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut NtPathName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Query the attributes and free the buffer now */
    Status = ZwQueryAttributesFile(&mut ObjectAttributes, &mut BasicInformation);
    RtlReleaseRelativeName(&mut RelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);

    /* Check if we failed */
    if !NT_SUCCESS(Status) {
        /* Check if we failed because the file is in use */
        if Status == STATUS_SHARING_VIOLATION || Status == STATUS_ACCESS_DENIED {
            /* Check if the caller wants this to be considered OK */
            Result = if SucceedIfBusy != 0 { 1u8 } else { 0u8 };
        } else {
            /* A failure because the file didn't exist */
            Result = 0u8;
        }
    } else {
        /* The file exists */
        Result = 1u8;
    }

    /* Return the result */
    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlDoesFileExists_UStr(FileName: PUNICODE_STRING) -> BOOLEAN {
    RtlDoesFileExists_UstrEx(FileName, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn RtlDoesFileExists_UEx(
    FileName: PCWSTR,
    SucceedIfBusy: BOOLEAN,
) -> BOOLEAN {
    let mut NameString: _UNICODE_STRING = core::mem::zeroed();

    /* Create the unicode name*/
    if NT_SUCCESS(RtlInitUnicodeStringEx(&mut NameString, FileName)) {
        /* Call the unicode function */
        return RtlDoesFileExists_UstrEx(&NameString, SucceedIfBusy);
    }

    /* Fail */
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleaseRelativeName(RelativeName: PRTL_RELATIVE_NAME_U) {
    if !(*RelativeName).CurDirRef.is_null() {
        if InterlockedDecrement(core::ptr::addr_of_mut!((*(*RelativeName).CurDirRef).RefCount)) == 0 {
            NtClose((*(*RelativeName).CurDirRef).Handle);
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*RelativeName).CurDirRef as PVOID);
        }
        (*RelativeName).CurDirRef = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetLongestNtPathLength() -> ULONG {
    260 + (RtlpDosDevicesUncPrefix.Length as ULONG) / core::mem::size_of::<WCHAR>() as ULONG
        + core::mem::size_of::<CHAR>() as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetLengthWithoutTrailingPathSeparators(
    Flags: ULONG,
    PathString: PCUNICODE_STRING,
    Length: PULONG,
) -> NTSTATUS {
    let mut NumChars: ULONG;

    if Length.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Length = 0;

    if PathString.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if Flags != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    NumChars = (*PathString).Length as ULONG / core::mem::size_of::<u16>() as ULONG;

    while NumChars > 0 {
        let c = *(*PathString).Buffer.add((NumChars - 1) as usize);
        if c == '\\' as u16 || c == '/' as u16 {
            NumChars -= 1;
        } else {
            break;
        }
    }

    *Length = NumChars;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDetermineDosPathNameType_U(Path: PCWSTR) -> RTL_PATH_TYPE {
    let is_sep = |c: u16| c == b'\\' as u16 || c == b'/' as u16;

    if is_sep(*Path.offset(0)) {
        if !is_sep(*Path.offset(1)) {
            return RtlPathTypeRooted;
        }
        if *Path.offset(2) != b'.' as u16 && *Path.offset(2) != b'?' as u16 {
            return RtlPathTypeUncAbsolute;
        }
        if is_sep(*Path.offset(3)) {
            return RtlPathTypeLocalDevice;
        }
        if *Path.offset(3) != 0 {
            return RtlPathTypeUncAbsolute;
        }
        RtlPathTypeRootLocalDevice
    } else {
        if *Path.offset(0) == 0 || *Path.offset(1) != b':' as u16 {
            return RtlPathTypeRelative;
        }
        if is_sep(*Path.offset(2)) {
            return RtlPathTypeDriveAbsolute;
        }
        RtlPathTypeDriveRelative
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsDosDeviceName_U(Path: PCWSTR) -> ULONG {
    let mut PathString: _UNICODE_STRING = core::mem::zeroed();

    let Status = RtlInitUnicodeStringEx(&mut PathString, Path);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    RtlIsDosDeviceName_Ustr(&PathString)
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetCurrentDirectory_U(
    MaximumLength: ULONG,
    Buffer: PWSTR,
) -> ULONG {
    let mut Length: ULONG;
    let Bytes: ULONG;
    let CurDir: PCURDIR;
    let CurDirName: PWSTR;

    RtlAcquirePebLock();
    let Peb = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    CurDir = core::ptr::addr_of_mut!((*(*Peb).ProcessParameters).CurrentDirectory);

    CurDirName = (*CurDir).DosPath.Buffer;
    Length = (*CurDir).DosPath.Length as ULONG / core::mem::size_of::<u16>() as ULONG;

    Bytes = Length * core::mem::size_of::<u16>() as ULONG;
    if (Length <= 1) || (*CurDirName.offset((Length - 2) as isize) == b':' as u16) {
        if MaximumLength <= Bytes {
            RtlReleasePebLock();
            return Bytes + core::mem::size_of::<u16>() as ULONG;
        }
    } else {
        if MaximumLength < Bytes {
            RtlReleasePebLock();
            return Bytes;
        }
    }

    RtlCopyMemory(
        Buffer as *mut core::ffi::c_void,
        CurDirName as *const core::ffi::c_void,
        Bytes as usize,
    );

    if (Length <= 1) || (*Buffer.offset((Length - 2) as isize) != b':' as u16) {
        *Buffer.offset((Length - 1) as isize) = 0;
        Length -= 1;
    } else {
        *Buffer.offset(Length as isize) = 0;
    }

    RtlReleasePebLock();
    Length * core::mem::size_of::<u16>() as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetCurrentDirectory_U(Path: PUNICODE_STRING) -> NTSTATUS {
    let CurDir: PCURDIR;
    let mut Status: NTSTATUS;
    let mut PathType: RTL_PATH_TYPE = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FullPath: _UNICODE_STRING = core::mem::zeroed();
    let mut NtName: _UNICODE_STRING = core::mem::zeroed();
    let mut OldCurDir: PRTLP_CURDIR_REF = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileFsDeviceInfo: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();
    let SavedLength: ULONG;
    let CharLength: ULONG;
    let FullPathLength: ULONG;
    let mut OldHandle: HANDLE = core::ptr::null_mut();
    let mut CurDirHandle: HANDLE = core::ptr::null_mut();
    let mut OldCurDirHandle: HANDLE = core::ptr::null_mut();

    RtlInitEmptyUnicodeString(&mut NtName, core::ptr::null_mut(), 0);

    if RtlIsDosDeviceName_Ustr(Path) != 0 {
        return STATUS_NOT_A_DIRECTORY;
    }

    RtlAcquirePebLock();

    let Peb = (*(*NtCurrentTeb()).ProcessEnvironmentBlock_or_field()) as PPEB;
    let _ = Peb;
    let PebPtr = NtCurrentPeb();
    CurDir = core::ptr::addr_of_mut!((*(*PebPtr).ProcessParameters).CurrentDirectory);

    'leave_block: {
        if ((*CurDir).Handle as ULONG_PTR) & (RTL_CURDIR_ALL_FLAGS as ULONG_PTR)
            == RTL_CURDIR_DROP_OLD_HANDLE as ULONG_PTR
        {
            OldHandle = (*CurDir).Handle;
            (*CurDir).Handle = core::ptr::null_mut();
        }

        FullPath.Buffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            (*CurDir).DosPath.MaximumLength as SIZE_T,
        ) as PWSTR;
        if FullPath.Buffer.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'leave_block;
        }

        FullPath.Length = 0;
        FullPath.MaximumLength = (*CurDir).DosPath.MaximumLength;

        FullPathLength = RtlGetFullPathName_Ustr(
            Path,
            FullPath.MaximumLength as ULONG,
            FullPath.Buffer,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut PathType,
        );
        if FullPathLength == 0 {
            Status = STATUS_OBJECT_NAME_INVALID;
            break 'leave_block;
        }

        SavedLength = FullPath.MaximumLength as ULONG;
        CharLength = FullPathLength / core::mem::size_of::<u16>() as ULONG;

        if FullPathLength > FullPath.MaximumLength as ULONG {
            Status = STATUS_NAME_TOO_LONG;
            break 'leave_block;
        }

        if RtlDosPathNameToNtPathName_U(
            FullPath.Buffer,
            &mut NtName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            Status = STATUS_OBJECT_NAME_INVALID;
            break 'leave_block;
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_INHERIT) as ULONG;
        ObjectAttributes.ObjectName = &mut NtName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        if ((*CurDir).Handle as ULONG_PTR) & (RTL_CURDIR_ALL_FLAGS as ULONG_PTR)
            == RTL_CURDIR_ALL_FLAGS as ULONG_PTR
        {
            CurDirHandle =
                (((*CurDir).Handle as ULONG_PTR) & !(RTL_CURDIR_ALL_FLAGS as ULONG_PTR)) as HANDLE;
            (*CurDir).Handle = core::ptr::null_mut();

            Status = NtQueryVolumeInformationFile(
                CurDirHandle,
                &mut IoStatusBlock,
                &mut FileFsDeviceInfo as *mut _ as PVOID,
                core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
                FileFsDeviceInformation,
            );
            if !NT_SUCCESS(Status) {
                Status = RtlSetCurrentDirectory_U(Path);
                break 'leave_block;
            }
        } else {
            Status = NtOpenFile(
                &mut CurDirHandle,
                SYNCHRONIZE | FILE_TRAVERSE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
            );
            if !NT_SUCCESS(Status) {
                break 'leave_block;
            }

            Status = NtQueryVolumeInformationFile(
                CurDirHandle,
                &mut IoStatusBlock,
                &mut FileFsDeviceInfo as *mut _ as PVOID,
                core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
                FileFsDeviceInformation,
            );
            if !NT_SUCCESS(Status) {
                break 'leave_block;
            }
        }

        if FileFsDeviceInfo.Characteristics & FILE_REMOVABLE_MEDIA != 0 {
            CurDirHandle =
                ((CurDirHandle as ULONG_PTR) | (RTL_CURDIR_IS_REMOVABLE as ULONG_PTR)) as HANDLE;
        }

        FullPath.Length = FullPathLength as USHORT;

        if *FullPath.Buffer.offset((CharLength - 1) as isize) != OBJ_NAME_PATH_SEPARATOR as u16 {
            if (CharLength + 1) * core::mem::size_of::<u16>() as ULONG > SavedLength {
                Status = STATUS_NAME_TOO_LONG;
                break 'leave_block;
            }

            *FullPath.Buffer.offset(CharLength as isize) = OBJ_NAME_PATH_SEPARATOR as u16;
            *FullPath.Buffer.offset((CharLength + 1) as isize) = 0;
            FullPath.Length += core::mem::size_of::<u16>() as USHORT;
        }

        if !RtlpCurDirRef.is_null() && (*RtlpCurDirRef).RefCount == 1 {
            OldCurDirHandle = (*RtlpCurDirRef).Handle;
        } else {
            OldCurDir = RtlpCurDirRef;
            RtlpCurDirRef = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                core::mem::size_of::<RTLP_CURDIR_REF>() as SIZE_T,
            ) as PRTLP_CURDIR_REF;
            if RtlpCurDirRef.is_null() {
                RtlpCurDirRef = OldCurDir;
                OldCurDir = core::ptr::null_mut();
                Status = STATUS_NO_MEMORY;
                break 'leave_block;
            }

            (*RtlpCurDirRef).RefCount = 1;
        }

        (*CurDir).Handle = CurDirHandle;
        (*RtlpCurDirRef).Handle = CurDirHandle;
        CurDirHandle = core::ptr::null_mut();

        core::ptr::copy_nonoverlapping(
            FullPath.Buffer,
            (*CurDir).DosPath.Buffer,
            (FullPath.Length as usize + core::mem::size_of::<u16>()) / core::mem::size_of::<u16>(),
        );
        (*CurDir).DosPath.Length = FullPath.Length;

        Status = STATUS_SUCCESS;
    }

    RtlReleasePebLock();

    if !FullPath.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FullPath.Buffer as PVOID);
    }

    if !NtName.Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NtName.Buffer as PVOID);
    }

    if !CurDirHandle.is_null() {
        NtClose(CurDirHandle);
    }

    if !OldHandle.is_null() {
        NtClose(OldHandle);
    }

    if !OldCurDirHandle.is_null() {
        NtClose(OldCurDirHandle);
    }

    if !OldCurDir.is_null()
        && InterlockedDecrement(core::ptr::addr_of_mut!((*OldCurDir).RefCount)) == 0
    {
        NtClose((*OldCurDir).Handle);
        RtlFreeHeap(RtlGetProcessHeap(), 0, OldCurDir as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetFullPathName_UEx(
    FileName: PWSTR,
    BufferLength: ULONG,
    Buffer: PWSTR,
    FilePart: *mut PWSTR,
    InputPathType: *mut RTL_PATH_TYPE,
) -> ULONG {
    let mut FileNameString: _UNICODE_STRING = core::mem::zeroed();
    let status: NTSTATUS;

    if !InputPathType.is_null() {
        *InputPathType = 0;
    }

    status = RtlInitUnicodeStringEx(&mut FileNameString, FileName);
    if !NT_SUCCESS(status) {
        return 0;
    }

    RtlGetFullPathName_Ustr(
        &mut FileNameString,
        BufferLength,
        Buffer,
        FilePart as *mut PCWSTR,
        core::ptr::null_mut(),
        InputPathType,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetFullPathName_U(
    FileName: PCWSTR,
    Size: ULONG,
    Buffer: PWSTR,
    ShortName: *mut PWSTR,
) -> ULONG {
    let mut PathType: RTL_PATH_TYPE = 0;

    RtlGetFullPathName_UEx(
        FileName as PWSTR,
        Size,
        Buffer,
        ShortName,
        &mut PathType,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlDosPathNameToNtPathName_U(
    DosName: PCWSTR,
    NtName: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> u8 {
    if NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_U(
        0u8,
        DosName,
        NtName,
        PartName,
        RelativeName,
    )) {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlDosPathNameToNtPathName_U_WithStatus(
    DosName: PCWSTR,
    NtName: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> NTSTATUS {
    RtlpDosPathNameToRelativeNtPathName_U(0u8, DosName, NtName, PartName, RelativeName)
}

#[no_mangle]
pub unsafe extern "system" fn RtlDosPathNameToRelativeNtPathName_U(
    DosName: PCWSTR,
    NtName: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> BOOLEAN {
    if NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_U(
        1u8,
        DosName,
        NtName,
        PartName,
        RelativeName,
    )) {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlDosPathNameToRelativeNtPathName_U_WithStatus(
    DosName: PCWSTR,
    NtName: PUNICODE_STRING,
    PartName: *mut PCWSTR,
    RelativeName: PRTL_RELATIVE_NAME_U,
) -> NTSTATUS {
    RtlpDosPathNameToRelativeNtPathName_U(1u8, DosName, NtName, PartName, RelativeName)
}

#[no_mangle]
pub unsafe extern "system" fn RtlNtPathNameToDosPathName(
    Flags: ULONG,
    Path: PRTL_UNICODE_STRING_BUFFER,
    PathType: PULONG,
    _Unknown: PULONG,
) -> NTSTATUS {
    let mut UsePrefix: PCUNICODE_STRING = core::ptr::null();
    let mut AlternatePrefix: PCUNICODE_STRING = core::ptr::null();

    if !PathType.is_null() {
        *PathType = 0;
    }

    if Path.is_null() || Flags != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if RtlPrefixUnicodeString(
        core::ptr::addr_of!(RtlpDosDevicesUncPrefix),
        core::ptr::addr_of_mut!((*Path).String),
        1u8,
    ) != 0
    {
        UsePrefix = core::ptr::addr_of!(RtlpDosDevicesUncPrefix);
        AlternatePrefix = core::ptr::addr_of!(RtlpDoubleSlashPrefix);
        if !PathType.is_null() {
            *PathType = RTL_CONVERTED_UNC_PATH;
        }
    } else if RtlPrefixUnicodeString(
        core::ptr::addr_of!(RtlpDosDevicesPrefix),
        core::ptr::addr_of_mut!((*Path).String),
        0u8,
    ) != 0
    {
        UsePrefix = core::ptr::addr_of!(RtlpDosDevicesPrefix);
        if !PathType.is_null() {
            *PathType = RTL_CONVERTED_NT_PATH;
        }
    }

    if !UsePrefix.is_null() {
        let mut Len: USHORT = (*Path).String.Length - (*UsePrefix).Length;
        if !AlternatePrefix.is_null() {
            Len += (*AlternatePrefix).Length;
        }

        let Status = RtlEnsureBufferSize(0, core::ptr::addr_of_mut!((*Path).ByteBuffer), Len as ULONG);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if (Len as ULONG) + (core::mem::size_of::<u16>() as ULONG) <= (*Path).ByteBuffer.Size {
            if !AlternatePrefix.is_null() {
                memcpy(
                    (*Path).ByteBuffer.Buffer as *mut _,
                    (*AlternatePrefix).Buffer as *const _,
                    (*AlternatePrefix).Length as usize,
                );
                memmove(
                    (*Path).ByteBuffer.Buffer.offset((*AlternatePrefix).Length as isize) as *mut _,
                    (*Path).ByteBuffer.Buffer.offset((*UsePrefix).Length as isize) as *const _,
                    (Len - (*AlternatePrefix).Length) as usize,
                );
            } else {
                memmove(
                    (*Path).ByteBuffer.Buffer as *mut _,
                    (*Path).ByteBuffer.Buffer.offset((*UsePrefix).Length as isize) as *const _,
                    Len as usize,
                );
            }
            (*Path).String.Buffer = (*Path).ByteBuffer.Buffer as PWSTR;
            (*Path).String.Length = Len;
            (*Path).String.MaximumLength = (*Path).ByteBuffer.Size as USHORT;
            *(*Path).String.Buffer.offset((Len / core::mem::size_of::<u16>() as USHORT) as isize) = 0;
        }
        return STATUS_SUCCESS;
    }

    if !PathType.is_null() {
        match RtlDetermineDosPathNameType_Ustr(core::ptr::addr_of!((*Path).String)) {
            x if x == RtlPathTypeUncAbsolute
                || x == RtlPathTypeDriveAbsolute
                || x == RtlPathTypeLocalDevice
                || x == RtlPathTypeRootLocalDevice =>
            {
                *PathType = RTL_UNCHANGED_DOS_PATH;
            }
            x if x == RtlPathTypeUnknown
                || x == RtlPathTypeDriveRelative
                || x == RtlPathTypeRooted
                || x == RtlPathTypeRelative =>
            {
                *PathType = RTL_UNCHANGED_UNK_PATH;
            }
            _ => {}
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDosSearchPath_U(
    Path: PCWSTR,
    FileName: PCWSTR,
    mut Extension: PCWSTR,
    Size: ULONG,
    Buffer: PWSTR,
    PartName: *mut PWSTR,
) -> ULONG {
    let mut Status: NTSTATUS;
    let ExtensionLength: ULONG;
    let mut Length: ULONG;
    let FileNameLength: ULONG;
    let PathLength: ULONG;
    let mut TempString: _UNICODE_STRING = core::mem::zeroed();
    let NewBuffer: PWSTR;
    let mut BufferStart: PWSTR;
    let mut p: PCWSTR;
    let mut Path = Path;

    /* Check if this is an absolute path */
    if RtlDetermineDosPathNameType_U(FileName) != RtlPathTypeRelative {
        /* Check if the file exists */
        if RtlDoesFileExists_UEx(FileName, 1u8) != 0 {
            /* Get the full name, which does the DOS lookup */
            return RtlGetFullPathName_U(FileName, Size, Buffer, PartName);
        }

        /* Doesn't exist, so fail */
        return 0;
    }

    /* Scan the filename */
    p = FileName;
    while *p != 0 {
        /* Looking for an extension */
        if *p == b'.' as u16 {
            /* No extension string needed -- it's part of the filename */
            Extension = core::ptr::null_mut();
            break;
        }

        /* Next character */
        p = p.offset(1);
    }

    /* Do we have an extension? */
    if Extension.is_null() {
        /* Nope, don't worry about one */
        ExtensionLength = 0;
    } else {
        /* Build a temporary string to get the extension length */
        Status = RtlInitUnicodeStringEx(&mut TempString, Extension);
        if !NT_SUCCESS(Status) {
            return 0;
        }
        ExtensionLength = TempString.Length as ULONG;
    }

    /* Build a temporary string to get the path length */
    Status = RtlInitUnicodeStringEx(&mut TempString, Path);
    if !NT_SUCCESS(Status) {
        return 0;
    }
    PathLength = TempString.Length as ULONG;

    /* Build a temporary string to get the filename length */
    Status = RtlInitUnicodeStringEx(&mut TempString, FileName);
    if !NT_SUCCESS(Status) {
        return 0;
    }
    FileNameLength = TempString.Length as ULONG;

    /* Allocate the buffer for the new string name */
    NewBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (FileNameLength
            + ExtensionLength
            + PathLength
            + 3 * core::mem::size_of::<u16>() as ULONG) as usize,
    ) as PWSTR;
    if NewBuffer.is_null() {
        /* Fail the call */
        return 0;
    }

    /* Final loop to build the path */
    loop {
        /* Check if we have a valid character */
        BufferStart = NewBuffer;
        if *Path != 0 {
            /* Loop as long as there's no semicolon */
            while *Path != b';' as u16 {
                /* Copy the next character */
                *BufferStart = *Path;
                BufferStart = BufferStart.offset(1);
                Path = Path.offset(1);
                if *Path == 0 {
                    break;
                }
            }

            /* We found a semi-colon, to stop path processing on this loop */
            if *Path == b';' as u16 {
                Path = Path.offset(1);
            }
        }

        /* Add a terminating slash if needed */
        if BufferStart != NewBuffer && *BufferStart.offset(-1) != b'\\' as u16 {
            *BufferStart = b'\\' as u16;
            BufferStart = BufferStart.offset(1);
        }

        /* Bail out if we reached the end */
        if *Path == 0 {
            Path = core::ptr::null_mut();
        }

        /* Copy the file name and check if an extension is needed */
        core::ptr::copy_nonoverlapping(
            FileName as *const u8,
            BufferStart as *mut u8,
            FileNameLength as usize,
        );
        if ExtensionLength != 0 {
            /* Copy the extension too */
            core::ptr::copy_nonoverlapping(
                Extension as *const u8,
                (BufferStart as *mut u8).add(FileNameLength as usize),
                (ExtensionLength + core::mem::size_of::<u16>() as ULONG) as usize,
            );
        } else {
            /* Just NULL-terminate */
            *((BufferStart as *mut u8).add(FileNameLength as usize) as *mut u16) = 0;
        }

        /* Now, does this file exist? */
        if RtlDoesFileExists_UEx(NewBuffer, 0u8) != 0 {
            /* Call the full-path API to get the length */
            Length = RtlGetFullPathName_U(NewBuffer, Size, Buffer, PartName);
            break;
        }

        /* If we got here, path doesn't exist, so fail the call */
        Length = 0;
        if Path.is_null() {
            break;
        }
    }

    /* Free the allocation and return the length */
    RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer as PVOID);
    Length
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetFullPathName_UstrEx(
    FileName: PUNICODE_STRING,
    StaticString: PUNICODE_STRING,
    DynamicString: PUNICODE_STRING,
    StringUsed: *mut PUNICODE_STRING,
    FilePartSize: PSIZE_T,
    NameInvalid: PBOOLEAN,
    PathType: *mut RTL_PATH_TYPE,
    LengthNeeded: PSIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut StaticBuffer: PWCHAR;
    let mut ShortName: PCWCH = core::ptr::null_mut();
    let mut Length: ULONG;
    let StaticLength: USHORT;
    let mut TempDynamicString: _UNICODE_STRING = core::mem::zeroed();

    TempDynamicString.Buffer = core::ptr::null_mut();

    if !StringUsed.is_null() {
        *StringUsed = core::ptr::null_mut();
    }
    if !LengthNeeded.is_null() {
        *LengthNeeded = 0;
    }
    if !FilePartSize.is_null() {
        *FilePartSize = 0;
    }

    if !DynamicString.is_null() && StringUsed.is_null() && !StaticString.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if StaticString.is_null() {
        StaticLength = (260 * core::mem::size_of::<u16>()) as USHORT;
        StaticBuffer = RtlpAllocateStringMemory(260 * core::mem::size_of::<u16>() as u32, TAG_USTR);
        if StaticBuffer.is_null() {
            return STATUS_NO_MEMORY;
        }
    } else {
        StaticBuffer = (*StaticString).Buffer;
        StaticLength = (*StaticString).MaximumLength;
    }

    Length = RtlGetFullPathName_Ustr(
        FileName,
        StaticLength as ULONG,
        StaticBuffer,
        &mut ShortName,
        NameInvalid,
        PathType,
    );

    'quickie: {
        if Length == 0 {
            DbgPrint(b"%s(%d) - RtlGetFullPathName_Ustr() returned 0\n\0".as_ptr() as *const i8);
            Status = STATUS_OBJECT_NAME_INVALID;
            break 'quickie;
        }

        if !StaticString.is_null() && Length < StaticLength as ULONG {
            (*StaticString).Length = Length as USHORT;
            if !FilePartSize.is_null() {
                *FilePartSize = if !ShortName.is_null() {
                    (ShortName as usize - (*StaticString).Buffer as usize)
                        / core::mem::size_of::<u16>()
                } else {
                    0
                };
            }
            if !StringUsed.is_null() {
                *StringUsed = StaticString;
            }
            Status = STATUS_SUCCESS;
            break 'quickie;
        }

        if DynamicString.is_null() {
            if !LengthNeeded.is_null() {
                *LengthNeeded = Length as SIZE_T;
            }
            Status = STATUS_BUFFER_TOO_SMALL;
            break 'quickie;
        }

        if !StaticBuffer.is_null() && Length < StaticLength as ULONG {
            *StaticBuffer.add(Length as usize / core::mem::size_of::<u16>()) = 0;
            (*DynamicString).MaximumLength = StaticLength;
            (*DynamicString).Length = Length as USHORT;
            (*DynamicString).Buffer = StaticBuffer;
            if !FilePartSize.is_null() {
                *FilePartSize = if !ShortName.is_null() {
                    (ShortName as usize - StaticBuffer as usize) / core::mem::size_of::<u16>()
                } else {
                    0
                };
            }
            if !StringUsed.is_null() {
                *StringUsed = DynamicString;
            }
            StaticBuffer = core::ptr::null_mut();
            Status = STATUS_SUCCESS;
            break 'quickie;
        }

        RtlAcquirePebLock();

        'release: {
            Length = RtlGetFullPathName_Ustr(
                FileName,
                StaticLength as ULONG,
                StaticBuffer,
                &mut ShortName,
                NameInvalid,
                PathType,
            );
            if Length == 0 {
                DbgPrint(b"%s line %d: RtlGetFullPathName_Ustr() returned 0\n\0".as_ptr() as *const i8);
                Status = STATUS_OBJECT_NAME_INVALID;
                break 'release;
            }

            if !StaticString.is_null() && Length < StaticLength as ULONG {
                (*StaticString).Length = Length as USHORT;
                if !FilePartSize.is_null() {
                    *FilePartSize = if !ShortName.is_null() {
                        (ShortName as usize - (*StaticString).Buffer as usize)
                            / core::mem::size_of::<u16>()
                    } else {
                        0
                    };
                }
                if !StringUsed.is_null() {
                    *StringUsed = StaticString;
                }
                Status = STATUS_SUCCESS;
                break 'release;
            }

            if (Length as usize + core::mem::size_of::<u16>()) > 0xfffe {
                Status = STATUS_NAME_TOO_LONG;
                break 'release;
            }

            TempDynamicString.Buffer = RtlpAllocateStringMemory(
                Length + core::mem::size_of::<u16>() as u32,
                TAG_USTR,
            );
            if TempDynamicString.Buffer.is_null() {
                Status = STATUS_NO_MEMORY;
                break 'release;
            }

            TempDynamicString.MaximumLength = Length as USHORT + core::mem::size_of::<u16>() as USHORT;
            Length = RtlGetFullPathName_Ustr(
                FileName,
                Length,
                TempDynamicString.Buffer,
                &mut ShortName,
                NameInvalid,
                PathType,
            );
            if Length == 0 {
                DbgPrint(b"%s line %d: RtlGetFullPathName_Ustr() returned 0\n\0".as_ptr() as *const i8);
                Status = STATUS_OBJECT_NAME_INVALID;
                break 'release;
            }

            if Length > TempDynamicString.MaximumLength as ULONG {
                Status = STATUS_INTERNAL_ERROR;
                break 'release;
            }

            if !FilePartSize.is_null() {
                *FilePartSize = if !ShortName.is_null() {
                    (ShortName as usize - TempDynamicString.Buffer as usize)
                        / core::mem::size_of::<u16>()
                } else {
                    0
                };
            }

            *TempDynamicString
                .Buffer
                .add(Length as usize / core::mem::size_of::<u16>()) = 0;

            (*DynamicString).Buffer = TempDynamicString.Buffer;
            (*DynamicString).Length = Length as USHORT;
            (*DynamicString).MaximumLength = TempDynamicString.MaximumLength;
            if !StringUsed.is_null() {
                *StringUsed = DynamicString;
            }

            TempDynamicString.Buffer = core::ptr::null_mut();
            Status = STATUS_SUCCESS;
        }

        RtlReleasePebLock();
    }

    if !StaticString.is_null()
        && !StaticBuffer.is_null()
        && StaticBuffer != (*StaticString).Buffer
    {
        RtlpFreeStringMemory(StaticBuffer as *mut core::ffi::c_void, TAG_USTR);
    }
    if !TempDynamicString.Buffer.is_null() {
        RtlpFreeStringMemory(TempDynamicString.Buffer as *mut core::ffi::c_void, TAG_USTR);
    }

    if (Status as u32 & 0xC0000000) == 0xC0000000
        && Status != STATUS_NO_SUCH_FILE
        && Status != STATUS_BUFFER_TOO_SMALL
    {
        DbgPrint(b"RTL: %s - failing on filename %wZ with status %08lx\n\0".as_ptr() as *const i8);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlDoesFileExists_U(FileName: PCWSTR) -> BOOLEAN {
    RtlDoesFileExists_UEx(FileName, 1u8)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/rtl/path.c
//  * PURPOSE:         Path and current directory functions
//  * PROGRAMMERS:     Wine team
//  *                  Thomas Weidenmueller
//  *                  Gunnar Dalsnes
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <rtl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* DEFINITONS and MACROS ******************************************************/
//
// #define MAX_PFX_SIZE       16
//
// #define IS_PATH_SEPARATOR(x) (((x)==L'\\')||((x)==L'/'))
//
// #define RTL_CURDIR_IS_REMOVABLE 0x1
// #define RTL_CURDIR_DROP_OLD_HANDLE 0x2
// #define RTL_CURDIR_ALL_FLAGS (RTL_CURDIR_DROP_OLD_HANDLE | RTL_CURDIR_IS_REMOVABLE) // 0x3
// C_ASSERT(RTL_CURDIR_ALL_FLAGS == OBJ_HANDLE_TAGBITS);
//
//
// /* GLOBALS ********************************************************************/
//
// const UNICODE_STRING DeviceRootString = RTL_CONSTANT_STRING(L"\\\\.\\");
//
// const UNICODE_STRING RtlpDosDevicesUncPrefix = RTL_CONSTANT_STRING(L"\\??\\UNC\\");
// const UNICODE_STRING RtlpWin32NtRootSlash    = RTL_CONSTANT_STRING(L"\\\\?\\");
// const UNICODE_STRING RtlpDosSlashCONDevice   = RTL_CONSTANT_STRING(L"\\\\.\\CON");
// const UNICODE_STRING RtlpDosDevicesPrefix    = RTL_CONSTANT_STRING(L"\\??\\");
//
// const UNICODE_STRING RtlpDosLPTDevice = RTL_CONSTANT_STRING(L"LPT");
// const UNICODE_STRING RtlpDosCOMDevice = RTL_CONSTANT_STRING(L"COM");
// const UNICODE_STRING RtlpDosPRNDevice = RTL_CONSTANT_STRING(L"PRN");
// const UNICODE_STRING RtlpDosAUXDevice = RTL_CONSTANT_STRING(L"AUX");
// const UNICODE_STRING RtlpDosCONDevice = RTL_CONSTANT_STRING(L"CON");
// const UNICODE_STRING RtlpDosNULDevice = RTL_CONSTANT_STRING(L"NUL");
//
// const UNICODE_STRING RtlpDoubleSlashPrefix   = RTL_CONSTANT_STRING(L"\\\\");
//
// static const UNICODE_STRING RtlpDefaultExtension = RTL_CONSTANT_STRING(L".DLL");
// static const UNICODE_STRING RtlpDotLocal = RTL_CONSTANT_STRING(L".Local\\");
// static const UNICODE_STRING RtlpPathDividers = RTL_CONSTANT_STRING(L"\\/");
//
//
// PRTLP_CURDIR_REF RtlpCurDirRef;
//
// /* PATH-1 Rust forward decls — the bodies of these 6 fns live in
//  * sdk/lib/rust/rtl-strings/ cabi (Wave PATH-1). They're #if 0'd
//  * out below; the rest of path.c calls them through these decls. */
// RTL_PATH_TYPE NTAPI RtlDetermineDosPathNameType_Ustr(IN PCUNICODE_STRING PathString);
// ULONG NTAPI RtlIsDosDeviceName_Ustr(IN PCUNICODE_STRING PathString);
// ULONG NTAPI RtlGetLongestNtPathLength(VOID);
// NTSTATUS NTAPI RtlGetLengthWithoutTrailingPathSeparators(IN ULONG Flags,
//                                                         IN PCUNICODE_STRING PathString,
//                                                         OUT PULONG Length);
// RTL_PATH_TYPE NTAPI RtlDetermineDosPathNameType_U(IN PCWSTR Path);
// ULONG NTAPI RtlIsDosDeviceName_U(IN PCWSTR Path);
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// #if 0 /* Replaced by Rust (sdk/lib/rust/rtl-strings/ cabi, Wave PATH-1) */
// RTL_PATH_TYPE
// NTAPI
// RtlDetermineDosPathNameType_Ustr(IN PCUNICODE_STRING PathString)
// {
//     PWCHAR Path;
//     ULONG Chars;
//
//     Path = PathString->Buffer;
//     Chars = PathString->Length / sizeof(WCHAR);
//
//     /* Return if there are no characters */
//     if (!Chars) return RtlPathTypeRelative;
//
//     /*
//      * The algorithm is similar to RtlDetermineDosPathNameType_U but here we
//      * actually check for the path length before touching the characters
//      */
//     if (IS_PATH_SEPARATOR(Path[0]))
//     {
//         if ((Chars < 2) || !(IS_PATH_SEPARATOR(Path[1]))) return RtlPathTypeRooted;                /* \x             */
//         if ((Chars < 3) || ((Path[2] != L'.') && (Path[2] != L'?'))) return RtlPathTypeUncAbsolute;/* \\x            */
//         if ((Chars >= 4) && (IS_PATH_SEPARATOR(Path[3]))) return RtlPathTypeLocalDevice;           /* \\.\x or \\?\x */
//         if (Chars != 3) return RtlPathTypeUncAbsolute;                                             /* \\.x or \\?x   */
//         return RtlPathTypeRootLocalDevice;                                                         /* \\. or \\?     */
//     }
//     else
//     {
//         if ((Chars < 2) || (Path[1] != L':')) return RtlPathTypeRelative;                          /* x              */
//         if ((Chars < 3) || !(IS_PATH_SEPARATOR(Path[2]))) return RtlPathTypeDriveRelative;         /* x:             */
//         return RtlPathTypeDriveAbsolute;                                                           /* x:\            */
//     }
// }
//
// ULONG
// NTAPI
// RtlIsDosDeviceName_Ustr(IN PCUNICODE_STRING PathString)
// {
//     UNICODE_STRING PathCopy;
//     PWCHAR Start, End;
//     USHORT PathChars, ColonCount = 0;
//     USHORT ReturnOffset = 0, ReturnLength, OriginalLength;
//     WCHAR c;
//
//     /* Validate the input */
//     if (!PathString) return 0;
//
//     /* Check what type of path this is */
//     switch (RtlDetermineDosPathNameType_Ustr(PathString))
//     {
//         /* Fail for UNC or unknown paths */
//         case RtlPathTypeUnknown:
//         case RtlPathTypeUncAbsolute:
//             return 0;
//
//         /* Make special check for the CON device */
//         case RtlPathTypeLocalDevice:
//             if (RtlEqualUnicodeString(PathString, &RtlpDosSlashCONDevice, TRUE))
//             {
//                 /* This should return 0x80006 */
//                 return MAKELONG(RtlpDosCONDevice.Length, DeviceRootString.Length);
//             }
//             return 0;
//
//         default:
//             break;
//     }
//
//     /* Make a copy of the string */
//     PathCopy = *PathString;
//     OriginalLength = PathString->Length;
//
//     /* Return if there's no characters */
//     PathChars = PathCopy.Length / sizeof(WCHAR);
//     if (!PathChars) return 0;
//
//     /* Check for drive path and truncate */
//     if (PathCopy.Buffer[PathChars - 1] == L':')
//     {
//         /* Fixup the lengths */
//         PathCopy.Length -= sizeof(WCHAR);
//         if (!--PathChars) return 0;
//
//         /* Remember this for later */
//         ColonCount = 1;
//     }
//
//     /* Check for extension or space, and truncate */
//     do
//     {
//         /* Stop if we hit something else than a space or period */
//         c = PathCopy.Buffer[PathChars - 1];
//         if ((c != L'.') && (c != L' ')) break;
//
//         /* Fixup the lengths */
//         PathCopy.Length -= sizeof(WCHAR);
//
//         /* Remember this for later */
//         ColonCount++;
//     } while (--PathChars);
//
//     /* Anything still left? */
//     if (PathChars)
//     {
//         /* Loop from the end */
//         for (End = &PathCopy.Buffer[PathChars - 1];
//              End >= PathCopy.Buffer;
//              --End)
//         {
//             /* Check if the character is a path or drive separator */
//             c = *End;
//             if (IS_PATH_SEPARATOR(c) || ((c == L':') && (End == PathCopy.Buffer + 1)))
//             {
//                 /* Get the next lower case character */
//                 End++;
//                 c = RtlpDowncaseUnicodeChar(*End);
//
//                 /* Check if it's a DOS device (LPT, COM, PRN, AUX, or NUL) */
//                 if ((End < &PathCopy.Buffer[OriginalLength / sizeof(WCHAR)]) &&
//                     ((c == L'l') || (c == L'c') || (c == L'p') || (c == L'a') || (c == L'n')))
//                 {
//                     /* Calculate the offset */
//                     ReturnOffset = (USHORT)((PCHAR)End - (PCHAR)PathCopy.Buffer);
//
//                     /* Build the final string */
//                     PathCopy.Length = OriginalLength - ReturnOffset - (ColonCount * sizeof(WCHAR));
//                     PathCopy.Buffer = End;
//
//                     /* Save new amount of chars in the path */
//                     PathChars = PathCopy.Length / sizeof(WCHAR);
//
//                     break;
//                 }
//                 else
//                 {
//                     return 0;
//                 }
//             }
//         }
//
//         /* Get the next lower case character and check if it's a DOS device */
//         c = RtlpDowncaseUnicodeChar(*PathCopy.Buffer);
//         if ((c != L'l') && (c != L'c') && (c != L'p') && (c != L'a') && (c != L'n'))
//         {
//             /* Not LPT, COM, PRN, AUX, or NUL */
//             return 0;
//         }
//     }
//
//     /* Now skip past any extra extension or drive letter characters */
//     Start = PathCopy.Buffer;
//     End = &Start[PathChars];
//     while (Start < End)
//     {
//         c = *Start;
//         if ((c == L'.') || (c == L':')) break;
//         Start++;
//     }
//
//     /* And then go backwards to get rid of spaces */
//     while ((Start > PathCopy.Buffer) && (Start[-1] == L' ')) --Start;
//
//     /* Finally see how many characters are left, and that's our size */
//     PathChars = (USHORT)(Start - PathCopy.Buffer);
//     PathCopy.Length = PathChars * sizeof(WCHAR);
//
//     /* Check if this is a COM or LPT port, which has a digit after it */
//     if ((PathChars == 4) &&
//         (iswdigit(PathCopy.Buffer[3]) && (PathCopy.Buffer[3] != L'0')))
//     {
//         /* Don't compare the number part, just check for LPT or COM */
//         PathCopy.Length -= sizeof(WCHAR);
//         if ((RtlEqualUnicodeString(&PathCopy, &RtlpDosLPTDevice, TRUE)) ||
//             (RtlEqualUnicodeString(&PathCopy, &RtlpDosCOMDevice, TRUE)))
//         {
//             /* Found it */
//             ReturnLength = sizeof(L"COM1") - sizeof(WCHAR);
//             return MAKELONG(ReturnLength, ReturnOffset);
//         }
//     }
//     else if ((PathChars == 3) &&
//              ((RtlEqualUnicodeString(&PathCopy, &RtlpDosPRNDevice, TRUE)) ||
//               (RtlEqualUnicodeString(&PathCopy, &RtlpDosAUXDevice, TRUE)) ||
//               (RtlEqualUnicodeString(&PathCopy, &RtlpDosNULDevice, TRUE)) ||
//               (RtlEqualUnicodeString(&PathCopy, &RtlpDosCONDevice, TRUE))))
//     {
//         /* Otherwise this was something like AUX, NUL, PRN, or CON */
//         ReturnLength = sizeof(L"AUX") - sizeof(WCHAR);
//         return MAKELONG(ReturnLength, ReturnOffset);
//     }
//
//     /* Otherwise, this is not a valid DOS device */
//     return 0;
// }
// #endif /* PATH-1 Rust override: RtlDetermineDosPathNameType_Ustr + RtlIsDosDeviceName_Ustr */
//
// NTSTATUS
// NTAPI
// RtlpCheckDeviceName(IN PUNICODE_STRING FileName,
//                     IN ULONG Length,
//                     OUT PBOOLEAN NameInvalid)
// {
//     PWCHAR Buffer;
//     NTSTATUS Status;
//
//     /* Allocate a large enough buffer */
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, FileName->Length);
//     if (Buffer)
//     {
//         /* Assume failure */
//         *NameInvalid = TRUE;
//
//         /* Copy the filename */
//         RtlCopyMemory(Buffer, FileName->Buffer, FileName->Length);
//
//         /* And add a dot at the end */
//         Buffer[Length / sizeof(WCHAR)] = L'.';
//         Buffer[(Length / sizeof(WCHAR)) + 1] = UNICODE_NULL;
//
//         /* Check if the file exists or not */
//         *NameInvalid = RtlDoesFileExists_U(Buffer) ? FALSE: TRUE;
//
//         /* Get rid of the buffer now */
//         Status = RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//     }
//     else
//     {
//         /* Assume the name is ok, but fail the call */
//         *NameInvalid = FALSE;
//         Status = STATUS_NO_MEMORY;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
//
//
// /******************************************************************
//  *    RtlpCollapsePath (from WINE)
//  *
//  * Helper for RtlGetFullPathName_U
//  *
//  * 1) Converts slashes into backslashes and gets rid of duplicated ones;
//  * 2) Gets rid of . and .. components in the path.
//  *
//  * Returns the full path length without its terminating NULL character.
//  */
// static ULONG
// RtlpCollapsePath(PWSTR Path, /* ULONG PathBufferSize, ULONG PathLength, */ ULONG mark, BOOLEAN SkipTrailingPathSeparators)
// {
//     PWSTR p, next;
//
//     // FIXME: Do not suppose NULL-terminated strings!!
//
//     SIZE_T PathLength = wcslen(Path);
//     PWSTR EndBuffer = Path + PathLength; // Path + PathBufferSize / sizeof(WCHAR);
//     PWSTR EndPath;
//
//     /* Convert slashes into backslashes */
//     for (p = Path; *p; p++)
//     {
//         if (*p == L'/') *p = L'\\';
//     }
//
//     /* Collapse duplicate backslashes */
//     next = Path + max( 1, mark );
//     for (p = next; *p; p++)
//     {
//         if (*p != L'\\' || next[-1] != L'\\') *next++ = *p;
//     }
//     *next = UNICODE_NULL;
//     EndPath = next;
//
//     p = Path + mark;
//     while (*p)
//     {
//         if (*p == L'.')
//         {
//             switch (p[1])
//             {
//                 case UNICODE_NULL:  /* final . */
//                     if (p > Path + mark) p--;
//                     *p = UNICODE_NULL;
//                     EndPath = p;
//                     continue;
//
//                 case L'\\': /* .\ component */
//                     next = p + 2;
//                     // ASSERT(EndPath - next == wcslen(next));
//                     RtlMoveMemory(p, next, (EndPath - next + 1) * sizeof(WCHAR));
//                     EndPath -= (next - p);
//                     continue;
//
//                 case L'.':
//                     if (p[2] == L'\\')  /* ..\ component */
//                     {
//                         next = p + 3;
//                         if (p > Path + mark)
//                         {
//                             p--;
//                             while (p > Path + mark && p[-1] != L'\\') p--;
//                         }
//                         // ASSERT(EndPath - next == wcslen(next));
//                         RtlMoveMemory(p, next, (EndPath - next + 1) * sizeof(WCHAR));
//                         EndPath -= (next - p);
//                         continue;
//                     }
//                     else if (p[2] == UNICODE_NULL)  /* final .. */
//                     {
//                         if (p > Path + mark)
//                         {
//                             p--;
//                             while (p > Path + mark && p[-1] != L'\\') p--;
//                             if (p > Path + mark) p--;
//                         }
//                         *p = UNICODE_NULL;
//                         EndPath = p;
//                         continue;
//                     }
//                     break;
//             }
//         }
//
//         /* Skip to the next component */
//         while (*p && *p != L'\\') p++;
//         if (*p == L'\\')
//         {
//             /* Remove last dot in previous dir name */
//             if (p > Path + mark && p[-1] == L'.')
//             {
//                 // ASSERT(EndPath - p == wcslen(p));
//                 RtlMoveMemory(p - 1, p, (EndPath - p + 1) * sizeof(WCHAR));
//                 EndPath--;
//             }
//             else
//             {
//                 p++;
//             }
//         }
//     }
//
//     /* Remove trailing backslashes if needed (after the UNC part if it exists) */
//     if (SkipTrailingPathSeparators)
//     {
//         while (p > Path + mark && IS_PATH_SEPARATOR(p[-1])) p--;
//     }
//
//     /* Remove trailing spaces and dots (for all the path) */
//     while (p > Path && (p[-1] == L' ' || p[-1] == L'.')) p--;
//
//     /*
//      * Zero-out the discarded buffer zone, starting just after
//      * the path string and going up to the end of the buffer.
//      * It also NULL-terminate the path string.
//      */
//     ASSERT(EndBuffer >= p);
//     RtlZeroMemory(p, (EndBuffer - p + 1) * sizeof(WCHAR));
//
//     /* Return the real path length */
//     PathLength = (p - Path);
//     // ASSERT(PathLength == wcslen(Path));
//     return (PathLength * sizeof(WCHAR));
// }
//
// /******************************************************************
//  *    RtlpSkipUNCPrefix (from WINE)
//  *
//  * Helper for RtlGetFullPathName_U
//  *
//  * Skips the \\share\dir part of a file name and returns the new position
//  * (which can point on the last backslash of "dir\").
//  */
// static SIZE_T
// RtlpSkipUNCPrefix(PCWSTR FileNameBuffer)
// {
//     PCWSTR UncPath = FileNameBuffer + 2;
//     DPRINT("RtlpSkipUNCPrefix(%S)\n", FileNameBuffer);
//
//     while (*UncPath && !IS_PATH_SEPARATOR(*UncPath)) UncPath++;  /* share name */
//     while (IS_PATH_SEPARATOR(*UncPath)) UncPath++;
//     while (*UncPath && !IS_PATH_SEPARATOR(*UncPath)) UncPath++;  /* dir name */
//     /* while (IS_PATH_SEPARATOR(*UncPath)) UncPath++; */
//
//     return (UncPath - FileNameBuffer);
// }
//
// NTSTATUS
// NTAPI
// RtlpApplyLengthFunction(IN ULONG Flags,
//                         IN ULONG Type,
//                         IN PVOID UnicodeStringOrUnicodeStringBuffer,
//                         IN NTSTATUS(NTAPI* LengthFunction)(ULONG, PUNICODE_STRING, PULONG))
// {
//     NTSTATUS Status;
//     PUNICODE_STRING String;
//     ULONG Length;
//
//     if (Flags || UnicodeStringOrUnicodeStringBuffer == NULL || LengthFunction == NULL)
//     {
//         DPRINT1("ERROR: Flags=0x%x, UnicodeStringOrUnicodeStringBuffer=%p, LengthFunction=%p\n",
//             Flags, UnicodeStringOrUnicodeStringBuffer, LengthFunction);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (Type == sizeof(UNICODE_STRING))
//     {
//         String = (PUNICODE_STRING)UnicodeStringOrUnicodeStringBuffer;
//     }
//     else if (Type == sizeof(RTL_UNICODE_STRING_BUFFER))
//     {
//         String = &((PRTL_UNICODE_STRING_BUFFER)UnicodeStringOrUnicodeStringBuffer)->String;
//     }
//     else
//     {
//         DPRINT1("ERROR: Type = %u\n", Type);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Length = 0;
//     Status = LengthFunction(0, String, &Length);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (Length > UNICODE_STRING_MAX_CHARS)
//         return STATUS_NAME_TOO_LONG;
//
//     String->Length = (USHORT)(Length * sizeof(WCHAR));
//
//     if (Type == sizeof(RTL_UNICODE_STRING_BUFFER))
//     {
//         String->Buffer[Length] = UNICODE_NULL;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RtlGetLengthWithoutLastFullDosOrNtPathElement(IN ULONG Flags,
//                                               IN PCUNICODE_STRING Path,
//                                               OUT PULONG LengthOut)
// {
//     static const UNICODE_STRING PathDividers = RTL_CONSTANT_STRING(L"\\/");
//     USHORT Position;
//     RTL_PATH_TYPE PathType;
//
//     /* All failure paths have this in common, so simplify code */
//     if (LengthOut)
//         *LengthOut = 0;
//
//     if (Flags || !Path || !LengthOut)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((Path->Length / sizeof(WCHAR)) == 0)
//     {
//         /* Nothing to do here */
//         return STATUS_SUCCESS;
//     }
//
//
//     PathType = RtlDetermineDosPathNameType_Ustr(Path);
//     switch (PathType)
//     {
//     case RtlPathTypeLocalDevice:
//         // Handle \\\\?\\C:\\ with the last ':' or '\\' missing:
//         if (Path->Length / sizeof(WCHAR) < 7 ||
//             Path->Buffer[5] != ':' ||
//             !IS_PATH_SEPARATOR(Path->Buffer[6]))
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//         break;
//     case RtlPathTypeRooted:
//         // "\\??\\"
//         break;
//     case RtlPathTypeUncAbsolute:
//         // "\\\\"
//         break;
//     case RtlPathTypeDriveAbsolute:
//         // "C:\\"
//         break;
//     default:
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Find the last path separator */
//     if (!NT_SUCCESS(RtlFindCharInUnicodeString(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END, Path, &PathDividers, &Position)))
//         Position = 0;
//
//     /* Is it the last char of the string? */
//     if (Position && Position + sizeof(WCHAR) == Path->Length)
//     {
//         UNICODE_STRING Tmp = *Path;
//         Tmp.Length = Position;
//
//         /* Keep walking path separators to eliminate multiple next to eachother */
//         while (Tmp.Length > sizeof(WCHAR) && IS_PATH_SEPARATOR(Tmp.Buffer[Tmp.Length / sizeof(WCHAR)]))
//             Tmp.Length -= sizeof(WCHAR);
//
//         /* Find the previous occurence */
//         if (!NT_SUCCESS(RtlFindCharInUnicodeString(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END, &Tmp, &PathDividers, &Position)))
//             Position = 0;
//     }
//
//     /* Simplify code by working in chars instead of bytes */
//     if (Position)
//         Position /= sizeof(WCHAR);
//
//     if (Position)
//     {
//         // Keep walking path separators to eliminate multiple next to eachother, but ensure we leave one in place!
//         while (Position > 1 && IS_PATH_SEPARATOR(Path->Buffer[Position - 1]))
//             Position--;
//     }
//
//     if (Position > 0)
//     {
//         /* Return a length, not an index */
//         *LengthOut = Position + 1;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RtlComputePrivatizedDllName_U(
//     _In_ PUNICODE_STRING DllName,
//     _Inout_ PUNICODE_STRING RealName,
//     _Inout_ PUNICODE_STRING LocalName)
// {
//     static const UNICODE_STRING ExtensionChar = RTL_CONSTANT_STRING(L".");
//
//     USHORT Position;
//     UNICODE_STRING ImagePathName, DllNameOnly, CopyRealName, CopyLocalName;
//     BOOLEAN HasExtension;
//     ULONG RequiredSize;
//     NTSTATUS Status;
//     C_ASSERT(sizeof(UNICODE_NULL) == sizeof(WCHAR));
//
//     CopyRealName = *RealName;
//     CopyLocalName = *LocalName;
//
//
//     /* Get the image path */
//     ImagePathName = RtlGetCurrentPeb()->ProcessParameters->ImagePathName;
//
//     /* Check if it's not normalized */
//     if (!(RtlGetCurrentPeb()->ProcessParameters->Flags & RTL_USER_PROCESS_PARAMETERS_NORMALIZED))
//     {
//         /* Normalize it */
//         ImagePathName.Buffer = (PWSTR)((ULONG_PTR)ImagePathName.Buffer + (ULONG_PTR)RtlGetCurrentPeb()->ProcessParameters);
//     }
//
//
//     if (!NT_SUCCESS(RtlFindCharInUnicodeString(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
//                                                DllName, &RtlpPathDividers, &Position)))
//     {
//         DllNameOnly = *DllName;
//     }
//     else
//     {
//         /* Just keep the dll name, ignore path components */
//         Position += sizeof(WCHAR);
//         DllNameOnly.Buffer = DllName->Buffer + Position / sizeof(WCHAR);
//         DllNameOnly.Length = DllName->Length - Position;
//         DllNameOnly.MaximumLength = DllName->MaximumLength - Position;
//     }
//
//     if (!NT_SUCCESS(RtlFindCharInUnicodeString(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
//                                                &DllNameOnly, &ExtensionChar, &Position)))
//     {
//         Position = 0;
//     }
//
//     HasExtension = Position > 1;
//
//     /* First we create the c:\path\processname.exe.Local\something.dll path */
//     RequiredSize = ImagePathName.Length + RtlpDotLocal.Length + DllNameOnly.Length +
//         (HasExtension ? 0 : RtlpDefaultExtension.Length) + sizeof(UNICODE_NULL);
//
//     /* This is not going to work out */
//     if (RequiredSize > UNICODE_STRING_MAX_BYTES)
//         return STATUS_NAME_TOO_LONG;
//
//     /* We need something extra */
//     if (RequiredSize > CopyLocalName.MaximumLength)
//     {
//         CopyLocalName.Buffer = RtlpAllocateStringMemory(RequiredSize, TAG_USTR);
//         if (CopyLocalName.Buffer == NULL)
//             return STATUS_NO_MEMORY;
//         CopyLocalName.MaximumLength = RequiredSize;
//     }
//     /* Now build the entire path */
//     CopyLocalName.Length = 0;
//     Status = RtlAppendUnicodeStringToString(&CopyLocalName, &ImagePathName);
//     ASSERT(NT_SUCCESS(Status));
//     if (NT_SUCCESS(Status))
//     {
//         Status = RtlAppendUnicodeStringToString(&CopyLocalName, &RtlpDotLocal);
//         ASSERT(NT_SUCCESS(Status));
//     }
//     if (NT_SUCCESS(Status))
//     {
//         Status = RtlAppendUnicodeStringToString(&CopyLocalName, &DllNameOnly);
//         ASSERT(NT_SUCCESS(Status));
//     }
//     /* Do we need to append an extension? */
//     if (NT_SUCCESS(Status) && !HasExtension)
//     {
//         Status = RtlAppendUnicodeStringToString(&CopyLocalName, &RtlpDefaultExtension);
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         /* then we create the c:\path\something.dll path */
//         if (NT_SUCCESS(RtlFindCharInUnicodeString(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
//                                                   &ImagePathName, &RtlpPathDividers, &Position)))
//         {
//             ImagePathName.Length = Position + sizeof(WCHAR);
//         }
//
//         RequiredSize = ImagePathName.Length + DllNameOnly.Length +
//             (HasExtension ? 0 : RtlpDefaultExtension.Length) + sizeof(UNICODE_NULL);
//
//         if (RequiredSize >= UNICODE_STRING_MAX_BYTES)
//         {
//             Status = STATUS_NAME_TOO_LONG;
//         }
//         else
//         {
//             if (RequiredSize > CopyRealName.MaximumLength)
//             {
//                 CopyRealName.Buffer = RtlpAllocateStringMemory(RequiredSize, TAG_USTR);
//                 if (CopyRealName.Buffer == NULL)
//                     Status = STATUS_NO_MEMORY;
//                 CopyRealName.MaximumLength = RequiredSize;
//             }
//             CopyRealName.Length = 0;
//             if (NT_SUCCESS(Status))
//             {
//                 Status = RtlAppendUnicodeStringToString(&CopyRealName, &ImagePathName);
//                 ASSERT(NT_SUCCESS(Status));
//             }
//             if (NT_SUCCESS(Status))
//             {
//                 Status = RtlAppendUnicodeStringToString(&CopyRealName, &DllNameOnly);
//                 ASSERT(NT_SUCCESS(Status));
//             }
//             if (NT_SUCCESS(Status) && !HasExtension)
//             {
//                 Status = RtlAppendUnicodeStringToString(&CopyRealName, &RtlpDefaultExtension);
//                 ASSERT(NT_SUCCESS(Status));
//             }
//         }
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (CopyRealName.Buffer && CopyRealName.Buffer != RealName->Buffer)
//             RtlpFreeStringMemory(CopyRealName.Buffer, TAG_USTR);
//         if (CopyLocalName.Buffer && CopyLocalName.Buffer != LocalName->Buffer)
//             RtlpFreeStringMemory(CopyLocalName.Buffer, TAG_USTR);
//         return Status;
//     }
//
//     *RealName = CopyRealName;
//     *LocalName = CopyLocalName;
//     return STATUS_SUCCESS;
// }
//
// ULONG
// NTAPI
// RtlGetFullPathName_Ustr(
//     _In_ PUNICODE_STRING FileName,
//     _In_ ULONG Size,
//     _Out_z_bytecap_(Size) PWSTR Buffer,
//     _Out_opt_ PCWSTR *ShortName,
//     _Out_opt_ PBOOLEAN InvalidName,
//     _Out_ RTL_PATH_TYPE *PathType)
// {
//     NTSTATUS Status;
//     PWCHAR FileNameBuffer;
//     ULONG FileNameLength, FileNameChars, DosLength, DosLengthOffset, FullLength;
//     BOOLEAN SkipTrailingPathSeparators;
//     WCHAR c;
//
//
//     ULONG               reqsize = 0;
//     PCWSTR              ptr;
//
//     PCUNICODE_STRING    CurDirName;
//     UNICODE_STRING      EnvVarName, EnvVarValue;
//     WCHAR EnvVarNameBuffer[4];
//
//     ULONG  PrefixCut    = 0;    // Where the path really starts (after the skipped prefix)
//     PWCHAR Prefix       = NULL; // pointer to the string to be inserted as the new path prefix
//     ULONG  PrefixLength = 0;
//     PWCHAR Source;
//     ULONG  SourceLength;
//
//
//     /* For now, assume the name is valid */
//     DPRINT("Filename: %wZ\n", FileName);
//     DPRINT("Size and buffer: %lx %p\n", Size, Buffer);
//     if (InvalidName) *InvalidName = FALSE;
//
//     /* Handle initial path type and failure case */
//     *PathType = RtlPathTypeUnknown;
//     if ((FileName->Length == 0) || (FileName->Buffer[0] == UNICODE_NULL)) return 0;
//
//     /* Break filename into component parts */
//     FileNameBuffer = FileName->Buffer;
//     FileNameLength = FileName->Length;
//     FileNameChars  = FileNameLength / sizeof(WCHAR);
//
//     /* Kill trailing spaces */
//     c = FileNameBuffer[FileNameChars - 1];
//     while ((FileNameLength != 0) && (c == L' '))
//     {
//         /* Keep going, ignoring the spaces */
//         FileNameLength -= sizeof(WCHAR);
//         if (FileNameLength != 0) c = FileNameBuffer[FileNameLength / sizeof(WCHAR) - 1];
//     }
//
//     /* Check if anything is left */
//     if (FileNameLength == 0) return 0;
//
//     /*
//      * Check whether we'll need to skip trailing path separators in the
//      * computed full path name. If the original file name already contained
//      * trailing separators, then we keep them in the full path name. On the
//      * other hand, if the original name didn't contain any trailing separators
//      * then we'll skip it in the full path name.
//      */
//     SkipTrailingPathSeparators = !IS_PATH_SEPARATOR(FileNameBuffer[FileNameChars - 1]);
//
//     /* Check if this is a DOS name */
//     DosLength = RtlIsDosDeviceName_Ustr(FileName);
//     DPRINT("DOS length for filename: %lx %wZ\n", DosLength, FileName);
//     if (DosLength != 0)
//     {
//         /* Zero out the short name */
//         if (ShortName) *ShortName = NULL;
//
//         /* See comment for RtlIsDosDeviceName_Ustr if this is confusing... */
//         DosLengthOffset = HIWORD(DosLength);
//         DosLength       = LOWORD(DosLength);
//
//         /* Do we have a DOS length, and does the caller want validity? */
//         if (InvalidName && (DosLengthOffset != 0))
//         {
//             /* Do the check */
//             Status = RtlpCheckDeviceName(FileName, DosLengthOffset, InvalidName);
//
//             /* If the check failed, or the name is invalid, fail here */
//             if (!NT_SUCCESS(Status)) return 0;
//             if (*InvalidName) return 0;
//         }
//
//         /* Add the size of the device root and check if it fits in the size */
//         FullLength = DosLength + DeviceRootString.Length;
//         if (FullLength < Size)
//         {
//             /* Add the device string */
//             RtlMoveMemory(Buffer, DeviceRootString.Buffer, DeviceRootString.Length);
//
//             /* Now add the DOS device name */
//             RtlMoveMemory((PCHAR)Buffer + DeviceRootString.Length,
//                           (PCHAR)FileNameBuffer + DosLengthOffset,
//                           DosLength);
//
//             /* Null terminate */
//             *(PWCHAR)((ULONG_PTR)Buffer + FullLength) = UNICODE_NULL;
//             return FullLength;
//         }
//
//         /* Otherwise, there's no space, so return the buffer size needed */
//         if ((FullLength + sizeof(UNICODE_NULL)) > UNICODE_STRING_MAX_BYTES) return 0;
//         return FullLength + sizeof(UNICODE_NULL);
//     }
//
//     /* Zero-out the destination buffer. FileName must be different from Buffer */
//     RtlZeroMemory(Buffer, Size);
//
//     /* Get the path type */
//     *PathType = RtlDetermineDosPathNameType_U(FileNameBuffer);
//
//
//
//     /**********************************************
//      **    CODE REWRITING IS HAPPENING THERE     **
//      **********************************************/
//     Source       = FileNameBuffer;
//     SourceLength = FileNameLength;
//     EnvVarValue.Buffer = NULL;
//
//     /* Lock the PEB to get the current directory */
//     RtlAcquirePebLock();
//     CurDirName = &NtCurrentPeb()->ProcessParameters->CurrentDirectory.DosPath;
//
//     switch (*PathType)
//     {
//         case RtlPathTypeUncAbsolute:        /* \\foo   */
//         {
//             PrefixCut = RtlpSkipUNCPrefix(FileNameBuffer);
//             break;
//         }
//
//         case RtlPathTypeLocalDevice:        /* \\.\foo */
//         {
//             PrefixCut = 4;
//             break;
//         }
//
//         case RtlPathTypeDriveAbsolute:      /* c:\foo  */
//         {
//             ASSERT(FileNameBuffer[1] == L':');
//             ASSERT(IS_PATH_SEPARATOR(FileNameBuffer[2]));
//
//             // FileNameBuffer[0] = RtlpUpcaseUnicodeChar(FileNameBuffer[0]);
//             Prefix = FileNameBuffer;
//             PrefixLength = 3 * sizeof(WCHAR);
//             Source += 3;
//             SourceLength -= 3 * sizeof(WCHAR);
//
//             PrefixCut = 3;
//             break;
//         }
//
//         case RtlPathTypeDriveRelative:      /* c:foo   */
//         {
//             WCHAR CurDrive, NewDrive;
//
//             Source += 2;
//             SourceLength -= 2 * sizeof(WCHAR);
//
//             CurDrive = RtlpUpcaseUnicodeChar(CurDirName->Buffer[0]);
//             NewDrive = RtlpUpcaseUnicodeChar(FileNameBuffer[0]);
//
//             if ((NewDrive != CurDrive) || CurDirName->Buffer[1] != L':')
//             {
//                 EnvVarNameBuffer[0] = L'=';
//                 EnvVarNameBuffer[1] = NewDrive;
//                 EnvVarNameBuffer[2] = L':';
//                 EnvVarNameBuffer[3] = UNICODE_NULL;
//
//                 EnvVarName.Length = 3 * sizeof(WCHAR);
//                 EnvVarName.MaximumLength = EnvVarName.Length + sizeof(WCHAR);
//                 EnvVarName.Buffer = EnvVarNameBuffer;
//
//                 // FIXME: Is it possible to use the user-given buffer ?
//                 // RtlInitEmptyUnicodeString(&EnvVarValue, NULL, Size);
//                 EnvVarValue.Length = 0;
//                 EnvVarValue.MaximumLength = (USHORT)Size;
//                 EnvVarValue.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, Size);
//                 if (EnvVarValue.Buffer == NULL)
//                 {
//                     Prefix       = NULL;
//                     PrefixLength = 0;
//                     goto Quit;
//                 }
//
//                 Status = RtlQueryEnvironmentVariable_U(NULL, &EnvVarName, &EnvVarValue);
//                 switch (Status)
//                 {
//                     case STATUS_SUCCESS:
//                         /*
//                          * (From Wine)
//                          * FIXME: Win2k seems to check that the environment
//                          * variable actually points to an existing directory.
//                          * If not, root of the drive is used (this seems also
//                          * to be the only place in RtlGetFullPathName that the
//                          * existence of a part of a path is checked).
//                          */
//                         EnvVarValue.Buffer[EnvVarValue.Length / sizeof(WCHAR)] = L'\\';
//                         Prefix       = EnvVarValue.Buffer;
//                         PrefixLength = EnvVarValue.Length + sizeof(WCHAR); /* Append trailing '\\' */
//                         break;
//
//                     case STATUS_BUFFER_TOO_SMALL:
//                         reqsize = EnvVarValue.Length + SourceLength + sizeof(UNICODE_NULL);
//                         goto Quit;
//
//                     default:
//                         DPRINT1("RtlQueryEnvironmentVariable_U(\"%wZ\") returned 0x%08lx\n", &EnvVarName, Status);
//
//                         EnvVarNameBuffer[0] = NewDrive;
//                         EnvVarNameBuffer[1] = L':';
//                         EnvVarNameBuffer[2] = L'\\';
//                         EnvVarNameBuffer[3] = UNICODE_NULL;
//                         Prefix       = EnvVarNameBuffer;
//                         PrefixLength = 3 * sizeof(WCHAR);
//
//                         RtlFreeHeap(RtlGetProcessHeap(), 0, EnvVarValue.Buffer);
//                         EnvVarValue.Buffer = NULL;
//                         break;
//                 }
//                 PrefixCut = 3;
//                 break;
//             }
//             /* Fall through */
//             DPRINT("RtlPathTypeDriveRelative - Using fall-through to RtlPathTypeRelative\n");
//         }
//
//         case RtlPathTypeRelative:           /* foo     */
//         {
//             Prefix       = CurDirName->Buffer;
//             PrefixLength = CurDirName->Length;
//             if (CurDirName->Buffer[1] != L':')
//             {
//                 PrefixCut = RtlpSkipUNCPrefix(CurDirName->Buffer);
//             }
//             else
//             {
//                 PrefixCut = 3;
//             }
//             break;
//         }
//
//         case RtlPathTypeRooted:             /* \xxx    */
//         {
//             if (CurDirName->Buffer[1] == L':')
//             {
//                 // The path starts with "C:\"
//                 ASSERT(CurDirName->Buffer[1] == L':');
//                 ASSERT(IS_PATH_SEPARATOR(CurDirName->Buffer[2]));
//
//                 Prefix = CurDirName->Buffer;
//                 PrefixLength = 3 * sizeof(WCHAR); // Skip "C:\"
//
//                 PrefixCut = 3;      // Source index location incremented of + 3
//             }
//             else
//             {
//                 PrefixCut = RtlpSkipUNCPrefix(CurDirName->Buffer);
//                 PrefixLength = PrefixCut * sizeof(WCHAR);
//                 Prefix = CurDirName->Buffer;
//             }
//             break;
//         }
//
//         case RtlPathTypeRootLocalDevice:    /* \\.     */
//         {
//             Prefix       = DeviceRootString.Buffer;
//             PrefixLength = DeviceRootString.Length;
//             Source += 3;
//             SourceLength -= 3 * sizeof(WCHAR);
//
//             PrefixCut = 4;
//             break;
//         }
//
//         case RtlPathTypeUnknown:
//             goto Quit;
//     }
//
//     /* Do we have enough space for storing the full path? */
//     reqsize = PrefixLength;
//     if (reqsize + SourceLength + sizeof(WCHAR) > Size)
//     {
//         /* Not enough space, return needed size (including terminating '\0') */
//         reqsize += SourceLength + sizeof(WCHAR);
//         goto Quit;
//     }
//
//     /*
//      * Build the full path
//      */
//     /* Copy the path's prefix */
//     if (PrefixLength) RtlMoveMemory(Buffer, Prefix, PrefixLength);
//     /* Copy the remaining part of the path */
//     RtlMoveMemory(Buffer + PrefixLength / sizeof(WCHAR), Source, SourceLength + sizeof(WCHAR));
//
//     /* Some cleanup */
//     Prefix = NULL;
//     if (EnvVarValue.Buffer)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, EnvVarValue.Buffer);
//         EnvVarValue.Buffer = NULL;
//     }
//
//     /*
//      * Finally, put the path in canonical form (remove redundant . and ..,
//      * (back)slashes...) and retrieve the length of the full path name
//      * (without its terminating null character) (in chars).
//      */
//     reqsize = RtlpCollapsePath(Buffer, /* Size, reqsize, */ PrefixCut, SkipTrailingPathSeparators);
//
//     /* Find the file part, which is present after the last path separator */
//     if (ShortName)
//     {
//         ptr = wcsrchr(Buffer, L'\\');
//         if (ptr) ++ptr; // Skip it
//
//         /*
//          * For UNC paths, the file part is after the \\share\dir part of the path.
//          */
//         PrefixCut = (*PathType == RtlPathTypeUncAbsolute ? PrefixCut : 3);
//
//         if (ptr && *ptr && (ptr >= Buffer + PrefixCut))
//         {
//             *ShortName = ptr;
//         }
//         else
//         {
//             /* Zero-out the short name */
//             *ShortName = NULL;
//         }
//     }
//
// Quit:
//     /* Release PEB lock */
//     RtlReleasePebLock();
//
//     return reqsize;
// }
//
// NTSTATUS
// NTAPI
// RtlpWin32NTNameToNtPathName_U(IN PUNICODE_STRING DosPath,
//                               OUT PUNICODE_STRING NtPath,
//                               OUT PCWSTR *PartName,
//                               OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     ULONG DosLength;
//     PWSTR NewBuffer, p;
//
//     /* Validate the input */
//     if (!DosPath) return STATUS_OBJECT_NAME_INVALID;
//
//     /* Validate the DOS length */
//     DosLength = DosPath->Length;
//     if (DosLength >= UNICODE_STRING_MAX_BYTES) return STATUS_NAME_TOO_LONG;
//
//     /* Make space for the new path */
//     NewBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                 0,
//                                 DosLength + sizeof(UNICODE_NULL));
//     if (!NewBuffer) return STATUS_NO_MEMORY;
//
//     /* Copy the prefix, and then the rest of the DOS path, and NULL-terminate */
//     RtlCopyMemory(NewBuffer, RtlpDosDevicesPrefix.Buffer, RtlpDosDevicesPrefix.Length);
//     RtlCopyMemory((PCHAR)NewBuffer + RtlpDosDevicesPrefix.Length,
//                   DosPath->Buffer + RtlpDosDevicesPrefix.Length / sizeof(WCHAR),
//                   DosPath->Length - RtlpDosDevicesPrefix.Length);
//     NewBuffer[DosLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Did the caller send a relative name? */
//     if (RelativeName)
//     {
//         /* Zero initialize it */
//         RtlInitEmptyUnicodeString(&RelativeName->RelativeName, NULL, 0);
//         RelativeName->ContainingDirectory = NULL;
//         RelativeName->CurDirRef = 0;
//     }
//
//     /* Did the caller request a partial name? */
//     if (PartName)
//     {
//         /* Loop from the back until we find a path separator */
//         p = &NewBuffer[DosLength / sizeof(WCHAR)];
//         while (--p > NewBuffer)
//         {
//             /* We found a path separator, move past it */
//             if (*p == OBJ_NAME_PATH_SEPARATOR)
//             {
//                 ++p;
//                 break;
//             }
//         }
//
//         /* Check whether a separator was found and if something remains */
//         if ((p > NewBuffer) && *p)
//         {
//             /* What follows the path separator is the partial name */
//             *PartName = p;
//         }
//         else
//         {
//             /* The path ends with a path separator, no partial name */
//             *PartName = NULL;
//         }
//     }
//
//     /* Build the final NT path string */
//     NtPath->Buffer = NewBuffer;
//     NtPath->Length = (USHORT)DosLength;
//     NtPath->MaximumLength = (USHORT)DosLength + sizeof(UNICODE_NULL);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RtlpDosPathNameToRelativeNtPathName_Ustr(IN BOOLEAN HaveRelative,
//                                          IN PCUNICODE_STRING DosName,
//                                          OUT PUNICODE_STRING NtName,
//                                          OUT PCWSTR *PartName,
//                                          OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     WCHAR BigBuffer[MAX_PATH + 1];
//     PWCHAR PrefixBuffer, NewBuffer, Buffer;
//     ULONG MaxLength, PathLength, PrefixLength, PrefixCut, LengthChars, Length;
//     UNICODE_STRING CapturedDosName, PartNameString, FullPath;
//     BOOLEAN QuickPath;
//     RTL_PATH_TYPE InputPathType, BufferPathType;
//     NTSTATUS Status;
//     BOOLEAN NameInvalid;
//     PCURDIR CurrentDirectory;
//
//     /* Assume MAX_PATH for now */
//     DPRINT("Relative: %lx DosName: %wZ NtName: %p, PartName: %p, RelativeName: %p\n",
//             HaveRelative, DosName, NtName, PartName, RelativeName);
//     MaxLength = sizeof(BigBuffer);
//
//     /* Validate the input */
//     if (!DosName) return STATUS_OBJECT_NAME_INVALID;
//
//     /* Capture input string */
//     CapturedDosName = *DosName;
//
//     /* Check for the presence or absence of the NT prefix "\\?\" form */
//     // if (!RtlPrefixUnicodeString(&RtlpWin32NtRootSlash, &CapturedDosName, FALSE))
//     if ((CapturedDosName.Length <= RtlpWin32NtRootSlash.Length) ||
//         (CapturedDosName.Buffer[0] != RtlpWin32NtRootSlash.Buffer[0]) ||
//         (CapturedDosName.Buffer[1] != RtlpWin32NtRootSlash.Buffer[1]) ||
//         (CapturedDosName.Buffer[2] != RtlpWin32NtRootSlash.Buffer[2]) ||
//         (CapturedDosName.Buffer[3] != RtlpWin32NtRootSlash.Buffer[3]))
//     {
//         /* NT prefix not present */
//
//         /* Quick path won't be used */
//         QuickPath = FALSE;
//
//         /* Use the static buffer */
//         Buffer = BigBuffer;
//         MaxLength += RtlpDosDevicesUncPrefix.Length;
//
//         /* Allocate a buffer to hold the path */
//         NewBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, MaxLength);
//         DPRINT("MaxLength: %lx\n", MaxLength);
//         if (!NewBuffer) return STATUS_NO_MEMORY;
//     }
//     else
//     {
//         /* NT prefix present */
//
//         /* Use the optimized path after acquiring the lock */
//         QuickPath = TRUE;
//         NewBuffer = NULL;
//     }
//
//     /* Lock the PEB and check if the quick path can be used */
//     RtlAcquirePebLock();
//     if (QuickPath)
//     {
//         /* Some simple fixups will get us the correct path */
//         DPRINT("Quick path\n");
//         Status = RtlpWin32NTNameToNtPathName_U(&CapturedDosName,
//                                                NtName,
//                                                PartName,
//                                                RelativeName);
//
//         /* Release the lock, we're done here */
//         RtlReleasePebLock();
//         return Status;
//     }
//
//     /* Call the main function to get the full path name and length */
//     PathLength = RtlGetFullPathName_Ustr(&CapturedDosName,
//                                          MAX_PATH * sizeof(WCHAR),
//                                          Buffer,
//                                          PartName,
//                                          &NameInvalid,
//                                          &InputPathType);
//     if ((NameInvalid) || !(PathLength) || (PathLength > (MAX_PATH * sizeof(WCHAR))))
//     {
//         /* Invalid name, fail */
//         DPRINT("Invalid name: %lx Path Length: %lx\n", NameInvalid, PathLength);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer);
//         RtlReleasePebLock();
//         return STATUS_OBJECT_NAME_INVALID;
//     }
//
//     /* Start by assuming the path starts with \??\ (DOS Devices Path) */
//     PrefixLength = RtlpDosDevicesPrefix.Length;
//     PrefixBuffer = RtlpDosDevicesPrefix.Buffer;
//     PrefixCut = 0;
//
//     /* Check where it really is */
//     BufferPathType = RtlDetermineDosPathNameType_U(Buffer);
//     DPRINT("Buffer: %S Type: %lx\n", Buffer, BufferPathType);
//     switch (BufferPathType)
//     {
//         /* It's actually a UNC path in \??\UNC\ */
//         case RtlPathTypeUncAbsolute:
//             PrefixLength = RtlpDosDevicesUncPrefix.Length;
//             PrefixBuffer = RtlpDosDevicesUncPrefix.Buffer;
//             PrefixCut = 2;
//             break;
//
//         case RtlPathTypeLocalDevice:
//             /* We made a good guess, go with it but skip the \??\ */
//             PrefixCut = 4;
//             break;
//
//         case RtlPathTypeDriveAbsolute:
//         case RtlPathTypeDriveRelative:
//         case RtlPathTypeRooted:
//         case RtlPathTypeRelative:
//             /* Our guess was good, roll with it */
//             break;
//
//         /* Nothing else is expected */
//         default:
//             ASSERT(FALSE);
//     }
//
//     /* Now copy the prefix and the buffer */
//     RtlCopyMemory(NewBuffer, PrefixBuffer, PrefixLength);
//     RtlCopyMemory((PCHAR)NewBuffer + PrefixLength,
//                   Buffer + PrefixCut,
//                   PathLength - (PrefixCut * sizeof(WCHAR)));
//
//     /* Compute the length */
//     Length = PathLength + PrefixLength - PrefixCut * sizeof(WCHAR);
//     LengthChars = Length / sizeof(WCHAR);
//
//     /* Setup the actual NT path string and terminate it */
//     NtName->Buffer = NewBuffer;
//     NtName->Length = (USHORT)Length;
//     NtName->MaximumLength = (USHORT)MaxLength;
//     NewBuffer[LengthChars] = UNICODE_NULL;
//     DPRINT("New buffer: %S\n", NewBuffer);
//     DPRINT("NT Name: %wZ\n", NtName);
//
//     /* Check if a partial name was requested */
//     if ((PartName) && (*PartName))
//     {
//         /* Convert to Unicode */
//         Status = RtlInitUnicodeStringEx(&PartNameString, *PartName);
//         if (NT_SUCCESS(Status))
//         {
//             /* Set the partial name */
//             *PartName = &NewBuffer[LengthChars - (PartNameString.Length / sizeof(WCHAR))];
//         }
//         else
//         {
//             /* Fail */
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer);
//             RtlReleasePebLock();
//             return Status;
//         }
//     }
//
//     /* Check if a relative name was asked for */
//     if (RelativeName)
//     {
//         /* Setup the structure */
//         RtlInitEmptyUnicodeString(&RelativeName->RelativeName, NULL, 0);
//         RelativeName->ContainingDirectory = NULL;
//         RelativeName->CurDirRef = NULL;
//
//         /* Check if the input path itself was relative */
//         if (InputPathType == RtlPathTypeRelative)
//         {
//             /* Get current directory */
//             CurrentDirectory = &(NtCurrentPeb()->ProcessParameters->CurrentDirectory);
//             if (CurrentDirectory->Handle)
//             {
//                 Status = RtlInitUnicodeStringEx(&FullPath, Buffer);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer);
//                     RtlReleasePebLock();
//                     return Status;
//                 }
//
//                 /* If current directory is bigger than full path, there's no way */
//                 if (CurrentDirectory->DosPath.Length > FullPath.Length)
//                 {
//                     RtlReleasePebLock();
//                     return Status;
//                 }
//
//                 /* File is in current directory */
//                 if (RtlEqualUnicodeString(&FullPath, &CurrentDirectory->DosPath, TRUE))
//                 {
//                     /* Make relative name string */
//                     RelativeName->RelativeName.Buffer = (PWSTR)((ULONG_PTR)NewBuffer + PrefixLength + FullPath.Length - PrefixCut * sizeof(WCHAR));
//                     RelativeName->RelativeName.Length = (USHORT)(PathLength - FullPath.Length);
//                     /* If relative name starts with \, skip it */
//                     if (RelativeName->RelativeName.Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//                     {
//                         RelativeName->RelativeName.Buffer++;
//                         RelativeName->RelativeName.Length -= sizeof(WCHAR);
//                     }
//                     RelativeName->RelativeName.MaximumLength = RelativeName->RelativeName.Length;
//                     DPRINT("RelativeName: %wZ\n", &(RelativeName->RelativeName));
//
//                     if (!HaveRelative)
//                     {
//                         RelativeName->ContainingDirectory = CurrentDirectory->Handle;
//                         return Status;
//                     }
//
//                     /* Give back current directory data & reference counter */
//                     RelativeName->CurDirRef = RtlpCurDirRef;
//                     if (RelativeName->CurDirRef)
//                     {
//                         InterlockedIncrement(&RtlpCurDirRef->RefCount);
//                     }
//
//                     RelativeName->ContainingDirectory = CurrentDirectory->Handle;
//                 }
//             }
//         }
//     }
//
//     /* Done */
//     RtlReleasePebLock();
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// RtlpDosPathNameToRelativeNtPathName_U(IN BOOLEAN HaveRelative,
//                                       IN PCWSTR DosName,
//                                       OUT PUNICODE_STRING NtName,
//                                       OUT PCWSTR *PartName,
//                                       OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     NTSTATUS Status;
//     UNICODE_STRING NameString;
//
//     /* Create the unicode name */
//     Status = RtlInitUnicodeStringEx(&NameString, DosName);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the unicode function */
//         Status = RtlpDosPathNameToRelativeNtPathName_Ustr(HaveRelative,
//                                                           &NameString,
//                                                           NtName,
//                                                           PartName,
//                                                           RelativeName);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// RtlDosPathNameToRelativeNtPathName_Ustr(IN PCUNICODE_STRING DosName,
//                                         OUT PUNICODE_STRING NtName,
//                                         OUT PCWSTR *PartName,
//                                         OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     /* Call the internal function */
//     ASSERT(RelativeName);
//     return NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_Ustr(TRUE,
//                                                                DosName,
//                                                                NtName,
//                                                                PartName,
//                                                                RelativeName));
// }
//
// BOOLEAN
// NTAPI
// RtlDoesFileExists_UstrEx(IN PCUNICODE_STRING FileName,
//                          IN BOOLEAN SucceedIfBusy)
// {
//     BOOLEAN Result;
//     RTL_RELATIVE_NAME_U RelativeName;
//     UNICODE_STRING NtPathName;
//     PVOID Buffer;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     FILE_BASIC_INFORMATION BasicInformation;
//
//     /* Get the NT Path */
//     Result = RtlDosPathNameToRelativeNtPathName_Ustr(FileName,
//                                                      &NtPathName,
//                                                      NULL,
//                                                      &RelativeName);
//     if (!Result) return FALSE;
//
//     /* Save the buffer */
//     Buffer = NtPathName.Buffer;
//
//     /* Check if we have a relative name */
//     if (RelativeName.RelativeName.Length)
//     {
//         /* Use it */
//         NtPathName = RelativeName.RelativeName;
//     }
//     else
//     {
//         /* Otherwise ignore it */
//         RelativeName.ContainingDirectory = NULL;
//     }
//
//     /* Initialize the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtPathName,
//                                OBJ_CASE_INSENSITIVE,
//                                RelativeName.ContainingDirectory,
//                                NULL);
//
//     /* Query the attributes and free the buffer now */
//     Status = ZwQueryAttributesFile(&ObjectAttributes, &BasicInformation);
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
//     /* Check if we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Check if we failed because the file is in use */
//         if ((Status == STATUS_SHARING_VIOLATION) ||
//             (Status == STATUS_ACCESS_DENIED))
//         {
//             /* Check if the caller wants this to be considered OK */
//             Result = SucceedIfBusy ? TRUE : FALSE;
//         }
//         else
//         {
//             /* A failure because the file didn't exist */
//             Result = FALSE;
//         }
//     }
//     else
//     {
//         /* The file exists */
//         Result = TRUE;
//     }
//
//     /* Return the result */
//     return Result;
// }
//
// BOOLEAN
// NTAPI
// RtlDoesFileExists_UStr(IN PUNICODE_STRING FileName)
// {
//     /* Call the updated API */
//     return RtlDoesFileExists_UstrEx(FileName, TRUE);
// }
//
// BOOLEAN
// NTAPI
// RtlDoesFileExists_UEx(IN PCWSTR FileName,
//                       IN BOOLEAN SucceedIfBusy)
// {
//     UNICODE_STRING NameString;
//
//     /* Create the unicode name*/
//     if (NT_SUCCESS(RtlInitUnicodeStringEx(&NameString, FileName)))
//     {
//         /* Call the unicode function */
//         return RtlDoesFileExists_UstrEx(&NameString, SucceedIfBusy);
//     }
//
//     /* Fail */
//     return FALSE;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// RtlReleaseRelativeName(IN PRTL_RELATIVE_NAME_U RelativeName)
// {
//     /* Check if a directory reference was grabbed */
//     if (RelativeName->CurDirRef)
//     {
//         /* Decrease reference count */
//         if (!InterlockedDecrement(&RelativeName->CurDirRef->RefCount))
//         {
//             /* If no one uses it any longer, close handle & free */
//             NtClose(RelativeName->CurDirRef->Handle);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeName->CurDirRef);
//         }
//         RelativeName->CurDirRef = NULL;
//     }
// }
//
// #if 0 /* Replaced by Rust (Wave PATH-1): RtlGetLongestNtPathLength */
// ULONG
// NTAPI
// RtlGetLongestNtPathLength(VOID)
// {
//     /*
//      * The longest NT path is a DOS path that actually sits on a UNC path (ie:
//      * a mapped network drive), which is accessed through the DOS Global?? path.
//      * This is, and has always been equal to, 269 characters, except in Wine
//      * which claims this is 277. Go figure.
//      */
//     return MAX_PATH + RtlpDosDevicesUncPrefix.Length / sizeof(WCHAR) + sizeof(ANSI_NULL);
// }
// #endif /* PATH-1 Rust override */
//
// #if 0 /* Replaced by Rust (Wave PATH-1): RtlGetLengthWithoutTrailingPathSeparators */
// NTSTATUS
// NTAPI
// RtlGetLengthWithoutTrailingPathSeparators(IN  ULONG Flags,
//                                           IN  PCUNICODE_STRING PathString,
//                                           OUT PULONG Length)
// {
//     ULONG NumChars;
//
//     /* Parameters validation */
//     if (Length == NULL) return STATUS_INVALID_PARAMETER;
//
//     *Length = 0;
//
//     if (PathString == NULL) return STATUS_INVALID_PARAMETER;
//
//     /* No flags are supported yet */
//     if (Flags != 0) return STATUS_INVALID_PARAMETER;
//
//     NumChars = PathString->Length / sizeof(WCHAR);
//
//     /*
//      * Notice that we skip the last character, therefore:
//      * - if we have: "some/path/f" we test for: "some/path/"
//      * - if we have: "some/path/"  we test for: "some/path"
//      * - if we have: "s" we test for: ""
//      * - if we have: "" then NumChars was already zero and we aren't there
//      */
//
//     while (NumChars > 0 && IS_PATH_SEPARATOR(PathString->Buffer[NumChars - 1]))
//     {
//         --NumChars;
//     }
//
//     *Length = NumChars;
//     return STATUS_SUCCESS;
// }
// #endif /* PATH-1 Rust override: RtlGetLengthWithoutTrailingPathSeparators */
//
// #if 0 /* Replaced by Rust (Wave PATH-1): RtlDetermineDosPathNameType_U + RtlIsDosDeviceName_U */
// RTL_PATH_TYPE
// NTAPI
// RtlDetermineDosPathNameType_U(IN PCWSTR Path)
// {
//     DPRINT("RtlDetermineDosPathNameType_U %S\n", Path);
//
//     /* Unlike the newer RtlDetermineDosPathNameType_U we assume 4 characters */
//     if (IS_PATH_SEPARATOR(Path[0]))
//     {
//         if (!IS_PATH_SEPARATOR(Path[1])) return RtlPathTypeRooted;                /* \x             */
//         if ((Path[2] != L'.') && (Path[2] != L'?')) return RtlPathTypeUncAbsolute;/* \\x            */
//         if (IS_PATH_SEPARATOR(Path[3])) return RtlPathTypeLocalDevice;            /* \\.\x or \\?\x */
//         if (Path[3]) return RtlPathTypeUncAbsolute;                               /* \\.x or \\?x   */
//         return RtlPathTypeRootLocalDevice;                                        /* \\. or \\?     */
//     }
//     else
//     {
//         if (!(Path[0]) || (Path[1] != L':')) return RtlPathTypeRelative;          /* x              */
//         if (IS_PATH_SEPARATOR(Path[2])) return RtlPathTypeDriveAbsolute;          /* x:\            */
//         return RtlPathTypeDriveRelative;                                          /* x:             */
//     }
// }
//
// ULONG
// NTAPI
// RtlIsDosDeviceName_U(IN PCWSTR Path)
// {
//     UNICODE_STRING PathString;
//     NTSTATUS Status;
//
//     /* Build the string */
//     Status = RtlInitUnicodeStringEx(&PathString, Path);
//     if (!NT_SUCCESS(Status)) return 0;
//
//     /*
//      * Returns 0 if name is not valid DOS device name, or DWORD with
//      * offset in bytes to DOS device name from beginning of buffer in high word
//      * and size in bytes of DOS device name in low word
//      */
//      return RtlIsDosDeviceName_Ustr(&PathString);
// }
// #endif /* PATH-1 Rust override */
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// RtlGetCurrentDirectory_U(
//     _In_ ULONG MaximumLength,
//     _Out_bytecap_(MaximumLength) PWSTR Buffer)
// {
//     ULONG Length, Bytes;
//     PCURDIR CurDir;
//     PWSTR CurDirName;
//     DPRINT("RtlGetCurrentDirectory %lu %p\n", MaximumLength, Buffer);
//
//     /* Lock the PEB to get the current directory */
//     RtlAcquirePebLock();
//     CurDir = &NtCurrentPeb()->ProcessParameters->CurrentDirectory;
//
//     /* Get the buffer and character length */
//     CurDirName = CurDir->DosPath.Buffer;
//     Length = CurDir->DosPath.Length / sizeof(WCHAR);
//     ASSERT((CurDirName != NULL) && (Length > 0));
//
//     /*
//      * DosPath.Buffer should always have a trailing slash. There is an assert
//      * below which checks for this.
//      *
//      * This function either returns x:\ for a root (keeping the original buffer)
//      * or it returns x:\path\foo for a directory (replacing the trailing slash
//      * with a NULL.
//      */
//     Bytes = Length * sizeof(WCHAR);
//     if ((Length <= 1) || (CurDirName[Length - 2] == L':'))
//     {
//         /* Check if caller does not have enough space */
//         if (MaximumLength <= Bytes)
//         {
//             /* Call has no space for it, fail, add the trailing slash */
//             RtlReleasePebLock();
//             return Bytes + sizeof(OBJ_NAME_PATH_SEPARATOR);
//         }
//     }
//     else
//     {
//         /* Check if caller does not have enough space */
//         if (MaximumLength < Bytes)
//         {
//             /* Call has no space for it, fail */
//             RtlReleasePebLock();
//             return Bytes;
//         }
//     }
//
//     /* Copy the buffer since we seem to have space */
//     RtlCopyMemory(Buffer, CurDirName, Bytes);
//
//     /* The buffer should end with a path separator */
//     ASSERT(Buffer[Length - 1] == OBJ_NAME_PATH_SEPARATOR);
//
//     /* Again check for our two cases (drive root vs path) */
//     if ((Length <= 1) || (Buffer[Length - 2] != L':'))
//     {
//         /* Replace the trailing slash with a null */
//         Buffer[Length - 1] = UNICODE_NULL;
//         --Length;
//     }
//     else
//     {
//         /* Append the null char since there's no trailing slash */
//         Buffer[Length] = UNICODE_NULL;
//     }
//
//     /* Release PEB lock */
//     RtlReleasePebLock();
//     DPRINT("CurrentDirectory %S\n", Buffer);
//     return Length * sizeof(WCHAR);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// RtlSetCurrentDirectory_U(IN PUNICODE_STRING Path)
// {
//     PCURDIR CurDir;
//     NTSTATUS Status;
//     RTL_PATH_TYPE PathType;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING FullPath, NtName;
//     PRTLP_CURDIR_REF OldCurDir = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     FILE_FS_DEVICE_INFORMATION FileFsDeviceInfo;
//     ULONG SavedLength, CharLength, FullPathLength;
//     HANDLE OldHandle = NULL, CurDirHandle = NULL, OldCurDirHandle = NULL;
//
//     DPRINT("RtlSetCurrentDirectory_U %wZ\n", Path);
//
//     /* Initialize for failure case */
//     RtlInitEmptyUnicodeString(&NtName, NULL, 0);
//
//     /* Can't set current directory on DOS device */
//     if (RtlIsDosDeviceName_Ustr(Path))
//     {
//         return STATUS_NOT_A_DIRECTORY;
//     }
//
//     /* Get current directory */
//     RtlAcquirePebLock();
//     CurDir = &NtCurrentPeb()->ProcessParameters->CurrentDirectory;
//
//     /* Check if we have to drop current handle */
//     if (((ULONG_PTR)(CurDir->Handle) & RTL_CURDIR_ALL_FLAGS) == RTL_CURDIR_DROP_OLD_HANDLE)
//     {
//         OldHandle = CurDir->Handle;
//         CurDir->Handle = NULL;
//     }
//
//     /* Allocate a buffer for full path (using max possible length */
//     FullPath.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, CurDir->DosPath.MaximumLength);
//     if (!FullPath.Buffer)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Leave;
//     }
//
//     /* Init string */
//     FullPath.Length = 0;
//     FullPath.MaximumLength = CurDir->DosPath.MaximumLength;
//
//     /* Get new directory full path */
//     FullPathLength = RtlGetFullPathName_Ustr(Path, FullPath.MaximumLength, FullPath.Buffer, NULL, NULL, &PathType);
//     if (!FullPathLength)
//     {
//         Status = STATUS_OBJECT_NAME_INVALID;
//         goto Leave;
//     }
//
//     SavedLength = FullPath.MaximumLength;
//     CharLength = FullPathLength / sizeof(WCHAR);
//
//     if (FullPathLength > FullPath.MaximumLength)
//     {
//         Status = STATUS_NAME_TOO_LONG;
//         goto Leave;
//     }
//
//     /* Translate it to NT name */
//     if (!RtlDosPathNameToNtPathName_U(FullPath.Buffer, &NtName, NULL, NULL))
//     {
//         Status = STATUS_OBJECT_NAME_INVALID;
//         goto Leave;
//     }
//
//    InitializeObjectAttributes(&ObjectAttributes, &NtName,
//                               OBJ_CASE_INSENSITIVE | OBJ_INHERIT,
//                               NULL, NULL);
//
//     /* If previous current directory was removable, then check it for dropping */
//     if (((ULONG_PTR)(CurDir->Handle) & RTL_CURDIR_ALL_FLAGS) == RTL_CURDIR_ALL_FLAGS)
//     {
//         /* Get back normal handle */
//         CurDirHandle = (HANDLE)((ULONG_PTR)(CurDir->Handle) & ~RTL_CURDIR_ALL_FLAGS);
//         CurDir->Handle = NULL;
//
//         /* Get device information */
//         Status = NtQueryVolumeInformationFile(CurDirHandle,
//                                               &IoStatusBlock,
//                                               &FileFsDeviceInfo,
//                                               sizeof(FileFsDeviceInfo),
//                                               FileFsDeviceInformation);
//         /* Retry without taking care of removable device */
//         if (!NT_SUCCESS(Status))
//         {
//             Status = RtlSetCurrentDirectory_U(Path);
//             goto Leave;
//         }
//     }
//     else
//     {
//         /* Open directory */
//         Status = NtOpenFile(&CurDirHandle,
//                             SYNCHRONIZE | FILE_TRAVERSE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE,
//                             FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
//         if (!NT_SUCCESS(Status)) goto Leave;
//
//         /* Get device information */
//         Status = NtQueryVolumeInformationFile(CurDirHandle,
//                                               &IoStatusBlock,
//                                               &FileFsDeviceInfo,
//                                               sizeof(FileFsDeviceInfo),
//                                               FileFsDeviceInformation);
//         if (!NT_SUCCESS(Status)) goto Leave;
//     }
//
//     /* If device is removable, mark handle */
//     if (FileFsDeviceInfo.Characteristics & FILE_REMOVABLE_MEDIA)
//     {
//         CurDirHandle = (HANDLE)((ULONG_PTR)CurDirHandle | RTL_CURDIR_IS_REMOVABLE);
//     }
//
//     FullPath.Length = (USHORT)FullPathLength;
//
//     /* If full path isn't \ terminated, do it */
//     if (FullPath.Buffer[CharLength - 1] != OBJ_NAME_PATH_SEPARATOR)
//     {
//         if ((CharLength + 1) * sizeof(WCHAR) > SavedLength)
//         {
//             Status = STATUS_NAME_TOO_LONG;
//             goto Leave;
//         }
//
//         FullPath.Buffer[CharLength] = OBJ_NAME_PATH_SEPARATOR;
//         FullPath.Buffer[CharLength + 1] = UNICODE_NULL;
//         FullPath.Length += sizeof(WCHAR);
//     }
//
//     /* If we have previous current directory with only us as reference, save it */
//     if (RtlpCurDirRef != NULL && RtlpCurDirRef->RefCount == 1)
//     {
//         OldCurDirHandle = RtlpCurDirRef->Handle;
//     }
//     else
//     {
//         /* Allocate new current directory struct saving previous one */
//         OldCurDir = RtlpCurDirRef;
//         RtlpCurDirRef = RtlAllocateHeap(RtlGetProcessHeap(), 0, sizeof(RTLP_CURDIR_REF));
//         if (!RtlpCurDirRef)
//         {
//             RtlpCurDirRef = OldCurDir;
//             OldCurDir = NULL;
//             Status = STATUS_NO_MEMORY;
//             goto Leave;
//         }
//
//         /* Set reference to 1 (us) */
//         RtlpCurDirRef->RefCount = 1;
//     }
//
//     /* Save new data */
//     CurDir->Handle = CurDirHandle;
//     RtlpCurDirRef->Handle = CurDirHandle;
//     CurDirHandle = NULL;
//
//     /* Copy full path */
//     RtlCopyMemory(CurDir->DosPath.Buffer, FullPath.Buffer, FullPath.Length + sizeof(WCHAR));
//     CurDir->DosPath.Length = FullPath.Length;
//
//     Status = STATUS_SUCCESS;
//
// Leave:
//     RtlReleasePebLock();
//
//     if (FullPath.Buffer)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FullPath.Buffer);
//     }
//
//     if (NtName.Buffer)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NtName.Buffer);
//     }
//
//     if (CurDirHandle) NtClose(CurDirHandle);
//
//     if (OldHandle) NtClose(OldHandle);
//
//     if (OldCurDirHandle) NtClose(OldCurDirHandle);
//
//     if (OldCurDir && InterlockedDecrement(&OldCurDir->RefCount) == 0)
//     {
//         NtClose(OldCurDir->Handle);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, OldCurDir);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// RtlGetFullPathName_UEx(
//     _In_ PWSTR FileName,
//     _In_ ULONG BufferLength,
//     _Out_writes_bytes_(BufferLength) PWSTR Buffer,
//     _Out_opt_ PWSTR *FilePart,
//     _Out_opt_ RTL_PATH_TYPE *InputPathType)
// {
//     UNICODE_STRING FileNameString;
//     NTSTATUS status;
//
//     if (InputPathType)
//         *InputPathType = 0;
//
//     /* Build the string */
//     status = RtlInitUnicodeStringEx(&FileNameString, FileName);
//     if (!NT_SUCCESS(status)) return 0;
//
//     /* Call the extended function */
//     return RtlGetFullPathName_Ustr(
//         &FileNameString,
//         BufferLength,
//         Buffer,
//         (PCWSTR*)FilePart,
//         NULL,
//         InputPathType);
// }
//
// /******************************************************************
//  *    RtlGetFullPathName_U  (NTDLL.@)
//  *
//  * Returns the number of bytes written to buffer (not including the
//  * terminating NULL) if the function succeeds, or the required number of bytes
//  * (including the terminating NULL) if the buffer is too small.
//  *
//  * file_part will point to the filename part inside buffer (except if we use
//  * DOS device name, in which case file_in_buf is NULL)
//  *
//  * @implemented
//  */
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// RtlGetFullPathName_U(
//     _In_ PCWSTR FileName,
//     _In_ ULONG Size,
//     _Out_z_bytecap_(Size) PWSTR Buffer,
//     _Out_opt_ PWSTR *ShortName)
// {
//     RTL_PATH_TYPE PathType;
//
//     /* Call the extended function */
//     return RtlGetFullPathName_UEx((PWSTR)FileName,
//                                    Size,
//                                    Buffer,
//                                    ShortName,
//                                    &PathType);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// RtlDosPathNameToNtPathName_U(IN PCWSTR DosName,
//                              OUT PUNICODE_STRING NtName,
//                              OUT PCWSTR *PartName,
//                              OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     /* Call the internal function */
//     return NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_U(FALSE,
//                                                             DosName,
//                                                             NtName,
//                                                             PartName,
//                                                             RelativeName));
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// RtlDosPathNameToNtPathName_U_WithStatus(IN PCWSTR DosName,
//                                         OUT PUNICODE_STRING NtName,
//                                         OUT PCWSTR *PartName,
//                                         OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     /* Call the internal function */
//     return RtlpDosPathNameToRelativeNtPathName_U(FALSE,
//                                                  DosName,
//                                                  NtName,
//                                                  PartName,
//                                                  RelativeName);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// RtlDosPathNameToRelativeNtPathName_U(IN PCWSTR DosName,
//                                      OUT PUNICODE_STRING NtName,
//                                      OUT PCWSTR *PartName,
//                                      OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     /* Call the internal function */
//     ASSERT(RelativeName);
//     return NT_SUCCESS(RtlpDosPathNameToRelativeNtPathName_U(TRUE,
//                                                             DosName,
//                                                             NtName,
//                                                             PartName,
//                                                             RelativeName));
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// RtlDosPathNameToRelativeNtPathName_U_WithStatus(IN PCWSTR DosName,
//                                                 OUT PUNICODE_STRING NtName,
//                                                 OUT PCWSTR *PartName,
//                                                 OUT PRTL_RELATIVE_NAME_U RelativeName)
// {
//     /* Call the internal function */
//     ASSERT(RelativeName);
//     return RtlpDosPathNameToRelativeNtPathName_U(TRUE,
//                                                  DosName,
//                                                  NtName,
//                                                  PartName,
//                                                  RelativeName);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS NTAPI RtlNtPathNameToDosPathName(IN ULONG Flags,
//                                           IN OUT PRTL_UNICODE_STRING_BUFFER Path,
//                                           OUT PULONG PathType,
//                                           PULONG Unknown)
// {
//     PCUNICODE_STRING UsePrefix = NULL, AlternatePrefix = NULL;
//
//     if (PathType)
//         *PathType = 0;
//
//     if (!Path || Flags)
//         return STATUS_INVALID_PARAMETER;
//
//     /* The initial check is done on Path->String */
//     if (RtlPrefixUnicodeString(&RtlpDosDevicesUncPrefix, &Path->String, TRUE))
//     {
//         UsePrefix = &RtlpDosDevicesUncPrefix;
//         AlternatePrefix = &RtlpDoubleSlashPrefix;
//         if (PathType)
//             *PathType = RTL_CONVERTED_UNC_PATH;
//     }
//     else if (RtlPrefixUnicodeString(&RtlpDosDevicesPrefix, &Path->String, FALSE))
//     {
//         UsePrefix = &RtlpDosDevicesPrefix;
//         if (PathType)
//             *PathType = RTL_CONVERTED_NT_PATH;
//     }
//
//     if (UsePrefix)
//     {
//         NTSTATUS Status;
//
//         USHORT Len = Path->String.Length - UsePrefix->Length;
//         if (AlternatePrefix)
//             Len += AlternatePrefix->Length;
//
//         Status = RtlEnsureBufferSize(0, &Path->ByteBuffer, Len);
//         if (!NT_SUCCESS(Status))
//             return Status;
//
//         if (Len + sizeof(UNICODE_NULL) <= Path->ByteBuffer.Size)
//         {
//             /* Then, the contents of Path->ByteBuffer are always used... */
//             if (AlternatePrefix)
//             {
//                 memcpy(Path->ByteBuffer.Buffer, AlternatePrefix->Buffer, AlternatePrefix->Length);
//                 memmove(Path->ByteBuffer.Buffer + AlternatePrefix->Length, Path->ByteBuffer.Buffer + UsePrefix->Length,
//                     Len - AlternatePrefix->Length);
//             }
//             else
//             {
//                 memmove(Path->ByteBuffer.Buffer, Path->ByteBuffer.Buffer + UsePrefix->Length, Len);
//             }
//             Path->String.Buffer = (PWSTR)Path->ByteBuffer.Buffer;
//             Path->String.Length = Len;
//             Path->String.MaximumLength = Path->ByteBuffer.Size;
//             Path->String.Buffer[Len / sizeof(WCHAR)] = UNICODE_NULL;
//         }
//         return STATUS_SUCCESS;
//     }
//
//     if (PathType)
//     {
//         switch (RtlDetermineDosPathNameType_Ustr(&Path->String))
//         {
//         case RtlPathTypeUncAbsolute:
//         case RtlPathTypeDriveAbsolute:
//         case RtlPathTypeLocalDevice:
//         case RtlPathTypeRootLocalDevice:
//             *PathType = RTL_UNCHANGED_DOS_PATH;
//             break;
//         case RtlPathTypeUnknown:
//         case RtlPathTypeDriveRelative:
//         case RtlPathTypeRooted:
//         case RtlPathTypeRelative:
//             *PathType = RTL_UNCHANGED_UNK_PATH;
//             break;
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// RtlDosSearchPath_U(IN PCWSTR Path,
//                    IN PCWSTR FileName,
//                    IN PCWSTR Extension,
//                    IN ULONG Size,
//                    IN PWSTR Buffer,
//                    OUT PWSTR *PartName)
// {
//     NTSTATUS Status;
//     ULONG ExtensionLength, Length, FileNameLength, PathLength;
//     UNICODE_STRING TempString;
//     PWCHAR NewBuffer, BufferStart;
//     PCWSTR p;
//
//     /* Check if this is an absolute path */
//     if (RtlDetermineDosPathNameType_U(FileName) != RtlPathTypeRelative)
//     {
//         /* Check if the file exists */
//         if (RtlDoesFileExists_UEx(FileName, TRUE))
//         {
//             /* Get the full name, which does the DOS lookup */
//             return RtlGetFullPathName_U(FileName, Size, Buffer, PartName);
//         }
//
//         /* Doesn't exist, so fail */
//         return 0;
//     }
//
//     /* Scan the filename */
//     p = FileName;
//     while (*p)
//     {
//         /* Looking for an extension */
//         if (*p == L'.')
//         {
//             /* No extension string needed -- it's part of the filename */
//             Extension = NULL;
//             break;
//         }
//
//         /* Next character */
//         p++;
//     }
//
//     /* Do we have an extension? */
//     if (!Extension)
//     {
//         /* Nope, don't worry about one */
//         ExtensionLength = 0;
//     }
//     else
//     {
//         /* Build a temporary string to get the extension length */
//         Status = RtlInitUnicodeStringEx(&TempString, Extension);
//         if (!NT_SUCCESS(Status)) return 0;
//         ExtensionLength = TempString.Length;
//     }
//
//     /* Build a temporary string to get the path length */
//     Status = RtlInitUnicodeStringEx(&TempString, Path);
//     if (!NT_SUCCESS(Status)) return 0;
//     PathLength = TempString.Length;
//
//     /* Build a temporary string to get the filename length */
//     Status = RtlInitUnicodeStringEx(&TempString, FileName);
//     if (!NT_SUCCESS(Status)) return 0;
//     FileNameLength = TempString.Length;
//
//     /* Allocate the buffer for the new string name */
//     NewBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                 0,
//                                 FileNameLength +
//                                 ExtensionLength +
//                                 PathLength +
//                                 3 * sizeof(WCHAR));
//     if (!NewBuffer)
//     {
//         /* Fail the call */
//         DbgPrint("%s: Failing due to out of memory (RtlAllocateHeap failure)\n",
//                  __FUNCTION__);
//         return 0;
//     }
//
//     /* Final loop to build the path */
//     while (TRUE)
//     {
//         /* Check if we have a valid character */
//         BufferStart = NewBuffer;
//         if (*Path)
//         {
//             /* Loop as long as there's no semicolon */
//             while (*Path != L';')
//             {
//                 /* Copy the next character */
//                 *BufferStart++ = *Path++;
//                 if (!*Path) break;
//             }
//
//             /* We found a semi-colon, to stop path processing on this loop */
//             if (*Path == L';') ++Path;
//         }
//
//         /* Add a terminating slash if needed */
//         if ((BufferStart != NewBuffer) && (BufferStart[-1] != OBJ_NAME_PATH_SEPARATOR))
//         {
//             *BufferStart++ = OBJ_NAME_PATH_SEPARATOR;
//         }
//
//         /* Bail out if we reached the end */
//         if (!*Path) Path = NULL;
//
//         /* Copy the file name and check if an extension is needed */
//         RtlCopyMemory(BufferStart, FileName, FileNameLength);
//         if (ExtensionLength)
//         {
//             /* Copy the extension too */
//             RtlCopyMemory((PCHAR)BufferStart + FileNameLength,
//                           Extension,
//                           ExtensionLength + sizeof(WCHAR));
//         }
//         else
//         {
//             /* Just NULL-terminate */
//             *(PWCHAR)((PCHAR)BufferStart + FileNameLength) = UNICODE_NULL;
//         }
//
//         /* Now, does this file exist? */
//         if (RtlDoesFileExists_UEx(NewBuffer, FALSE))
//         {
//             /* Call the full-path API to get the length */
//             Length = RtlGetFullPathName_U(NewBuffer, Size, Buffer, PartName);
//             break;
//         }
//
//         /* If we got here, path doesn't exist, so fail the call */
//         Length = 0;
//         if (!Path) break;
//     }
//
//     /* Free the allocation and return the length */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer);
//     return Length;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// RtlGetFullPathName_UstrEx(IN PUNICODE_STRING FileName,
//                           IN PUNICODE_STRING StaticString,
//                           IN PUNICODE_STRING DynamicString,
//                           IN PUNICODE_STRING *StringUsed,
//                           IN PSIZE_T FilePartSize,
//                           OUT PBOOLEAN NameInvalid,
//                           OUT RTL_PATH_TYPE* PathType,
//                           OUT PSIZE_T LengthNeeded)
// {
//     NTSTATUS Status;
//     PWCHAR StaticBuffer;
//     PCWCH ShortName;
//     ULONG Length;
//     USHORT StaticLength;
//     UNICODE_STRING TempDynamicString;
//
//     /* Initialize all our locals */
//     ShortName = NULL;
//     StaticBuffer = NULL;
//     TempDynamicString.Buffer = NULL;
//
//     /* Initialize the input parameters */
//     if (StringUsed) *StringUsed = NULL;
//     if (LengthNeeded) *LengthNeeded = 0;
//     if (FilePartSize) *FilePartSize = 0;
//
//     /* Check for invalid parameters */
//     if ((DynamicString) && !(StringUsed) && (StaticString))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if we did not get an input string */
//     if (!StaticString)
//     {
//         /* Allocate one */
//         StaticLength = MAX_PATH * sizeof(WCHAR);
//         StaticBuffer = RtlpAllocateStringMemory(MAX_PATH * sizeof(WCHAR), TAG_USTR);
//         if (!StaticBuffer) return STATUS_NO_MEMORY;
//     }
//     else
//     {
//         /* Use the one we received */
//         StaticBuffer = StaticString->Buffer;
//         StaticLength = StaticString->MaximumLength;
//     }
//
//     /* Call the lower-level function */
//     Length = RtlGetFullPathName_Ustr(FileName,
//                                      StaticLength,
//                                      StaticBuffer,
//                                      &ShortName,
//                                      NameInvalid,
//                                      PathType);
//     DPRINT("Length: %u StaticBuffer: %S\n", Length, StaticBuffer);
//     if (!Length)
//     {
//         /* Fail if it failed */
//         DbgPrint("%s(%d) - RtlGetFullPathName_Ustr() returned 0\n",
//                  __FUNCTION__,
//                  __LINE__);
//         Status = STATUS_OBJECT_NAME_INVALID;
//         goto Quickie;
//     }
//
//     /* Check if it fits inside our static string */
//     if ((StaticString) && (Length < StaticLength))
//     {
//         /* Set the final length */
//         StaticString->Length = (USHORT)Length;
//
//         /* Set the file part size */
//         if (FilePartSize) *FilePartSize = ShortName ? (ShortName - StaticString->Buffer) : 0;
//
//         /* Return the static string if requested */
//         if (StringUsed) *StringUsed = StaticString;
//
//         /* We are done with success */
//         Status = STATUS_SUCCESS;
//         goto Quickie;
//     }
//
//     /* Did we not have an input dynamic string ?*/
//     if (!DynamicString)
//     {
//         /* Return the length we need */
//         if (LengthNeeded) *LengthNeeded = Length;
//
//         /* And fail such that the caller can try again */
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Quickie;
//     }
//
//     /* Check if it fits in our static buffer */
//     if ((StaticBuffer) && (Length < StaticLength))
//     {
//         /* NULL-terminate it */
//         StaticBuffer[Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//         /* Set the settings for the dynamic string the caller sent */
//         DynamicString->MaximumLength = StaticLength;
//         DynamicString->Length = (USHORT)Length;
//         DynamicString->Buffer = StaticBuffer;
//
//         /* Set the part size */
//         if (FilePartSize) *FilePartSize = ShortName ? (ShortName - StaticBuffer) : 0;
//
//         /* Return the dynamic string if requested */
//         if (StringUsed) *StringUsed = DynamicString;
//
//         /* Do not free the static buffer on exit, and return success */
//         StaticBuffer = NULL;
//         Status = STATUS_SUCCESS;
//         goto Quickie;
//     }
//
//     /* Now try again under the PEB lock */
//     RtlAcquirePebLock();
//     Length = RtlGetFullPathName_Ustr(FileName,
//                                      StaticLength,
//                                      StaticBuffer,
//                                      &ShortName,
//                                      NameInvalid,
//                                      PathType);
//     if (!Length)
//     {
//         /* It failed */
//         DbgPrint("%s line %d: RtlGetFullPathName_Ustr() returned 0\n",
//                  __FUNCTION__, __LINE__);
//         Status = STATUS_OBJECT_NAME_INVALID;
//         goto Release;
//     }
//
//     /* Check if it fits inside our static string now */
//     if ((StaticString) && (Length < StaticLength))
//     {
//         /* Set the final length */
//         StaticString->Length = (USHORT)Length;
//
//         /* Set the file part size */
//         if (FilePartSize) *FilePartSize = ShortName ? (ShortName - StaticString->Buffer) : 0;
//
//         /* Return the static string if requested */
//         if (StringUsed) *StringUsed = StaticString;
//
//         /* We are done with success */
//         Status = STATUS_SUCCESS;
//         goto Release;
//     }
//
//     /* Check if the path won't even fit in a real string */
//     if ((Length + sizeof(WCHAR)) > UNICODE_STRING_MAX_BYTES)
//     {
//         /* Name is way too long, fail */
//         Status = STATUS_NAME_TOO_LONG;
//         goto Release;
//     }
//
//     /* Allocate the string to hold the path name now */
//     TempDynamicString.Buffer = RtlpAllocateStringMemory(Length + sizeof(WCHAR),
//                                                         TAG_USTR);
//     if (!TempDynamicString.Buffer)
//     {
//         /* Out of memory, fail */
//         Status = STATUS_NO_MEMORY;
//         goto Release;
//     }
//
//     /* Add space for a NULL terminator, and now check the full path */
//     TempDynamicString.MaximumLength = (USHORT)Length + sizeof(UNICODE_NULL);
//     Length = RtlGetFullPathName_Ustr(FileName,
//                                      Length,
//                                      TempDynamicString.Buffer,
//                                      &ShortName,
//                                      NameInvalid,
//                                      PathType);
//     if (!Length)
//     {
//         /* Some path error, so fail out */
//         DbgPrint("%s line %d: RtlGetFullPathName_Ustr() returned 0\n",
//                  __FUNCTION__, __LINE__);
//         Status = STATUS_OBJECT_NAME_INVALID;
//         goto Release;
//     }
//
//     /* It should fit in the string we just allocated */
//     ASSERT(Length < (TempDynamicString.MaximumLength - sizeof(WCHAR)));
//     if (Length > TempDynamicString.MaximumLength)
//     {
//         /* This is really weird and would mean some kind of race */
//         Status = STATUS_INTERNAL_ERROR;
//         goto Release;
//     }
//
//     /* Return the file part size */
//     if (FilePartSize) *FilePartSize = ShortName ? (ShortName - TempDynamicString.Buffer) : 0;
//
//     /* Terminate the whole string now */
//     TempDynamicString.Buffer[Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Finalize the string and return it to the user */
//     DynamicString->Buffer = TempDynamicString.Buffer;
//     DynamicString->Length = (USHORT)Length;
//     DynamicString->MaximumLength = TempDynamicString.MaximumLength;
//     if (StringUsed) *StringUsed = DynamicString;
//
//     /* Return success and make sure we don't free the buffer on exit */
//     TempDynamicString.Buffer = NULL;
//     Status = STATUS_SUCCESS;
//
// Release:
//     /* Release the PEB lock */
//     RtlReleasePebLock();
//
// Quickie:
//     /* Free any buffers we should be freeing */
//     DPRINT("Status: %lx %S %S\n", Status, StaticBuffer, TempDynamicString.Buffer);
//     if ((StaticString) && (StaticBuffer) && (StaticBuffer != StaticString->Buffer))
//     {
//         RtlpFreeStringMemory(StaticBuffer, TAG_USTR);
//     }
//     if (TempDynamicString.Buffer)
//     {
//         RtlpFreeStringMemory(TempDynamicString.Buffer, TAG_USTR);
//     }
//
//     /* Print out any unusual errors */
//     if ((NT_ERROR(Status)) &&
//         (Status != STATUS_NO_SUCH_FILE) && (Status != STATUS_BUFFER_TOO_SMALL))
//     {
//         DbgPrint("RTL: %s - failing on filename %wZ with status %08lx\n",
//                 __FUNCTION__, FileName, Status);
//     }
//
//     /* Return, we're all done */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// RtlDosSearchPath_Ustr(IN ULONG Flags,
//                       IN PUNICODE_STRING PathString,
//                       IN PUNICODE_STRING FileNameString,
//                       IN PUNICODE_STRING ExtensionString,
//                       IN PUNICODE_STRING CallerBuffer,
//                       IN OUT PUNICODE_STRING DynamicString OPTIONAL,
//                       OUT PUNICODE_STRING* FullNameOut OPTIONAL,
//                       OUT PSIZE_T FilePartSize OPTIONAL,
//                       OUT PSIZE_T LengthNeeded OPTIONAL)
// {
//     WCHAR StaticCandidateBuffer[MAX_PATH];
//     UNICODE_STRING StaticCandidateString;
//     NTSTATUS Status;
//     RTL_PATH_TYPE PathType;
//     PWCHAR p, End, CandidateEnd, SegmentEnd;
//     SIZE_T WorstCaseLength, NamePlusExtLength, SegmentSize, ByteCount, PathSize, MaxPathSize = 0;
//     USHORT ExtensionLength = 0;
//     PUNICODE_STRING FullIsolatedPath;
//     DPRINT("DOS Path Search: %lx %wZ %wZ %wZ %wZ %wZ\n",
//             Flags, PathString, FileNameString, ExtensionString, CallerBuffer, DynamicString);
//
//     /* Initialize the input string */
//     RtlInitEmptyUnicodeString(&StaticCandidateString,
//                               StaticCandidateBuffer,
//                               sizeof(StaticCandidateBuffer));
//
//     /* Initialize optional arguments */
//     if (FullNameOut ) *FullNameOut  = NULL;
//     if (FilePartSize) *FilePartSize = 0;
//     if (LengthNeeded) *LengthNeeded = 0;
//     if (DynamicString)
//     {
//         DynamicString->Length = DynamicString->MaximumLength = 0;
//         DynamicString->Buffer = NULL;
//     }
//
//     /* Check for invalid parameters */
//     if ((Flags & ~7) ||
//         !(PathString) ||
//         !(FileNameString) ||
//         ((CallerBuffer) && (DynamicString) && !(FullNameOut)))
//     {
//         /* Fail */
//         DbgPrint("%s: Invalid parameters passed\n", __FUNCTION__);
//         Status = STATUS_INVALID_PARAMETER;
//         goto Quickie;
//     }
//
//     /* First check what kind of path this is */
//     PathType = RtlDetermineDosPathNameType_Ustr(FileNameString);
//
//     /* Check if the caller wants to prevent relative .\ and ..\ paths */
//     if ((Flags & 2) &&
//          (PathType == RtlPathTypeRelative) &&
//          (FileNameString->Length >= (2 * sizeof(WCHAR))) &&
//          (FileNameString->Buffer[0] == L'.') &&
//          ((IS_PATH_SEPARATOR(FileNameString->Buffer[1])) ||
//           ((FileNameString->Buffer[1] == L'.') &&
//            ((FileNameString->Length >= (3 * sizeof(WCHAR))) &&
//            (IS_PATH_SEPARATOR(FileNameString->Buffer[2]))))))
//     {
//         /* Yes, and this path is like that, so make it seem unknown */
//         PathType = RtlPathTypeUnknown;
//     }
//
//     /* Now check relative vs non-relative paths */
//     if (PathType == RtlPathTypeRelative)
//     {
// #ifndef _NTOSKRNL_
//         /* Does the caller want SxS? (user-mode only — SxS / activation
//          * contexts do not exist in the kernel, and
//          * RtlDosApplyFileIsolationRedirection_Ustr is an ntdll-only
//          * symbol, so guard the whole branch out of the _NTOSKRNL_ build
//          * to keep path.c.obj link-clean in ntoskrnl.) */
//         if (Flags & 1)
//         {
//             /* Apply the SxS magic */
//             FullIsolatedPath = NULL;
//             Status = RtlDosApplyFileIsolationRedirection_Ustr(TRUE,
//                                                               FileNameString,
//                                                               ExtensionString,
//                                                               CallerBuffer,
//                                                               DynamicString,
//                                                               &FullIsolatedPath,
//                                                               NULL,
//                                                               FilePartSize,
//                                                               LengthNeeded);
//             if (NT_SUCCESS(Status))
//             {
//                 /* We found the SxS path, return it */
//                 if (FullNameOut) *FullNameOut = FullIsolatedPath;
//                 goto Quickie;
//             }
//             else if (Status != STATUS_SXS_KEY_NOT_FOUND)
//             {
//                 /* Critical SxS error, fail */
//                 DbgPrint("%s: Failing because call to "
//                          "RtlDosApplyIsolationRedirection_Ustr(%wZ) failed with "
//                           "status 0x%08lx\n",
//                          __FUNCTION__,
//                          FileNameString,
//                          Status);
//                 goto Quickie;
//             }
//         }
// #endif /* !_NTOSKRNL_ */
//
//         /* No SxS key found, or not requested, check if there's an extension */
//         if (ExtensionString)
//         {
//             /* Save the extension length, and check if there's a file name */
//             ExtensionLength = ExtensionString->Length;
//             if (FileNameString->Length)
//             {
//                 /* Start parsing the file name */
//                 End = &FileNameString->Buffer[FileNameString->Length / sizeof(WCHAR)];
//                 while (End > FileNameString->Buffer)
//                 {
//                     /* If we find a path separator, there's no extension */
//                     --End;
//                     if (IS_PATH_SEPARATOR(*End)) break;
//
//                     /* Otherwise, did we find an extension dot? */
//                     if (*End == L'.')
//                     {
//                         /* Ignore what the caller sent it, use the filename's */
//                         ExtensionString = NULL;
//                         ExtensionLength = 0;
//                         break;
//                     }
//                 }
//             }
//         }
//
//         /* Check if we got a path */
//         if (PathString->Length)
//         {
//             /* Start parsing the path name, looking for path separators */
//             End = &PathString->Buffer[PathString->Length / sizeof(WCHAR)];
//             p = End;
//             while (p > PathString->Buffer)
//             {
//                 if (*--p == L';')
//                 {
//                     /* This is the size of the path -- handle a trailing slash */
//                     PathSize = End - p - 1;
//                     if ((PathSize) && !(IS_PATH_SEPARATOR(*(End - 1)))) PathSize++;
//
//                     /* Check if we found a bigger path than before */
//                     if (PathSize > MaxPathSize) MaxPathSize = PathSize;
//
//                     /* Keep going with the path after this path separator */
//                     End = p;
//                 }
//             }
//
//             /* This is the trailing path, run the same code as above */
//             PathSize = End - p;
//             if ((PathSize) && !(IS_PATH_SEPARATOR(*(End - 1)))) PathSize++;
//             if (PathSize > MaxPathSize) MaxPathSize = PathSize;
//
//             /* Finally, convert the largest path size into WCHAR */
//             MaxPathSize *= sizeof(WCHAR);
//         }
//
//         /* Use the extension, the file name, and the largest path as the size */
//         WorstCaseLength = ExtensionLength +
//                           FileNameString->Length +
//                           (USHORT)MaxPathSize +
//                           sizeof(UNICODE_NULL);
//         if (WorstCaseLength > UNICODE_STRING_MAX_BYTES)
//         {
//             /* It has to fit in a registry string, if not, fail here */
//             DbgPrint("%s returning STATUS_NAME_TOO_LONG because the computed "
//                      "worst case file name length is %Iu bytes\n",
//                      __FUNCTION__,
//                      WorstCaseLength);
//             Status = STATUS_NAME_TOO_LONG;
//             goto Quickie;
//         }
//
//         /* Scan the path now, to see if we can find the file */
//         p = PathString->Buffer;
//         End = &p[PathString->Length / sizeof(WCHAR)];
//         while (p < End)
//         {
//             /* Find out where this path ends */
//             for (SegmentEnd = p;
//                  ((SegmentEnd != End) && (*SegmentEnd != L';'));
//                  SegmentEnd++);
//
//             /* Compute the size of this path */
//             ByteCount = SegmentSize = (SegmentEnd - p) * sizeof(WCHAR);
//
//             /* Handle trailing slash if there isn't one */
//             if ((SegmentSize) && !(IS_PATH_SEPARATOR(*(SegmentEnd - 1))))
//             {
//                 /* Add space for one */
//                 SegmentSize += sizeof(OBJ_NAME_PATH_SEPARATOR);
//             }
//
//             /* Now check if our initial static buffer is too small */
//             if (StaticCandidateString.MaximumLength <
//                 (SegmentSize + ExtensionLength + FileNameString->Length + sizeof(UNICODE_NULL)))
//             {
//                 /* At this point we should've been using our static buffer */
//                 ASSERT(StaticCandidateString.Buffer == StaticCandidateBuffer);
//                 if (StaticCandidateString.Buffer != StaticCandidateBuffer)
//                 {
//                     /* Something is really messed up if this was the dynamic string */
//                     DbgPrint("%s: internal error #1; "
//                              "CandidateString.Buffer = %p; "
//                              "StaticCandidateBuffer = %p\n",
//                             __FUNCTION__,
//                             StaticCandidateString.Buffer,
//                             StaticCandidateBuffer);
//                     Status = STATUS_INTERNAL_ERROR;
//                     goto Quickie;
//                 }
//
//                 /* We checked before that the maximum possible size shoudl fit! */
//                 ASSERT((SegmentSize + FileNameString->Length + ExtensionLength) <
//                         UNICODE_STRING_MAX_BYTES);
//                 if ((SegmentSize + ExtensionLength + FileNameString->Length) >
//                     (UNICODE_STRING_MAX_BYTES - sizeof(WCHAR)))
//                 {
//                     /* For some reason it's not fitting anymore. Something messed up */
//                     DbgPrint("%s: internal error #2; SegmentSize = %u, "
//                              "FileName->Length = %u, DefaultExtensionLength = %u\n",
//                              __FUNCTION__,
//                              SegmentSize,
//                              FileNameString->Length,
//                              ExtensionLength);
//                     Status = STATUS_INTERNAL_ERROR;
//                     goto Quickie;
//                 }
//
//                 /* Now allocate the dynamic string */
//                 StaticCandidateString.MaximumLength = WorstCaseLength;
//                 StaticCandidateString.Buffer = RtlpAllocateStringMemory(WorstCaseLength,
//                                                                         TAG_USTR);
//                 if (!StaticCandidateString.Buffer)
//                 {
//                     /* Out of memory, fail */
//                     DbgPrint("%s: Unable to allocate %u byte buffer for path candidate\n",
//                              __FUNCTION__,
//                              StaticCandidateString.MaximumLength);
//                     Status = STATUS_NO_MEMORY;
//                     goto Quickie;
//                 }
//             }
//
//             /* Copy the path in the string */
//             RtlCopyMemory(StaticCandidateString.Buffer, p, ByteCount);
//
//             /* Get to the end of the string, and add the trailing slash if missing */
//             CandidateEnd = &StaticCandidateString.Buffer[ByteCount / sizeof(WCHAR)];
//             if ((SegmentSize) && (SegmentSize != ByteCount))
//             {
//                 *CandidateEnd++ = OBJ_NAME_PATH_SEPARATOR;
//             }
//
//             /* Copy the filename now */
//             RtlCopyMemory(CandidateEnd,
//                           FileNameString->Buffer,
//                           FileNameString->Length);
//             CandidateEnd += (FileNameString->Length / sizeof(WCHAR));
//
//             /* Check if there was an extension */
//             if (ExtensionString)
//             {
//                 /* Copy the extension too */
//                 RtlCopyMemory(CandidateEnd,
//                               ExtensionString->Buffer,
//                               ExtensionString->Length);
//                           CandidateEnd += (ExtensionString->Length / sizeof(WCHAR));
//             }
//
//             /* We are done, terminate it */
//             *CandidateEnd = UNICODE_NULL;
//
//             /* Now set the final length of the string so it becomes valid */
//             StaticCandidateString.Length = (USHORT)(CandidateEnd -
//                                             StaticCandidateString.Buffer) *
//                                            sizeof(WCHAR);
//             ASSERT(StaticCandidateString.Length < StaticCandidateString.MaximumLength);
//
//             /* Check if this file exists */
//             DPRINT("BUFFER: %S\n", StaticCandidateString.Buffer);
//             if (RtlDoesFileExists_UEx(StaticCandidateString.Buffer, FALSE))
//             {
//                 /* Awesome, it does, now get the full path */
//                 Status = RtlGetFullPathName_UstrEx(&StaticCandidateString,
//                                                    CallerBuffer,
//                                                    DynamicString,
//                                                    (PUNICODE_STRING*)FullNameOut,
//                                                    FilePartSize,
//                                                    NULL,
//                                                    &PathType,
//                                                    LengthNeeded);
//                 if (!(NT_SUCCESS(Status)) &&
//                     ((Status != STATUS_NO_SUCH_FILE) &&
//                      (Status != STATUS_BUFFER_TOO_SMALL)))
//                 {
//                     DbgPrint("%s: Failing because we thought we found %wZ on "
//                              "the search path, but RtlGetfullPathNameUStrEx() "
//                              "returned %08lx\n",
//                              __FUNCTION__,
//                              &StaticCandidateString,
//                              Status);
//                 }
//                 DPRINT("Status: %lx BUFFER: %S\n", Status, CallerBuffer->Buffer);
//                 goto Quickie;
//             }
//             else
//             {
//                 /* Otherwise, move to the next path */
//                 if (SegmentEnd != End)
//                 {
//                     /* Handle the case of the path separator trailing */
//                     p = SegmentEnd + 1;
//                 }
//                 else
//                 {
//                     p = SegmentEnd;
//                 }
//             }
//         }
//
//         /* Loop finished and we didn't break out -- fail */
//         Status = STATUS_NO_SUCH_FILE;
//     }
//     else
//     {
//         /* We have a full path, so check if it does exist */
//         DPRINT("%wZ\n", FileNameString);
//         if (!RtlDoesFileExists_UstrEx(FileNameString, TRUE))
//         {
//             /* It doesn't exist, did we have an extension? */
//             if (!(ExtensionString) || !(ExtensionString->Length))
//             {
//                 /* No extension, so just fail */
//                 Status = STATUS_NO_SUCH_FILE;
//                 goto Quickie;
//             }
//
//             /* There was an extension, check if the filename already had one */
//             if (!(Flags & 4) && (FileNameString->Length))
//             {
//                 /* Parse the filename */
//                 p = FileNameString->Buffer;
//                 End = &p[FileNameString->Length / sizeof(WCHAR)];
//                 while (End > p)
//                 {
//                     /* If there's a path separator, there's no extension */
//                     --End;
//                     if (IS_PATH_SEPARATOR(*End)) break;
//
//                     /* Othwerwise, did we find an extension dot? */
//                     if (*End == L'.')
//                     {
//                         /* File already had an extension, so fail */
//                         Status = STATUS_NO_SUCH_FILE;
//                         goto Quickie;
//                     }
//                 }
//             }
//
//             /* So there is an extension, we'll try again by adding it */
//             NamePlusExtLength = FileNameString->Length +
//                                 ExtensionString->Length +
//                                 sizeof(UNICODE_NULL);
//             if (NamePlusExtLength > UNICODE_STRING_MAX_BYTES)
//             {
//                 /* It won't fit in any kind of valid string, so fail */
//                 DbgPrint("%s: Failing because filename plus extension (%Iu bytes) is too big\n",
//                          __FUNCTION__,
//                          NamePlusExtLength);
//                 Status = STATUS_NAME_TOO_LONG;
//                 goto Quickie;
//             }
//
//             /* Fill it fit in our temporary string? */
//             if (NamePlusExtLength > StaticCandidateString.MaximumLength)
//             {
//                 /* It won't fit anymore, allocate a dynamic string for it */
//                 StaticCandidateString.MaximumLength = NamePlusExtLength;
//                 StaticCandidateString.Buffer = RtlpAllocateStringMemory(NamePlusExtLength,
//                                                                         TAG_USTR);
//                 if (!StaticCandidateString.Buffer)
//                 {
//                     /* Ran out of memory, so fail */
//                     DbgPrint("%s: Failing because allocating the dynamic filename buffer failed\n",
//                              __FUNCTION__);
//                     Status = STATUS_NO_MEMORY;
//                     goto Quickie;
//                 }
//             }
//
//             /* Copy the filename */
//             RtlCopyUnicodeString(&StaticCandidateString, FileNameString);
//
//             /* Copy the extension */
//             RtlAppendUnicodeStringToString(&StaticCandidateString,
//                                            ExtensionString);
//
//             DPRINT("SB: %wZ\n", &StaticCandidateString);
//
//             /* And check if this file now exists */
//             if (!RtlDoesFileExists_UstrEx(&StaticCandidateString, TRUE))
//             {
//                 /* Still no joy, fail out */
//                 Status = STATUS_NO_SUCH_FILE;
//                 goto Quickie;
//             }
//
//             /* File was found, get the final full path */
//             Status = RtlGetFullPathName_UstrEx(&StaticCandidateString,
//                                                CallerBuffer,
//                                                DynamicString,
//                                                (PUNICODE_STRING*)FullNameOut,
//                                                FilePartSize,
//                                                NULL,
//                                                &PathType,
//                                                LengthNeeded);
//             if (!(NT_SUCCESS(Status)) && (Status != STATUS_NO_SUCH_FILE))
//             {
//                 DbgPrint("%s: Failing on \"%wZ\" because RtlGetfullPathNameUStrEx() "
//                          "failed with status %08lx\n",
//                          __FUNCTION__,
//                          &StaticCandidateString,
//                          Status);
//             }
//             DPRINT("Status: %lx BUFFER: %S\n", Status, CallerBuffer->Buffer);
//         }
//         else
//         {
//             /* File was found on the first try, get the final full path */
//             Status = RtlGetFullPathName_UstrEx(FileNameString,
//                                                CallerBuffer,
//                                                DynamicString,
//                                                (PUNICODE_STRING*)FullNameOut,
//                                                FilePartSize,
//                                                NULL,
//                                                &PathType,
//                                                LengthNeeded);
//             if (!(NT_SUCCESS(Status)) &&
//                 ((Status != STATUS_NO_SUCH_FILE) &&
//                 (Status != STATUS_BUFFER_TOO_SMALL)))
//             {
//                 DbgPrint("%s: Failing because RtlGetfullPathNameUStrEx() on %wZ "
//                          "failed with status %08lx\n",
//                          __FUNCTION__,
//                          FileNameString,
//                          Status);
//             }
//             DPRINT("Status: %lx BUFFER: %S\n", Status, CallerBuffer->Buffer);
//         }
//     }
//
// Quickie:
//     /* Anything that was not an error, turn into STATUS_SUCCESS */
//     if (NT_SUCCESS(Status)) Status = STATUS_SUCCESS;
//
//     /* Check if we had a dynamic string */
//     if ((StaticCandidateString.Buffer) &&
//         (StaticCandidateString.Buffer != StaticCandidateBuffer))
//     {
//         /* Free it */
//         RtlFreeUnicodeString(&StaticCandidateString);
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// RtlDoesFileExists_U(IN PCWSTR FileName)
// {
//     /* Call the new function */
//     return RtlDoesFileExists_UEx(FileName, TRUE);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const APPCOMPAT_LAYER_KEY: [WCHAR; 68] = [
    '\\' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    's' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const REGISTRY_MACHINE: [WCHAR; 18] = [
    '\\' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const SPACE_ONLY: [WCHAR; 2] = [' ' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

pub const DISALLOWED_LAYER_CHARS: [WCHAR; 4] = [
    ' ' as i32 as WCHAR,
    '#' as i32 as WCHAR,
    '!' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const LAYER_SEPARATORS: [WCHAR; 3] = [
    ' ' as i32 as WCHAR,
    '\t' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const SIGN_MEDIA_FMT: [WCHAR; 17] = [
    'S' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    '=' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'X' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut g_QueryFlag: ACCESS_MASK = 0xffffffff as ACCESS_MASK;

pub const GPLK_USER: u32 = 1;

pub const GPLK_MACHINE: u32 = 2;

pub const MAX_LAYER_LENGTH: u32 = 256;

pub const LAYER_APPLY_TO_SYSTEM_EXES: u32 = 1;

pub const LAYER_UNK_FLAG2: u32 = 2;

// REG_SZ: already imported from ntkrnl_sys::constants

#[repr(C)]
pub struct SDB_TMP_STR {
    pub Str: _UNICODE_STRING,
    pub FixedBuffer: [u16; 260], // WCHAR FixedBuffer[MAX_PATH]; MAX_PATH = 260
}
pub type PSDB_TMP_STR = *mut SDB_TMP_STR;
#[no_mangle]
pub unsafe extern "C" fn SdbpInitTempStr(String: PSDB_TMP_STR) {
    (*String).Str.Buffer = core::ptr::addr_of_mut!((*String).FixedBuffer) as *mut WCHAR as PWSTR;
    (*String).Str.Length = 0 as USHORT;
    (*String).Str.MaximumLength = core::mem::size_of::<[WCHAR; 260]>() as USHORT;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpFreeTempStr(String: PSDB_TMP_STR) {
    if (*String).Str.Buffer != core::ptr::addr_of_mut!((*String).FixedBuffer) as PWSTR {
        SdbFree!((*String).Str.Buffer as LPVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SdbpResizeTempStr(String: PSDB_TMP_STR, newLength: WORD) {
    if newLength > (*String).Str.MaximumLength {
        SdbpFreeTempStr(String);
        (*String).Str.MaximumLength = ((newLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as USHORT;
        (*String).Str.Buffer = SdbAlloc!((*String).Str.MaximumLength as usize) as PWSTR;
        (*String).Str.Length = 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SdbpGetLongPathName(wszPath: PCWSTR, Result: PSDB_TMP_STR) -> BOOL {
    let mut max: DWORD = (*Result).Str.MaximumLength as DWORD / 2;
    let mut ret: DWORD = GetLongPathNameW(wszPath as LPCWSTR, (*Result).Str.Buffer as LPWSTR, max);
    if ret != 0 {
        if ret >= max {
            SdbpResizeTempStr(Result, ret as WORD);
            max = (*Result).Str.MaximumLength as DWORD / 2;
            ret = GetLongPathNameW(wszPath as LPCWSTR, (*Result).Str.Buffer as LPWSTR, max);
        }
        if ret != 0 && ret < max {
            (*Result).Str.Length = ret.wrapping_mul(2) as USHORT;
            return TRUE as BOOL;
        }
    }
    SHIM_ERR!(
        b"Failed to convert short path to long path error 0x%lx\n\0".as_ptr() as PCSTR,
        GetLastError(),
    );
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpIsPathOnRemovableMedia(Path: PCWSTR) -> BOOL {
    let mut tmp: [WCHAR; 4] = [0x0041u16, 0x003A, 0x005C, 0x0000];
    let mut type_0: ULONG = 0;
    if Path.is_null() || *Path.add(0) == UNICODE_NULL {
        SHIM_ERR!(b"Invalid argument\n\0".as_ptr() as PCSTR);
        return (FALSE) as BOOL;
    }
    match *Path.add(1) {
        0x003Au16 => {}
        0x005Cu16 => {
            SHIM_INFO!(b"\"%S\" is a network path.\n\0".as_ptr() as PCSTR, Path);
            return (FALSE) as BOOL;
        }
        _ => {
            SHIM_INFO!(b"\"%S\" not a full path we can operate on.\n\0".as_ptr() as PCSTR, Path);
            return (FALSE) as BOOL;
        }
    }
    tmp[0] = *Path.add(0);
    type_0 = GetDriveTypeW(core::ptr::addr_of_mut!(tmp) as *mut WCHAR as LPCWSTR) as ULONG;

    return (type_0 == DRIVE_REMOVABLE || type_0 == DRIVE_CDROM) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpBuildSignMediaId(LongPath: PSDB_TMP_STR) -> BOOL {
    let mut Scratch: SDB_TMP_STR = core::mem::zeroed();
    let mut Ptr: PWCHAR = core::ptr::null_mut();
    SdbpInitTempStr(core::ptr::addr_of_mut!(Scratch));
    SdbpResizeTempStr(
        core::ptr::addr_of_mut!(Scratch),
        (((*LongPath).Str.Length as usize) / core::mem::size_of::<WCHAR>()
            + 30 as usize) as WORD,
    );
    StringCbCopyNW(
        Scratch.Str.Buffer,
        (Scratch.Str.MaximumLength) as _,
        (*LongPath).Str.Buffer,
        ((*LongPath).Str.Length) as _,
    );
    Ptr = wcsrchr((*LongPath).Str.Buffer, '\\' as WCHAR) as PWCHAR;
    if !Ptr.is_null() {
        let mut FindHandle: HANDLE = core::ptr::null_mut();
        let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
        *Ptr.offset(1 as isize) = '*' as WCHAR;
        *Ptr.offset(2 as isize) = '\0' as WCHAR;
        FindHandle = FindFirstFileW(
            (*LongPath).Str.Buffer,
            core::ptr::addr_of_mut!(FindData),
        );
        if FindHandle != INVALID_HANDLE_VALUE {
            let mut SignMedia: DWORD = 0;
            loop {
                if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0
                    && FindData.nFileSizeLow != 0
                {
                    SignMedia = (SignMedia << 1) ^ FindData.nFileSizeLow;
                }
                if !(FindNextFileW(FindHandle, core::ptr::addr_of_mut!(FindData)) != 0) {
                    break;
                }
            }

            FindClose(FindHandle);
            SdbpResizeTempStr(
                LongPath,
                (((*LongPath).Str.Length as u32 >> 1)
                    + 20) as WORD,
            );
            StringCbPrintfW!(
                (*LongPath).Str.Buffer,
                (*LongPath).Str.MaximumLength,
                SIGN_MEDIA_FMT.as_ptr(),
                SignMedia,
                Scratch.Str.Buffer.offset(3 as isize),
            );
            (*LongPath).Str.Length = ((SdbpStrlen((*LongPath).Str.Buffer) as USHORT
                as usize)
                * core::mem::size_of::<WCHAR>()) as USHORT;
            SdbpFreeTempStr(core::ptr::addr_of_mut!(Scratch));
            return TRUE as BOOL;
        }
    }
    SdbpFreeTempStr(core::ptr::addr_of_mut!(Scratch));
    SdbpFreeTempStr(LongPath);
    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpResolvePath(LongPath: PSDB_TMP_STR, wszPath: PCWSTR) -> BOOL {
    SdbpInitTempStr(LongPath);
    if SdbpGetLongPathName(wszPath, LongPath) == 0 {
        SdbpFreeTempStr(LongPath);
        return FALSE as BOOL;
    }
    if SdbpIsPathOnRemovableMedia((*LongPath).Str.Buffer as PCWSTR) != 0 {
        return SdbpBuildSignMediaId(LongPath);
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64QueryFlag() -> ACCESS_MASK {
    if g_QueryFlag == 0xffffffff as ACCESS_MASK {
        let mut wow64_ptr: ULONG_PTR = 0 as ULONG_PTR;
        let mut Status: NTSTATUS = NtQueryInformationProcess(
            NtCurrentProcess(),
            ProcessWow64Information,
            core::ptr::addr_of_mut!(wow64_ptr) as PVOID,
            core::mem::size_of::<ULONG_PTR>() as ULONG,
            core::ptr::null_mut(),
        );
        g_QueryFlag = if NT_SUCCESS!(Status) && wow64_ptr != 0 as ULONG_PTR {
            KEY_WOW64_64KEY as ACCESS_MASK
        } else {
            0 as ACCESS_MASK
        };
    }
    return g_QueryFlag;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpOpenKey(
    mut FullPath: PUNICODE_STRING,
    mut bMachine: BOOL,
    mut Access: ACCESS_MASK,
    mut KeyHandle: PHANDLE,
) -> NTSTATUS {
    let mut BasePath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut LayersKey: *const WCHAR = (APPCOMPAT_LAYER_KEY).as_ptr();
    let mut ObjectLayer: _OBJECT_ATTRIBUTES = RTL_INIT_OBJECT_ATTRIBUTES!(FullPath, OBJ_CASE_INSENSITIVE);
    let mut Status: NTSTATUS = 0;
    (*FullPath).Buffer = core::ptr::null_mut::<WCHAR>();
    (*FullPath).MaximumLength = 0;
    (*FullPath).Length = (*FullPath).MaximumLength;
    if bMachine != 0 {
        RtlInitUnicodeString(&raw mut BasePath, (REGISTRY_MACHINE).as_ptr());
    } else {
        Status = RtlFormatCurrentUserKeyPath(&raw mut BasePath);
        if !NT_SUCCESS!(Status) {
            SHIM_ERR!(b"Unable to acquire user registry key, Error: 0x%lx\n\0".as_ptr() as PCSTR, Status);
            return Status;
        }
    }
    (*FullPath).MaximumLength =
        (BasePath.Length as DWORD).wrapping_add(SdbpStrsize(LayersKey as PCWSTR)) as USHORT;
    (*FullPath).Buffer = SdbAlloc!((*FullPath).MaximumLength as SIZE_T) as PWSTR;
    (*FullPath).Length = 0;
    RtlAppendUnicodeStringToString(FullPath, &raw mut BasePath as PCUNICODE_STRING);
    if bMachine == 0 {
        RtlFreeUnicodeString(&raw mut BasePath);
    }
    RtlAppendUnicodeToString(FullPath, LayersKey as PCWSTR);

    Status = NtOpenKey(KeyHandle, Access | Wow64QueryFlag(), &raw mut ObjectLayer);
    if !NT_SUCCESS!(Status) {
        SHIM_ERR!(b"Unable to open Key  \"%wZ\" Status 0x%lx\n\0".as_ptr() as PCSTR, FullPath, Status);
        SdbFree!((*FullPath).Buffer as LPVOID);
        (*FullPath).Buffer = core::ptr::null_mut::<WCHAR>();
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpGetPermLayersInternal(
    FullPath: PUNICODE_STRING,
    pwszLayers: PWSTR,
    pdwBytes: PDWORD,
    bMachine: BOOL,
) -> BOOL {
    let mut FullKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut ValueBuffer: [ULONG; ((MAX_LAYER_LENGTH as usize) * ::core::mem::size_of::<WCHAR>() + ::core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + ::core::mem::size_of::<ULONG>() - 1) / ::core::mem::size_of::<ULONG>()] =
        [0; ((MAX_LAYER_LENGTH as usize) * ::core::mem::size_of::<WCHAR>() + ::core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + ::core::mem::size_of::<ULONG>() - 1) / ::core::mem::size_of::<ULONG>()];
    let mut PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION =
        &raw mut ValueBuffer as *mut ULONG as PVOID as PKEY_VALUE_PARTIAL_INFORMATION;
    let mut Length: ULONG = 0 as ULONG;
    let mut KeyHandle: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut Status: NTSTATUS = 0;

    Status = SdbpOpenKey(
        &raw mut FullKey,
        bMachine,
        KEY_QUERY_VALUE as ACCESS_MASK,
        &raw mut KeyHandle,
    );
    if NT_SUCCESS!(Status) {
        Status = NtQueryValueKey(
            KeyHandle,
            FullPath,
            KeyValuePartialInformation,
            PartialInfo as PVOID,
            ::core::mem::size_of::<[ULONG; ((MAX_LAYER_LENGTH as usize) * ::core::mem::size_of::<WCHAR>() + ::core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + ::core::mem::size_of::<ULONG>() - 1) / ::core::mem::size_of::<ULONG>()]>() as ULONG,
            &raw mut Length,
        );
        if NT_SUCCESS!(Status) {
            StringCbCopyNW(
                pwszLayers as STRSAFE_LPWSTR,
                *pdwBytes as size_t,
                &raw mut (*PartialInfo).Data as *mut UCHAR as STRSAFE_LPCWSTR,
                (*PartialInfo).DataLength as size_t,
            );
            *pdwBytes = (*PartialInfo).DataLength as DWORD;
        } else {
            SHIM_INFO!(
                b"Failed to read value info from Key \"%wZ\" Status 0x%lx\n\0".as_ptr() as PCSTR,
                &raw mut FullKey,
                Status,
            );
        }
        NtClose(KeyHandle);
        SdbFree!(FullKey.Buffer as LPVOID);
    }
    return NT_SUCCESS!(Status) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SdbDeletePermLayerKeys(wszPath: PCWSTR, bMachine: BOOL) -> BOOL {
    let mut FullKey: _UNICODE_STRING = core::mem::zeroed();
    let mut LongPath: SDB_TMP_STR = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if SdbpResolvePath(&raw mut LongPath, wszPath) == 0 {
        return (FALSE) as BOOL;
    }

    Status = SdbpOpenKey(&raw mut FullKey, bMachine, KEY_SET_VALUE as ACCESS_MASK, &raw mut KeyHandle);
    if NT_SUCCESS!(Status) {
        Status = NtDeleteValueKey(KeyHandle, &raw mut LongPath.Str);
        if !NT_SUCCESS!(Status) {
            if g_ShimDebugLevel != 0 {
                SHIM_INFO!(
                    b"Failed to delete value from Key \"%wZ\" Status 0x%lx\n\0".as_ptr() as PCSTR,
                    &raw mut FullKey,
                    Status,
                );
            }
            /* This is what we want, so if the key didnt exist, we should not fail :) */
            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }
        }
        NtClose(KeyHandle);
        SdbFree!(FullKey.Buffer as LPVOID);
    }
    SdbpFreeTempStr(&raw mut LongPath);
    return NT_SUCCESS!(Status) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpMatchLayer(start: PCWSTR, end: PCWSTR, compare: PCWSTR) -> BOOL {
    let mut len: usize;
    if end.is_null() {
        return (_wcsicmp(start, compare) != 0) as BOOL ^ 1;
    }
    len = end.offset_from(start) as usize;
    return (wcslen(compare) == len && _wcsnicmp(start, compare, len) == 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpAppendLayer(target: PWSTR, len: DWORD, layer: PCWSTR, end: PCWSTR) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    if *target.offset(0 as isize) != 0 {
        Status = StringCbCatW(
            target as STRSAFE_LPWSTR,
            len as size_t,
            (SPACE_ONLY.as_ptr()) as STRSAFE_LPCWSTR,
        ) as NTSTATUS;
    }

    if NT_SUCCESS!(Status) {
        if !end.is_null() {
            Status = StringCbCatNW(
                target as STRSAFE_LPWSTR,
                len as size_t,
                layer as STRSAFE_LPCWSTR,
                (end.offset_from(layer) as i64 as size_t)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            ) as NTSTATUS;
        } else {
            Status = StringCbCatW(
                target as STRSAFE_LPWSTR,
                len as size_t,
                layer as STRSAFE_LPCWSTR,
            ) as NTSTATUS;
        }
    }

    return NT_SUCCESS!(Status) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AllowPermLayer(Path: PCWSTR) -> BOOL {
    let mut tmp: [WCHAR; 4] = [0x0041u16, 0x003A, 0x005C, 0x0000];
    let mut type_: ULONG;
    if Path.is_null() {
        SHIM_ERR!(b"Invalid argument\n\0".as_ptr() as *const i8);
        return FALSE as BOOL;
    }
    let c: WCHAR = *Path.offset(1 as isize);
    if c == 0x003Au16 {
        // case ':': break;
    } else if c == 0x005Cu16 {
        SHIM_INFO!(b"\"%S\" is a network path.\n\0".as_ptr() as *const i8, Path);
        return FALSE as BOOL;
    } else {
        SHIM_INFO!(b"\"%S\" not a full path we can operate on.\n\0".as_ptr() as *const i8, Path);
        return FALSE as BOOL;
    }
    tmp[0 as usize] = *Path.offset(0 as isize);
    type_ = GetDriveTypeW(core::ptr::addr_of_mut!(tmp) as LPCWSTR) as ULONG;
    if type_ == DRIVE_REMOTE as ULONG {
        /* The logging here indicates that it does not like a CDROM or removable media, but it only
            seems to bail out on a media that reports it is remote...
            I have included correct logging, I doubt anyone would parse the logging, so this shouldnt break anything. */
        SHIM_INFO!(b"\"%S\" is on a remote drive.\n\0".as_ptr() as *const i8, Path);
        return FALSE as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetPermLayerKeys(
    mut wszPath: PCWSTR,
    mut pwszLayers: PWSTR,
    mut pdwBytes: PDWORD,
    mut dwFlags: DWORD,
) -> BOOL {
    let mut Result: BOOL = FALSE as BOOL;
    let mut LongPath: SDB_TMP_STR = core::mem::zeroed();
    let mut dwBytes: DWORD = 0;
    let mut dwTotal: DWORD = 0;
    if wszPath.is_null() || pdwBytes.is_null() {
        SHIM_ERR!(b"NULL parameter passed for wszPath or pdwBytes.\n\0" as *const u8 as PCSTR);
        return FALSE as BOOL;
    }

    if SdbpResolvePath(core::ptr::addr_of_mut!(LongPath), wszPath) == 0 {
        return FALSE as BOOL;
    }
    dwBytes = *pdwBytes;
    if (dwFlags & GPLK_MACHINE) != 0 {
        if SdbpGetPermLayersInternal(
            core::ptr::addr_of_mut!(LongPath.Str),
            pwszLayers,
            core::ptr::addr_of_mut!(dwBytes),
            TRUE as BOOL,
        ) != 0
        {
            Result = TRUE as BOOL;
            dwTotal = dwBytes.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD); /* Compensate for the nullterm. */
            pwszLayers = pwszLayers.add((dwTotal / (core::mem::size_of::<WCHAR>() as DWORD)) as usize);
            dwBytes = (*pdwBytes).wrapping_sub(dwBytes);
            if (dwFlags & GPLK_USER) != 0 {
                let fresh31 = pwszLayers;
                pwszLayers = pwszLayers.offset(1);
                *fresh31 = 0x0020u16;
                *pwszLayers = 0x0000u16;
                dwBytes = dwBytes.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD);
                dwTotal = dwTotal.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD);
            }
        }
    }
    if (dwFlags & GPLK_USER) != 0 {
        if SdbpGetPermLayersInternal(
            core::ptr::addr_of_mut!(LongPath.Str),
            pwszLayers,
            core::ptr::addr_of_mut!(dwBytes),
            FALSE as BOOL,
        ) != 0
        {
            Result = TRUE as BOOL;
            dwTotal = dwTotal.wrapping_add(dwBytes.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD)); /* Compensate for the nullterm. */
        } else if dwTotal > 0
            && *pwszLayers.offset(-(1 as isize)) == 0x0020u16
        {
            *pwszLayers.offset(-(1 as isize)) = '\0' as u16;
            dwTotal = dwTotal.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD);
        }
    }
    if dwTotal != 0 {
        dwTotal = dwTotal.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD);
    }
    *pdwBytes = dwTotal;
    SdbpFreeTempStr(core::ptr::addr_of_mut!(LongPath));
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn SdbSetPermLayerKeys(wszPath: PCWSTR, wszLayers: PCWSTR, bMachine: BOOL) -> BOOL {
    let mut FullKey: _UNICODE_STRING = core::mem::zeroed();
    let mut LongPath: SDB_TMP_STR = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if wszLayers.is_null() || *wszLayers == '\0' as WCHAR {
        return SdbDeletePermLayerKeys(wszPath, bMachine);
    }

    if SdbpResolvePath(core::ptr::addr_of_mut!(LongPath), wszPath) == 0 {
        return (FALSE) as BOOL;
    }

    Status = SdbpOpenKey(
        core::ptr::addr_of_mut!(FullKey),
        bMachine,
        KEY_SET_VALUE as ACCESS_MASK,
        core::ptr::addr_of_mut!(KeyHandle),
    );
    if NT_SUCCESS!(Status) {
        Status = NtSetValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(LongPath.Str),
            0 as ULONG,
            REG_SZ as ULONG,
            wszLayers as PVOID,
            SdbpStrsize(wszLayers) as ULONG,
        );
        if !NT_SUCCESS!(Status) {
            SHIM_INFO!(
                b"Failed to write a value to Key \"%wZ\" Status 0x%lx\n\0".as_ptr() as *const i8,
                core::ptr::addr_of_mut!(FullKey),
                Status,
            );
            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }
        }
        NtClose(KeyHandle);
        SdbFree!(FullKey.Buffer as PVOID);
    }
    SdbpFreeTempStr(core::ptr::addr_of_mut!(LongPath));
    return NT_SUCCESS!(Status) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetPermLayerState(
    mut wszPath: PCWSTR,
    mut wszLayer: PCWSTR,
    mut dwFlags: DWORD,
    mut bMachine: BOOL,
    mut bEnable: BOOL,
) -> BOOL {
    let mut fullLayer: [WCHAR; MAX_LAYER_LENGTH as usize] = [0 as WCHAR; MAX_LAYER_LENGTH as usize];
    let mut newLayer: [WCHAR; MAX_LAYER_LENGTH as usize] = [0 as WCHAR; MAX_LAYER_LENGTH as usize];
    let mut dwBytes: DWORD = ::core::mem::size_of::<[WCHAR; MAX_LAYER_LENGTH as usize]>() as DWORD;
    let mut dwWriteFlags: DWORD = 0 as DWORD;
    let mut start: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut p: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    if wszLayer.is_null() {
        if g_ShimDebugLevel != 0 {
            ShimDbgPrint(
                SHIM_ERR,
                b"SetPermLayerState\0" as *const u8 as PCSTR,
                b"Invalid argument\n\0" as *const u8 as PCSTR,
            );
        }
        return FALSE as BOOL;
    }
    if dwFlags & !(LAYER_APPLY_TO_SYSTEM_EXES | LAYER_UNK_FLAG2) as DWORD != 0 {
        if g_ShimDebugLevel != 0 {
            ShimDbgPrint(
                SHIM_ERR,
                b"SetPermLayerState\0" as *const u8 as PCSTR,
                b"Invalid flags\n\0" as *const u8 as PCSTR,
            );
        }
        return FALSE as BOOL;
    }
    p = wcspbrk(
        wszLayer as *const wchar_t,
        DISALLOWED_LAYER_CHARS.as_ptr(),
    ) as PWSTR;
    if !p.is_null() {
        match *p as ::core::ffi::c_int {
            32 => {
                if g_ShimDebugLevel != 0 {
                    ShimDbgPrint(
                        SHIM_ERR,
                        b"SetPermLayerState\0" as *const u8 as PCSTR,
                        b"Only one layer can be passed in at a time.\n\0" as *const u8 as PCSTR,
                    );
                }
                return FALSE as BOOL;
            }
            35 | 33 => {
                if g_ShimDebugLevel != 0 {
                    ShimDbgPrint(
                        SHIM_ERR,
                        b"SetPermLayerState\0" as *const u8 as PCSTR,
                        b"Flags cannot be passed in with the layer name.\n\0" as *const u8 as PCSTR,
                    );
                }
                return FALSE as BOOL;
            }
            _ => {}
        }
    }
    if SdbGetPermLayerKeys(
        wszPath,
        &raw mut fullLayer as PWSTR,
        &raw mut dwBytes,
        (if bMachine != 0 {
            GPLK_MACHINE
        } else {
            GPLK_USER
        }) as DWORD,
    ) == 0
    {
        fullLayer[0 as usize] = '\0' as i32 as WCHAR;
        dwBytes = ::core::mem::size_of::<[WCHAR; MAX_LAYER_LENGTH as usize]>() as DWORD;
    }
    start = &raw mut fullLayer as *mut WCHAR as PWSTR;
    while *start as ::core::ffi::c_int == '!' as i32
        || *start as ::core::ffi::c_int == '#' as i32
        || *start as ::core::ffi::c_int == ' ' as i32
        || *start as ::core::ffi::c_int == '\t' as i32
    {
        if *start as ::core::ffi::c_int == '#' as i32 {
            dwWriteFlags |= LAYER_APPLY_TO_SYSTEM_EXES as DWORD;
        } else if *start as ::core::ffi::c_int == '!' as i32 {
            dwWriteFlags |= LAYER_UNK_FLAG2 as DWORD;
        }
        start = start.offset(1);
    }
    if bEnable != 0 {
        dwWriteFlags |= dwFlags;
    } else {
        dwWriteFlags &= !dwFlags;
    }
    p = &raw mut newLayer as *mut WCHAR as PWSTR;
    if dwWriteFlags & LAYER_UNK_FLAG2 as DWORD != 0 {
        let fresh32 = p;
        p = p.offset(1);
        *fresh32 = '!' as i32 as WCHAR;
    }
    if dwWriteFlags & LAYER_APPLY_TO_SYSTEM_EXES as DWORD != 0 {
        let fresh33 = p;
        p = p.offset(1);
        *fresh33 = '#' as i32 as WCHAR;
    }
    loop {
        while *start as ::core::ffi::c_int == ' ' as i32
            || *start as ::core::ffi::c_int == '\t' as i32
        {
            start = start.offset(1);
        }
        if *start as ::core::ffi::c_int == '\0' as i32 {
            break;
        }
        p = wcspbrk(
            start as *const wchar_t,
            LAYER_SEPARATORS.as_ptr(),
        ) as PWSTR;
        if SdbpMatchLayer(start as PCWSTR, p as PCWSTR, wszLayer) == 0 {
            SdbpAppendLayer(
                &raw mut newLayer as PWSTR,
                ::core::mem::size_of::<[WCHAR; MAX_LAYER_LENGTH as usize]>() as DWORD,
                start as PCWSTR,
                p as PCWSTR,
            );
        }
        start = p.offset(1 as isize);
        if p.is_null() {
            break;
        }
    }
    if bEnable != 0
        && *wszLayer.offset(0 as isize) as ::core::ffi::c_int != 0
    {
        SdbpAppendLayer(
            &raw mut newLayer as PWSTR,
            ::core::mem::size_of::<[WCHAR; MAX_LAYER_LENGTH as usize]>() as DWORD,
            wszLayer,
            ::core::ptr::null::<WCHAR>(),
        );
    }
    return SdbSetPermLayerKeys(wszPath, &raw mut newLayer as *mut WCHAR as PCWSTR, bMachine);
}

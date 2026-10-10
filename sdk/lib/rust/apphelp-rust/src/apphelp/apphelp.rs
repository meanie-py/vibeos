use crate::apphelp::sdbapi::SdbCloseDatabase;
use crate::apphelp::sdbapi::SdbGUIDToString;
use crate::apphelp::sdbapi::SdbGetDatabaseInformation;
use crate::apphelp::hsdb::SdbGetMatchingExe;
use crate::apphelp::hsdb::SdbInitDatabase;
use crate::apphelp::sdbapi::SdbOpenDatabase;
use crate::apphelp::hsdb::SdbReleaseDatabase;
use crate::apphelp::sdbapi::SdbpHeapDeinit;
use crate::apphelp::sdbapi::SdbpHeapInit;
use crate::apphelp::layer::Wow64QueryFlag;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_15DA2C9A7C0D: [u16; 13] = [
    0x0044u16, 0x0061, 0x0074, 0x0061, 0x0062, 0x0061, 0x0073, 0x0065, 0x0050, 0x0061,
    0x0074, 0x0068, 0x0000,
];
pub static RUSTCONV_WSTR_2366E523D254: [u16; 17] = [
    0x0053u16, 0x0048, 0x0049, 0x004D, 0x005F, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047,
    0x005F, 0x004C, 0x0045, 0x0056, 0x0045, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_3593AA9CD48D: [u16; 25] = [
    0x0044u16, 0x0061, 0x0074, 0x0061, 0x0062, 0x0061, 0x0073, 0x0065, 0x0049, 0x006E,
    0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0054, 0x0069, 0x006D, 0x0065, 0x0053,
    0x0074, 0x0061, 0x006D, 0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_3F48D08B0F0E: [u16; 20] = [
    0x0044u16, 0x0061, 0x0074, 0x0061, 0x0062, 0x0061, 0x0073, 0x0065, 0x0044, 0x0065,
    0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_8BED78639745: [u16; 13] = [
    0x0044u16, 0x0061, 0x0074, 0x0061, 0x0062, 0x0061, 0x0073, 0x0065, 0x0054, 0x0079,
    0x0070, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_AC68F666F7EE: [u16; 91] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x006F,
    0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063,
    0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E,
    0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C, 0x0043, 0x0075,
    0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069,
    0x006F, 0x006E, 0x005C, 0x0041, 0x0070, 0x0070, 0x0043, 0x006F, 0x006D, 0x0070,
    0x0061, 0x0074, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x005C, 0x0049, 0x006E,
    0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0065, 0x0064, 0x0053, 0x0044, 0x0042,
    0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut InstalledSDBKeyName: _UNICODE_STRING = RTL_CONSTANT_STRING_STATIC!(RUSTCONV_WSTR_AC68F666F7EE);

pub const DPFLTR_APPCOMPAT_ID: ULONG = 123;

pub const MAX_GUID_STRING_LEN: usize = 39;

const _: () = assert!(MAX_GUID_STRING_LEN == 39);

#[no_mangle]
pub static mut g_ShimDebugLevel: ULONG = !0;

pub static mut g_hInstance: HMODULE = unsafe { core::mem::zeroed() };

pub const APPHELP_DONTWRITE_REASON: ULONG = 2;

pub const APPHELP_CLEARBITS: ULONG = 0x100; /* TODO: Investigate */

pub const APPHELP_IGNORE_ENVIRONMENT: ULONG = 0x400;

pub const APPHELP_VALID_RESULT: ULONG = 0x10000;

pub const APPHELP_RESULT_NOTFOUND: ULONG = 0x20000;

pub const APPHELP_RESULT_FOUND: ULONG = 0x40000;
#[no_mangle]
pub unsafe extern "C" fn ApphelppInitDebugLevel() {
    let DebugKey: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_2366E523D254.as_ptr());
    let mut DebugValue: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut NewLevel: ULONG = SHIM_ERR;
    let mut Buffer: [u16; 40] = [0u16; 40];

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(DebugValue),
        Buffer.as_mut_ptr(),
        core::mem::size_of_val(&Buffer) as u16,
    );

    Status = RtlQueryEnvironmentVariable_U(
        core::ptr::null_mut(),
        core::ptr::addr_of!(DebugKey),
        core::ptr::addr_of_mut!(DebugValue),
    );

    if NT_SUCCESS!(Status) {
        if !NT_SUCCESS!(RtlUnicodeStringToInteger(
            core::ptr::addr_of!(DebugValue),
            10,
            core::ptr::addr_of_mut!(NewLevel),
        )) {
            NewLevel = SHIM_ERR;
        }
    }
    g_ShimDebugLevel = NewLevel;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinst: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            g_hInstance = hinst;
            DisableThreadLibraryCalls(hinst);
            SdbpHeapInit();
        }
        DLL_PROCESS_DETACH => {
            SdbpHeapDeinit();
        }
        _ => {}
    }
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ApphelpCheckInstallShieldPackage(ptr: LPVOID, path: LPCWSTR) -> BOOL {
    SHIM_WARN!(b"stub: ptr=%p, path='%S'\n\0".as_ptr() as *const i8, ptr, path);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn ApphelpCheckShellObject(ObjectCLSID: *const GUID, bShimIfNecessary: BOOL, pullFlags: *mut ULONGLONG) -> BOOL {
    let mut GuidString: [u16; MAX_GUID_STRING_LEN] = [0u16; MAX_GUID_STRING_LEN];
    if ObjectCLSID.is_null() || SdbGUIDToString(ObjectCLSID, GuidString.as_mut_ptr(), (RTL_NUMBER_OF!(GuidString)) as SIZE_T) == 0 {
        GuidString[0] = 0x0000u16;
    }
    SHIM_WARN!(b"stub: ObjectCLSID='%S', bShimIfNecessary=%d, pullFlags=%p)\n\0".as_ptr() as *const i8, GuidString.as_ptr(), bShimIfNecessary, pullFlags);

    if !pullFlags.is_null() {
        *pullFlags = 0;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn ShimDbgPrint_v(mut Level: SHIM_LOG_LEVEL, FunctionName: PCSTR, Format: PCSTR, ap_list: *mut ::core::ffi::c_void) -> BOOL {
    let mut Buffer: [i8; 512] = [0i8; 512];
    // The C trampoline passes `&ap`; a va_list is a `char*` on every Windows target we build.
    let ArgList: *mut i8 = *(ap_list as *mut *mut i8);
    let mut Current: *mut i8 = Buffer.as_mut_ptr();
    let mut LevelStr: PCSTR;
    let mut Length: usize = core::mem::size_of::<[i8; 512]>();

    if g_ShimDebugLevel == !0 {
        ApphelppInitDebugLevel();
    }

    if Level > g_ShimDebugLevel {
        return FALSE as BOOL;
    }

    match Level {
        SHIM_ERR => {
            LevelStr = b"Err \0".as_ptr() as PCSTR;
            Level = DPFLTR_MASK | (1 << DPFLTR_ERROR_LEVEL);
        }
        SHIM_WARN => {
            LevelStr = b"Warn\0".as_ptr() as PCSTR;
            Level = DPFLTR_MASK | (1 << DPFLTR_WARNING_LEVEL);
        }
        SHIM_INFO => {
            LevelStr = b"Info\0".as_ptr() as PCSTR;
            Level = DPFLTR_MASK | (1 << DPFLTR_INFO_LEVEL);
        }
        _ => {
            LevelStr = b"User\0".as_ptr() as PCSTR;
            Level = DPFLTR_MASK | (1 << DPFLTR_INFO_LEVEL);
        }
    }
    StringCchPrintfExA!(
        Current,
        Length,
        core::ptr::addr_of_mut!(Current),
        core::ptr::addr_of_mut!(Length),
        STRSAFE_NULL_ON_FAILURE,
        b"[%s][%-20s] \0".as_ptr() as PCSTR,
        LevelStr,
        FunctionName
    );

    StringCchVPrintfExA!(
        Current,
        Length,
        core::ptr::addr_of_mut!(Current),
        core::ptr::addr_of_mut!(Length),
        STRSAFE_NULL_ON_FAILURE,
        Format,
        ArgList
    );

    let APPCOMPAT_USE_DBGPRINTEX: bool = false;
    if APPCOMPAT_USE_DBGPRINTEX {
        return if NT_SUCCESS!(DbgPrintEx(
            DPFLTR_APPCOMPAT_ID,
            Level,
            b"%s\0".as_ptr() as PCSTR,
            Buffer.as_ptr()
        )) {
            1
        } else {
            0
        };
    }

    DbgPrint(b"%s\0".as_ptr() as PCSTR, Buffer.as_ptr());
    return (TRUE) as BOOL;
}

extern "C" {
    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust
    // cannot define one); it `va_start`s and calls `ShimDbgPrint_v` above.
    pub fn ShimDbgPrint(Level: SHIM_LOG_LEVEL, FunctionName: PCSTR, Format: PCSTR, ...) -> BOOL;
}


#[no_mangle]
pub unsafe extern "system" fn ApphelpCheckRunAppEx(
    FileHandle: HANDLE,
    Unk1: PVOID,
    Unk2: PVOID,
    ApplicationName: PCWSTR,
    Environment: PVOID,
    ExeType: USHORT,
    Reason: PULONG,
    SdbQueryAppCompatData: *mut PVOID,
    SdbQueryAppCompatDataSize: PULONG,
    SxsData: *mut PVOID,
    SxsDataSize: PULONG,
    FusionFlags: PULONG,
    SomeFlag1: PULONG64,
    SomeFlag2: PULONG,
) -> BOOL {
    let mut result: *mut SDBQUERYRESULT = core::ptr::null_mut();
    let mut hsdb: HSDB = core::ptr::null_mut();
    let mut dwFlags: DWORD = 0;

    if !SxsData.is_null() {
        *SxsData = core::ptr::null_mut();
    }
    if !SxsDataSize.is_null() {
        *SxsDataSize = 0;
    }
    if !FusionFlags.is_null() {
        *FusionFlags = 0;
    }
    if !SomeFlag1.is_null() {
        *SomeFlag1 = 0;
    }
    if !SomeFlag2.is_null() {
        *SomeFlag2 = 0;
    }
    if !Reason.is_null() {
        dwFlags = *Reason;
    }

    dwFlags &= !APPHELP_CLEARBITS;

    result = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SDBQUERYRESULT>() as SIZE_T,
    ) as *mut SDBQUERYRESULT;
    *SdbQueryAppCompatData = result as PVOID;
    if !SdbQueryAppCompatDataSize.is_null() {
        *SdbQueryAppCompatDataSize = core::mem::size_of::<SDBQUERYRESULT>() as ULONG;
    }

    hsdb = SdbInitDatabase(
        HID_DOS_PATHS | SDB_DATABASE_MAIN_SHIM,
        core::ptr::null_mut(),
    );
    if !hsdb.is_null() {
        let mut FoundMatch: BOOL;
        let mut MatchingExeFlags: DWORD = 0;

        if (dwFlags & APPHELP_IGNORE_ENVIRONMENT) != 0 {
            MatchingExeFlags |= SDBGMEF_IGNORE_ENVIRONMENT;
        }

        FoundMatch = SdbGetMatchingExe(
            hsdb,
            ApplicationName,
            core::ptr::null_mut(),
            Environment as LPCWSTR,
            MatchingExeFlags,
            result,
        );
        if FileHandle != INVALID_HANDLE_VALUE {
            dwFlags |= APPHELP_VALID_RESULT;
            dwFlags |= if FoundMatch != 0 {
                APPHELP_RESULT_FOUND
            } else {
                APPHELP_RESULT_NOTFOUND
            };
        }

        SdbReleaseDatabase(hsdb);
    }

    if !Reason.is_null() && (dwFlags & APPHELP_DONTWRITE_REASON) == 0 {
        *Reason = dwFlags;
    }

    /* We should _ALWAYS_ return TRUE here, unless we want to block an application from starting! */
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbRegisterDatabaseEx(
    pszDatabasePath: LPCWSTR,
    dwDatabaseType: DWORD,
    pTimeStamp: PULONGLONG,
) -> BOOL {
    let mut pdb: PDB = SdbOpenDatabase(pszDatabasePath, DOS_PATH);
    let mut Information: DB_INFORMATION = core::mem::zeroed();
    let mut GuidBuffer: [WCHAR; MAX_GUID_STRING_LEN] = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyAccess: ACCESS_MASK = 0;
    let mut ObjectKey: _OBJECT_ATTRIBUTES = RTL_INIT_OBJECT_ATTRIBUTES!(core::ptr::addr_of_mut!(KeyName), OBJ_CASE_INSENSITIVE);
    let mut Status: NTSTATUS = 0;
    let mut InstalledSDBKey: HANDLE = core::ptr::null_mut();

    if pdb.is_null()
    {
        SHIM_ERR!(b"Unable to open DB %S\n\0".as_ptr() as *const i8, pszDatabasePath);
        return FALSE as BOOL;
    }

    if SdbGetDatabaseInformation(pdb, core::ptr::addr_of_mut!(Information)) == 0 ||
        (Information.dwFlags & DB_INFO_FLAGS_VALID_GUID) == 0
    {
        SHIM_ERR!(b"Unable to retrieve DB info\n\0".as_ptr() as *const i8);
        SdbCloseDatabase(pdb);
        return FALSE as BOOL;
    }

    if SdbGUIDToString(core::ptr::addr_of_mut!(Information.Id), GuidBuffer.as_mut_ptr(), RTL_NUMBER_OF!(GuidBuffer) as SIZE_T) == 0
    {
        SHIM_ERR!(b"Unable to Convert GUID to string\n\0".as_ptr() as *const i8);
        SdbCloseDatabase(pdb);
        return FALSE as BOOL;
    }

    KeyName = InstalledSDBKeyName;
    KeyAccess = Wow64QueryFlag() | KEY_WRITE | KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS;
    Status = NtCreateKey(
        core::ptr::addr_of_mut!(InstalledSDBKey),
        KeyAccess,
        core::ptr::addr_of_mut!(ObjectKey),
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS!(Status)
    {
        let mut DbKey: HANDLE = core::ptr::null_mut();

        RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyName), GuidBuffer.as_mut_ptr());
        ObjectKey.RootDirectory = InstalledSDBKey;

        Status = NtCreateKey(
            core::ptr::addr_of_mut!(DbKey),
            KeyAccess,
            core::ptr::addr_of_mut!(ObjectKey),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS!(Status)
        {
            let mut DatabasePathKey: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_15DA2C9A7C0D.as_ptr());
            let mut DatabaseInstallTimeStampKey: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_3593AA9CD48D.as_ptr());
            let mut DatabaseTypeKey: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_8BED78639745.as_ptr());
            let mut DatabaseDescriptionKey: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_3F48D08B0F0E.as_ptr());

            Status = NtSetValueKey(
                DbKey,
                core::ptr::addr_of_mut!(DatabasePathKey),
                0,
                REG_SZ,
                pszDatabasePath as PVOID,
                ((wcslen(pszDatabasePath) as ULONG) + 1) * (core::mem::size_of::<WCHAR>() as ULONG),
            );
            if !NT_SUCCESS!(Status)
            {
                SHIM_ERR!(b"Unable to write %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(DatabasePathKey));
            }

            if NT_SUCCESS!(Status)
            {
                let mut ulTimeStamp: ULARGE_INTEGER = core::mem::zeroed();
                if !pTimeStamp.is_null()
                {
                    ulTimeStamp.QuadPart = *pTimeStamp;
                }
                else
                {
                    let mut fi: FILETIME = core::mem::zeroed();
                    GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(fi));
                    ulTimeStamp.__bindgen_anon_1.LowPart = fi.dwLowDateTime;
                    ulTimeStamp.__bindgen_anon_1.HighPart = fi.dwHighDateTime;
                }
                Status = NtSetValueKey(
                    DbKey,
                    core::ptr::addr_of_mut!(DatabaseInstallTimeStampKey),
                    0,
                    REG_QWORD,
                    core::ptr::addr_of_mut!(ulTimeStamp.QuadPart) as PVOID,
                    core::mem::size_of::<ULARGE_INTEGER>() as ULONG,
                );
                if !NT_SUCCESS!(Status)
                {
                    SHIM_ERR!(b"Unable to write %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(DatabaseInstallTimeStampKey));
                }
            }

            if NT_SUCCESS!(Status)
            {
                Status = NtSetValueKey(
                    DbKey,
                    core::ptr::addr_of_mut!(DatabaseTypeKey),
                    0,
                    REG_DWORD,
                    core::ptr::addr_of!(dwDatabaseType) as PVOID,
                    core::mem::size_of::<DWORD>() as ULONG,
                );
                if !NT_SUCCESS!(Status)
                {
                    SHIM_ERR!(b"Unable to write %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(DatabaseTypeKey));
                }
            }

            if NT_SUCCESS!(Status) && !Information.Description.is_null()
            {
                Status = NtSetValueKey(
                    DbKey,
                    core::ptr::addr_of_mut!(DatabaseDescriptionKey),
                    0,
                    REG_SZ,
                    Information.Description as PVOID,
                    ((wcslen(Information.Description) as ULONG) + 1) * (core::mem::size_of::<WCHAR>() as ULONG),
                );
                if !NT_SUCCESS!(Status)
                {
                    SHIM_ERR!(b"Unable to write %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(DatabaseDescriptionKey));
                }
            }

            NtClose(DbKey);

            if NT_SUCCESS!(Status)
            {
                SHIM_INFO!(b"Installed %wS as %wZ\n\0".as_ptr() as *const i8, pszDatabasePath, core::ptr::addr_of_mut!(KeyName));
            }
        }
        else
        {
            SHIM_ERR!(b"Unable to create key %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(KeyName));
        }

        NtClose(InstalledSDBKey);
    }
    else
    {
        SHIM_ERR!(b"Unable to create key %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(KeyName));
    }

    SdbCloseDatabase(pdb);

    NT_SUCCESS!(Status) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SdbRegisterDatabase(
    pszDatabasePath: LPCWSTR,
    dwDatabaseType: DWORD,
) -> BOOL {
    return SdbRegisterDatabaseEx(pszDatabasePath, dwDatabaseType, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SdbUnregisterDatabase(pguidDB: *const GUID) -> BOOL {
    let mut KeyBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut GuidBuffer: [u16; MAX_GUID_STRING_LEN] = [0; MAX_GUID_STRING_LEN];
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyAccess: ACCESS_MASK;
    let mut ObjectKey: _OBJECT_ATTRIBUTES = RTL_INIT_OBJECT_ATTRIBUTES!(core::ptr::addr_of_mut!(KeyName), OBJ_CASE_INSENSITIVE);
    let mut Status: NTSTATUS;
    let mut DbKey: HANDLE = core::ptr::null_mut();

    if SdbGUIDToString(pguidDB, GuidBuffer.as_mut_ptr(), RTL_NUMBER_OF!(GuidBuffer) as _) == 0 {
        SHIM_ERR!(b"Unable to Convert GUID to string\n\0".as_ptr() as *const i8);
        return FALSE as BOOL;
    }

    RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!(KeyName), KeyBuffer.as_mut_ptr(), core::mem::size_of::<[u16; MAX_PATH as usize]>() as _);
    RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(KeyName), core::ptr::addr_of!(InstalledSDBKeyName));
    RtlAppendUnicodeToString(core::ptr::addr_of_mut!(KeyName), RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    RtlAppendUnicodeToString(core::ptr::addr_of_mut!(KeyName), GuidBuffer.as_ptr());

    KeyAccess = Wow64QueryFlag() | DELETE;
    Status = NtCreateKey(core::ptr::addr_of_mut!(DbKey), KeyAccess, core::ptr::addr_of_mut!(ObjectKey), 0, core::ptr::null_mut(), 0, core::ptr::null_mut());
    if !NT_SUCCESS!(Status) {
        SHIM_ERR!(b"Unable to open %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of!(KeyName));
        return FALSE as BOOL;
    }

    Status = NtDeleteKey(DbKey);
    if !NT_SUCCESS!(Status) {
        SHIM_ERR!(b"Unable to delete %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of!(KeyName));
    }

    NtClose(DbKey);
    return NT_SUCCESS!(Status) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ShimDumpCache(hwnd: HWND, hInstance: HINSTANCE, lpszCmdLine: LPCSTR, nCmdShow: i32) -> BOOL {
    return BaseDumpAppcompatCache();
}

#[no_mangle]
pub unsafe extern "system" fn ShimFlushCache(hwnd: HWND, hInstance: HINSTANCE, lpszCmdLine: LPCSTR, nCmdShow: i32) -> BOOL {
    return BaseFlushAppcompatCache();
}

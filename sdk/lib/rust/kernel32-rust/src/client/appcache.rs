use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut g_ShimsDisabled: ULONG = -(1 as ::core::ffi::c_int) as ULONG;

static mut g_ApphelpInitialized: BOOL = FALSE;

static mut g_pApphelpCheckRunAppEx: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut g_pSdbPackAppCompatData: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub const NDEBUG: () = ();

pub type tApphelpCheckRunAppEx = unsafe extern "system" fn(
    FileHandle: HANDLE,
    Unk1: PVOID,
    Unk2: PVOID,
    ApplicationName: PWCHAR,
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
) -> BOOL;

pub type tSdbPackAppCompatData = unsafe extern "system" fn(
    hsdb: PVOID,
    pQueryResult: PVOID,
    ppData: *mut PVOID,
    dwSize: *mut DWORD,
) -> BOOL;

pub const APPHELP_VALID_RESULT: ULONG = 0x10000;
pub const APPHELP_RESULT_NOTFOUND: ULONG = 0x20000;
pub const APPHELP_RESULT_FOUND: ULONG = 0x40000;

#[no_mangle]
pub unsafe extern "system" fn IsShimInfrastructureDisabled() -> BOOLEAN {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut KeyInfo: KEY_VALUE_PARTIAL_INFORMATION = KEY_VALUE_PARTIAL_INFORMATION {
        TitleIndex: 0,
        Type: 0,
        DataLength: 0,
        Data: [0; 1],
    };
    let mut ResultLength: ULONG = 0;
    let mut OptionKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 67]>() - core::mem::size_of::<u16>()) as USHORT,
        MaximumLength: core::mem::size_of::<[u16; 67]>() as USHORT,
        Buffer: {
            const S: &[u8; 134] = b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0A\0C\0H\0I\0N\0E\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0S\0a\0f\0e\0B\0o\0o\0t\0\\\0O\0p\0t\0i\0o\0n\0\0\0";
            core::mem::transmute::<&[u8; 134], &[u16; 67]>(S).as_ptr() as PWSTR
        },
    };
    let mut AppCompatKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 84]>() - core::mem::size_of::<u16>()) as USHORT,
        MaximumLength: core::mem::size_of::<[u16; 84]>() as USHORT,
        Buffer: {
            const S: &[u8; 168] = b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0A\0C\0H\0I\0N\0E\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0S\0e\0s\0s\0i\0o\0n\0 \0M\0a\0n\0a\0g\0e\0r\0\\\0A\0p\0p\0C\0o\0m\0p\0a\0t\0i\0b\0i\0l\0i\0t\0y\0\0\0";
            core::mem::transmute::<&[u8; 168], &[u16; 84]>(S).as_ptr() as PWSTR
        },
    };
    let mut PolicyKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 64]>() - core::mem::size_of::<u16>()) as USHORT,
        MaximumLength: core::mem::size_of::<[u16; 64]>() as USHORT,
        Buffer: {
            const S: &[u8; 128] = b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0A\0C\0H\0I\0N\0E\0\\\0S\0o\0f\0t\0w\0a\0r\0e\0\\\0P\0o\0l\0i\0c\0i\0e\0s\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0A\0p\0p\0C\0o\0m\0p\0a\0t\0\0\0";
            core::mem::transmute::<&[u8; 128], &[u16; 64]>(S).as_ptr() as PWSTR
        },
    };
    let mut OptionValue: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 12]>() - core::mem::size_of::<u16>()) as USHORT,
        MaximumLength: core::mem::size_of::<[u16; 12]>() as USHORT,
        Buffer: {
            const S: &[u8; 24] = b"O\0p\0t\0i\0o\0n\0V\0a\0l\0u\0e\0\0\0";
            core::mem::transmute::<&[u8; 24], &[u16; 12]>(S).as_ptr() as PWSTR
        },
    };
    let mut DisableAppCompat: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 17]>() - core::mem::size_of::<u16>()) as USHORT,
        MaximumLength: core::mem::size_of::<[u16; 17]>() as USHORT,
        Buffer: {
            const S: &[u8; 34] = b"D\0i\0s\0a\0b\0l\0e\0A\0p\0p\0C\0o\0m\0p\0a\0t\0\0\0";
            core::mem::transmute::<&[u8; 34], &[u16; 17]>(S).as_ptr() as PWSTR
        },
    };
    let mut DisableEngine: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 14]>() - core::mem::size_of::<u16>()) as USHORT,
        MaximumLength: core::mem::size_of::<[u16; 14]>() as USHORT,
        Buffer: {
            const S: &[u8; 28] = b"D\0i\0s\0a\0b\0l\0e\0E\0n\0g\0i\0n\0e\0\0\0";
            core::mem::transmute::<&[u8; 28], &[u16; 14]>(S).as_ptr() as PWSTR
        },
    };
    let mut OptionKeyAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &mut OptionKey as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut AppCompatKeyAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &mut AppCompatKey as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut PolicyKeyAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &mut PolicyKey as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    if g_ShimsDisabled == !0 {
        let mut DisableShims: ULONG = 0;

        Status = NtOpenKey(
            &mut KeyHandle as *mut HANDLE,
            KEY_QUERY_VALUE,
            &mut OptionKeyAttributes as *mut _OBJECT_ATTRIBUTES,
        );
        if NT_SUCCESS(Status) {
            Status = NtQueryValueKey(
                KeyHandle,
                &mut OptionValue as *mut _UNICODE_STRING,
                KeyValuePartialInformation,
                &mut KeyInfo as *mut KEY_VALUE_PARTIAL_INFORMATION as *mut core::ffi::c_void,
                core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
                &mut ResultLength as *mut ULONG,
            );
            NtClose(KeyHandle);
            if NT_SUCCESS(Status)
                && KeyInfo.Type == REG_DWORD
                && KeyInfo.DataLength as usize == core::mem::size_of::<ULONG>()
                && *KeyInfo.Data.as_ptr().offset(0) != 0
            {
                DisableShims = 1;
            }
        }

        if DisableShims == 0 {
            Status = NtOpenKey(
                &mut KeyHandle as *mut HANDLE,
                KEY_QUERY_VALUE,
                &mut AppCompatKeyAttributes as *mut _OBJECT_ATTRIBUTES,
            );
            if NT_SUCCESS(Status) {
                Status = NtQueryValueKey(
                    KeyHandle,
                    &mut DisableAppCompat as *mut _UNICODE_STRING,
                    KeyValuePartialInformation,
                    &mut KeyInfo as *mut KEY_VALUE_PARTIAL_INFORMATION as *mut core::ffi::c_void,
                    core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
                    &mut ResultLength as *mut ULONG,
                );
                NtClose(KeyHandle);
                if NT_SUCCESS(Status)
                    && KeyInfo.Type == REG_DWORD
                    && KeyInfo.DataLength as usize == core::mem::size_of::<ULONG>()
                    && *KeyInfo.Data.as_ptr().offset(0) != 0
                {
                    DisableShims = 1;
                }
            }
        }

        if DisableShims == 0 {
            Status = NtOpenKey(
                &mut KeyHandle as *mut HANDLE,
                KEY_QUERY_VALUE,
                &mut PolicyKeyAttributes as *mut _OBJECT_ATTRIBUTES,
            );
            if NT_SUCCESS(Status) {
                Status = NtQueryValueKey(
                    KeyHandle,
                    &mut DisableEngine as *mut _UNICODE_STRING,
                    KeyValuePartialInformation,
                    &mut KeyInfo as *mut KEY_VALUE_PARTIAL_INFORMATION as *mut core::ffi::c_void,
                    core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
                    &mut ResultLength as *mut ULONG,
                );
                NtClose(KeyHandle);
                if NT_SUCCESS(Status)
                    && KeyInfo.Type == REG_DWORD
                    && KeyInfo.DataLength as usize == core::mem::size_of::<ULONG>()
                    && *KeyInfo.Data.as_ptr().offset(0) != 0
                {
                    DisableShims = 1;
                }
            }
        }

        g_ShimsDisabled = DisableShims;
    }

    if g_ShimsDisabled != 0 { 1 } else { 0 }
}
#[no_mangle]
pub unsafe extern "C" fn BasepShimCacheCheckBypass(
    ApplicationName: PCWSTR,
    FileHandle: HANDLE,
    Environment: PCWSTR,
    bUnknown: BOOL,
    pdwReason: PULONG,
) -> BOOL {
    return 0i32;
}
#[no_mangle]
pub unsafe extern "C" fn BasepShimCacheSearch(
    ApplicationName: PCWSTR,
    FileHandle: HANDLE,
) -> BOOL {
    let mut Lookup: APPHELP_CACHE_SERVICE_LOOKUP = core::mem::zeroed();
    RtlInitUnicodeString(&mut Lookup.ImageName, ApplicationName);
    Lookup.ImageHandle = FileHandle;
    return NT_SUCCESS(NtApphelpCacheControl(ApphelpCacheServiceLookup, &mut Lookup)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn BasepCheckCacheExcludeList(ApplicationName: PCWSTR) -> BOOL {
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn BasepCheckCacheExcludeCustom(ApplicationName: PCWSTR) -> BOOL {
    return 0i32;
}
#[no_mangle]
pub unsafe extern "C" fn BasepShimCacheRemoveEntry(
    ApplicationName: PCWSTR,
) {
    let mut Lookup: APPHELP_CACHE_SERVICE_LOOKUP = core::mem::zeroed();
    RtlInitUnicodeString(&mut Lookup.ImageName, ApplicationName);
    Lookup.ImageHandle = INVALID_HANDLE_VALUE;
    NtApphelpCacheControl(ApphelpCacheServiceRemove, &mut Lookup);
}
#[no_mangle]
pub unsafe extern "C" fn BasepShimCacheLookup(
    ApplicationName: PCWSTR,
    FileHandle: HANDLE,
) -> BOOL {
    if BasepShimCacheSearch(ApplicationName, FileHandle) == 0 {
        return FALSE;
    }
    if BasepCheckCacheExcludeList(ApplicationName) == 0
        || BasepCheckCacheExcludeCustom(ApplicationName) == 0
    {
        BasepShimCacheRemoveEntry(ApplicationName);
        return FALSE;
    }
    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn BaseCheckAppcompatCache(
    ApplicationName: PCWSTR,
    FileHandle: HANDLE,
    Environment: PCWSTR,
    pdwReason: PULONG,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut dwReason: ULONG = 0;

    dwReason = 0;
    if BasepShimCacheCheckBypass(
        ApplicationName,
        FileHandle,
        Environment,
        1,
        core::ptr::addr_of_mut!(dwReason),
    ) != 0
    {
        dwReason |= 2;
    } else {
        ret = BasepShimCacheLookup(ApplicationName, FileHandle);
        if ret == 0 {
            dwReason |= 1;
        }
    }

    if !pdwReason.is_null() {
        *pdwReason = dwReason;
    }

    return ret;
}
unsafe fn BaseInitApphelp() {
    let mut Buffer: [WCHAR; MAX_PATH as usize * 2] = [0; MAX_PATH as usize * 2];
    let mut DllPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ApphelpAddress: PVOID = core::ptr::null_mut();
    let mut pApphelpCheckRunAppEx: PVOID = core::ptr::null_mut();
    let mut pSdbPackAppCompatData: PVOID = core::ptr::null_mut();

    RtlInitEmptyUnicodeString(
        &mut DllPath,
        Buffer.as_mut_ptr(),
        core::mem::size_of::<[WCHAR; MAX_PATH as usize * 2]>() as USHORT,
    );
    RtlCopyUnicodeString(&mut DllPath, &BaseWindowsDirectory);
    RtlAppendUnicodeToString(
        &mut DllPath,
        "\\system32\\apphelp.dll\0".encode_utf16().collect::<Vec<u16>>().as_ptr() as PCWSTR,
    );

    if NT_SUCCESS(LdrLoadDll(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut DllPath,
        &mut ApphelpAddress,
    )) {
        let mut ProcName: ANSI_STRING = ANSI_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };

        RtlInitAnsiString(
            &mut ProcName,
            b"ApphelpCheckRunAppEx\0".as_ptr() as PCSZ,
        );
        if !NT_SUCCESS(LdrGetProcedureAddress(
            ApphelpAddress,
            &mut ProcName,
            0,
            &mut pApphelpCheckRunAppEx,
        )) {
            pApphelpCheckRunAppEx = core::ptr::null_mut();
        }

        RtlInitAnsiString(
            &mut ProcName,
            b"SdbPackAppCompatData\0".as_ptr() as PCSZ,
        );
        if !NT_SUCCESS(LdrGetProcedureAddress(
            ApphelpAddress,
            &mut ProcName,
            0,
            &mut pSdbPackAppCompatData,
        )) {
            pSdbPackAppCompatData = core::ptr::null_mut();
        }
    }

    if InterlockedCompareExchangePointer(
        &mut g_pApphelpCheckRunAppEx,
        RtlEncodeSystemPointer(pApphelpCheckRunAppEx),
        core::ptr::null_mut(),
    ).is_null() {
        g_pSdbPackAppCompatData = RtlEncodeSystemPointer(pSdbPackAppCompatData);
    }
}

#[no_mangle]
pub unsafe extern "system" fn BaseCheckRunApp(
    FileHandle: HANDLE,
    ApplicationName: PWCHAR,
    Environment: PWCHAR,
    ExeType: USHORT,
    pReason: PULONG,
    SdbQueryAppCompatData: *mut PVOID,
    SdbQueryAppCompatDataSize: PULONG,
    SxsData: *mut PVOID,
    SxsDataSize: PULONG,
    FusionFlags: PULONG,
) -> BOOL {
    let mut Reason: ULONG = 0;
    let mut Flags1: ULONG64 = 0;
    let mut Flags2: ULONG = 0;
    let mut Continue: BOOL = 0;
    let mut NeedCleanup: BOOL = 0;
    let mut pApphelpCheckRunAppEx: Option<tApphelpCheckRunAppEx>;
    let mut pSdbPackAppCompatData: Option<tSdbPackAppCompatData>;
    let mut QueryResult: PVOID = core::ptr::null_mut();
    let mut QueryResultSize: ULONG = 0;

    if g_ApphelpInitialized == 0 {
        BaseInitApphelp();
        g_ApphelpInitialized = 1;
    }

    pApphelpCheckRunAppEx = core::mem::transmute::<PVOID, Option<tApphelpCheckRunAppEx>>(
        RtlDecodeSystemPointer(g_pApphelpCheckRunAppEx),
    );
    pSdbPackAppCompatData = core::mem::transmute::<PVOID, Option<tSdbPackAppCompatData>>(
        RtlDecodeSystemPointer(g_pSdbPackAppCompatData),
    );

    if pApphelpCheckRunAppEx.is_none() || pSdbPackAppCompatData.is_none() {
        return 1;
    }

    if !pReason.is_null() {
        Reason = *pReason;
    }

    Continue = pApphelpCheckRunAppEx.expect("non-null function pointer")(
        FileHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ApplicationName,
        Environment,
        ExeType,
        &raw mut Reason,
        &raw mut QueryResult,
        &raw mut QueryResultSize,
        SxsData,
        SxsDataSize,
        FusionFlags,
        &raw mut Flags1,
        &raw mut Flags2,
    );

    if !pReason.is_null() {
        *pReason = Reason;
    }

    if Continue != 0 {
        if (Reason & (APPHELP_VALID_RESULT | APPHELP_RESULT_FOUND)) == (APPHELP_VALID_RESULT | APPHELP_RESULT_FOUND) {
            if pSdbPackAppCompatData.expect("non-null function pointer")(
                core::ptr::null_mut(),
                QueryResult,
                SdbQueryAppCompatData,
                SdbQueryAppCompatDataSize,
            ) == 0
            {
                NeedCleanup = 1;
            }
        } else {
            NeedCleanup = 1;
        }
    }

    if !QueryResult.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, QueryResult);
    }

    if NeedCleanup != 0 {
        BasepFreeAppCompatData(*SdbQueryAppCompatData, *SxsData);
        *SdbQueryAppCompatData = core::ptr::null_mut();
        if !SdbQueryAppCompatDataSize.is_null() {
            *SdbQueryAppCompatDataSize = 0;
        }
        *SxsData = core::ptr::null_mut();
        if !SxsDataSize.is_null() {
            *SxsDataSize = 0;
        }
    }

    return Continue;
}

#[no_mangle]
pub unsafe extern "system" fn BasepCheckBadapp(
    FileHandle: HANDLE,
    ApplicationName: PWCHAR,
    Environment: PWCHAR,
    ExeType: USHORT,
    SdbQueryAppCompatData: *mut PVOID,
    SdbQueryAppCompatDataSize: PULONG,
    SxsData: *mut PVOID,
    SxsDataSize: PULONG,
    FusionFlags: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Reason: ULONG = 0;

    if IsShimInfrastructureDisabled() != 0 {
        Status = STATUS_SUCCESS;
    } else {
        if BaseCheckAppcompatCache(
            ApplicationName as PCWSTR,
            FileHandle,
            Environment as PCWSTR,
            &mut Reason,
        ) == 0
        {
            if BaseCheckRunApp(
                FileHandle,
                ApplicationName,
                Environment,
                ExeType,
                &mut Reason,
                SdbQueryAppCompatData,
                SdbQueryAppCompatDataSize,
                SxsData,
                SxsDataSize,
                FusionFlags,
            ) == 0
            {
                Status = STATUS_ACCESS_DENIED;
            }
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn BaseDumpAppcompatCache() -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtApphelpCacheControl(ApphelpCacheServiceDump, core::ptr::null_mut());
    return NT_SUCCESS(Status) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn BaseFlushAppcompatCache() -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtApphelpCacheControl(ApphelpCacheServiceFlush, core::ptr::null_mut());
    return (Status >= 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn BasepFreeAppCompatData(
    AppCompatData: PVOID,
    AppCompatSxsData: PVOID,
) {
    if !AppCompatData.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AppCompatData);
    }
    if !AppCompatSxsData.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AppCompatSxsData);
    }
}
#[no_mangle]
pub unsafe extern "system" fn BaseUpdateAppcompatCache(
    Unknown1: ULONG,
    Unknown2: ULONG,
    Unknown3: ULONG,
) {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        461i32,
        b"BaseUpdateAppcompatCache\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            461i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn BaseCleanupAppcompatCache() -> NTSTATUS {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as u32);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
            .as_ptr() as *const i8,
        471i32,
        b"BaseCleanupAppcompatCache\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
                .as_ptr() as *const i8,
            471i32,
        );
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn BaseCleanupAppcompatCacheSupport(
    pUnknown: PVOID,
) -> NTSTATUS {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
            .as_ptr() as *const i8,
        482i32,
        b"BaseCleanupAppcompatCacheSupport\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
                .as_ptr() as *const i8,
            482i32,
        );
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn BaseInitAppcompatCache() -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
            .as_ptr() as *const i8,
        493i32,
        b"BaseInitAppcompatCache\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
                .as_ptr() as *const i8,
            493i32,
        );
    }
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn BaseInitAppcompatCacheSupport() -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
            .as_ptr() as *const i8,
        504i32,
        b"BaseInitAppcompatCacheSupport\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
                .as_ptr() as *const i8,
            504i32,
        );
    }
    return 0i32;
}
#[no_mangle]
pub unsafe extern "system" fn GetComPlusPackageInstallStatus() -> *mut core::ffi::c_void {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as u32);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
            .as_ptr() as *const i8,
        515i32,
        b"GetComPlusPackageInstallStatus\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
                .as_ptr() as *const i8,
            515i32,
        );
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn SetComPlusPackageInstallStatus(
    ComPlusPackage: ULONG,
) -> BOOL {
    let mut Status: NTSTATUS;

    if ComPlusPackage & !1 != 0 {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0;
    }

    Status = NtSetSystemInformation(
        SystemComPlusPackage,
        &raw const ComPlusPackage as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetTermsrvAppInstallMode(bInstallMode: BOOL) {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0".as_ptr() as *const ::core::ffi::c_char,
        555 as ::core::ffi::c_int,
        b"SetTermsrvAppInstallMode\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0".as_ptr() as *const ::core::ffi::c_char,
            555 as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn TermsrvAppInstallMode() -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        565 as ::core::ffi::c_int,
        b"TermsrvAppInstallMode\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/kernel32/client/appcache.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            565 as ::core::ffi::c_int,
        );
    }
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/appcache.c
//  * PURPOSE:         Application Compatibility Cache
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// ULONG g_ShimsDisabled = -1;
// static BOOL g_ApphelpInitialized = FALSE;
// static PVOID g_pApphelpCheckRunAppEx;
// static PVOID g_pSdbPackAppCompatData;
//
// typedef BOOL (WINAPI *tApphelpCheckRunAppEx)(HANDLE FileHandle, PVOID Unk1, PVOID Unk2, PWCHAR ApplicationName, PVOID Environment, USHORT ExeType, PULONG Reason,
//                                              PVOID* SdbQueryAppCompatData, PULONG SdbQueryAppCompatDataSize, PVOID* SxsData, PULONG SxsDataSize,
//                                              PULONG FusionFlags, PULONG64 SomeFlag1, PULONG SomeFlag2);
// typedef BOOL (WINAPI *tSdbPackAppCompatData)(PVOID hsdb, PVOID pQueryResult, PVOID* ppData, DWORD *dwSize);
//
// #define APPHELP_VALID_RESULT        0x10000
// #define APPHELP_RESULT_NOTFOUND     0x20000
// #define APPHELP_RESULT_FOUND        0x40000
//
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN
// WINAPI
// IsShimInfrastructureDisabled(VOID)
// {
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     KEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//     ULONG ResultLength;
//     UNICODE_STRING OptionKey = RTL_CONSTANT_STRING(L"\\Registry\\MACHINE\\System\\CurrentControlSet\\Control\\SafeBoot\\Option");
//     UNICODE_STRING AppCompatKey = RTL_CONSTANT_STRING(L"\\Registry\\MACHINE\\System\\CurrentControlSet\\Control\\Session Manager\\AppCompatibility");
//     UNICODE_STRING PolicyKey = RTL_CONSTANT_STRING(L"\\Registry\\MACHINE\\Software\\Policies\\Microsoft\\Windows\\AppCompat");
//     UNICODE_STRING OptionValue = RTL_CONSTANT_STRING(L"OptionValue");
//     UNICODE_STRING DisableAppCompat = RTL_CONSTANT_STRING(L"DisableAppCompat");
//     UNICODE_STRING DisableEngine = RTL_CONSTANT_STRING(L"DisableEngine");
//     OBJECT_ATTRIBUTES OptionKeyAttributes = RTL_CONSTANT_OBJECT_ATTRIBUTES(&OptionKey, OBJ_CASE_INSENSITIVE);
//     OBJECT_ATTRIBUTES AppCompatKeyAttributes = RTL_CONSTANT_OBJECT_ATTRIBUTES(&AppCompatKey, OBJ_CASE_INSENSITIVE);
//     OBJECT_ATTRIBUTES PolicyKeyAttributes = RTL_CONSTANT_OBJECT_ATTRIBUTES(&PolicyKey, OBJ_CASE_INSENSITIVE);
//
//     /*
//      * This is a TROOLEAN, -1 means we haven't yet figured it out.
//      * 0 means shims are enabled, and 1 means shims are disabled!
//      */
//     if (g_ShimsDisabled == -1)
//     {
//         ULONG DisableShims = FALSE;
//
//         /* Open the safe mode key */
//         Status = NtOpenKey(&KeyHandle, KEY_QUERY_VALUE, &OptionKeyAttributes);
//         if (NT_SUCCESS(Status))
//         {
//             /* Check if this is safemode */
//             Status = NtQueryValueKey(KeyHandle,
//                                      &OptionValue,
//                                      KeyValuePartialInformation,
//                                      &KeyInfo,
//                                      sizeof(KeyInfo),
//                                      &ResultLength);
//             NtClose(KeyHandle);
//             if ((NT_SUCCESS(Status)) &&
//                  (KeyInfo.Type == REG_DWORD) &&
//                  (KeyInfo.DataLength == sizeof(ULONG)) &&
//                  (KeyInfo.Data[0] != FALSE))
//             {
//                 /* It is, so disable shims! */
//                 DisableShims = TRUE;
//             }
//         }
//
//         if (!DisableShims)
//         {
//             /* Open the app compatibility engine settings key */
//             Status = NtOpenKey(&KeyHandle, KEY_QUERY_VALUE, &AppCompatKeyAttributes);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Check if the app compat engine is turned off */
//                 Status = NtQueryValueKey(KeyHandle,
//                                          &DisableAppCompat,
//                                          KeyValuePartialInformation,
//                                          &KeyInfo,
//                                          sizeof(KeyInfo),
//                                          &ResultLength);
//                 NtClose(KeyHandle);
//                 if ((NT_SUCCESS(Status)) &&
//                     (KeyInfo.Type == REG_DWORD) &&
//                     (KeyInfo.DataLength == sizeof(ULONG)) &&
//                     (KeyInfo.Data[0] == TRUE))
//                 {
//                     /* It is, so disable shims! */
//                     DisableShims = TRUE;
//                 }
//             }
//         }
//         if (!DisableShims)
//         {
//             /* Finally, open the app compatibility policy key */
//             Status = NtOpenKey(&KeyHandle, KEY_QUERY_VALUE, &PolicyKeyAttributes);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Check if the system policy disables app compat */
//                 Status = NtQueryValueKey(KeyHandle,
//                                          &DisableEngine,
//                                          KeyValuePartialInformation,
//                                          &KeyInfo,
//                                          sizeof(KeyInfo),
//                                          &ResultLength);
//                 NtClose(KeyHandle);
//                 if ((NT_SUCCESS(Status)) &&
//                     (KeyInfo.Type == REG_DWORD) &&
//                     (KeyInfo.DataLength == sizeof(ULONG)) &&
//                     (KeyInfo.Data[0] == TRUE))
//                 {
//                     /* It does, so disable shims! */
//                     DisableShims = TRUE;
//                 }
//             }
//         }
//         g_ShimsDisabled = DisableShims;
//     }
//
//     /* Return if shims are disabled or not ("Enabled == 1" means disabled!) */
//     return g_ShimsDisabled ? TRUE : FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// BasepShimCacheCheckBypass(
//      PCWSTR ApplicationName,
//      HANDLE FileHandle,
//      PCWSTR Environment,
//      BOOL bUnknown,
//      PULONG pdwReason)
// {
//     DPRINT("fixme:(%S, %p, %S, %d, %p)\n", ApplicationName, FileHandle, Environment, bUnknown,
//            pdwReason);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// BasepShimCacheSearch(
//      PCWSTR ApplicationName,
//      HANDLE FileHandle)
// {
//     APPHELP_CACHE_SERVICE_LOOKUP Lookup;
//     RtlInitUnicodeString(&Lookup.ImageName, ApplicationName);
//     Lookup.ImageHandle = FileHandle;
//     return NT_SUCCESS(NtApphelpCacheControl(ApphelpCacheServiceLookup, &Lookup));
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// BasepCheckCacheExcludeList(
//      PCWSTR ApplicationName)
// {
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// BasepCheckCacheExcludeCustom(
//      PCWSTR ApplicationName)
// {
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// BasepShimCacheRemoveEntry(
//      PCWSTR ApplicationName)
// {
//     APPHELP_CACHE_SERVICE_LOOKUP Lookup;
//     RtlInitUnicodeString(&Lookup.ImageName, ApplicationName);
//     Lookup.ImageHandle = INVALID_HANDLE_VALUE;
//     NtApphelpCacheControl(ApphelpCacheServiceRemove, &Lookup);
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// BasepShimCacheLookup(
//      PCWSTR ApplicationName,
//      HANDLE FileHandle)
// {
//     DPRINT("fixme:(%S, %p)\n", ApplicationName, FileHandle);
//
//     if (!BasepShimCacheSearch(ApplicationName, FileHandle))
//         return FALSE;
//
//     if (!BasepCheckCacheExcludeList(ApplicationName) ||
//         !BasepCheckCacheExcludeCustom(ApplicationName))
//     {
//         BasepShimCacheRemoveEntry(ApplicationName);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// BaseCheckAppcompatCache(
//      PCWSTR ApplicationName,
//      HANDLE FileHandle,
//      PCWSTR Environment,
//      PULONG pdwReason)
// {
//     BOOL ret = FALSE;
//     ULONG dwReason;
//
//     DPRINT("(%S, %p, %S, %p)\n", ApplicationName, FileHandle, Environment, pdwReason);
//
//     dwReason = 0;
//     if (BasepShimCacheCheckBypass(ApplicationName, FileHandle, Environment, TRUE, &dwReason))
//     {
//         dwReason |= 2;
//     }
//     else
//     {
//         ret = BasepShimCacheLookup(ApplicationName, FileHandle);
//         if (!ret)
//             dwReason |= 1;
//     }
//
//     if (pdwReason)
//         *pdwReason = dwReason;
//
//     return ret;
// }
//
// static
// VOID
// BaseInitApphelp(VOID)
// {
//     WCHAR Buffer[MAX_PATH*2];
//     UNICODE_STRING DllPath = {0};
//     PVOID ApphelpAddress;
//     PVOID pApphelpCheckRunAppEx = NULL, pSdbPackAppCompatData = NULL;
//
//     RtlInitEmptyUnicodeString(&DllPath, Buffer, sizeof(Buffer));
//     RtlCopyUnicodeString(&DllPath, &BaseWindowsDirectory);
//     RtlAppendUnicodeToString(&DllPath, L"\\system32\\apphelp.dll");
//
//     if (NT_SUCCESS(LdrLoadDll(NULL, NULL, &DllPath, &ApphelpAddress)))
//     {
//         ANSI_STRING ProcName;
//
//         RtlInitAnsiString(&ProcName, "ApphelpCheckRunAppEx");
//         if (!NT_SUCCESS(LdrGetProcedureAddress(ApphelpAddress, &ProcName, 0, &pApphelpCheckRunAppEx)))
//             pApphelpCheckRunAppEx = NULL;
//
//         RtlInitAnsiString(&ProcName, "SdbPackAppCompatData");
//         if (!NT_SUCCESS(LdrGetProcedureAddress(ApphelpAddress, &ProcName, 0, &pSdbPackAppCompatData)))
//             pSdbPackAppCompatData = NULL;
//     }
//
//     if (InterlockedCompareExchangePointer(&g_pApphelpCheckRunAppEx, RtlEncodeSystemPointer(pApphelpCheckRunAppEx), NULL) == NULL)
//     {
//         g_pSdbPackAppCompatData = RtlEncodeSystemPointer(pSdbPackAppCompatData);
//     }
// }
//
// /*
//  *
//  */
// BOOL
// WINAPI
// BaseCheckRunApp(IN HANDLE FileHandle,
//                  IN PWCHAR ApplicationName,
//                  IN PWCHAR Environment,
//                  IN USHORT ExeType,
//                  IN PULONG pReason,
//                  IN PVOID* SdbQueryAppCompatData,
//                  IN PULONG SdbQueryAppCompatDataSize,
//                  IN PVOID* SxsData,
//                  IN PULONG SxsDataSize,
//                  OUT PULONG FusionFlags)
// {
//     ULONG Reason = 0;
//     ULONG64 Flags1 = 0;
//     ULONG Flags2 = 0;
//     BOOL Continue, NeedCleanup = FALSE;
//     tApphelpCheckRunAppEx pApphelpCheckRunAppEx;
//     tSdbPackAppCompatData pSdbPackAppCompatData;
//     PVOID QueryResult = NULL;
//     ULONG QueryResultSize = 0;
//
//     if (!g_ApphelpInitialized)
//     {
//         BaseInitApphelp();
//         g_ApphelpInitialized = TRUE;
//     }
//
//     pApphelpCheckRunAppEx = RtlDecodeSystemPointer(g_pApphelpCheckRunAppEx);
//     pSdbPackAppCompatData = RtlDecodeSystemPointer(g_pSdbPackAppCompatData);
//
//     if (!pApphelpCheckRunAppEx || !pSdbPackAppCompatData)
//         return TRUE;
//
//     if (pReason)
//         Reason = *pReason;
//
//     Continue = pApphelpCheckRunAppEx(FileHandle, NULL, NULL, ApplicationName, Environment, ExeType, &Reason,
//         &QueryResult, &QueryResultSize, SxsData, SxsDataSize, FusionFlags, &Flags1, &Flags2);
//
//     if (pReason)
//         *pReason = Reason;
//
//     if (Continue)
//     {
//         if ((Reason & (APPHELP_VALID_RESULT|APPHELP_RESULT_FOUND)) == (APPHELP_VALID_RESULT|APPHELP_RESULT_FOUND))
//         {
//             if (!pSdbPackAppCompatData(NULL, QueryResult, SdbQueryAppCompatData, SdbQueryAppCompatDataSize))
//             {
//                 DPRINT1("SdbPackAppCompatData returned a failure!\n");
//                 NeedCleanup = TRUE;
//             }
//         }
//         else
//         {
//             NeedCleanup = TRUE;
//         }
//     }
//
//     if (QueryResult)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, QueryResult);
//
//     if (NeedCleanup)
//     {
//         BasepFreeAppCompatData(*SdbQueryAppCompatData, *SxsData);
//         *SdbQueryAppCompatData = NULL;
//         if (SdbQueryAppCompatDataSize)
//             *SdbQueryAppCompatDataSize = 0;
//         *SxsData = NULL;
//         if (SxsDataSize)
//             *SxsDataSize = 0;
//     }
//
//     return Continue;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// BasepCheckBadapp(IN HANDLE FileHandle,
//                  IN PWCHAR ApplicationName,
//                  IN PWCHAR Environment,
//                  IN USHORT ExeType,
//                  IN PVOID* SdbQueryAppCompatData,
//                  IN PULONG SdbQueryAppCompatDataSize,
//                  IN PVOID* SxsData,
//                  IN PULONG SxsDataSize,
//                  OUT PULONG FusionFlags)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Reason = 0;
//
//     /* Is shimming enabled by group policy? */
//     if (IsShimInfrastructureDisabled())
//     {
//         /* Nothing to worry about */
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* It is, check if we know about this app */
//         if (!BaseCheckAppcompatCache(ApplicationName,
//                                      FileHandle,
//                                      Environment,
//                                      &Reason))
//         {
//             if (!BaseCheckRunApp(FileHandle, ApplicationName, Environment, ExeType, &Reason,
//                                 SdbQueryAppCompatData, SdbQueryAppCompatDataSize, SxsData, SxsDataSize, FusionFlags))
//             {
//                 Status = STATUS_ACCESS_DENIED;
//             }
//         }
//     }
//
//     /* Return caller the status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// BaseDumpAppcompatCache(VOID)
// {
//     NTSTATUS Status;
//
//     Status = NtApphelpCacheControl(ApphelpCacheServiceDump, NULL);
//     return NT_SUCCESS(Status);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// BaseFlushAppcompatCache(VOID)
// {
//     NTSTATUS Status;
//
//     Status = NtApphelpCacheControl(ApphelpCacheServiceFlush, NULL);
//     return NT_SUCCESS(Status);
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// BasepFreeAppCompatData(IN PVOID AppCompatData,
//                        IN PVOID AppCompatSxsData)
// {
//     /* Free the input pointers if present */
//     if (AppCompatData) RtlFreeHeap(RtlGetProcessHeap(), 0, AppCompatData);
//     if (AppCompatSxsData) RtlFreeHeap(RtlGetProcessHeap(), 0, AppCompatSxsData);
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// BaseUpdateAppcompatCache(ULONG Unknown1,
//                          ULONG Unknown2,
//                          ULONG Unknown3)
// {
//     STUB;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// BaseCleanupAppcompatCache(VOID)
// {
//     STUB;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// BaseCleanupAppcompatCacheSupport(PVOID pUnknown)
// {
//     STUB;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// BaseInitAppcompatCache(VOID)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// BaseInitAppcompatCacheSupport(VOID)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// PVOID
// WINAPI
// GetComPlusPackageInstallStatus(VOID)
// {
//     STUB;
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetComPlusPackageInstallStatus(IN ULONG ComPlusPackage)
// {
//     NTSTATUS Status;
//
//     DPRINT("(0x%X)\n", ComPlusPackage);
//
//     if (ComPlusPackage & ~1)
//     {
//         DPRINT1("0x%lX\n", ComPlusPackage);
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     Status = NtSetSystemInformation(SystemComPlusPackage, &ComPlusPackage, sizeof(ComPlusPackage));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("0x%lX\n", Status);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// SetTermsrvAppInstallMode(IN BOOL bInstallMode)
// {
//     STUB;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// TermsrvAppInstallMode(VOID)
// {
//     STUB;
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

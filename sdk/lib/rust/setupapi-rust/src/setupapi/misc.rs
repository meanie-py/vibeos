// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0049D77223E9: [u16; 17] = [
    0x0072u16, 0x0065, 0x0070, 0x0061, 0x0069, 0x0072, 0x005C, 0x0073, 0x0065, 0x0074,
    0x0075, 0x0070, 0x002E, 0x006C, 0x006F, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_A5DCF5B8418D: [u16; 2] = [0x007Bu16, 0x0000];
pub static RUSTCONV_WSTR_DD5E8FF17033: [u16; 2] = [0x007Du16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_7 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_6 {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_7,
}

static mut BackSlash: [WCHAR; 2] = ['\\' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut TranslationRegKey: [WCHAR; 25] = [
    '\\' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut setupact: HANDLE = unsafe { INVALID_HANDLE_VALUE };

static mut setuperr: HANDLE = unsafe { INVALID_HANDLE_VALUE };

static mut critsect_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut setupapi_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut critsect_debug.ProcessLocksList,
            Blink: &raw mut critsect_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut setupapi_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const critsect_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

#[no_mangle]
pub static mut GlobalSetupFlags: DWORD = 0 as DWORD;

#[repr(C)]
pub struct callback_context {
    pub has_extracted: BOOL,
    pub target: LPCWSTR,
}
#[no_mangle]
pub unsafe extern "C" fn GetFunctionPointer(
    InstallerName: PWSTR,
    ModulePointer: *mut HMODULE,
    FunctionPointer: *mut PVOID,
) -> DWORD {
    let mut hModule: HMODULE = core::ptr::null_mut();
    let mut FunctionNameA: LPSTR = core::ptr::null_mut();
    let mut Comma: PWCHAR = core::ptr::null_mut();
    let mut rc: DWORD = 0;

    *ModulePointer = core::ptr::null_mut();
    *FunctionPointer = core::ptr::null_mut();

    'cleanup: {
        Comma = strchrW(InstallerName, ',' as u16);
        if Comma.is_null() {
            rc = ERROR_INVALID_PARAMETER as DWORD;
            break 'cleanup;
        }

        /* Load library */
        *Comma = '\0' as u16;
        hModule = LoadLibraryW(InstallerName as LPCWSTR) as HMODULE;
        *Comma = ',' as u16;
        if hModule.is_null() {
            rc = GetLastError() as DWORD;
            break 'cleanup;
        }

        /* Skip comma spaces */
        while *Comma == ',' as u16 || isspaceW(*Comma) != 0 {
            Comma = Comma.offset(1);
        }

        /* W->A conversion for function name */
        FunctionNameA = pSetupUnicodeToMultiByte(Comma as LPCWSTR, CP_ACP as UINT) as LPSTR;
        if FunctionNameA.is_null() {
            rc = GetLastError() as DWORD;
            break 'cleanup;
        }

        /* Search function */
        *FunctionPointer = core::mem::transmute::<FARPROC, PVOID>(
            GetProcAddress(hModule as HINSTANCE, FunctionNameA as LPCSTR),
        );
        if (*FunctionPointer).is_null() {
            rc = GetLastError() as DWORD;
            break 'cleanup;
        }

        *ModulePointer = hModule;
        rc = ERROR_SUCCESS as DWORD;
    }

    if rc != ERROR_SUCCESS as DWORD && !hModule.is_null() {
        FreeLibrary(hModule);
    }
    MyFree(FunctionNameA as LPVOID);
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn FreeFunctionPointer(
    mut ModulePointer: HMODULE,
    mut FunctionPointer: PVOID,
) -> DWORD {
    if ModulePointer.is_null() {
        return ERROR_SUCCESS as DWORD;
    }
    if FreeLibrary(ModulePointer) != 0 {
        return ERROR_SUCCESS as DWORD;
    } else {
        return GetLastError();
    }
}

#[no_mangle]
pub unsafe extern "system" fn MyFree(lpMem: LPVOID) {
    HeapFree(GetProcessHeap(), 0, lpMem);
}

#[no_mangle]
pub unsafe extern "system" fn MyMalloc(dwSize: DWORD) -> LPVOID {
    return HeapAlloc(GetProcessHeap(), 0, (dwSize) as SIZE_T);
}

#[no_mangle]
pub unsafe extern "system" fn MyRealloc(lpSrc: LPVOID, dwSize: DWORD) -> LPVOID {
    if lpSrc.is_null() {
        return HeapAlloc(GetProcessHeap(), 0, dwSize as SIZE_T);
    }

    return HeapReAlloc(GetProcessHeap(), 0, lpSrc, dwSize as SIZE_T);
}

#[no_mangle]
pub unsafe extern "system" fn pSetupDuplicateString(lpSrc: LPCWSTR) -> LPWSTR {
    let mut lpDst: LPWSTR;

    lpDst = MyMalloc(
        ((lstrlenW(lpSrc) + 1) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
    ) as LPWSTR;
    if lpDst.is_null() {
        return core::ptr::null_mut::<WCHAR>();
    }

    strcpyW(lpDst, lpSrc);

    return lpDst;
}
#[no_mangle]
pub unsafe extern "system" fn QueryRegistryValue(
    hKey: HKEY,
    lpValueName: LPCWSTR,
    lpData: *mut LPBYTE,
    lpType: LPDWORD,
    lpcbData: LPDWORD,
) -> LONG {
    let mut lError: LONG = 0;

    /* Get required buffer size */
    *lpcbData = 0;
    lError = RegQueryValueExW(hKey, lpValueName, core::ptr::null_mut(), lpType, core::ptr::null_mut(), lpcbData);
    if lError != (ERROR_SUCCESS as LONG) {
        return lError;
    }

    /* Allocate buffer */
    *lpData = MyMalloc(*lpcbData) as LPBYTE;
    if (*lpData).is_null() {
        return (ERROR_NOT_ENOUGH_MEMORY) as LONG;
    }

    /* Query registry value */
    lError = RegQueryValueExW(hKey, lpValueName, core::ptr::null_mut(), lpType, *lpData, lpcbData);
    if lError != (ERROR_SUCCESS as LONG) {
        MyFree(*lpData as LPVOID);
    }

    return lError;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupMultiByteToUnicode(lpMultiByteStr: LPCSTR, uCodePage: UINT) -> LPWSTR {
    let mut lpUnicodeStr: LPWSTR = core::ptr::null_mut();
    let mut nLength: i32 = 0;

    DPRINT1!("%s %d\n", debugstr_a(lpMultiByteStr as *const i8), uCodePage);

    nLength = MultiByteToWideChar(uCodePage, 0, lpMultiByteStr,
                                  -1, core::ptr::null_mut(), 0);
    if nLength == 0 {
        return core::ptr::null_mut();
    }

    lpUnicodeStr = MyMalloc(
        (nLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
    ) as LPWSTR;
    if lpUnicodeStr.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    if MultiByteToWideChar(uCodePage, 0, lpMultiByteStr,
                             nLength, lpUnicodeStr, nLength) == 0
    {
        MyFree(lpUnicodeStr as LPVOID);
        return core::ptr::null_mut();
    }

    return lpUnicodeStr;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupUnicodeToMultiByte(lpUnicodeStr: LPCWSTR, uCodePage: UINT) -> LPSTR {
    let mut lpMultiByteStr: LPSTR = core::ptr::null_mut();
    let mut nLength: i32 = 0;
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/misc.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"pSetupUnicodeToMultiByte\0" as *const u8 as *const core::ffi::c_char,
            332 as i32,
            b"%s %d\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(lpUnicodeStr as *const WCHAR),
            uCodePage,
        );
    }
    nLength = WideCharToMultiByte(
        uCodePage,
        0 as DWORD,
        lpUnicodeStr,
        -(1 as i32),
        core::ptr::null_mut(),
        0 as i32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if nLength == 0 as i32 {
        return core::ptr::null_mut();
    }
    lpMultiByteStr = MyMalloc(nLength as DWORD) as LPSTR;
    if lpMultiByteStr.is_null() {
        return core::ptr::null_mut();
    }
    if WideCharToMultiByte(
        uCodePage,
        0 as DWORD,
        lpUnicodeStr,
        -(1 as i32),
        lpMultiByteStr,
        nLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        MyFree(lpMultiByteStr as LPVOID);
        return core::ptr::null_mut();
    }
    return lpMultiByteStr;
}

#[no_mangle]
pub unsafe extern "system" fn DoesUserHavePrivilege(lpPrivilegeName: LPCWSTR) -> BOOL {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;
    let mut lpPrivileges: PTOKEN_PRIVILEGES = core::ptr::null_mut();
    let mut PrivilegeLuid: LUID = core::mem::zeroed::<LUID>();
    let mut i: DWORD = 0;
    let mut bResult: BOOL = (FALSE as i32);

    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, core::ptr::addr_of_mut!(hToken)) == 0 {
        return (FALSE) as BOOL;
    }

    if GetTokenInformation(hToken, TokenPrivileges, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(dwSize)) == 0 {
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            CloseHandle(hToken);
            return (FALSE) as BOOL;
        }
    }

    lpPrivileges = MyMalloc(dwSize) as PTOKEN_PRIVILEGES;
    if lpPrivileges.is_null() {
        CloseHandle(hToken);
        return (FALSE) as BOOL;
    }

    if GetTokenInformation(hToken, TokenPrivileges, lpPrivileges as PVOID, dwSize, core::ptr::addr_of_mut!(dwSize)) == 0 {
        MyFree(lpPrivileges as LPVOID);
        CloseHandle(hToken);
        return (FALSE) as BOOL;
    }

    CloseHandle(hToken);

    if LookupPrivilegeValueW(core::ptr::null::<WCHAR>(), lpPrivilegeName, core::ptr::addr_of_mut!(PrivilegeLuid)) == 0 {
        MyFree(lpPrivileges as LPVOID);
        return (FALSE) as BOOL;
    }

    i = 0;
    while i < (*lpPrivileges).PrivilegeCount {
        if (*(core::ptr::addr_of_mut!((*lpPrivileges).Privileges) as *mut LUID_AND_ATTRIBUTES).offset(i as isize))
            .Luid
            .HighPart
            == PrivilegeLuid.HighPart
            && (*(core::ptr::addr_of_mut!((*lpPrivileges).Privileges) as *mut LUID_AND_ATTRIBUTES).offset(i as isize))
                .Luid
                .LowPart
                == PrivilegeLuid.LowPart
        {
            bResult = (TRUE as i32);
        }
        i = i.wrapping_add(1);
    }

    MyFree(lpPrivileges as LPVOID);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupEnablePrivilege(lpPrivilegeName: LPCWSTR, bEnable: BOOL) -> BOOL {
    let mut Privileges: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL;

    if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &raw mut hToken) == 0 {
        return (FALSE) as BOOL;
    }

    Privileges.PrivilegeCount = 1;
    Privileges.Privileges[0].Attributes = if bEnable != 0 {
        SE_PRIVILEGE_ENABLED
    } else {
        0
    };

    if LookupPrivilegeValueW(
        core::ptr::null::<WCHAR>(),
        lpPrivilegeName,
        &raw mut Privileges.Privileges[0].Luid,
    ) == 0
    {
        CloseHandle(hToken);
        return (FALSE) as BOOL;
    }

    bResult = AdjustTokenPrivileges(
        hToken,
        (FALSE) as BOOL,
        &raw mut Privileges,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    CloseHandle(hToken);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn DelayedMove(lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR) -> BOOL {
    return MoveFileExW(lpExistingFileName, lpNewFileName,
                       MOVEFILE_REPLACE_EXISTING | MOVEFILE_DELAY_UNTIL_REBOOT);
}

#[no_mangle]
pub unsafe extern "system" fn FileExists(lpFileName: LPCWSTR, lpFileFindData: LPWIN32_FIND_DATAW) -> BOOL {
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut uErrorMode: UINT = 0;
    let mut dwError: DWORD = 0;

    uErrorMode = SetErrorMode(SEM_FAILCRITICALERRORS as UINT);

    hFind = FindFirstFileW(lpFileName, core::ptr::addr_of_mut!(FindData));
    if hFind == INVALID_HANDLE_VALUE {
        dwError = GetLastError();
        SetErrorMode(uErrorMode);
        SetLastError(dwError);
        return (FALSE) as BOOL;
    }

    FindClose(hFind);

    if !lpFileFindData.is_null() {
        memcpy(
            lpFileFindData as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(FindData) as *const core::ffi::c_void,
            core::mem::size_of::<WIN32_FIND_DATAW>() as size_t,
        );
    }

    SetErrorMode(uErrorMode);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CaptureStringArg(pSrc: LPCWSTR, pDst: *mut LPWSTR) -> DWORD {
    if pDst.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    *pDst = pSetupDuplicateString(pSrc);

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupCaptureAndConvertAnsiArg(pSrc: LPCSTR, pDst: *mut LPWSTR) -> DWORD {
    if pDst.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    *pDst = pSetupMultiByteToUnicode(pSrc, CP_ACP as UINT);

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupOpenAndMapFileForRead(
    lpFileName: LPCWSTR,
    lpSize: LPDWORD,
    lpFile: LPHANDLE,
    lpMapping: LPHANDLE,
    lpBuffer: *mut LPVOID,
) -> DWORD {
    let dwError: DWORD;

    *lpFile = CreateFileW(lpFileName, GENERIC_READ, FILE_SHARE_READ, core::ptr::null_mut(),
                          OPEN_EXISTING, 0, core::ptr::null_mut());
    if *lpFile == INVALID_HANDLE_VALUE {
        return GetLastError();
    }

    *lpSize = GetFileSize(*lpFile, core::ptr::null_mut());
    if *lpSize == INVALID_FILE_SIZE {
        dwError = GetLastError();
        CloseHandle(*lpFile);
        return dwError;
    }

    *lpMapping = CreateFileMappingW(*lpFile, core::ptr::null_mut(), PAGE_READONLY, 0,
                                    *lpSize, core::ptr::null());
    if *lpMapping == core::ptr::null_mut() {
        dwError = GetLastError();
        CloseHandle(*lpFile);
        return dwError;
    }

    *lpBuffer = MapViewOfFile(*lpMapping, FILE_MAP_READ, 0, 0, (*lpSize) as SIZE_T);
    if *lpBuffer == core::ptr::null_mut() {
        dwError = GetLastError();
        CloseHandle(*lpMapping);
        CloseHandle(*lpFile);
        return dwError;
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupUnmapAndCloseFile(hFile: HANDLE, hMapping: HANDLE, lpBuffer: LPVOID) -> BOOL {
    if UnmapViewOfFile(lpBuffer) == 0 {
        return (FALSE) as BOOL;
    }

    if CloseHandle(hMapping) == 0 {
        return (FALSE) as BOOL;
    }

    if CloseHandle(hFile) == 0 {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn StampFileSecurity(lpFileName: LPCWSTR, pSecurityDescriptor: PSECURITY_DESCRIPTOR) -> DWORD {
    if SetFileSecurityW(lpFileName, (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION) as SECURITY_INFORMATION, pSecurityDescriptor) == 0 {
        return GetLastError();
    }

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn TakeOwnershipOfFile(lpFileName: LPCWSTR) -> DWORD {
    let mut SecDesc: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut pOwner: PTOKEN_OWNER = core::ptr::null_mut();
    let mut dwError: DWORD;
    let mut dwSize: DWORD = 0;

    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, core::ptr::addr_of_mut!(hToken)) == 0 {
        return GetLastError();
    }

    'fail: {
        if GetTokenInformation(hToken, TokenOwner, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(dwSize)) == 0 {
            break 'fail;
        }

        pOwner = MyMalloc(dwSize) as PTOKEN_OWNER;
        if pOwner.is_null() {
            CloseHandle(hToken);
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        if GetTokenInformation(hToken, TokenOwner, pOwner as LPVOID, dwSize, core::ptr::addr_of_mut!(dwSize)) == 0 {
            break 'fail;
        }

        if InitializeSecurityDescriptor(core::ptr::addr_of_mut!(SecDesc) as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION) == 0 {
            break 'fail;
        }

        if SetSecurityDescriptorOwner(core::ptr::addr_of_mut!(SecDesc) as PSECURITY_DESCRIPTOR, (*pOwner).Owner, (FALSE) as BOOL) == 0 {
            break 'fail;
        }

        if SetFileSecurityW(lpFileName, OWNER_SECURITY_INFORMATION, core::ptr::addr_of_mut!(SecDesc) as PSECURITY_DESCRIPTOR) == 0 {
            break 'fail;
        }

        MyFree(pOwner as LPVOID);
        CloseHandle(hToken);

        return ERROR_SUCCESS;
    }

    // fail:
    dwError = GetLastError();

    MyFree(pOwner as LPVOID);

    if !hToken.is_null() {
        CloseHandle(hToken);
    }

    return dwError;
}
#[no_mangle]
pub unsafe extern "system" fn RetreiveFileSecurity(
    lpFileName: LPCWSTR,
    pSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut SecDesc: PSECURITY_DESCRIPTOR;
    let mut dwSize: DWORD = 0x100;
    let mut dwError: DWORD;

    SecDesc = MyMalloc(dwSize) as PSECURITY_DESCRIPTOR;
    if SecDesc.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    if GetFileSecurityW(
        lpFileName,
        (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION) as SECURITY_INFORMATION,
        SecDesc,
        dwSize,
        core::ptr::addr_of_mut!(dwSize),
    ) != 0
    {
        *pSecurityDescriptor = SecDesc;
        return ERROR_SUCCESS;
    }

    dwError = GetLastError();
    if dwError != ERROR_INSUFFICIENT_BUFFER {
        MyFree(SecDesc as LPVOID);
        return dwError;
    }

    SecDesc = MyRealloc(SecDesc as LPVOID, dwSize) as PSECURITY_DESCRIPTOR;
    if SecDesc.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    if GetFileSecurityW(
        lpFileName,
        (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION) as SECURITY_INFORMATION,
        SecDesc,
        dwSize,
        core::ptr::addr_of_mut!(dwSize),
    ) != 0
    {
        *pSecurityDescriptor = SecDesc;
        return ERROR_SUCCESS;
    }

    dwError = GetLastError();
    MyFree(SecDesc as LPVOID);

    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupGetGlobalFlags() -> DWORD {
    return GlobalSetupFlags;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupModifyGlobalFlags(mask: DWORD, flags: DWORD) {
    if false {
        DPRINT1!("stub\n");
    }
    GlobalSetupFlags = (GlobalSetupFlags & !mask) | (flags & mask);
}

#[no_mangle]
pub unsafe extern "system" fn pSetupSetGlobalFlags(flags: DWORD) {
    pSetupModifyGlobalFlags(0xFFFFFFFF, flags);
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetNonInteractiveMode() -> BOOL {
    return (GlobalSetupFlags & PSPGF_NONINTERACTIVE as DWORD) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetNonInteractiveMode(NonInteractiveFlag: BOOL) -> BOOL {
    let mut OldValue: BOOL;
    OldValue = (GlobalSetupFlags & PSPGF_NONINTERACTIVE as DWORD) as BOOL;
    pSetupModifyGlobalFlags(PSPGF_NONINTERACTIVE as DWORD,
                            if NonInteractiveFlag != 0 { PSPGF_NONINTERACTIVE as DWORD } else { 0 as DWORD });

    return OldValue;
}
#[no_mangle]
pub unsafe extern "system" fn AssertFail(lpFile: LPSTR, uLine: UINT, lpMessage: LPSTR) {
    let mut szModule: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szBuffer: [CHAR; 2048] = [0; 2048];
    let mut lpName: LPSTR;

    GetModuleFileNameA(hInstance, szModule.as_mut_ptr() as LPSTR, MAX_PATH);
    lpName = strrchr(szModule.as_mut_ptr(), '\\' as i32) as LPSTR;
    if !lpName.is_null() {
        lpName = lpName.offset(1);
    } else {
        lpName = szModule.as_mut_ptr() as LPSTR;
    }

    wsprintfA(
        szBuffer.as_mut_ptr() as LPSTR,
        b"Assertion failure at line %u in file %s: %s\n\nCall DebugBreak()?\0".as_ptr() as LPCSTR,
        uLine,
        lpFile,
        lpMessage,
    );

    if MessageBoxA(
        core::ptr::null_mut::<HWND__>(),
        szBuffer.as_mut_ptr() as LPCSTR,
        lpName as LPCSTR,
        (MB_SETFOREGROUND | MB_TASKMODAL | MB_ICONERROR | MB_YESNO) as UINT,
    ) == (IDYES as i32)
    {
        DebugBreak();
    }
}
#[no_mangle]
pub unsafe extern "system" fn GetSetFileTimestamp(
    lpFileName: LPCWSTR,
    lpCreationTime: LPFILETIME,
    lpLastAccessTime: LPFILETIME,
    lpLastWriteTime: LPFILETIME,
    bSetFileTime: BOOLEAN,
) -> DWORD {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut bRet: BOOLEAN;
    let mut dwError: DWORD = ERROR_SUCCESS;

    hFile = CreateFileW(
        lpFileName,
        if bSetFileTime != 0 { GENERIC_WRITE } else { GENERIC_READ },
        (FILE_SHARE_READ | FILE_SHARE_WRITE),
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE {
        return GetLastError();
    }

    if bSetFileTime != 0 {
        bRet = SetFileTime(hFile, lpCreationTime, lpLastAccessTime, lpLastWriteTime) as BOOLEAN;
    } else {
        bRet = GetFileTime(hFile, lpCreationTime, lpLastAccessTime, lpLastWriteTime) as BOOLEAN;
    }

    if bRet == (FALSE as BOOLEAN) {
        dwError = GetLastError();
    }

    CloseHandle(hFile);

    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupGetFileTitle(lpFileName: LPCWSTR) -> LPWSTR {
    let mut ptr: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut ret: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut c: WCHAR = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/misc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"pSetupGetFileTitle\0" as *const u8 as *const ::core::ffi::c_char,
            1004 as ::core::ffi::c_int,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(lpFileName as *const WCHAR),
        );
    }
    ptr = lpFileName as LPWSTR;
    ret = ptr;
    loop {
        c = *ptr;
        if c == 0 as WCHAR {
            break;
        }
        ptr = ptr.offset(1);
        if c == '\\' as u16 || c == '/' as u16 || c == ':' as u16 {
            ret = ptr;
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupConcatenatePaths(
    lpPath: LPWSTR,
    lpAppend: LPCWSTR,
    dwBufferSize: DWORD,
    lpRequiredSize: LPDWORD,
) -> BOOL {
    let mut dwPathSize: DWORD;
    let mut dwAppendSize: DWORD;
    let mut dwTotalSize: DWORD;
    let mut bBackslash: BOOL = (FALSE as i32);

    dwPathSize = lstrlenW(lpPath as LPCWSTR) as DWORD;

    /* Ignore trailing backslash */
    if *lpPath.offset(dwPathSize.wrapping_sub(1) as isize) == ('\\' as u16) {
        dwPathSize = dwPathSize.wrapping_sub(1);
    }

    dwAppendSize = lstrlenW(lpAppend) as DWORD;

    /* Does the source string have a leading backslash? */
    if *lpAppend.offset(0 as isize) == ('\\' as u16) {
        bBackslash = (TRUE as i32);
        dwAppendSize = dwAppendSize.wrapping_sub(1);
    }

    dwTotalSize = dwPathSize.wrapping_add(dwAppendSize).wrapping_add(2);
    if !lpRequiredSize.is_null() {
        *lpRequiredSize = dwTotalSize;
    }

    /* Append a backslash to the destination string */
    if bBackslash == (FALSE as BOOL) {
        if dwPathSize < dwBufferSize {
            *lpPath.offset(dwPathSize.wrapping_sub(1) as isize) = '\\' as u16;
            dwPathSize = dwPathSize.wrapping_add(1);
        }
    }

    if dwPathSize.wrapping_add(dwAppendSize) < dwBufferSize {
        lstrcpynW(
            lpPath.offset(dwPathSize as isize),
            lpAppend,
            dwAppendSize as ::core::ffi::c_int,
        );
    }

    if dwBufferSize >= dwTotalSize {
        *lpPath.offset(dwTotalSize.wrapping_sub(1) as isize) = 0;
    }

    return (dwBufferSize >= dwTotalSize) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupCenterWindowRelativeToParent(mut hwnd: HWND) {
    let mut hwndOwner: HWND = core::ptr::null_mut();
    let mut ptOrigin: POINT = POINT { x: 0, y: 0 };
    let mut rcWindow: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcOwner: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut nWindowWidth: INT = 0;
    let mut nWindowHeight: INT = 0;
    let mut nOwnerWidth: INT = 0;
    let mut nOwnerHeight: INT = 0;
    let mut posX: INT = 0;
    let mut posY: INT = 0;

    hwndOwner = GetWindow(hwnd, GW_OWNER as UINT);
    if hwndOwner.is_null() {
        return;
    }

    ptOrigin.x = 0 as LONG;
    ptOrigin.y = 0 as LONG;
    ClientToScreen(hwndOwner, &raw mut ptOrigin);

    GetWindowRect(hwnd, &raw mut rcWindow);
    GetClientRect(hwndOwner, &raw mut rcOwner);

    nWindowWidth = (rcWindow.right - rcWindow.left) as INT;
    nWindowHeight = (rcWindow.bottom - rcWindow.top) as INT;

    nOwnerWidth = (rcOwner.right - rcOwner.left) as INT;
    nOwnerHeight = (rcOwner.bottom - rcOwner.top) as INT;

    posX = (((nOwnerWidth - nWindowWidth) / 2) + ptOrigin.x) as INT;
    posY = (((nOwnerHeight - nWindowHeight) / 2) + ptOrigin.y) as INT;

    MoveWindow(
        hwnd,
        posX,
        posY,
        nWindowWidth,
        nWindowHeight,
        0 as BOOL,
    );
}

#[no_mangle]
pub unsafe extern "system" fn pSetupGetVersionInfoFromImage(
    lpFileName: LPWSTR,
    lpFileVersion: PULARGE_INTEGER,
    lpVersionVarSize: LPWORD,
) -> BOOL {
    let mut dwHandle: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut lpInfo: LPVOID = core::ptr::null_mut();
    let mut uSize: UINT = 0;
    let mut lpFixedInfo: *mut VS_FIXEDFILEINFO = core::ptr::null_mut();
    let mut lpVarSize: LPWORD = core::ptr::null_mut();

    dwSize = GetFileVersionInfoSizeW(lpFileName as LPCWSTR, core::ptr::addr_of_mut!(dwHandle));
    if dwSize == 0 {
        return (FALSE) as BOOL;
    }

    lpInfo = MyMalloc(dwSize) as LPVOID;
    if lpInfo.is_null() {
        return (FALSE) as BOOL;
    }

    if GetFileVersionInfoW(
        lpFileName as LPCWSTR,
        0,
        dwSize,
        lpInfo as PVOID,
    ) == 0
    {
        MyFree(lpInfo);
        return (FALSE) as BOOL;
    }

    if VerQueryValueW(
        lpInfo as LPCVOID,
        core::ptr::addr_of!(BackSlash) as LPCWSTR,
        core::ptr::addr_of_mut!(lpFixedInfo) as *mut LPVOID,
        core::ptr::addr_of_mut!(uSize),
    ) == 0
    {
        MyFree(lpInfo);
        return (FALSE) as BOOL;
    }

    (*lpFileVersion).__bindgen_anon_1.LowPart = (*lpFixedInfo).dwFileVersionLS;
    (*lpFileVersion).__bindgen_anon_1.HighPart = (*lpFixedInfo).dwFileVersionMS;

    *lpVersionVarSize = 0;
    if VerQueryValueW(
        lpInfo as LPCVOID,
        core::ptr::addr_of!(TranslationRegKey) as LPCWSTR,
        core::ptr::addr_of_mut!(lpVarSize) as *mut LPVOID,
        core::ptr::addr_of_mut!(uSize),
    ) == 0
    {
        MyFree(lpInfo);
        return (TRUE) as BOOL;
    }

    if uSize >= 4 {
        *lpVersionVarSize = *lpVarSize;
    }

    MyFree(lpInfo);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupUninstallOEMInfW(inf_file: PCWSTR, flags: DWORD, reserved: PVOID) -> BOOL {
    static mut infW: [u16; 6] = [0x005Cu16, 0x0069, 0x006E, 0x0066, 0x005C, 0x0000];
    let mut target: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if inf_file.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if GetWindowsDirectoryW(
        &raw mut target as LPWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as UINT,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    strcatW(&raw mut target as *mut WCHAR, &raw const infW as *const WCHAR);
    strcatW(&raw mut target as *mut WCHAR, inf_file as *const WCHAR);

    if flags & SUOI_FORCEDELETE != 0 {
        return DeleteFileW(&raw mut target as *mut WCHAR as LPCWSTR);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupUninstallOEMInfA(
    inf_file: PCSTR,
    flags: DWORD,
    reserved: PVOID,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut inf_fileW: *mut WCHAR = core::ptr::null_mut();

    if !inf_file.is_null() && {
        inf_fileW = strdupAtoW(inf_file);
        inf_fileW.is_null()
    } {
        return (FALSE) as BOOL;
    }
    ret = SetupUninstallOEMInfW(inf_fileW as PCWSTR, flags, reserved);
    HeapFree(GetProcessHeap(), 0, inf_fileW as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn InstallCatalog(
    catalog: LPCSTR,
    basename: LPCSTR,
    fullname: LPSTR,
) -> DWORD {
    // FIXME("%s, %s, %p\n", debugstr_a(catalog), debugstr_a(basename), fullname);
    0
}

#[no_mangle]
pub unsafe extern "system" fn pSetupInstallCatalog(catalog: LPCWSTR, basename: LPCWSTR, fullname: LPWSTR) -> DWORD {
    let mut admin: HCATADMIN = core::ptr::null_mut();
    let mut cat: HCATINFO = core::ptr::null_mut();

    if CryptCATAdminAcquireContext(core::ptr::addr_of_mut!(admin), core::ptr::null(), 0) == 0 {
        return GetLastError();
    }

    cat = CryptCATAdminAddCatalog(admin, catalog as PWSTR, basename as PWSTR, 0);
    if cat.is_null() {
        let rc: DWORD = GetLastError();
        CryptCATAdminReleaseContext(admin, 0);
        return rc;
    }
    CryptCATAdminReleaseCatalogContext(admin, cat, 0);
    CryptCATAdminReleaseContext(admin, 0);

    if !fullname.is_null() {
        FIXME!("not returning full installed catalog path\n");
    }

    return NO_ERROR;
}

unsafe extern "C" fn detect_compression_type(file: LPCWSTR) -> UINT {
    let mut size: DWORD = 0;
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut type_: UINT = FILE_COMPRESSION_NONE;
    static LZ_MAGIC: [BYTE; 8] = [0x53, 0x5a, 0x44, 0x44, 0x88, 0xf0, 0x27, 0x33];
    static MSZIP_MAGIC: [BYTE; 4] = [0x4b, 0x57, 0x41, 0x4a];
    static NTCAB_MAGIC: [BYTE; 4] = [0x4d, 0x53, 0x43, 0x46];
    let mut buffer: [BYTE; 8] = [0; 8];

    handle = CreateFileW(file, GENERIC_READ, 0, core::ptr::null_mut(), OPEN_EXISTING, 0, core::ptr::null_mut());
    if handle == INVALID_HANDLE_VALUE {
        ERR!("cannot open file %s\n", debugstr_w(file));
        return FILE_COMPRESSION_NONE;
    }
    if ReadFile(handle, core::ptr::addr_of_mut!(buffer) as *mut BYTE as PVOID, core::mem::size_of::<[BYTE; 8]>() as DWORD, core::ptr::addr_of_mut!(size), core::ptr::null_mut()) == 0 || size as usize != core::mem::size_of::<[BYTE; 8]>() as usize {
        CloseHandle(handle);
        return FILE_COMPRESSION_NONE;
    }
    if memcmp(core::ptr::addr_of_mut!(buffer) as *mut BYTE as *const core::ffi::c_void, core::ptr::addr_of!(LZ_MAGIC) as *const BYTE as *const core::ffi::c_void, core::mem::size_of_val(&LZ_MAGIC) as size_t) == 0 {
        type_ = FILE_COMPRESSION_WINLZA;
    } else if memcmp(core::ptr::addr_of_mut!(buffer) as *mut BYTE as *const core::ffi::c_void, core::ptr::addr_of!(MSZIP_MAGIC) as *const BYTE as *const core::ffi::c_void, core::mem::size_of_val(&MSZIP_MAGIC) as size_t) == 0 {
        type_ = FILE_COMPRESSION_MSZIP;
    } else if memcmp(core::ptr::addr_of_mut!(buffer) as *mut BYTE as *const core::ffi::c_void, core::ptr::addr_of!(NTCAB_MAGIC) as *const BYTE as *const core::ffi::c_void, core::mem::size_of_val(&NTCAB_MAGIC) as size_t) == 0 {
        type_ = FILE_COMPRESSION_MSZIP; /* not a typo */
    }

    CloseHandle(handle);
    return type_;
}

unsafe extern "C" fn get_file_size(file: LPCWSTR, size: *mut DWORD) -> BOOL {
    let mut handle: HANDLE;

    handle = CreateFileW(file, GENERIC_READ, 0, core::ptr::null_mut(), OPEN_EXISTING, 0, core::ptr::null_mut());
    if handle == INVALID_HANDLE_VALUE {
        ERR!("cannot open file %s\n", debugstr_w(file));
        return (FALSE) as BOOL;
    }
    *size = GetFileSize(handle, core::ptr::null_mut());
    CloseHandle(handle);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn get_file_sizes_none(source: LPCWSTR, source_size: *mut DWORD, target_size: *mut DWORD) -> BOOL {
    let mut size: DWORD = 0;
    if get_file_size(source, core::ptr::addr_of_mut!(size)) == 0 { return (FALSE) as BOOL; }
    if !source_size.is_null() { *source_size = size; }
    if !target_size.is_null() { *target_size = size; }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn get_file_sizes_lz(source: LPCWSTR, source_size: *mut DWORD, target_size: *mut DWORD) -> BOOL {
    let mut size: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);

    if !source_size.is_null() {
        if get_file_size(source, core::ptr::addr_of_mut!(size)) == 0 {
            ret = (FALSE as i32);
        } else {
            *source_size = size;
        }
    }
    if !target_size.is_null() {
        let mut file: INT = 0;
        let mut of: OFSTRUCT = core::mem::zeroed();

        file = LZOpenFileW(source as LPWSTR, core::ptr::addr_of_mut!(of), OF_READ as WORD);
        if file < 0 {
            ERR!("cannot open source file for reading\n");
            return (FALSE) as BOOL;
        }
        *target_size = LZSeek(file, 0, 2) as DWORD;
        LZClose(file);
    }
    return ret;
}
unsafe extern "system" fn file_compression_info_callback(
    mut context: PVOID,
    mut notification: UINT,
    mut param1: UINT_PTR,
    mut param2: UINT_PTR,
) -> UINT {
    let mut size: *mut DWORD = context as *mut DWORD;
    let mut info: *mut FILE_IN_CABINET_INFO_W = param1 as *mut FILE_IN_CABINET_INFO_W;
    match notification {
        SPFILENOTIFY_FILEINCABINET => {
            *size = (*info).FileSize;
            return FILEOP_SKIP as UINT;
        }
        _ => return NO_ERROR as UINT,
    };
}

unsafe extern "C" fn get_file_sizes_cab(
    mut source: LPCWSTR,
    mut source_size: *mut DWORD,
    mut target_size: *mut DWORD,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);
    if !source_size.is_null() {
        if get_file_size(source, core::ptr::addr_of_mut!(size)) == 0 {
            ret = (FALSE as i32);
        } else {
            *source_size = size;
        }
    }
    if !target_size.is_null() {
        ret = SetupIterateCabinetW(
            source as PCWSTR,
            0,
            Some(
                file_compression_info_callback,
            ),
            target_size as PVOID,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFileCompressionInfoExA(
    source: PCSTR,
    name: PSTR,
    len: DWORD,
    required: PDWORD,
    source_size: PDWORD,
    target_size: PDWORD,
    type_: PUINT,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut nameW: *mut WCHAR = core::ptr::null_mut();
    let mut sourceW: *mut WCHAR = core::ptr::null_mut();
    let mut nb_chars: DWORD = 0;
    let mut nameA: LPSTR = core::ptr::null_mut();

    if source.is_null() || {
        sourceW = pSetupMultiByteToUnicode(source as LPCSTR, CP_ACP as UINT) as *mut WCHAR;
        sourceW.is_null()
    } {
        return (FALSE) as BOOL;
    }

    if !name.is_null() {
        ret = SetupGetFileCompressionInfoExW(
            sourceW as PCWSTR,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(nb_chars),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        nameW = HeapAlloc(
            GetProcessHeap(),
            0,
            (nb_chars as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if nameW.is_null() {
            MyFree(sourceW as LPVOID);
            return (FALSE) as BOOL;
        }
    }
    ret = SetupGetFileCompressionInfoExW(
        sourceW as PCWSTR,
        nameW as PWSTR,
        nb_chars,
        core::ptr::addr_of_mut!(nb_chars),
        source_size,
        target_size,
        type_,
    );
    if ret != 0 {
        nameA = pSetupUnicodeToMultiByte(nameW as LPCWSTR, CP_ACP as UINT) as LPSTR;
        if !nameA.is_null() {
            if !name.is_null() && len >= nb_chars {
                lstrcpyA(name as LPSTR, nameA as LPCSTR);
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                ret = (FALSE as i32);
            }
            MyFree(nameA as LPVOID);
        }
    }
    if !required.is_null() {
        *required = nb_chars;
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, nameW as PVOID);
    MyFree(sourceW as LPVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFileCompressionInfoExW(
    source: PCWSTR,
    name: PWSTR,
    len: DWORD,
    required: PDWORD,
    source_size: PDWORD,
    target_size: PDWORD,
    type_: PUINT,
) -> BOOL {
    let mut comp: UINT = 0;
    let mut ret: BOOL = (FALSE as i32);
    let mut source_len: DWORD = 0;

    if source.is_null() {
        return (FALSE) as BOOL;
    }

    source_len = (lstrlenW(source as LPCWSTR) + 1) as DWORD;
    if !required.is_null() {
        *required = source_len;
    }
    if !name.is_null() && len >= source_len {
        lstrcpyW(name as LPWSTR, source as LPCWSTR);
        ret = TRUE as BOOL;
    } else {
        return (FALSE) as BOOL;
    }

    comp = detect_compression_type(source as LPCWSTR);
    if !type_.is_null() {
        *type_ = comp;
    }

    match comp {
        FILE_COMPRESSION_MSZIP | FILE_COMPRESSION_NTCAB => {
            ret = get_file_sizes_cab(
                source as LPCWSTR,
                source_size as *mut DWORD,
                target_size as *mut DWORD,
            );
        }
        FILE_COMPRESSION_NONE => {
            ret = get_file_sizes_none(
                source as LPCWSTR,
                source_size as *mut DWORD,
                target_size as *mut DWORD,
            );
        }
        FILE_COMPRESSION_WINLZA => {
            ret = get_file_sizes_lz(
                source as LPCWSTR,
                source_size as *mut DWORD,
                target_size as *mut DWORD,
            );
        }
        _ => {}
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFileCompressionInfoA(
    source: PCSTR,
    name: *mut PSTR,
    source_size: PDWORD,
    target_size: PDWORD,
    type_0: PUINT,
) -> DWORD {
    let mut ret: BOOL = 0;
    let mut error: DWORD = 0;
    let mut required: DWORD = 0;
    let mut actual_name: LPSTR = core::ptr::null_mut::<CHAR>();

    TRACE!(
        "%s, %p, %p, %p, %p\n",
        debugstr_a(source as *const i8),
        name,
        source_size,
        target_size,
        type_0
    );

    if source.is_null()
        || name.is_null()
        || source_size.is_null()
        || target_size.is_null()
        || type_0.is_null()
    {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    ret = SetupGetFileCompressionInfoExA(
        source,
        core::ptr::null_mut::<CHAR>(),
        0 as DWORD,
        core::ptr::addr_of_mut!(required),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<core::ffi::c_uint>(),
    );
    actual_name = MyMalloc(required) as LPSTR;
    if actual_name.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    ret = SetupGetFileCompressionInfoExA(
        source,
        actual_name as PSTR,
        required,
        core::ptr::addr_of_mut!(required),
        source_size,
        target_size,
        type_0,
    );
    if ret == 0 {
        error = GetLastError();
        MyFree(actual_name as LPVOID);
        return error;
    }
    *name = actual_name as PSTR;
    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFileCompressionInfoW(
    source: PCWSTR,
    name: *mut PWSTR,
    source_size: PDWORD,
    target_size: PDWORD,
    type_0: PUINT,
) -> DWORD {
    let mut ret: BOOL;
    let mut error: DWORD;
    let mut required: DWORD = core::mem::zeroed();
    let mut actual_name: LPWSTR;

    if source.is_null() || name.is_null() || source_size.is_null() || target_size.is_null() || type_0.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    ret = SetupGetFileCompressionInfoExW(
        source,
        core::ptr::null_mut::<WCHAR>(),
        0,
        &mut required,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<UINT>(),
    );
    actual_name = MyMalloc(
        required.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
    ) as LPWSTR;
    if actual_name.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    ret = SetupGetFileCompressionInfoExW(
        source,
        actual_name as PWSTR,
        required,
        &mut required,
        source_size,
        target_size,
        type_0,
    );
    if ret == 0 {
        error = GetLastError();
        MyFree(actual_name as LPVOID);
        return error;
    }
    *name = actual_name as PWSTR;
    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn decompress_file_lz(source: LPCWSTR, target: LPCWSTR) -> DWORD {
    let mut ret: DWORD = 0;
    let mut error: LONG = 0;
    let mut src: INT = 0;
    let mut dst: INT = 0;
    let mut sof: OFSTRUCT = core::mem::zeroed();
    let mut dof: OFSTRUCT = core::mem::zeroed();

    src = LZOpenFileW(source as LPWSTR, core::ptr::addr_of_mut!(sof), OF_READ as WORD);
    if src < 0 {
        ERR!("cannot open source file for reading\n");
        return ERROR_FILE_NOT_FOUND;
    }
    dst = LZOpenFileW(target as LPWSTR, core::ptr::addr_of_mut!(dof), OF_CREATE as WORD);
    if dst < 0 {
        ERR!("cannot open target file for writing\n");
        LZClose(src);
        return ERROR_FILE_NOT_FOUND;
    }
    error = LZCopy(src, dst);
    if error >= 0 {
        ret = ERROR_SUCCESS;
    } else {
        WARN!("failed to decompress file %d\n", error);
        ret = ERROR_INVALID_DATA;
    }

    LZClose(src);
    LZClose(dst);
    return ret;
}
unsafe extern "system" fn decompress_or_copy_callback(
    mut context: PVOID,
    mut notification: UINT,
    mut param1: UINT_PTR,
    mut param2: UINT_PTR,
) -> UINT {
    let mut context_info: *mut callback_context = context as *mut callback_context;
    let mut info: *mut FILE_IN_CABINET_INFO_W = param1 as *mut FILE_IN_CABINET_INFO_W;
    match notification {
        SPFILENOTIFY_FILEINCABINET => {
            if (*context_info).has_extracted != 0 {
                return FILEOP_ABORT as UINT;
            }
            TRACE!(
                "Requesting extraction of cabinet file %s\n",
                wine_dbgstr_w((*info).NameInCabinet as *const WCHAR)
            );
            strcpyW(
                core::ptr::addr_of_mut!((*info).FullTargetName) as *mut WCHAR,
                (*context_info).target as *const WCHAR,
            );
            (*context_info).has_extracted = TRUE as BOOL;
            return FILEOP_DOIT as UINT;
        }
        _ => return NO_ERROR as UINT,
    };
}

unsafe extern "C" fn decompress_file_cab(source: LPCWSTR, target: LPCWSTR) -> DWORD {
    let mut context: callback_context = callback_context {
        has_extracted: 0,
        target: target,
    };
    let mut ret: BOOL = 0;

    ret = SetupIterateCabinetW(
        source as PCWSTR,
        0,
        Some(decompress_or_copy_callback),
        core::ptr::addr_of_mut!(context) as PVOID,
    );

    if ret != 0 {
        return ERROR_SUCCESS;
    } else {
        return GetLastError();
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetupDecompressOrCopyFileA(
    source: PCSTR,
    target: PCSTR,
    type_: PUINT,
) -> DWORD {
    let mut ret: DWORD = 0 as DWORD;
    let mut sourceW: *mut WCHAR = core::ptr::null_mut();
    let mut targetW: *mut WCHAR = core::ptr::null_mut();

    if !source.is_null() && {
        sourceW = pSetupMultiByteToUnicode(source as LPCSTR, CP_ACP as UINT) as *mut WCHAR;
        sourceW.is_null()
    } {
        return FALSE as DWORD;
    }
    if !target.is_null() && {
        targetW = pSetupMultiByteToUnicode(target as LPCSTR, CP_ACP as UINT) as *mut WCHAR;
        targetW.is_null()
    } {
        MyFree(sourceW as LPVOID);
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    ret = SetupDecompressOrCopyFileW(sourceW as PCWSTR, targetW as PCWSTR, type_);

    MyFree(sourceW as LPVOID);
    MyFree(targetW as LPVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDecompressOrCopyFileW(
    source: PCWSTR,
    target: PCWSTR,
    type_: PUINT,
) -> DWORD {
    let mut comp: UINT;
    let mut ret: DWORD = ERROR_INVALID_PARAMETER;

    if source.is_null() || target.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if type_.is_null() {
        comp = detect_compression_type(source);
    } else {
        comp = *type_;
    }

    match comp {
        FILE_COMPRESSION_NONE => {
            if CopyFileW(source, target, (FALSE) as BOOL) != 0 {
                ret = ERROR_SUCCESS;
            } else {
                ret = GetLastError();
            }
        }
        FILE_COMPRESSION_WINLZA => {
            ret = decompress_file_lz(source, target);
        }
        FILE_COMPRESSION_NTCAB | FILE_COMPRESSION_MSZIP => {
            ret = decompress_file_cab(source, target);
        }
        _ => {
            WARN!("unknown compression type %d\n", comp);
        }
    }

    return ret;
}

unsafe extern "C" fn TrimGuidString(szString: PCWSTR, szNewString: LPWSTR) -> BOOL {
    let mut szBuffer: [WCHAR; 39] = [0; 39];
    let mut Index: INT;

    if wcslen(szString) == 38 {
        if (*szString.offset(0 as isize) == 0x007Bu16)
            && (*szString.offset(37 as isize) == 0x007Du16)
        {
            Index = 0;
            while (Index as usize) < wcslen(szString) {
                szBuffer[Index as usize] = *szString.offset((Index + 1) as isize);
                Index += 1;
            }

            szBuffer[36 as usize] = 0x0000u16;
            wcscpy(szNewString, core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR);
            return (TRUE) as BOOL;
        }
    }
    *szNewString.offset(0 as isize) = 0x0000u16;
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupGuidFromString(pString: PCWSTR, lpGUID: LPGUID) -> DWORD {
    let mut Status: RPC_STATUS = 0;
    let mut szBuffer: [WCHAR; 39] = [0; 39];

    if TrimGuidString(pString, szBuffer.as_mut_ptr()) == 0 {
        return RPC_S_INVALID_STRING_UUID as DWORD;
    }

    Status = UuidFromStringW(szBuffer.as_mut_ptr(), lpGUID);
    if Status != RPC_S_OK as RPC_STATUS {
        return RPC_S_INVALID_STRING_UUID as DWORD;
    }

    return NO_ERROR as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringFromGuid(lpGUID: LPGUID, pString: PWSTR, dwStringLen: DWORD) -> DWORD {
    let mut Status: RPC_STATUS = 0;
    let mut rpcBuffer: RPC_WSTR = core::ptr::null_mut();
    let mut szBuffer: [WCHAR; 39] = [0; 39];

    if dwStringLen < 39 {
        return ERROR_INSUFFICIENT_BUFFER;
    }

    Status = UuidToStringW(lpGUID as *mut UUID, core::ptr::addr_of_mut!(rpcBuffer));
    if Status != RPC_S_OK {
        return Status as DWORD;
    }

    wcscpy(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR, RUSTCONV_WSTR_A5DCF5B8418D.as_ptr());
    wcscat(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR, rpcBuffer);
    wcscat(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR, RUSTCONV_WSTR_DD5E8FF17033.as_ptr());

    wcscpy(pString, core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR);

    RpcStringFreeW(core::ptr::addr_of_mut!(rpcBuffer));
    return NO_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupIsGuidNull(lpGUID: LPGUID) -> BOOL {
    return (IsEqualGUID!(lpGUID, core::ptr::addr_of!(GUID_NULL))) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupIsUserAdmin() -> BOOL {
    let mut Authority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, SECURITY_NT_AUTHORITY as u8],
    };
    let mut bResult: BOOL = (FALSE as i32);
    let mut lpSid: PSID = core::ptr::null_mut();

    if AllocateAndInitializeSid(
        core::ptr::addr_of_mut!(Authority),
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS,
        0,
        0,
        0,
        0,
        0,
        0,
        core::ptr::addr_of_mut!(lpSid),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if CheckTokenMembership(core::ptr::null_mut(), lpSid, core::ptr::addr_of_mut!(bResult)) == 0 {
        bResult = (FALSE as i32);
    }

    FreeSid(lpSid);

    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn SetupInitializeFileLogW(LogFileName: LPCWSTR, Flags: DWORD) -> HSPFILELOG {
    let mut Log: *mut FileLog = core::ptr::null_mut();
    let mut hLog: HANDLE = core::ptr::null_mut();
    let mut Windir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ret: DWORD = 0;

    if (Flags & SPFILELOG_SYSTEMLOG) != 0 {
        if pSetupIsUserAdmin() == 0 && (Flags & SPFILELOG_QUERYONLY) == 0 {
            /* insufficient privileges */
            SetLastError(ERROR_ACCESS_DENIED);
            return INVALID_HANDLE_VALUE;
        }

        if !LogFileName.is_null() || (Flags & SPFILELOG_FORCENEW) != 0 {
            /* invalid parameter */
            SetLastError(ERROR_INVALID_PARAMETER);
            return INVALID_HANDLE_VALUE;
        }

        ret = GetSystemWindowsDirectoryW(core::ptr::addr_of_mut!(Windir) as LPWSTR, MAX_PATH) as DWORD;
        if ret == 0 || ret >= MAX_PATH {
            /* generic failure */
            return INVALID_HANDLE_VALUE;
        }

        /* append path */
        wcscat(core::ptr::addr_of_mut!(Windir) as *mut WCHAR, RUSTCONV_WSTR_0049D77223E9.as_ptr());
    } else {
        if LogFileName.is_null() {
            /* invalid parameter */
            SetLastError(ERROR_INVALID_PARAMETER);
            return INVALID_HANDLE_VALUE;
        }
        /* copy filename */
        wcsncpy(core::ptr::addr_of_mut!(Windir) as *mut WCHAR, LogFileName as *const WCHAR, (MAX_PATH) as _);
    }

    if FileExists(core::ptr::addr_of_mut!(Windir) as LPCWSTR, core::ptr::null_mut()) != 0 {
        /* take ownership */
        ret = TakeOwnershipOfFile(core::ptr::addr_of_mut!(Windir) as LPCWSTR);

        if ret != ERROR_SUCCESS {
            /* failed */
            SetLastError(ret);
            return INVALID_HANDLE_VALUE;
        }

        if SetFileAttributesW(core::ptr::addr_of_mut!(Windir) as LPCWSTR, FILE_ATTRIBUTE_NORMAL) == 0 {
            /* failed */
            return INVALID_HANDLE_VALUE;
        }

        if (Flags & SPFILELOG_FORCENEW) != 0 {
            if DeleteFileW(core::ptr::addr_of_mut!(Windir) as LPCWSTR) == 0 {
                /* failed */
                return INVALID_HANDLE_VALUE;
            }
        }
    }

    /* open log file */
    hLog = CreateFileW(
        core::ptr::addr_of_mut!(Windir) as LPCWSTR,
        if (Flags & SPFILELOG_QUERYONLY) != 0 { GENERIC_READ } else { GENERIC_WRITE },
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    if hLog == INVALID_HANDLE_VALUE {
        /* failed */
        return INVALID_HANDLE_VALUE;
    }

    /* close log handle */
    CloseHandle(hLog);

    /* allocate file log struct */
    Log = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<FileLog>() as SIZE_T,
    ) as *mut FileLog;
    if Log.is_null() {
        /* not enough memory */
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return INVALID_HANDLE_VALUE;
    }

    /* initialize log */
    (*Log).LogName = HeapAlloc(
        GetProcessHeap(),
        0,
        (wcslen(core::ptr::addr_of_mut!(Windir) as *mut WCHAR) as SIZE_T)
            .wrapping_add(1 as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if (*Log).LogName.is_null() {
        /* not enough memory */
        HeapFree(GetProcessHeap(), 0, Log as PVOID);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return INVALID_HANDLE_VALUE;
    }

    wcscpy((*Log).LogName as *mut WCHAR, core::ptr::addr_of_mut!(Windir) as *mut WCHAR);
    (*Log).ReadOnly = Flags & SPFILELOG_QUERYONLY;
    (*Log).SystemLog = Flags & SPFILELOG_SYSTEMLOG;

    return Log as HSPFILELOG;
}

#[no_mangle]
pub unsafe extern "system" fn SetupInitializeFileLogA(LogFileName: LPCSTR, Flags: DWORD) -> HSPFILELOG {
    let mut hLog: HSPFILELOG;
    let mut LogFileNameW: LPWSTR = core::ptr::null_mut();

    if !LogFileName.is_null() {
        LogFileNameW = strdupAtoW(LogFileName);

        if LogFileNameW.is_null() {
            /* not enough memory */
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return INVALID_HANDLE_VALUE;
        }

        hLog = SetupInitializeFileLogW(LogFileNameW as LPCWSTR, Flags);
        HeapFree(GetProcessHeap(), 0 as DWORD, LogFileNameW as PVOID);
    } else {
        hLog = SetupInitializeFileLogW(core::ptr::null(), Flags);
    }

    return hLog;
}

#[no_mangle]
pub unsafe extern "system" fn SetupTerminateFileLog(FileLogHandle: HANDLE) -> BOOL {
    let mut Log: *mut FileLog = core::ptr::null_mut();

    Log = FileLogHandle as *mut FileLog;

    /* free file log handle */
    HeapFree(GetProcessHeap(), 0, (*Log).LogName as PVOID);
    HeapFree(GetProcessHeap(), 0, Log as PVOID);

    SetLastError(ERROR_SUCCESS as DWORD);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupCloseLog() {
    EnterCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));

    CloseHandle(setupact);
    setupact = INVALID_HANDLE_VALUE;

    CloseHandle(setuperr);
    setuperr = INVALID_HANDLE_VALUE;

    LeaveCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenLog(reserved: BOOL) -> BOOL {
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    static setupactlog: [WCHAR; 14] = [
    0x005Cu16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0061, 0x0063, 0x0074, 0x002E,
    0x006C, 0x006F, 0x0067, 0x0000,
];
    static setuperrlog: [WCHAR; 14] = [
    0x005Cu16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0065, 0x0072, 0x0072, 0x002E,
    0x006C, 0x006F, 0x0067, 0x0000,
];

    EnterCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));

    if setupact != INVALID_HANDLE_VALUE && setuperr != INVALID_HANDLE_VALUE {
        LeaveCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));
        return (TRUE) as BOOL;
    }

    GetWindowsDirectoryW(core::ptr::addr_of_mut!(path) as LPWSTR, MAX_PATH);
    lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, core::ptr::addr_of!(setupactlog) as LPCWSTR);

    setupact = CreateFileW(
        core::ptr::addr_of_mut!(path) as *mut WCHAR as LPCWSTR,
        FILE_GENERIC_WRITE,
        FILE_SHARE_WRITE | FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if setupact == INVALID_HANDLE_VALUE {
        LeaveCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));
        return (FALSE) as BOOL;
    }

    SetFilePointer(setupact, 0, core::ptr::null_mut(), FILE_END);

    GetWindowsDirectoryW(core::ptr::addr_of_mut!(path) as LPWSTR, MAX_PATH);
    lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, core::ptr::addr_of!(setuperrlog) as LPCWSTR);

    setuperr = CreateFileW(
        core::ptr::addr_of_mut!(path) as *mut WCHAR as LPCWSTR,
        FILE_GENERIC_WRITE,
        FILE_SHARE_WRITE | FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if setuperr == INVALID_HANDLE_VALUE {
        CloseHandle(setupact);
        setupact = INVALID_HANDLE_VALUE;
        LeaveCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));
        return (FALSE) as BOOL;
    }

    SetFilePointer(setuperr, 0, core::ptr::null_mut(), FILE_END);

    LeaveCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupLogErrorA(mut message: LPCSTR, mut severity: DWORD) -> BOOL {
    static null: [u8; 7] = *b"(null)\0";
    let mut ret: BOOL = 0;
    let mut written: DWORD = 0;
    let mut len: DWORD = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));

    'done: {
        if setupact == INVALID_HANDLE_VALUE || setuperr == INVALID_HANDLE_VALUE {
            SetLastError(ERROR_FILE_INVALID as DWORD);
            ret = (FALSE as i32);
            break 'done;
        }

        if message.is_null() {
            message = core::ptr::addr_of!(null) as *const i8 as LPCSTR;
        }

        len = lstrlenA(message) as DWORD;

        ret = WriteFile(
            setupact,
            message as LPCVOID,
            len,
            core::ptr::addr_of_mut!(written),
            core::ptr::null_mut(),
        );
        if ret == 0 {
            break 'done;
        }

        if severity >= LogSevMaximum {
            ret = (FALSE as i32);
            break 'done;
        }

        if severity > LogSevInformation {
            ret = WriteFile(
                setuperr,
                message as LPCVOID,
                len,
                core::ptr::addr_of_mut!(written),
                core::ptr::null_mut(),
            );
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(setupapi_cs));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupLogErrorW(message: LPCWSTR, severity: LogSeverity) -> BOOL {
    let mut msg: LPSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut ret: BOOL = 0;
    if !message.is_null() {
        len = WideCharToMultiByte(CP_ACP, 0, message, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as DWORD;
        msg = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T) as LPSTR;
        if msg.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return (FALSE) as BOOL;
        }
        WideCharToMultiByte(CP_ACP, 0, message, -1, msg, len as ::core::ffi::c_int, core::ptr::null(), core::ptr::null_mut());
    }

    /* This is the normal way to proceed. The log files are ASCII files
     * and W is to be converted.
     */
    ret = SetupLogErrorA(msg as LPCSTR, severity);

    HeapFree(GetProcessHeap(), 0, msg as PVOID);
    return ret;
}

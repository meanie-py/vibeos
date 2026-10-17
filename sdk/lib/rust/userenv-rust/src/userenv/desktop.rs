use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::RegOpenKeyExW;
use crate::prelude::RegCloseKey;
use crate::prelude::RegQueryValueExW;
use core::ffi::c_void;

extern "C" {
    fn wcscpy(dst: LPWSTR, src: LPCWSTR) -> LPWSTR;
    // userenv-internal helpers are cdecl (not WINAPI) — match their extern "C" definitions.
    fn CreateDirectoryPath(lpPath: LPCWSTR, lpSecurityAttributes: LPVOID) -> BOOL;
    fn LoadDynamicImports(pDynOle32: *mut DYN_MODULE, pOle32: *mut DYN_FUNCS) -> BOOL;
    fn UnloadDynamicImports(pOle32: *mut DYN_FUNCS) -> BOOL;
}
extern "system" {
    fn FindFirstFileW(lpFileName: LPCWSTR, lpFindFileData: *mut WIN32_FIND_DATAW) -> HANDLE;
    fn FindClose(hFindFile: HANDLE) -> BOOL;
    // SUCCEEDED removed: winerror.h MACRO ((HRESULT)(hr) >= 0), not a linkable fn → prelude shim.
    fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL;
    fn RemoveDirectoryW(lpPathName: LPCWSTR) -> BOOL;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;

// L"\"
static BACKSLASH_W: [u16; 2] = [b'\\' as u16, 0];
// L".lnk"
static DOT_LNK_W: [u16; 5] = [b'.' as u16, b'l' as u16, b'n' as u16, b'k' as u16, 0];
// L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders"
static USER_SHELL_FOLDERS_KEY: [u16; 70] = [
    b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16,
    b'e' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16,
    b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16, b'C' as u16,
    b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16,
    b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
    b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b'e' as u16,
    b'r' as u16, b'\\' as u16, b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b' ' as u16,
    b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16, b' ' as u16, b'F' as u16,
    b'o' as u16, b'l' as u16, b'd' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0,
];
// L"Common Desktop"
static COMMON_DESKTOP_VALUE: [u16; 15] = [
    b'C' as u16, b'o' as u16, b'm' as u16, b'm' as u16, b'o' as u16, b'n' as u16, b' ' as u16,
    b'D' as u16, b'e' as u16, b's' as u16, b'k' as u16, b't' as u16, b'o' as u16, b'p' as u16,
    0,
];
// L"Desktop"
static DESKTOP_VALUE: [u16; 8] = [
    b'D' as u16, b'e' as u16, b's' as u16, b'k' as u16, b't' as u16, b'o' as u16, b'p' as u16,
    0,
];
unsafe extern "C" fn GetDesktopPath(mut bCommonPath: BOOL, mut lpDesktopPath: LPWSTR) -> BOOL {
    let mut szPath: [u16; 260] = [0; 260];
    let mut dwLength: u32 = 0;
    let mut dwType: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: i32 = 0;

    Error = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        USER_SHELL_FOLDERS_KEY.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    );
    if Error != ERROR_SUCCESS as i32 {
        SetLastError(Error as u32);
        return 0;
    }

    dwLength = (MAX_PATH as usize * core::mem::size_of::<u16>()) as u32;
    Error = RegQueryValueExW(
        hKey,
        if bCommonPath != 0 {
            COMMON_DESKTOP_VALUE.as_ptr() as LPCWSTR
        } else {
            DESKTOP_VALUE.as_ptr() as LPCWSTR
        },
        core::ptr::null_mut(),
        &mut dwType,
        &mut szPath as *mut u16 as LPBYTE,
        &mut dwLength,
    );
    if Error != ERROR_SUCCESS as i32 {
        RegCloseKey(hKey);
        SetLastError(Error as u32);
        return 0;
    }

    RegCloseKey(hKey);

    if dwType == REG_EXPAND_SZ {
        ExpandEnvironmentStringsW(
            &mut szPath as *mut u16 as LPCWSTR,
            lpDesktopPath,
            MAX_PATH,
        );
    } else {
        wcscpy(lpDesktopPath, &mut szPath as *mut u16);
    }

    1
}
unsafe extern "C" fn GetProgramsPath(mut bCommonPath: BOOL, mut lpProgramsPath: LPWSTR) -> BOOL {
    let mut szPath: [u16; 260] = [0; 260];
    let mut dwLength: u32 = 0;
    let mut dwType: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut Error: i32 = 0;

    Error = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        win32macros_rust::wide_bytes!(b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0E\0x\0p\0l\0o\0r\0e\0r\0\\\0U\0s\0e\0r\0 \0S\0h\0e\0l\0l\0 \0F\0o\0l\0d\0e\0r\0s\0\0\0") as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    );
    if Error != ERROR_SUCCESS as i32 {
        SetLastError(Error as u32);
        return 0;
    }

    dwLength = (MAX_PATH as usize * core::mem::size_of::<u16>()) as u32;
    Error = RegQueryValueExW(
        hKey,
        if bCommonPath != 0 {
            win32macros_rust::wide_bytes!(b"C\0o\0m\0m\0o\0n\0 \0P\0r\0o\0g\0r\0a\0m\0s\0\0\0") as LPCWSTR
        } else {
            win32macros_rust::wide_bytes!(b"P\0r\0o\0g\0r\0a\0m\0s\0\0\0") as LPCWSTR
        },
        core::ptr::null_mut(),
        &mut dwType,
        &mut szPath as *mut u16 as LPBYTE,
        &mut dwLength,
    );
    if Error != ERROR_SUCCESS as i32 {
        RegCloseKey(hKey);
        SetLastError(Error as u32);
        return 0;
    }

    RegCloseKey(hKey);

    if dwType == REG_EXPAND_SZ {
        ExpandEnvironmentStringsW(
            &mut szPath as *mut u16 as LPCWSTR,
            lpProgramsPath,
            MAX_PATH as u32,
        );
    } else {
        wcscpy(lpProgramsPath, &mut szPath as *mut u16 as LPCWSTR);
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn AddDesktopItemA(
    bCommonItem: BOOL,
    lpItemName: LPCSTR,
    lpArguments: LPCSTR,
    lpIconLocation: LPCSTR,
    iIcon: INT,
    lpWorkingDirectory: LPCSTR,
    wHotKey: WORD,
    iShowCmd: INT,
) -> BOOL {
    let mut ItemName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Arguments: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut IconLocation: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut WorkingDirectory: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bResult: BOOL;

    if RtlCreateUnicodeStringFromAsciiz(&mut ItemName, lpItemName as LPSTR) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if RtlCreateUnicodeStringFromAsciiz(&mut Arguments, lpArguments as LPSTR) == 0 {
        RtlFreeUnicodeString(&mut ItemName);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if RtlCreateUnicodeStringFromAsciiz(&mut IconLocation, lpIconLocation as LPSTR) == 0 {
        RtlFreeUnicodeString(&mut Arguments);
        RtlFreeUnicodeString(&mut ItemName);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if !lpWorkingDirectory.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut WorkingDirectory, lpWorkingDirectory as LPSTR) == 0 {
            RtlFreeUnicodeString(&mut IconLocation);
            RtlFreeUnicodeString(&mut Arguments);
            RtlFreeUnicodeString(&mut ItemName);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }

    bResult = AddDesktopItemW(
        bCommonItem,
        ItemName.Buffer,
        Arguments.Buffer,
        IconLocation.Buffer,
        iIcon,
        if !lpWorkingDirectory.is_null() {
            WorkingDirectory.Buffer
        } else {
            core::ptr::null_mut()
        },
        wHotKey,
        iShowCmd,
    );

    if !lpWorkingDirectory.is_null() {
        RtlFreeUnicodeString(&mut WorkingDirectory);
    }

    RtlFreeUnicodeString(&mut IconLocation);
    RtlFreeUnicodeString(&mut Arguments);
    RtlFreeUnicodeString(&mut ItemName);

    bResult
}

// IShellLinkW / IPersistFile, as far as AddDesktopItemW and AddItemW reach into them. Slot
// order is shobjidl.idl's / objidl.idl's (checked against syssetup-rust's bindgen
// IShellLinkWVtbl / IPersistFileVtbl); the slots these two never call are kept as
// pointer-sized placeholders so every used slot sits at its real offset.
#[repr(C)]
struct IShellLinkW {
    lpVtbl: *const IShellLinkWVtbl,
}
#[repr(C)]
struct IShellLinkWVtbl {
    QueryInterface:
        unsafe extern "system" fn(This: *mut IShellLinkW, riid: *const IID, ppv: *mut LPVOID) -> HRESULT,
    AddRef: usize,
    Release: unsafe extern "system" fn(This: *mut IShellLinkW) -> ULONG,
    GetPath: usize,
    GetIDList: usize,
    SetIDList: usize,
    GetDescription: usize,
    SetDescription: unsafe extern "system" fn(This: *mut IShellLinkW, pszName: LPCWSTR) -> HRESULT,
    GetWorkingDirectory: usize,
    SetWorkingDirectory: unsafe extern "system" fn(This: *mut IShellLinkW, pszDir: LPCWSTR) -> HRESULT,
    GetArguments: usize,
    SetArguments: unsafe extern "system" fn(This: *mut IShellLinkW, pszArgs: LPCWSTR) -> HRESULT,
    GetHotkey: usize,
    SetHotkey: unsafe extern "system" fn(This: *mut IShellLinkW, wHotkey: WORD) -> HRESULT,
    GetShowCmd: usize,
    SetShowCmd: unsafe extern "system" fn(This: *mut IShellLinkW, iShowCmd: INT) -> HRESULT,
    GetIconLocation: usize,
    SetIconLocation:
        unsafe extern "system" fn(This: *mut IShellLinkW, pszIconPath: LPCWSTR, iIcon: INT) -> HRESULT,
    SetRelativePath: usize,
    Resolve: usize,
    SetPath: unsafe extern "system" fn(This: *mut IShellLinkW, pszFile: LPCWSTR) -> HRESULT,
}
#[repr(C)]
struct IPersistFile {
    lpVtbl: *const IPersistFileVtbl,
}
#[repr(C)]
struct IPersistFileVtbl {
    QueryInterface: usize,
    AddRef: usize,
    Release: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    GetClassID: usize,
    IsDirty: usize,
    Load: usize,
    Save: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCWSTR, fRemember: BOOL) -> HRESULT,
    SaveCompleted: usize,
    GetCurFile: usize,
}
// From the uuid static library userenv links (dll/win32/userenv/CMakeLists.txt keeps it for these).
extern "C" {
    static CLSID_ShellLink: CLSID;
    static IID_IShellLinkW: IID;
    static IID_IPersistFile: IID;
}
// wtypes.idl: CLSCTX_INPROC_SERVER = 0x1 (not in this crate's bindgen island).
const CLSCTX_INPROC_SERVER: DWORD = 0x1;
// L"%HOMEDRIVE%%HOMEPATH%"
static HOME_DIR_W: [u16; 22] = [
    b'%' as u16, b'H' as u16, b'O' as u16, b'M' as u16, b'E' as u16, b'D' as u16, b'R' as u16,
    b'I' as u16, b'V' as u16, b'E' as u16, b'%' as u16, b'%' as u16, b'H' as u16, b'O' as u16,
    b'M' as u16, b'E' as u16, b'P' as u16, b'A' as u16, b'T' as u16, b'H' as u16, b'%' as u16, 0,
];

/// The shell-link half shared by AddDesktopItemW and AddItemW: create a ShellLink object,
/// describe the target, and persist it to `szLinkPath`. Both C bodies carry this sequence
/// verbatim; the original translation replaced it with `bResult = TRUE`, so both exports
/// reported success while never writing the .lnk.
unsafe fn SaveShellLink(
    Ole32: *mut DYN_FUNCS,
    szLinkPath: LPCWSTR,
    lpItemName: LPCWSTR,
    szCommand: LPCWSTR,
    szArguments: LPCWSTR,
    lpIconLocation: LPCWSTR,
    iIcon: INT,
    lpWorkingDirectory: LPCWSTR,
    wHotKey: WORD,
    iShowCmd: INT,
) -> BOOL {
    let fns = (*Ole32).fn_.__bindgen_anon_1.as_ref();
    // LoadDynamicImports fails unless it resolved every entry, so these are all Some.
    let (Some(CoInitialize), Some(CoCreateInstance), Some(CoUninitialize)) =
        (fns.CoInitialize, fns.CoCreateInstance, fns.CoUninitialize)
    else {
        return 0;
    };
    let mut psl: *mut IShellLinkW = core::ptr::null_mut();
    let mut ppf: *mut IPersistFile = core::ptr::null_mut();
    let mut bResult: BOOL = 0;

    CoInitialize(core::ptr::null_mut());

    let mut hr = CoCreateInstance(
        core::ptr::addr_of!(CLSID_ShellLink),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_IShellLinkW),
        core::ptr::addr_of_mut!(psl) as *mut LPVOID,
    );
    if SUCCEEDED(hr) == 0 {
        CoUninitialize();
        return 0;
    }

    hr = ((*(*psl).lpVtbl).QueryInterface)(
        psl,
        core::ptr::addr_of!(IID_IPersistFile),
        core::ptr::addr_of_mut!(ppf) as *mut LPVOID,
    );
    if SUCCEEDED(hr) != 0 {
        let vt = &*(*psl).lpVtbl;
        (vt.SetDescription)(psl, lpItemName);
        (vt.SetPath)(psl, szCommand);
        (vt.SetArguments)(psl, szArguments);
        (vt.SetIconLocation)(psl, lpIconLocation, iIcon);
        if !lpWorkingDirectory.is_null() {
            (vt.SetWorkingDirectory)(psl, lpWorkingDirectory);
        } else {
            (vt.SetWorkingDirectory)(psl, HOME_DIR_W.as_ptr());
        }
        (vt.SetHotkey)(psl, wHotKey);
        (vt.SetShowCmd)(psl, iShowCmd);

        hr = ((*(*ppf).lpVtbl).Save)(ppf, szLinkPath, TRUE);
        if SUCCEEDED(hr) != 0 {
            bResult = TRUE;
        }

        ((*(*ppf).lpVtbl).Release)(ppf);
    }

    ((*(*psl).lpVtbl).Release)(psl);

    CoUninitialize();

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn AddDesktopItemW(
    bCommonDesktop: BOOL,
    lpItemName: LPCWSTR,
    lpArguments: LPCWSTR,
    lpIconLocation: LPCWSTR,
    iIcon: INT,
    lpWorkingDirectory: LPCWSTR,
    wHotKey: WORD,
    iShowCmd: INT,
) -> BOOL {
    let mut Ole32: DYN_FUNCS = core::mem::zeroed();
    let mut szLinkPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szArguments: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szCommand: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE;
    let mut Ptr: LPWSTR;
    let mut dwLength: DWORD;

    if GetDesktopPath(bCommonDesktop, core::ptr::addr_of_mut!(szLinkPath[0])) == 0 {
        return 0;
    }

    hFind = FindFirstFileW(core::ptr::addr_of_mut!(szLinkPath[0]), core::ptr::addr_of_mut!(FindData));
    if hFind == INVALID_HANDLE_VALUE {
        if CreateDirectoryPath(core::ptr::addr_of_mut!(szLinkPath[0]), core::ptr::null_mut()) == 0 {
            return 0;
        }
    } else {
        FindClose(hFind);
    }

    wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), BACKSLASH_W.as_ptr() as *const WCHAR);
    wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), lpItemName);
    wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), DOT_LNK_W.as_ptr() as *const WCHAR);

    Ptr = wcschr(lpArguments, b' ' as WCHAR);
    if !Ptr.is_null() {
        dwLength = Ptr.offset_from(lpArguments) as DWORD;
        memcpy(
            core::ptr::addr_of_mut!(szCommand[0]) as *mut core::ffi::c_void,
            lpArguments as *const core::ffi::c_void,
            (dwLength as usize) * core::mem::size_of::<WCHAR>(),
        );
        szCommand[dwLength as usize] = 0;
        Ptr = Ptr.offset(1);
        wcscpy(core::ptr::addr_of_mut!(szArguments[0]), Ptr);
    } else {
        wcscpy(core::ptr::addr_of_mut!(szCommand[0]), lpArguments);
        szArguments[0] = 0;
    }

    if LoadDynamicImports(core::ptr::addr_of_mut!(DynOle32), core::ptr::addr_of_mut!(Ole32)) == 0 {
        return 0;
    }

    let bResult = SaveShellLink(
        core::ptr::addr_of_mut!(Ole32),
        szLinkPath.as_ptr(),
        lpItemName,
        szCommand.as_ptr(),
        szArguments.as_ptr(),
        lpIconLocation,
        iIcon,
        lpWorkingDirectory,
        wHotKey,
        iShowCmd,
    );

    UnloadDynamicImports(core::ptr::addr_of_mut!(Ole32));

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn DeleteDesktopItemA(mut bCommonItem: BOOL, mut lpItemName: LPCSTR) -> BOOL {
    let mut ItemName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bResult: BOOL = 0;
    if RtlCreateUnicodeStringFromAsciiz(&mut ItemName, lpItemName as LPSTR) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as u32);
        return 0;
    }
    bResult = DeleteDesktopItemW(bCommonItem, ItemName.Buffer);
    RtlFreeUnicodeString(&mut ItemName);
    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteDesktopItemW(bCommonItem: BOOL, lpItemName: LPCWSTR) -> BOOL {
    let mut szLinkPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if GetDesktopPath(bCommonItem, szLinkPath.as_mut_ptr()) == 0 {
        return 0;
    }

    wcscat(szLinkPath.as_mut_ptr(), BACKSLASH_W.as_ptr() as *const u16);
    wcscat(szLinkPath.as_mut_ptr(), lpItemName);
    wcscat(szLinkPath.as_mut_ptr(), DOT_LNK_W.as_ptr() as *const u16);

    DeleteFileW(szLinkPath.as_ptr())
}

#[no_mangle]
pub unsafe extern "system" fn CreateGroupA(lpGroupName: LPCSTR, bCommonGroup: BOOL) -> BOOL {
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    
    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(GroupName), lpGroupName as LPSTR) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    
    let bResult = CreateGroupW(GroupName.Buffer, bCommonGroup);
    
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(GroupName));
    
    bResult
}

#[no_mangle]
pub unsafe extern "system" fn CreateGroupW(lpGroupName: LPCWSTR, bCommonGroup: BOOL) -> BOOL {
    let mut szGroupPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if lpGroupName.is_null() || *lpGroupName == 0 {
        return 1;
    }

    if GetProgramsPath(bCommonGroup, szGroupPath.as_mut_ptr()) == 0 {
        return 0;
    }

    wcscat(szGroupPath.as_mut_ptr(), win32macros_rust::wide_bytes!(b"\\\0\0\0") as *const u16);
    wcscat(szGroupPath.as_mut_ptr(), lpGroupName);

    if CreateDirectoryPath(szGroupPath.as_ptr(), core::ptr::null_mut()) == 0 {
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn DeleteGroupA(mut lpGroupName: LPCSTR, mut bCommonGroup: BOOL) -> BOOL {
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bResult: BOOL = 0;
    
    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(GroupName), lpGroupName as LPSTR) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    
    bResult = DeleteGroupW(GroupName.Buffer, bCommonGroup);
    
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(GroupName));
    
    return bResult;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteGroupW(mut lpGroupName: LPCWSTR, mut bCommonGroup: BOOL) -> BOOL {
    let mut szGroupPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if lpGroupName.is_null() || *lpGroupName as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return TRUE;
    }

    if GetProgramsPath(bCommonGroup, &raw mut szGroupPath as LPWSTR) == 0 {
        return FALSE;
    }

    wcscat(
        &raw mut szGroupPath as *mut WCHAR,
        BACKSLASH_W.as_ptr() as *const WCHAR,
    );
    wcscat(
        &raw mut szGroupPath as *mut WCHAR,
        lpGroupName as *const WCHAR,
    );

    if RemoveDirectoryPath(&raw mut szGroupPath as *mut WCHAR as LPCWSTR) == 0 {
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn AddItemA(
    lpGroupName: LPCSTR,
    bCommonGroup: BOOL,
    lpItemName: LPCSTR,
    lpArguments: LPCSTR,
    lpIconLocation: LPCSTR,
    iIcon: i32,
    lpWorkingDirectory: LPCSTR,
    wHotKey: u16,
    iShowCmd: i32,
) -> BOOL {
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ItemName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Arguments: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut IconLocation: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut WorkingDirectory: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bResult: BOOL;

    if RtlCreateUnicodeStringFromAsciiz(
        core::ptr::addr_of_mut!(ItemName),
        lpItemName as LPSTR,
    ) == 0
    {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as u32);
        return 0;
    }

    if RtlCreateUnicodeStringFromAsciiz(
        core::ptr::addr_of_mut!(Arguments),
        lpArguments as LPSTR,
    ) == 0
    {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(ItemName));
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as u32);
        return 0;
    }

    if RtlCreateUnicodeStringFromAsciiz(
        core::ptr::addr_of_mut!(IconLocation),
        lpIconLocation as LPSTR,
    ) == 0
    {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(Arguments));
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(ItemName));
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as u32);
        return 0;
    }

    if !lpGroupName.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!(GroupName),
            lpGroupName as LPSTR,
        ) == 0
        {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(IconLocation));
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(Arguments));
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(ItemName));
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as u32);
            return 0;
        }
    }

    if !lpWorkingDirectory.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!(WorkingDirectory),
            lpWorkingDirectory as LPSTR,
        ) == 0
        {
            if !lpGroupName.is_null() {
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(GroupName));
            }
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(IconLocation));
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(Arguments));
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(ItemName));
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as u32);
            return 0;
        }
    }

    bResult = AddItemW(
        if !lpGroupName.is_null() {
            GroupName.Buffer
        } else {
            core::ptr::null_mut()
        },
        bCommonGroup,
        ItemName.Buffer,
        Arguments.Buffer,
        IconLocation.Buffer,
        iIcon,
        if !lpWorkingDirectory.is_null() {
            WorkingDirectory.Buffer
        } else {
            core::ptr::null_mut()
        },
        wHotKey,
        iShowCmd,
    );

    if !lpGroupName.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(GroupName));
    }

    if !lpWorkingDirectory.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(WorkingDirectory));
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(IconLocation));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(Arguments));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ItemName));

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn AddItemW(
    mut lpGroupName: LPCWSTR,
    mut bCommonGroup: BOOL,
    mut lpItemName: LPCWSTR,
    mut lpArguments: LPCWSTR,
    mut lpIconLocation: LPCWSTR,
    mut iIcon: i32,
    mut lpWorkingDirectory: LPCWSTR,
    mut wHotKey: u16,
    mut iShowCmd: i32,
) -> BOOL {
    let mut Ole32: DYN_FUNCS = core::mem::zeroed();
    let mut szLinkPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szArguments: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szCommand: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE;
    let mut Ptr: *mut u16;
    let mut dwLength: u32;
    let mut bResult: BOOL;

    bResult = 0;

    if GetProgramsPath(bCommonGroup, core::ptr::addr_of_mut!(szLinkPath[0])) == 0 {
        return 0;
    }

    if !lpGroupName.is_null() && *lpGroupName != 0 {
        wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), BACKSLASH_W.as_ptr() as *const u16);
        wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), lpGroupName);

        hFind = FindFirstFileW(core::ptr::addr_of_mut!(szLinkPath[0]), core::ptr::addr_of_mut!(FindData));
        if hFind == INVALID_HANDLE_VALUE {
            if CreateGroupW(lpGroupName, bCommonGroup) == 0 {
                return 0;
            }
        } else {
            FindClose(hFind);
        }
    }

    wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), BACKSLASH_W.as_ptr() as *const u16);
    wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), lpItemName);
    wcscat(core::ptr::addr_of_mut!(szLinkPath[0]), DOT_LNK_W.as_ptr());

    Ptr = wcschr(lpArguments, ' ' as u16);
    if !Ptr.is_null() {
        dwLength = ((Ptr as usize - lpArguments as usize) / 2) as u32;
        memcpy(
            core::ptr::addr_of_mut!(szCommand[0]) as *mut core::ffi::c_void,
            lpArguments as *const core::ffi::c_void,
            (dwLength as usize) * 2,
        );
        szCommand[dwLength as usize] = 0;
        Ptr = Ptr.offset(1);
        wcscpy(core::ptr::addr_of_mut!(szArguments[0]), Ptr);
    } else {
        wcscpy(core::ptr::addr_of_mut!(szCommand[0]), lpArguments);
        szArguments[0] = 0;
    }

    if LoadDynamicImports(core::ptr::addr_of_mut!(DynOle32), core::ptr::addr_of_mut!(Ole32)) == 0 {
        return 0;
    }

    bResult = SaveShellLink(
        core::ptr::addr_of_mut!(Ole32),
        szLinkPath.as_ptr(),
        lpItemName,
        szCommand.as_ptr(),
        szArguments.as_ptr(),
        lpIconLocation,
        iIcon,
        lpWorkingDirectory,
        wHotKey,
        iShowCmd,
    );

    UnloadDynamicImports(core::ptr::addr_of_mut!(Ole32));

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn DeleteItemA(
    lpGroupName: LPCSTR,
    bCommonGroup: BOOL,
    lpItemName: LPCSTR,
    bDeleteGroup: BOOL,
) -> BOOL {
    let mut GroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ItemName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bResult: BOOL;

    if !lpGroupName.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut GroupName, lpGroupName as LPSTR) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }

    if RtlCreateUnicodeStringFromAsciiz(&mut ItemName, lpItemName as LPSTR) == 0 {
        if !lpGroupName.is_null() {
            RtlFreeUnicodeString(&mut GroupName);
        }
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    bResult = DeleteItemW(
        if !lpGroupName.is_null() {
            GroupName.Buffer
        } else {
            core::ptr::null_mut()
        },
        bCommonGroup,
        ItemName.Buffer,
        bDeleteGroup,
    );

    RtlFreeUnicodeString(&mut ItemName);
    if !lpGroupName.is_null() {
        RtlFreeUnicodeString(&mut GroupName);
    }

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn DeleteItemW(
    lpGroupName: LPCWSTR,
    bCommonGroup: BOOL,
    lpItemName: LPCWSTR,
    bDeleteGroup: BOOL,
) -> BOOL {
    let mut szItemPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut Ptr: LPWSTR;

    if GetProgramsPath(bCommonGroup, &mut szItemPath as LPWSTR) == 0 {
        return 0;
    }

    if !lpGroupName.is_null() && *lpGroupName != 0 {
        wcscat(&mut szItemPath as LPWSTR, BACKSLASH_W.as_ptr() as LPCWSTR);
        wcscat(&mut szItemPath as LPWSTR, lpGroupName);
    }

    wcscat(&mut szItemPath as LPWSTR, BACKSLASH_W.as_ptr() as LPCWSTR);
    wcscat(&mut szItemPath as LPWSTR, lpItemName);
    wcscat(&mut szItemPath as LPWSTR, win32macros_rust::wide_bytes!(b".\0l\0n\0k\0\0\0") as LPCWSTR);

    if DeleteFileW(&szItemPath as LPCWSTR) == 0 {
        return 0;
    }

    if bDeleteGroup != 0 {
        Ptr = wcsrchr(&szItemPath as LPCWSTR, b'\\' as u16);
        if Ptr.is_null() {
            return 1;
        }

        *Ptr = 0;
        RemoveDirectoryW(&szItemPath as LPCWSTR);
    }

    1
}
// enforce_import_abi: moved 9 import decl(s) into the extern block their
// exporting DLL's .spec requires: ExpandEnvironmentStringsW->system, RegCloseKey->system, RegOpenKeyExW->system, RegQueryValueExW->system, SetLastError->system, memcpy->C, wcscat->C, wcschr->C, wcsrchr->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "C" {
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn wcscat(dest: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    fn wcsrchr(str_: LPCWSTR, ch: u16) -> LPWSTR;
}
extern "system" {
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: u32) -> u32;
    fn SetLastError(dwErrCode: u32);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2004 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/userenv/desktop.c
//  * PURPOSE:         Desktop and start menu support functions.
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "precomp.h"
//
// #include <shlobj.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ***************************************************************/
//
// static
// BOOL
// GetDesktopPath(BOOL bCommonPath,
//                LPWSTR lpDesktopPath)
// {
//     WCHAR szPath[MAX_PATH];
//     DWORD dwLength;
//     DWORD dwType;
//     HKEY hKey;
//     LONG Error;
//
//     DPRINT("GetDesktopPath() called\n");
//
//     Error = RegOpenKeyExW(HKEY_CURRENT_USER,
//                           L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders",
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed\n");
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     dwLength = MAX_PATH * sizeof(WCHAR);
//     Error = RegQueryValueExW(hKey,
//                              bCommonPath ? L"Common Desktop" : L"Desktop",
//                              0,
//                              &dwType,
//                              (LPBYTE)szPath,
//                              &dwLength);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegQueryValueExW() failed\n");
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     if (dwType == REG_EXPAND_SZ)
//     {
//         ExpandEnvironmentStringsW(szPath,
//                                   lpDesktopPath,
//                                   MAX_PATH);
//     }
//     else
//     {
//         wcscpy(lpDesktopPath, szPath);
//     }
//
//     DPRINT("GetDesktopPath() done\n");
//
//     return TRUE;
// }
//
//
// static
// BOOL
// GetProgramsPath(BOOL bCommonPath,
//                 LPWSTR lpProgramsPath)
// {
//     WCHAR szPath[MAX_PATH];
//     DWORD dwLength;
//     DWORD dwType;
//     HKEY hKey;
//     LONG Error;
//
//     DPRINT("GetProgramsPath() called\n");
//
//     Error = RegOpenKeyExW(HKEY_CURRENT_USER,
//                           L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders",
//                           0,
//                           KEY_QUERY_VALUE,
//                           &hKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed\n");
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     dwLength = MAX_PATH * sizeof(WCHAR);
//     Error = RegQueryValueExW(hKey,
//                              bCommonPath ? L"Common Programs" : L"Programs",
//                              0,
//                              &dwType,
//                              (LPBYTE)szPath,
//                              &dwLength);
//     if (Error != ERROR_SUCCESS)
//     {
//         DPRINT1("RegQueryValueExW() failed\n");
//         RegCloseKey(hKey);
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     RegCloseKey(hKey);
//
//     if (dwType == REG_EXPAND_SZ)
//     {
//         ExpandEnvironmentStringsW(szPath,
//                                   lpProgramsPath,
//                                   MAX_PATH);
//     }
//     else
//     {
//         wcscpy(lpProgramsPath,
//                szPath);
//     }
//
//     DPRINT("GetProgramsPath() done\n");
//
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// AddDesktopItemA(BOOL bCommonItem,
//                 LPCSTR lpItemName,
//                 LPCSTR lpArguments,
//                 LPCSTR lpIconLocation,
//                 INT iIcon,
//                 LPCSTR lpWorkingDirectory, /* Optional */
//                 WORD wHotKey,
//                 INT iShowCmd)
// {
//     UNICODE_STRING ItemName;
//     UNICODE_STRING Arguments;
//     UNICODE_STRING IconLocation;
//     UNICODE_STRING WorkingDirectory;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&ItemName,
//                                           (LPSTR)lpItemName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&Arguments,
//                                           (LPSTR)lpArguments))
//     {
//         RtlFreeUnicodeString(&ItemName);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&IconLocation,
//                                           (LPSTR)lpIconLocation))
//     {
//         RtlFreeUnicodeString(&Arguments);
//         RtlFreeUnicodeString(&ItemName);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (lpWorkingDirectory != NULL)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&WorkingDirectory,
//                                               (LPSTR)lpWorkingDirectory))
//         {
//             RtlFreeUnicodeString(&IconLocation);
//             RtlFreeUnicodeString(&Arguments);
//             RtlFreeUnicodeString(&ItemName);
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//
//     bResult = AddDesktopItemW(bCommonItem,
//                               ItemName.Buffer,
//                               Arguments.Buffer,
//                               IconLocation.Buffer,
//                               iIcon,
//                               (lpWorkingDirectory != NULL) ? WorkingDirectory.Buffer : NULL,
//                               wHotKey,
//                               iShowCmd);
//
//     if (lpWorkingDirectory != NULL)
//     {
//         RtlFreeUnicodeString(&WorkingDirectory);
//     }
//
//     RtlFreeUnicodeString(&IconLocation);
//     RtlFreeUnicodeString(&Arguments);
//     RtlFreeUnicodeString(&ItemName);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// AddDesktopItemW(BOOL bCommonDesktop,
//                 LPCWSTR lpItemName,
//                 LPCWSTR lpArguments,
//                 LPCWSTR lpIconLocation,
//                 INT iIcon,
//                 LPCWSTR lpWorkingDirectory,  /* Optional */
//                 WORD wHotKey,
//                 INT iShowCmd)
// {
//     DYN_FUNCS Ole32;
//     WCHAR szLinkPath[MAX_PATH];
//     WCHAR szArguments[MAX_PATH];
//     WCHAR szCommand[MAX_PATH];
//     WIN32_FIND_DATAW FindData;
//     HANDLE hFind;
//     LPWSTR Ptr;
//     DWORD dwLength;
//     IShellLinkW* psl;
//     IPersistFile* ppf;
//     HRESULT hr;
//     BOOL bResult;
//
//     DPRINT("AddDesktopItemW() called\n");
//
//     bResult = FALSE;
//
//     if (!GetDesktopPath(bCommonDesktop, szLinkPath))
//     {
//         DPRINT1("GetDesktopPath() failed\n");
//         return FALSE;
//     }
//     DPRINT("Desktop path: '%S'\n", szLinkPath);
//
//     /* Make sure the path exists */
//     hFind = FindFirstFileW(szLinkPath,
//                            &FindData);
//     if (hFind == INVALID_HANDLE_VALUE)
//     {
//         DPRINT("'%S' does not exist\n", szLinkPath);
//
//         /* Create directory path */
//         if (!CreateDirectoryPath(szLinkPath, NULL))
//             return FALSE;
//     }
//     else
//     {
//         DPRINT("'%S' exists\n", szLinkPath);
//         FindClose(hFind);
//     }
//
//     /* Append backslash, item name and ".lnk" extension */
//     wcscat(szLinkPath, L"\\");
//     wcscat(szLinkPath, lpItemName);
//     wcscat(szLinkPath, L".lnk");
//     DPRINT("Link path: '%S'\n", szLinkPath);
//
//     /* Split 'lpArguments' string into command and arguments */
//     Ptr = wcschr(lpArguments, L' ');
//     DPRINT("Ptr %p  lpArguments %p\n", Ptr, lpArguments);
//     if (Ptr != NULL)
//     {
//         dwLength = (DWORD)(Ptr - lpArguments);
//         DPRINT("dwLength %lu\n", dwLength);
//         memcpy(szCommand, lpArguments, dwLength * sizeof(WCHAR));
//         szCommand[dwLength] = 0;
//         Ptr++;
//         wcscpy(szArguments, Ptr);
//     }
//     else
//     {
//         wcscpy(szCommand, lpArguments);
//         szArguments[0] = 0;
//     }
//     DPRINT("szCommand: '%S'\n", szCommand);
//     DPRINT("szArguments: '%S'\n", szArguments);
//
//     /* Dynamically load ole32.dll */
//     if (!LoadDynamicImports(&DynOle32, &Ole32))
//     {
//         DPRINT1("USERENV: Unable to load OLE32.DLL\n");
//         return FALSE;
//     }
//
//     Ole32.fn.CoInitialize(NULL);
//
//     hr = Ole32.fn.CoCreateInstance(&CLSID_ShellLink,
//                                    NULL,
//                                    CLSCTX_INPROC_SERVER,
//                                    &IID_IShellLinkW,
//                                    (LPVOID*)&psl);
//     if (!SUCCEEDED(hr))
//     {
//         Ole32.fn.CoUninitialize();
//         UnloadDynamicImports(&Ole32);
//         return FALSE;
//     }
//
//     hr = psl->lpVtbl->QueryInterface(psl,
//                                      &IID_IPersistFile,
//                                      (LPVOID*)&ppf);
//     if (SUCCEEDED(hr))
//     {
//         psl->lpVtbl->SetDescription(psl,
//                                     lpItemName);
//
//         psl->lpVtbl->SetPath(psl,
//                              szCommand);
//
//         psl->lpVtbl->SetArguments(psl,
//                                   szArguments);
//
//         psl->lpVtbl->SetIconLocation(psl,
//                                      lpIconLocation,
//                                      iIcon);
//
//         if (lpWorkingDirectory != NULL)
//         {
//             psl->lpVtbl->SetWorkingDirectory(psl,
//                                              lpWorkingDirectory);
//         }
//         else
//         {
//             psl->lpVtbl->SetWorkingDirectory(psl,
//                                              L"%HOMEDRIVE%%HOMEPATH%");
//         }
//
//         psl->lpVtbl->SetHotkey(psl,
//                                wHotKey);
//
//         psl->lpVtbl->SetShowCmd(psl,
//                                 iShowCmd);
//
//         hr = ppf->lpVtbl->Save(ppf,
//                                szLinkPath,
//                                TRUE);
//         if (SUCCEEDED(hr))
//             bResult = TRUE;
//
//         ppf->lpVtbl->Release(ppf);
//     }
//
//     psl->lpVtbl->Release(psl);
//
//     Ole32.fn.CoUninitialize();
//
//     UnloadDynamicImports(&Ole32);
//
//     DPRINT("AddDesktopItemW() done\n");
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// DeleteDesktopItemA(BOOL bCommonItem,
//                    LPCSTR lpItemName)
// {
//     UNICODE_STRING ItemName;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&ItemName,
//                                           (LPSTR)lpItemName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     bResult = DeleteDesktopItemW(bCommonItem,
//                                  ItemName.Buffer);
//
//     RtlFreeUnicodeString(&ItemName);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// DeleteDesktopItemW(BOOL bCommonItem,
//                    LPCWSTR lpItemName)
// {
//     WCHAR szLinkPath[MAX_PATH];
//
//     DPRINT("DeleteDesktopItemW() called\n");
//
//     if (!GetDesktopPath(bCommonItem, szLinkPath))
//     {
//         DPRINT1("GetDesktopPath() failed\n");
//         return FALSE;
//     }
//
//     wcscat(szLinkPath, L"\\");
//     wcscat(szLinkPath, lpItemName);
//     wcscat(szLinkPath, L".lnk");
//     DPRINT("Link path: '%S'\n", szLinkPath);
//
//     return DeleteFileW (szLinkPath);
// }
//
//
// BOOL
// WINAPI
// CreateGroupA(LPCSTR lpGroupName,
//              BOOL bCommonGroup)
// {
//     UNICODE_STRING GroupName;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&GroupName,
//                                           (LPSTR)lpGroupName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     bResult = CreateGroupW(GroupName.Buffer, bCommonGroup);
//
//     RtlFreeUnicodeString(&GroupName);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// CreateGroupW(LPCWSTR lpGroupName,
//              BOOL bCommonGroup)
// {
//     WCHAR szGroupPath[MAX_PATH];
//
//     DPRINT1("CreateGroupW() called\n");
//
//     if (lpGroupName == NULL || *lpGroupName == 0)
//         return TRUE;
//
//     if (!GetProgramsPath(bCommonGroup, szGroupPath))
//     {
//         DPRINT1("GetProgramsPath() failed\n");
//         return FALSE;
//     }
//     DPRINT1("Programs path: '%S'\n", szGroupPath);
//
//     wcscat(szGroupPath, L"\\");
//     wcscat(szGroupPath, lpGroupName);
//     DPRINT1("Group path: '%S'\n", szGroupPath);
//
//     /* Create directory path */
//     if (!CreateDirectoryPath (szGroupPath, NULL))
//         return FALSE;
//
//     /* FIXME: Notify the shell */
//
//     DPRINT1("CreateGroupW() done\n");
//
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// DeleteGroupA(LPCSTR lpGroupName,
//              BOOL bCommonGroup)
// {
//     UNICODE_STRING GroupName;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&GroupName,
//                                           (LPSTR)lpGroupName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     bResult = DeleteGroupW(GroupName.Buffer, bCommonGroup);
//
//     RtlFreeUnicodeString(&GroupName);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// DeleteGroupW(LPCWSTR lpGroupName,
//              BOOL bCommonGroup)
// {
//     WCHAR szGroupPath[MAX_PATH];
//
//     DPRINT("DeleteGroupW() called\n");
//
//     if (lpGroupName == NULL || *lpGroupName == 0)
//         return TRUE;
//
//     if (!GetProgramsPath(bCommonGroup, szGroupPath))
//     {
//         DPRINT1("GetProgramsPath() failed\n");
//         return FALSE;
//     }
//     DPRINT("Programs path: '%S'\n", szGroupPath);
//
//     wcscat(szGroupPath, L"\\");
//     wcscat(szGroupPath, lpGroupName);
//     DPRINT("Group path: '%S'\n", szGroupPath);
//
//     /* Remove directory path */
//     if (!RemoveDirectoryPath (szGroupPath))
//         return FALSE;
//
//     /* FIXME: Notify the shell */
//
//     DPRINT("DeleteGroupW() done\n");
//
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// AddItemA(LPCSTR lpGroupName,  /* Optional */
//          BOOL bCommonGroup,
//          LPCSTR lpItemName,
//          LPCSTR lpArguments,
//          LPCSTR lpIconLocation,
//          INT iIcon,
//          LPCSTR lpWorkingDirectory,  /* Optional */
//          WORD wHotKey,
//          INT iShowCmd)
// {
//     UNICODE_STRING GroupName;
//     UNICODE_STRING ItemName;
//     UNICODE_STRING Arguments;
//     UNICODE_STRING IconLocation;
//     UNICODE_STRING WorkingDirectory;
//     BOOL bResult;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&ItemName,
//                                           (LPSTR)lpItemName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&Arguments,
//                                           (LPSTR)lpArguments))
//     {
//         RtlFreeUnicodeString(&ItemName);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&IconLocation,
//                                           (LPSTR)lpIconLocation))
//     {
//         RtlFreeUnicodeString(&Arguments);
//         RtlFreeUnicodeString(&ItemName);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (lpGroupName != NULL)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&GroupName,
//                                               (LPSTR)lpGroupName))
//         {
//             RtlFreeUnicodeString(&IconLocation);
//             RtlFreeUnicodeString(&Arguments);
//             RtlFreeUnicodeString(&ItemName);
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//
//     if (lpWorkingDirectory != NULL)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&WorkingDirectory,
//                                               (LPSTR)lpWorkingDirectory))
//         {
//             if (lpGroupName != NULL)
//             {
//                 RtlFreeUnicodeString(&GroupName);
//             }
//             RtlFreeUnicodeString(&IconLocation);
//             RtlFreeUnicodeString(&Arguments);
//             RtlFreeUnicodeString(&ItemName);
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//
//     bResult = AddItemW((lpGroupName != NULL) ? GroupName.Buffer : NULL,
//                        bCommonGroup,
//                        ItemName.Buffer,
//                        Arguments.Buffer,
//                        IconLocation.Buffer,
//                        iIcon,
//                        (lpWorkingDirectory != NULL) ? WorkingDirectory.Buffer : NULL,
//                        wHotKey,
//                        iShowCmd);
//
//     if (lpGroupName != NULL)
//     {
//         RtlFreeUnicodeString(&GroupName);
//     }
//
//     if (lpWorkingDirectory != NULL)
//     {
//         RtlFreeUnicodeString(&WorkingDirectory);
//     }
//
//     RtlFreeUnicodeString(&IconLocation);
//     RtlFreeUnicodeString(&Arguments);
//     RtlFreeUnicodeString(&ItemName);
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// AddItemW(LPCWSTR lpGroupName,  /* Optional */
//          BOOL bCommonGroup,
//          LPCWSTR lpItemName,
//          LPCWSTR lpArguments,
//          LPCWSTR lpIconLocation,
//          INT iIcon,
//          LPCWSTR lpWorkingDirectory,  /* Optional */
//          WORD wHotKey,
//          INT iShowCmd)
// {
//     DYN_FUNCS Ole32;
//     WCHAR szLinkPath[MAX_PATH];
//     WCHAR szArguments[MAX_PATH];
//     WCHAR szCommand[MAX_PATH];
//     WIN32_FIND_DATAW FindData;
//     HANDLE hFind;
//     LPWSTR Ptr;
//     DWORD dwLength;
//     IShellLinkW* psl;
//     IPersistFile* ppf;
//     HRESULT hr;
//     BOOL bResult;
//
//     DPRINT("AddItemW() called\n");
//
//     bResult = FALSE;
//
//     if (!GetProgramsPath(bCommonGroup, szLinkPath))
//     {
//         DPRINT1("GetProgramsPath() failed\n");
//         return FALSE;
//     }
//
//     DPRINT("Programs path: '%S'\n", szLinkPath);
//
//     if (lpGroupName != NULL && *lpGroupName != 0)
//     {
//         wcscat(szLinkPath, L"\\");
//         wcscat(szLinkPath, lpGroupName);
//
//         /* Make sure the path exists */
//         hFind = FindFirstFileW(szLinkPath,
//                                &FindData);
//         if (hFind == INVALID_HANDLE_VALUE)
//         {
//             DPRINT("'%S' does not exist\n", szLinkPath);
//             if (!CreateGroupW(lpGroupName,
//                               bCommonGroup))
//                 return FALSE;
//         }
//         else
//         {
//             DPRINT("'%S' exists\n", szLinkPath);
//             FindClose(hFind);
//         }
//     }
//
//     wcscat(szLinkPath, L"\\");
//     wcscat(szLinkPath, lpItemName);
//     wcscat(szLinkPath, L".lnk");
//     DPRINT("Link path: '%S'\n", szLinkPath);
//
//     /* Split 'lpArguments' string into command and arguments */
//     Ptr = wcschr(lpArguments, L' ');
//     DPRINT("Ptr %p  lpArguments %p\n", Ptr, lpArguments);
//     if (Ptr != NULL)
//     {
//         dwLength = (DWORD)(Ptr - lpArguments);
//         DPRINT("dwLength %lu\n", dwLength);
//         memcpy(szCommand, lpArguments, dwLength * sizeof(WCHAR));
//         szCommand[dwLength] = 0;
//         Ptr++;
//         wcscpy(szArguments, Ptr);
//     }
//     else
//     {
//         wcscpy(szCommand, lpArguments);
//         szArguments[0] = 0;
//     }
//     DPRINT("szCommand: '%S'\n", szCommand);
//     DPRINT("szArguments: '%S'\n", szArguments);
//
//     /* Dynamically load ole32.dll */
//     if (!LoadDynamicImports(&DynOle32, &Ole32))
//     {
//         DPRINT1("USERENV: Unable to load OLE32.DLL\n");
//         return FALSE;
//     }
//
//     Ole32.fn.CoInitialize(NULL);
//
//     hr = Ole32.fn.CoCreateInstance(&CLSID_ShellLink,
//                                    NULL,
//                                    CLSCTX_INPROC_SERVER,
//                                    &IID_IShellLinkW,
//                                    (LPVOID*)&psl);
//     if (!SUCCEEDED(hr))
//     {
//         Ole32.fn.CoUninitialize();
//         UnloadDynamicImports(&Ole32);
//         return FALSE;
//     }
//
//     hr = psl->lpVtbl->QueryInterface(psl,
//                                      &IID_IPersistFile,
//                                      (LPVOID*)&ppf);
//     if (SUCCEEDED(hr))
//     {
//         psl->lpVtbl->SetDescription(psl,
//                                     lpItemName);
//
//         psl->lpVtbl->SetPath(psl,
//                              szCommand);
//
//         psl->lpVtbl->SetArguments(psl,
//                                   szArguments);
//
//         psl->lpVtbl->SetIconLocation(psl,
//                                      lpIconLocation,
//                                      iIcon);
//
//         if (lpWorkingDirectory != NULL)
//         {
//             psl->lpVtbl->SetWorkingDirectory(psl,
//                                              lpWorkingDirectory);
//         }
//         else
//         {
//             psl->lpVtbl->SetWorkingDirectory(psl,
//                                              L"%HOMEDRIVE%%HOMEPATH%");
//         }
//
//         psl->lpVtbl->SetHotkey(psl,
//                                wHotKey);
//
//         psl->lpVtbl->SetShowCmd(psl,
//                                 iShowCmd);
//
//         hr = ppf->lpVtbl->Save(ppf,
//                                szLinkPath,
//                                TRUE);
//         if (SUCCEEDED(hr))
//             bResult = TRUE;
//
//         ppf->lpVtbl->Release(ppf);
//     }
//
//     psl->lpVtbl->Release(psl);
//
//     Ole32.fn.CoUninitialize();
//     UnloadDynamicImports(&Ole32);
//
//     DPRINT("AddItemW() done\n");
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// DeleteItemA(LPCSTR lpGroupName, /* Optional */
//             BOOL bCommonGroup,
//             LPCSTR lpItemName,
//             BOOL bDeleteGroup)
// {
//     UNICODE_STRING GroupName;
//     UNICODE_STRING ItemName;
//     BOOL bResult;
//
//     if (lpGroupName != NULL)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&GroupName,
//                                               (LPSTR)lpGroupName))
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&ItemName,
//                                           (LPSTR)lpItemName))
//     {
//         if (lpGroupName != NULL)
//         {
//             RtlFreeUnicodeString(&GroupName);
//         }
//
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     bResult = DeleteItemW((lpGroupName != NULL) ? GroupName.Buffer : NULL,
//                           bCommonGroup,
//                           ItemName.Buffer,
//                           bDeleteGroup);
//
//     RtlFreeUnicodeString(&ItemName);
//     if (lpGroupName != NULL)
//     {
//         RtlFreeUnicodeString(&GroupName);
//     }
//
//     return bResult;
// }
//
//
// BOOL
// WINAPI
// DeleteItemW(LPCWSTR lpGroupName, /* Optional */
//             BOOL bCommonGroup,
//             LPCWSTR lpItemName,
//             BOOL bDeleteGroup)
// {
//     WCHAR szItemPath[MAX_PATH];
//     LPWSTR Ptr;
//
//     DPRINT("DeleteItemW() called\n");
//
//     if (!GetProgramsPath(bCommonGroup, szItemPath))
//     {
//         DPRINT1("GetProgramsPath() failed\n");
//         return FALSE;
//     }
//     DPRINT("Programs path: '%S'\n", szItemPath);
//
//     if (lpGroupName != NULL && *lpGroupName != 0)
//     {
//         wcscat(szItemPath, L"\\");
//         wcscat(szItemPath, lpGroupName);
//     }
//
//     wcscat(szItemPath, L"\\");
//     wcscat(szItemPath, lpItemName);
//     wcscat(szItemPath, L".lnk");
//     DPRINT("Item path: '%S'\n", szItemPath);
//
//     if (!DeleteFileW(szItemPath))
//         return FALSE;
//
//     /* FIXME: Notify the shell */
//
//     if (bDeleteGroup)
//     {
//         Ptr = wcsrchr(szItemPath, L'\\');
//         if (Ptr == NULL)
//             return TRUE;
//
//         *Ptr = 0;
//         DPRINT("Item path: '%S'\n", szItemPath);
//         if (RemoveDirectoryW(szItemPath))
//         {
//             /* FIXME: Notify the shell */
//         }
//     }
//
//     DPRINT("DeleteItemW() done\n");
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

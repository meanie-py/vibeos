use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use crate::resources::*;
use crate::msgina_consts::*;
use crate::strsafe::*;
use crate::lsa::{ConnectToLsa, MyLogonUser};
use crate::utils::{DuplicateString, ReadRegDwordValue, ReadRegSzValue, RegOpenLoggedOnHKCU};
use crate::gui::GinaGraphicalUI;
use crate::tui::GinaTextUI;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// WINAPI = stdcall on i386 → extern "system" (NOT extern "C"/cdecl: a cdecl decl
// references the undecorated `_LoadLibraryW` and fails to link vs the importlib's
// stdcall `_LoadLibraryW@4`).
extern "system" {
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    fn GetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_msgina: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"msgina\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    &raw const __wine_dbch_msgina as *mut __wine_debug_channel;

// #[no_mangle]: the C dimmedwindow.cpp references `extern HINSTANCE hDllInstance`,
// so this global must export the undecorated `_hDllInstance` data symbol.
#[no_mangle]
pub static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut pGinaUI: PGINA_UI = ::core::ptr::null::<_GINA_UI>() as *mut _GINA_UI;

static mut SystemAuthority: SID_IDENTIFIER_AUTHORITY = _SID_IDENTIFIER_AUTHORITY {
    Value: [
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        5 as ::core::ffi::c_int as BYTE,
    ],
};

static mut AdminSid: PSID = ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub type pThemeWait = Option<unsafe extern "system" fn(dwTimeout: DWORD) -> DWORD>;
pub type pThemeWatch = Option<unsafe extern "system" fn() -> BOOL>;

#[no_mangle]
pub unsafe extern "system" fn WlxNegotiate(
    dwWinlogonVersion: DWORD,
    pdwDllVersion: PDWORD,
) -> BOOL {
    TRACE!(
        b"WlxNegotiate(%lx, %p)\n\0" as *const u8 as *const i8,
        dwWinlogonVersion,
        pdwDllVersion,
    );
    if pdwDllVersion.is_null() || dwWinlogonVersion < WLX_VERSION_1_3 {
        return 0i32;
    }
    *pdwDllVersion = WLX_VERSION_1_3;
    1i32
}

unsafe fn ChooseGinaUI() {
    let mut ControlKey: HKEY = 0 as HKEY;
    let mut SystemStartOptions: *mut u16 = core::ptr::null_mut();
    let mut CurrentOption: *mut u16;
    let mut NextOption: *mut u16;
    let mut ConsoleBoot: u8 = 0;
    let mut rc: i32;

    rc = RegOpenKeyExW(
        0x80000002u32 as HKEY, // HKEY_LOCAL_MACHINE
        // was a phase-broken interleaved literal (`\\0C` = '\', '0', 'C') that
        // decoded as "SYSTE M" + CJK garbage — RegOpenKeyExW always failed
        w!("SYSTEM\\CurrentControlSet\\Control").as_ptr(),
        0u32,
        KEY_QUERY_VALUE,
        &raw mut ControlKey,
    ) as i32;

    rc = ReadRegSzValue(
        ControlKey,
        w!("SystemStartOptions").as_ptr() as *const u16,
        &raw mut SystemStartOptions,
    ) as i32;

    if rc == ERROR_SUCCESS as i32 {
        CurrentOption = SystemStartOptions;
        while !CurrentOption.is_null() {
            NextOption = wcschr(CurrentOption, b' ' as u16);
            if !NextOption.is_null() {
                *NextOption = 0;
            }
            if _wcsicmp(CurrentOption, w!("CONSOLE").as_ptr() as *const u16) == 0 {
                TRACE!(
                    b"Found %S. Switching to console boot\n\0".as_ptr() as *const i8,
                    CurrentOption,
                );
                ConsoleBoot = 1;
                break;
            }
            CurrentOption = if !NextOption.is_null() {
                NextOption.offset(1)
            } else {
                core::ptr::null_mut()
            };
        }
    }

    if ConsoleBoot != 0 {
        pGinaUI = &raw mut GinaTextUI as *mut GINA_UI;
    } else {
        pGinaUI = &raw mut GinaGraphicalUI as *mut GINA_UI;
    }

    if !ControlKey.is_null() {
        RegCloseKey(ControlKey);
    }
    HeapFree(
        GetProcessHeap(),
        0u32,
        SystemStartOptions as *mut core::ffi::c_void,
    );
}

unsafe fn SafeGetUnicodeString(
    pInput: *const LSA_UNICODE_STRING,
    pszOutput: PWSTR,
    cchMax: usize,
) -> BOOL {
    let hr: HRESULT = StringCbCopyNExW(
        pszOutput,
        cchMax * core::mem::size_of::<u16>(),
        (*pInput).Buffer,
        (*pInput).Length as usize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        STRSAFE_NO_TRUNCATION | STRSAFE_NULL_ON_FAILURE,
    );
    (hr == S_OK) as BOOL
}

unsafe fn GetLsaDefaultPassword(pgContext: PGINA_CONTEXT) -> BOOL {
    let mut hPolicy: LSA_HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Name: LSA_UNICODE_STRING = LSA_UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut pPwd: *mut LSA_UNICODE_STRING = core::ptr::null_mut::<LSA_UNICODE_STRING>();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
        ObjectName: core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut Status: NTSTATUS = LsaOpenPolicy(
        core::ptr::null_mut::<_UNICODE_STRING>(),
        &raw mut ObjectAttributes as *mut core::ffi::c_void,
        POLICY_GET_PRIVATE_INFORMATION,
        &raw mut hPolicy,
    );
    if !NT_SUCCESS(Status) {
        return 0i32;
    }
    RtlInitUnicodeString(
        &raw mut Name,
        { static __WS1: [u16; 16] = unsafe { core::mem::transmute::<[u8; 32], [u16; 16]>(*b"D\0e\0f\0a\0u\0l\0t\0P\0a\0s\0s\0w\0o\0r\0d\0\0\0",) }; __WS1.as_ptr() as PCWSTR },
    );
    Status = LsaRetrievePrivateData(hPolicy, &raw mut Name, &raw mut pPwd);
    LsaClose(hPolicy);
    if Status == STATUS_SUCCESS {
        if SafeGetUnicodeString(
            pPwd,
            (*pgContext).Password.as_mut_ptr() as PWSTR,
            256usize,
        ) == 0i32
        {
            Status = STATUS_BUFFER_TOO_SMALL;
        }
        RtlSecureZeroMemory((*pPwd).Buffer as *mut core::ffi::c_void, ((*pPwd).Length as usize).try_into().unwrap());
        LsaFreeMemory(pPwd as *mut core::ffi::c_void);
    }
    (Status == STATUS_SUCCESS) as BOOL
}

unsafe fn GetRegistrySettings(pgContext: PGINA_CONTEXT) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwValue: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwDisableCAD: DWORD = 0;
    let mut rc: LONG = 0;

    rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        core::mem::transmute::<[u8; 108], [core::ffi::c_ushort; 54]>(
            *b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0W\0i\0n\0l\0o\0g\0o\0n\0\0\0",
        ).as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE as REGSAM,
        &raw mut hKey,
    ) as LONG;
    if rc != ERROR_SUCCESS as i32 {
        if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_WARN as i32) != 0 {
            ros_dbg_log(
                __WINE_DBCL_WARN,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                b"GetRegistrySettings\0".as_ptr() as *const i8,
                167,
                b"RegOpenKeyExW() failed with error %lu\n\0".as_ptr() as *const i8,
                rc,
            );
        }
        return 0i32;
    }

    rc = ReadRegDwordValue(
        hKey,
        core::mem::transmute::<[u8; 30], [core::ffi::c_ushort; 15]>(
            *b"A\0u\0t\0o\0A\0d\0m\0i\0n\0L\0o\0g\0o\0n\0\0\0",
        ).as_ptr() as PCWSTR,
        &raw mut dwValue,
    ) as i32;
    if rc == ERROR_SUCCESS as i32 {
        (*pgContext).bAutoAdminLogon = (dwValue != 0) as BOOL;
    }
    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
            b"GetRegistrySettings\0".as_ptr() as *const i8,
            174,
            b"bAutoAdminLogon: %s\n\0".as_ptr() as *const i8,
            if (*pgContext).bAutoAdminLogon != 0 {
                b"TRUE\0".as_ptr() as *const i8
            } else {
                b"FALSE\0".as_ptr() as *const i8
            },
        );
    }

    rc = ReadRegDwordValue(
        hKey,
        core::mem::transmute::<[u8; 22], [core::ffi::c_ushort; 11]>(
            *b"D\0i\0s\0a\0b\0l\0e\0C\0A\0D\0\0\0",
        ).as_ptr() as PCWSTR,
        &raw mut dwDisableCAD,
    ) as i32;
    if rc == ERROR_SUCCESS as i32 {
        if dwDisableCAD != 0 {
            (*pgContext).bDisableCAD = 1;
        }
    }
    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
            b"GetRegistrySettings\0".as_ptr() as *const i8,
            183,
            b"bDisableCAD: %s\n\0".as_ptr() as *const i8,
            if (*pgContext).bDisableCAD != 0 {
                b"TRUE\0".as_ptr() as *const i8
            } else {
                b"FALSE\0".as_ptr() as *const i8
            },
        );
    }

    (*pgContext).bShutdownWithoutLogon = 1;
    rc = ReadRegDwordValue(
        hKey,
        core::mem::transmute::<[u8; 42], [core::ffi::c_ushort; 21]>(
            *b"S\0h\0u\0t\0d\0o\0w\0n\0W\0i\0t\0h\0o\0u\0t\0L\0o\0g\0o\0n\0\0\0",
        ).as_ptr() as PCWSTR,
        &raw mut dwValue,
    ) as i32;
    if rc == ERROR_SUCCESS as i32 {
        (*pgContext).bShutdownWithoutLogon = (dwValue != 0) as BOOL;
    }

    rc = ReadRegDwordValue(
        hKey,
        core::mem::transmute::<[u8; 48], [core::ffi::c_ushort; 24]>(
            *b"D\0o\0n\0t\0D\0i\0s\0p\0l\0a\0y\0L\0a\0s\0t\0U\0s\0e\0r\0N\0a\0m\0e\0\0\0",
        ).as_ptr() as PCWSTR,
        &raw mut dwValue,
    ) as i32;
    if rc == ERROR_SUCCESS as i32 {
        (*pgContext).bDontDisplayLastUserName = (dwValue != 0) as BOOL;
    }

    rc = ReadRegDwordValue(
        hKey,
        core::mem::transmute::<[u8; 40], [core::ffi::c_ushort; 20]>(
            *b"I\0g\0n\0o\0r\0e\0S\0h\0i\0f\0t\0O\0v\0e\0r\0r\0i\0d\0e\0\0\0",
        ).as_ptr() as PCWSTR,
        &raw mut dwValue,
    ) as i32;
    if rc == ERROR_SUCCESS as i32 {
        (*pgContext).bIgnoreShiftOverride = (dwValue != 0) as BOOL;
    }

    dwSize = core::mem::size_of::<[u16; 256]>() as DWORD;
    rc = RegQueryValueExW(
        hKey,
        core::mem::transmute::<[u8; 32], [core::ffi::c_ushort; 16]>(
            *b"D\0e\0f\0a\0u\0l\0t\0U\0s\0e\0r\0N\0a\0m\0e\0\0\0",
        ).as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut (*pgContext).UserName as LPBYTE,
        &raw mut dwSize,
    ) as LONG;

    dwSize = core::mem::size_of::<[u16; 256]>() as DWORD;
    rc = RegQueryValueExW(
        hKey,
        core::mem::transmute::<[u8; 36], [core::ffi::c_ushort; 18]>(
            *b"D\0e\0f\0a\0u\0l\0t\0D\0o\0m\0a\0i\0n\0N\0a\0m\0e\0\0\0",
        ).as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut (*pgContext).DomainName as LPBYTE,
        &raw mut dwSize,
    ) as LONG;

    dwSize = core::mem::size_of::<[u16; 256]>() as DWORD;
    rc = RegQueryValueExW(
        hKey,
        core::mem::transmute::<[u8; 32], [core::ffi::c_ushort; 16]>(
            *b"D\0e\0f\0a\0u\0l\0t\0P\0a\0s\0s\0w\0o\0r\0d\0\0\0",
        ).as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut (*pgContext).Password as LPBYTE,
        &raw mut dwSize,
    ) as LONG;
    if rc != 0 {
        GetLsaDefaultPassword(pgContext);
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    return 1i32;
}

unsafe fn InitThemeSupport() {

    const DLL_NAME: [u16; 11] = [
        0x0073, 0x0068, 0x0073, 0x0076, 0x0063, 0x0073, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000,
    ]; // "shsvcs.dll"

    let hDll = LoadLibraryW(DLL_NAME.as_ptr());
    if hDll.is_null() {
        return;
    }

    let mut theme_wait: pThemeWait = None;
    let mut theme_watch: pThemeWatch = None;

    theme_wait = core::mem::transmute::<FARPROC, pThemeWait>(
        GetProcAddress(hDll as HINSTANCE, 2 as LPCSTR),
    );
    theme_watch = core::mem::transmute::<FARPROC, pThemeWatch>(
        GetProcAddress(hDll as HINSTANCE, 1 as LPCSTR),
    );

    if theme_wait.is_some() && theme_watch.is_some() {
        (theme_wait.unwrap())(5000);
        (theme_watch.unwrap())();
    }
}
#[no_mangle]
pub unsafe extern "system" fn WlxInitialize(
    lpWinsta: LPWSTR,
    hWlx: HANDLE,
    pvReserved: PVOID,
    pWinlogonFunctions: PVOID,
    pWlxContext: *mut PVOID,
) -> BOOL {
    let _ = pvReserved;

    InitThemeSupport();

    let pgContext: PGINA_CONTEXT = LocalAlloc(
        (LMEM_FIXED | LMEM_ZEROINIT) as UINT,
        core::mem::size_of::<GINA_CONTEXT>() as SIZE_T,
    ) as PGINA_CONTEXT;

    if pgContext.is_null() {
        if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_WARN as i32) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/msgina/msgina.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"WlxInitialize\0" as *const u8 as *const core::ffi::c_char,
                275i32,
                b"LocalAlloc() failed\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return 0i32;
    }

    if GetRegistrySettings(pgContext) == 0 {
        if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_WARN as i32) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/msgina/msgina.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"WlxInitialize\0" as *const u8 as *const core::ffi::c_char,
                281i32,
                b"GetRegistrySettings() failed\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        LocalFree(pgContext as HLOCAL);
        return 0i32;
    }

    *pWlxContext = pgContext as PVOID;
    (*pgContext).hDllInstance = hDllInstance;
    (*pgContext).pWlxFuncs = pWinlogonFunctions as PWLX_DISPATCH_VERSION_1_3;
    (*pgContext).hWlx = hWlx;
    (*pgContext).station = lpWinsta;
    (*pgContext).hStatusWindow = core::ptr::null_mut::<HWND__>();
    (*(*pgContext).pWlxFuncs)
        .WlxUseCtrlAltDel
        .expect("non-null function pointer")(hWlx);
    (*pgContext).nShutdownAction = WLX_SAS_ACTION_SHUTDOWN_POWER_OFF as ULONG;
    ChooseGinaUI();
    return (*pGinaUI).Initialize.expect("non-null function pointer")(pgContext);
}

#[no_mangle]
pub unsafe extern "system" fn WlxScreenSaverNotify(
    pWlxContext: LPVOID,
    pSecure: *mut i32,
) -> i32 {
    *pSecure = 0i32;
    1i32
}

#[no_mangle]
pub unsafe extern "system" fn WlxStartApplication(
    pWlxContext: PVOID,
    pszDesktopName: PWSTR,
    pEnvironment: PVOID,
    pszCmdLine: PWSTR,
) -> BOOL {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;

    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(StartupInfo).cast::<::core::ffi::c_void>(),
        0,
        core::mem::size_of::<STARTUPINFOW>(),
    );

    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(ProcessInformation).cast::<::core::ffi::c_void>(),
        0,
        core::mem::size_of::<PROCESS_INFORMATION>(),
    );

    let mut CurrentDirectory: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hAppToken: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut len: UINT;
    let mut ret: BOOL;

    len = GetWindowsDirectoryW(&raw mut CurrentDirectory as LPWSTR, MAX_PATH as UINT);
    if len == 0 || len > MAX_PATH as UINT {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"WlxStartApplication\0".as_ptr() as *const ::core::ffi::c_char,
                417 as ::core::ffi::c_int,
                b"GetWindowsDirectoryW() failed\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    ret = DuplicateTokenEx(
        (*pgContext).UserToken,
        MAXIMUM_ALLOWED as DWORD,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>() as *mut core::ffi::c_void,
        SecurityImpersonation,
        TokenPrimary,
        &raw mut hAppToken,
    );
    if ret == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"WlxStartApplication\0".as_ptr() as *const ::core::ffi::c_char,
                424 as ::core::ffi::c_int,
                b"DuplicateTokenEx() failed with error %lu\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                GetLastError(),
            );
        }
        return 0;
    }

    // ZeroMemory already done above

    StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
    StartupInfo.lpTitle = pszCmdLine as LPWSTR;
    StartupInfo.dwFlags = STARTF_USESHOWWINDOW;
    StartupInfo.wShowWindow = SW_SHOW as u16;
    StartupInfo.lpDesktop = pszDesktopName as LPWSTR;

    len = GetWindowsDirectoryW(&raw mut CurrentDirectory as LPWSTR, MAX_PATH as UINT);
    if len == 0 || len > MAX_PATH as UINT {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"WlxStartApplication\0".as_ptr() as *const ::core::ffi::c_char,
                439 as ::core::ffi::c_int,
                b"GetWindowsDirectoryW() failed\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    ret = CreateProcessAsUserW(
        hAppToken,
        pszCmdLine as LPCWSTR,
        ::core::ptr::null_mut::<WCHAR>(),
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>() as *mut core::ffi::c_void,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>() as *mut core::ffi::c_void,
        0, // FALSE
        CREATE_UNICODE_ENVIRONMENT,
        pEnvironment,
        &raw mut CurrentDirectory as *mut WCHAR as LPCWSTR,
        &raw mut StartupInfo as *mut core::ffi::c_void,
        &raw mut ProcessInformation as *mut core::ffi::c_void,
    );

    CloseHandle(ProcessInformation.hProcess);
    CloseHandle(ProcessInformation.hThread);
    CloseHandle(hAppToken);

    if ret == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"WlxStartApplication\0".as_ptr() as *const ::core::ffi::c_char,
                458 as ::core::ffi::c_int,
                b"CreateProcessAsUserW() failed with error %lu\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                GetLastError(),
            );
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn WlxActivateUserShell(
    pWlxContext: PVOID,
    pszDesktopName: PWSTR,
    pszMprLogonScript: PWSTR,
    pEnvironment: PVOID,
) -> BOOL {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;
    let mut hKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut hKeyCurrentUser: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut BufSize: DWORD;
    let mut ValueType: DWORD = 0;
    let mut len: DWORD;
    let mut rc: LONG;
    let mut ret: BOOL;
    let mut pszUserInitApp: [WCHAR; (MAX_PATH as usize) + 1] = [0; (MAX_PATH as usize) + 1];
    let mut pszExpUserInitApp: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let _ = pszMprLogonScript; // UNREFERENCED_PARAMETER

    TRACE!(b"WlxActivateUserShell()\n\0".as_ptr() as *const core::ffi::c_char);

    rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        { static __WS3: [WCHAR; 54] = unsafe { core::mem::transmute::<[u8; 108], [WCHAR; 54]>(*b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0W\0i\0n\0l\0o\0g\0o\0n\0\0\0") }; __WS3.as_ptr() as LPCWSTR },
        0u32,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as LONG;
    if rc != ERROR_SUCCESS as LONG {
        WARN!(b"RegOpenKeyExW() failed with error %lu\n\0".as_ptr() as *const core::ffi::c_char, rc as core::ffi::c_ulong);
        return FALSE.try_into().unwrap();
    }

    BufSize = (core::mem::size_of::<[WCHAR; (MAX_PATH as usize) + 1]>() - core::mem::size_of::<WCHAR>()) as DWORD;
    rc = RegQueryValueExW(
        hKey,
        { static __WS4: [WCHAR; 9] = unsafe { core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"U\0s\0e\0r\0i\0n\0i\0t\0\0\0") }; __WS4.as_ptr() as LPCWSTR },
        core::ptr::null_mut::<DWORD>(),
        &mut ValueType,
        pszUserInitApp.as_mut_ptr() as *mut u8,
        &mut BufSize,
    ) as LONG;
    RegCloseKey(hKey);
    if rc != ERROR_SUCCESS as LONG || (ValueType != REG_SZ && ValueType != REG_EXPAND_SZ) {
        WARN!(b"RegQueryValueExW() failed with error %lu\n\0".as_ptr() as *const core::ffi::c_char, rc as core::ffi::c_ulong);
        return FALSE.try_into().unwrap();
    }
    pszUserInitApp[MAX_PATH as usize] = 0; // UNICODE_NULL

    let dest_len = pszExpUserInitApp.len() as DWORD;
    len = ExpandEnvironmentStringsW(
        pszUserInitApp.as_ptr() as LPCWSTR,
        pszExpUserInitApp.as_mut_ptr() as LPWSTR,
        dest_len,
    );
    if len > dest_len {
        WARN!(b"ExpandEnvironmentStringsW() failed. Required size %lu\n\0".as_ptr() as *const core::ffi::c_char, len as core::ffi::c_ulong);
        return FALSE.try_into().unwrap();
    }

    ret = WlxStartApplication(
        pWlxContext,
        pszDesktopName,
        pEnvironment,
        pszExpUserInitApp.as_mut_ptr() as PWSTR,
    );
    if ret == 0 {
        return ret;
    }

    rc = RegOpenLoggedOnHKCU(
        (*pgContext).UserToken,
        KEY_SET_VALUE as REGSAM,
        &mut hKeyCurrentUser,
    ) as i32;
    if rc != ERROR_SUCCESS as LONG {
        ERR!(b"RegOpenLoggedOnHKCU() failed with error %ld\n\0".as_ptr() as *const core::ffi::c_char, rc);
        return ret;
    }

    rc = RegOpenKeyExW(
        hKeyCurrentUser,
        { static __WS5: [WCHAR; 51] = unsafe { core::mem::transmute::<[u8; 102], [WCHAR; 51]>(*b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0E\0x\0p\0l\0o\0r\0e\0r\0\0\0") }; __WS5.as_ptr() as LPCWSTR },
        0u32,
        KEY_SET_VALUE,
        &mut hKey,
    ) as LONG;
    if rc == ERROR_SUCCESS as LONG {
        len = wcslen((*pgContext).UserName.as_ptr()) as DWORD + 1;
        RegSetValueExW(
            hKey,
            { static __WS6: [WCHAR; 16] = unsafe { core::mem::transmute::<[u8; 32], [WCHAR; 16]>(*b"L\0o\0g\0o\0n\0 \0U\0s\0e\0r\0 \0N\0a\0m\0e\0\0\0") }; __WS6.as_ptr() as LPCWSTR },
            0u32,
            REG_SZ,
            (*pgContext).UserName.as_ptr() as *const u8,
            len.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
        ) as i32;
        RegCloseKey(hKey);
    }
    RegCloseKey(hKeyCurrentUser);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn WlxLoggedOnSAS(
    pWlxContext: PVOID,
    dwSasType: DWORD,
    pReserved: PVOID,
) -> INT {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;
    let mut SasAction: INT = WLX_SAS_ACTION_NONE.try_into().unwrap();

    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
            b"WlxLoggedOnSAS\0".as_ptr() as *const i8,
            573 as i32,
            b"WlxLoggedOnSAS(0x%lx)\n\0".as_ptr() as *const i8,
            dwSasType,
        );
    }

    match dwSasType {
        1 | 0 => {
            SasAction = (*pGinaUI).LoggedOnSAS.expect("non-null function pointer")(pgContext, dwSasType);
        }
        5 => {
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_FIXME as i32) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                    b"WlxLoggedOnSAS\0".as_ptr() as *const i8,
                    587 as i32,
                    b"WlxLoggedOnSAS: SasType WLX_SAS_TYPE_SC_INSERT not supported!\n\0".as_ptr() as *const i8,
                );
            }
        }
        6 => {
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_FIXME as i32) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                    b"WlxLoggedOnSAS\0".as_ptr() as *const i8,
                    592 as i32,
                    b"WlxLoggedOnSAS: SasType WLX_SAS_TYPE_SC_REMOVE not supported!\n\0".as_ptr() as *const i8,
                );
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_WARN as i32) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                    b"WlxLoggedOnSAS\0".as_ptr() as *const i8,
                    597 as i32,
                    b"WlxLoggedOnSAS: Unknown SasType: 0x%x\n\0".as_ptr() as *const i8,
                    dwSasType,
                );
            }
        }
    }

    SasAction
}

#[no_mangle]
pub unsafe extern "system" fn WlxDisplayStatusMessage(
    pWlxContext: PVOID,
    hDesktop: HDESK,
    dwOptions: DWORD,
    pTitle: PWSTR,
    pMessage: PWSTR,
) -> BOOL {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;

    if ((*__wine_dbch___default).flags as c_int & (1 << __WINE_DBCL_TRACE as c_int)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0".as_ptr() as *const c_char,
            b"WlxDisplayStatusMessage\0".as_ptr() as *const c_char,
            618i32,
            b"WlxDisplayStatusMessage(\"%S\")\n\0".as_ptr() as *const c_char,
            pMessage,
        );
    }

    (*pGinaUI).DisplayStatusMessage.unwrap()(pgContext, hDesktop, dwOptions, pTitle, pMessage)
}

#[no_mangle]
pub unsafe extern "system" fn WlxRemoveStatusMessage(pWlxContext: PVOID) -> BOOL {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;
    TRACE!(b"WlxRemoveStatusMessage()\n\0".as_ptr() as PCHAR);
    (*pGinaUI).RemoveStatusMessage.expect("non-null function pointer")(pgContext)
}
#[no_mangle]
pub unsafe extern "C" fn DoAdminUnlock(
    pgContext: PGINA_CONTEXT,
    UserName: PWSTR,
    Domain: PWSTR,
    Password: PWSTR,
) -> BOOL {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut Groups: PTOKEN_GROUPS = core::ptr::null_mut();
    let mut bIsAdmin: BOOL = 0;
    let mut Size: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;
    let mut SubStatus: NTSTATUS = STATUS_SUCCESS;

    if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
            b"DoAdminUnlock\0".as_ptr() as *const i8,
            653,
            b"(%S %S %S)\n\0".as_ptr() as *const i8,
            UserName,
            Domain,
            Password,
        );
    }

    Status = ConnectToLsa(pgContext);
    if Status < 0 {
        if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_WARN) != 0 {
            ros_dbg_log(
                __WINE_DBCL_WARN,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                b"DoAdminUnlock\0".as_ptr() as *const i8,
                658,
                b"ConnectToLsa() failed\n\0".as_ptr() as *const i8,
            );
        }
        return 0;
    }

    Status = MyLogonUser(
        (*pgContext).LsaHandle,
        (*pgContext).AuthenticationPackage,
        UserName,
        Domain,
        Password,
        &raw mut (*pgContext).UserToken,
        &raw mut SubStatus,
    );
    if Status < 0 {
        if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_WARN) != 0 {
            ros_dbg_log(
                __WINE_DBCL_WARN,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                b"DoAdminUnlock\0".as_ptr() as *const i8,
                671,
                b"MyLogonUser() failed\n\0".as_ptr() as *const i8,
            );
        }
        return 0;
    }

    Status = NtQueryInformationToken(hToken, TokenGroups, core::ptr::null_mut(), 0, &raw mut Size);
    if Status != STATUS_SUCCESS && Status != STATUS_BUFFER_TOO_SMALL {
        if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                b"DoAdminUnlock\0".as_ptr() as *const i8,
                682,
                b"NtQueryInformationToken() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    } else {
        Groups = HeapAlloc(GetProcessHeap(), 0, Size as usize) as PTOKEN_GROUPS;
        if Groups.is_null() {
            if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                    b"DoAdminUnlock\0".as_ptr() as *const i8,
                    689,
                    b"HeapAlloc() failed\n\0".as_ptr() as *const i8,
                );
            }
        } else {
            Status = NtQueryInformationToken(
                hToken,
                TokenGroups,
                Groups as *mut c_void,
                Size,
                &raw mut Size,
            );
            if Status < 0 {
                if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                        b"DoAdminUnlock\0".as_ptr() as *const i8,
                        700,
                        b"NtQueryInformationToken() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                        Status,
                    );
                }
            } else {
                i = 0;
                while i < (*Groups).GroupCount {
                    // Groups is a C flexible array (`SID_AND_ATTRIBUTES
                    // Groups[ANYSIZE_ARRAY]`, xdk/setypes.h:1030) in a buffer sized by
                    // the NtQueryInformationToken query above, so bindgen's
                    // [SID_AND_ATTRIBUTES; 1] bounds-checks this walk and panics for any
                    // token carrying more than one group -- which is every real token.
                    let group = (&raw const (*Groups).Groups[0]).add(i as usize);
                    if RtlEqualSid((*group).Sid, AdminSid) != 0 {
                        if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
                            ros_dbg_log(
                                __WINE_DBCL_TRACE,
                                __wine_dbch___default,
                                b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
                                b"DoAdminUnlock\0".as_ptr() as *const i8,
                                708,
                                b"Member of Admins group\n\0".as_ptr() as *const i8,
                            );
                        }
                        bIsAdmin = 1;
                        break;
                    }
                    i = i.wrapping_add(1);
                }
            }
        }
    }

    if !Groups.is_null() {
        HeapFree(GetProcessHeap(), 0, Groups as *mut c_void);
    }
    if !hToken.is_null() {
        CloseHandle(hToken);
    }
    bIsAdmin
}
#[no_mangle]
pub unsafe extern "C" fn DoLoginTasks(
    pgContext: PGINA_CONTEXT,
    UserName: PWSTR,
    Domain: PWSTR,
    Password: PWSTR,
    SubStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = ConnectToLsa(pgContext);
    if !NT_SUCCESS(Status) {
        if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_WARN as i32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/msgina/msgina.c\0".as_ptr()
                    as *const i8,
                b"DoLoginTasks\0".as_ptr() as *const i8,
                738i32,
                b"ConnectToLsa() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
        return Status;
    }

    Status = MyLogonUser(
        (*pgContext).LsaHandle,
        (*pgContext).AuthenticationPackage,
        UserName as LPWSTR,
        Domain as LPWSTR,
        Password as LPWSTR,
        &raw mut (*pgContext).UserToken,
        SubStatus,
    );

    if !NT_SUCCESS(Status) {
        if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_WARN as i32)) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/msgina/msgina.c\0".as_ptr()
                    as *const i8,
                b"DoLoginTasks\0".as_ptr() as *const i8,
                751i32,
                b"MyLogonUser() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CreateProfile(
    pgContext: PGINA_CONTEXT,
    UserName: PWSTR,
    Domain: PWSTR,
    Password: PWSTR,
) -> BOOL {
    let mut pProfile: PWLX_PROFILE_V2_0 = core::ptr::null_mut();
    let mut pEnvironment: PWSTR = core::ptr::null_mut();
    let mut Stats: TOKEN_STATISTICS = core::mem::zeroed();
    let mut cbStats: DWORD = 0;
    let mut cbSize: DWORD = 0;
    let mut dwLength: DWORD = 0;

    GetLocalTime(core::ptr::addr_of_mut!((*pgContext).LogonTime));

    wcscpy(core::ptr::addr_of_mut!((*pgContext).UserName) as *mut wchar_t, UserName as *const wchar_t);

    if Domain.is_null() || *Domain == 0 {
        dwLength = (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as DWORD;
        GetComputerNameW(core::ptr::addr_of_mut!((*pgContext).DomainName) as LPWSTR, core::ptr::addr_of_mut!(dwLength));
    } else {
        wcscpy(core::ptr::addr_of_mut!((*pgContext).DomainName) as *mut wchar_t, Domain as *const wchar_t);
    }

    let UserName: PWSTR = core::ptr::addr_of_mut!((*pgContext).UserName) as *mut WCHAR;
    let Domain: PWSTR = core::ptr::addr_of_mut!((*pgContext).DomainName) as *mut WCHAR;

    pProfile = LocalAlloc((LMEM_FIXED | LMEM_ZEROINIT) as UINT, core::mem::size_of::<_WLX_PROFILE_V2_0>() as SIZE_T) as PWLX_PROFILE_V2_0;
    if pProfile.is_null() {
        if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_WARN as i32 != 0 {
            let __dbch = __wine_dbch___default;
            let __dbcl = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/msgina/msgina.c\0" as *const u8 as *const i8,
                b"CreateProfile\0" as *const u8 as *const i8,
                805,
                b"HeapAlloc() failed\n\0" as *const u8 as *const i8,
            );
        }
    } else {
        (*pProfile).dwType = WLX_PROFILE_TYPE_V2_0 as DWORD;
        (*pProfile).pszProfile = core::ptr::null_mut();
        (*pProfile).pszPolicy = core::ptr::null_mut();
        (*pProfile).pszNetworkDefaultUserProfile = core::ptr::null_mut();
        (*pProfile).pszServerName = core::ptr::null_mut();

        cbSize = (core::mem::size_of::<[u16; 15]>()
            + wcslen(Domain as *const u16) as usize * core::mem::size_of::<WCHAR>()
            + core::mem::size_of::<WCHAR>()) as DWORD;

        pEnvironment = LocalAlloc(LMEM_FIXED as UINT, cbSize as SIZE_T) as PWSTR;
        if pEnvironment.is_null() {
            if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_WARN as i32 != 0 {
                let __dbch = __wine_dbch___default;
                let __dbcl = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/msgina/msgina.c\0" as *const u8 as *const i8,
                    b"CreateProfile\0" as *const u8 as *const i8,
                    868,
                    b"LocalAlloc() failed\n\0" as *const u8 as *const i8,
                );
            }
        } else {
            StringCbPrintfW!(
                pEnvironment as STRSAFE_LPWSTR,
                cbSize as size_t,
                { static __WS2: [u16; 18] = unsafe { core::mem::transmute::<[u8; 36], [u16; 18]>(*b"L\0O\0G\0O\0N\0S\0E\0R\0V\0E\0R\0=\0\\\0\\\0%\0w\0s\0\0\0") }; __WS2.as_ptr() as STRSAFE_LPCWSTR },
                Domain,
            );
            if wcslen(pEnvironment as *const u16) as u32 != (cbSize as u32 / core::mem::size_of::<WCHAR>() as u32) - 2 {
                RtlAssert(
                    b"wcslen(pEnvironment) == cbSize / sizeof(WCHAR) - 2\0" as *const u8 as PVOID,
                    b"dll/win32/msgina/msgina.c\0" as *const u8 as PVOID,
                    873,
                    core::ptr::null_mut(),
                );
            }
            *pEnvironment.offset(((cbSize as usize / core::mem::size_of::<WCHAR>()) - 1).try_into().unwrap()) = UNICODE_NULL;
            (*pProfile).pszEnvironment = pEnvironment;

            if GetTokenInformation(
                (*pgContext).UserToken,
                TokenStatistics,
                &raw mut Stats as PVOID,
                core::mem::size_of::<TOKEN_STATISTICS>() as DWORD,
                &raw mut cbStats,
            ) == 0 {
                if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_WARN as i32 != 0 {
                    let __dbch = __wine_dbch___default;
                    let __dbcl = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/msgina/msgina.c\0" as *const u8 as *const i8,
                        b"CreateProfile\0" as *const u8 as *const i8,
                        885,
                        b"Couldn't get Authentication Id from user token!\n\0" as *const u8 as *const i8,
                    );
                }
            } else {
                *(*pgContext).pAuthenticationId = Stats.AuthenticationId;
                (*(*pgContext).pMprNotifyInfo).pszUserName = DuplicateString(UserName as PCWSTR);
                (*(*pgContext).pMprNotifyInfo).pszDomain = DuplicateString(Domain as PCWSTR);
                (*(*pgContext).pMprNotifyInfo).pszPassword = DuplicateString(Password as PCWSTR);
                (*(*pgContext).pMprNotifyInfo).pszOldPassword = core::ptr::null_mut();
                *(*pgContext).pdwOptions = 0;
                *(*pgContext).pProfile = pProfile as PVOID;
                return 1;
            }
        }
    }

    if !pEnvironment.is_null() {
        LocalFree(pEnvironment as HLOCAL);
    }
    if !pProfile.is_null() {
        LocalFree(pProfile as HLOCAL);
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn WlxDisplaySASNotice(pWlxContext: PVOID) {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;

    // TRACE("WlxDisplaySASNotice(%p)\n", pWlxContext)
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0"
                .as_ptr() as *const i8,
            b"WlxDisplaySASNotice\0".as_ptr() as *const i8,
            916i32,
            b"WlxDisplaySASNotice(%p)\n\0".as_ptr() as *const i8,
            pWlxContext,
        );
    }

    if GetSystemMetrics(SM_REMOTESESSION.try_into().unwrap()) != 0 {
        /* User is remotely logged on. Don't display a notice */
        ((*(*pgContext).pWlxFuncs).WlxSasNotify.expect("non-null function pointer"))(
            (*pgContext).hWlx,
            WLX_SAS_TYPE_CTRL_ALT_DEL as u32,
        );
        return;
    }

    if (*pgContext).bAutoAdminLogon != 0 {
        if (*pgContext).bIgnoreShiftOverride != 0 || GetKeyState(VK_SHIFT.try_into().unwrap()) as i32 >= 0 {
            /* Don't display the window, we want to do an automatic logon */
            ((*(*pgContext).pWlxFuncs).WlxSasNotify.expect("non-null function pointer"))(
                (*pgContext).hWlx,
                WLX_SAS_TYPE_CTRL_ALT_DEL as u32,
            );
            return;
        }

        (*pgContext).bAutoAdminLogon = 0i32; // FALSE
    }

    if (*pgContext).bDisableCAD != 0 {
        ((*(*pgContext).pWlxFuncs).WlxSasNotify.expect("non-null function pointer"))(
            (*pgContext).hWlx,
            WLX_SAS_TYPE_CTRL_ALT_DEL as u32,
        );
        return;
    }

    (*pGinaUI)
        .DisplaySASNotice
        .expect("non-null function pointer")(pgContext);

    // TRACE("WlxDisplaySASNotice() done\n")
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0"
                .as_ptr() as *const i8,
            b"WlxDisplaySASNotice\0".as_ptr() as *const i8,
            946i32,
            b"WlxDisplaySASNotice() done\n\0".as_ptr() as *const i8,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn WlxLoggedOutSAS(
    mut pWlxContext: PVOID,
    mut dwSasType: DWORD,
    mut pAuthenticationId: PLUID,
    mut pLogonSid: PSID,
    mut pdwOptions: PDWORD,
    mut phToken: PHANDLE,
    mut pMprNotifyInfo: PWLX_MPR_NOTIFY_INFO,
    mut pProfile: *mut PVOID,
) -> INT {
    let mut pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;
    let mut res: INT = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/msgina/msgina.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"WlxLoggedOutSAS\0" as *const u8 as *const core::ffi::c_char,
            966 as core::ffi::c_int,
            b"WlxLoggedOutSAS()\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }
    (*pgContext).pAuthenticationId = pAuthenticationId;
    (*pgContext).pdwOptions = pdwOptions;
    (*pgContext).pMprNotifyInfo = pMprNotifyInfo;
    (*pgContext).pProfile = pProfile;
    res = (*pGinaUI).LoggedOutSAS.expect("non-null function pointer")(pgContext);
    if res == WLX_SAS_ACTION_LOGON.try_into().unwrap() {
        *phToken = (*pgContext).UserToken as *mut core::ffi::c_void;
    }
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn WlxWkstaLockedSAS(
    pWlxContext: PVOID,
    dwSasType: DWORD,
) -> INT {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;
    let _ = dwSasType;

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0".as_ptr() as *const i8,
            b"WlxWkstaLockedSAS\0".as_ptr() as *const i8,
            995i32,
            b"WlxWkstaLockedSAS()\n\0".as_ptr() as *const i8,
        );
    }

    return ((*pGinaUI).LockedSAS).expect("non-null function pointer")(pgContext);
}
#[no_mangle]
pub unsafe extern "system" fn WlxDisplayLockedNotice(pWlxContext: PVOID) {
    let pgContext: PGINA_CONTEXT = pWlxContext as PGINA_CONTEXT;

    TRACE!(b"WlxDisplayLockedNotice()\n\0".as_ptr() as *const i8);

    if (*pgContext).bDisableCAD != 0i32 {
        let pWlxFuncs = (*pgContext).pWlxFuncs;
        ((*pWlxFuncs).WlxSasNotify.expect("non-null function pointer"))(
            (*pgContext).hWlx,
            WLX_SAS_TYPE_CTRL_ALT_DEL as u32,
        );
        return;
    }

    let pGinaUI_ptr = pGinaUI;
    ((*pGinaUI_ptr).DisplayLockedNotice.expect("non-null function pointer"))(pgContext);
}

#[no_mangle]
pub unsafe extern "system" fn WlxIsLogoffOk(pWlxContext: *mut core::ffi::c_void) -> i32 {
    TRACE!(b"WlxIsLogoffOk()\n\0".as_ptr() as *const i8);
    UNREFERENCED_PARAMETER!(pWlxContext);
    1i32
}
#[no_mangle]
pub unsafe extern "system" fn WlxLogoff(pWlxContext: *mut core::ffi::c_void) {
    let pgContext: *mut _GINA_CONTEXT = pWlxContext as *mut _GINA_CONTEXT;

    if ((*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/msgina/msgina.c\0"
                .as_ptr() as *const i8,
            b"WlxLogoff\0".as_ptr() as *const i8,
            1046i32,
            b"WlxLogoff(%p)\n\0".as_ptr() as *const i8,
            pWlxContext,
        );
    }

    (*pgContext).pAuthenticationId = core::ptr::null_mut();
    (*pgContext).pdwOptions = core::ptr::null_mut();
    (*pgContext).pMprNotifyInfo = core::ptr::null_mut();
    (*pgContext).pProfile = core::ptr::null_mut();

    core::ptr::write_bytes(
        &raw mut (*pgContext).LogonTime as *mut core::ffi::c_void,
        0u8,
        core::mem::size_of::<SYSTEMTIME>(),
    );

    RtlSecureZeroMemory(
        &raw mut (*pgContext).Password as *mut u16 as *mut core::ffi::c_void,
        (core::mem::size_of::<[u16; 256]>() as u32).into(),
    );

    CloseHandle((*pgContext).UserToken);
    (*pgContext).UserToken = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn WlxShutdown(pWlxContext: *mut core::ffi::c_void, ShutdownType: u32) {
    let pgContext = pWlxContext as *mut _GINA_CONTEXT;
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"msgina.c\0".as_ptr() as *const i8,
            b"WlxShutdown\0".as_ptr() as *const i8,
            1082i32,
            b"WlxShutdown(%p %lx)\n\0".as_ptr() as *const i8,
            pWlxContext,
            ShutdownType,
        );
    }
    (*pgContext).AuthenticationPackage = 0u32;
    let Status: i32 = LsaDeregisterLogonProcess((*pgContext).LsaHandle);
    if !NT_SUCCESS(Status) {
        if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"msgina.c\0".as_ptr() as *const i8,
                b"WlxShutdown\0".as_ptr() as *const i8,
                1089i32,
                b"LsaDeregisterLogonProcess failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH as u32 {
        hDllInstance = hinstDLL;
        RtlAllocateAndInitializeSid(
            &raw mut SystemAuthority,
            2u8,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_ADMINS as u32,
            SECURITY_NULL_RID as u32,
            SECURITY_NULL_RID as u32,
            SECURITY_NULL_RID as u32,
            SECURITY_NULL_RID as u32,
            SECURITY_NULL_RID as u32,
            SECURITY_NULL_RID as u32,
            &raw mut AdminSid,
        );
    } else if dwReason == DLL_PROCESS_DETACH as u32 {
        if !AdminSid.is_null() {
            RtlFreeSid(AdminSid);
        }
    }
    return 1; // TRUE
}

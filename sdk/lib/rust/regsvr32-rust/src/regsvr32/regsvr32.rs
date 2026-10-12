use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_30F74109FAE0: [u16; 18] = [
    0x0044u16, 0x006C, 0x006C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0065,
    0x0072, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_432764A97422: [u16; 20] = [
    0x0044u16, 0x006C, 0x006C, 0x0055, 0x006E, 0x0072, 0x0065, 0x0067, 0x0069, 0x0073,
    0x0074, 0x0065, 0x0072, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_81C1993C4D09: [u16; 9] = [0x0052u16, 0x0065, 0x0067, 0x0053, 0x0076, 0x0072, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_C2R_WSTR_E9564764F178: [u16; 11] = [
    0x0044u16, 0x006C, 0x006C, 0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C,
    0x0000,
];

#[no_mangle]
pub static mut szDllRegister: LPCSTR = b"DllRegisterServer\0" as *const u8 as LPCSTR;

#[no_mangle]
pub static mut szDllUnregister: LPCSTR = b"DllUnregisterServer\0" as *const u8 as LPCSTR;

#[no_mangle]
pub static mut szDllInstall: LPCSTR = b"DllInstall\0" as *const u8 as LPCSTR;

#[no_mangle]
pub static mut tszDllRegister: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_30F74109FAE0.as_ptr() as LPCWSTR
};

#[no_mangle]
pub static mut tszDllUnregister: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_432764A97422.as_ptr() as LPCWSTR
};

#[no_mangle]
pub static mut tszDllInstall: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_E9564764F178.as_ptr() as LPCWSTR
};

#[no_mangle]
pub static mut ModuleTitle: LPCTSTR = unsafe {
    RUSTCONV_C2R_WSTR_81C1993C4D09.as_ptr() as LPCTSTR
};

#[no_mangle]
pub static mut UsageMessage: [TCHAR; 4096] = [0; 4096];

#[no_mangle]
pub static mut NoDllSpecified: [TCHAR; 4096] = [0; 4096];

#[no_mangle]
pub static mut InvalidFlag: [TCHAR; 4096] = [0; 4096];

#[no_mangle]
pub static mut SwitchN_NoI: [TCHAR; 4096] = [0; 4096];

#[no_mangle]
pub static mut DllNotLoaded: [TCHAR; 4096] = [0; 4096];

#[no_mangle]
pub static mut MissingEntry: [TCHAR; 4096] = [0; 4096];

#[no_mangle]
pub static mut FailureMessage: [TCHAR; 4096] = [0; 4096];

#[no_mangle]
pub static mut SuccessMessage: [TCHAR; 4096] = [0; 4096];

// C function-pointer typedefs: nullable, so Option<fn> (pointer-sized, NULL = None).
pub type DLLREGISTER = Option<unsafe extern "system" fn() -> HRESULT>;
pub type DLLINSTALL = Option<unsafe extern "system" fn(bInstall: BOOL, lpwCmdLine: LPWSTR) -> HRESULT>;

pub const EXITCODE_SUCCESS: i32 = 0;
pub const EXITCODE_PARAMERROR: i32 = 1;
pub const EXITCODE_LOADERROR: i32 = 3;
pub const EXITCODE_NOENTRY: i32 = 4;
pub const EXITCODE_FAILURE: i32 = 5;


// (&[0x0052, 0x0065, 0x0067, 0x0053, 0x0076, 0x0072, 0x0033, 0x0032, 0x0000u16]).as_ptr() /* _T "RegSvr32" */ is a wide literal under UNICODE; no envelope token was provided for it.
#[no_mangle]
pub unsafe extern "C" fn DisplayMessage(bConsole: BOOL, bSilent: BOOL, lpMessage: LPCTSTR, lpTitle: LPCTSTR, uType: UINT) {
    if bSilent == 0 {
        MessageBox(core::ptr::null_mut(), lpMessage, lpTitle, uType);
    }
    if bConsole != 0 {
        _tprintf(
            ['%' as u16, 's' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, '\n' as u16, 0u16].as_ptr() as LPCTSTR,
            lpTitle,
            lpMessage,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPTSTR,
    nCmdShow: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut argc: ::core::ffi::c_int = 0;
    let mut argv: *mut LPTSTR = core::ptr::null_mut();
    let mut lptDllName: LPTSTR = core::ptr::null_mut();
    let mut lptDllCmdLine: LPTSTR = core::ptr::null_mut();
    let mut lptMsgBuffer: LPTSTR = core::ptr::null_mut();
    let mut lptFuncName: LPCTSTR = core::ptr::null();
    let mut lpFuncName: LPCSTR = core::ptr::null();
    let mut lpwDllCmdLine: LPWSTR = core::ptr::null_mut();
    let mut bUnregister: BOOL = 0;
    let mut bSilent: BOOL = 0;
    let mut bConsole: BOOL = 0;
    let mut bInstall: BOOL = 0;
    let mut bNoRegister: BOOL = 0;
    let mut nDllCount: UINT = 0;
    let mut fuOldErrorMode: UINT = 0;
    let mut hDll: HMODULE = core::ptr::null_mut();
    let mut fnDllRegister: DLLREGISTER = None;
    let mut fnDllInstall: DLLINSTALL = None;
    let mut hResult: HRESULT = 0;
    let mut dwErr: DWORD = 0;
    let mut nRetValue: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;

    // Get Langues msg
    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_UsageMessage as u32),
        core::ptr::addr_of_mut!(UsageMessage) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );
    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_NoDllSpecified as u32),
        core::ptr::addr_of_mut!(NoDllSpecified) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );
    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_InvalidFlag as u32),
        core::ptr::addr_of_mut!(InvalidFlag) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );
    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_SwitchN_NoI as u32),
        core::ptr::addr_of_mut!(SwitchN_NoI) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );

    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_DllNotLoaded as u32),
        core::ptr::addr_of_mut!(DllNotLoaded) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );
    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_MissingEntry as u32),
        core::ptr::addr_of_mut!(MissingEntry) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );
    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_FailureMessage as u32),
        core::ptr::addr_of_mut!(FailureMessage) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );
    LoadString(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (IDS_SuccessMessage as u32),
        core::ptr::addr_of_mut!(SuccessMessage) as LPTSTR,
        RC_STRING_MAX_SIZE,
    );

    // Get command-line in argc-argv format
    argv = CommandLineToArgv(
        GetCommandLine(),
        core::ptr::addr_of_mut!(argc),
    ) as *mut LPTSTR;

    // Initialize variables
    lptFuncName = core::ptr::null();
    lptDllCmdLine = core::ptr::null_mut();
    nDllCount = 0;
    bUnregister = (FALSE as i32);
    bSilent = (FALSE as i32);
    bConsole = (FALSE as i32);
    bInstall = (FALSE as i32);
    bNoRegister = (FALSE as i32);

    // Find all arguments starting with a slash (/)
    i = 1;
    while i < argc {
        if *(*argv.offset(i as isize)) == ('/' as u16) || *(*argv.offset(i as isize)) == '-' as TCHAR {
            match *((*argv.offset(i as isize)).offset(1 as isize)) as ::core::ffi::c_int {
                // ('u' as u16) / ('U' as u16)
                117 | 85 => {
                    bUnregister = (TRUE as i32);
                }
                // ('s' as u16) / ('S' as u16)
                115 | 83 => {
                    bSilent = (TRUE as i32);
                }
                // ('c' as u16) / ('C' as u16)
                99 | 67 => {
                    bConsole = (TRUE as i32);
                }
                // ('i' as u16) / ('I' as u16)
                105 | 73 => {
                    bInstall = (TRUE as i32);
                    lptDllCmdLine = *argv.offset(i as isize);
                    while *lptDllCmdLine != 0 as TCHAR && *lptDllCmdLine != (':' as u16) {
                        lptDllCmdLine = lptDllCmdLine.offset(1);
                    }
                    if *lptDllCmdLine == (':' as u16) {
                        lptDllCmdLine = lptDllCmdLine.offset(1);
                    }
                }
                // ('n' as u16) / ('N' as u16)
                110 | 78 => {
                    bNoRegister = (TRUE as i32);
                }
                _ => {
                    if lptFuncName.is_null() {
                        lptFuncName = *argv.offset(i as isize) as LPCTSTR;
                    }
                }
            }
        } else {
            nDllCount = nDllCount.wrapping_add(1);
        }
        i += 1;
    }

    // An unrecognized flag was used, display a message and show available options
    if !lptFuncName.is_null() {
        lptMsgBuffer = malloc(
            (_tcslen(core::ptr::addr_of_mut!(UsageMessage) as LPTSTR)
                - 2
                + _tcslen(core::ptr::addr_of_mut!(InvalidFlag) as LPTSTR)
                - 2
                + _tcslen(lptFuncName)
                + 1)
                * (core::mem::size_of::<TCHAR>() as usize),
        ) as LPTSTR;
        _stprintf(
            lptMsgBuffer.add(
                _tcslen(core::ptr::addr_of_mut!(UsageMessage) as LPTSTR)
                    - 2,
            ),
            core::ptr::addr_of_mut!(InvalidFlag) as LPTSTR,
            lptFuncName,
        );
        _stprintf(
            lptMsgBuffer,
            core::ptr::addr_of_mut!(UsageMessage) as LPTSTR,
            lptMsgBuffer.add(
                _tcslen(core::ptr::addr_of_mut!(UsageMessage) as LPTSTR)
                    - 2,
            ),
        );
        DisplayMessage(
            bConsole,
            bSilent,
            lptMsgBuffer as LPCTSTR,
            ModuleTitle,
            MB_ICONEXCLAMATION,
        );
        free(lptMsgBuffer as *mut ::core::ffi::c_void);
        GlobalFree(argv as HGLOBAL);
        return EXITCODE_PARAMERROR;
    }

    // /n was used without /i, display a message and show available options
    if bNoRegister != 0 && bInstall == 0 {
        lptMsgBuffer = malloc(
            (_tcslen(core::ptr::addr_of_mut!(UsageMessage) as LPTSTR)
                - 2
                + _tcslen(core::ptr::addr_of_mut!(SwitchN_NoI) as LPTSTR)
                + 1)
                * (core::mem::size_of::<TCHAR>() as usize),
        ) as LPTSTR;
        _stprintf(
            lptMsgBuffer,
            core::ptr::addr_of_mut!(UsageMessage) as LPTSTR,
            core::ptr::addr_of_mut!(SwitchN_NoI) as LPTSTR,
        );
        DisplayMessage(
            bConsole,
            bSilent,
            lptMsgBuffer as LPCTSTR,
            ModuleTitle,
            MB_ICONEXCLAMATION,
        );
        free(lptMsgBuffer as *mut ::core::ffi::c_void);
        GlobalFree(argv as HGLOBAL);
        return EXITCODE_PARAMERROR;
    }

    // No dll was specified, display a message and show available options
    if nDllCount == 0 {
        lptMsgBuffer = malloc(
            (_tcslen(core::ptr::addr_of_mut!(UsageMessage) as LPTSTR)
                - 2
                + _tcslen(core::ptr::addr_of_mut!(NoDllSpecified) as LPTSTR)
                + 1)
                * (core::mem::size_of::<TCHAR>() as usize),
        ) as LPTSTR;
        _stprintf(
            lptMsgBuffer,
            core::ptr::addr_of_mut!(UsageMessage) as LPTSTR,
            core::ptr::addr_of_mut!(NoDllSpecified) as LPTSTR,
        );
        DisplayMessage(
            bConsole,
            bSilent,
            lptMsgBuffer as LPCTSTR,
            ModuleTitle,
            MB_ICONEXCLAMATION,
        );
        free(lptMsgBuffer as *mut ::core::ffi::c_void);
        GlobalFree(argv as HGLOBAL);
        return EXITCODE_PARAMERROR;
    }

    nRetValue = EXITCODE_SUCCESS;
    if bUnregister == 0 {
        lpFuncName = szDllRegister;
        lptFuncName = tszDllRegister as LPCTSTR;
    } else {
        lpFuncName = szDllUnregister;
        lptFuncName = tszDllUnregister as LPCTSTR;
    }

    if !lptDllCmdLine.is_null() {
        lpwDllCmdLine = ConvertToWideChar!(lptDllCmdLine);
    } else {
        lpwDllCmdLine = core::ptr::null_mut();
    }

    // Initialize OLE32 before attempting to register the
    // dll.  Some dll's require this to register properly
    OleInitialize(core::ptr::null_mut());

    // (Un)register every dll whose filename was passed in the command-line string
    i = 1;
    while i < argc && nRetValue == EXITCODE_SUCCESS {
        // Arguments that do not start with a slash (/) are filenames
        if *(*argv.offset(i as isize)) != ('/' as u16) {
            lptDllName = *argv.offset(i as isize);

            fuOldErrorMode = SetErrorMode(SEM_FAILCRITICALERRORS);
            // Everything is all setup, so load the dll now
            hDll = LoadLibraryEx(
                lptDllName as LPCWSTR,
                core::ptr::null_mut(),
                LOAD_WITH_ALTERED_SEARCH_PATH,
            ) as HMODULE;
            if !hDll.is_null() {
                SetErrorMode(fuOldErrorMode);
                if bNoRegister == 0 {
                    // Get the address of DllRegisterServer or DllUnregisterServer
                    fnDllRegister = ::core::mem::transmute::<FARPROC, DLLREGISTER>(
                        GetProcAddress(hDll as HINSTANCE, lpFuncName),
                    );
                    if fnDllRegister.is_some() {
                        // If the function exists, call it
                        hResult = (fnDllRegister.unwrap_unchecked())();
                        if hResult == S_OK {
                            // (Un)register succeeded, display a message
                            lptMsgBuffer = malloc(
                                (_tcslen(core::ptr::addr_of_mut!(SuccessMessage) as LPTSTR)
                                    - 4
                                    + _tcslen(lptFuncName)
                                    + _tcslen(lptDllName)
                                    + 1)
                                    * (core::mem::size_of::<TCHAR>() as usize),
                            ) as LPTSTR;
                            _stprintf(
                                lptMsgBuffer,
                                core::ptr::addr_of_mut!(SuccessMessage) as LPTSTR,
                                lptFuncName,
                                lptDllName,
                            );
                            DisplayMessage(
                                bConsole,
                                bSilent,
                                lptMsgBuffer as LPCTSTR,
                                ModuleTitle,
                                MB_ICONINFORMATION,
                            );
                        } else {
                            // (Un)register failed, display a message
                            lptMsgBuffer = malloc(
                                (_tcslen(core::ptr::addr_of_mut!(FailureMessage) as LPTSTR)
                                    + _tcslen(lptFuncName)
                                    + _tcslen(lptDllName)
                                    + 1)
                                    * (core::mem::size_of::<TCHAR>() as usize),
                            ) as LPTSTR;
                            _stprintf(
                                lptMsgBuffer,
                                core::ptr::addr_of_mut!(FailureMessage) as LPTSTR,
                                lptFuncName,
                                lptDllName,
                                hResult,
                            );
                            DisplayMessage(
                                bConsole,
                                bSilent,
                                lptMsgBuffer as LPCTSTR,
                                ModuleTitle,
                                MB_ICONEXCLAMATION,
                            );
                        }
                        free(lptMsgBuffer as *mut ::core::ffi::c_void);
                        if hResult != S_OK {
                            nRetValue = EXITCODE_FAILURE;
                        }
                    } else {
                        FreeLibrary(hDll);
                        // Dll(Un)register was not found, display an error message
                        lptMsgBuffer = malloc(
                            (_tcslen(core::ptr::addr_of_mut!(MissingEntry) as LPTSTR)
                                - 8
                                + _tcslen(lptFuncName) * 2
                                + _tcslen(lptDllName) * 2
                                + 1)
                                * (core::mem::size_of::<TCHAR>() as usize),
                        ) as LPTSTR;
                        _stprintf(
                            lptMsgBuffer,
                            core::ptr::addr_of_mut!(MissingEntry) as LPTSTR,
                            lptDllName,
                            lptFuncName,
                            lptFuncName,
                            lptDllName,
                        );
                        DisplayMessage(
                            bConsole,
                            bSilent,
                            lptMsgBuffer as LPCTSTR,
                            ModuleTitle,
                            MB_ICONEXCLAMATION,
                        );
                        free(lptMsgBuffer as *mut ::core::ffi::c_void);
                        nRetValue = EXITCODE_NOENTRY;
                    }
                }

                if bInstall != 0 && nRetValue == EXITCODE_SUCCESS {
                    // Get the address of DllInstall
                    fnDllInstall = ::core::mem::transmute::<FARPROC, DLLINSTALL>(
                        GetProcAddress(hDll as HINSTANCE, szDllInstall),
                    );
                    if fnDllInstall.is_some() {
                        // If the function exists, call it
                        if bUnregister == 0 {
                            hResult = (fnDllInstall.unwrap_unchecked())(1, lpwDllCmdLine);
                        } else {
                            hResult = (fnDllInstall.unwrap_unchecked())(0, lpwDllCmdLine);
                        }
                        if hResult == S_OK {
                            // (Un)install succeeded, display a message
                            lptMsgBuffer = malloc(
                                (_tcslen(core::ptr::addr_of_mut!(SuccessMessage) as LPTSTR)
                                    - 4
                                    + _tcslen(tszDllInstall as LPCTSTR)
                                    + _tcslen(lptDllName)
                                    + 1)
                                    * (core::mem::size_of::<TCHAR>() as usize),
                            ) as LPTSTR;
                            _stprintf(
                                lptMsgBuffer,
                                core::ptr::addr_of_mut!(SuccessMessage) as LPTSTR,
                                tszDllInstall as LPCTSTR,
                                lptDllName,
                            );
                            DisplayMessage(
                                bConsole,
                                bSilent,
                                lptMsgBuffer as LPCTSTR,
                                ModuleTitle,
                                MB_ICONINFORMATION,
                            );
                        } else {
                            // (Un)install failed, display a message
                            lptMsgBuffer = malloc(
                                (_tcslen(core::ptr::addr_of_mut!(FailureMessage) as LPTSTR)
                                    + _tcslen(tszDllInstall as LPCTSTR)
                                    + _tcslen(lptDllName)
                                    + 1)
                                    * (core::mem::size_of::<TCHAR>() as usize),
                            ) as LPTSTR;
                            _stprintf(
                                lptMsgBuffer,
                                core::ptr::addr_of_mut!(FailureMessage) as LPTSTR,
                                tszDllInstall as LPCTSTR,
                                lptDllName,
                                hResult,
                            );
                            DisplayMessage(
                                bConsole,
                                bSilent,
                                lptMsgBuffer as LPCTSTR,
                                ModuleTitle,
                                MB_ICONEXCLAMATION,
                            );
                        }
                        free(lptMsgBuffer as *mut ::core::ffi::c_void);
                        if hResult != S_OK {
                            nRetValue = EXITCODE_FAILURE;
                        }
                    } else {
                        // DllInstall was not found, display an error message
                        lptMsgBuffer = malloc(
                            (_tcslen(core::ptr::addr_of_mut!(MissingEntry) as LPTSTR)
                                - 8
                                + _tcslen(tszDllInstall as LPCTSTR) * 2
                                + _tcslen(lptDllName) * 2
                                + 1)
                                * (core::mem::size_of::<TCHAR>() as usize),
                        ) as LPTSTR;
                        _stprintf(
                            lptMsgBuffer,
                            core::ptr::addr_of_mut!(MissingEntry) as LPTSTR,
                            lptDllName,
                            tszDllInstall as LPCTSTR,
                            tszDllInstall as LPCTSTR,
                            lptDllName,
                        );
                        DisplayMessage(
                            bConsole,
                            bSilent,
                            lptMsgBuffer as LPCTSTR,
                            ModuleTitle,
                            MB_ICONEXCLAMATION,
                        );
                        free(lptMsgBuffer as *mut ::core::ffi::c_void);
                        nRetValue = EXITCODE_NOENTRY;
                    }
                }

                // The dll function has finished executing, so unload it
                FreeLibrary(hDll);
            } else {
                // The dll could not be loaded; display an error message
                dwErr = GetLastError();
                SetErrorMode(fuOldErrorMode);
                lptMsgBuffer = malloc(
                    (_tcslen(core::ptr::addr_of_mut!(DllNotLoaded) as LPTSTR)
                        + 2
                        + _tcslen(lptDllName)
                        + 1)
                        * (core::mem::size_of::<TCHAR>() as usize),
                ) as LPTSTR;
                _stprintf(
                    lptMsgBuffer,
                    core::ptr::addr_of_mut!(DllNotLoaded) as LPTSTR,
                    lptDllName,
                    dwErr,
                );
                DisplayMessage(
                    bConsole,
                    bSilent,
                    lptMsgBuffer as LPCTSTR,
                    ModuleTitle,
                    MB_ICONEXCLAMATION,
                );
                free(lptMsgBuffer as *mut ::core::ffi::c_void);
                nRetValue = EXITCODE_LOADERROR;
            }
        }
        i += 1;
    }

    if !lpwDllCmdLine.is_null() {
        FreeConvertedWideChar!(lpwDllCmdLine);
    }
    GlobalFree(argv as HGLOBAL);
    OleUninitialize();
    return nRetValue;
}

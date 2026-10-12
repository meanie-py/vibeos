use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_7F0C8E8DD21F: [u16; 7] = [0x0052u16, 0x0075, 0x006E, 0x0044, 0x004C, 0x004C, 0x0000];
pub static RUSTCONV_C2R_WSTR_D767B87F69DC: [u16; 9] = [0x0072u16, 0x0075, 0x006E, 0x0064, 0x006C, 0x006C, 0x0033, 0x0032, 0x0000];

#[no_mangle]
pub static mut rundll32_wtitle: LPCTSTR = unsafe {
    RUSTCONV_C2R_WSTR_D767B87F69DC.as_ptr() as LPCTSTR
};

#[no_mangle]
pub static mut rundll32_wclass: LPCTSTR = unsafe {
    RUSTCONV_C2R_WSTR_7F0C8E8DD21F.as_ptr() as LPCTSTR
};

#[no_mangle]
pub static mut ModuleFileName: [TCHAR; 261] = [0; 261];

#[no_mangle]
pub static mut ModuleTitle: LPTSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

// WIN32_NO_STATUS: object-like macro with no value, nothing to emit.
// UNICODE is assumed defined, so _UNICODE/UNICODE macro setup is not emitted.
// ConvertToWideChar, FreeConvertedWideChar, ConvertToMultiByte and FreeConvertedMultiByte
// are function-like macros in the UNICODE branch and are skipped.

pub type DllWinMainW = Option<unsafe extern "system" fn(
    hWnd: HWND,
    hInstance: HINSTANCE,
    lpwCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32>;

pub type DllWinMainA = Option<unsafe extern "system" fn(
    hWnd: HWND,
    hInstance: HINSTANCE,
    lpCmdLine: LPSTR,
    nCmdShow: i32,
) -> i32>;

pub static mut rundll32_wtitle_buf: [TCHAR; 9] = [
    b'r' as TCHAR, b'u' as TCHAR, b'n' as TCHAR, b'd' as TCHAR,
    b'l' as TCHAR, b'l' as TCHAR, b'3' as TCHAR, b'2' as TCHAR,
    0 as TCHAR,
];

pub static mut rundll32_wclass_buf: [TCHAR; 7] = [
    b'R' as TCHAR, b'u' as TCHAR, b'n' as TCHAR, b'D' as TCHAR,
    b'L' as TCHAR, b'L' as TCHAR,
    0 as TCHAR,
];

#[repr(C)]
pub struct FIND_OWNED {
    pub hwndOwner: HWND,
    pub hwndTarget: HWND,
}
pub type PFIND_OWNED = *mut FIND_OWNED;
#[no_mangle]
pub unsafe extern "system" fn CommandLineToArgv(lpCmdLine: LPCTSTR, lpArgc: *mut ::core::ffi::c_int) -> *mut LPTSTR {
    let mut argv: *mut LPTSTR;
    let mut lpSrc: LPTSTR;
    let mut lpDest: LPTSTR;
    let mut lpArg: LPTSTR;
    let mut argc: ::core::ffi::c_int;
    let mut nBSlash: ::core::ffi::c_int;
    let mut bInQuotes: BOOL;
    let mut bFirstChar: BOOL;
    let mut nNames: ::core::ffi::c_int;

    // If null was passed in for lpCmdLine, there are no arguments
    if lpCmdLine.is_null() {
        if !lpArgc.is_null() {
            *lpArgc = 0;
        }
        return core::ptr::null_mut();
    }

    lpSrc = lpCmdLine as LPTSTR;
    // Skip spaces at beginning
    while *lpSrc == (' ' as u16) || *lpSrc == ('\t' as u16) {
        lpSrc = lpSrc.offset(1);
    }

    // If command-line starts with null, there are no arguments
    if *lpSrc == 0 {
        if !lpArgc.is_null() {
            *lpArgc = 0;
        }
        return core::ptr::null_mut();
    }

    lpArg = lpSrc;
    argc = 0;
    nBSlash = 0;
    bInQuotes = (FALSE as i32);
    bFirstChar = (TRUE as i32);
    nNames = 0;

    // Count the number of arguments
    // The C is this same `while`: its `continue` skips `lpSrc++` on purpose, because the
    // whitespace skip just before it has already advanced lpSrc.
    while nNames < 4 { // nolint: dowhile-continue (faithful: see above)
        if *lpSrc == 0
            || (*lpSrc == (',' as u16) && nNames == 2)
            || ((*lpSrc == (' ' as u16) || *lpSrc == ('\t' as u16)) && bInQuotes == 0)
        {
            // Whitespace not enclosed in quotes signals the start of another argument
            argc += 1;

            // Skip whitespace between arguments
            while *lpSrc == (' ' as u16)
                || *lpSrc == ('\t' as u16)
                || (*lpSrc == (',' as u16) && nNames == 2)
            {
                lpSrc = lpSrc.offset(1);
            }
            if *lpSrc == 0 {
                break;
            }
            if nNames >= 3 {
                // Increment the count for the last argument
                argc += 1;
                break;
            }
            nBSlash = 0;
            bFirstChar = (TRUE as i32);
            continue;
        } else if *lpSrc == ('\\' as u16) {
            // Count consecutive backslashes
            nBSlash += 1;
            bFirstChar = (FALSE as i32);
        } else if *lpSrc == ('\"' as u16) && (nBSlash & 1) == 0 {
            // Open or close quotes
            bInQuotes = (bInQuotes == 0) as ::core::ffi::c_int as BOOL;
            nBSlash = 0;
        } else {
            // Some other character
            nBSlash = 0;
            if bFirstChar != 0
                && ((*lpSrc != ('/' as u16) && nNames <= 1) || nNames > 1)
            {
                nNames += 1;
            }
            bFirstChar = (FALSE as i32);
        }
        lpSrc = lpSrc.offset(1);
    }

    // Allocate space for the pointers in argv and the strings in one block
    argv = malloc(
        (argc as usize) * ::core::mem::size_of::<LPTSTR>()
            + (_tcslen(lpArg) + 1) * core::mem::size_of::<TCHAR>(),
    ) as *mut LPTSTR;

    if argv.is_null() {
        // Memory allocation failed
        if !lpArgc.is_null() {
            *lpArgc = 0;
        }
        return core::ptr::null_mut();
    }

    lpSrc = lpArg;
    lpDest = argv.offset(argc as isize) as LPTSTR;
    lpArg = lpDest;
    argc = 0;
    nBSlash = 0;
    bInQuotes = (FALSE as i32);
    bFirstChar = (TRUE as i32);
    nNames = 0;

    // Fill the argument array
    while nNames < 4 {
        if *lpSrc == 0
            || (*lpSrc == (',' as u16) && nNames == 2)
            || ((*lpSrc == (' ' as u16) || *lpSrc == ('\t' as u16)) && bInQuotes == 0)
        {
            // Whitespace not enclosed in quotes signals the start of another argument
            // Null-terminate argument
            let fresh0 = lpDest;
            lpDest = lpDest.offset(1);
            *fresh0 = 0;
            let idx0 = argc;
            argc += 1;
            *argv.offset(idx0 as isize) = lpArg;

            // Skip whitespace between arguments
            while *lpSrc == (' ' as u16)
                || *lpSrc == ('\t' as u16)
                || (*lpSrc == (',' as u16) && nNames == 2)
            {
                lpSrc = lpSrc.offset(1);
            }
            if *lpSrc == 0 {
                break;
            }
            lpArg = lpDest;
            if nNames >= 3 {
                // Copy the rest of the command-line to the last argument
                let idx1 = argc;
                argc += 1;
                *argv.offset(idx1 as isize) = lpArg;
                _tcscpy(lpArg, lpSrc);
                break;
            }
            nBSlash = 0;
            bFirstChar = (TRUE as i32);
            continue;
        } else if *lpSrc == ('\\' as u16) {
            let fresh1 = lpDest;
            lpDest = lpDest.offset(1);
            *fresh1 = ('\\' as u16);
            lpSrc = lpSrc.offset(1);

            // Count consecutive backslashes
            nBSlash += 1;
            bFirstChar = (FALSE as i32);
        } else if *lpSrc == ('\"' as u16) {
            if (nBSlash & 1) == 0 {
                // If an even number of backslashes are before the quotes,
                // the quotes don't go in the output
                lpDest = lpDest.offset(-((nBSlash / 2) as isize));
                bInQuotes = (bInQuotes == 0) as ::core::ffi::c_int as BOOL;
            } else {
                // If an odd number of backslashes are before the quotes,
                // output a quote
                lpDest = lpDest.offset(
                    -(((nBSlash + 1) / 2) as isize),
                );
                let fresh2 = lpDest;
                lpDest = lpDest.offset(1);
                *fresh2 = ('\"' as u16);
                bFirstChar = (FALSE as i32);
            }
            lpSrc = lpSrc.offset(1);
            nBSlash = 0;
        } else {
            // Copy other characters
            if bFirstChar != 0
                && ((*lpSrc != ('/' as u16) && nNames <= 1) || nNames > 1)
            {
                nNames += 1;
            }
            let fresh3 = lpSrc;
            lpSrc = lpSrc.offset(1);
            let fresh4 = lpDest;
            lpDest = lpDest.offset(1);
            *fresh4 = *fresh3;
            nBSlash = 0;
            bFirstChar = (FALSE as i32);
        }
    }

    if !lpArgc.is_null() {
        *lpArgc = argc;
    }
    return argv;
}
#[no_mangle]
pub unsafe extern "C" fn GetModuleTitle() {
    let mut lpStr: LPTSTR;
    GetModuleFileName(core::ptr::null_mut(), core::ptr::addr_of_mut!(ModuleFileName) as LPTSTR, MAX_PATH);
    ModuleTitle = core::ptr::addr_of_mut!(ModuleFileName) as LPTSTR;

    lpStr = core::ptr::addr_of_mut!(ModuleFileName) as LPTSTR;
    while *lpStr != 0 {
        if *lpStr == '\\' as TCHAR {
            ModuleTitle = lpStr.offset(1);
        }
        lpStr = lpStr.offset(1);
    }

    static EXE_STR: [u16; 5] = ['.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0];

    lpStr = ModuleTitle;
    while *lpStr != 0 {
        if _tcsicmp(lpStr, EXE_STR.as_ptr() as LPCTSTR) == 0 {
            break;
        }
        lpStr = lpStr.offset(1);
    }

    *lpStr = 0 as TCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn DuplicateToMultiByte(
    mut lptString: LPCTSTR,
    mut nBufferSize: size_t,
) -> LPSTR {
    let mut lpString: LPSTR = core::ptr::null_mut::<CHAR>();
    let mut nStrLen: size_t = 0;
    nStrLen = _tcslen(lptString) + 1 as size_t;
    if nBufferSize == 0 as size_t {
        nBufferSize = nStrLen;
    }
    lpString = malloc(nBufferSize) as LPSTR;
    WideCharToMultiByte(
        0,
        0,
        lptString,
        nStrLen as core::ffi::c_int,
        lpString,
        nBufferSize as core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    return lpString;
}

unsafe extern "system" fn EnumWindowsProc(hwnd: HWND, lParam: LPARAM) -> BOOL {
    let pFindOwned: PFIND_OWNED = lParam as PFIND_OWNED;
    if (*pFindOwned).hwndOwner == GetWindow(hwnd, GW_OWNER as UINT) {
        (*pFindOwned).hwndTarget = hwnd;
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn EmptyWindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    'sw: {
        if uMsg == WM_POPUPSYSTEMMENU || uMsg == WM_SYSCOMMAND {
            /* Find the owned window */
            let mut FindOwned: FIND_OWNED = FIND_OWNED {
                hwndOwner: hWnd,
                hwndTarget: core::ptr::null_mut(),
            };
            EnumWindows(
                Some(EnumWindowsProc),
                core::ptr::addr_of_mut!(FindOwned) as LPARAM,
            );
            /* Forward message */
            if !FindOwned.hwndTarget.is_null() {
                PostMessageW(FindOwned.hwndTarget, uMsg, wParam, lParam);
            }
            break 'sw;
        }
        if uMsg == WM_ACTIVATE {
            /* Find the owned window */
            let mut FindOwned: FIND_OWNED = FIND_OWNED {
                hwndOwner: hWnd,
                hwndTarget: core::ptr::null_mut(),
            };
            EnumWindows(
                Some(EnumWindowsProc),
                core::ptr::addr_of_mut!(FindOwned) as LPARAM,
            );
            if !FindOwned.hwndTarget.is_null() {
                if (LOWORD!(wParam) as ::core::ffi::c_int) != (WA_INACTIVE as i32) {
                    /* To be activated */
                    SetActiveWindow(FindOwned.hwndTarget);
                    return 0 as LRESULT;
                }
            }
            /* Fall through */
        }
        /* default: */
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    return 0 as LRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterBlankClass(hInstance: HINSTANCE, hPrevInstance: HINSTANCE) -> BOOL {
    let mut wcex: WNDCLASSEX = core::mem::zeroed();

    wcex.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    wcex.style = 0 as UINT;
    wcex.lpfnWndProc =
        Some(EmptyWindowProc);
    wcex.cbClsExtra = 0 as ::core::ffi::c_int;
    wcex.cbWndExtra = 0 as ::core::ffi::c_int;
    wcex.hInstance = hInstance;
    wcex.hIcon = core::ptr::null_mut();
    wcex.hCursor = core::ptr::null_mut();
    wcex.hbrBackground = core::ptr::null_mut();
    wcex.lpszMenuName = core::ptr::null();
    wcex.lpszClassName = rundll32_wclass as LPCWSTR;
    wcex.hIconSm = core::ptr::null_mut();

    return (RegisterClassEx(core::ptr::addr_of_mut!(wcex))
        != (0 as ATOM)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPTSTR,
    nCmdShow: i32,
) -> i32 {
    let mut argc: i32 = 0;
    let mut szMsg: [TCHAR; (RC_STRING_MAX_SIZE as _)] = [0 as TCHAR; (RC_STRING_MAX_SIZE as _)];

    let mut argv: *mut LPTSTR = core::ptr::null_mut();
    let mut lptCmdLine: LPTSTR = core::ptr::null_mut();
    let mut lptDllName: LPTSTR = core::ptr::null_mut();
    let mut lptFuncName: LPTSTR = core::ptr::null_mut();
    let mut lptMsgBuffer: LPTSTR = core::ptr::null_mut();
    let mut EmptyStr: [TCHAR; 1] = [0 as TCHAR; 1];
    let mut ResolvedFile: [WCHAR; (MAX_PATH + 1) as _] =
        [0 as WCHAR; (MAX_PATH + 1) as _];
    let mut lpManifestName: *mut WCHAR = core::ptr::null_mut();
    let mut lpFuncName: LPSTR = core::ptr::null_mut();
    let mut lpaCmdLine: LPSTR = core::ptr::null_mut();
    let mut lpwCmdLine: LPWSTR = core::ptr::null_mut();
    let mut hDll: HMODULE = core::ptr::null_mut();
    let mut fnDllWinMainW: DllWinMainW = None;
    let mut fnDllWinMainA: DllWinMainA = None;
    let mut hWindow: HWND = core::ptr::null_mut();
    let mut i: i32 = 0;
    let mut nStrLen: size_t = 0;

    let mut ActCtx: ACTCTXW = ACTCTXW {
        cbSize: core::mem::size_of::<ACTCTX>() as ULONG,
        dwFlags: ACTCTX_FLAG_RESOURCE_NAME_VALID as DWORD,
        lpSource: core::ptr::null(),
        wProcessorArchitecture: 0,
        wLangId: 0,
        lpAssemblyDirectory: core::ptr::null(),
        lpResourceName: core::ptr::null(),
        lpApplicationName: core::ptr::null(),
        hModule: core::ptr::null_mut(),
    };
    let mut hActCtx: HANDLE = core::ptr::null_mut();
    let mut cookie: ULONG_PTR = 0;
    let mut bActivated: BOOL = 0;

    // Get command-line in argc-argv format
    argv = CommandLineToArgv(GetCommandLine(), core::ptr::addr_of_mut!(argc));

    // Skip all beginning arguments starting with a slash (/)
    i = 1;
    while i < argc {
        if *(*argv.offset(i as isize)) != '/' as TCHAR {
            break;
        }
        i += 1;
    }

    // If no dll was specified, there is nothing to do
    if i >= argc {
        if !argv.is_null() {
            free(argv as *mut core::ffi::c_void);
        }
        return 0;
    }

    let fresh0 = i;
    i += 1;
    lptDllName = *argv.offset(fresh0 as isize);

    // The next argument, which specifies the name of the dll function,
    // can either have a comma between it and the dll filename or a space.
    // Using a comma here is the preferred method
    if i < argc {
        let fresh1 = i;
        i += 1;
        lptFuncName = *argv.offset(fresh1 as isize);
    } else {
        lptFuncName = core::ptr::addr_of_mut!(EmptyStr) as LPTSTR;
    }

    // If no function name was specified, nothing needs to be done
    if *lptFuncName == 0 {
        if !argv.is_null() {
            free(argv as *mut core::ffi::c_void);
        }
        return 0;
    }

    // The rest of the arguments will be passed to dll function
    if i < argc {
        lptCmdLine = *argv.offset(i as isize);
    } else {
        lptCmdLine = core::ptr::addr_of_mut!(EmptyStr) as LPTSTR;
    }

    lpManifestName = lptDllName as *mut WCHAR;
    if GetFileAttributesW(lptDllName as LPCWSTR) == INVALID_FILE_ATTRIBUTES {
        let mut FilePart: LPWSTR = core::ptr::null_mut();
        if SearchPathW(
            core::ptr::null(),
            lptDllName as LPCWSTR,
            core::ptr::null(),
            (ResolvedFile.len() as DWORD).wrapping_sub(1 as DWORD),
            ResolvedFile.as_mut_ptr(),
            core::ptr::addr_of_mut!(FilePart),
        ) != 0
        {
            lpManifestName = ResolvedFile.as_mut_ptr();
        }
    }

    // FIXME: If there is a .manifest file next to the input file, we should use that instead!
    ActCtx.lpSource = lpManifestName as LPCWSTR;
    ActCtx.lpResourceName = (123 as usize) as LPCWSTR;
    hActCtx = CreateActCtx(core::ptr::addr_of_mut!(ActCtx));
    bActivated = if hActCtx != INVALID_HANDLE_VALUE {
        ActivateActCtx(hActCtx, core::ptr::addr_of_mut!(cookie))
    } else {
        (FALSE as i32)
    };

    // Everything is all setup, so load the dll now
    hDll = LoadLibrary(lptDllName as LPCWSTR);
    if !hDll.is_null() {
        nStrLen = _tcslen(lptFuncName as *const TCHAR);
        // Make a non-unicode version of the function name,
        // since that is all GetProcAddress accepts
        lpFuncName = DuplicateToMultiByte(
            lptFuncName as LPCTSTR,
            nStrLen.wrapping_add(2 as size_t),
        );

        if true {
            // UNICODE branch
            *lpFuncName.offset(nStrLen as isize) = b'W' as CHAR;
            *lpFuncName.offset(nStrLen.wrapping_add(1 as size_t) as isize) =
                0 as CHAR;
            // Get address of unicode version of the dll function if it exists
            fnDllWinMainW = core::mem::transmute::<FARPROC, DllWinMainW>(
                GetProcAddress(hDll as HINSTANCE, lpFuncName as LPCSTR),
            );
            fnDllWinMainA = None;
            if fnDllWinMainW.is_none() {
                // If no unicode function was found, get the address of the non-unicode function
                *lpFuncName.offset(nStrLen as isize) = b'A' as CHAR;
                fnDllWinMainA = core::mem::transmute::<FARPROC, DllWinMainA>(
                    GetProcAddress(hDll as HINSTANCE, lpFuncName as LPCSTR),
                );
                if fnDllWinMainA.is_none() {
                    // If first non-unicode function was not found, get the address
                    // of the other non-unicode function
                    *lpFuncName.offset(nStrLen as isize) = 0 as CHAR;
                    fnDllWinMainA = core::mem::transmute::<FARPROC, DllWinMainA>(
                        GetProcAddress(hDll as HINSTANCE, lpFuncName as LPCSTR),
                    );
                }
            }
        } else {
            // non-UNICODE branch
            // Get address of non-unicode version of the dll function if it exists
            fnDllWinMainA = core::mem::transmute::<FARPROC, DllWinMainA>(
                GetProcAddress(hDll as HINSTANCE, lpFuncName as LPCSTR),
            );
            fnDllWinMainW = None;
            if fnDllWinMainA.is_none() {
                // If first non-unicode function was not found, get the address
                // of the other non-unicode function
                *lpFuncName.offset(nStrLen as isize) = b'A' as CHAR;
                *lpFuncName.offset(nStrLen.wrapping_add(1 as size_t) as isize) =
                    0 as CHAR;
                fnDllWinMainA = core::mem::transmute::<FARPROC, DllWinMainA>(
                    GetProcAddress(hDll as HINSTANCE, lpFuncName as LPCSTR),
                );
                if fnDllWinMainA.is_none() {
                    // If non-unicode function was not found, get the address of the unicode function
                    *lpFuncName.offset(nStrLen as isize) = b'W' as CHAR;
                    fnDllWinMainW = core::mem::transmute::<FARPROC, DllWinMainW>(
                        GetProcAddress(hDll as HINSTANCE, lpFuncName as LPCSTR),
                    );
                }
            }
        }

        free(lpFuncName as *mut core::ffi::c_void);

        if RegisterBlankClass(hInstance, hPrevInstance) == 0 {
            FreeLibrary(hDll);
            if bActivated != 0 {
                DeactivateActCtx(0 as DWORD, cookie);
            }
            return 0;
        }
        // Create a window so we can pass a window handle to
        // the dll function; this is required
        hWindow = CreateWindowEx(
            0,
            rundll32_wclass,
            rundll32_wtitle,
            0,
            CW_USEDEFAULT,
            0,
            CW_USEDEFAULT,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            hInstance,
            core::ptr::null_mut(),
        );

        if fnDllWinMainW.is_some() {
            // Convert the command-line string to unicode and call the dll function
            lpwCmdLine = ConvertToWideChar!(lptCmdLine);
            fnDllWinMainW.unwrap_unchecked()(
                hWindow,
                hInstance,
                lpwCmdLine,
                nCmdShow,
            );
            FreeConvertedWideChar!(lpwCmdLine);
        } else if fnDllWinMainA.is_some() {
            // Convert the command-line string to ansi and call the dll function
            lpaCmdLine = ConvertToMultiByte!(lptCmdLine);
            fnDllWinMainA.unwrap_unchecked()(
                hWindow,
                hInstance,
                lpaCmdLine,
                nCmdShow,
            );
            FreeConvertedMultiByte!((lpaCmdLine).cast());
        } else {
            // The specified dll function was not found; display an error message
            GetModuleTitle();
            LoadString(
                GetModuleHandle(core::ptr::null_mut()),
                IDS_MissingEntry as UINT,
                szMsg.as_mut_ptr(),
                (RC_STRING_MAX_SIZE as i32),
            );

            lptMsgBuffer = malloc(
                (_tcslen(szMsg.as_mut_ptr() as *const TCHAR)
                    .wrapping_sub(4 as size_t))
                .wrapping_add(_tcslen(lptFuncName as *const TCHAR))
                .wrapping_add(_tcslen(lptDllName as *const TCHAR))
                .wrapping_add(1 as size_t)
                .wrapping_mul(core::mem::size_of::<TCHAR>() as size_t),
            ) as LPTSTR;
            _stprintf(lptMsgBuffer, szMsg.as_mut_ptr(), lptFuncName, lptDllName);
            MessageBox(
                core::ptr::null_mut(),
                lptMsgBuffer,
                ModuleTitle,
                MB_ICONERROR,
            );
            free(lptMsgBuffer as *mut core::ffi::c_void);
        }

        DestroyWindow(hWindow);
        UnregisterClass(rundll32_wclass, hInstance);

        // The dll function has finished executing, so unload it
        FreeLibrary(hDll);
    } else {
        // The dll could not be loaded; display an error message
        GetModuleTitle();
        LoadString(
            GetModuleHandle(core::ptr::null_mut()),
            IDS_DllNotLoaded as UINT,
            szMsg.as_mut_ptr(),
            (RC_STRING_MAX_SIZE as i32),
        );

        lptMsgBuffer = malloc(
            (_tcslen(szMsg.as_mut_ptr() as *const TCHAR)
                .wrapping_sub(2 as size_t))
            .wrapping_add(_tcslen(lptDllName as *const TCHAR))
            .wrapping_add(1 as size_t)
            .wrapping_mul(core::mem::size_of::<TCHAR>() as size_t),
        ) as LPTSTR;
        _stprintf(lptMsgBuffer, szMsg.as_mut_ptr(), lptDllName);

        MessageBox(
            core::ptr::null_mut(),
            lptMsgBuffer,
            ModuleTitle,
            MB_ICONERROR,
        );
        free(lptMsgBuffer as *mut core::ffi::c_void);
    }

    if bActivated != 0 {
        DeactivateActCtx(0 as DWORD, cookie);
    }

    if !argv.is_null() {
        free(argv as *mut core::ffi::c_void);
    }
    return 0; /* rundll32 always returns 0! */
}

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut szWindowClass: [TCHAR; 19] = unsafe {
    ::core::mem::transmute::<[u8; 38], [TCHAR; 19]>(
        *b"W\0e\0l\0c\0o\0m\0e\0W\0i\0n\0d\0o\0w\0C\0l\0a\0s\0s\0\0\0",
    )
};

#[no_mangle]
pub static mut szAppTitle: [TCHAR; 80] = [0; 80];

#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hWndMain: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hWndCheckButton: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hWndCloseButton: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut bDisplayCheckBox: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut bDisplayExitBtn: BOOL = (TRUE as i32);

#[no_mangle]
pub static mut dwNumberTopics: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut pTopics: *mut PTOPIC = ::core::ptr::null::<PTOPIC>() as *mut PTOPIC;

#[no_mangle]
pub static mut fnOldBtn: WNDPROC = None;

#[no_mangle]
pub static mut szDefaultTitle: [TCHAR; 80] = [0; 80];

#[no_mangle]
pub static mut szDefaultDesc: [TCHAR; 1024] = [0; 1024];

#[no_mangle]
pub static mut nTopic: INT = -(1 as INT);

#[no_mangle]
pub static mut nDefaultTopic: INT = -(1 as INT);

#[no_mangle]
pub static mut hdcMem: HDC = ::core::ptr::null::<HDC__>() as *mut HDC__;

#[no_mangle]
pub static mut hTitleBitmap: HBITMAP = ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__;

#[no_mangle]
pub static mut hDefaultTopicBitmap: HBITMAP = ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__;

#[no_mangle]
pub static mut hFontTopicButton: HFONT = ::core::ptr::null::<HFONT__>() as *mut HFONT__;

#[no_mangle]
pub static mut hFontTopicTitle: HFONT = ::core::ptr::null::<HFONT__>() as *mut HFONT__;

#[no_mangle]
pub static mut hFontTopicDescription: HFONT = ::core::ptr::null::<HFONT__>() as *mut HFONT__;

#[no_mangle]
pub static mut hFontCheckButton: HFONT = ::core::ptr::null::<HFONT__>() as *mut HFONT__;

#[no_mangle]
pub static mut hbrLightBlue: HBRUSH = ::core::ptr::null::<HBRUSH__>() as *mut HBRUSH__;

#[no_mangle]
pub static mut hbrDarkBlue: HBRUSH = ::core::ptr::null::<HBRUSH__>() as *mut HBRUSH__;

#[no_mangle]
pub static mut rcTitlePanel: RECT = tagRECT {
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
};

#[no_mangle]
pub static mut rcLeftPanel: RECT = tagRECT {
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
};

#[no_mangle]
pub static mut rcRightPanel: RECT = tagRECT {
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
};

// LIGHT_BLUE = RGB(214, 239, 247)
pub const LIGHT_BLUE: COLORREF = 214 | (239 << 8) | (247 << 16);

// DARK_BLUE = RGB(107, 123, 140)
pub const DARK_BLUE: COLORREF = 107 | (123 << 8) | (140 << 16);

pub const TITLE_WIDTH: i32 = 480;

pub const TITLE_HEIGHT: i32 = 93;

pub const BUFFER_SIZE: i32 = 1024;

pub const TOPIC_TITLE_LENGTH: usize = 80;

pub const TOPIC_DESC_LENGTH: usize = 1024;

#[repr(C)]
pub struct _TOPIC {
    pub hBitmap: HBITMAP,
    pub hWndButton: HWND,
    /*
     * TRUE : szCommand contains a command (e.g. executable to run);
     * FALSE: szCommand contains a custom "Welcome"/AutoRun action.
     */
    pub bIsCommand: BOOL,
    pub szText: [TCHAR; 80],
    pub szTitle: [TCHAR; TOPIC_TITLE_LENGTH],
    pub szDesc: [TCHAR; TOPIC_DESC_LENGTH],
    pub szCommand: [TCHAR; 512],
    pub szArgs: [TCHAR; 512],
}
pub type TOPIC = _TOPIC;
pub type PTOPIC = *mut TOPIC;

pub const TOPIC_BTN_ID_BASE: i32 = 100;

// MainWndProc prototype: INT_PTR __stdcall MainWndProc(HWND, UINT, WPARAM, LPARAM)
// Declared here as a prototype only; emitted with its function definition.
#[no_mangle]
pub unsafe extern "C" fn GetLocaleName(mut Locale: LCID, mut lpLCData: LPTSTR, mut cchData: SIZE_T) -> INT {
    let mut cchRet: INT = 0;
    let mut cchRet2: INT = 0;

    cchRet = GetLocaleInfo(Locale, LOCALE_SNAME, lpLCData, cchData as i32) as INT;
    if cchRet != 0 || (GetLastError() != ERROR_INVALID_FLAGS) {
        return cchRet;
    }

    cchRet = GetLocaleInfo(Locale, LOCALE_SISO639LANGNAME, lpLCData, cchData as i32) as INT;
    if cchRet <= 1 {
        return cchRet;
    }

    lpLCData = lpLCData.offset((cchRet - 1) as isize);
    cchData = cchData.wrapping_sub((cchRet - 1) as SIZE_T);
    if cchData <= 1 {
        return cchRet;
    }

    cchRet2 = GetLocaleInfo(
        Locale,
        LOCALE_SISO3166CTRYNAME,
        lpLCData.offset(1 as isize),
        cchData.wrapping_sub(1) as i32,
    ) as INT;
    if cchRet2 <= 1 {
        return cchRet;
    }
    cchRet += cchRet2;

    *lpLCData = ('-' as u16);

    return cchRet;
}
#[no_mangle]
pub unsafe extern "C" fn TranslateEscapes(mut lpString: LPTSTR) {
    let mut pEscape: LPTSTR = core::ptr::null_mut::<WCHAR>(); // Next backslash escape sequence.

    while !lpString.is_null() && *lpString != 0 {
        /* Find the next backslash escape sequence */
        pEscape = _tcschr(lpString as *const WCHAR, ('\\' as u16)) as LPTSTR;
        if pEscape.is_null() {
            break;
        }

        /* Go past the escape backslash */
        lpString = pEscape.add(1 as usize);

        /* Find which sequence it is */
        match *lpString as core::ffi::c_int {
            0 => {
                // *pEscape = ('\0' as u16); // Enable if one wants to convert \<NULL> into <NULL>.
                // lpString = pEscape + 1; // Loop will stop at the next iteration.
            }

            /* New-line and carriage return */
            110 | 114 => {
                if *lpString == ('n' as u16) as WCHAR {
                    *pEscape = ('\n' as u16) as WCHAR;
                } else if *lpString == ('r' as u16) as WCHAR {
                    *pEscape = ('\r' as u16) as WCHAR;
                }

                memmove(
                    lpString as *mut core::ffi::c_void,
                    lpString.add(1 as usize) as *const core::ffi::c_void,
                    ((_tcslen(lpString.add(1 as usize) as *const WCHAR) as size_t
                        + 1 as size_t)
                        * (core::mem::size_of::<TCHAR>() as size_t)),
                );
            }

            /* \xhhhh hexadecimal character specification */
            120 => {
                let mut lpStringNew: LPTSTR = core::ptr::null_mut::<WCHAR>();
                *pEscape = _tcstoul(
                    lpString.add(1 as usize) as *const WCHAR,
                    core::ptr::addr_of_mut!(lpStringNew),
                    16 as core::ffi::c_int,
                ) as WCHAR;
                memmove(
                    lpString as *mut core::ffi::c_void,
                    lpStringNew as *const core::ffi::c_void,
                    ((_tcslen(lpStringNew as *const WCHAR) as size_t
                        + 1 as size_t)
                        * (core::mem::size_of::<TCHAR>() as size_t)),
                );
            }

            /* Unknown escape sequence, ignore it */
            _ => {
                lpString = lpString.add(1);
            }
        }
    }
}

unsafe extern "C" fn ExpandInstallerPath(
    pInstallerName: PCTSTR,
    pInstallerPath: PTSTR,
    PathSize: SIZE_T,
) -> BOOL {
    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
    let mut cchInstallerNameLen: SIZE_T = 0;
    let mut ptr: PTSTR = core::ptr::null_mut();
    let mut dwAttribs: DWORD = 0;

    cchInstallerNameLen = _tcslen(pInstallerName) as SIZE_T;
    if PathSize < cchInstallerNameLen {
        /* The buffer is not large enough to contain the installer file name */
        *pInstallerPath = 0;
        return (FALSE) as BOOL;
    }

    /*
     * First, try to find the installer using the default drive, under
     * the directory whose name corresponds to the currently-running
     * CPU architecture.
     */
    GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));

    *pInstallerPath = 0;
    GetModuleFileName(
        core::ptr::null_mut(),
        pInstallerPath as LPWSTR,
        PathSize.wrapping_sub(cchInstallerNameLen).wrapping_sub(1) as DWORD,
    );
    ptr = _tcschr(pInstallerPath, ('\\' as u16)) as PTSTR;
    if !ptr.is_null() {
        ptr = ptr.add(1);
        *ptr = 0;
    } else {
        *pInstallerPath = 0;
    }

    /* Append the corresponding CPU architecture */
    match (SystemInfo


        .__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture as ::core::ffi::c_int) as u32
    {
        PROCESSOR_ARCHITECTURE_INTEL => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0049, 0x0033, 0x0038, 0x0036, 0x0000u16]).as_ptr() /* TEXT "I386" */);
        }
        PROCESSOR_ARCHITECTURE_MIPS => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x004d, 0x0049, 0x0050, 0x0053, 0x0000u16]).as_ptr() /* TEXT "MIPS" */);
        }
        PROCESSOR_ARCHITECTURE_ALPHA => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0041, 0x004c, 0x0050, 0x0048, 0x0041, 0x0000u16]).as_ptr() /* TEXT "ALPHA" */);
        }
        PROCESSOR_ARCHITECTURE_PPC => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0050, 0x0050, 0x0043, 0x0000u16]).as_ptr() /* TEXT "PPC" */);
        }
        PROCESSOR_ARCHITECTURE_SHX => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0053, 0x0048, 0x0058, 0x0000u16]).as_ptr() /* TEXT "SHX" */);
        }
        PROCESSOR_ARCHITECTURE_ARM => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0041, 0x0052, 0x004d, 0x0000u16]).as_ptr() /* TEXT "ARM" */);
        }
        PROCESSOR_ARCHITECTURE_IA64 => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0049, 0x0041, 0x0036, 0x0034, 0x0000u16]).as_ptr() /* TEXT "IA64" */);
        }
        PROCESSOR_ARCHITECTURE_ALPHA64 => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0041, 0x004c, 0x0050, 0x0048, 0x0041, 0x0036, 0x0034, 0x0000u16]).as_ptr() /* TEXT "ALPHA64" */);
        }
        PROCESSOR_ARCHITECTURE_AMD64 => {
            StringCchCat(pInstallerPath, (PathSize) as _, (&[0x0041, 0x004d, 0x0044, 0x0036, 0x0034, 0x0000u16]).as_ptr() /* TEXT "AMD64" */);
        }
        // case PROCESSOR_ARCHITECTURE_MSIL: /* .NET CPU-independent code */
        PROCESSOR_ARCHITECTURE_UNKNOWN | _ => {
            SystemInfo


                .__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture = PROCESSOR_ARCHITECTURE_UNKNOWN as WORD;
        }
    }

    if SystemInfo


        .__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture as ::core::ffi::c_int
        != (PROCESSOR_ARCHITECTURE_UNKNOWN as i32)
    {
        StringCchCat(pInstallerPath, (PathSize) as _, (&[0x005c, 0x0000u16]).as_ptr() /* TEXT "\\" */);
    }
    StringCchCat(pInstallerPath, (PathSize) as _, pInstallerName);

    dwAttribs = GetFileAttributes(pInstallerPath);
    if (dwAttribs != INVALID_FILE_ATTRIBUTES) && ((dwAttribs & FILE_ATTRIBUTE_DIRECTORY) == 0) {
        /* We have found the installer */
        return (TRUE) as BOOL;
    }

    /*
     * We failed. Try to find the installer from either the current
     * ReactOS installation directory, or from our current directory.
     */
    *pInstallerPath = 0;
    if GetWindowsDirectory(
        pInstallerPath,
        PathSize.wrapping_sub(cchInstallerNameLen).wrapping_sub(1) as UINT,
    ) != 0
    {
        StringCchCat(pInstallerPath, (PathSize) as _, (&[0x005c, 0x0000u16]).as_ptr() /* TEXT "\\" */);
    }
    StringCchCat(pInstallerPath, (PathSize) as _, pInstallerName);

    dwAttribs = GetFileAttributes(pInstallerPath);
    if (dwAttribs != INVALID_FILE_ATTRIBUTES) && ((dwAttribs & FILE_ATTRIBUTE_DIRECTORY) == 0) {
        /* We have found the installer */
        return (TRUE) as BOOL;
    }

    /* Installer not found */
    *pInstallerPath = 0;
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTopicList() {
    dwNumberTopics = 0;
    pTopics = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn AddNewTopic() -> PTOPIC {
    let mut pTopic: PTOPIC = core::ptr::null_mut::<_TOPIC>();
    let mut pTopicsTmp: *mut PTOPIC = core::ptr::null_mut::<PTOPIC>();

    /* Allocate (or reallocate) the list of topics */
    if pTopics.is_null() {
        pTopicsTmp = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (dwNumberTopics.wrapping_add(1 as DWORD) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<PTOPIC>() as SIZE_T),
        ) as *mut PTOPIC;
    } else {
        pTopicsTmp = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            pTopics as PVOID,
            (dwNumberTopics.wrapping_add(1 as DWORD) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<PTOPIC>() as SIZE_T),
        ) as *mut PTOPIC;
    }
    if pTopicsTmp.is_null() {
        return core::ptr::null_mut::<_TOPIC>(); // Cannot reallocate more
    }
    pTopics = pTopicsTmp;

    /* Allocate a new topic entry */
    pTopic = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<_TOPIC>() as SIZE_T,
    ) as PTOPIC;
    if pTopic.is_null() {
        return core::ptr::null_mut::<_TOPIC>(); // Cannot reallocate more
    }
    let idx = dwNumberTopics;
    dwNumberTopics = dwNumberTopics.wrapping_add(1);
    *pTopics.offset(idx as isize) = pTopic;

    /* Return the allocated topic entry */
    return pTopic;
}
#[no_mangle]
pub unsafe extern "C" fn AddNewTopicEx(
    szText: LPTSTR,
    szTitle: LPTSTR,
    szDesc: LPTSTR,
    szCommand: LPTSTR,
    szArgs: LPTSTR,
    szAction: LPTSTR,
) -> PTOPIC {
    let mut pTopic: PTOPIC = AddNewTopic();
    if pTopic.is_null() {
        return core::ptr::null_mut();
    }

    if !szText.is_null() && *szText != 0 {
        StringCchCopy(
            &raw mut (*pTopic).szText as LPTSTR,
            (ARRAYSIZE!((*pTopic).szText)) as _,
            szText,
        );
    } else {
        *(&raw mut (*pTopic).szText as *mut TCHAR) = 0 as TCHAR;
    }

    if !szTitle.is_null() && *szTitle != 0 {
        StringCchCopy(
            &raw mut (*pTopic).szTitle as LPTSTR,
            (ARRAYSIZE!((*pTopic).szTitle)) as _,
            szTitle,
        );
    } else {
        *(&raw mut (*pTopic).szTitle as *mut TCHAR) = 0 as TCHAR;
    }

    if !szDesc.is_null() && *szDesc != 0 {
        StringCchCopy(
            &raw mut (*pTopic).szDesc as LPTSTR,
            (ARRAYSIZE!((*pTopic).szDesc)) as _,
            szDesc,
        );
        TranslateEscapes(&raw mut (*pTopic).szDesc as LPTSTR);
    } else {
        *(&raw mut (*pTopic).szDesc as *mut TCHAR) = 0 as TCHAR;
    }

    if !szCommand.is_null() && *szCommand != 0 {
        (*pTopic).bIsCommand = TRUE as BOOL;

        /* Check for special applications: ReactOS Installer */
        if _tcsicmp(
            szCommand,
            [
                    'r' as u16, 'e' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'o' as u16,
                    's' as u16, '.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0u16,
                ]
                .as_ptr(),
        ) == 0
        {
            ExpandInstallerPath(
                szCommand,
                &raw mut (*pTopic).szCommand as LPTSTR,
                (ARRAYSIZE!((*pTopic).szCommand)) as SIZE_T,
            );
        } else {
            /* Expand any environment string in the command line */
            let mut dwSize: DWORD = ExpandEnvironmentStringsW(
                szCommand,
                core::ptr::null_mut(),
                0,
            );
            if dwSize <= ARRAYSIZE!((*pTopic).szCommand) as DWORD {
                ExpandEnvironmentStringsW(
                    szCommand,
                    &raw mut (*pTopic).szCommand as LPTSTR,
                    ARRAYSIZE!((*pTopic).szCommand),
                );
            } else {
                StringCchCopy(
                    &raw mut (*pTopic).szCommand as LPTSTR,
                    (ARRAYSIZE!((*pTopic).szCommand)) as _,
                    szCommand,
                );
            }
        }
    } else {
        (*pTopic).bIsCommand = FALSE as BOOL;
        *(&raw mut (*pTopic).szCommand as *mut TCHAR) = 0 as TCHAR;
    }

    /* Only care about command arguments if we actually have a command */
    if *(&raw mut (*pTopic).szCommand as *mut TCHAR) != 0 {
        if !szArgs.is_null() && *szArgs != 0 {
            StringCchCopy(
                &raw mut (*pTopic).szArgs as LPTSTR,
                (ARRAYSIZE!((*pTopic).szArgs)) as _,
                szArgs,
            );
        } else {
            /* Check for special applications: ReactOS Shell */
            if /* pTopic->szCommand && */ *(&raw mut (*pTopic).szCommand as *mut TCHAR) != 0
                && _tcsicmp(
                    &raw mut (*pTopic).szCommand as *mut TCHAR,
                    [
                            'e' as u16, 'x' as u16, 'p' as u16, 'l' as u16, 'o' as u16,
                            'r' as u16, 'e' as u16, 'r' as u16, '.' as u16, 'e' as u16,
                            'x' as u16, 'e' as u16, 0u16,
                        ]
                        .as_ptr(),
                ) == 0
            {
                StringCchCopy(
                    &raw mut (*pTopic).szArgs as LPTSTR,
                    (ARRAYSIZE!((*pTopic).szArgs)) as _,
                    (['\\' as u16, 0u16].as_ptr()),
                );
            } else {
                *(&raw mut (*pTopic).szArgs as *mut TCHAR) = 0 as TCHAR;
            }
        }
    } else {
        *(&raw mut (*pTopic).szArgs as *mut TCHAR) = 0 as TCHAR;
    }

    /* Only care about custom actions if we actually don't have a command */
    if *(&raw mut (*pTopic).szCommand as *mut TCHAR) == 0 && !szAction.is_null() && *szAction != 0 {
        /*
         * Re-use the pTopic->szCommand member. We distinguish with respect to
         * a regular command by using the pTopic->bIsCommand flag.
         */
        (*pTopic).bIsCommand = FALSE as BOOL;
        StringCchCopy(
            &raw mut (*pTopic).szCommand as LPTSTR,
            (ARRAYSIZE!((*pTopic).szCommand)) as _,
            szAction,
        );
        TranslateEscapes(&raw mut (*pTopic).szCommand as LPTSTR);
    }

    return pTopic;
}

unsafe extern "C" fn LoadLocalizedResourcesInternal() {
    let mut i: UINT;
    let mut lpszCommand: LPTSTR;
    let mut lpszAction: LPTSTR;
    let mut newTopic: TOPIC = TOPIC {
        hBitmap: core::ptr::null_mut(),
        hWndButton: core::ptr::null_mut(),
        bIsCommand: 0,
        szText: [0; 80],
        szTitle: [0; TOPIC_TITLE_LENGTH],
        szDesc: [0; TOPIC_DESC_LENGTH],
        szCommand: [0; 512],
        szArgs: [0; 512],
    };
    let mut pTopic: *mut TOPIC;

    i = 0;
    while i < (MAX_NUMBER_INTERNAL_TOPICS as u32) {
        lpszCommand = core::ptr::null_mut();
        lpszAction = core::ptr::null_mut();

        /* Retrieve the information */
        if LoadString(hInstance, IDS_TOPIC_BUTTON0 + i, &raw mut newTopic.szText as LPWSTR, (ARRAYSIZE!(newTopic.szText)) as i32) == 0 {
            *(&raw mut newTopic.szText as *mut TCHAR) = 0 as TCHAR;
        }
        if LoadString(hInstance, IDS_TOPIC_TITLE0 + i, &raw mut newTopic.szTitle as LPWSTR, (ARRAYSIZE!(newTopic.szTitle)) as i32) == 0 {
            *(&raw mut newTopic.szTitle as *mut TCHAR) = 0 as TCHAR;
        }
        if LoadString(hInstance, IDS_TOPIC_DESC0 + i, &raw mut newTopic.szDesc as LPWSTR, (ARRAYSIZE!(newTopic.szDesc)) as i32) == 0 {
            *(&raw mut newTopic.szDesc as *mut TCHAR) = 0 as TCHAR;
        }

        if LoadString(hInstance, IDS_TOPIC_COMMAND0 + i, &raw mut newTopic.szCommand as LPWSTR, (ARRAYSIZE!(newTopic.szCommand)) as i32) == 0 {
            *(&raw mut newTopic.szCommand as *mut TCHAR) = 0 as TCHAR;
        }

        /* Only care about command arguments if we actually have a command */
        if *(&raw mut newTopic.szCommand as *mut TCHAR) != 0 {
            lpszCommand = &raw mut newTopic.szCommand as *mut TCHAR as LPTSTR;
            if LoadString(hInstance, IDS_TOPIC_CMD_ARGS0 + i, &raw mut newTopic.szArgs as LPWSTR, (ARRAYSIZE!(newTopic.szArgs)) as i32) == 0 {
                *(&raw mut newTopic.szArgs as *mut TCHAR) = 0 as TCHAR;
            }
        }
        /* Only care about custom actions if we actually don't have a command */
        else {
            lpszAction = &raw mut newTopic.szCommand as *mut TCHAR as LPTSTR;
            if LoadString(hInstance, IDS_TOPIC_ACTION0 + i, &raw mut newTopic.szCommand as LPWSTR, (ARRAYSIZE!(newTopic.szCommand)) as i32) == 0 {
                *(&raw mut newTopic.szCommand as *mut TCHAR) = 0 as TCHAR;
            }
        }

        /* Allocate a new topic */
        pTopic = AddNewTopicEx(
            &raw mut newTopic.szText as LPTSTR,
            &raw mut newTopic.szTitle as LPTSTR,
            &raw mut newTopic.szDesc as LPTSTR,
            lpszCommand,
            &raw mut newTopic.szArgs as LPTSTR,
            lpszAction,
        ) as *mut TOPIC;
        if pTopic.is_null() {
            break; // Cannot reallocate more
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn LoadLocalizedResourcesFromINI(Locale: LCID, lpResPath: LPTSTR) -> BOOL {
    let mut dwRet: DWORD = 0;
    let mut dwAttribs: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut szBuffer: [TCHAR; (LOCALE_NAME_MAX_LENGTH as _)] = [0; (LOCALE_NAME_MAX_LENGTH as _)];
    let mut szIniPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpszSections: LPTSTR = core::ptr::null_mut();
    let mut lpszSection: LPTSTR = core::ptr::null_mut();
    let mut lpszCommand: LPTSTR = core::ptr::null_mut();
    let mut lpszAction: LPTSTR = core::ptr::null_mut();
    let mut newTopic: TOPIC = core::mem::zeroed();
    let mut pTopic: *mut TOPIC = core::ptr::null_mut();

    /* Retrieve the locale name (on which the INI file name is based) */
    dwRet = GetLocaleName(
        Locale,
        core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
        (ARRAYSIZE!(szBuffer)) as SIZE_T,
    ) as DWORD;
    if dwRet == 0 {
        /* Fall back to english (US) */
        StringCchCopy(
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
            (ARRAYSIZE!(szBuffer)) as _,
            ['e' as TCHAR, 'n' as TCHAR, '-' as TCHAR, 'U' as TCHAR, 'S' as TCHAR, 0 as TCHAR]
                .as_ptr() as LPCTSTR,
        );
    }

    /* Build the INI file name */
    StringCchPrintf!(
        core::ptr::addr_of_mut!(szIniPath) as LPTSTR,
        ARRAYSIZE!(szIniPath),
        ['%' as TCHAR, 's' as TCHAR, '\\' as TCHAR, '%' as TCHAR, 's' as TCHAR, '.' as TCHAR,
         'i' as TCHAR, 'n' as TCHAR, 'i' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
        lpResPath,
        core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
    );

    /* Verify that the file exists, otherwise fall back to english (US) */
    dwAttribs = GetFileAttributes(core::ptr::addr_of_mut!(szIniPath) as LPCTSTR);
    if (dwAttribs == INVALID_FILE_ATTRIBUTES)
        || ((dwAttribs & FILE_ATTRIBUTE_DIRECTORY) != 0)
    {
        StringCchCopy(
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
            (ARRAYSIZE!(szBuffer)) as _,
            ['e' as TCHAR, 'n' as TCHAR, '-' as TCHAR, 'U' as TCHAR, 'S' as TCHAR, 0 as TCHAR]
                .as_ptr() as LPCTSTR,
        );

        StringCchPrintf!(
            core::ptr::addr_of_mut!(szIniPath) as LPTSTR,
            ARRAYSIZE!(szIniPath),
            ['%' as TCHAR, 's' as TCHAR, '\\' as TCHAR, '%' as TCHAR, 's' as TCHAR, '.' as TCHAR,
             'i' as TCHAR, 'n' as TCHAR, 'i' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
            lpResPath,
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
        );
    }

    /* Verify that the file exists, otherwise fall back to internal (localized) resource */
    dwAttribs = GetFileAttributes(core::ptr::addr_of_mut!(szIniPath) as LPCTSTR);
    if (dwAttribs == INVALID_FILE_ATTRIBUTES)
        || ((dwAttribs & FILE_ATTRIBUTE_DIRECTORY) != 0)
    {
        return (FALSE) as BOOL; // For localized resources, see the general function.
    }

    /* Try to load the default localized strings */
    GetPrivateProfileString(
        ['D' as TCHAR, 'e' as TCHAR, 'f' as TCHAR, 'a' as TCHAR, 'u' as TCHAR, 'l' as TCHAR,
         't' as TCHAR, 's' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
        ['A' as TCHAR, 'p' as TCHAR, 'p' as TCHAR, 'T' as TCHAR, 'i' as TCHAR, 't' as TCHAR,
         'l' as TCHAR, 'e' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
        ['R' as TCHAR, 'e' as TCHAR, 'a' as TCHAR, 'c' as TCHAR, 't' as TCHAR, 'O' as TCHAR,
         'S' as TCHAR, ' ' as TCHAR, '-' as TCHAR, ' ' as TCHAR, 'W' as TCHAR, 'e' as TCHAR,
         'l' as TCHAR, 'c' as TCHAR, 'o' as TCHAR, 'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR]
            .as_ptr() as LPCTSTR, /* default */
        core::ptr::addr_of_mut!(szAppTitle) as LPTSTR,
        ARRAYSIZE!(szAppTitle),
        core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
    );
    GetPrivateProfileString(
        ['D' as TCHAR, 'e' as TCHAR, 'f' as TCHAR, 'a' as TCHAR, 'u' as TCHAR, 'l' as TCHAR,
         't' as TCHAR, 's' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
        ['D' as TCHAR, 'e' as TCHAR, 'f' as TCHAR, 'a' as TCHAR, 'u' as TCHAR, 'l' as TCHAR,
         't' as TCHAR, 'T' as TCHAR, 'o' as TCHAR, 'p' as TCHAR, 'i' as TCHAR, 'c' as TCHAR,
         'T' as TCHAR, 'i' as TCHAR, 't' as TCHAR, 'l' as TCHAR, 'e' as TCHAR, 0 as TCHAR]
            .as_ptr() as LPCTSTR,
        [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
        core::ptr::addr_of_mut!(szDefaultTitle) as LPTSTR,
        ARRAYSIZE!(szDefaultTitle),
        core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
    );
    if GetPrivateProfileString(
        ['D' as TCHAR, 'e' as TCHAR, 'f' as TCHAR, 'a' as TCHAR, 'u' as TCHAR, 'l' as TCHAR,
         't' as TCHAR, 's' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
        ['D' as TCHAR, 'e' as TCHAR, 'f' as TCHAR, 'a' as TCHAR, 'u' as TCHAR, 'l' as TCHAR,
         't' as TCHAR, 'T' as TCHAR, 'o' as TCHAR, 'p' as TCHAR, 'i' as TCHAR, 'c' as TCHAR,
         'D' as TCHAR, 'e' as TCHAR, 's' as TCHAR, 'c' as TCHAR, 'r' as TCHAR, 'i' as TCHAR,
         'p' as TCHAR, 't' as TCHAR, 'i' as TCHAR, 'o' as TCHAR, 'n' as TCHAR, 0 as TCHAR]
            .as_ptr() as LPCTSTR,
        [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
        core::ptr::addr_of_mut!(szDefaultDesc) as LPTSTR,
        ARRAYSIZE!(szDefaultDesc),
        core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
    ) == 0
    {
        *(core::ptr::addr_of_mut!(szDefaultDesc) as *mut TCHAR) = 0 as TCHAR;
    } else {
        TranslateEscapes(core::ptr::addr_of_mut!(szDefaultDesc) as LPTSTR);
    }

    /* Allocate a buffer big enough to hold all the section names */
    dwSize = BUFFER_SIZE as DWORD;
    loop {
        lpszSections = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (dwSize as SIZE_T).wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
        ) as LPTSTR;
        if lpszSections.is_null() {
            return TRUE as BOOL; // FIXME!
        }
        dwRet = GetPrivateProfileSectionNames(
            lpszSections,
            dwSize,
            core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
        );
        if dwRet < dwSize.wrapping_sub(2 as DWORD) {
            break;
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpszSections as PVOID,
        );
        dwSize = dwSize.wrapping_add(BUFFER_SIZE as DWORD);
    }

    /* Loop over the sections and load the topics */
    lpszSection = lpszSections;
    'sections: while !lpszSection.is_null() && *lpszSection != 0 {
        'body: {
            /* Ignore everything that is not a topic */
            if _tcsnicmp(
                lpszSection,
                ['T' as TCHAR, 'o' as TCHAR, 'p' as TCHAR, 'i' as TCHAR, 'c' as TCHAR, 0 as TCHAR]
                    .as_ptr() as LPCTSTR,
                (5 as SIZE_T) as _,
            ) != 0 as INT
            {
                break 'body; // continue
            }

            lpszCommand = core::ptr::null_mut();
            lpszAction = core::ptr::null_mut();

            /* Retrieve the information */
            GetPrivateProfileString(
                lpszSection,
                ['M' as TCHAR, 'e' as TCHAR, 'n' as TCHAR, 'u' as TCHAR, 'T' as TCHAR,
                 'e' as TCHAR, 'x' as TCHAR, 't' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
                [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
                core::ptr::addr_of_mut!(newTopic.szText) as LPTSTR,
                ARRAYSIZE!(newTopic.szText),
                core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
            );
            GetPrivateProfileString(
                lpszSection,
                ['T' as TCHAR, 'i' as TCHAR, 't' as TCHAR, 'l' as TCHAR, 'e' as TCHAR, 0 as TCHAR]
                    .as_ptr() as LPCTSTR,
                [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
                core::ptr::addr_of_mut!(newTopic.szTitle) as LPTSTR,
                ARRAYSIZE!(newTopic.szTitle),
                core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
            );
            GetPrivateProfileString(
                lpszSection,
                ['D' as TCHAR, 'e' as TCHAR, 's' as TCHAR, 'c' as TCHAR, 'r' as TCHAR,
                 'i' as TCHAR, 'p' as TCHAR, 't' as TCHAR, 'i' as TCHAR, 'o' as TCHAR,
                 'n' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
                [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
                core::ptr::addr_of_mut!(newTopic.szDesc) as LPTSTR,
                ARRAYSIZE!(newTopic.szDesc),
                core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
            );

            GetPrivateProfileString(
                lpszSection,
                ['C' as TCHAR, 'o' as TCHAR, 'n' as TCHAR, 'f' as TCHAR, 'i' as TCHAR,
                 'g' as TCHAR, 'C' as TCHAR, 'o' as TCHAR, 'm' as TCHAR, 'm' as TCHAR,
                 'a' as TCHAR, 'n' as TCHAR, 'd' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
                [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
                core::ptr::addr_of_mut!(newTopic.szCommand) as LPTSTR,
                ARRAYSIZE!(newTopic.szCommand),
                core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
            );

            /* Only care about command arguments if we actually have a command */
            if *(core::ptr::addr_of_mut!(newTopic.szCommand) as *mut TCHAR) != 0 {
                lpszCommand = core::ptr::addr_of_mut!(newTopic.szCommand) as LPTSTR;
                GetPrivateProfileString(
                    lpszSection,
                    ['C' as TCHAR, 'o' as TCHAR, 'n' as TCHAR, 'f' as TCHAR, 'i' as TCHAR,
                     'g' as TCHAR, 'A' as TCHAR, 'r' as TCHAR, 'g' as TCHAR, 's' as TCHAR,
                     0 as TCHAR].as_ptr() as LPCTSTR,
                    [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
                    core::ptr::addr_of_mut!(newTopic.szArgs) as LPTSTR,
                    ARRAYSIZE!(newTopic.szArgs),
                    core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
                );
            }
            /* Only care about custom actions if we actually don't have a command */
            else
            // if (!*newTopic.szCommand)
            {
                lpszAction = core::ptr::addr_of_mut!(newTopic.szCommand) as LPTSTR;
                GetPrivateProfileString(
                    lpszSection,
                    ['A' as TCHAR, 'c' as TCHAR, 't' as TCHAR, 'i' as TCHAR, 'o' as TCHAR,
                     'n' as TCHAR, 0 as TCHAR].as_ptr() as LPCTSTR,
                    [0 as TCHAR].as_ptr() as LPCTSTR, /* default */
                    core::ptr::addr_of_mut!(newTopic.szCommand) as LPTSTR,
                    ARRAYSIZE!(newTopic.szCommand),
                    core::ptr::addr_of_mut!(szIniPath) as LPCTSTR,
                );
            }

            /* Allocate a new topic */
            pTopic = AddNewTopicEx(
                core::ptr::addr_of_mut!(newTopic.szText) as LPTSTR,
                core::ptr::addr_of_mut!(newTopic.szTitle) as LPTSTR,
                core::ptr::addr_of_mut!(newTopic.szDesc) as LPTSTR,
                lpszCommand,
                core::ptr::addr_of_mut!(newTopic.szArgs) as LPTSTR,
                lpszAction,
            ) as *mut TOPIC;
            if pTopic.is_null() {
                break 'sections; // Cannot reallocate more
            }
        }
        lpszSection = lpszSection.offset(
            _tcslen(lpszSection).wrapping_add((1 as SIZE_T) as _)
                as isize,
        );
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        lpszSections as PVOID,
    );

    return TRUE as BOOL;
}

unsafe extern "C" fn LoadConfiguration() {
    let mut dwAttribs: DWORD = 0;
    let mut bLoadDefaultResources: BOOL = 0;
    let mut szAppPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szIniPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szResPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Initialize the topic list */
    InitializeTopicList();

    /*
     * First, try to load the default internal (localized) strings.
     * They can be redefined by the localized INI files.
     */
    if LoadString(hInstance, IDS_APPTITLE, addr_of_mut!(szAppTitle) as LPTSTR, (ARRAYSIZE!(szAppTitle)) as i32) == 0 {
        StringCchCopy(
            addr_of_mut!(szAppTitle) as LPTSTR,
            (ARRAYSIZE!(szAppTitle)) as _,
            [
                'R' as TCHAR, 'e' as TCHAR, 'a' as TCHAR, 'c' as TCHAR, 't' as TCHAR,
                'O' as TCHAR, 'S' as TCHAR, ' ' as TCHAR, '-' as TCHAR, ' ' as TCHAR,
                'W' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'c' as TCHAR, 'o' as TCHAR,
                'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR,
            ]
            .as_ptr(),
        );
    }
    if LoadString(hInstance, IDS_DEFAULT_TOPIC_TITLE, addr_of_mut!(szDefaultTitle) as LPTSTR, (ARRAYSIZE!(szDefaultTitle)) as i32) == 0 {
        *(addr_of_mut!(szDefaultTitle) as *mut TCHAR) = 0 as TCHAR;
    }
    if LoadString(hInstance, IDS_DEFAULT_TOPIC_DESC, addr_of_mut!(szDefaultDesc) as LPTSTR, (ARRAYSIZE!(szDefaultDesc)) as i32) == 0 {
        *(addr_of_mut!(szDefaultDesc) as *mut TCHAR) = 0 as TCHAR;
    }

    /* Retrieve the full path to this application */
    GetModuleFileName(core::ptr::null_mut(), addr_of_mut!(szAppPath) as LPTSTR, ARRAYSIZE!(szAppPath) as DWORD);
    if *(addr_of_mut!(szAppPath) as *mut TCHAR) != 0 {
        let mut lpFileName: LPTSTR = _tcsrchr(addr_of_mut!(szAppPath) as LPTSTR, '\\' as TCHAR) as LPTSTR;
        if !lpFileName.is_null() {
            *lpFileName = 0 as TCHAR;
        } else {
            *(addr_of_mut!(szAppPath) as *mut TCHAR) = 0 as TCHAR;
        }
    }

    /* Build the full INI file path name */
    StringCchPrintf!(
        addr_of_mut!(szIniPath) as LPTSTR,
        ARRAYSIZE!(szIniPath),
        [
            '%' as TCHAR, 's' as TCHAR, '\\' as TCHAR, 'w' as TCHAR, 'e' as TCHAR,
            'l' as TCHAR, 'c' as TCHAR, 'o' as TCHAR, 'm' as TCHAR, 'e' as TCHAR,
            '.' as TCHAR, 'i' as TCHAR, 'n' as TCHAR, 'i' as TCHAR, 0 as TCHAR,
        ]
        .as_ptr(),
        addr_of_mut!(szAppPath) as LPTSTR,
    );

    /* Verify that the file exists, otherwise use the default configuration */
    dwAttribs = GetFileAttributes(addr_of_mut!(szIniPath) as LPTSTR);
    if (dwAttribs == INVALID_FILE_ATTRIBUTES as DWORD) ||
        ((dwAttribs & FILE_ATTRIBUTE_DIRECTORY as DWORD) != 0)
    {
        /* Use the default internal (localized) resources */
        LoadLocalizedResourcesInternal();
        return;
    }

    /* Load the settings from the INI configuration file */
    bDisplayCheckBox = (GetPrivateProfileInt(
        [
            'W' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'c' as TCHAR, 'o' as TCHAR,
            'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR,
        ]
        .as_ptr(),
        [
            'D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'p' as TCHAR, 'l' as TCHAR,
            'a' as TCHAR, 'y' as TCHAR, 'C' as TCHAR, 'h' as TCHAR, 'e' as TCHAR,
            'c' as TCHAR, 'k' as TCHAR, 'B' as TCHAR, 'o' as TCHAR, 'x' as TCHAR,
            0 as TCHAR,
        ]
        .as_ptr(),
        FALSE as INT, /* default */
        addr_of_mut!(szIniPath) as LPTSTR,
    ) != 0) as BOOL;
    bDisplayExitBtn = (GetPrivateProfileInt(
        [
            'W' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'c' as TCHAR, 'o' as TCHAR,
            'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR,
        ]
        .as_ptr(),
        [
            'D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'p' as TCHAR, 'l' as TCHAR,
            'a' as TCHAR, 'y' as TCHAR, 'E' as TCHAR, 'x' as TCHAR, 'i' as TCHAR,
            't' as TCHAR, 'B' as TCHAR, 'u' as TCHAR, 't' as TCHAR, 't' as TCHAR,
            'o' as TCHAR, 'n' as TCHAR, 0 as TCHAR,
        ]
        .as_ptr(),
        TRUE as INT, /* default */
        addr_of_mut!(szIniPath) as LPTSTR,
    ) != 0) as BOOL;

    /* Load the default internal (localized) resources if needed */
    bLoadDefaultResources = (GetPrivateProfileInt(
        [
            'W' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'c' as TCHAR, 'o' as TCHAR,
            'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR,
        ]
        .as_ptr(),
        [
            'L' as TCHAR, 'o' as TCHAR, 'a' as TCHAR, 'd' as TCHAR, 'D' as TCHAR,
            'e' as TCHAR, 'f' as TCHAR, 'a' as TCHAR, 'u' as TCHAR, 'l' as TCHAR,
            't' as TCHAR, 'R' as TCHAR, 'e' as TCHAR, 's' as TCHAR, 'o' as TCHAR,
            'u' as TCHAR, 'r' as TCHAR, 'c' as TCHAR, 'e' as TCHAR, 's' as TCHAR,
            0 as TCHAR,
        ]
        .as_ptr(),
        FALSE as INT, /* default */
        addr_of_mut!(szIniPath) as LPTSTR,
    ) != 0) as BOOL;
    if bLoadDefaultResources != 0 {
        LoadLocalizedResourcesInternal();
    }

    GetPrivateProfileString(
        [
            'W' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'c' as TCHAR, 'o' as TCHAR,
            'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR,
        ]
        .as_ptr(),
        [
            'R' as TCHAR, 'e' as TCHAR, 's' as TCHAR, 'o' as TCHAR, 'u' as TCHAR,
            'r' as TCHAR, 'c' as TCHAR, 'e' as TCHAR, 'D' as TCHAR, 'i' as TCHAR,
            'r' as TCHAR, 0 as TCHAR,
        ]
        .as_ptr(),
        [0 as TCHAR].as_ptr(), /* default */
        addr_of_mut!(szResPath) as LPTSTR,
        ARRAYSIZE!(szResPath) as DWORD,
        addr_of_mut!(szIniPath) as LPTSTR,
    );

    /* Set the current directory to the one of this application, and retrieve the resources */
    SetCurrentDirectory(addr_of_mut!(szAppPath) as LPTSTR);
    if LoadLocalizedResourcesFromINI(LOCALE_USER_DEFAULT, addr_of_mut!(szResPath) as LPTSTR) == 0 {
        /*
         * Loading localized resources from INI file failed, try to load the
         * internal resources only if they were not already loaded earlier.
         */
        if bLoadDefaultResources == 0 {
            LoadLocalizedResourcesInternal();
        }
    }
}

unsafe extern "C" fn FreeResources() {
    if pTopics.is_null() {
        return;
    }
    loop {
        let fresh15 = dwNumberTopics;
        dwNumberTopics = dwNumberTopics.wrapping_sub(1);
        if !(fresh15 != 0) {
            break;
        }
        if !(*pTopics.offset(dwNumberTopics as isize)).is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                *pTopics.offset(dwNumberTopics as isize) as PVOID,
            );
        }
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pTopics as PVOID,
    );
    pTopics = ::core::ptr::null_mut::<PTOPIC>();
    dwNumberTopics = 0 as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInst: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpszCmdLine: LPTSTR,
    nCmdShow: INT,
) -> INT {
    let mut hMutex: HANDLE = core::ptr::null_mut();
    let mut wndclass: WNDCLASSEX = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let mut hWndFocus: HWND = core::ptr::null_mut();
    let mut xPos: INT = 0;
    let mut yPos: INT = 0;
    let mut xWidth: INT = 0;
    let mut yHeight: INT = 0;
    let mut rcWindow: RECT = core::mem::zeroed();
    let mut hMainIcon: HICON = core::ptr::null_mut();
    let mut hSystemMenu: HMENU = core::ptr::null_mut();
    let mut dwStyle: DWORD = (WS_OVERLAPPED | WS_CLIPCHILDREN | WS_CLIPSIBLINGS |
                    WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX) as DWORD;

    let mut BitmapInfo: BITMAP = core::mem::zeroed();
    let mut ulInnerWidth: ULONG = TITLE_WIDTH as ULONG;
    let mut ulInnerHeight: ULONG = ((TITLE_WIDTH * 3) / 4) as ULONG;
    let mut ulTitleHeight: ULONG = (TITLE_HEIGHT + 3) as ULONG;

    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(lpszCmdLine);

    /* Ensure only one instance is running */
    hMutex = CreateMutex(core::ptr::null_mut(), (FALSE as i32), addr_of_mut!(szWindowClass) as *mut TCHAR as LPCWSTR);
    if !hMutex.is_null() && (GetLastError() == ERROR_ALREADY_EXISTS as DWORD) {
        /* If already started, find its window */
        hWndMain = FindWindow(addr_of_mut!(szWindowClass) as *mut TCHAR as LPCWSTR, core::ptr::null());

        /* Activate window */
        ShowWindow(hWndMain, SW_SHOWNORMAL as INT);
        SetForegroundWindow(hWndMain);

        /* Close the mutex handle and quit */
        CloseHandle(hMutex);
        return 0 as INT;
    }

    hInstance = hInst;

    /* Load icons */
    hMainIcon = LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_MAIN) as LPCWSTR);

    /* Register the window class */
    wndclass.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    wndclass.style = (CS_HREDRAW | CS_VREDRAW) as UINT;
    wndclass.lpfnWndProc = core::mem::transmute::<
        Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
        WNDPROC,
    >(Some(
        MainWndProc,
    ));
    wndclass.cbClsExtra = 0 as INT;
    wndclass.cbWndExtra = 0 as INT;
    wndclass.hInstance = hInstance;
    wndclass.hIcon = hMainIcon;
    wndclass.hIconSm = core::ptr::null_mut();
    wndclass.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW as LPCWSTR);
    wndclass.hbrBackground = core::ptr::null_mut();
    wndclass.lpszMenuName = core::ptr::null();
    wndclass.lpszClassName = addr_of_mut!(szWindowClass) as *mut TCHAR as LPCWSTR;

    RegisterClassEx(addr_of_mut!(wndclass));

    /* Load the banner bitmap, and compute the window dimensions */
    hTitleBitmap = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_TITLE_BITMAP) as LPCWSTR);
    if !hTitleBitmap.is_null() {
        GetObject(hTitleBitmap as HANDLE, core::mem::size_of::<BITMAP>() as INT, addr_of_mut!(BitmapInfo) as LPVOID);
        ulInnerWidth = BitmapInfo.bmWidth as ULONG;
        ulInnerHeight = ulInnerWidth.wrapping_mul(3 as ULONG).wrapping_div(4 as ULONG);
        ulTitleHeight = (BitmapInfo.bmHeight + 3 as LONG) as ULONG;
        DeleteObject(hTitleBitmap as HGDIOBJ);
    }
    ulInnerHeight = ulInnerHeight.wrapping_sub(GetSystemMetrics(SM_CYCAPTION as INT) as ULONG);

    rcWindow.top = 0 as LONG;
    rcWindow.bottom = ulInnerHeight.wrapping_sub(1 as ULONG) as LONG;
    rcWindow.left = 0 as LONG;
    rcWindow.right = ulInnerWidth.wrapping_sub(1 as ULONG) as LONG;

    AdjustWindowRect(addr_of_mut!(rcWindow), dwStyle, FALSE as BOOL);
    xWidth = (rcWindow.right - rcWindow.left) as INT;
    yHeight = (rcWindow.bottom - rcWindow.top) as INT;

    /* Compute the window position */
    xPos = ((GetSystemMetrics((SM_CXSCREEN as i32)) - xWidth) / 2) as INT;
    yPos = ((GetSystemMetrics((SM_CYSCREEN as i32)) - yHeight) / 2) as INT;

    rcTitlePanel.top = 0 as LONG;
    rcTitlePanel.bottom = ulTitleHeight as LONG;
    rcTitlePanel.left = 0 as LONG;
    rcTitlePanel.right = ulInnerWidth.wrapping_sub(1 as ULONG) as LONG;

    rcLeftPanel.top = rcTitlePanel.bottom;
    rcLeftPanel.bottom = ulInnerHeight.wrapping_sub(1 as ULONG) as LONG;
    rcLeftPanel.left = 0 as LONG;
    rcLeftPanel.right = ulInnerWidth.wrapping_div(3 as ULONG) as LONG;

    rcRightPanel.top = rcLeftPanel.top;
    rcRightPanel.bottom = rcLeftPanel.bottom;
    rcRightPanel.left = rcLeftPanel.right;
    rcRightPanel.right = ulInnerWidth.wrapping_sub(1 as ULONG) as LONG;

    /* Load the configuration and the resources */
    LoadConfiguration();

    /* Create main window */
    hWndMain = CreateWindow!(
        addr_of_mut!(szWindowClass) as *mut TCHAR as LPCWSTR,
        addr_of_mut!(szAppTitle) as *mut TCHAR as LPCWSTR,
        dwStyle,
        xPos,
        yPos,
        xWidth,
        yHeight,
        0 as HWND,
        0 as HMENU,
        hInstance,
        core::ptr::null_mut(),
    );

    hSystemMenu = GetSystemMenu(hWndMain, FALSE as BOOL);
    if !hSystemMenu.is_null() {
        RemoveMenu(hSystemMenu, SC_SIZE as UINT, MF_BYCOMMAND as UINT);
        RemoveMenu(hSystemMenu, SC_MAXIMIZE as UINT, MF_BYCOMMAND as UINT);
    }

    ShowWindow(hWndMain, nCmdShow);
    UpdateWindow(hWndMain);

    while GetMessage(addr_of_mut!(msg), core::ptr::null_mut(), 0 as UINT, 0 as UINT) != FALSE as INT {
        /* Check for ENTER key presses */
        if msg.message == WM_KEYDOWN as UINT && msg.wParam == VK_RETURN as WPARAM {
            /*
             * The user pressed the ENTER key. Retrieve the handle to the
             * child window that has the keyboard focus, and send it a
             * WM_COMMAND message.
             */
            hWndFocus = GetFocus();
            if !hWndFocus.is_null() {
                SendMessage(hWndMain, WM_COMMAND as UINT,
                            GetDlgCtrlID(hWndFocus) as WPARAM, hWndFocus as LPARAM);
            }
        }
        /* Allow using keyboard navigation */
        else if IsDialogMessage(hWndMain, addr_of_mut!(msg)) == 0 {
            TranslateMessage(addr_of_mut!(msg));
            DispatchMessage(addr_of_mut!(msg));
        }
    }

    /* Cleanup */
    FreeResources();

    /* Close the mutex handle and quit */
    CloseHandle(hMutex);
    return msg.wParam as INT;
}
#[no_mangle]
pub unsafe extern "system" fn ButtonSubclassWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut wParamOld: WPARAM = 0;
    static mut lParamOld: LPARAM = 0;

    let mut i: LONG;

    if uMsg == WM_MOUSEMOVE as UINT {
        /* Ignore mouse-move messages on the same point */
        if wParam == wParamOld && lParam == lParamOld {
            return 0 as INT_PTR;
        }

        /* Retrieve the topic index of this button */
        i = (GetWindowLongPtr(hWnd, GWLP_ID) - (TOPIC_BTN_ID_BASE as LONG_PTR)) as LONG;

        /*
         * Change the focus to this button if the current topic index differs
         * (we will receive WM_SETFOCUS afterwards).
         */
        if nTopic as LONG != i {
            SetFocus(hWnd);
        }

        wParamOld = wParam;
        lParamOld = lParam;
    } else if uMsg == WM_SETFOCUS as UINT {
        /* Retrieve the topic index of this button */
        i = (GetWindowLongPtr(hWnd, GWLP_ID) - (TOPIC_BTN_ID_BASE as LONG_PTR)) as LONG;

        /* Change the current topic index and repaint the description panel */
        if nTopic as LONG != i {
            nTopic = i as INT;
            InvalidateRect(hWndMain, core::ptr::addr_of_mut!(rcRightPanel), (TRUE) as BOOL);
        }
    } else if uMsg == WM_KILLFOCUS as UINT {
        /*
         * We lost focus, either because the user changed button focus,
         * or because the main window to which we belong went inactivated.
         * If we are in the latter case, we ignore the focus change.
         * If we are in the former case, we reset to the default topic.
         */
        if GetParent(hWnd) == GetForegroundWindow() {
            nTopic = -1 as INT;
            InvalidateRect(hWndMain, core::ptr::addr_of_mut!(rcRightPanel), (TRUE) as BOOL);
        }
    }

    return CallWindowProc(fnOldBtn, hWnd, uMsg, wParam, lParam) as INT_PTR;
}

unsafe extern "C" fn RunAction(nTopicArg: INT) -> BOOL {
    let mut Command: PCWSTR = core::ptr::null();
    let mut Args: PCWSTR = core::ptr::null();

    if nTopicArg < 0 {
        return (TRUE) as BOOL;
    }

    Command = core::ptr::addr_of_mut!((**pTopics.offset(nTopicArg as isize)).szCommand) as *mut TCHAR as PCWSTR;
    if *Command == 0 {
        return (TRUE) as BOOL;
    }

    /* Check for known actions */
    if (**pTopics.offset(nTopicArg as isize)).bIsCommand == 0 {
        let exit_s: [u16; 7] = ['<' as u16, 'e' as u16, 'x' as u16, 'i' as u16, 't' as u16, '>' as u16, 0];
        if _tcsicmp(Command, exit_s.as_ptr()) == 0 {
            return (FALSE) as BOOL;
        }

        let msg_s: [u16; 7] = ['<' as u16, 'm' as u16, 's' as u16, 'g' as u16, '>' as u16, 0, 0];
        if _tcsnicmp(Command, msg_s.as_ptr(), 5) == 0 {
            let reactos_s: [u16; 9] = ['R' as u16, 'e' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'O' as u16, 'S' as u16, 0, 0];
            MessageBox(
                hWndMain,
                Command.offset(5 as isize),
                reactos_s.as_ptr(),
                MB_OK | MB_TASKMODAL,
            );
            return (TRUE) as BOOL;
        }
    } else {
        /* Run the command */
        Args = core::ptr::addr_of_mut!((**pTopics.offset(nTopicArg as isize)).szArgs) as *mut TCHAR as PCWSTR;
        if *Args == 0 {
            Args = core::ptr::null();
        }
        ShellExecute(
            core::ptr::null_mut(),
            core::ptr::null(),
            Command,
            Args,
            core::ptr::null(),
            (SW_SHOWDEFAULT as i32),
        );
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn GetButtonHeight(
    hDC: HDC,
    hFont: HFONT,
    szText: LPCTSTR,
    dwWidth: DWORD,
) -> DWORD {
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();

    rect.left = 0 as LONG;
    rect.right = dwWidth.wrapping_sub(20 as DWORD) as LONG;
    rect.top = 0 as LONG;
    rect.bottom = 25 as LONG;

    hOldFont = SelectObject(hDC, hFont as HGDIOBJ) as HFONT;
    DrawText(
        hDC,
        szText,
        -(1 as core::ffi::c_int),
        core::ptr::addr_of_mut!(rect),
        (DT_TOP | DT_CALCRECT | DT_WORDBREAK) as UINT,
    );
    SelectObject(hDC, hOldFont as HGDIOBJ);

    return (rect.bottom - rect.top + 14 as LONG) as DWORD;
}

unsafe extern "C" fn OnCreate(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut i: UINT = 0;
    let mut nLength: INT = 0;
    let mut ScreenDC: HDC = core::ptr::null_mut();
    let mut lf: LOGFONT = core::mem::zeroed();
    let mut dwTop: DWORD = 0;
    let mut dwHeight: DWORD = 0 as DWORD;
    let mut szText: [TCHAR; 80] = [0; 80];
    let button_class: [u16; 7] = ['B' as u16, 'U' as u16, 'T' as u16, 'T' as u16, 'O' as u16, 'N' as u16, 0];
    let tahoma: [u16; 7] = ['T' as u16, 'a' as u16, 'h' as u16, 'o' as u16, 'm' as u16, 'a' as u16, 0];

    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(lParam);

    hbrLightBlue = CreateSolidBrush(LIGHT_BLUE);
    hbrDarkBlue = CreateSolidBrush(DARK_BLUE);

    ZeroMemory!(core::ptr::addr_of_mut!(lf) as *mut ::core::ffi::c_void, core::mem::size_of::<LOGFONT>());

    lf.lfEscapement = 0 as LONG;
    lf.lfOrientation = 0 as LONG; // TA_BASELINE;
    // lf.lfItalic = lf.lfUnderline = lf.lfStrikeOut = FALSE;
    lf.lfCharSet = ANSI_CHARSET as BYTE;
    lf.lfOutPrecision = OUT_DEFAULT_PRECIS as BYTE;
    lf.lfClipPrecision = CLIP_DEFAULT_PRECIS as BYTE;
    lf.lfQuality = DEFAULT_QUALITY as BYTE;
    lf.lfPitchAndFamily = FF_DONTCARE as BYTE;
    if LoadString(
        hInstance,
        IDS_FONTNAME as UINT,
        core::ptr::addr_of_mut!(lf.lfFaceName) as *mut TCHAR as LPTSTR,
        ARRAYSIZE!(lf.lfFaceName) as INT,
    ) == 0 as INT
    {
        StringCchCopy(
            core::ptr::addr_of_mut!(lf.lfFaceName) as *mut TCHAR as LPTSTR,
            (ARRAYSIZE!(lf.lfFaceName)) as _,
            tahoma.as_ptr() as LPCTSTR,
        );
    }

    /* Topic title font */
    lf.lfHeight = -(18 as INT) as LONG;
    lf.lfWidth = 0 as LONG;
    lf.lfWeight = FW_NORMAL as LONG;
    hFontTopicTitle = CreateFontIndirect(core::ptr::addr_of_mut!(lf));

    /* Topic description font */
    lf.lfHeight = -(11 as INT) as LONG;
    lf.lfWidth = 0 as LONG;
    lf.lfWeight = FW_THIN as LONG;
    hFontTopicDescription = CreateFontIndirect(core::ptr::addr_of_mut!(lf));

    /* Topic button font */
    lf.lfHeight = -(11 as INT) as LONG;
    lf.lfWidth = 0 as LONG;
    lf.lfWeight = FW_BOLD as LONG;
    hFontTopicButton = CreateFontIndirect(core::ptr::addr_of_mut!(lf));

    /* Load title bitmap */
    if !hTitleBitmap.is_null() {
        hTitleBitmap = LoadBitmap(
            hInstance,
            MAKEINTRESOURCE!(IDB_TITLE_BITMAP) as LPCTSTR,
        );
    }

    /* Load topic bitmaps */
    hDefaultTopicBitmap = LoadBitmap(
        hInstance,
        MAKEINTRESOURCE!(IDB_DEFAULT_TOPIC_BITMAP) as LPCTSTR,
    );
    i = 0 as UINT;
    while i < dwNumberTopics {
        // FIXME: Not implemented yet!
        // pTopics[i]->hBitmap = LoadBitmap(hInstance, MAKEINTRESOURCE(IDB_TOPIC_BITMAP0 + i));
        (**pTopics.offset(i as isize)).hBitmap = core::ptr::null_mut();
        i += 1;
    }

    ScreenDC = GetWindowDC(hWnd);
    hdcMem = CreateCompatibleDC(ScreenDC);
    ReleaseDC(hWnd, ScreenDC);

    /* Load and create the menu buttons */
    dwTop = rcLeftPanel.top as DWORD;
    i = 0 as UINT;
    while i < dwNumberTopics {
        if (**pTopics.offset(i as isize)).szText[0] != 0 {
            dwHeight = GetButtonHeight(
                hdcMem,
                hFontTopicButton,
                core::ptr::addr_of_mut!((**pTopics.offset(i as isize)).szText) as *mut TCHAR as LPCTSTR,
                (rcLeftPanel.right - rcLeftPanel.left) as DWORD,
            );

            (**pTopics.offset(i as isize)).hWndButton = CreateWindow!(
                button_class.as_ptr() as LPCTSTR,
                core::ptr::addr_of_mut!((**pTopics.offset(i as isize)).szText) as *mut TCHAR as LPCTSTR,
                ((if (**pTopics.offset(i as isize)).szCommand[0] != 0 {
                    0
                } else {
                    WS_DISABLED
                }) as DWORD)
                    | WS_CHILDWINDOW as DWORD
                    | WS_VISIBLE as DWORD
                    | WS_TABSTOP as DWORD
                    | BS_MULTILINE as DWORD
                    | BS_OWNERDRAW as DWORD,
                rcLeftPanel.left as INT,
                dwTop as INT,
                (rcLeftPanel.right - rcLeftPanel.left) as INT,
                dwHeight as INT,
                hWnd,
                IntToPtr!((TOPIC_BTN_ID_BASE + (i) as i32) as INT) as HMENU, // Similar to SetWindowLongPtr(GWLP_ID)
                hInstance,
                core::ptr::null_mut(),
            );
            nDefaultTopic = i as INT;
            SendMessage(
                (**pTopics.offset(i as isize)).hWndButton,
                WM_SETFONT as UINT,
                hFontTopicButton as WPARAM,
                MAKELPARAM!(TRUE as WORD, 0 as WORD),
            );
            fnOldBtn = core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtr(
                (**pTopics.offset(i as isize)).hWndButton,
                GWLP_WNDPROC as INT,
                ButtonSubclassWndProc as LONG_PTR,
            ));
        } else {
            (**pTopics.offset(i as isize)).hWndButton = core::ptr::null_mut();
        }

        dwTop += dwHeight;
        i += 1;
    }

    /* Create the checkbox */
    if bDisplayCheckBox != 0 {
        nLength = LoadString(
            hInstance,
            IDS_CHECKTEXT as UINT,
            core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPTSTR,
            ARRAYSIZE!(szText) as INT,
        );
        if nLength > 0 as INT {
            lf.lfHeight = -(10 as INT) as LONG;
            lf.lfWidth = 0 as LONG;
            lf.lfWeight = FW_THIN as LONG;
            hFontCheckButton = CreateFontIndirect(core::ptr::addr_of_mut!(lf));

            hWndCheckButton = CreateWindow!(
                button_class.as_ptr() as LPCTSTR,
                core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPCTSTR,
                (WS_CHILDWINDOW as DWORD)
                    | WS_VISIBLE as DWORD
                    | WS_TABSTOP as DWORD
                    | BS_AUTOCHECKBOX as DWORD
                    | BS_MULTILINE as DWORD /**/| BS_FLAT as DWORD /**/,
                (rcLeftPanel.left + 8 as LONG) as INT,
                (rcLeftPanel.bottom - 8 as LONG - 13 as LONG) as INT,
                (rcLeftPanel.right - rcLeftPanel.left - 16 as LONG) as INT,
                13 as INT,
                hWnd,
                (IDC_CHECKBUTTON as usize) as HMENU,
                hInstance,
                core::ptr::null_mut(),
            );
            SendMessage(
                hWndCheckButton,
                WM_SETFONT as UINT,
                hFontCheckButton as WPARAM,
                MAKELPARAM!(TRUE as WORD, 0 as WORD),
            );
        } else {
            hFontCheckButton = core::ptr::null_mut();
            hWndCheckButton = core::ptr::null_mut();
        }
    }

    /* Create the "Exit" button */
    if bDisplayExitBtn != 0 {
        nLength = LoadString(
            hInstance,
            IDS_CLOSETEXT as UINT,
            core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPTSTR,
            ARRAYSIZE!(szText) as INT,
        );
        if nLength > 0 as INT {
            hWndCloseButton = CreateWindow!(
                button_class.as_ptr() as LPCTSTR,
                core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPCTSTR,
                (WS_CHILDWINDOW as DWORD)
                    | WS_VISIBLE as DWORD
                    | WS_TABSTOP as DWORD
                    | BS_FLAT as DWORD,
                (rcRightPanel.right - 8 as LONG - 57 as LONG) as INT,
                (rcRightPanel.bottom - 8 as LONG - 21 as LONG) as INT,
                57 as INT,
                21 as INT,
                hWnd,
                (IDC_CLOSEBUTTON as usize) as HMENU,
                hInstance,
                core::ptr::null_mut(),
            );
            nDefaultTopic = -(1 as INT);
            SendMessage(
                hWndCloseButton,
                WM_SETFONT as UINT,
                hFontTopicButton as WPARAM,
                MAKELPARAM!(TRUE as WORD, 0 as WORD),
            );
        } else {
            hWndCloseButton = core::ptr::null_mut();
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn OnCommand(mut hWnd: HWND, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    UNREFERENCED_PARAMETER!(lParam);

    /* Retrieve the low-word from wParam */
    wParam = LOWORD!(wParam) as WPARAM;

    /* Execute action */
    if wParam == (IDC_CLOSEBUTTON as WPARAM) {
        DestroyWindow(hWnd);
    } else if wParam.wrapping_sub(TOPIC_BTN_ID_BASE as WPARAM) < dwNumberTopics as WPARAM {
        if RunAction(wParam.wrapping_sub(TOPIC_BTN_ID_BASE as WPARAM) as INT) == (FALSE as i32) {
            DestroyWindow(hWnd); // Corresponds to a <exit> action.
        }
    }

    return 0;
}

unsafe extern "C" fn PaintBanner(hdc: HDC, rcPanel: LPRECT) {
    let mut hOldBitmap: HBITMAP = ::core::ptr::null_mut::<HBITMAP__>();
    let mut hOldBrush: HBRUSH = ::core::ptr::null_mut::<HBRUSH__>();

    /* Title bitmap */
    hOldBitmap = SelectObject(hdcMem, hTitleBitmap as HGDIOBJ) as HBITMAP;
    BitBlt(
        hdc,
        (*rcPanel).left as INT,
        (*rcPanel).top as INT,
        ((*rcPanel).right - (*rcPanel).left) as INT,
        ((*rcPanel).bottom - (3 as LONG)) as INT,
        hdcMem,
        0 as INT,
        0 as INT,
        SRCCOPY as DWORD,
    );
    SelectObject(hdcMem, hOldBitmap as HGDIOBJ);

    /* Dark blue line */
    hOldBrush = SelectObject(hdc, hbrDarkBlue as HGDIOBJ) as HBRUSH;
    PatBlt(
        hdc,
        (*rcPanel).left as ::core::ffi::c_int,
        ((*rcPanel).bottom - (3 as LONG)) as ::core::ffi::c_int,
        ((*rcPanel).right - (*rcPanel).left) as ::core::ffi::c_int,
        3 as ::core::ffi::c_int,
        PATCOPY as DWORD,
    );
    SelectObject(hdc, hOldBrush as HGDIOBJ);
}

unsafe extern "C" fn OnPaint(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut hPen: HPEN = core::ptr::null_mut();
    let mut hOldPen: HPEN;
    let mut hdc: HDC;
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hOldBitmap: HBITMAP = core::ptr::null_mut();
    let mut hOldBrush: HBRUSH;
    let mut hOldFont: HFONT;
    let mut rcTitle: RECT = core::mem::zeroed();
    let mut rcDescription: RECT = core::mem::zeroed();
    let mut bmpInfo: BITMAP = core::mem::zeroed();
    let mut szVersion: [TCHAR; 50] = [0; 50];
    let mut lpTitle: LPTSTR = core::ptr::null_mut();
    let mut lpDesc: LPTSTR = core::ptr::null_mut();

    let _ = wParam;
    let _ = lParam;

    hdc = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));

    /* Banner panel */
    PaintBanner(hdc, core::ptr::addr_of_mut!(rcTitlePanel));

    /* Left panel */
    hOldBrush = SelectObject(hdc, hbrLightBlue as HGDIOBJ) as HBRUSH;
    PatBlt(
        hdc,
        rcLeftPanel.left as core::ffi::c_int,
        rcLeftPanel.top as core::ffi::c_int,
        (rcLeftPanel.right - rcLeftPanel.left) as core::ffi::c_int,
        (rcLeftPanel.bottom - rcLeftPanel.top) as core::ffi::c_int,
        PATCOPY as DWORD,
    );
    SelectObject(hdc, hOldBrush as HGDIOBJ);

    /* Right panel */
    hOldBrush = SelectObject(hdc, GetStockObject((WHITE_BRUSH as i32)) as HGDIOBJ) as HBRUSH;
    PatBlt(
        hdc,
        rcRightPanel.left as core::ffi::c_int,
        rcRightPanel.top as core::ffi::c_int,
        (rcRightPanel.right - rcRightPanel.left) as core::ffi::c_int,
        (rcRightPanel.bottom - rcRightPanel.top) as core::ffi::c_int,
        PATCOPY as DWORD,
    );
    SelectObject(hdc, hOldBrush as HGDIOBJ);

    /* Draw dark vertical line */
    hPen = CreatePen(
        PS_SOLID as core::ffi::c_int,
        0 as core::ffi::c_int,
        DARK_BLUE,
    );
    hOldPen = SelectObject(hdc, hPen as HGDIOBJ) as HPEN;
    MoveToEx(
        hdc,
        rcRightPanel.left as core::ffi::c_int,
        rcRightPanel.top as core::ffi::c_int,
        core::ptr::null_mut(),
    );
    LineTo(
        hdc,
        rcRightPanel.left as core::ffi::c_int,
        rcRightPanel.bottom as core::ffi::c_int,
    );
    SelectObject(hdc, hOldPen as HGDIOBJ);
    DeleteObject(hPen as HGDIOBJ);

    /* Draw topic bitmap */
    if (nTopic == -(1 as INT)) && !hDefaultTopicBitmap.is_null() {
        GetObject(
            hDefaultTopicBitmap as HANDLE,
            core::mem::size_of::<BITMAP>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(bmpInfo) as LPVOID,
        );
        hOldBitmap = SelectObject(hdcMem, hDefaultTopicBitmap as HGDIOBJ) as HBITMAP;
        BitBlt(
            hdc,
            (rcRightPanel.right - bmpInfo.bmWidth) as INT,
            (rcRightPanel.bottom - bmpInfo.bmHeight) as INT,
            bmpInfo.bmWidth as INT,
            bmpInfo.bmHeight as INT,
            hdcMem,
            0 as INT,
            0 as INT,
            SRCCOPY as DWORD,
        );
    } else if (nTopic != -(1 as INT))
        && !(*(*pTopics.offset(nTopic as isize))).hBitmap.is_null()
    {
        GetObject(
            (*(*pTopics.offset(nTopic as isize))).hBitmap as HANDLE,
            core::mem::size_of::<BITMAP>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(bmpInfo) as LPVOID,
        );
        hOldBitmap = SelectObject(
            hdcMem,
            (*(*pTopics.offset(nTopic as isize))).hBitmap as HGDIOBJ,
        ) as HBITMAP;
        BitBlt(
            hdc,
            (rcRightPanel.right - bmpInfo.bmWidth) as INT,
            (rcRightPanel.bottom - bmpInfo.bmHeight) as INT,
            bmpInfo.bmWidth as INT,
            bmpInfo.bmHeight as INT,
            hdcMem,
            0 as INT,
            0 as INT,
            SRCCOPY as DWORD,
        );
    }

    if nTopic == -(1 as INT) {
        lpTitle = core::ptr::addr_of_mut!(szDefaultTitle) as *mut TCHAR as LPTSTR;
        lpDesc = core::ptr::addr_of_mut!(szDefaultDesc) as *mut TCHAR as LPTSTR;
    } else {
        lpTitle = core::ptr::addr_of_mut!((*(*pTopics.offset(nTopic as isize))).szTitle) as *mut TCHAR as LPTSTR;
        lpDesc = core::ptr::addr_of_mut!((*(*pTopics.offset(nTopic as isize))).szDesc) as *mut TCHAR as LPTSTR;
    }

    SetBkMode(hdc, TRANSPARENT as INT);

    /* Draw version information */
    /* "ReactOS " prefix as an explicit wide array (NUL-terminated) */
    let szVersionPrefix: [TCHAR; 9] = [
        'R' as TCHAR, 'e' as TCHAR, 'a' as TCHAR, 'c' as TCHAR, 't' as TCHAR,
        'O' as TCHAR, 'S' as TCHAR, ' ' as TCHAR, 0,
    ];
    let mut szVersionBuf: [TCHAR; 50] = [0; 50];
    // TEXT(KERNEL_VERSION_STR): the version header's string is ANSI bytes; widen each one.
    let szVersionTail: &[u8] = KERNEL_VERSION_STR;
    let mut vi: usize = 0;
    while vi < 8 {
        szVersionBuf[vi] = szVersionPrefix[vi];
        vi += 1;
    }
    let mut vj: usize = 0;
    while vj < szVersionTail.len() && szVersionTail[vj] != 0 && 8 + vj < szVersionBuf.len() - 1 {
        szVersionBuf[8 + vj] = szVersionTail[vj] as TCHAR;
        vj += 1;
    }
    szVersionBuf[8 + vj] = 0;
    StringCchCopy(
        core::ptr::addr_of_mut!(szVersion) as *mut TCHAR as STRSAFE_LPWSTR,
        ARRAYSIZE!(szVersion) as size_t,
        core::ptr::addr_of!(szVersionBuf) as *const TCHAR as STRSAFE_LPCWSTR,
    );

    /*
     * Compute the original rect (position & size) of the version info,
     * depending whether the checkbox is displayed (version info in the
     * right panel) or not (version info in the left panel).
     */
    if bDisplayCheckBox != 0 {
        rcTitle = rcRightPanel;
    } else {
        rcTitle = rcLeftPanel;
    }

    rcTitle.left = rcTitle.left + 8 as LONG;
    rcTitle.right = rcTitle.right - 5 as LONG;
    rcTitle.top = rcTitle.bottom - 43 as LONG;
    rcTitle.bottom = rcTitle.bottom - 8 as LONG;

    hOldFont = SelectObject(hdc, hFontTopicDescription as HGDIOBJ) as HFONT;
    DrawText(
        hdc,
        core::ptr::addr_of_mut!(szVersion) as *mut TCHAR as LPCWSTR,
        -(1 as INT),
        core::ptr::addr_of_mut!(rcTitle),
        (DT_BOTTOM | DT_CALCRECT | DT_SINGLELINE) as UINT,
    );
    SetTextColor(hdc, GetSysColor((COLOR_WINDOWTEXT as i32)) as COLORREF);
    DrawText(
        hdc,
        core::ptr::addr_of_mut!(szVersion) as *mut TCHAR as LPCWSTR,
        -(1 as INT),
        core::ptr::addr_of_mut!(rcTitle),
        (DT_BOTTOM | DT_SINGLELINE) as UINT,
    );
    SelectObject(hdc, hOldFont as HGDIOBJ);

    /* Draw topic title */
    rcTitle.left = rcRightPanel.left + 12 as LONG;
    rcTitle.right = rcRightPanel.right - 8 as LONG;
    rcTitle.top = rcRightPanel.top + 8 as LONG;
    rcTitle.bottom = rcTitle.top + 57 as LONG;
    hOldFont = SelectObject(hdc, hFontTopicTitle as HGDIOBJ) as HFONT;
    DrawText(
        hdc,
        lpTitle as LPCWSTR,
        -(1 as INT),
        core::ptr::addr_of_mut!(rcTitle),
        (DT_TOP | DT_CALCRECT) as UINT,
    );
    SetTextColor(hdc, DARK_BLUE);
    DrawText(
        hdc,
        lpTitle as LPCWSTR,
        -(1 as INT),
        core::ptr::addr_of_mut!(rcTitle),
        DT_TOP as UINT,
    );
    SelectObject(hdc, hOldFont as HGDIOBJ);

    /* Draw topic description */
    rcDescription.left = rcRightPanel.left + 12 as LONG;
    rcDescription.right = rcRightPanel.right - 8 as LONG;
    rcDescription.top = rcTitle.bottom + 8 as LONG;
    rcDescription.bottom = rcRightPanel.bottom - 20 as LONG;
    hOldFont = SelectObject(hdc, hFontTopicDescription as HGDIOBJ) as HFONT;
    SetTextColor(hdc, GetSysColor((COLOR_WINDOWTEXT as i32)) as COLORREF);
    DrawText(
        hdc,
        lpDesc as LPCWSTR,
        -(1 as INT),
        core::ptr::addr_of_mut!(rcDescription),
        (DT_TOP | DT_WORDBREAK) as UINT,
    );
    SelectObject(hdc, hOldFont as HGDIOBJ);

    SetBkMode(hdc, OPAQUE as INT);

    SelectObject(hdcMem, hOldBrush as HGDIOBJ);
    SelectObject(hdcMem, hOldBitmap as HGDIOBJ);

    EndPaint(hWnd, core::ptr::addr_of_mut!(ps));

    return 0 as LRESULT;
}

unsafe extern "C" fn OnDrawItem(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut lpDis: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;
    let mut hPen: HPEN = core::ptr::null_mut();
    let mut hOldPen: HPEN = core::ptr::null_mut();
    let mut hOldBrush: HBRUSH = core::ptr::null_mut();
    let mut iBkMode: INT = 0;
    let mut szText: [TCHAR; 80] = [0 as TCHAR; 80];

    UNREFERENCED_PARAMETER!(hWnd);
    UNREFERENCED_PARAMETER!(wParam);

    if (*lpDis).CtlID as ULONG == (TOPIC_BTN_ID_BASE + nTopic) as ULONG {
        hOldBrush = SelectObject((*lpDis).hDC, GetStockObject((WHITE_BRUSH as i32))) as HBRUSH;
    } else {
        hOldBrush = SelectObject((*lpDis).hDC, hbrLightBlue as HGDIOBJ) as HBRUSH;
    }

    PatBlt(
        (*lpDis).hDC,
        (*lpDis).rcItem.left as ::core::ffi::c_int,
        (*lpDis).rcItem.top as ::core::ffi::c_int,
        (*lpDis).rcItem.right as ::core::ffi::c_int,
        (*lpDis).rcItem.bottom as ::core::ffi::c_int,
        PATCOPY as DWORD,
    );
    SelectObject((*lpDis).hDC, hOldBrush as HGDIOBJ);

    hPen = CreatePen((PS_SOLID as i32), 0 as ::core::ffi::c_int, DARK_BLUE);
    hOldPen = SelectObject((*lpDis).hDC, hPen as HGDIOBJ) as HPEN;
    MoveToEx(
        (*lpDis).hDC,
        (*lpDis).rcItem.left as ::core::ffi::c_int,
        ((*lpDis).rcItem.bottom - 1 as LONG) as ::core::ffi::c_int,
        core::ptr::null_mut(),
    );
    LineTo(
        (*lpDis).hDC,
        (*lpDis).rcItem.right as ::core::ffi::c_int,
        ((*lpDis).rcItem.bottom - 1 as LONG) as ::core::ffi::c_int,
    );
    SelectObject((*lpDis).hDC, hOldPen as HGDIOBJ);
    DeleteObject(hPen as HGDIOBJ);

    InflateRect(
        core::ptr::addr_of_mut!((*lpDis).rcItem),
        -(10 as ::core::ffi::c_int),
        -(4 as ::core::ffi::c_int),
    );
    OffsetRect(
        core::ptr::addr_of_mut!((*lpDis).rcItem),
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    GetWindowText((*lpDis).hwndItem, szText.as_mut_ptr(), ARRAYSIZE!(szText) as ::core::ffi::c_int);
    SetTextColor(
        (*lpDis).hDC,
        GetSysColor(if IsWindowEnabled((*lpDis).hwndItem) != 0 {
            (COLOR_WINDOWTEXT as i32)
        } else {
            (COLOR_GRAYTEXT as i32)
        }) as COLORREF,
    );
    iBkMode = SetBkMode((*lpDis).hDC, (TRANSPARENT as i32)) as INT;
    DrawText(
        (*lpDis).hDC,
        szText.as_mut_ptr(),
        -(1 as ::core::ffi::c_int),
        core::ptr::addr_of_mut!((*lpDis).rcItem),
        (DT_TOP | DT_LEFT | DT_WORDBREAK) as UINT,
    );
    SetBkMode((*lpDis).hDC, iBkMode as ::core::ffi::c_int);

    return TRUE as LRESULT;
}

unsafe extern "C" fn OnMouseMove(mut hWnd: HWND, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    static mut wParamOld: WPARAM = 0;
    static mut lParamOld: LPARAM = 0;

    /* Ignore mouse-move messages on the same point */
    if wParam == wParamOld && lParam == lParamOld {
        return 0 as LRESULT;
    }

    /*
     * If the user moves the mouse over the main window, outside of the
     * topic buttons, reset the current topic to the default one and
     * change the focus to some other default button (to keep keyboard
     * navigation possible).
     */
    if nTopic != -1 {
        let mut nOldTopic: INT = nTopic;
        nTopic = -1 as INT;
        /* Also repaint the buttons, otherwise nothing repaints... */
        InvalidateRect(
            (**pTopics.offset(nOldTopic as isize)).hWndButton,
            core::ptr::null::<RECT>(),
            (TRUE) as BOOL,
        );

        /* Set the focus to some other default button */
        if !hWndCheckButton.is_null() {
            SetFocus(hWndCheckButton);
        } else if !hWndCloseButton.is_null() {
            SetFocus(hWndCloseButton);
        }
        // SetFocus(hWnd);

        /* Repaint the description panel */
        InvalidateRect(
            hWndMain,
            core::ptr::addr_of_mut!(rcRightPanel) as LPCRECT,
            (TRUE) as BOOL,
        );
    }

    wParamOld = wParam;
    lParamOld = lParam;

    return 0 as LRESULT;
}

unsafe extern "C" fn OnCtlColorStatic(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    UNREFERENCED_PARAMETER!(hWnd);

    if lParam as HWND == hWndCheckButton {
        SetBkMode(wParam as HDC, (TRANSPARENT as i32));
        return hbrLightBlue as LRESULT;
    }

    return 0;
}

unsafe extern "C" fn OnActivate(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    UNREFERENCED_PARAMETER!(hWnd);
    UNREFERENCED_PARAMETER!(lParam);

    if wParam != (WA_INACTIVE as _) {
        /*
         * The main window is re-activated, set the focus back to
         * either the current topic or a default button.
         */
        if nTopic != -1 {
            SetFocus((**pTopics.offset(nTopic as isize)).hWndButton);
        } else if !hWndCheckButton.is_null() {
            SetFocus(hWndCheckButton);
        } else if !hWndCloseButton.is_null() {
            SetFocus(hWndCloseButton);
        }

        // InvalidateRect(hWndMain, &rcRightPanel, TRUE);
    }

    return 0;
}

unsafe extern "C" fn OnDestroy(mut hWnd: HWND, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    let mut i: UINT = 0;

    UNREFERENCED_PARAMETER!(hWnd);
    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(lParam);

    i = 0 as UINT;
    while (i as DWORD) < dwNumberTopics {
        if !(**pTopics.offset(i as isize)).hWndButton.is_null() {
            DestroyWindow((**pTopics.offset(i as isize)).hWndButton);
        }
        i = i.wrapping_add(1);
    }

    if !hWndCloseButton.is_null() {
        DestroyWindow(hWndCloseButton);
    }

    if !hWndCheckButton.is_null() {
        DestroyWindow(hWndCheckButton);
    }

    DeleteDC(hdcMem);

    /* Delete bitmaps */
    DeleteObject(hDefaultTopicBitmap as HGDIOBJ);
    DeleteObject(hTitleBitmap as HGDIOBJ);
    i = 0 as UINT;
    while (i as DWORD) < dwNumberTopics {
        if !(**pTopics.offset(i as isize)).hBitmap.is_null() {
            DeleteObject((**pTopics.offset(i as isize)).hBitmap as HGDIOBJ);
        }
        i = i.wrapping_add(1);
    }

    DeleteObject(hFontTopicTitle as HGDIOBJ);
    DeleteObject(hFontTopicDescription as HGDIOBJ);
    DeleteObject(hFontTopicButton as HGDIOBJ);

    if !hFontCheckButton.is_null() {
        DeleteObject(hFontCheckButton as HGDIOBJ);
    }

    DeleteObject(hbrLightBlue as HGDIOBJ);
    DeleteObject(hbrDarkBlue as HGDIOBJ);

    return 0 as LRESULT;
}
#[no_mangle]
pub unsafe extern "system" fn MainWndProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_CREATE => {
            return OnCreate(hWnd, wParam, lParam) as INT_PTR;
        }
        WM_COMMAND => {
            return OnCommand(hWnd, wParam, lParam) as INT_PTR;
        }
        WM_ACTIVATE => {
            return OnActivate(hWnd, wParam, lParam) as INT_PTR;
        }
        WM_PAINT => {
            return OnPaint(hWnd, wParam, lParam) as INT_PTR;
        }
        WM_DRAWITEM => {
            return OnDrawItem(hWnd, wParam, lParam) as INT_PTR;
        }
        WM_CTLCOLORSTATIC => {
            return OnCtlColorStatic(hWnd, wParam, lParam) as INT_PTR;
        }
        WM_MOUSEMOVE => {
            return OnMouseMove(hWnd, wParam, lParam) as INT_PTR;
        }
        WM_DESTROY => {
            OnDestroy(hWnd, wParam, lParam);
            PostQuitMessage(0);
            return 0;
        }
        _ => {}
    }

    return DefWindowProc(hWnd, uMsg, wParam, lParam) as INT_PTR;
}

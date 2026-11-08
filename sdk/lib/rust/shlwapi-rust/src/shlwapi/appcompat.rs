use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut g_fAppCompatInitialized: BOOL = (FALSE as i32);

static mut g_dwAppCompatFlags: DWORD = 0 as DWORD;

static mut g_appCompatFlagMaps: [FLAGMAP; 19] = [
    tagFLAGMAP {
        flags: SHACF_CONTEXTMENU as DWORD,
        name: b"CONTEXTMENU\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_CORELINTERNETENUM as DWORD,
        name: b"CORELINTERNETENUM\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_OLDCREATEVIEWWND as DWORD,
        name: b"OLDCREATEVIEWWND\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_WIN95DEFVIEW as DWORD,
        name: b"WIN95DEFVIEW\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_DOCOBJECT as DWORD,
        name: b"DOCOBJECT\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_FLUSHNOWAITALWAYS as DWORD,
        name: b"FLUSHNOWAITALWAYS\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_MYCOMPUTERFIRST as DWORD,
        name: b"MYCOMPUTERFIRST\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_OLDREGITEMGDN as DWORD,
        name: b"OLDREGITEMGDN\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_LOADCOLUMNHANDLER as DWORD,
        name: b"LOADCOLUMNHANDLER\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_ANSI as DWORD,
        name: b"ANSI\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_STAROFFICE5PRINTER as DWORD,
        name: b"STAROFFICE5PRINTER\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_NOVALIDATEFSIDS as DWORD,
        name: b"NOVALIDATEFSIDS\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_WIN95SHLEXEC as DWORD,
        name: b"WIN95SHLEXEC\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_FILEOPENNEEDSEXT as DWORD,
        name: b"FILEOPENNEEDSEXT\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_WIN95BINDTOOBJECT as DWORD,
        name: b"WIN95BINDTOOBJECT\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_IGNOREENUMRESET as DWORD,
        name: b"IGNOREENUMRESET\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_ANSIDISPLAYNAMES as DWORD,
        name: b"ANSIDISPLAYNAMES\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_FILEOPENBOGUSCTRLID as DWORD,
        name: b"FILEOPENBOGUSCTRLID\0" as *const u8 as LPCSTR,
    },
    tagFLAGMAP {
        flags: SHACF_FORCELFNIDLIST as DWORD,
        name: b"FORCELFNIDLIST\0" as *const u8 as LPCSTR,
    },
];

pub const MAJOR_VER_ONLY: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x01\0") };

static mut g_appCompatInfo: [APPCOMPATINFO; 35] = [
    tagAPPCOMPATINFO {
        pszAppName: b"abcmm.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: SHACF_LOADCOLUMNHANDLER as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"autorun.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"4.00.950\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_ANSI as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"autorun.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"4.10.1998\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_ANSI as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"coreldrw.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x017\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_OLDREGITEMGDN as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"dad9.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x019\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_CORELINTERNETENUM as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"filler51.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: SHACF_OLDREGITEMGDN as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"hotdog4.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: SHACF_DOCOBJECT as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"msmoney.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"7.05.1107\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_WIN95SHLEXEC as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"pdexplo.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x011\0" as *const u8 as PCSTR,
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_MYCOMPUTERFIRST) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"pdexplo.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x012\0" as *const u8 as PCSTR,
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_MYCOMPUTERFIRST) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"pdexplo.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x013\0" as *const u8 as PCSTR,
        dwCompatFlags: (SHACF_MYCOMPUTERFIRST | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"pdxwin32.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"pfim80.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"powerpnt.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x018\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_WIN95SHLEXEC as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"pp70.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: SHACF_LOADCOLUMNHANDLER as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"pp80.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: SHACF_LOADCOLUMNHANDLER as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"prwin70.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_CORELINTERNETENUM) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"prwin8.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"prwin9.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x019\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_CORELINTERNETENUM as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"ps80.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"ps80.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: SHACF_OLDREGITEMGDN as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"qfinder.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"qpw.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x017\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_CONTEXTMENU as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"qpw.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x018\0" as *const u8 as PCSTR,
        dwCompatFlags: (SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN | SHACF_ANSIDISPLAYNAMES)
            as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"qpw.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x019\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_CORELINTERNETENUM as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"rnaapp.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: SHACF_CONTEXTMENU as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"sitebuilder.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"sizemgr.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x013\0" as *const u8 as PCSTR,
        dwCompatFlags: (SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"smartctr.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"96.0\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_CONTEXTMENU as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"soffice.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x015\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_STAROFFICE5PRINTER as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"ua80.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"ue32.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"2.00.0.0\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_OLDREGITEMGDN as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"wpwin7.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CONTEXTMENU | SHACF_CORELINTERNETENUM) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"wpwin8.exe\0" as *const u8 as PCSTR,
        pszAppVersion: ::core::ptr::null::<CHAR>(),
        dwCompatFlags: (SHACF_CORELINTERNETENUM | SHACF_OLDREGITEMGDN) as DWORD,
    },
    tagAPPCOMPATINFO {
        pszAppName: b"wpwin9.exe\0" as *const u8 as PCSTR,
        pszAppVersion: b"\x019\0" as *const u8 as PCSTR,
        dwCompatFlags: SHACF_CORELINTERNETENUM as DWORD,
    },
];

static mut g_wndCompatInfo: [WNDCOMPATINFO; 2] = [
    tagWNDCOMPATINFO {
        pszLengthAndClassName: b"\tbosa_sdm_\0" as *const u8 as PCSTR,
        dwCompatFlags: (SHACF_UNKNOWN1 | SHACF_UNKNOWN2) as DWORD,
    },
    tagWNDCOMPATINFO {
        pszLengthAndClassName: b"\x18File Open Message Window\0" as *const u8 as PCSTR,
        dwCompatFlags: (SHACF_UNKNOWN1 | SHACF_UNKNOWN2) as DWORD,
    },
];

pub static PRODUCT_VER_ENGLISH_US_UTF16: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"\\StringFileInfo\\040904E4\\ProductVersion\0",
    )
};

pub static PRODUCT_VER_GERMAN_UTF16: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"\\StringFileInfo\\040704E4\\ProductVersion\0",
    )
};

pub static PRODUCT_VER_ENGLISH_US_WE: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"\\StringFileInfo\\040904B0\\ProductVersion\0",
    )
};

pub static PRODUCT_VER_ENGLISH_US_NEUTRAL: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"\\StringFileInfo\\04090000\\ProductVersion\0",
    )
};

pub static PRODUCT_VER_SWEDISH_WE: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"\\StringFileInfo\\041D04B0\\ProductVersion\0",
    )
};

// WINE_DEFAULT_DEBUG_CHANNEL(shell): debug channel handled by the crate's DPRINT macros.


pub type FLAGMAP = tagFLAGMAP;
pub type PFLAGMAP = *mut FLAGMAP;

pub type APPCOMPATINFO = tagAPPCOMPATINFO;
pub type PAPPCOMPATINFO = *mut APPCOMPATINFO;

pub type WNDCOMPATINFO = tagWNDCOMPATINFO;
pub type PWNDCOMPATINFO = *mut WNDCOMPATINFO;

// The first byte indicates the string length

#[repr(C)]
pub struct APPCOMPATENUM {
    pub pItems: PWNDCOMPATINFO,
    pub nItems: SIZE_T,
    pub dwProcessId: DWORD,
    pub iFound: INT,
}
pub type PAPPCOMPATENUM = *mut APPCOMPATENUM;

#[repr(C)]
pub struct LANGANDCODEPAGE {
    pub wLanguage: WORD,
    pub wCodePage: WORD,
}
pub type PLANGANDCODEPAGE = *mut LANGANDCODEPAGE;

pub const SHACF_TO_INIT: DWORD = SHACF_CONTEXTMENU
    | SHACF_DOCOBJECT
    | SHACF_CORELINTERNETENUM
    | SHACF_MYCOMPUTERFIRST
    | SHACF_OLDREGITEMGDN
    | SHACF_LOADCOLUMNHANDLER
    | SHACF_ANSI
    | SHACF_WIN95SHLEXEC
    | SHACF_STAROFFICE5PRINTER
    | SHACF_NOVALIDATEFSIDS
    | SHACF_FILEOPENNEEDSEXT
    | SHACF_WIN95BINDTOOBJECT
    | SHACF_IGNOREENUMRESET
    | SHACF_ANSIDISPLAYNAMES
    | SHACF_FILEOPENBOGUSCTRLID
    | SHACF_FORCELFNIDLIST;

unsafe extern "C" fn SHLWAPI_GetMappedFlags(hKey: HKEY, pEntries: *const FLAGMAP, nEntries: UINT) -> DWORD {
    let mut flags: DWORD = 0;
    let mut iEntry: UINT = 0;
    while iEntry < nEntries {
        let mut error: DWORD = SHGetValueA(
            hKey,
            core::ptr::null::<CHAR>(),
            (*pEntries.offset(iEntry as isize)).name,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::null_mut::<DWORD>(),
        );
        if error == ERROR_SUCCESS as DWORD {
            flags |= (*pEntries.offset(iEntry as isize)).flags;
        }
        iEntry = iEntry.wrapping_add(1);
    }
    return flags;
}

unsafe extern "system" fn SHLWAPI_WndCompatEnumProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    let pEnum: PAPPCOMPATENUM = lParam as PAPPCOMPATENUM;

    let mut szClass: [CHAR; 256] = [0; 256];
    if (*pEnum).nItems == 0
        || GetClassNameA(
            hWnd,
            core::ptr::addr_of_mut!(szClass) as *mut CHAR as LPSTR,
            _countof!(szClass) as INT,
        ) == 0
    {
        return (TRUE) as BOOL; // Ignore and continue
    }

    // Search for the target window in pEnum
    let cchClass: INT = lstrlenA(core::ptr::addr_of_mut!(szClass) as *mut CHAR as LPCSTR) as INT;
    let mut iItem: UINT = 0 as UINT;
    while (iItem as SIZE_T) < (*pEnum).nItems {
        let pszLengthAndClassName: PCSTR = (*(*pEnum).pItems.offset(iItem as isize)).pszLengthAndClassName;

        let mut cchLength: INT = *pszLengthAndClassName.offset(0 as isize) as INT; // The first byte represents the length
        if cchClass < cchLength {
            cchLength = cchClass; // Ignore trailing characters
        }

        // Compare the string
        if StrCmpNA(
            core::ptr::addr_of_mut!(szClass) as *mut CHAR as LPCSTR,
            pszLengthAndClassName.offset(1 as isize) as LPCSTR,
            cchLength,
        ) == 0 // Class name matched
        {
            // Get the process ID
            let mut dwProcessId: DWORD = 0;
            GetWindowThreadProcessId(hWnd, core::ptr::addr_of_mut!(dwProcessId));
            if dwProcessId == (*pEnum).dwProcessId // Same process
            {
                (*pEnum).iFound = iItem as INT; // Found
                return (FALSE) as BOOL; // Quit enumeration
            }
        }
        iItem = iItem.wrapping_add(1);
    }

    return (TRUE) as BOOL; // Continue enumeration
}

unsafe extern "C" fn SHLWAPI_GetModuleVersion(pszFileName: PCSTR, ppszDest: *mut PSTR) -> HRESULT {
    let mut dwHandle: DWORD = 0;
    let mut pbData: PBYTE = core::ptr::null_mut();
    let mut pvData: PVOID = core::ptr::null_mut();

    *ppszDest = core::ptr::null_mut();

    // Obtain the version info
    let mut size: UINT = GetFileVersionInfoSizeA(pszFileName, core::ptr::addr_of_mut!(dwHandle)) as UINT;
    if size == 0 {
        return E_FAIL;
    }
    pbData = LocalAlloc(LPTR, size as SIZE_T) as PBYTE;
    if pbData.is_null() {
        ERR!("E_OUTOFMEMORY\n");
        return E_OUTOFMEMORY;
    }
    GetFileVersionInfoA(pszFileName, dwHandle, size, pbData as PVOID);

    // Getting the product version
    let mut hr: HRESULT = E_FAIL;
    if (VerQueryValueA(
        pbData as LPCVOID,
        (PRODUCT_VER_ENGLISH_US_UTF16).as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(pvData),
        core::ptr::addr_of_mut!(size),
    ) != 0
        || VerQueryValueA(
            pbData as LPCVOID,
            (PRODUCT_VER_GERMAN_UTF16).as_ptr() as LPCSTR,
            core::ptr::addr_of_mut!(pvData),
            core::ptr::addr_of_mut!(size),
        ) != 0
        || VerQueryValueA(
            pbData as LPCVOID,
            (PRODUCT_VER_ENGLISH_US_WE).as_ptr() as LPCSTR,
            core::ptr::addr_of_mut!(pvData),
            core::ptr::addr_of_mut!(size),
        ) != 0
        || VerQueryValueA(
            pbData as LPCVOID,
            (PRODUCT_VER_ENGLISH_US_NEUTRAL).as_ptr() as LPCSTR,
            core::ptr::addr_of_mut!(pvData),
            core::ptr::addr_of_mut!(size),
        ) != 0
        || VerQueryValueA(
            pbData as LPCVOID,
            (PRODUCT_VER_SWEDISH_WE).as_ptr() as LPCSTR,
            core::ptr::addr_of_mut!(pvData),
            core::ptr::addr_of_mut!(size),
        ) != 0)
        && size != 0
    {
        // NOTE: *ppszDest must be freed using LocalFree later
        *ppszDest = StrDupA(pvData as PSTR) as PSTR;
        hr = if !(*ppszDest).is_null() {
            S_OK
        } else {
            E_OUTOFMEMORY
        };
    } else if VerQueryValueA(
        pbData as LPCVOID,
        b"\\VarFileInfo\\Translation\0".as_ptr() as LPCSTR,
        core::ptr::addr_of_mut!(pvData),
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        let mut pDataSaved: PVOID = pvData;
        let mut pEntry: PLANGANDCODEPAGE = pvData as PLANGANDCODEPAGE;
        while (pEntry as PBYTE).add(core::mem::size_of::<LANGANDCODEPAGE>())
            <= (pDataSaved as PBYTE).add(size as usize)
        {
            let mut szPath: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            wnsprintfA(
                core::ptr::addr_of_mut!(szPath) as *mut CHAR as LPSTR,
                (_countof!(szPath)) as INT,
                b"\\StringFileInfo\\%04X%04X\\ProductVersion\0".as_ptr() as LPCSTR,
                (*pEntry).wLanguage as core::ffi::c_int,
                (*pEntry).wCodePage as core::ffi::c_int,
            );
            if VerQueryValueA(
                pbData as LPCVOID,
                core::ptr::addr_of_mut!(szPath) as *mut CHAR as LPCSTR,
                core::ptr::addr_of_mut!(pvData),
                core::ptr::addr_of_mut!(size),
            ) != 0
                && size != 0
            {
                // NOTE: *ppszDest must be freed using LocalFree later
                *ppszDest = StrDupA(pvData as PSTR) as PSTR;
                hr = if !(*ppszDest).is_null() {
                    S_OK
                } else {
                    E_OUTOFMEMORY
                };
            }
            pEntry = pEntry.add(1);
        }
    }

    if hr < 0 {
        WARN!("hr: 0x%lX\n", hr);
    }

    LocalFree(pbData as HLOCAL);
    return hr;
}

unsafe extern "C" fn SHLWAPI_DoesModuleVersionMatch(
    pszFileName: PCSTR,
    pszVersionPattern: PCSTR,
) -> BOOL {
    if pszVersionPattern.is_null() {
        return (TRUE) as BOOL;
    }

    let mut pszModuleVersion: PSTR = core::ptr::null_mut();
    let mut hr: HRESULT = SHLWAPI_GetModuleVersion(pszFileName, core::ptr::addr_of_mut!(pszModuleVersion));
    if hr < 0 {
        return (FALSE) as BOOL;
    }

    let mut ret: BOOL = (FALSE as i32);
    if *pszVersionPattern.offset(0 as isize) as c_int
        == MAJOR_VER_ONLY[0] as c_int
    {
        // Truncate at comma (',') if present
        let mut pchComma: PSTR = StrChrA(pszModuleVersion as LPCSTR, ',' as i32 as WORD) as PSTR;
        if !pchComma.is_null() {
            *pchComma = ANSI_NULL;
        }

        // Truncate at dot ('.') if present
        let mut pchDot: PSTR = StrChrA(pszModuleVersion as LPCSTR, '.' as i32 as WORD) as PSTR;
        if !pchDot.is_null() {
            *pchDot = ANSI_NULL;
        }

        ret = (lstrcmpiA(
            pszModuleVersion as LPCSTR,
            pszVersionPattern.offset(1 as isize) as LPCSTR,
        ) == 0) as c_int as BOOL;
    } else {
        // Otherwise, perform a normal match
        let mut pchAsterisk: PSTR = StrChrA(pszVersionPattern as LPCSTR, '*' as i32 as WORD) as PSTR;
        if !pchAsterisk.is_null() {
            let mut cchPrefix: INT = pchAsterisk.offset_from(pszVersionPattern as PSTR) as isize as INT;
            if cchPrefix > 0 {
                ret = (StrCmpNIA(
                    pszModuleVersion as LPCSTR,
                    pszVersionPattern as LPCSTR,
                    cchPrefix,
                ) == 0) as c_int as BOOL;
            }
        }

        if ret == 0 {
            ret = (lstrcmpiA(pszModuleVersion as LPCSTR, pszVersionPattern as LPCSTR)
                == 0) as c_int as BOOL;
        }
    }

    LocalFree(pszModuleVersion as HLOCAL);
    ret
}

unsafe extern "C" fn SHLWAPI_GetRegistryCompatFlags(pszPath: PCSTR) -> DWORD {
    // Build the path to the "application compatibility" registry key
    let mut szText: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    wnsprintfA(
        core::ptr::addr_of_mut!(szText) as LPSTR,
        _countof!(szText) as INT,
        b"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\ShellCompatibility\\Applications\\%s\0" as *const u8 as LPCSTR,
        PathFindFileNameA(pszPath as LPCSTR),
    );

    // Open the key
    let mut hKey: HKEY = core::ptr::null_mut();
    let samDesired: REGSAM = (KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS) as REGSAM;
    let mut error: LSTATUS = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE as HKEY,
        core::ptr::addr_of_mut!(szText) as *mut CHAR as LPCSTR,
        0 as DWORD,
        samDesired,
        core::ptr::addr_of_mut!(hKey),
    );
    if error != ERROR_SUCCESS as LSTATUS {
        ERR!("error: %lu\n", error);
        return 0 as DWORD; // Failed
    }

    // Build the base directory
    let mut szBaseDir: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    lstrcpynA(
        core::ptr::addr_of_mut!(szBaseDir) as LPSTR,
        pszPath as LPCSTR,
        _countof!(szBaseDir) as INT,
    );
    PathRemoveFileSpecA(core::ptr::addr_of_mut!(szBaseDir) as LPSTR);

    // Search from the registry key
    let mut dwCompatFlags: DWORD = 0 as DWORD;
    szText[0 as usize] = ANSI_NULL as CHAR; // The first attempt is for the non-subkey path
    let mut dwIndex: DWORD = 0 as DWORD;
    while error == ERROR_SUCCESS as LSTATUS {
        let mut hSubKey: HKEY = core::ptr::null_mut();
        error = RegOpenKeyExA(
            hKey,
            core::ptr::addr_of_mut!(szText) as *mut CHAR as LPCSTR,
            0 as DWORD,
            KEY_QUERY_VALUE as REGSAM,
            core::ptr::addr_of_mut!(hSubKey),
        );
        if error != ERROR_SUCCESS as LSTATUS {
            break;
        }

        // Get the "RequiredFile" value
        let mut szRequired: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut cbData: DWORD = core::mem::size_of::<[CHAR; (MAX_PATH) as _]>() as DWORD;
        error = SHGetValueA(
            hSubKey,
            core::ptr::null(),
            b"RequiredFile\0" as *const u8 as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szRequired) as *mut CHAR as LPVOID,
            core::ptr::addr_of_mut!(cbData),
        ) as LSTATUS;
        let mut bValueExists: BOOL = (error == ERROR_SUCCESS as LSTATUS) as BOOL;
        let mut bRequiredFileExists: BOOL = FALSE as BOOL;
        if bValueExists != 0 // Does the "RequiredFile" value exist?
        {
            // Build required file path to szText
            PathCombineA(
                core::ptr::addr_of_mut!(szText) as LPSTR,
                core::ptr::addr_of_mut!(szBaseDir) as *mut CHAR as LPCSTR,
                core::ptr::addr_of_mut!(szRequired) as *mut CHAR as LPCSTR,
            );

            // Now szText is a full path
            TRACE!(
                "RequiredFile: %s\n",
                wine_dbgstr_a(core::ptr::addr_of_mut!(szRequired) as *mut CHAR)
            );
            TRACE!(
                "szText: %s\n",
                wine_dbgstr_a(core::ptr::addr_of_mut!(szText) as *mut CHAR)
            );
            bRequiredFileExists = (GetFileAttributesA(
                core::ptr::addr_of_mut!(szText) as *mut CHAR as LPCSTR,
            ) != INVALID_FILE_ATTRIBUTES as DWORD) as BOOL;
        }

        // If the "RequiredFile" value doesn't exist, or the file specified by szText exists:
        if bValueExists == 0 || bRequiredFileExists != 0 {
            // Check the "Version" value if necessary
            error = SHGetValueA(
                hSubKey,
                core::ptr::null(),
                b"Version\0" as *const u8 as LPCSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szText) as *mut CHAR as LPVOID,
                core::ptr::addr_of_mut!(cbData),
            ) as LSTATUS;
            let mut pszVersionPattern: PCSTR = (if error == ERROR_SUCCESS as LSTATUS {
                core::ptr::addr_of_mut!(szText) as *mut CHAR
            } else {
                core::ptr::null_mut()
            }) as PCSTR;
            TRACE!(
                "pszVersionPattern: %s\n",
                wine_dbgstr_a(pszVersionPattern)
            );

            // Does the pattern match?
            if SHLWAPI_DoesModuleVersionMatch(pszPath, pszVersionPattern) != 0 {
                // Add additional flags from the registry key
                dwCompatFlags |= SHLWAPI_GetMappedFlags(
                    hSubKey,
                    core::ptr::addr_of_mut!(g_appCompatFlagMaps) as *mut FLAGMAP,
                    _countof!(g_appCompatFlagMaps) as UINT,
                );
            }
        }

        RegCloseKey(hSubKey);

        // Go to the next sub-key
        dwIndex = dwIndex.wrapping_add(1);
        error = RegEnumKeyA(
            hKey,
            dwIndex,
            core::ptr::addr_of_mut!(szText) as LPSTR,
            _countof!(szText) as DWORD,
        );
    }

    RegCloseKey(hKey);

    return dwCompatFlags;
}

unsafe extern "C" fn SHLWAPI_InitAppCompat() -> DWORD {
    if GetProcessVersion(0) >= MAKELONG!(0, 5) as DWORD {
        return 0; // Flags are not needed
    }

    // Get module pathname
    let mut szModulePathA: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    if GetModuleFileNameA(core::ptr::null_mut(), szModulePathA.as_mut_ptr(), _countof!(szModulePathA)) == 0 {
        return 0;
    }

    let pszFileName: PCSTR = PathFindFileNameA(szModulePathA.as_mut_ptr() as LPCSTR) as PCSTR; // Get the file title from the path

    // Search the file title from g_appCompatInfo
    let mut dwAppCompatFlags: DWORD = 0;
    let mut iItem: UINT = 0;
    while (iItem as usize) < _countof!(g_appCompatInfo) as usize {
        let pInfo: *const APPCOMPATINFO = core::ptr::addr_of!(g_appCompatInfo[iItem as usize]);
        if lstrcmpiA((*pInfo).pszAppName as LPCSTR, pszFileName as LPCSTR) == 0
            && SHLWAPI_DoesModuleVersionMatch(pszFileName, (*pInfo).pszAppVersion) != 0
        {
            // Found, set the flags
            dwAppCompatFlags = g_appCompatInfo[iItem as usize].dwCompatFlags;
            break;
        }
        iItem += 1;
    }

    // Add additional flags from the registry
    dwAppCompatFlags |= SHLWAPI_GetRegistryCompatFlags(pszFileName);

    return dwAppCompatFlags;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetAppCompatFlags(dwMask: DWORD) -> DWORD {
    // Initialize and retrieve flags if necessary
    if (dwMask & SHACF_TO_INIT) != 0 {
        if g_fAppCompatInitialized == 0 {
            let dwAppCompatFlags: DWORD = SHLWAPI_InitAppCompat();
            InterlockedExchange(
                addr_of_mut!(g_dwAppCompatFlags) as PLONG as *mut ::core::ffi::c_long,
                dwAppCompatFlags as ::core::ffi::c_long,
            );

            g_fAppCompatInitialized = (TRUE as i32); // Mark as initialized
        }
    }

    // Retrieve additional flags if necessary
    let mut dwAppCompatFlags: DWORD = g_dwAppCompatFlags;
    if dwAppCompatFlags != 0 && (dwMask & (SHACF_UNKNOWN1 | SHACF_UNKNOWN2)) != 0 {
        // Find the target window and its flags using g_wndCompatInfo
        let mut data: APPCOMPATENUM = APPCOMPATENUM {
            pItems: addr_of_mut!(g_wndCompatInfo) as PWNDCOMPATINFO,
            nItems: _countof!(g_wndCompatInfo) as _,
            dwProcessId: GetCurrentProcessId(),
            iFound: -1,
        };
        EnumWindows(
            Some(SHLWAPI_WndCompatEnumProc),
            addr_of_mut!(data) as LPARAM,
        );

        // Add the target flags if a match is found
        if data.iFound >= 0 {
            dwAppCompatFlags |= g_wndCompatInfo[data.iFound as usize].dwCompatFlags;
        }

        dwAppCompatFlags |= SHACF_UNKNOWN3;

        InterlockedExchange(
            addr_of_mut!(g_dwAppCompatFlags) as PLONG as *mut ::core::ffi::c_long,
            dwAppCompatFlags as ::core::ffi::c_long,
        );
    }

    return dwAppCompatFlags;
}

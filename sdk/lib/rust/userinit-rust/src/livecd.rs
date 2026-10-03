use winsrv_sys::winsrv::*;
use crate::types::*;
use crate::types::hInstance;
use crate::preamble::*;
// ReadRegSzKey / ExpandInstallerPath are #[no_mangle] in userinit.rs (defined
// in userinit.c, used here cross-TU as in the original C).
use crate::userinit::{ReadRegSzKey, ExpandInstallerPath};
use crate::preamble::{EnumSystemLocalesW, DialogBoxParamW};

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


// CRT (swprintf/wcslen/wcstoul/_wcsicmp/wcstol/…) lives in crate::preamble.

// ---- file-scope preamble (translated) ----
pub static mut hList: HWND = core::ptr::null_mut();
pub static mut hLocaleList: HWND = core::ptr::null_mut();
pub static mut bSpain: BOOL = 0;

// _LIVECD_UNATTEND / LIVECD_UNATTEND now live in crate::types (shared).

unsafe fn InitLogo(pImgInfo: PIMGINFO, hwndDlg: HWND) {
    let mut logoBitmap: BITMAP = core::mem::zeroed();
    let mut maskBitmap: BITMAP = core::mem::zeroed();
    let mut bmpi: BITMAPINFO = core::mem::zeroed();
    let hDC: HDC;
    let hDCLogo: HDC;
    let hDCMask: HDC;
    let mut hMask: HBITMAP = core::ptr::null_mut();
    let mut hLogo: HBITMAP = core::ptr::null_mut();
    let mut hAlphaLogo: HBITMAP = core::ptr::null_mut();
    let mut pBits: *mut COLORREF = core::ptr::null_mut();
    let mut line: INT;
    let mut column: INT;

    hDC = GetDC(hwndDlg);
    hDCLogo = CreateCompatibleDC(core::ptr::null_mut());
    hDCMask = CreateCompatibleDC(core::ptr::null_mut());

    'cleanup: {
        if hDC.is_null() || hDCLogo.is_null() || hDCMask.is_null() {
            break 'cleanup;
        }

        core::ptr::write_bytes(pImgInfo as *mut u8, 0, core::mem::size_of::<IMGINFO>());
        core::ptr::write_bytes(&mut bmpi as *mut BITMAPINFO as *mut u8, 0, core::mem::size_of::<BITMAPINFO>());

        hLogo = LoadImageW(hInstance, MAKEINTRESOURCEW(IDB_ROSLOGO), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR) as HBITMAP;
        hMask = LoadImageW(hInstance, MAKEINTRESOURCEW(IDB_ROSMASK), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR) as HBITMAP;

        if hLogo.is_null() || hMask.is_null() {
            break 'cleanup;
        }

        GetObjectW(hLogo as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<BITMAP>() as i32, &mut logoBitmap as *mut BITMAP as *mut core::ffi::c_void);
        GetObjectW(hMask as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<BITMAP>() as i32, &mut maskBitmap as *mut BITMAP as *mut core::ffi::c_void);

        if logoBitmap.bmHeight != maskBitmap.bmHeight || logoBitmap.bmWidth != maskBitmap.bmWidth {
            break 'cleanup;
        }

        bmpi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFO>() as u32;
        bmpi.bmiHeader.biWidth = logoBitmap.bmWidth;
        bmpi.bmiHeader.biHeight = logoBitmap.bmHeight;
        bmpi.bmiHeader.biPlanes = 1;
        bmpi.bmiHeader.biBitCount = 32;
        bmpi.bmiHeader.biCompression = BI_RGB as u32;
        bmpi.bmiHeader.biSizeImage = (4 * logoBitmap.bmWidth * logoBitmap.bmHeight) as u32;

        hAlphaLogo = CreateDIBSection(hDC, &mut bmpi, DIB_RGB_COLORS, &mut pBits as *mut *mut COLORREF as *mut *mut core::ffi::c_void, core::ptr::null_mut(), 0);
        if hAlphaLogo.is_null() {
            break 'cleanup;
        }

        SelectObject(hDCLogo, hLogo as *mut core::ffi::c_void);
        SelectObject(hDCMask, hMask as *mut core::ffi::c_void);

        line = logoBitmap.bmHeight - 1;
        while line >= 0 {
            column = 0;
            while column < logoBitmap.bmWidth {
                let alpha: COLORREF = GetPixel(hDCMask, column, line) & 0xFF;
                let Color: COLORREF = GetPixel(hDCLogo, column, line);
                let r: DWORD;
                let g: DWORD;
                let b: DWORD;

                r = GetRValue(Color) as DWORD * alpha / 255;
                g = GetGValue(Color) as DWORD * alpha / 255;
                b = GetBValue(Color) as DWORD * alpha / 255;

                *pBits = b | (g << 8) | (r << 16) | (alpha << 24);
                pBits = pBits.add(1);

                column += 1;
            }
            line -= 1;
        }

        (*pImgInfo).hBitmap = hAlphaLogo;
        (*pImgInfo).cxSource = logoBitmap.bmWidth;
        (*pImgInfo).cySource = logoBitmap.bmHeight;
        (*pImgInfo).iBits = logoBitmap.bmBitsPixel as INT;
        (*pImgInfo).iPlanes = logoBitmap.bmPlanes as INT;
    }

    if !hMask.is_null() {
        DeleteObject(hMask as *mut core::ffi::c_void);
    }
    if !hLogo.is_null() {
        DeleteObject(hLogo as *mut core::ffi::c_void);
    }
    if !hDCMask.is_null() {
        DeleteDC(hDCMask);
    }
    if !hDCLogo.is_null() {
        DeleteDC(hDCLogo);
    }
    if !hDC.is_null() {
        ReleaseDC(hwndDlg, hDC);
    }
}
#[no_mangle]
pub unsafe extern "C" fn IsLiveCD() -> BOOL {
    let mut ControlKey: HKEY = core::ptr::null_mut();
    let mut SystemStartOptions: LPWSTR = core::ptr::null_mut();
    let mut CurrentOption: LPWSTR;
    let mut NextOption: LPWSTR;
    let mut rc: LONG;
    let mut ret: BOOL = 0;

    'cleanup: {
        rc = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            REGSTR_PATH_CURRENT_CONTROL_SET(),
            0,
            KEY_QUERY_VALUE,
            &mut ControlKey,
        );
        if rc != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }

        rc = ReadRegSzKey(
            ControlKey,
            w!("SystemStartOptions").as_ptr() as LPWSTR,
            &mut SystemStartOptions,
        );
        if rc != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }

        CurrentOption = SystemStartOptions;
        while !CurrentOption.is_null() {
            NextOption = wcschr(CurrentOption, b' ' as u16);
            if !NextOption.is_null() {
                *NextOption = 0;
            }
            if _wcsicmp(
                CurrentOption,
                w!("MININT").as_ptr() as LPWSTR,
            ) == 0
            {
                ret = 1;
                break 'cleanup;
            }
            CurrentOption = if !NextOption.is_null() {
                NextOption.add(1)
            } else {
                core::ptr::null_mut()
            };
        }
    }

    if !ControlKey.is_null() {
        RegCloseKey(ControlKey);
    }
    HeapFree(GetProcessHeap(), 0, SystemStartOptions as *mut _);

    ret
}
unsafe extern "system" fn LocalesEnumProc(lpLocale: LPTSTR) -> BOOL {
    let lcid: LCID;
    let mut lang: [WCHAR; 255] = [0; 255];
    let index: INT;
    let mut bNoShow: BOOL = FALSE as BOOL;

    lcid = wcstoul(lpLocale, core::ptr::null_mut(), 16) as LCID;

    if IsValidLocale(lcid, LCID_INSTALLED) == 0 {
        return TRUE as BOOL;
    }

    if lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT)
        || lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT)
    {
        if bSpain == FALSE as BOOL {
            LoadStringW(hInstance, IDS_SPAIN, lang.as_mut_ptr(), ARRAYSIZE(&lang) as INT);
            bSpain = TRUE as BOOL;
        } else {
            bNoShow = TRUE as BOOL;
        }
    } else {
        GetLocaleInfoW(lcid, LOCALE_SLANGUAGE, lang.as_mut_ptr(), ARRAYSIZE(&lang) as INT);
    }

    if bNoShow == FALSE as BOOL {
        index = SendMessageW(hList, CB_ADDSTRING, 0, lang.as_ptr() as LPARAM) as INT;

        SendMessageW(hList, CB_SETITEMDATA, index as WPARAM, lcid as LPARAM);
    }

    TRUE as BOOL
}

unsafe fn CreateLanguagesList(hwnd: HWND, pState: PSTATE) {
    let mut langSel: [u16; 255] = [0; 255];
    let mut Locale: LCID = 0;

    hList = hwnd;
    bSpain = 0;
    EnumSystemLocalesW(Some(LocalesEnumProc), LCID_SUPPORTED);

    if (*(*pState).Unattend).bEnabled != 0 {
        Locale = (*(*pState).Unattend).LocaleID;
    }

    if Locale == 0 {
        Locale = GetUserDefaultLCID();
    }
    GetLocaleInfoW(Locale, LOCALE_SLANGUAGE, langSel.as_mut_ptr(), langSel.len() as i32);

    SendMessageW(hList, CB_SELECTSTRING, -1i32 as WPARAM, langSel.as_ptr() as LPARAM);
}

/// Upstream f69f04bc67c's rewrite. The old body leaked the layout DLL on every successful
/// LoadStringW and the registry key when "Layout Text" failed, loaded the DLL as code
/// instead of as a data file, and parsed "@file,-id" by shifting the string in place.
unsafe fn GetLayoutName(
    pszKLID: LPCWSTR,
    pszName: LPWSTR,
    NameSize: usize,
) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut ret: BOOL;
    let mut dwBufLen: DWORD;
    let mut szBuf: [u16; 1 + MAX_PATH as usize + 2 + 10] = [0; 1 + MAX_PATH as usize + 2 + 10]; // "@file_path,-resID"
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if NameSize < 1 {
        return 0;
    }
    *pszName = 0;

    wsprintfW(szBuf.as_mut_ptr(), L_STR_FMT(), pszKLID);
    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuf.as_ptr(), 0, KEY_QUERY_VALUE, &mut hKey) != ERROR_SUCCESS as i32 {
        return 0;
    }

    'quit: {
        dwBufLen = core::mem::size_of_val(&szBuf) as DWORD;
        if RegQueryValueExW(
            hKey,
            L_DISPNAME(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            szBuf.as_mut_ptr() as LPBYTE,
            &mut dwBufLen,
        ) == ERROR_SUCCESS as i32
            && szBuf[0] == '@' as u16
        {
            'parse: {
                // wcsrchr(szBuf, L',')
                let Len = wcslen(szBuf.as_ptr());
                let Some(Comma) = (0..Len).rev().find(|&i| szBuf[i] == ',' as u16) else {
                    break 'parse;
                };
                szBuf[Comma] = 0;
                let DllName = szBuf.as_ptr().add(1);

                let mut Index = wcstol(szBuf.as_ptr().add(Comma + 1), core::ptr::null_mut(), 10) as i32;
                if Index >= 0 {
                    // Upstream: "Cannot handle non-negative indices".
                    break 'parse;
                }
                Index = -Index;

                if ExpandEnvironmentStringsW(DllName, szPath.as_mut_ptr(), szPath.len() as DWORD) != 0 {
                    let hLib = LoadLibraryExW(szPath.as_ptr(), core::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE);
                    if !hLib.is_null() {
                        ret = (LoadStringW(hLib as _, Index as UINT, szPath.as_mut_ptr(), szPath.len() as i32) != 0) as BOOL;
                        FreeLibrary(hLib as _);
                        if ret != 0 {
                            // StringCchCopyW(pszName, NameSize, szPath)
                            let mut i = 0usize;
                            while i + 1 < NameSize && szPath[i] != 0 {
                                *pszName.add(i) = szPath[i];
                                i += 1;
                            }
                            *pszName.add(i) = 0;
                            break 'quit;
                        }
                    }
                }
            }
        }

        dwBufLen = (NameSize * core::mem::size_of::<WCHAR>()) as DWORD;
        ret = (RegQueryValueExW(
            hKey,
            L_LAYOUTTEXT(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pszName as LPBYTE,
            &mut dwBufLen,
        ) == ERROR_SUCCESS as i32) as BOOL;
    }

    RegCloseKey(hKey);
    ret
}

unsafe fn SetKeyboardLayout(hwnd: HWND) {
    let iCurSel: INT;
    let ulLayoutId: ULONG;
    let hKl: HKL;
    let mut szLayoutId: [WCHAR; 9] = [0; 9];

    iCurSel = SendMessageW(hwnd, CB_GETCURSEL, 0, 0) as INT;
    if iCurSel == CB_ERR as INT {
        return;
    }

    ulLayoutId = SendMessageW(hwnd, CB_GETITEMDATA, iCurSel as WPARAM, 0) as ULONG;
    if ulLayoutId == CB_ERR as ULONG {
        return;
    }

    let fmt: [WCHAR; 6] = [
        '%' as WCHAR,
        '0' as WCHAR,
        '8' as WCHAR,
        'l' as WCHAR,
        'x' as WCHAR,
        0,
    ];
    swprintf(szLayoutId.as_mut_ptr(), fmt.as_ptr(), ulLayoutId);

    hKl = LoadKeyboardLayoutW(
        szLayoutId.as_ptr(),
        (KLF_ACTIVATE | KLF_REPLACELANG | KLF_SETFORPROCESS) as UINT,
    );
    SystemParametersInfoW(
        SPI_SETDEFAULTINPUTLANG as UINT,
        0,
        core::ptr::addr_of!(hKl) as *mut core::ffi::c_void,
        SPIF_SENDCHANGE as UINT,
    );
}
unsafe fn SelectKeyboardForLanguage(
    hwnd: HWND,
    lcid: LCID) {
    let mut i: i32;
    let nCount: i32;
    let mut LayoutId: LCID;

    nCount = SendMessageW(hwnd, CB_GETCOUNT, 0, 0) as i32;

    i = 0;
    while i < nCount {
        LayoutId = SendMessageW(hwnd, CB_GETITEMDATA, i as WPARAM, 0) as LCID;

        if LANGIDFROMLCID(LayoutId as u32) == LANGIDFROMLCID(lcid as u32) {
            SendMessageW(hwnd, CB_SETCURSEL, i as WPARAM, 0);
            return;
        }
        i += 1;
    }

    i = 0;
    while i < nCount {
        LayoutId = SendMessageW(hwnd, CB_GETITEMDATA, i as WPARAM, 0) as LCID;

        if PRIMARYLANGID(LayoutId) == PRIMARYLANGID(lcid) {
            SendMessageW(hwnd, CB_SETCURSEL, i as WPARAM, 0);
            return;
        }
        i += 1;
    }
}

unsafe fn CreateKeyboardLayoutList(
    hItemsList: HWND,
) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szLayoutId: [u16; 9] = [0; 9];
    let mut szCurrentLayoutId: [u16; 9] = [0; 9];
    let mut KeyName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwIndex: DWORD = 0;
    let mut dwSize: DWORD;
    let _iIndex: INT;
    let mut lError: LONG;
    let _ulLayoutId: ULONG;

    if GetKeyboardLayoutNameW(szCurrentLayoutId.as_mut_ptr()) == 0 {
        wcscpy(szCurrentLayoutId.as_mut_ptr(), L_00000409());
    }

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        L_KeyboardLayouts(),
        0,
        KEY_ENUMERATE_SUB_KEYS,
        &mut hKey,
    );
    if lError != ERROR_SUCCESS as LONG {
        return;
    }

    loop {
        dwSize = szLayoutId.len() as DWORD;

        lError = RegEnumKeyExW(
            hKey,
            dwIndex,
            szLayoutId.as_mut_ptr(),
            &mut dwSize,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if lError != ERROR_SUCCESS as LONG {
            break;
        }

        GetLayoutName(szLayoutId.as_ptr(), KeyName.as_mut_ptr(), KeyName.len());

        let iIndex = SendMessageW(
            hItemsList,
            CB_ADDSTRING,
            0,
            KeyName.as_ptr() as LPARAM,
        ) as INT;

        let ulLayoutId = wcstoul(szLayoutId.as_ptr(), core::ptr::null_mut(), 16);
        SendMessageW(
            hItemsList,
            CB_SETITEMDATA,
            iIndex as WPARAM,
            ulLayoutId as LPARAM,
        );

        // Case-insensitive (upstream c45c83a4dc2): GetKeyboardLayoutNameW returns uppercase
        // hex, the registry subkeys may not be, so the current layout was never preselected.
        if _wcsicmp(szLayoutId.as_ptr(), szCurrentLayoutId.as_ptr()) == 0 {
            SendMessageW(
                hItemsList,
                CB_SETCURSEL,
                iIndex as WPARAM,
                0 as LPARAM,
            );
        }

        dwIndex += 1;
    }

    RegCloseKey(hKey);
}

unsafe fn InitializeDefaultUserLocale(pNewLcid: PLCID) {

    #[repr(C)]
    struct LocaleDataEntry {
        LCType: LCTYPE,
        pValue: PWSTR,
    }

    let mut szBuffer: [u16; 80] = [0; 80];
    let mut ptr: PWSTR = core::ptr::null_mut();
    let mut hLocaleKey: HKEY = core::ptr::null_mut();
    let mut ret: u32;
    let mut dwSize: u32;
    let lcid: LCID;

    const HKEY_USERS: HKEY = 0x80000003u32 as usize as HKEY;

    let locale_data: [LocaleDataEntry; 37] = [
        LocaleDataEntry { LCType: LOCALE_SDECIMAL, pValue: w!("sDecimal").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_STHOUSAND, pValue: w!("sThousand").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SNEGATIVESIGN, pValue: w!("sNegativeSign").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SPOSITIVESIGN, pValue: w!("sPositiveSign").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SGROUPING, pValue: w!("sGrouping").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SLIST, pValue: w!("sList").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SNATIVEDIGITS, pValue: w!("sNativeDigits").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_INEGNUMBER, pValue: w!("iNegNumber").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_IDIGITS, pValue: w!("iDigits").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ILZERO, pValue: w!("iLZero").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_IMEASURE, pValue: w!("iMeasure").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_IDIGITSUBSTITUTION, pValue: w!("NumShape").as_ptr() as PWSTR },

        LocaleDataEntry { LCType: LOCALE_SCURRENCY, pValue: w!("sCurrency").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SMONDECIMALSEP, pValue: w!("sMonDecimalSep").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SMONTHOUSANDSEP, pValue: w!("sMonThousandSep").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SMONGROUPING, pValue: w!("sMonGrouping").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ICURRENCY, pValue: w!("iCurrency").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_INEGCURR, pValue: w!("iNegCurr").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ICURRDIGITS, pValue: w!("iCurrDigits").as_ptr() as PWSTR },

        LocaleDataEntry { LCType: LOCALE_STIMEFORMAT, pValue: w!("sTimeFormat").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_STIME, pValue: w!("sTime").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_S1159, pValue: w!("s1159").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_S2359, pValue: w!("s2359").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ITIME, pValue: w!("iTime").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ITIMEMARKPOSN, pValue: w!("iTimePrefix").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ITLZERO, pValue: w!("iTLZero").as_ptr() as PWSTR },

        LocaleDataEntry { LCType: LOCALE_SLONGDATE, pValue: w!("sLongDate").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SSHORTDATE, pValue: w!("sShortDate").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SDATE, pValue: w!("sDate").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_IFIRSTDAYOFWEEK, pValue: w!("iFirstDayOfWeek").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_IFIRSTWEEKOFYEAR, pValue: w!("iFirstWeekOfYear").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_IDATE, pValue: w!("iDate").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ICALENDARTYPE, pValue: w!("iCalendarType").as_ptr() as PWSTR },

        LocaleDataEntry { LCType: LOCALE_SCOUNTRY, pValue: w!("sCountry").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_SABBREVLANGNAME, pValue: w!("sLanguage").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: LOCALE_ICOUNTRY, pValue: w!("iCountry").as_ptr() as PWSTR },
        LocaleDataEntry { LCType: 0, pValue: core::ptr::null_mut() },
    ];

    let key_path: [u16; 41] = [
        0x2e, 0x44, 0x45, 0x46, 0x41, 0x55, 0x4c, 0x54, 0x5c, 0x43, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c,
        0x20, 0x50, 0x61, 0x6e, 0x65, 0x6c, 0x5c, 0x49, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x74, 0x69,
        0x6f, 0x6e, 0x61, 0x6c, 0x00, 0, 0, 0, 0,
    ];

    ret = RegOpenKeyExW(
        HKEY_USERS,
        key_path.as_ptr() as PWSTR,
        0,
        KEY_READ | KEY_WRITE,
        &mut hLocaleKey,
    ) as u32;
    if ret != ERROR_SUCCESS {
        return;
    }

    let locale_name: [u16; 7] = [0x4c, 0x6f, 0x63, 0x61, 0x6c, 0x65, 0x00];
    let fmt: [u16; 6] = [0x25, 0x30, 0x38, 0x6c, 0x78, 0x00];

    'done: {
        if pNewLcid.is_null() {
            dwSize = (9 * core::mem::size_of::<u16>()) as u32;
            ret = RegQueryValueExW(
                hLocaleKey,
                locale_name.as_ptr() as PWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                szBuffer.as_mut_ptr() as PBYTE,
                &mut dwSize,
            ) as u32;
            if ret != ERROR_SUCCESS {
                break 'done;
            }

            lcid = wcstoul(szBuffer.as_ptr(), &mut ptr, 16) as LCID;
            if lcid == 0 {
                break 'done;
            }
        } else {
            lcid = *pNewLcid;

            swprintf(szBuffer.as_mut_ptr(), fmt.as_ptr(), lcid);
            RegSetValueExW(
                hLocaleKey,
                locale_name.as_ptr() as PWSTR,
                0,
                REG_SZ,
                szBuffer.as_ptr() as PBYTE,
                ((wcslen(szBuffer.as_ptr()) + 1) * core::mem::size_of::<u16>()) as u32,
            );
        }

        let mut i = 0;
        while !locale_data[i].pValue.is_null() {
            if GetLocaleInfoW(
                lcid,
                locale_data[i].LCType | LOCALE_NOUSEROVERRIDE,
                szBuffer.as_mut_ptr(),
                szBuffer.len() as i32,
            ) != 0
            {
                RegSetValueExW(
                    hLocaleKey,
                    locale_data[i].pValue,
                    0,
                    REG_SZ,
                    szBuffer.as_ptr() as PBYTE,
                    ((wcslen(szBuffer.as_ptr()) + 1) * core::mem::size_of::<u16>()) as u32,
                );
            }

            i += 1;
        }
    }

    RegCloseKey(hLocaleKey);
}
#[no_mangle]
pub unsafe extern "C" fn CenterWindow(hWnd: HWND) {
    let mut hWndParent: HWND;
    let mut rcParent: RECT = core::mem::zeroed();
    let mut rcWindow: RECT = core::mem::zeroed();

    hWndParent = GetParent(hWnd);
    if hWndParent == core::ptr::null_mut() {
        hWndParent = GetDesktopWindow();
    }

    GetWindowRect(hWndParent, core::ptr::addr_of_mut!(rcParent));
    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rcWindow));

    SetWindowPos(
        hWnd,
        core::ptr::null_mut(),
        ((rcParent.right - rcParent.left) - (rcWindow.right - rcWindow.left)) / 2,
        ((rcParent.bottom - rcParent.top) - (rcWindow.bottom - rcWindow.top)) / 2,
        0,
        0,
        SWP_NOSIZE,
    );
}

unsafe fn OnDrawItem(
    lpDrawItem: LPDRAWITEMSTRUCT,
    pState: PSTATE,
    uCtlID: UINT,
) {
    let hdcMem: HDC;
    let left: LONG;

    if (*lpDrawItem).CtlID == uCtlID {
        left = ((*lpDrawItem).rcItem.right - (*pState).ImageInfo.cxSource) / 2;

        hdcMem = CreateCompatibleDC((*lpDrawItem).hDC);
        if !hdcMem.is_null() {
            let BlendFunc = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            SelectObject(hdcMem, (*pState).ImageInfo.hBitmap as *mut core::ffi::c_void);
            GdiAlphaBlend(
                (*lpDrawItem).hDC,
                left,
                (*lpDrawItem).rcItem.top,
                (*pState).ImageInfo.cxSource,
                (*pState).ImageInfo.cySource,
                hdcMem,
                0,
                0,
                (*pState).ImageInfo.cxSource,
                (*pState).ImageInfo.cySource,
                BlendFunc,
            );
            DeleteDC(hdcMem);
        }
    }
}
unsafe extern "system" fn LocaleDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {

    let mut pState: PSTATE;

    pState = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PSTATE;

    match uMsg {
        WM_INITDIALOG => {
            pState = lParam as PSTATE;
            SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, pState as DWORD_PTR as LONG_PTR);

            CenterWindow(hwndDlg);

            CreateLanguagesList(GetDlgItem(hwndDlg, IDC_LANGUAGELIST as INT), pState);
            CreateKeyboardLayoutList(GetDlgItem(hwndDlg, IDC_LAYOUTLIST as INT));
            if (*(*pState).Unattend).bEnabled != 0 {
                PostMessageW(
                    hwndDlg,
                    WM_COMMAND,
                    ((IDOK as u32) | ((BN_CLICKED as u32) << 16)) as WPARAM,
                    0,
                );
            }
            return 0;
        }

        WM_DRAWITEM => {
            OnDrawItem(lParam as LPDRAWITEMSTRUCT, pState, IDC_LOCALELOGO);
            return 1;
        }

        WM_COMMAND => {
            match (wParam & 0xFFFF) as u32 {
                IDC_LANGUAGELIST => {
                    if ((wParam >> 16) & 0xFFFF) as u32 == CBN_SELCHANGE {
                        let NewLcid: LCID;
                        let iCurSel: INT;

                        iCurSel = SendDlgItemMessageW(
                            hwndDlg,
                            IDC_LANGUAGELIST as INT,
                            CB_GETCURSEL,
                            0,
                            0,
                        ) as INT;
                        if iCurSel == CB_ERR as INT {
                            return 0;
                        }

                        NewLcid = SendDlgItemMessageW(
                            hwndDlg,
                            IDC_LANGUAGELIST as INT,
                            CB_GETITEMDATA,
                            iCurSel as WPARAM,
                            0,
                        ) as LCID;
                        if NewLcid == CB_ERR as LCID {
                            return 0;
                        }

                        SelectKeyboardForLanguage(
                            GetDlgItem(hwndDlg, IDC_LAYOUTLIST as INT),
                            NewLcid,
                        );
                    }
                }

                IDOK => {
                    if ((wParam >> 16) & 0xFFFF) as u32 == BN_CLICKED {
                        let mut NewLcid: LCID;
                        let iCurSel: INT;

                        iCurSel = SendDlgItemMessageW(
                            hwndDlg,
                            IDC_LANGUAGELIST as INT,
                            CB_GETCURSEL,
                            0,
                            0,
                        ) as INT;
                        if iCurSel == CB_ERR as INT {
                            return 0;
                        }

                        NewLcid = SendDlgItemMessageW(
                            hwndDlg,
                            IDC_LANGUAGELIST as INT,
                            CB_GETITEMDATA,
                            iCurSel as WPARAM,
                            0,
                        ) as LCID;
                        if NewLcid == CB_ERR as LCID {
                            return 0;
                        }

                        NtSetDefaultLocale(1u8, NewLcid);

                        InitializeDefaultUserLocale(core::ptr::addr_of_mut!(NewLcid));

                        SetThreadLocale(NewLcid);

                        SetKeyboardLayout(GetDlgItem(hwndDlg, IDC_LAYOUTLIST as INT));

                        (*pState).NextPage = STARTPAGE;
                        EndDialog(hwndDlg, (wParam & 0xFFFF) as INT_PTR);
                    }
                }

                IDCANCEL => {
                    if ((wParam >> 16) & 0xFFFF) as u32 == BN_CLICKED {
                        static mut szMsg: [u16; RC_STRING_MAX_SIZE as usize] =
                            [0; RC_STRING_MAX_SIZE as usize];
                        let ret: INT;
                        LoadStringW(
                            GetModuleHandleW(core::ptr::null_mut()) as _,
                            IDS_CANCEL_CONFIRM,
                            szMsg.as_mut_ptr(),
                            szMsg.len() as INT,
                        );
                        ret = MessageBoxW(
                            hwndDlg,
                            szMsg.as_ptr(),
                            L_ReactOS_LiveCD(),
                            MB_ICONWARNING | MB_YESNO | MB_DEFBUTTON2,
                        );
                        if ret == IDOK as INT || ret == IDYES as INT {
                            (*pState).NextPage = DONE;
                            (*pState).Run = REBOOT;
                            EndDialog(hwndDlg, (wParam & 0xFFFF) as INT_PTR);
                        }
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    0
}
unsafe extern "system" fn StartDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pState: PSTATE;

    pState = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PSTATE;

    match uMsg {
        WM_INITDIALOG => {
            let mut Installer: [WCHAR; MAX_PATH as usize] = core::mem::zeroed();

            pState = lParam as PSTATE;
            SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, pState as DWORD_PTR as LONG_PTR);

            CenterWindow(hwndDlg);

            Installer[0] = UNICODE_NULL as WCHAR;
            if ExpandInstallerPath(
                winsrv_sys::w!("reactos.exe").as_ptr(),
                Installer.as_mut_ptr(),
                Installer.len() as _,
            ) == 0
            {
                EnableWindow(GetDlgItem(hwndDlg, IDC_INSTALL as INT), FALSE as i32);
            }

            if (*(*pState).Unattend).bEnabled != 0 || Installer[0] == UNICODE_NULL as WCHAR {
                PostMessageW(
                    hwndDlg,
                    WM_COMMAND,
                    MAKEWPARAM(IDC_RUN as u16, BN_CLICKED as u16) as WPARAM,
                    0,
                );
            }
            return FALSE as INT_PTR;
        }

        WM_DRAWITEM => {
            OnDrawItem(lParam as LPDRAWITEMSTRUCT, pState, IDC_STARTLOGO);
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            if HIWORD(wParam as u32) == BN_CLICKED as u16 {
                match LOWORD(wParam as u32) as u32 {
                    IDC_RUN => {
                        (*pState).NextPage = DONE;
                        (*pState).Run = SHELL;
                        EndDialog(hwndDlg, LOWORD(wParam as u32) as INT_PTR);
                    }

                    IDC_INSTALL => {
                        (*pState).NextPage = DONE;
                        (*pState).Run = INSTALLER;
                        EndDialog(hwndDlg, LOWORD(wParam as u32) as INT_PTR);
                    }

                    IDOK => {
                        (*pState).NextPage = LOCALEPAGE;
                        EndDialog(hwndDlg, LOWORD(wParam as u32) as INT_PTR);
                    }

                    IDCANCEL => {
                        if HIWORD(wParam as u32) == BN_CLICKED as u16 {
                            static mut szMsg: [WCHAR; RC_STRING_MAX_SIZE as usize] =
                                [0; RC_STRING_MAX_SIZE as usize];
                            let ret: INT;
                            LoadStringW(
                                GetModuleHandleW(core::ptr::null_mut()),
                                IDS_CANCEL_CONFIRM,
                                szMsg.as_mut_ptr(),
                                szMsg.len() as _,
                            );
                            ret = MessageBoxW(
                                hwndDlg,
                                szMsg.as_ptr(),
                                winsrv_sys::w!("ReactOS LiveCD").as_ptr(),
                                MB_ICONWARNING | MB_YESNO | MB_DEFBUTTON2,
                            );
                            if ret == IDOK as INT || ret == IDYES as INT {
                                (*pState).NextPage = DONE;
                                (*pState).Run = REBOOT;
                                EndDialog(hwndDlg, LOWORD(wParam as u32) as INT_PTR);
                            }
                        }
                    }

                    _ => {}
                }
            }
        }

        _ => {}
    }

    FALSE as INT_PTR
}
#[no_mangle]
pub unsafe extern "C" fn ParseUnattend(UnattendInf: LPCWSTR, pUnattend: *mut LIVECD_UNATTEND) {
    let mut Buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    (*pUnattend).bEnabled = 0;

    if GetPrivateProfileStringW(
        winsrv_sys::w!("Unattend").as_ptr(),
        winsrv_sys::w!("Signature").as_ptr(),
        winsrv_sys::w!("").as_ptr(),
        Buffer.as_mut_ptr(),
        Buffer.len() as u32,
        UnattendInf,
    ) == 0
    {
        return;
    }

    if _wcsicmp(
        Buffer.as_ptr(),
        winsrv_sys::w!("$ReactOS$").as_ptr(),
    ) != 0
        && _wcsicmp(
            Buffer.as_ptr(),
            winsrv_sys::w!("$Windows NT$").as_ptr(),
        ) != 0
    {
        return;
    }

    if GetPrivateProfileStringW(
        winsrv_sys::w!("Unattend").as_ptr(),
        winsrv_sys::w!("UnattendSetupEnabled").as_ptr(),
        winsrv_sys::w!("").as_ptr(),
        Buffer.as_mut_ptr(),
        Buffer.len() as u32,
        UnattendInf,
    ) == 0
    {
        return;
    }

    if _wcsicmp(
        Buffer.as_ptr(),
        winsrv_sys::w!("yes").as_ptr(),
    ) != 0
    {
        return;
    }

    (*pUnattend).bEnabled = 1;
    (*pUnattend).LocaleID = 0;

    if GetPrivateProfileStringW(
        winsrv_sys::w!("Unattend").as_ptr(),
        winsrv_sys::w!("LocaleID").as_ptr(),
        winsrv_sys::w!("").as_ptr(),
        Buffer.as_mut_ptr(),
        Buffer.len() as u32,
        UnattendInf,
    ) != 0
        && Buffer[0] != 0
    {
        (*pUnattend).LocaleID = wcstol(Buffer.as_ptr(), core::ptr::null_mut(), 16) as LCID;
    }
}
#[no_mangle]
pub unsafe extern "C" fn RunLiveCD(pState: PSTATE) {
    let mut Unattend: LIVECD_UNATTEND = core::mem::zeroed();
    let mut UnattendInf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    InitLogo(core::ptr::addr_of_mut!((*pState).ImageInfo), core::ptr::null_mut());

    GetWindowsDirectoryW(UnattendInf.as_mut_ptr(), UnattendInf.len() as u32);
    wcscat(UnattendInf.as_mut_ptr(), winsrv_sys::w!("\\unattend.inf").as_ptr());
    ParseUnattend(UnattendInf.as_ptr(), core::ptr::addr_of_mut!(Unattend));
    (*pState).Unattend = core::ptr::addr_of_mut!(Unattend);

    while (*pState).NextPage != DONE {
        match (*pState).NextPage {
            LOCALEPAGE => {
                DialogBoxParamW(
                    hInstance,
                    MAKEINTRESOURCEW(IDD_LOCALEPAGE),
                    core::ptr::null_mut(),
                    Some(LocaleDlgProc),
                    pState as LPARAM,
                );
            }
            STARTPAGE => {
                DialogBoxParamW(
                    hInstance,
                    MAKEINTRESOURCEW(IDD_STARTPAGE),
                    core::ptr::null_mut(),
                    Some(StartDlgProc),
                    pState as LPARAM,
                );
            }
            _ => {}
        }
    }

    DeleteObject((*pState).ImageInfo.hBitmap as *mut core::ffi::c_void);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Userinit Logon Application
//  * FILE:        base/system/userinit/livecd.c
//  * PROGRAMMERS: Eric Kohl
//  */
//
// #include "userinit.h"
//
// HWND hList;
// HWND hLocaleList;
// BOOL bSpain = FALSE;
//
// typedef struct _LIVECD_UNATTEND
// {
//     BOOL bEnabled;
//     LCID LocaleID;
// } LIVECD_UNATTEND;
//
//
// /*
//  * Taken and adapted from dll/cpl/sysdm/general.c
//  */
// static VOID
// InitLogo(PIMGINFO pImgInfo, HWND hwndDlg)
// {
//     BITMAP logoBitmap;
//     BITMAP maskBitmap;
//     BITMAPINFO bmpi;
//     HDC hDC, hDCLogo, hDCMask;
//     HBITMAP hMask = NULL, hLogo = NULL;
//     HBITMAP hAlphaLogo = NULL;
//     COLORREF *pBits;
//     INT line, column;
//
//     hDC = GetDC(hwndDlg);
//     hDCLogo = CreateCompatibleDC(NULL);
//     hDCMask = CreateCompatibleDC(NULL);
//
//     if (hDC == NULL || hDCLogo == NULL || hDCMask == NULL)
//         goto Cleanup;
//
//     ZeroMemory(pImgInfo, sizeof(*pImgInfo));
//     ZeroMemory(&bmpi, sizeof(bmpi));
//
//     hLogo = (HBITMAP)LoadImageW(hInstance, MAKEINTRESOURCEW(IDB_ROSLOGO), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR);
//     hMask = (HBITMAP)LoadImageW(hInstance, MAKEINTRESOURCEW(IDB_ROSMASK), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR);
//
//     if (hLogo == NULL || hMask == NULL)
//         goto Cleanup;
//
//     GetObjectW(hLogo, sizeof(logoBitmap), &logoBitmap);
//     GetObjectW(hMask, sizeof(maskBitmap), &maskBitmap);
//
//     if (logoBitmap.bmHeight != maskBitmap.bmHeight || logoBitmap.bmWidth != maskBitmap.bmWidth)
//         goto Cleanup;
//
//     bmpi.bmiHeader.biSize = sizeof(BITMAPINFO);
//     bmpi.bmiHeader.biWidth = logoBitmap.bmWidth;
//     bmpi.bmiHeader.biHeight = logoBitmap.bmHeight;
//     bmpi.bmiHeader.biPlanes = 1;
//     bmpi.bmiHeader.biBitCount = 32;
//     bmpi.bmiHeader.biCompression = BI_RGB;
//     bmpi.bmiHeader.biSizeImage = 4 * logoBitmap.bmWidth * logoBitmap.bmHeight;
//
//     /* Create a premultiplied bitmap */
//     hAlphaLogo = CreateDIBSection(hDC, &bmpi, DIB_RGB_COLORS, (PVOID*)&pBits, 0, 0);
//     if (!hAlphaLogo)
//         goto Cleanup;
//
//     SelectObject(hDCLogo, hLogo);
//     SelectObject(hDCMask, hMask);
//
//     for (line = logoBitmap.bmHeight - 1; line >= 0; line--)
//     {
//         for (column = 0; column < logoBitmap.bmWidth; column++)
//         {
//             COLORREF alpha = GetPixel(hDCMask, column, line) & 0xFF;
//             COLORREF Color = GetPixel(hDCLogo, column, line);
//             DWORD r, g, b;
//
//             r = GetRValue(Color) * alpha / 255;
//             g = GetGValue(Color) * alpha / 255;
//             b = GetBValue(Color) * alpha / 255;
//
//             *pBits++ = b | (g << 8) | (r << 16) | (alpha << 24);
//         }
//     }
//
//     pImgInfo->hBitmap = hAlphaLogo;
//     pImgInfo->cxSource = logoBitmap.bmWidth;
//     pImgInfo->cySource = logoBitmap.bmHeight;
//     pImgInfo->iBits = logoBitmap.bmBitsPixel;
//     pImgInfo->iPlanes = logoBitmap.bmPlanes;
//
// Cleanup:
//     if (hMask != NULL) DeleteObject(hMask);
//     if (hLogo != NULL) DeleteObject(hLogo);
//     if (hDCMask != NULL) DeleteDC(hDCMask);
//     if (hDCLogo != NULL) DeleteDC(hDCLogo);
//     if (hDC != NULL) ReleaseDC(hwndDlg, hDC);
// }
//
//
// BOOL
// IsLiveCD(VOID)
// {
//     HKEY ControlKey = NULL;
//     LPWSTR SystemStartOptions = NULL;
//     LPWSTR CurrentOption, NextOption; /* Pointers into SystemStartOptions */
//     LONG rc;
//     BOOL ret = FALSE;
//
//     rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                        REGSTR_PATH_CURRENT_CONTROL_SET(),
//                        0,
//                        KEY_QUERY_VALUE,
//                        &ControlKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("RegOpenKeyExW() failed with error %lu\n", rc);
//         goto cleanup;
//     }
//
//     rc = ReadRegSzKey(ControlKey, L"SystemStartOptions", &SystemStartOptions);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("ReadRegSzKey() failed with error %lu\n", rc);
//         goto cleanup;
//     }
//
//     /* Check for CONSOLE switch in SystemStartOptions */
//     CurrentOption = SystemStartOptions;
//     while (CurrentOption)
//     {
//         NextOption = wcschr(CurrentOption, L' ');
//         if (NextOption)
//             *NextOption = L'\0';
//         if (_wcsicmp(CurrentOption, L"MININT") == 0)
//         {
//             TRACE("Found 'MININT' boot option\n");
//             ret = TRUE;
//             goto cleanup;
//         }
//         CurrentOption = NextOption ? NextOption + 1 : NULL;
//     }
//
// cleanup:
//     if (ControlKey != NULL)
//         RegCloseKey(ControlKey);
//     HeapFree(GetProcessHeap(), 0, SystemStartOptions);
//
//     TRACE("IsLiveCD() returning %u\n", ret);
//     return ret;
// }
//
//
// static BOOL __stdcall
// LocalesEnumProc(LPTSTR lpLocale)
// {
//     LCID lcid;
//     WCHAR lang[255];
//     INT index;
//     BOOL bNoShow = FALSE;
//
//     lcid = wcstoul(lpLocale, NULL, 16);
//
//     /* Display only languages with installed support */
//     if (!IsValidLocale(lcid, LCID_INSTALLED))
//         return TRUE;
//
//     // See http://archives.miloush.net/michkap/archive/2006/09/23/768178.html for why we handle spain differently
//     if (lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
//         lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT))
//     {
//         if (bSpain == FALSE)
//         {
//             LoadStringW(hInstance, IDS_SPAIN, lang, ARRAYSIZE(lang));
//             bSpain = TRUE;
//         }
//         else
//         {
//             bNoShow = TRUE;
//         }
//     }
//     else
//     {
//         GetLocaleInfoW(lcid, LOCALE_SLANGUAGE, lang, ARRAYSIZE(lang));
//     }
//
//     if (bNoShow == FALSE)
//     {
//         index = SendMessageW(hList,
//                              CB_ADDSTRING,
//                              0,
//                              (LPARAM)lang);
//
//         SendMessageW(hList,
//                      CB_SETITEMDATA,
//                      index,
//                      (LPARAM)lcid);
//     }
//
//     return TRUE;
// }
//
//
// static VOID
// CreateLanguagesList(HWND hwnd, PSTATE pState)
// {
//     WCHAR langSel[255];
//     LCID Locale = 0;
//
//     hList = hwnd;
//     bSpain = FALSE;
//     EnumSystemLocalesW(Some(LocalesEnumProc), LCID_SUPPORTED);
//
//     if (pState->Unattend->bEnabled)
//         Locale = pState->Unattend->LocaleID;
//
//     if (!Locale)
//     {
//         /* Select current locale */
//         /* or should it be System and not user? */
//         Locale = GetUserDefaultLCID();
//     }
//     GetLocaleInfoW(Locale, LOCALE_SLANGUAGE, langSel, ARRAYSIZE(langSel));
//
//     SendMessageW(hList,
//                  CB_SELECTSTRING,
//                  -1,
//                  (LPARAM)langSel);
// }
//
//
// static
// BOOL
// GetLayoutName(
//     LPCWSTR szLCID,
//     LPWSTR szName)
// {
//     HKEY hKey;
//     DWORD dwBufLen;
//     WCHAR szBuf[MAX_PATH], szDispName[MAX_PATH], szIndex[MAX_PATH], szPath[MAX_PATH];
//     HANDLE hLib;
//     UINT i, j, k;
//
//     wsprintfW(szBuf, L"SYSTEM\\CurrentControlSet\\Control\\Keyboard Layouts\\%s", szLCID);
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuf, 0, KEY_QUERY_VALUE, &hKey) == ERROR_SUCCESS)
//     {
//         dwBufLen = sizeof(szDispName);
//
//         if (RegQueryValueExW(hKey, L"Layout Display Name", NULL, NULL, (LPBYTE)szDispName, &dwBufLen) == ERROR_SUCCESS)
//         {
//             if (szDispName[0] == '@')
//             {
//                 for (i = 0; i < wcslen(szDispName); i++)
//                 {
//                     if ((szDispName[i] == ',') && (szDispName[i + 1] == '-'))
//                     {
//                         for (j = i + 2, k = 0; j < wcslen(szDispName)+1; j++, k++)
//                         {
//                             szIndex[k] = szDispName[j];
//                         }
//                         szDispName[i - 1] = '\0';
//                         break;
//                     }
//                     else
//                         szDispName[i] = szDispName[i + 1];
//                 }
//
//                 if (ExpandEnvironmentStringsW(szDispName, szPath, ARRAYSIZE(szPath)))
//                 {
//                     hLib = LoadLibraryW(szPath);
//                     if (hLib)
//                     {
//                         if (LoadStringW(hLib, _wtoi(szIndex), szPath, ARRAYSIZE(szPath)) != 0)
//                         {
//                             wcscpy(szName, szPath);
//                             RegCloseKey(hKey);
//                             return TRUE;
//                         }
//                         FreeLibrary(hLib);
//                     }
//                 }
//             }
//         }
//
//         dwBufLen = sizeof(szBuf);
//
//         if (RegQueryValueExW(hKey, L"Layout Text", NULL, NULL, (LPBYTE)szName, &dwBufLen) == ERROR_SUCCESS)
//         {
//             RegCloseKey(hKey);
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
//
// static
// VOID
// SetKeyboardLayout(
//     HWND hwnd)
// {
//     INT iCurSel;
//     ULONG ulLayoutId;
//     HKL hKl;
//     WCHAR szLayoutId[9];
//
//     iCurSel = SendMessageW(hwnd, CB_GETCURSEL, 0, 0);
//     if (iCurSel == CB_ERR)
//         return;
//
//     ulLayoutId = (ULONG)SendMessageW(hwnd, CB_GETITEMDATA, iCurSel, 0);
//     if (ulLayoutId == (ULONG)CB_ERR)
//         return;
//
//     swprintf(szLayoutId, L"%08lx", ulLayoutId);
//
//     hKl = LoadKeyboardLayoutW(szLayoutId, KLF_ACTIVATE | KLF_REPLACELANG | KLF_SETFORPROCESS);
//     SystemParametersInfoW(SPI_SETDEFAULTINPUTLANG, 0, &hKl, SPIF_SENDCHANGE);
// }
//
//
// static
// VOID
// SelectKeyboardForLanguage(
//     HWND hwnd,
//     LCID lcid)
// {
//     INT i, nCount;
//     LCID LayoutId;
//
//     TRACE("LCID: %08lx\n", lcid);
//     TRACE("LangID: %04lx\n", LANGIDFROMLCID(lcid));
//
//     nCount = SendMessageW(hwnd, CB_GETCOUNT, 0, 0);
//
//     for (i = 0; i < nCount; i++)
//     {
//         LayoutId = (LCID)SendMessageW(hwnd, CB_GETITEMDATA, i, 0);
//         TRACE("Layout: %08lx\n", LayoutId);
//
//         if (LANGIDFROMLCID(LayoutId) == LANGIDFROMLCID(lcid))
//         {
//             TRACE("Found 1: %08lx --> %08lx\n", LayoutId, lcid);
//             SendMessageW(hwnd, CB_SETCURSEL, i, 0);
//             return;
//         }
//     }
//
//     for (i = 0; i < nCount; i++)
//     {
//         LayoutId = (LCID)SendMessageW(hwnd, CB_GETITEMDATA, i, 0);
//         TRACE("Layout: %08lx\n", LayoutId);
//
//         if (PRIMARYLANGID(LayoutId) == PRIMARYLANGID(lcid))
//         {
//             TRACE("Found 2: %08lx --> %08lx\n", LayoutId, lcid);
//             SendMessageW(hwnd, CB_SETCURSEL, i, 0);
//             return;
//         }
//     }
//
//     TRACE("No match found!\n");
// }
//
//
// static
// VOID
// CreateKeyboardLayoutList(
//     HWND hItemsList)
// {
//     HKEY hKey;
//     WCHAR szLayoutId[9], szCurrentLayoutId[9];
//     WCHAR KeyName[MAX_PATH];
//     DWORD dwIndex = 0;
//     DWORD dwSize;
//     INT iIndex;
//     LONG lError;
//     ULONG ulLayoutId;
//
//     if (!GetKeyboardLayoutNameW(szCurrentLayoutId))
//         wcscpy(szCurrentLayoutId, L"00000409");
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"System\\CurrentControlSet\\Control\\Keyboard Layouts",
//                            0,
//                            KEY_ENUMERATE_SUB_KEYS,
//                            &hKey);
//     if (lError != ERROR_SUCCESS)
//         return;
//
//     while (TRUE)
//     {
//         dwSize = ARRAYSIZE(szLayoutId);
//
//         lError = RegEnumKeyExW(hKey,
//                                dwIndex,
//                                szLayoutId,
//                                &dwSize,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL);
//         if (lError != ERROR_SUCCESS)
//             break;
//
//         GetLayoutName(szLayoutId, KeyName);
//
//         iIndex = (INT)SendMessageW(hItemsList, CB_ADDSTRING, 0, (LPARAM)KeyName);
//
//         ulLayoutId = wcstoul(szLayoutId, NULL, 16);
//         SendMessageW(hItemsList, CB_SETITEMDATA, iIndex, (LPARAM)ulLayoutId);
//
//         if (wcscmp(szLayoutId, szCurrentLayoutId) == 0)
//         {
//             SendMessageW(hItemsList, CB_SETCURSEL, (WPARAM)iIndex, (LPARAM)0);
//         }
//
//         dwIndex++;
//     }
//
//     RegCloseKey(hKey);
// }
//
//
// static
// VOID
// InitializeDefaultUserLocale(
//     PLCID pNewLcid)
// {
//     WCHAR szBuffer[80];
//     PWSTR ptr;
//     HKEY hLocaleKey;
//     DWORD ret;
//     DWORD dwSize;
//     LCID lcid;
//     INT i;
//
//     struct {LCTYPE LCType; PWSTR pValue;} LocaleData[] = {
//         /* Number */
//         {LOCALE_SDECIMAL, L"sDecimal"},
//         {LOCALE_STHOUSAND, L"sThousand"},
//         {LOCALE_SNEGATIVESIGN, L"sNegativeSign"},
//         {LOCALE_SPOSITIVESIGN, L"sPositiveSign"},
//         {LOCALE_SGROUPING, L"sGrouping"},
//         {LOCALE_SLIST, L"sList"},
//         {LOCALE_SNATIVEDIGITS, L"sNativeDigits"},
//         {LOCALE_INEGNUMBER, L"iNegNumber"},
//         {LOCALE_IDIGITS, L"iDigits"},
//         {LOCALE_ILZERO, L"iLZero"},
//         {LOCALE_IMEASURE, L"iMeasure"},
//         {LOCALE_IDIGITSUBSTITUTION, L"NumShape"},
//
//         /* Currency */
//         {LOCALE_SCURRENCY, L"sCurrency"},
//         {LOCALE_SMONDECIMALSEP, L"sMonDecimalSep"},
//         {LOCALE_SMONTHOUSANDSEP, L"sMonThousandSep"},
//         {LOCALE_SMONGROUPING, L"sMonGrouping"},
//         {LOCALE_ICURRENCY, L"iCurrency"},
//         {LOCALE_INEGCURR, L"iNegCurr"},
//         {LOCALE_ICURRDIGITS, L"iCurrDigits"},
//
//         /* Time */
//         {LOCALE_STIMEFORMAT, L"sTimeFormat"},
//         {LOCALE_STIME, L"sTime"},
//         {LOCALE_S1159, L"s1159"},
//         {LOCALE_S2359, L"s2359"},
//         {LOCALE_ITIME, L"iTime"},
//         {LOCALE_ITIMEMARKPOSN, L"iTimePrefix"},
//         {LOCALE_ITLZERO, L"iTLZero"},
//
//         /* Date */
//         {LOCALE_SLONGDATE, L"sLongDate"},
//         {LOCALE_SSHORTDATE, L"sShortDate"},
//         {LOCALE_SDATE, L"sDate"},
//         {LOCALE_IFIRSTDAYOFWEEK, L"iFirstDayOfWeek"},
//         {LOCALE_IFIRSTWEEKOFYEAR, L"iFirstWeekOfYear"},
//         {LOCALE_IDATE, L"iDate"},
//         {LOCALE_ICALENDARTYPE, L"iCalendarType"},
//
//         /* Misc */
//         {LOCALE_SCOUNTRY, L"sCountry"},
//         {LOCALE_SABBREVLANGNAME, L"sLanguage"},
//         {LOCALE_ICOUNTRY, L"iCountry"},
//         {0, NULL}};
//
//     ret = RegOpenKeyExW(HKEY_USERS,
//                         L".DEFAULT\\Control Panel\\International",
//                         0,
//                         KEY_READ | KEY_WRITE,
//                         &hLocaleKey);
//     if (ret != ERROR_SUCCESS)
//     {
//         return;
//     }
//
//     if (pNewLcid == NULL)
//     {
//         dwSize = 9 * sizeof(WCHAR);
//         ret = RegQueryValueExW(hLocaleKey,
//                                L"Locale",
//                                NULL,
//                                NULL,
//                                (PBYTE)szBuffer,
//                                &dwSize);
//         if (ret != ERROR_SUCCESS)
//             goto done;
//
//         lcid = (LCID)wcstoul(szBuffer, &ptr, 16);
//         if (lcid == 0)
//             goto done;
//     }
//     else
//     {
//         lcid = *pNewLcid;
//
//         swprintf(szBuffer, L"%08lx", lcid);
//         RegSetValueExW(hLocaleKey,
//                        L"Locale",
//                        0,
//                        REG_SZ,
//                        (PBYTE)szBuffer,
//                        (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//     }
//
//     i = 0;
//     while (LocaleData[i].pValue != NULL)
//     {
//         if (GetLocaleInfoW(lcid,
//                            LocaleData[i].LCType | LOCALE_NOUSEROVERRIDE,
//                            szBuffer,
//                            ARRAYSIZE(szBuffer)))
//         {
//             RegSetValueExW(hLocaleKey,
//                            LocaleData[i].pValue,
//                            0,
//                            REG_SZ,
//                            (PBYTE)szBuffer,
//                            (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//         }
//
//         i++;
//     }
//
// done:
//     RegCloseKey(hLocaleKey);
// }
//
//
// VOID
// CenterWindow(HWND hWnd)
// {
//     HWND hWndParent;
//     RECT rcParent;
//     RECT rcWindow;
//
//     hWndParent = GetParent(hWnd);
//     if (hWndParent == NULL)
//         hWndParent = GetDesktopWindow();
//
//     GetWindowRect(hWndParent, &rcParent);
//     GetWindowRect(hWnd, &rcWindow);
//
//     SetWindowPos(hWnd,
//                  HWND_TOP,
//                  ((rcParent.right - rcParent.left) - (rcWindow.right - rcWindow.left)) / 2,
//                  ((rcParent.bottom - rcParent.top) - (rcWindow.bottom - rcWindow.top)) / 2,
//                  0,
//                  0,
//                  SWP_NOSIZE);
// }
//
//
// static
// VOID
// OnDrawItem(
//      LPDRAWITEMSTRUCT lpDrawItem,
//      PSTATE pState,
//      UINT uCtlID)
// {
//     HDC hdcMem;
//     LONG left;
//
//     if (lpDrawItem->CtlID == uCtlID)
//     {
//         /* Position image in centre of dialog */
//         left = (lpDrawItem->rcItem.right - pState->ImageInfo.cxSource) / 2;
//
//         hdcMem = CreateCompatibleDC(lpDrawItem->hDC);
//         if (hdcMem != NULL)
//         {
//             static BLENDFUNCTION BlendFunc = {AC_SRC_OVER, 0, 255, AC_SRC_ALPHA};
//
//             SelectObject(hdcMem, pState->ImageInfo.hBitmap);
//             GdiAlphaBlend(lpDrawItem->hDC,
//                           left,
//                           lpDrawItem->rcItem.top,
//                           pState->ImageInfo.cxSource,
//                           pState->ImageInfo.cySource,
//                           hdcMem,
//                           0, 0,
//                           pState->ImageInfo.cxSource,
//                           pState->ImageInfo.cySource,
//                           BlendFunc);
//             DeleteDC(hdcMem);
//         }
//     }
// }
//
//
// static
// INT_PTR
// __stdcall
// LocaleDlgProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     PSTATE pState;
//
//     /* Retrieve pointer to the state */
//     pState = (PSTATE)GetWindowLongPtrW(hwndDlg, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             /* Save pointer to the state */
//             pState = (PSTATE)lParam;
//             SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, (DWORD_PTR)pState);
//
//             /* Center the dialog window */
//             CenterWindow(hwndDlg);
//
//             /* Fill the language and keyboard layout lists */
//             CreateLanguagesList(GetDlgItem(hwndDlg, IDC_LANGUAGELIST as INT), pState);
//             CreateKeyboardLayoutList(GetDlgItem(hwndDlg, IDC_LAYOUTLIST as INT));
//             if (pState->Unattend->bEnabled)
//             {
//                 /* Advance to the next page */
//                 PostMessageW(hwndDlg, WM_COMMAND, MAKEWPARAM(IDOK, BN_CLICKED), 0L);
//             }
//             return FALSE;
//
//         case WM_DRAWITEM:
//             OnDrawItem((LPDRAWITEMSTRUCT)lParam,
//                        pState,
//                        IDC_LOCALELOGO);
//             return TRUE;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_LANGUAGELIST:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         LCID NewLcid;
//                         INT iCurSel;
//
//                         iCurSel = SendDlgItemMessageW(hwndDlg,
//                                                      IDC_LANGUAGELIST,
//                                                      CB_GETCURSEL,
//                                                      0,
//                                                      0);
//                         if (iCurSel == CB_ERR)
//                             break;
//
//                         NewLcid = SendDlgItemMessageW(hwndDlg,
//                                                       IDC_LANGUAGELIST,
//                                                       CB_GETITEMDATA,
//                                                       iCurSel,
//                                                       0);
//                         if (NewLcid == (LCID)CB_ERR)
//                             break;
//
//                         TRACE("LCID: 0x%08lx\n", NewLcid);
//                         SelectKeyboardForLanguage(GetDlgItem(hwndDlg, IDC_LAYOUTLIST as INT),
//                                                   NewLcid);
//                     }
//                     break;
//
//                 case IDOK:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         LCID NewLcid;
//                         INT iCurSel;
//
//                         iCurSel = SendDlgItemMessageW(hwndDlg,
//                                                      IDC_LANGUAGELIST,
//                                                      CB_GETCURSEL,
//                                                      0,
//                                                      0);
//                         if (iCurSel == CB_ERR)
//                             break;
//
//                         NewLcid = SendDlgItemMessageW(hwndDlg,
//                                                       IDC_LANGUAGELIST,
//                                                       CB_GETITEMDATA,
//                                                       iCurSel,
//                                                       0);
//                         if (NewLcid == (LCID)CB_ERR)
//                             break;
//
//                         /* Set the locale for the current thread */
//                         NtSetDefaultLocale(TRUE, NewLcid);
//
//                         /* Store the locale settings in the registry */
//                         InitializeDefaultUserLocale(&NewLcid);
//
//                         /* Set UI language for this thread */
//                         SetThreadLocale(NewLcid);
//
//                         SetKeyboardLayout(GetDlgItem(hwndDlg, IDC_LAYOUTLIST as INT));
//
//                         pState->NextPage = STARTPAGE;
//                         EndDialog(hwndDlg, LOWORD(wParam));
//                     }
//                     break;
//
//                 case IDCANCEL:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         static WCHAR szMsg[RC_STRING_MAX_SIZE];
//                         INT ret;
//                         LoadStringW(GetModuleHandleW(NULL), IDS_CANCEL_CONFIRM, szMsg, ARRAYSIZE(szMsg));
//                         ret = MessageBoxW(hwndDlg, szMsg, L"ReactOS LiveCD", MB_ICONWARNING | MB_YESNO | MB_DEFBUTTON2);
//                         if (ret == IDOK || ret == IDYES)
//                         {
//                             pState->NextPage = DONE;
//                             pState->Run = REBOOT;
//                             EndDialog(hwndDlg, LOWORD(wParam));
//                         }
//                     }
//                     break;
//
//                 default:
//                     break;
//             }
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static
// INT_PTR
// __stdcall
// StartDlgProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     PSTATE pState;
//
//     /* Retrieve pointer to the state */
//     pState = (PSTATE)GetWindowLongPtrW(hwndDlg, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             WCHAR Installer[MAX_PATH];
//
//             /* Save pointer to the state */
//             pState = (PSTATE)lParam;
//             SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, (DWORD_PTR)pState);
//
//             /* Center the dialog window */
//             CenterWindow(hwndDlg);
//
//             /* Check whether we can find the ReactOS installer. If not,
//              * disable the "Install" button and directly start the LiveCD. */
//             *Installer = UNICODE_NULL;
//             if (!ExpandInstallerPath(L"reactos.exe", Installer, ARRAYSIZE(Installer)))
//                 EnableWindow(GetDlgItem(hwndDlg, IDC_INSTALL as INT), FALSE);
//
//             if (pState->Unattend->bEnabled || (*Installer == UNICODE_NULL))
//             {
//                 /* Click on the 'Run' button */
//                 PostMessageW(hwndDlg, WM_COMMAND, MAKEWPARAM(IDC_RUN, BN_CLICKED), 0L);
//             }
//             return FALSE;
//         }
//
//         case WM_DRAWITEM:
//             OnDrawItem((LPDRAWITEMSTRUCT)lParam,
//                        pState,
//                        IDC_STARTLOGO);
//             return TRUE;
//
//         case WM_COMMAND:
//             if (HIWORD(wParam) == BN_CLICKED)
//             {
//                 switch (LOWORD(wParam))
//                 {
//                     case IDC_RUN:
//                         pState->NextPage = DONE;
//                         pState->Run = SHELL;
//                         EndDialog(hwndDlg, LOWORD(wParam));
//                         break;
//
//                     case IDC_INSTALL:
//                         pState->NextPage = DONE;
//                         pState->Run = INSTALLER;
//                         EndDialog(hwndDlg, LOWORD(wParam));
//                         break;
//
//                     case IDOK:
//                         pState->NextPage = LOCALEPAGE;
//                         EndDialog(hwndDlg, LOWORD(wParam));
//                         break;
//
//                     case IDCANCEL:
//                         if (HIWORD(wParam) == BN_CLICKED)
//                         {
//                             static WCHAR szMsg[RC_STRING_MAX_SIZE];
//                             INT ret;
//                             LoadStringW(GetModuleHandleW(NULL), IDS_CANCEL_CONFIRM, szMsg, ARRAYSIZE(szMsg));
//                             ret = MessageBoxW(hwndDlg, szMsg, L"ReactOS LiveCD", MB_ICONWARNING | MB_YESNO | MB_DEFBUTTON2);
//                             if (ret == IDOK || ret == IDYES)
//                             {
//                                 pState->NextPage = DONE;
//                                 pState->Run = REBOOT;
//                                 EndDialog(hwndDlg, LOWORD(wParam));
//                             }
//                         }
//                         break;
//
//                     default:
//                         break;
//                 }
//             }
//             break;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// VOID ParseUnattend(LPCWSTR UnattendInf, LIVECD_UNATTEND* pUnattend)
// {
//     WCHAR Buffer[MAX_PATH];
//
//     pUnattend->bEnabled = FALSE;
//
//     if (!GetPrivateProfileStringW(L"Unattend", L"Signature", L"", Buffer, _countof(Buffer), UnattendInf))
//     {
//         ERR("Unable to parse Signature\n");
//         return;
//     }
//
//     if (_wcsicmp(Buffer, L"$ReactOS$") && _wcsicmp(Buffer, L"$Windows NT$"))
//     {
//         TRACE("Unknown signature: %S\n", Buffer);
//         return;
//     }
//
//     if (!GetPrivateProfileStringW(L"Unattend", L"UnattendSetupEnabled", L"", Buffer, _countof(Buffer), UnattendInf))
//     {
//         ERR("Unable to parse UnattendSetupEnabled\n");
//         return;
//     }
//
//     if (_wcsicmp(Buffer, L"yes"))
//     {
//         TRACE("Unattended setup is not enabled\n", Buffer);
//         return;
//     }
//
//     pUnattend->bEnabled = TRUE;
//     pUnattend->LocaleID = 0;
//
//     if (GetPrivateProfileStringW(L"Unattend", L"LocaleID", L"", Buffer, _countof(Buffer), UnattendInf) && Buffer[0])
//     {
//         pUnattend->LocaleID = wcstol(Buffer, NULL, 16);
//     }
// }
//
// VOID
// RunLiveCD(
//     PSTATE pState)
// {
//     LIVECD_UNATTEND Unattend = {0};
//     WCHAR UnattendInf[MAX_PATH];
//
//     InitLogo(&pState->ImageInfo, NULL);
//
//     GetWindowsDirectoryW(UnattendInf, _countof(UnattendInf));
//     wcscat(UnattendInf, L"\\unattend.inf");
//     ParseUnattend(UnattendInf, &Unattend);
//     pState->Unattend = &Unattend;
//
//     while (pState->NextPage != DONE)
//     {
//         switch (pState->NextPage)
//         {
//             case LOCALEPAGE:
//                 DialogBoxParamW(hInstance,
//                                 MAKEINTRESOURCEW(IDD_LOCALEPAGE),
//                                 NULL,
//                                 Some(LocaleDlgProc),
//                                 (LPARAM)pState);
//                 break;
//
//             case STARTPAGE:
//                 DialogBoxParamW(hInstance,
//                                 MAKEINTRESOURCEW(IDD_STARTPAGE),
//                                 NULL,
//                                 Some(StartDlgProc),
//                                 (LPARAM)pState);
//                 break;
//
//             default:
//                 break;
//         }
//     }
//
//     DeleteObject(pState->ImageInfo.hBitmap);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

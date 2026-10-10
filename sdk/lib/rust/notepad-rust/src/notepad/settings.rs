use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_B8C79E4A376C: [u16; 27] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x004E,
    0x006F, 0x0074, 0x0065, 0x0070, 0x0061, 0x0064, 0x0000,
];

static mut s_szRegistryKey: LPCTSTR = unsafe {
    RUSTCONV_C2R_WSTR_B8C79E4A376C.as_ptr() as LPCTSTR
};

pub static s_szRegistryKey_BUF: [TCHAR; 27] = [
    0x53, 0x6F, 0x66, 0x74, 0x77, 0x61, 0x72, 0x65, // Software
    0x5C, // \
    0x4D, 0x69, 0x63, 0x72, 0x6F, 0x73, 0x6F, 0x66, 0x74, // Microsoft
    0x5C, // \
    0x4E, 0x6F, 0x74, 0x65, 0x70, 0x61, 0x64, // Notepad
    0x00, // NUL
];

unsafe extern "C" fn HeightFromPointSize(dwPointSize: DWORD) -> LONG {
    let mut lHeight: LONG = 0;
    let mut hDC: HDC = core::ptr::null_mut();
    hDC = GetDC(core::ptr::null_mut());
    lHeight = -(MulDiv(dwPointSize as i32, GetDeviceCaps(hDC, (LOGPIXELSY as i32)), 720) as LONG);
    ReleaseDC(core::ptr::null_mut(), hDC);

    return lHeight;
}

unsafe extern "C" fn PointSizeFromHeight(lHeight: LONG) -> DWORD {
    let mut dwPointSize: DWORD;
    let mut hDC: HDC;

    hDC = GetDC(core::ptr::null_mut());
    dwPointSize = (-(MulDiv(lHeight, 720, GetDeviceCaps(hDC, (LOGPIXELSY as i32))))) as DWORD;
    ReleaseDC(core::ptr::null_mut(), hDC);

    /* round to nearest multiple of 10 */
    dwPointSize = dwPointSize.wrapping_add(5 as DWORD);
    dwPointSize = dwPointSize.wrapping_sub(dwPointSize % (10 as DWORD));

    return dwPointSize;
}

unsafe extern "C" fn QueryGeneric(
    hKey: HKEY,
    pszValueNameT: LPCTSTR,
    dwExpectedType: DWORD,
    pvResult: LPVOID,
    dwResultSize: DWORD,
) -> BOOL {
    let mut dwType: DWORD = 0;
    let mut cbData: DWORD;
    // C: `_alloca(dwResultSize)`. No stack allocator in no_std, so the scratch buffer is
    // process heap, freed on every exit; a failed allocation fails the query.
    let pTemp: LPVOID = heap_alloc(dwResultSize as usize);
    if pTemp.is_null() {
        return FALSE as BOOL;
    }

    ZeroMemory!(pTemp, dwResultSize as usize);

    cbData = dwResultSize;
    let ok = RegQueryValueEx(
        hKey,
        pszValueNameT,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(dwType),
        pTemp as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    ) as u32
        == ERROR_SUCCESS
        && dwType == dwExpectedType;
    if ok {
        memcpy(pvResult, pTemp as *const core::ffi::c_void, cbData as usize);
    }
    heap_free(pTemp);
    ok as BOOL
}

unsafe extern "C" fn QueryDword(hKey: HKEY, pszValueName: LPCTSTR, pdwResult: *mut DWORD) -> BOOL {
    return QueryGeneric(hKey, pszValueName, REG_DWORD as DWORD, pdwResult as LPVOID, core::mem::size_of::<DWORD>() as DWORD);
}

unsafe extern "C" fn QueryByte(hKey: HKEY, pszValueName: LPCTSTR, pbResult: *mut BYTE) -> BOOL {
    let mut dwResult: DWORD = 0;
    if QueryGeneric(hKey, pszValueName, REG_DWORD as DWORD, core::ptr::addr_of_mut!(dwResult) as LPVOID, core::mem::size_of::<DWORD>() as DWORD) == 0 {
        return (FALSE) as BOOL;
    }
    if dwResult >= 0x100 {
        return (FALSE) as BOOL;
    }
    *pbResult = dwResult as BYTE;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn QueryBool(hKey: HKEY, pszValueName: LPCTSTR, pbResult: *mut BOOL) -> BOOL {
    let mut dwResult: DWORD = 0;
    if QueryDword(hKey, pszValueName, core::ptr::addr_of_mut!(dwResult)) == 0 {
        return (FALSE) as BOOL;
    }
    *pbResult = if dwResult != 0 { (TRUE) as i32 } else { (FALSE) as i32 };
    return (TRUE) as BOOL;
}

unsafe extern "C" fn QueryString(
    mut hKey: HKEY,
    mut pszValueName: LPCTSTR,
    mut pszResult: LPTSTR,
    mut dwResultLength: DWORD,
) -> BOOL {
    if dwResultLength == 0 {
        return (FALSE) as BOOL;
    }
    if QueryGeneric(
        hKey,
        pszValueName,
        REG_SZ as DWORD,
        pszResult as LPVOID,
        dwResultLength.wrapping_mul(core::mem::size_of::<TCHAR>() as DWORD),
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    *pszResult.offset(dwResultLength.wrapping_sub(1) as isize) = 0;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn NOTEPAD_LoadSettingsFromRegistry(pWP: PWINDOWPLACEMENT) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut dwPointSize: DWORD;
    let mut x: DWORD = (CW_USEDEFAULT) as u32;
    let mut y: DWORD = (CW_USEDEFAULT) as u32;
    let mut cx: DWORD = 0;
    let mut cy: DWORD = 0;

    /* Set the default values */
    Globals.bShowStatusBar = (TRUE) as i32;
    Globals.bWrapLongLines = (FALSE) as i32;
    SetRect(
        &raw mut Globals.lMargins,
        750,
        1000,
        750,
        1000,
    );
    core::ptr::write_bytes(
        &raw mut Globals.lfFont as *mut u8,
        0,
        core::mem::size_of::<LOGFONT>(),
    );
    Globals.lfFont.lfCharSet = (DEFAULT_CHARSET as u8);
    dwPointSize = 100;
    Globals.lfFont.lfWeight = (FW_NORMAL as i32);
    Globals.lfFont.lfPitchAndFamily = (FIXED_PITCH | FF_MODERN) as u8;

    /* FIXME: Globals.fSaveWindowPositions = FALSE; */
    /* FIXME: Globals.fMLE_is_broken = FALSE; */

    /* Open the target registry key */
    if (RegOpenKey(HKEY_CURRENT_USER, s_szRegistryKey, &raw mut hKey)) as u32 != ERROR_SUCCESS {
        hKey = core::ptr::null_mut();
    }

    /* Load the values from registry */
    if !hKey.is_null() {
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'r' as u16, 'S' as u16, 'e' as u16, 't' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfCharSet,
        );
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'C' as u16, 'l' as u16, 'i' as u16, 'p' as u16, 'P' as u16, 'r' as u16, 'e' as u16, 'c' as u16, 'i' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfClipPrecision,
        );
        QueryDword(
            hKey,
            ['l' as u16, 'f' as u16, 'E' as u16, 's' as u16, 'c' as u16, 'a' as u16, 'p' as u16, 'e' as u16, 'm' as u16, 'e' as u16, 'n' as u16, 't' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfEscapement as *mut DWORD,
        );
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'I' as u16, 't' as u16, 'a' as u16, 'l' as u16, 'i' as u16, 'c' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfItalic,
        );
        QueryDword(
            hKey,
            ['l' as u16, 'f' as u16, 'O' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfOrientation as *mut DWORD,
        );
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'O' as u16, 'u' as u16, 't' as u16, 'P' as u16, 'r' as u16, 'e' as u16, 'c' as u16, 'i' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfOutPrecision,
        );
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'P' as u16, 'i' as u16, 't' as u16, 'c' as u16, 'h' as u16, 'A' as u16, 'n' as u16, 'd' as u16, 'F' as u16, 'a' as u16, 'm' as u16, 'i' as u16, 'l' as u16, 'y' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfPitchAndFamily,
        );
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'Q' as u16, 'u' as u16, 'a' as u16, 'l' as u16, 'i' as u16, 't' as u16, 'y' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfQuality,
        );
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'k' as u16, 'e' as u16, 'O' as u16, 'u' as u16, 't' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfStrikeOut,
        );
        QueryByte(
            hKey,
            ['l' as u16, 'f' as u16, 'U' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'r' as u16, 'l' as u16, 'i' as u16, 'n' as u16, 'e' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfUnderline,
        );
        QueryDword(
            hKey,
            ['l' as u16, 'f' as u16, 'W' as u16, 'e' as u16, 'i' as u16, 'g' as u16, 'h' as u16, 't' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfWeight as *mut DWORD,
        );
        QueryDword(
            hKey,
            ['i' as u16, 'P' as u16, 'o' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'S' as u16, 'i' as u16, 'z' as u16, 'e' as u16, 0].as_ptr(),
            &raw mut dwPointSize,
        );

        QueryBool(
            hKey,
            ['f' as u16, 'W' as u16, 'r' as u16, 'a' as u16, 'p' as u16, 0].as_ptr(),
            &raw mut Globals.bWrapLongLines,
        );
        QueryBool(
            hKey,
            ['f' as u16, 'S' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, 'B' as u16, 'a' as u16, 'r' as u16, 0].as_ptr(),
            &raw mut Globals.bShowStatusBar,
        );

        QueryDword(
            hKey,
            ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'L' as u16, 'e' as u16, 'f' as u16, 't' as u16, 0].as_ptr(),
            &raw mut Globals.lMargins.left as *mut DWORD,
        );
        QueryDword(
            hKey,
            ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'T' as u16, 'o' as u16, 'p' as u16, 0].as_ptr(),
            &raw mut Globals.lMargins.top as *mut DWORD,
        );
        QueryDword(
            hKey,
            ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'R' as u16, 'i' as u16, 'g' as u16, 'h' as u16, 't' as u16, 0].as_ptr(),
            &raw mut Globals.lMargins.right as *mut DWORD,
        );
        QueryDword(
            hKey,
            ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'B' as u16, 'o' as u16, 't' as u16, 't' as u16, 'o' as u16, 'm' as u16, 0].as_ptr(),
            &raw mut Globals.lMargins.bottom as *mut DWORD,
        );

        QueryDword(
            hKey,
            ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'X' as u16, 0].as_ptr(),
            &raw mut x,
        );
        QueryDword(
            hKey,
            ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'Y' as u16, 0].as_ptr(),
            &raw mut y,
        );
        QueryDword(
            hKey,
            ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'D' as u16, 'X' as u16, 0].as_ptr(),
            &raw mut cx,
        );
        QueryDword(
            hKey,
            ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'D' as u16, 'Y' as u16, 0].as_ptr(),
            &raw mut cy,
        );

        QueryString(
            hKey,
            ['s' as u16, 'e' as u16, 'a' as u16, 'r' as u16, 'c' as u16, 'h' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 0].as_ptr(),
            &raw mut Globals.szFindText as LPTSTR,
            (core::mem::size_of::<[TCHAR; 260]>() / core::mem::size_of::<TCHAR>()) as DWORD,
        );
        QueryString(
            hKey,
            ['r' as u16, 'e' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'c' as u16, 'e' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 0].as_ptr(),
            &raw mut Globals.szReplaceText as LPTSTR,
            (core::mem::size_of::<[TCHAR; 260]>() / core::mem::size_of::<TCHAR>()) as DWORD,
        );
    }

    (*pWP).length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
    (*pWP).flags = 0;
    (*pWP).showCmd = SW_SHOWDEFAULT;
    if (cy & 0x80000000) != 0 {
        cy &= !0x80000000;
        (*pWP).flags |= WPF_RESTORETOMAXIMIZED;
        (*pWP).showCmd = SW_SHOWMAXIMIZED;
    }
    (*pWP).rcNormalPosition.left = x as LONG;
    (*pWP).rcNormalPosition.right = x.wrapping_add(cx) as LONG;
    (*pWP).rcNormalPosition.top = y as LONG;
    (*pWP).rcNormalPosition.bottom = y.wrapping_add(cy) as LONG;
    (*pWP).ptMaxPosition.x = x as LONG;
    (*pWP).ptMaxPosition.y = y as LONG;

    Globals.lfFont.lfHeight = HeightFromPointSize(dwPointSize);
    if hKey.is_null()
        || QueryString(
            hKey,
            ['l' as u16, 'f' as u16, 'F' as u16, 'a' as u16, 'c' as u16, 'e' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0].as_ptr(),
            &raw mut Globals.lfFont.lfFaceName as LPTSTR,
            (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>()) as DWORD,
        ) == 0
    {
        LoadString(
            Globals.hInstance,
            STRING_DEFAULTFONT,
            &raw mut Globals.lfFont.lfFaceName as LPTSTR,
            (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        );
    }

    if hKey.is_null()
        || QueryString(
            hKey,
            ['s' as u16, 'z' as u16, 'H' as u16, 'e' as u16, 'a' as u16, 'd' as u16, 'e' as u16, 'r' as u16, 0].as_ptr(),
            &raw mut Globals.szHeader as LPTSTR,
            (core::mem::size_of::<[TCHAR; 260]>() / core::mem::size_of::<TCHAR>()) as DWORD,
        ) == 0
    {
        LoadString(
            Globals.hInstance,
            STRING_PAGESETUP_HEADERVALUE,
            &raw mut Globals.szHeader as LPTSTR,
            (core::mem::size_of::<[TCHAR; 260]>() / core::mem::size_of::<TCHAR>()) as ::core::ffi::c_int,
        );
    }

    if hKey.is_null()
        || QueryString(
            hKey,
            ['s' as u16, 'z' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'r' as u16, 0].as_ptr(),
            &raw mut Globals.szFooter as LPTSTR,
            (core::mem::size_of::<[TCHAR; 260]>() / core::mem::size_of::<TCHAR>()) as DWORD,
        ) == 0
    {
        LoadString(
            Globals.hInstance,
            STRING_PAGESETUP_FOOTERVALUE,
            &raw mut Globals.szFooter as LPTSTR,
            (core::mem::size_of::<[TCHAR; 260]>() / core::mem::size_of::<TCHAR>()) as ::core::ffi::c_int,
        );
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    /* WORKAROUND: Far East Asian users may not have suitable fixed-pitch fonts. */
    match ((GetUserDefaultLangID() as ::core::ffi::c_int) & 0x3ff) as u32 {
        LANG_CHINESE | LANG_JAPANESE | LANG_KOREAN => {
            Globals.lfFont.lfPitchAndFamily = (DEFAULT_PITCH | FF_DONTCARE) as u8;
        }
        _ => {}
    }

    /* ARM64: Only FreeSans.ttf is available on the ramdisk.
     * Override to a face/pitch that the font mapper can match. */
    if cfg!(target_arch = "aarch64") {
        _tcscpy(
            &raw mut Globals.lfFont.lfFaceName as LPTSTR,
            ['F' as u16, 'r' as u16, 'e' as u16, 'e' as u16, 'S' as u16, 'a' as u16, 'n' as u16, 's' as u16, 0].as_ptr(),
        );
        Globals.lfFont.lfPitchAndFamily = (DEFAULT_PITCH | FF_SWISS) as u8;
    }

    hFont = CreateFontIndirect(&raw mut Globals.lfFont);
    SendMessage(
        Globals.hEdit,
        WM_SETFONT,
        hFont as WPARAM,
        (TRUE) as _,
    );
    if !hFont.is_null() {
        if !Globals.hFont.is_null() {
            DeleteObject(Globals.hFont as HGDIOBJ);
        }
        Globals.hFont = hFont;
    }
}

unsafe extern "C" fn SaveDword(hKey: HKEY, pszValueNameT: LPCTSTR, mut dwValue: DWORD) -> BOOL {
    return (RegSetValueEx(
        hKey,
        pszValueNameT as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(dwValue) as LPBYTE as *const BYTE,
        core::mem::size_of::<DWORD>() as DWORD,
    ) == ERROR_SUCCESS as LSTATUS) as BOOL;
}

unsafe extern "C" fn SaveString(hKey: HKEY, pszValueNameT: LPCTSTR, pszValue: LPCTSTR) -> BOOL {
    return ((RegSetValueEx(hKey, pszValueNameT, 0 as DWORD, REG_SZ as DWORD, pszValue as LPBYTE, ((_tcslen(pszValue) as DWORD) * (core::mem::size_of::<TCHAR>() as DWORD)))) as u32 == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn NOTEPAD_SaveSettingsToRegistry() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut wp: WINDOWPLACEMENT = core::mem::zeroed();
    let mut x: UINT = 0;
    let mut y: UINT = 0;
    let mut cx: UINT = 0;
    let mut cy: UINT = 0;

    wp.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
    GetWindowPlacement(Globals.hMainWnd, core::ptr::addr_of_mut!(wp));
    x = wp.rcNormalPosition.left as UINT;
    y = wp.rcNormalPosition.top as UINT;
    cx = (wp.rcNormalPosition.right as UINT).wrapping_sub(x);
    cy = (wp.rcNormalPosition.bottom as UINT).wrapping_sub(y);
    if wp.flags & WPF_RESTORETOMAXIMIZED != 0 {
        cy |= 0x80000000;
    }

    if (RegCreateKeyEx(HKEY_CURRENT_USER, s_szRegistryKey,
                       0, core::ptr::null_mut(), 0, KEY_SET_VALUE, core::ptr::null_mut(),
                       core::ptr::addr_of_mut!(hKey), core::ptr::addr_of_mut!(dwDisposition))) as u32 == ERROR_SUCCESS
    {
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'r' as u16, 'S' as u16, 'e' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfCharSet as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'C' as u16, 'l' as u16, 'i' as u16, 'p' as u16, 'P' as u16, 'r' as u16, 'e' as u16, 'c' as u16, 'i' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfClipPrecision as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'E' as u16, 's' as u16, 'c' as u16, 'a' as u16, 'p' as u16, 'e' as u16, 'm' as u16, 'e' as u16, 'n' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfEscapement as DWORD);
        SaveString(hKey, ['l' as u16, 'f' as u16, 'F' as u16, 'a' as u16, 'c' as u16, 'e' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0u16].as_ptr() as LPCTSTR, core::ptr::addr_of_mut!(Globals.lfFont.lfFaceName) as *mut WCHAR as LPCTSTR);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'I' as u16, 't' as u16, 'a' as u16, 'l' as u16, 'i' as u16, 'c' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfItalic as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'O' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfOrientation as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'O' as u16, 'u' as u16, 't' as u16, 'P' as u16, 'r' as u16, 'e' as u16, 'c' as u16, 'i' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfOutPrecision as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'P' as u16, 'i' as u16, 't' as u16, 'c' as u16, 'h' as u16, 'A' as u16, 'n' as u16, 'd' as u16, 'F' as u16, 'a' as u16, 'm' as u16, 'i' as u16, 'l' as u16, 'y' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfPitchAndFamily as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'Q' as u16, 'u' as u16, 'a' as u16, 'l' as u16, 'i' as u16, 't' as u16, 'y' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfQuality as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'k' as u16, 'e' as u16, 'O' as u16, 'u' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfStrikeOut as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'U' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'r' as u16, 'l' as u16, 'i' as u16, 'n' as u16, 'e' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfUnderline as DWORD);
        SaveDword(hKey, ['l' as u16, 'f' as u16, 'W' as u16, 'e' as u16, 'i' as u16, 'g' as u16, 'h' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lfFont.lfWeight as DWORD);
        SaveDword(hKey, ['i' as u16, 'P' as u16, 'o' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'S' as u16, 'i' as u16, 'z' as u16, 'e' as u16, 0u16].as_ptr() as LPCTSTR, PointSizeFromHeight(Globals.lfFont.lfHeight));
        SaveDword(hKey, ['f' as u16, 'W' as u16, 'r' as u16, 'a' as u16, 'p' as u16, 0u16].as_ptr() as LPCTSTR, (if Globals.bWrapLongLines != 0 { 1 } else { 0 }) as DWORD);
        SaveDword(hKey, ['f' as u16, 'S' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, 'B' as u16, 'a' as u16, 'r' as u16, 0u16].as_ptr() as LPCTSTR, (if Globals.bShowStatusBar != 0 { 1 } else { 0 }) as DWORD);
        SaveString(hKey, ['s' as u16, 'z' as u16, 'H' as u16, 'e' as u16, 'a' as u16, 'd' as u16, 'e' as u16, 'r' as u16, 0u16].as_ptr() as LPCTSTR, core::ptr::addr_of_mut!(Globals.szHeader) as *mut TCHAR as LPCTSTR);
        SaveString(hKey, ['s' as u16, 'z' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'r' as u16, 0u16].as_ptr() as LPCTSTR, core::ptr::addr_of_mut!(Globals.szFooter) as *mut TCHAR as LPCTSTR);
        SaveDword(hKey, ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'L' as u16, 'e' as u16, 'f' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lMargins.left as DWORD);
        SaveDword(hKey, ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'T' as u16, 'o' as u16, 'p' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lMargins.top as DWORD);
        SaveDword(hKey, ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'R' as u16, 'i' as u16, 'g' as u16, 'h' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lMargins.right as DWORD);
        SaveDword(hKey, ['i' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'i' as u16, 'n' as u16, 'B' as u16, 'o' as u16, 't' as u16, 't' as u16, 'o' as u16, 'm' as u16, 0u16].as_ptr() as LPCTSTR, Globals.lMargins.bottom as DWORD);
        SaveDword(hKey, ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'X' as u16, 0u16].as_ptr() as LPCTSTR, x as DWORD);
        SaveDword(hKey, ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'Y' as u16, 0u16].as_ptr() as LPCTSTR, y as DWORD);
        SaveDword(hKey, ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'D' as u16, 'X' as u16, 0u16].as_ptr() as LPCTSTR, cx as DWORD);
        SaveDword(hKey, ['i' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 's' as u16, 'D' as u16, 'Y' as u16, 0u16].as_ptr() as LPCTSTR, cy as DWORD);
        SaveString(hKey, ['s' as u16, 'e' as u16, 'a' as u16, 'r' as u16, 'c' as u16, 'h' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 0u16].as_ptr() as LPCTSTR, core::ptr::addr_of_mut!(Globals.szFindText) as *mut TCHAR as LPCTSTR);
        SaveString(hKey, ['r' as u16, 'e' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'c' as u16, 'e' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 0u16].as_ptr() as LPCTSTR, core::ptr::addr_of_mut!(Globals.szReplaceText) as *mut TCHAR as LPCTSTR);

        RegCloseKey(hKey);
    }
}

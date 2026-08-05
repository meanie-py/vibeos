// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static g_CPColors: [u16; 23] = [
    b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b' ' as u16,
    b'P' as u16, b'a' as u16, b'n' as u16, b'e' as u16, b'l' as u16, b'\\' as u16, b'C' as u16, b'o' as u16,
    b'l' as u16, b'o' as u16, b'r' as u16, b's' as u16, 0, 0, 0,
];

pub static g_CPANewSchemes: [u16; 38] = [
    b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b' ' as u16,
    b'P' as u16, b'a' as u16, b'n' as u16, b'e' as u16, b'l' as u16, b'\\' as u16, b'A' as u16, b'p' as u16,
    b'p' as u16, b'e' as u16, b'a' as u16, b'r' as u16, b'a' as u16, b'n' as u16, b'c' as u16, b'e' as u16,
    b'\\' as u16, b'N' as u16, b'e' as u16, b'w' as u16, b' ' as u16, b'S' as u16, b'c' as u16, b'h' as u16,
    b'e' as u16, b'm' as u16, b'e' as u16, b's' as u16, 0, 0,
];

pub static g_CPMetrics: [u16; 36] = [
    b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b' ' as u16,
    b'P' as u16, b'a' as u16, b'n' as u16, b'e' as u16, b'l' as u16, b'\\' as u16, b'D' as u16, b'e' as u16,
    b's' as u16, b'k' as u16, b't' as u16, b'o' as u16, b'p' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'M' as u16, b'e' as u16, b't' as u16, b'r' as u16,
    b'i' as u16, b'c' as u16, b's' as u16, 0,
];

pub static g_SelectedStyle: [u16; 14] = [
    b'S' as u16, b'e' as u16, b'l' as u16, b'e' as u16, b'c' as u16, b't' as u16, b'e' as u16, b'd' as u16,
    b'S' as u16, b't' as u16, b'y' as u16, b'l' as u16, b'e' as u16, 0,
];

pub static g_RegColorNames: [PCWSTR; 31] = [
    [b'S' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'l' as u16, b'b' as u16, b'a' as u16, b'r' as u16, 0].as_ptr(),
    [b'B' as u16, b'a' as u16, b'c' as u16, b'k' as u16, b'g' as u16, b'r' as u16, b'o' as u16, b'u' as u16, b'n' as u16, b'd' as u16, 0].as_ptr(),
    [b'A' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'T' as u16, b'i' as u16, b't' as u16, b'l' as u16, b'e' as u16, 0].as_ptr(),
    [b'I' as u16, b'n' as u16, b'a' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'T' as u16, b'i' as u16, b't' as u16, b'l' as u16, b'e' as u16, 0].as_ptr(),
    [b'M' as u16, b'e' as u16, b'n' as u16, b'u' as u16, 0].as_ptr(),
    [b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, 0].as_ptr(),
    [b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'F' as u16, b'r' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0].as_ptr(),
    [b'M' as u16, b'e' as u16, b'n' as u16, b'u' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'T' as u16, b'i' as u16, b't' as u16, b'l' as u16, b'e' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'A' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'B' as u16, b'o' as u16, b'r' as u16, b'd' as u16, b'e' as u16, b'r' as u16, 0].as_ptr(),
    [b'I' as u16, b'n' as u16, b'a' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'B' as u16, b'o' as u16, b'r' as u16, b'd' as u16, b'e' as u16, b'r' as u16, 0].as_ptr(),
    [b'A' as u16, b'p' as u16, b'p' as u16, b'W' as u16, b'o' as u16, b'r' as u16, b'k' as u16, b'S' as u16, b'p' as u16, b'a' as u16, b'c' as u16, b'e' as u16, 0].as_ptr(),
    [b'H' as u16, b'i' as u16, b'l' as u16, b'i' as u16, b'g' as u16, b'h' as u16, b't' as u16, 0].as_ptr(),
    [b'H' as u16, b'i' as u16, b'l' as u16, b'i' as u16, b'g' as u16, b'h' as u16, b't' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, b'F' as u16, b'a' as u16, b'c' as u16, b'e' as u16, 0].as_ptr(),
    [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, b'S' as u16, b'h' as u16, b'a' as u16, b'd' as u16, b'o' as u16, b'w' as u16, 0].as_ptr(),
    [b'G' as u16, b'r' as u16, b'a' as u16, b'y' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'I' as u16, b'n' as u16, b'a' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'T' as u16, b'i' as u16, b't' as u16, b'l' as u16, b'e' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, b'H' as u16, b'i' as u16, b'l' as u16, b'i' as u16, b'g' as u16, b'h' as u16, b't' as u16, 0].as_ptr(),
    [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, b'D' as u16, b'k' as u16, b'S' as u16, b'h' as u16, b'a' as u16, b'd' as u16, b'o' as u16, b'w' as u16, 0].as_ptr(),
    [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, b'L' as u16, b'i' as u16, b'g' as u16, b'h' as u16, b't' as u16, 0].as_ptr(),
    [b'I' as u16, b'n' as u16, b'f' as u16, b'o' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0].as_ptr(),
    [b'I' as u16, b'n' as u16, b'f' as u16, b'o' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, 0].as_ptr(),
    [b'B' as u16, b'u' as u16, b't' as u16, b't' as u16, b'o' as u16, b'n' as u16, b'A' as u16, b'l' as u16, b't' as u16, b'e' as u16, b'r' as u16, b'n' as u16, b'a' as u16, b't' as u16, b'e' as u16, b'F' as u16, b'a' as u16, b'c' as u16, b'e' as u16, 0].as_ptr(),
    [b'H' as u16, b'o' as u16, b't' as u16, b'T' as u16, b'r' as u16, b'a' as u16, b'c' as u16, b'k' as u16, b'i' as u16, b'n' as u16, b'g' as u16, b'C' as u16, b'o' as u16, b'l' as u16, b'o' as u16, b'r' as u16, 0].as_ptr(),
    [b'G' as u16, b'r' as u16, b'a' as u16, b'd' as u16, b'i' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'A' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'T' as u16, b'i' as u16, b't' as u16, b'l' as u16, b'e' as u16, 0].as_ptr(),
    [b'G' as u16, b'r' as u16, b'a' as u16, b'd' as u16, b'i' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'I' as u16, b'n' as u16, b'a' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'T' as u16, b'i' as u16, b't' as u16, b'l' as u16, b'e' as u16, 0].as_ptr(),
    [b'M' as u16, b'e' as u16, b'n' as u16, b'u' as u16, b'H' as u16, b'i' as u16, b'l' as u16, b'i' as u16, b'g' as u16, b'h' as u16, b't' as u16, 0].as_ptr(),
    [b'M' as u16, b'e' as u16, b'n' as u16, b'u' as u16, b'B' as u16, b'a' as u16, b'r' as u16, 0].as_ptr(),
];

pub type ENUMTHEMESTYLE = Option<unsafe extern "system" fn(LPCWSTR, LPWSTR, u32, PTHEMENAMES) -> HRESULT>;
#[no_mangle]
pub unsafe extern "C" fn SchemeSetMetric(scheme: *mut COLOR_SCHEME, id: i32, value: i32) {
    match id {
        SIZE_BORDER_WIDTH => (*scheme).ncMetrics.iBorderWidth = value,
        SIZE_SCROLL_WIDTH => (*scheme).ncMetrics.iScrollWidth = value,
        SIZE_SCROLL_HEIGHT => (*scheme).ncMetrics.iScrollHeight = value,
        SIZE_CAPTION_WIDTH => (*scheme).ncMetrics.iCaptionWidth = value,
        SIZE_CAPTION_HEIGHT => (*scheme).ncMetrics.iCaptionHeight = value,
        SIZE_SM_CAPTION_WIDTH => (*scheme).ncMetrics.iSmCaptionWidth = value,
        SIZE_SM_CAPTION_HEIGHT => (*scheme).ncMetrics.iSmCaptionHeight = value,
        SIZE_MENU_WIDTH => (*scheme).ncMetrics.iMenuWidth = value,
        SIZE_MENU_HEIGHT => (*scheme).ncMetrics.iMenuHeight = value,
        SIZE_ICON => (*scheme).iIconSize = value,
        SIZE_ICON_SPACE_X => (*scheme).icMetrics.iHorzSpacing = value,
        SIZE_ICON_SPACE_Y => (*scheme).icMetrics.iVertSpacing = value,
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn SchemeGetMetric(scheme: *mut COLOR_SCHEME, id: i32) -> i32 {
    match id {
        SIZE_BORDER_WIDTH => (*scheme).ncMetrics.iBorderWidth,
        SIZE_SCROLL_WIDTH => (*scheme).ncMetrics.iScrollWidth,
        SIZE_SCROLL_HEIGHT => (*scheme).ncMetrics.iScrollHeight,
        SIZE_CAPTION_WIDTH => (*scheme).ncMetrics.iCaptionWidth,
        SIZE_CAPTION_HEIGHT => (*scheme).ncMetrics.iCaptionHeight,
        SIZE_SM_CAPTION_WIDTH => (*scheme).ncMetrics.iSmCaptionWidth,
        SIZE_SM_CAPTION_HEIGHT => (*scheme).ncMetrics.iSmCaptionHeight,
        SIZE_MENU_WIDTH => (*scheme).ncMetrics.iMenuWidth,
        SIZE_MENU_HEIGHT => (*scheme).ncMetrics.iMenuHeight,
        SIZE_ICON => (*scheme).iIconSize,
        SIZE_ICON_SPACE_X => (*scheme).icMetrics.iHorzSpacing,
        SIZE_ICON_SPACE_Y => (*scheme).icMetrics.iVertSpacing,
        _ => 0,
    }
}
#[no_mangle]
pub unsafe extern "C" fn SchemeGetFont(scheme: *mut COLOR_SCHEME, id: i32) -> PLOGFONTW {
    match id {
        FONT_CAPTION => core::ptr::addr_of_mut!((*scheme).ncMetrics.lfCaptionFont),
        FONT_SMCAPTION => core::ptr::addr_of_mut!((*scheme).ncMetrics.lfSmCaptionFont),
        FONT_MENU => core::ptr::addr_of_mut!((*scheme).ncMetrics.lfMenuFont),
        FONT_STATUS => core::ptr::addr_of_mut!((*scheme).ncMetrics.lfStatusFont),
        FONT_MESSAGE => core::ptr::addr_of_mut!((*scheme).ncMetrics.lfMessageFont),
        FONT_ICON => core::ptr::addr_of_mut!((*scheme).icMetrics.lfFont),
        _ => core::ptr::null_mut(),
    }
}
#[no_mangle]
pub unsafe extern "C" fn LoadCurrentScheme(scheme: *mut COLOR_SCHEME) -> i32 {
    let mut i: i32;
    let mut Result: i32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut ret: i32;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    let mut osvi: OSVERSIONINFO = core::mem::zeroed();

    for i in 0..NUM_COLORS {
        (*scheme).crColor[i as usize] = GetSysColor(i) as COLORREF;
    }

    (*scheme).ncMetrics.cbSize = NONCLIENTMETRICSW;

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        if GetVersionEx(core::ptr::addr_of_mut!(osvi)) != 0 {
            if osvi.dwMajorVersion <= 5 {
                (*scheme).ncMetrics.cbSize -= core::mem::size_of_val(&(*scheme).ncMetrics.iPaddedBorderWidth) as u32;
            }
        }
    }

    ret = SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS,
        NONCLIENTMETRICSW,
        core::ptr::addr_of_mut!((*scheme).ncMetrics) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    (*scheme).icMetrics.cbSize = ICONMETRICSW;
    ret = SystemParametersInfoW(
        SPI_GETICONMETRICS,
        ICONMETRICSW,
        core::ptr::addr_of_mut!((*scheme).icMetrics) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    ret = SystemParametersInfoW(
        SPI_GETFLATMENU,
        0,
        core::ptr::addr_of_mut!((*scheme).bFlatMenus) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    ret = SystemParametersInfoW(
        SPI_GETMENUANIMATION,
        0,
        core::ptr::addr_of_mut!((*scheme).Effects.bMenuAnimation) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    ret = SystemParametersInfoW(
        SPI_GETMENUFADE,
        0,
        core::ptr::addr_of_mut!((*scheme).Effects.bMenuFade) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    (*scheme).Effects.bTooltipAnimation = (*scheme).Effects.bMenuAnimation;
    (*scheme).Effects.bTooltipFade = (*scheme).Effects.bMenuFade;

    ret = SystemParametersInfoW(
        SPI_GETFONTSMOOTHING,
        0,
        core::ptr::addr_of_mut!((*scheme).Effects.bFontSmoothing) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    ret = SystemParametersInfoW(
        SPI_GETFONTSMOOTHINGTYPE,
        0,
        core::ptr::addr_of_mut!((*scheme).Effects.uiFontSmoothingType) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    ret = SystemParametersInfoW(
        SPI_GETDROPSHADOW,
        0,
        core::ptr::addr_of_mut!((*scheme).Effects.bDropShadow) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    ret = SystemParametersInfoW(
        SPI_GETDRAGFULLWINDOWS,
        0,
        core::ptr::addr_of_mut!((*scheme).Effects.bDragFullWindows) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    ret = SystemParametersInfoW(
        SPI_GETKEYBOARDCUES,
        0,
        core::ptr::addr_of_mut!((*scheme).Effects.bKeyboardCues) as LPVOID,
        0,
    );
    if ret == 0 {
        return FALSE;
    }

    Result = RegOpenKeyW(HKEY_CURRENT_USER, g_CPMetrics.as_ptr(), core::ptr::addr_of_mut!(hKey));
    if Result == ERROR_SUCCESS {
        (*scheme).iIconSize = SHRegGetIntW(hKey, b"Shell Icon Size\0".as_ptr() as LPCWSTR, 32);
        RegCloseKey(hKey);
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSchemeFromReg(scheme: *mut COLOR_SCHEME, pSelectedTheme: PTHEME_SELECTION) -> BOOL {
    let mut i: INT;
    let mut strValueName: [u16; 10] = [0; 10];
    let mut strSchemeKey: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut hkScheme: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut iSize: UINT64 = 0;
    let mut Ret: BOOL = TRUE;
    let mut result: LONG;

    wsprintf(
        strSchemeKey.as_mut_ptr(),
        b"%s\\%s\\Sizes\\%s\0".as_ptr() as PCWSTR,
        g_CPANewSchemes.as_ptr(),
        (*(*pSelectedTheme).Color).StyleName.as_ptr(),
        (*(*pSelectedTheme).Size).StyleName.as_ptr()
    );

    result = RegOpenKeyW(HKEY_CURRENT_USER, strSchemeKey.as_ptr(), &mut hkScheme);
    if result != ERROR_SUCCESS {
        return FALSE;
    }

    (*scheme).bFlatMenus = SHRegGetIntW(hkScheme, b"FlatMenus\0".as_ptr() as PCWSTR, 0);

    i = 0;
    while i < NUM_COLORS {
        wsprintf(strValueName.as_mut_ptr(), b"Color #%d\0".as_ptr() as PCWSTR, i);
        dwLength = core::mem::size_of_val(&COLORREF) as DWORD;
        result = RegQueryValueExW(
            hkScheme,
            strValueName.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            core::ptr::addr_of_mut!((*scheme).crColor[i as usize]) as LPBYTE,
            &mut dwLength
        );
        if result != ERROR_SUCCESS || dwType != REG_DWORD {
            (*scheme).crColor[i as usize] = GetSysColor(i);
        }
        i += 1;
    }

    i = 0;
    while i < NUM_FONTS {
        let lpfFont: PLOGFONTW = SchemeGetFont(scheme, i);

        wsprintf(strValueName.as_mut_ptr(), b"Font #%d\0".as_ptr() as PCWSTR, i);
        dwLength = core::mem::size_of_val(&LOGFONT) as DWORD;
        result = RegQueryValueExW(
            hkScheme,
            strValueName.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            lpfFont as LPBYTE,
            &mut dwLength
        );
        if result != ERROR_SUCCESS || dwType != REG_BINARY ||
           dwLength != core::mem::size_of_val(&LOGFONT) as DWORD {
            Ret = FALSE;
        }
        i += 1;
    }

    i = 0;
    while i < NUM_SIZES {
        wsprintf(strValueName.as_mut_ptr(), b"Size #%d\0".as_ptr() as PCWSTR, i);
        dwLength = core::mem::size_of_val(&UINT64) as DWORD;
        result = RegQueryValueExW(
            hkScheme,
            strValueName.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            core::ptr::addr_of_mut!(iSize) as LPBYTE,
            &mut dwLength
        );
        if result != ERROR_SUCCESS || dwType != REG_QWORD ||
           dwLength != core::mem::size_of_val(&UINT64) as DWORD {
        } else {
            SchemeSetMetric(scheme, i, iSize as i32);
        }
        i += 1;
    }

    RegCloseKey(hkScheme);

    Ret
}
#[no_mangle]
pub unsafe extern "C" fn ApplyScheme(scheme: *mut COLOR_SCHEME, pSelectedTheme: *mut THEME_SELECTION) {
    let mut i: i32;
    let mut Result: i32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut clText: [u16; 16] = [0; 16];
    let mut StyleName: *mut u16;
    let mut ColorList: [i32; NUM_COLORS] = [0; NUM_COLORS];

    i = 0;
    while i < NUM_COLORS {
        ColorList[i as usize] = i;
        i += 1;
    }
    SetSysColors(NUM_COLORS, ColorList.as_mut_ptr(), (*scheme).crColor.as_mut_ptr());

    Result = RegCreateKeyW(HKEY_CURRENT_USER, g_CPColors.as_ptr(), core::ptr::addr_of_mut!(hKey));
    if Result == ERROR_SUCCESS {
        i = 0;
        while i < NUM_COLORS {
            wsprintf(
                clText.as_mut_ptr(),
                b"L\"%d %d %d\"\0".as_ptr() as PCWSTR,
                GetRValue((*scheme).crColor[i as usize]),
                GetGValue((*scheme).crColor[i as usize]),
                GetBValue((*scheme).crColor[i as usize])
            );

            RegSetValueExW(
                hKey,
                g_RegColorNames[i as usize],
                0,
                REG_SZ,
                clText.as_mut_ptr() as LPBYTE,
                (lstrlen(clText.as_ptr()) + 1) * core::mem::size_of::<u16>() as i32
            );
            i += 1;
        }
        RegCloseKey(hKey);
    }

    SystemParametersInfoW(
        SPI_SETNONCLIENTMETRICS,
        core::mem::size_of_val(&NONCLIENTMETRICS) as u32,
        core::ptr::addr_of_mut!((*scheme).ncMetrics) as LPVOID,
        SPIF_UPDATEINIFILE | SPIF_SENDCHANGE
    );

    SystemParametersInfoW(
        SPI_SETICONMETRICS,
        core::mem::size_of_val(&ICONMETRICS) as u32,
        core::ptr::addr_of_mut!((*scheme).icMetrics) as LPVOID,
        SPIF_UPDATEINIFILE | SPIF_SENDCHANGE
    );

    (*scheme).Effects.bTooltipAnimation = (*scheme).Effects.bMenuAnimation;
    (*scheme).Effects.bTooltipFade = (*scheme).Effects.bMenuFade;

    SystemParametersInfoW(SPI_SETMENUANIMATION, 0, IntToPtr((*scheme).Effects.bMenuAnimation), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);
    SystemParametersInfoW(SPI_SETMENUFADE, 0, IntToPtr((*scheme).Effects.bMenuFade), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);

    SystemParametersInfoW(SPI_SETFONTSMOOTHING, (*scheme).Effects.bFontSmoothing, 0, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);
    SystemParametersInfoW(SPI_SETFONTSMOOTHINGTYPE, 0, IntToPtr((*scheme).Effects.uiFontSmoothingType), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);

    InvalidateRect(core::ptr::null_mut(), core::ptr::null_mut(), TRUE);

    SystemParametersInfoW(SPI_SETDROPSHADOW, 0, IntToPtr((*scheme).Effects.bDropShadow), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);

    SystemParametersInfoW(SPI_SETDRAGFULLWINDOWS, (*scheme).Effects.bDragFullWindows, 0, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);

    SystemParametersInfoW(SPI_SETKEYBOARDCUES, 0, IntToPtr((*scheme).Effects.bKeyboardCues), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);

    SystemParametersInfoW(SPI_SETFLATMENU, 0, IntToPtr((*scheme).bFlatMenus), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);

    SystemParametersInfoW(SPI_SETTOOLTIPANIMATION, 0, IntToPtr((*scheme).Effects.bTooltipAnimation), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);
    SystemParametersInfoW(SPI_SETTOOLTIPFADE, 0, IntToPtr((*scheme).Effects.bTooltipFade), SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);

    if !(*pSelectedTheme).Theme.is_null() && (*pSelectedTheme).ThemeActive == FALSE {
        StyleName = (*(*pSelectedTheme).Color).StyleName;
        SHSetValueW(
            HKEY_CURRENT_USER,
            g_CPANewSchemes.as_ptr(),
            g_SelectedStyle.as_ptr(),
            REG_SZ,
            StyleName as *const u8,
            (lstrlenW(StyleName) + 1) * core::mem::size_of::<u16>() as u32
        );
    }

    if (*pSelectedTheme).ThemeActive != 0 {
        SHSetValueW(HKEY_CURRENT_USER, b"L\"Control Panel\\\\Appearance\"\0".as_ptr() as PCWSTR, b"L\"Current\"\0".as_ptr() as PCWSTR, REG_SZ, core::ptr::null_mut(), 0);
        SHSetValueW(HKEY_CURRENT_USER, b"L\"Control Panel\\\\Appearance\"\0".as_ptr() as PCWSTR, b"L\"NewCurrent\"\0".as_ptr() as PCWSTR, REG_SZ, core::ptr::null_mut(), 0);
    } else if !(*pSelectedTheme).Color.is_null() {
        let ClassicSchemeName: PCWSTR = (*(*pSelectedTheme).Color).DisplayName;
        let cb: u32 = (lstrlenW(ClassicSchemeName) + 1) * core::mem::size_of::<u16>() as u32;
        SHSetValueW(HKEY_CURRENT_USER, b"L\"Control Panel\\\\Appearance\"\0".as_ptr() as PCWSTR, b"L\"Current\"\0".as_ptr() as PCWSTR, REG_SZ, ClassicSchemeName as *const u8, cb);
        SHSetValueW(HKEY_CURRENT_USER, b"L\"Control Panel\\\\Appearance\"\0".as_ptr() as PCWSTR, b"L\"NewCurrent\"\0".as_ptr() as PCWSTR, REG_SZ, ClassicSchemeName as *const u8, cb);
    }
}
unsafe extern "C" fn CreateTheme(pszName: LPCWSTR, pszDisplayName: LPCWSTR) -> *mut THEME {
    let pTheme: PTHEME;

    pTheme = malloc(core::mem::size_of::<THEME>()) as PTHEME;
    if pTheme.is_null() {
        return core::ptr::null_mut();
    }

    (*pTheme).DisplayName = _wcsdup(pszDisplayName);
    if (*pTheme).DisplayName.is_null() {
        free(pTheme as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*pTheme).ColoursList = core::ptr::null_mut();
    (*pTheme).NextTheme = core::ptr::null_mut();
    (*pTheme).SizesList = core::ptr::null_mut();

    if pszName.is_null() {
        (*pTheme).ThemeFileName = core::ptr::null_mut();
        return pTheme;
    }

    (*pTheme).ThemeFileName = _wcsdup(pszName);
    if (*pTheme).ThemeFileName.is_null() {
        free((*pTheme).DisplayName as *mut core::ffi::c_void);
        free(pTheme as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    pTheme
}
unsafe extern "C" fn CreateStyle(pszName: LPCWSTR, pszDisplayName: LPCWSTR) -> PTHEME_STYLE {
    let pStyle: PTHEME_STYLE = malloc(core::mem::size_of::<THEME_STYLE>()) as PTHEME_STYLE;
    if pStyle.is_null() {
        return core::ptr::null_mut();
    }

    (*pStyle).StyleName = _wcsdup(pszName);
    if (*pStyle).StyleName.is_null() {
        free(pStyle as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*pStyle).DisplayName = _wcsdup(pszDisplayName);
    if (*pStyle).DisplayName.is_null() {
        free((*pStyle).StyleName as *mut core::ffi::c_void);
        free(pStyle as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*pStyle).ChildStyle = core::ptr::null_mut();
    (*pStyle).NextStyle = core::ptr::null_mut();

    pStyle
}
unsafe extern "C" fn CleanupStyles(pStylesList: PTHEME_STYLE) {
    let mut pStyle: PTHEME_STYLE;
    let mut pStyleOld: PTHEME_STYLE;

    pStyle = pStylesList;
    while !pStyle.is_null() {
        if !(*pStyle).ChildStyle.is_null() {
            CleanupStyles((*pStyle).ChildStyle);
        }
        if !(*pStyle).DisplayName.is_null() {
            free((*pStyle).DisplayName);
        }
        if !(*pStyle).StyleName.is_null() {
            free((*pStyle).StyleName);
        }

        pStyleOld = pStyle;
        pStyle = (*pStyle).NextStyle;
        free(pStyleOld);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CleanupThemes(mut pThemeList: PTHEME) {
    let mut pTheme: PTHEME;
    let mut pThemeOld: PTHEME;

    pTheme = pThemeList;
    while !pTheme.is_null() {
        CleanupStyles((*pTheme).ColoursList);
        if !(*pTheme).SizesList.is_null() {
            CleanupStyles((*pTheme).SizesList);
        }
        if !(*pTheme).DisplayName.is_null() {
            free((*pTheme).DisplayName);
        }
        if !(*pTheme).ThemeFileName.is_null() {
            free((*pTheme).ThemeFileName);
        }

        pThemeOld = pTheme;
        pTheme = (*pTheme).NextTheme;
        free(pThemeOld);
    }
}
unsafe extern "C" fn FindStyle(pStylesList: PTHEME_STYLE, StyleName: PCWSTR) -> PTHEME_STYLE {
    let mut pStyle = pStylesList;
    while !pStyle.is_null() {
        if _wcsicmp((*pStyle).StyleName, StyleName) == 0 {
            return pStyle;
        }
        pStyle = (*pStyle).NextStyle;
    }

    pStylesList
}
unsafe extern "C" fn LoadSchemeSizes(hkScheme: HKEY) -> *mut THEME_STYLE {
    let mut hkSizes: HKEY = core::ptr::null_mut();
    let mut hkSize: HKEY = core::ptr::null_mut();
    let mut Result: i32;
    let mut iStyle: i32;
    let mut wstrSizeName: [u16; 5] = [0; 5];
    let mut wstrDisplayName: [u16; 50] = [0; 50];
    let mut List: *mut THEME_STYLE = core::ptr::null_mut();
    let mut pCurrentStyle: *mut THEME_STYLE;

    Result = RegOpenKeyW(hkScheme, b"S\0i\0z\0e\0s\0\0\0".as_ptr() as PCWSTR, &mut hkSizes);
    if Result != ERROR_SUCCESS {
        return core::ptr::null_mut();
    }

    iStyle = 0;
    while RegEnumKeyW(hkSizes, iStyle, wstrSizeName.as_mut_ptr(), 5) == ERROR_SUCCESS {
        iStyle += 1;

        Result = RegOpenKeyW(hkSizes, wstrSizeName.as_ptr(), &mut hkSize);
        if Result != ERROR_SUCCESS {
            continue;
        }

        Result = RegLoadMUIStringW(
            hkSize,
            b"D\0i\0s\0p\0l\0a\0y\0N\0a\0m\0e\0\0\0".as_ptr() as PCWSTR,
            wstrDisplayName.as_mut_ptr(),
            core::mem::size_of_val(&wstrDisplayName) as u32,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if Result != ERROR_SUCCESS {
            Result = RegLoadMUIStringW(
                hkSize,
                b"L\0e\0g\0a\0c\0y\0N\0a\0m\0e\0\0\0".as_ptr() as PCWSTR,
                wstrDisplayName.as_mut_ptr(),
                core::mem::size_of_val(&wstrDisplayName) as u32,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
        }

        if Result == ERROR_SUCCESS {
            pCurrentStyle = CreateStyle(wstrSizeName.as_ptr(), wstrDisplayName.as_ptr());
        } else {
            pCurrentStyle = CreateStyle(wstrSizeName.as_ptr(), wstrSizeName.as_ptr());
        }

        if !pCurrentStyle.is_null() {
            (*pCurrentStyle).NextStyle = List;
            List = pCurrentStyle;
        }

        RegCloseKey(hkSize);
    }

    RegCloseKey(hkSizes);
    List
}
unsafe extern "C" fn LoadClassicColorSchemes() -> THEME_STYLE {
    let mut Result: INT_PTR;
    let mut hkNewSchemes: HKEY = core::ptr::null_mut();
    let mut hkScheme: HKEY = core::ptr::null_mut();
    let mut iStyle: INT_PTR;
    let mut wstrStyleName: [u16; 5] = [0; 5];
    let mut wstrDisplayName: [u16; 50] = [0; 50];
    let mut List: *mut THEME_STYLE = core::ptr::null_mut();
    let mut pCurrentStyle: *mut THEME_STYLE;

    Result = RegOpenKeyW(HKEY_CURRENT_USER, g_CPANewSchemes.as_ptr(), &mut hkNewSchemes);
    if Result != ERROR_SUCCESS {
        return core::ptr::null_mut();
    }

    iStyle = 0;
    while RegEnumKeyW(hkNewSchemes, iStyle, wstrStyleName.as_mut_ptr(), 5) == ERROR_SUCCESS {
        iStyle += 1;

        Result = RegOpenKeyW(hkNewSchemes, wstrStyleName.as_ptr(), &mut hkScheme);
        if Result != ERROR_SUCCESS {
            continue;
        }

        Result = RegLoadMUIStringW(
            hkScheme,
            b"D\0i\0s\0p\0l\0a\0y\0N\0a\0m\0e\0\0\0".as_ptr() as LPCWSTR,
            wstrDisplayName.as_mut_ptr(),
            core::mem::size_of_val(&wstrDisplayName) as u32,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if Result != ERROR_SUCCESS {
            Result = RegLoadMUIStringW(
                hkScheme,
                b"L\0e\0g\0a\0c\0y\0N\0a\0m\0e\0\0\0".as_ptr() as LPCWSTR,
                wstrDisplayName.as_mut_ptr(),
                core::mem::size_of_val(&wstrDisplayName) as u32,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
        }

        if Result == ERROR_SUCCESS {
            pCurrentStyle = CreateStyle(wstrStyleName.as_ptr(), wstrDisplayName.as_ptr());
        } else {
            pCurrentStyle = CreateStyle(wstrStyleName.as_ptr(), wstrStyleName.as_ptr());
        }

        if pCurrentStyle != core::ptr::null_mut() {
            (*pCurrentStyle).NextStyle = List;
            (*pCurrentStyle).ChildStyle = LoadSchemeSizes(hkScheme);
            if (*pCurrentStyle).ChildStyle == core::ptr::null_mut() {
                CleanupStyles(pCurrentStyle);
            } else {
                List = pCurrentStyle;
            }
        }

        RegCloseKey(hkScheme);
    }

    RegCloseKey(hkNewSchemes);
    List
}
unsafe extern "C" fn EnumThemeStyles(pszThemeFileName: LPCWSTR, pfnEnumTheme: ENUMTHEMESTYLE) -> *mut THEME_STYLE {
    let mut index: DWORD = 0;
    let mut names: THEMENAMES = core::mem::zeroed();
    let mut List: *mut THEME_STYLE = core::ptr::null_mut();
    let mut ppPrevStyle: *mut *mut THEME_STYLE;
    let mut pCurrentStyle: *mut THEME_STYLE;

    ppPrevStyle = &mut List;

    while ((pfnEnumTheme(pszThemeFileName, core::ptr::null(), index, &mut names)) >= 0) >= 0 {
        index = index.wrapping_add(1);
        pCurrentStyle = CreateStyle(names.szName.as_ptr(), names.szDisplayName.as_ptr());
        if pCurrentStyle == core::ptr::null_mut() {
            break;
        }

        *ppPrevStyle = pCurrentStyle;
        ppPrevStyle = &mut (*pCurrentStyle).NextStyle;
    }

    List
}
#[no_mangle]
pub unsafe extern "C" fn LoadTheme(pszThemeFileName: LPCWSTR, pszThemeName: LPCWSTR) -> *mut THEME {
    let pTheme = CreateTheme(pszThemeFileName, pszThemeName);
    if pTheme.is_null() {
        return core::ptr::null_mut();
    }

    (*pTheme).SizesList = EnumThemeStyles(pszThemeFileName, EnumThemeSizes as ENUMTHEMESTYLE);
    (*pTheme).ColoursList = EnumThemeStyles(pszThemeFileName, EnumThemeColors as ENUMTHEMESTYLE);
    if (*pTheme).SizesList.is_null() || (*pTheme).ColoursList.is_null() {
        CleanupThemes(pTheme);
        return core::ptr::null_mut();
    }

    pTheme
}

#[no_mangle]
pub unsafe extern "system" fn EnumThemeProc(
    lpReserved: LPVOID,
    pszThemeFileName: LPCWSTR,
    pszThemeName: LPCWSTR,
    pszToolTip: LPCWSTR,
    lpReserved2: LPVOID,
    lpData: LPVOID,
) -> i32 {
    let List: *mut *mut THEME = lpData as *mut *mut THEME;
    let pTheme = LoadTheme(pszThemeFileName, pszThemeName);
    if pTheme == core::ptr::null_mut() {
        return FALSE;
    }

    (*pTheme).NextTheme = *List;
    *List = pTheme;

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn LoadThemes() -> *mut THEME {
    let mut hret: HRESULT;
    let mut pClassicTheme: *mut THEME;
    let mut strClassicTheme: [u16; 40] = [0; 40];
    let mut szThemesPath: [u16; MAX_PATH] = [0; MAX_PATH];
    let pszClassicTheme: LPCWSTR;
    let res: i32;

    res = LoadString(hApplet, IDS_CLASSIC_THEME, strClassicTheme.as_mut_ptr(), 40);
    pszClassicTheme = if res > 0 {
        strClassicTheme.as_ptr()
    } else {
        b"C\0l\0a\0s\0s\0i\0c\0 \0T\0h\0e\0m\0e\0\0\0".as_ptr() as LPCWSTR
    };
    pClassicTheme = CreateTheme(core::ptr::null(), pszClassicTheme);
    if pClassicTheme.is_null() {
        return core::ptr::null_mut();
    }
    (*pClassicTheme).ColoursList = LoadClassicColorSchemes();

    ZeroMemory(szThemesPath.as_mut_ptr() as LPVOID, core::mem::size_of_val(&szThemesPath));
    hret = SHGetFolderPathW(core::ptr::null_mut(), CSIDL_RESOURCES, core::ptr::null_mut(), SHGFP_TYPE_DEFAULT, szThemesPath.as_mut_ptr());
    if hret < 0 {
        return pClassicTheme;
    }
    lstrcatW(szThemesPath.as_mut_ptr(), b"\\\0T\0h\0e\0m\0e\0s\0\0\0".as_ptr() as LPCWSTR);

    hret = EnumThemes(szThemesPath.as_ptr(), EnumThemeProc, core::ptr::addr_of_mut!((*pClassicTheme).NextTheme) as LPVOID);
    if hret < 0 {
        (*pClassicTheme).NextTheme = core::ptr::null_mut();
        if (*pClassicTheme).ColoursList.is_null() {
            free((*pClassicTheme).DisplayName);
            free(pClassicTheme as LPVOID);
            return core::ptr::null_mut();
        }
    }

    pClassicTheme
}
#[no_mangle]
pub unsafe extern "C" fn FindOrAppendTheme(
    pThemeList: *mut THEME,
    pwszThemeFileName: LPCWSTR,
    pwszColorBuff: LPCWSTR,
    pwszSizeBuff: LPCWSTR,
    pSelectedTheme: *mut THEME_SELECTION,
) -> BOOL {
    let mut pTheme: *mut THEME;
    let mut pFoundTheme: *mut THEME = core::ptr::null_mut();

    ZeroMemory(
        pSelectedTheme as *mut core::ffi::c_void,
        core::mem::size_of::<THEME_SELECTION>(),
    );

    pTheme = pThemeList;
    while !pTheme.is_null() {
        if !(*pTheme).ThemeFileName.is_null()
            && _wcsicmp((*pTheme).ThemeFileName, pwszThemeFileName)
                == 0
        {
            pFoundTheme = pTheme;
            break;
        }

        if (*pTheme).NextTheme.is_null() {
            break;
        }

        pTheme = (*pTheme).NextTheme;
    }

    if pFoundTheme.is_null() {
        pFoundTheme = LoadTheme(pwszThemeFileName, pwszThemeFileName);
        if pFoundTheme.is_null() {
            return FALSE;
        }

        (*pTheme).NextTheme = pFoundTheme;
    }

    (*pSelectedTheme).ThemeActive = TRUE;
    (*pSelectedTheme).Theme = pFoundTheme;
    if !pwszColorBuff.is_null() {
        (*pSelectedTheme).Color = FindStyle((*pFoundTheme).ColoursList, pwszColorBuff);
    } else {
        (*pSelectedTheme).Color = (*pFoundTheme).ColoursList;
    }

    if !pwszSizeBuff.is_null() {
        (*pSelectedTheme).Size = FindStyle((*pFoundTheme).SizesList, pwszSizeBuff);
    } else {
        (*pSelectedTheme).Size = (*pFoundTheme).SizesList;
    }

    TRUE
}
#[no_mangle]
pub unsafe extern "C" fn GetActiveTheme(pThemeList: *mut THEME, pSelectedTheme: *mut THEME_SELECTION) -> BOOL {
    let mut szThemeFileName: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut szColorBuff: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut szSizeBuff: [u16; MAX_PATH] = [0; MAX_PATH];
    let hret: HRESULT;

    hret = GetCurrentThemeName(
        szThemeFileName.as_mut_ptr(),
        MAX_PATH,
        szColorBuff.as_mut_ptr(),
        MAX_PATH,
        szSizeBuff.as_mut_ptr(),
        MAX_PATH,
    );
    if ((hret) < 0) < 0 {
        return FALSE;
    }

    return FindOrAppendTheme(
        pThemeList,
        szThemeFileName.as_ptr(),
        szColorBuff.as_ptr(),
        szSizeBuff.as_ptr(),
        pSelectedTheme,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetActiveClassicTheme(pThemeList: PTHEME, pSelectedTheme: PTHEME_SELECTION) -> BOOL {
    let mut Result: INT;
    let mut szSelectedClassicScheme: [WCHAR; 5] = [0; 5];
    let mut szSelectedClassicSize: [WCHAR; 5] = [0; 5];
    let mut hkNewSchemes: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD;
    let mut dwDisplayNameLength: DWORD;
    let mut pCurrentStyle: PTHEME_STYLE;
    let mut pCurrentSize: PTHEME_STYLE;

    ZeroMemory(pSelectedTheme as *mut u8, core::mem::size_of::<THEME_SELECTION>());

    szSelectedClassicScheme[0] = 0;
    szSelectedClassicSize[0] = 0;

    Result = RegOpenKeyW(HKEY_CURRENT_USER, g_CPANewSchemes.as_ptr(), &mut hkNewSchemes);
    if Result != ERROR_SUCCESS {
        return FALSE;
    }

    dwType = REG_SZ;
    dwDisplayNameLength = core::mem::size_of_val(&szSelectedClassicScheme) as DWORD;
    Result = RegQueryValueEx(
        hkNewSchemes,
        b"S\0e\0l\0e\0c\0t\0e\0d\0S\0t\0y\0l\0e\0\0\0".as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        szSelectedClassicScheme.as_mut_ptr() as LPBYTE,
        &mut dwDisplayNameLength,
    );
    if Result == ERROR_SUCCESS {
        dwType = REG_SZ;
        dwDisplayNameLength = core::mem::size_of_val(&szSelectedClassicSize) as DWORD;
        Result = SHGetValue(
            hkNewSchemes,
            szSelectedClassicScheme.as_ptr(),
            b"S\0e\0l\0e\0c\0t\0e\0d\0S\0i\0z\0e\0\0\0".as_ptr() as LPCWSTR,
            &mut dwType,
            szSelectedClassicSize.as_mut_ptr() as LPVOID,
            &mut dwDisplayNameLength,
        );
    }

    RegCloseKey(hkNewSchemes);

    pCurrentStyle = FindStyle((*pThemeList).ColoursList, szSelectedClassicScheme.as_ptr());
    pCurrentSize = FindStyle((*pCurrentStyle).ChildStyle, szSelectedClassicSize.as_ptr());

    (*pSelectedTheme).Theme = pThemeList;
    (*pSelectedTheme).Color = pCurrentStyle;
    (*pSelectedTheme).Size = pCurrentSize;

    TRUE
}
#[no_mangle]
pub unsafe extern "C" fn ActivateTheme(pSelectedTheme: PTHEME_SELECTION) -> i32 {
    let mut hThemeFile: HTHEMEFILE = 0;
    let mut hret: HRESULT;

    if (*pSelectedTheme).ThemeActive != 0 {
        hret = OpenThemeFile(
            (*(*pSelectedTheme).Theme).ThemeFileName,
            (*(*pSelectedTheme).Color).StyleName,
            (*(*pSelectedTheme).Size).StyleName,
            &mut hThemeFile,
            0
        );

        if !((hret) >= 0) {
            return FALSE;
        }
    }

    hret = ApplyTheme(hThemeFile, 0, 0);

    if (*pSelectedTheme).ThemeActive != 0 {
        CloseThemeFile(hThemeFile);
    }

    ((hret) >= 0) as i32
}
#[no_mangle]
pub unsafe extern "C" fn LoadSchemeFromTheme(scheme: *mut COLOR_SCHEME, pSelectedTheme: *mut THEME_SELECTION) -> i32 {
    let mut hThemeFile: HTHEMEFILE = 0;
    let hret: HRESULT;
    let hTheme: HTHEME;
    let mut i: i32;

    hret = OpenThemeFile(
        (*(*pSelectedTheme).Theme).ThemeFileName,
        (*(*pSelectedTheme).Color).StyleName,
        (*(*pSelectedTheme).Size).StyleName,
        &mut hThemeFile,
        0
    );

    if !((hret) >= 0) {
        return FALSE;
    }

    hTheme = OpenThemeDataFromFile(
        hThemeFile,
        hCPLWindow,
        b"W\0I\0N\0D\0O\0W\0\0\0".as_ptr() as PCWSTR,
        0
    );

    if hTheme.is_null() {
        return FALSE;
    }

    i = 0;
    while i < NUM_COLORS {
        (*scheme).crColor[i as usize] = GetThemeSysColor(hTheme, i);
        i += 1;
    }

    (*scheme).ncMetrics.iBorderWidth = GetThemeSysSize(hTheme, SM_CXFRAME);
    (*scheme).ncMetrics.iScrollWidth = GetThemeSysSize(hTheme, SM_CXVSCROLL);
    (*scheme).ncMetrics.iScrollHeight = GetThemeSysSize(hTheme, SM_CYHSCROLL);
    (*scheme).ncMetrics.iCaptionWidth = GetThemeSysSize(hTheme, SM_CXSIZE);
    (*scheme).ncMetrics.iCaptionHeight = GetThemeSysSize(hTheme, SM_CYSIZE);
    (*scheme).ncMetrics.iSmCaptionWidth = GetThemeSysSize(hTheme, SM_CXSMSIZE);
    (*scheme).ncMetrics.iSmCaptionHeight = GetThemeSysSize(hTheme, SM_CYSMSIZE);
    (*scheme).ncMetrics.iMenuWidth = GetThemeSysSize(hTheme, SM_CXMENUSIZE);
    (*scheme).ncMetrics.iMenuHeight = GetThemeSysSize(hTheme, SM_CYMENUSIZE);

    GetThemeSysFont(hTheme, TMT_CAPTIONFONT, &mut (*scheme).ncMetrics.lfCaptionFont);
    GetThemeSysFont(hTheme, TMT_SMALLCAPTIONFONT, &mut (*scheme).ncMetrics.lfSmCaptionFont);
    GetThemeSysFont(hTheme, TMT_MENUFONT, &mut (*scheme).ncMetrics.lfMenuFont);
    GetThemeSysFont(hTheme, TMT_STATUSFONT, &mut (*scheme).ncMetrics.lfStatusFont);
    GetThemeSysFont(hTheme, TMT_MSGBOXFONT, &mut (*scheme).ncMetrics.lfMessageFont);
    GetThemeSysFont(hTheme, TMT_ICONTITLEFONT, &mut (*scheme).icMetrics.lfFont);

    (*scheme).bFlatMenus = GetThemeSysBool(hTheme, TMT_FLATMENUS);

    CloseThemeData(hTheme);

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn DrawThemePreview(
    hdcMem: HDC,
    scheme: *mut COLOR_SCHEME,
    pSelectedTheme: *mut THEME_SELECTION,
    prcWindow: PRECT,
) -> i32 {
    let hbrBack: HBRUSH;
    let hres: HRESULT;

    hbrBack = CreateSolidBrush((*scheme).crColor[COLOR_DESKTOP]);

    FillRect(hdcMem, prcWindow, hbrBack);
    DeleteObject(hbrBack);

    InflateRect(prcWindow, -8, -8);
    (*prcWindow).bottom -= 12;

    hres = DrawNCPreview(
        hdcMem,
        DNCP_DRAW_ALL,
        prcWindow,
        (*(*pSelectedTheme).Theme).ThemeFileName,
        (*(*pSelectedTheme).Color).StyleName,
        (*(*pSelectedTheme).Size).StyleName,
        core::ptr::addr_of_mut!((*scheme).ncMetrics),
        (*scheme).crColor.as_mut_ptr(),
    );

    if hres >= 0 { 1 } else { 0 }
}
#[no_mangle]
pub unsafe extern "C" fn ActivateThemeFile(pwszFile: LPCWSTR) -> BOOL {
    let mut pThemes: PTHEME;
    let mut selection: THEME_SELECTION = core::mem::zeroed();
    let mut scheme: COLOR_SCHEME = core::mem::zeroed();
    let mut ret: BOOL = FALSE;

    pThemes = LoadThemes();
    if pThemes.is_null() {
        return FALSE;
    }

    'cleanup: {
        LoadCurrentScheme(&mut scheme);

        if !pwszFile.is_null() {
            ret = FindOrAppendTheme(pThemes, pwszFile, core::ptr::null(), core::ptr::null(), &mut selection);
            if ret == 0 {
                break 'cleanup;
            }

            ret = LoadSchemeFromTheme(&mut scheme, &mut selection);
            if ret == 0 {
                break 'cleanup;
            }
        } else {
            ret = GetActiveClassicTheme(pThemes, &mut selection);
            if ret == 0 {
                break 'cleanup;
            }

            ret = LoadSchemeFromReg(&mut scheme, &mut selection);
            if ret == 0 {
                break 'cleanup;
            }
        }

        ret = ActivateTheme(&mut selection);
        if ret == 0 {
            break 'cleanup;
        }

        ApplyScheme(&mut scheme, &mut selection);

        ret = TRUE;
    }

    CleanupThemes(pThemes);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn ThemesPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let lpnm: LPNMHDR;

    match uMsg {
        WM_INITDIALOG => {}

        WM_COMMAND => {}

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;
            match (*lpnm).code {
                PSN_APPLY => {
                    SendMessage(
                        HWND_BROADCAST,
                        WM_SETTINGCHANGE,
                        0,
                        TEXT(b"\0".as_ptr()) as LPARAM,
                    );
                    return TRUE;
                }
                _ => {}
            }
        }

        WM_DESTROY => {}

        _ => {}
    }

    FALSE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/theme.c
//  * PURPOSE:         Handling themes and visual effects
//  *
//  * PROGRAMMERS:     Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  *                  Ismael Ferreras Morezuelas (swyterzone+reactos@gmail.com)
//  */
//
// #include "desk.h"
//
// #include <shlwapi.h>
// #include <uxtheme.h>
// #include <uxundoc.h>
// #include <vssym32.h>
//
// static const WCHAR g_CPColors[] = L"Control Panel\\Colors";
// static const WCHAR g_CPANewSchemes[] = L"Control Panel\\Appearance\\New Schemes";
// static const WCHAR g_CPMetrics[] = L"Control Panel\\Desktop\\WindowMetrics";
// static const WCHAR g_SelectedStyle[] = L"SelectedStyle";
//
// /******************************************************************************/
//
// /* This is the list of names for the colors stored in the registry */
// static const WCHAR *g_RegColorNames[NUM_COLORS] = {
//     L"Scrollbar",             /* 00 = COLOR_SCROLLBAR */
//     L"Background",            /* 01 = COLOR_DESKTOP */
//     L"ActiveTitle",           /* 02 = COLOR_ACTIVECAPTION  */
//     L"InactiveTitle",         /* 03 = COLOR_INACTIVECAPTION */
//     L"Menu",                  /* 04 = COLOR_MENU */
//     L"Window",                /* 05 = COLOR_WINDOW */
//     L"WindowFrame",           /* 06 = COLOR_WINDOWFRAME */
//     L"MenuText",              /* 07 = COLOR_MENUTEXT */
//     L"WindowText",            /* 08 = COLOR_WINDOWTEXT */
//     L"TitleText",             /* 09 = COLOR_CAPTIONTEXT */
//     L"ActiveBorder",          /* 10 = COLOR_ACTIVEBORDER */
//     L"InactiveBorder",        /* 11 = COLOR_INACTIVEBORDER */
//     L"AppWorkSpace",          /* 12 = COLOR_APPWORKSPACE */
//     L"Hilight",               /* 13 = COLOR_HIGHLIGHT */
//     L"HilightText",           /* 14 = COLOR_HIGHLIGHTTEXT */
//     L"ButtonFace",            /* 15 = COLOR_BTNFACE */
//     L"ButtonShadow",          /* 16 = COLOR_BTNSHADOW */
//     L"GrayText",              /* 17 = COLOR_GRAYTEXT */
//     L"ButtonText",            /* 18 = COLOR_BTNTEXT */
//     L"InactiveTitleText",     /* 19 = COLOR_INACTIVECAPTIONTEXT */
//     L"ButtonHilight",         /* 20 = COLOR_BTNHIGHLIGHT */
//     L"ButtonDkShadow",        /* 21 = COLOR_3DDKSHADOW */
//     L"ButtonLight",           /* 22 = COLOR_3DLIGHT */
//     L"InfoText",              /* 23 = COLOR_INFOTEXT */
//     L"InfoWindow",            /* 24 = COLOR_INFOBK */
//     L"ButtonAlternateFace",   /* 25 = COLOR_ALTERNATEBTNFACE */
//     L"HotTrackingColor",      /* 26 = COLOR_HOTLIGHT */
//     L"GradientActiveTitle",   /* 27 = COLOR_GRADIENTACTIVECAPTION */
//     L"GradientInactiveTitle", /* 28 = COLOR_GRADIENTINACTIVECAPTION */
//     L"MenuHilight",           /* 29 = COLOR_MENUHILIGHT */
//     L"MenuBar",               /* 30 = COLOR_MENUBAR */
// };
//
// /******************************************************************************/
//
// VOID
// SchemeSetMetric( COLOR_SCHEME *scheme, int id, int value)
// {
//     switch (id)
//     {
//         case SIZE_BORDER_WIDTH: scheme->ncMetrics.iBorderWidth = value; break;
//         case SIZE_SCROLL_WIDTH: scheme->ncMetrics.iScrollWidth = value; break;
//         case SIZE_SCROLL_HEIGHT: scheme->ncMetrics.iScrollHeight = value; break;
//         case SIZE_CAPTION_WIDTH: scheme->ncMetrics.iCaptionWidth = value; break;
//         case SIZE_CAPTION_HEIGHT: scheme->ncMetrics.iCaptionHeight = value; break;
//         case SIZE_SM_CAPTION_WIDTH: scheme->ncMetrics.iSmCaptionWidth = value; break;
//         case SIZE_SM_CAPTION_HEIGHT: scheme->ncMetrics.iSmCaptionHeight = value; break;
//         case SIZE_MENU_WIDTH: scheme->ncMetrics.iMenuWidth = value; break;
//         case SIZE_MENU_HEIGHT: scheme->ncMetrics.iMenuHeight = value; break;
//         case SIZE_ICON: scheme->iIconSize = value; break;
//         case SIZE_ICON_SPACE_X: scheme->icMetrics.iHorzSpacing = value; break;
//         case SIZE_ICON_SPACE_Y: scheme->icMetrics.iVertSpacing = value; break;
//     }
// }
//
// int
// SchemeGetMetric( COLOR_SCHEME *scheme, int id)
// {
//     switch (id)
//     {
//         case SIZE_BORDER_WIDTH: return scheme->ncMetrics.iBorderWidth;
//         case SIZE_SCROLL_WIDTH: return scheme->ncMetrics.iScrollWidth;
//         case SIZE_SCROLL_HEIGHT: return scheme->ncMetrics.iScrollHeight;
//         case SIZE_CAPTION_WIDTH: return scheme->ncMetrics.iCaptionWidth;
//         case SIZE_CAPTION_HEIGHT: return scheme->ncMetrics.iCaptionHeight;
//         case SIZE_SM_CAPTION_WIDTH: return scheme->ncMetrics.iSmCaptionWidth;
//         case SIZE_SM_CAPTION_HEIGHT: return scheme->ncMetrics.iSmCaptionHeight;
//         case SIZE_MENU_WIDTH: return scheme->ncMetrics.iMenuWidth;
//         case SIZE_MENU_HEIGHT: return scheme->ncMetrics.iMenuHeight;
//         case SIZE_ICON: return scheme->iIconSize;
//         case SIZE_ICON_SPACE_X: return scheme->icMetrics.iHorzSpacing;
//         case SIZE_ICON_SPACE_Y: return scheme->icMetrics.iVertSpacing;
//     }
//     return 0;
// }
//
// PLOGFONTW
// SchemeGetFont( COLOR_SCHEME *scheme, int id)
// {
//     switch (id)
//     {
//         case FONT_CAPTION: return &scheme->ncMetrics.lfCaptionFont;
//         case FONT_SMCAPTION: return &scheme->ncMetrics.lfSmCaptionFont;
//         case FONT_MENU: return &scheme->ncMetrics.lfMenuFont;
//         case FONT_STATUS: return &scheme->ncMetrics.lfStatusFont;
//         case FONT_MESSAGE: return &scheme->ncMetrics.lfMessageFont;
//         case FONT_ICON: return &scheme->icMetrics.lfFont;
//     }
//     return NULL;
// }
//
// /*
//  * LoadCurrentScheme: Populates the passed scheme based on the current system settings
//  */
// BOOL
// LoadCurrentScheme( COLOR_SCHEME *scheme)
// {
//     INT i, Result;
//     HKEY hKey;
//     BOOL ret;
// #if (WINVER >= 0x0600)
//     OSVERSIONINFO osvi;
// #endif
//
//     /* Load colors */
//     for (i = 0; i < NUM_COLORS; i++)
//     {
//         scheme->crColor[i] = (COLORREF)GetSysColor(i);
//     }
//
//     /* Load non client metrics */
//     scheme->ncMetrics.cbSize = sizeof(NONCLIENTMETRICSW);
//
// #if (WINVER >= 0x0600)
//     /* Size of NONCLIENTMETRICSA/W depends on current version of the OS.
//      * see:
//      *  https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-nonclientmetricsa
//      */
//     if (GetVersionEx(&osvi))
//     {
//         /* Windows XP and earlier */
//         if (osvi.dwMajorVersion <= 5)
//             scheme->ncMetrics.cbSize -= sizeof(scheme->ncMetrics.iPaddedBorderWidth);
//     }
// #endif
//
//     ret = SystemParametersInfoW(SPI_GETNONCLIENTMETRICS,
//                                 sizeof(NONCLIENTMETRICSW),
//                                 &scheme->ncMetrics,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* Load icon metrics */
//     scheme->icMetrics.cbSize = sizeof(ICONMETRICSW);
//     ret = SystemParametersInfoW(SPI_GETICONMETRICS,
//                                 sizeof(ICONMETRICSW),
//                                 &scheme->icMetrics,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* Load flat menu style */
//     ret = SystemParametersInfoW(SPI_GETFLATMENU,
//                                 0,
//                                 &scheme->bFlatMenus,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* Effects */
//     /* Use the following transition effect for menus and tooltips */
//     ret = SystemParametersInfoW(SPI_GETMENUANIMATION,
//                                 0,
//                                 &scheme->Effects.bMenuAnimation,
//                                 0);
//     if (!ret) return FALSE;
//
//     ret = SystemParametersInfoW(SPI_GETMENUFADE,
//                                 0,
//                                 &scheme->Effects.bMenuFade,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* FIXME: XP seems to use grayed checkboxes to reflect differences between menu and tooltips settings
//      * Just keep them in sync for now:
//      */
//     scheme->Effects.bTooltipAnimation  = scheme->Effects.bMenuAnimation;
//     scheme->Effects.bTooltipFade       = scheme->Effects.bMenuFade;
//
//     /* Use the following transition effect for menus and tooltips */
//     ret = SystemParametersInfoW(SPI_GETFONTSMOOTHING,
//                                 0,
//                                 &scheme->Effects.bFontSmoothing,
//                                 0);
//     if (!ret) return FALSE;
//
//     ret = SystemParametersInfoW(SPI_GETFONTSMOOTHINGTYPE,
//                                 0,
//                                 &scheme->Effects.uiFontSmoothingType,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* Show shadows under windows */
//     ret = SystemParametersInfoW(SPI_GETDROPSHADOW,
//                                 0,
//                                 &scheme->Effects.bDropShadow,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* Show content of windows during dragging */
//     ret = SystemParametersInfoW(SPI_GETDRAGFULLWINDOWS,
//                                 0,
//                                 &scheme->Effects.bDragFullWindows,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* Hide underlined letters for keyboard navigation until the Alt key is pressed */
//     ret = SystemParametersInfoW(SPI_GETKEYBOARDCUES,
//                                 0,
//                                 &scheme->Effects.bKeyboardCues,
//                                 0);
//     if (!ret) return FALSE;
//
//     /* Read the icon size from registry */
//     Result = RegOpenKeyW(HKEY_CURRENT_USER, g_CPMetrics, &hKey);
//     if(Result == ERROR_SUCCESS)
//     {
//         scheme->iIconSize = SHRegGetIntW(hKey, L"Shell Icon Size", 32);
//         RegCloseKey(hKey);
//     }
//
//     return TRUE;
// }
//
// /*
//  * LoadSchemeFromReg: Populates the passed scheme with values retrieved from registry
//  */
// BOOL
// LoadSchemeFromReg( COLOR_SCHEME *scheme,  PTHEME_SELECTION pSelectedTheme)
// {
//     INT i;
//     WCHAR strValueName[10], strSchemeKey[MAX_PATH];
//     HKEY hkScheme = NULL;
//     DWORD dwType, dwLength;
//     UINT64 iSize;
//     BOOL Ret = TRUE;
//     LONG result;
//
//     wsprintf(strSchemeKey, L"%s\\%s\\Sizes\\%s",
//              g_CPANewSchemes,
//              pSelectedTheme->Color->StyleName,
//              pSelectedTheme->Size->StyleName);
//
//     result = RegOpenKeyW(HKEY_CURRENT_USER, strSchemeKey, &hkScheme);
//     if (result != ERROR_SUCCESS) return FALSE;
//
//     scheme->bFlatMenus = SHRegGetIntW(hkScheme, L"FlatMenus", 0);
//
//     for (i = 0; i < NUM_COLORS; i++)
//     {
//         wsprintf(strValueName, L"Color #%d", i);
//         dwLength = sizeof(COLORREF);
//         result = RegQueryValueExW(hkScheme,
//                                   strValueName,
//                                   NULL,
//                                   &dwType,
//                                   (LPBYTE)&scheme->crColor[i],
//                                   &dwLength);
//         if (result != ERROR_SUCCESS || dwType != REG_DWORD)
//         {
//             /* Failed to read registry value, initialize with current setting for now */
//             scheme->crColor[i] = GetSysColor(i);
//         }
//     }
//
//     for (i = 0; i < NUM_FONTS; i++)
//     {
//         PLOGFONTW lpfFont = SchemeGetFont(scheme, i);
//
//         wsprintf(strValueName, L"Font #%d", i);
//         dwLength = sizeof(LOGFONT);
//         result = RegQueryValueExW(hkScheme,
//                                   strValueName,
//                                   NULL,
//                                   &dwType,
//                                   (LPBYTE)lpfFont,
//                                   &dwLength);
//         if (result != ERROR_SUCCESS || dwType != REG_BINARY ||
//             dwLength != sizeof(LOGFONT))
//         {
//             /* Failed to read registry value */
//             Ret = FALSE;
//         }
//     }
//
//     for (i = 0; i < NUM_SIZES; i++)
//     {
//         wsprintf(strValueName, L"Size #%d", i);
//         dwLength = sizeof(UINT64);
//         result = RegQueryValueExW(hkScheme,
//                                   strValueName,
//                                   NULL,
//                                   &dwType,
//                                   (LPBYTE)&iSize,
//                                   &dwLength);
//         if (result != ERROR_SUCCESS || dwType != REG_QWORD ||
//             dwLength != sizeof(UINT64))
//         {
//             /* Failed to read registry value, initialize with current setting for now */
//         }
//         else
//         {
//             SchemeSetMetric(scheme, i, (int)iSize);
//         }
//     }
//
//     RegCloseKey(hkScheme);
//
//     return Ret;
// }
//
// /*
//  * ApplyScheme: Applies the selected scheme and stores its id in the registry if needed
//  */
// VOID
// ApplyScheme( COLOR_SCHEME *scheme,  PTHEME_SELECTION pSelectedTheme)
// {
//     INT i, Result;
//     HKEY hKey;
//     WCHAR clText[16], *StyleName;
//     INT ColorList[NUM_COLORS];
//
//     /* Apply system colors  */
//     for (i = 0; i < NUM_COLORS; i++)
//         ColorList[i] = i;
//     SetSysColors(NUM_COLORS, ColorList, scheme->crColor);
//
//     /* Save colors to registry */
//     Result = RegCreateKeyW(HKEY_CURRENT_USER, g_CPColors, &hKey);
//     if (Result == ERROR_SUCCESS)
//     {
//         for (i = 0; i < NUM_COLORS; i++)
//         {
//             wsprintf(clText,
//                      L"%d %d %d",
//                      GetRValue(scheme->crColor[i]),
//                      GetGValue(scheme->crColor[i]),
//                      GetBValue(scheme->crColor[i]));
//
//             RegSetValueExW(hKey,
//                            g_RegColorNames[i],
//                            0,
//                            REG_SZ,
//                            (BYTE *)clText,
//                            (lstrlen(clText) + 1) * sizeof(WCHAR));
//         }
//         RegCloseKey(hKey);
//     }
//
//     /* Apply non client metrics */
//     SystemParametersInfoW(SPI_SETNONCLIENTMETRICS,
//                           sizeof(NONCLIENTMETRICS),
//                           &scheme->ncMetrics,
//                           SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);
//
//     /* Apply icon metrics */
//     SystemParametersInfoW(SPI_SETICONMETRICS,
//                           sizeof(ICONMETRICS),
//                           &scheme->icMetrics,
//                           SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);
//
//     /* Effects, save only when needed: */
//     /* FIXME: XP seems to use grayed checkboxes to reflect differences between menu and tooltips settings
//      * Just keep them in sync for now.
//      */
//
// #define SYS_CONFIG(__uiAction, __uiParam, __pvParam) \
//     SystemParametersInfoW(__uiAction, __uiParam, __pvParam, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE)
//
//     scheme->Effects.bTooltipAnimation  = scheme->Effects.bMenuAnimation;
//     scheme->Effects.bTooltipFade       = scheme->Effects.bMenuFade;
//
//     /* Use the following transition effect for menus and tooltips */
//     SYS_CONFIG(SPI_SETMENUANIMATION,             0, IntToPtr(scheme->Effects.bMenuAnimation));
//     SYS_CONFIG(SPI_SETMENUFADE,                  0, IntToPtr(scheme->Effects.bMenuFade));
//
//     /* Use the following method to smooth edges of screen fonts */
//     SYS_CONFIG(SPI_SETFONTSMOOTHING,             scheme->Effects.bFontSmoothing, 0);
//     SYS_CONFIG(SPI_SETFONTSMOOTHINGTYPE,         0, IntToPtr(scheme->Effects.uiFontSmoothingType));
//
//     /*
//      * Refresh and redraw all the windows, otherwise the font smoothing changes
//      * only appear after any future partial region invalidation.
//      * Not everyone listens for this WM_SETTINGCHANGE, including the shell and most third party programs.
//      */
//     InvalidateRect(NULL, NULL, TRUE);
//
//     /* Use large icons */
//     //SYS_CONFIG(SPI_GETDRAGFULLWINDOWS,   (PVOID) g->SchemeAdv.Effects.bMenuFade);
//
//     /* Show shadows under windows */
//     SYS_CONFIG(SPI_SETDROPSHADOW,                0, IntToPtr(scheme->Effects.bDropShadow));
//
//     /* Show window contents while dragging */
//     SYS_CONFIG(SPI_SETDRAGFULLWINDOWS,           scheme->Effects.bDragFullWindows, 0);
//
//     /* Hide underlined letters for keyboard navigation until I press the Alt key */
//     SYS_CONFIG(SPI_SETKEYBOARDCUES,              0, IntToPtr(scheme->Effects.bKeyboardCues));
//
//     SYS_CONFIG(SPI_SETFLATMENU,                  0, IntToPtr(scheme->bFlatMenus));
//
//     // SYS_CONFIG(SPI_SETACTIVEWINDOWTRACKING,   0, IntToPtr(scheme->Effects.bActiveWindowTracking));
//     // SYS_CONFIG(SPI_SETCOMBOBOXANIMATION,      0, IntToPtr(scheme->Effects.bComboBoxAnimation));
//     // SYS_CONFIG(SPI_SETLISTBOXSMOOTHSCROLLING, 0, IntToPtr(scheme->Effects.bListBoxSmoothScrolling));
//     // SYS_CONFIG(SPI_SETGRADIENTCAPTIONS,       0, IntToPtr(scheme->Effects.bGradientCaptions));
//     // SYS_CONFIG(SPI_SETACTIVEWNDTRKZORDER,     0, IntToPtr(scheme->Effects.bActiveWndTrkZorder));
//     // SYS_CONFIG(SPI_SETHOTTRACKING,            0, IntToPtr(scheme->Effects.bHotTracking));
//     // SYS_CONFIG(SPI_SETSELECTIONFADE,          0, IntToPtr(scheme->Effects.bSelectionFade));
//     SYS_CONFIG(SPI_SETTOOLTIPANIMATION,          0, IntToPtr(scheme->Effects.bTooltipAnimation));
//     SYS_CONFIG(SPI_SETTOOLTIPFADE,               0, IntToPtr(scheme->Effects.bTooltipFade));
//     // SYS_CONFIG(SPI_SETCURSORSHADOW,           0, IntToPtr(scheme->Effects.bCursorShadow));
//     // SYS_CONFIG(SPI_SETUIEFFECTS,              0, IntToPtr(scheme->Effects.bUiEffects));
//
// #undef SYS_CONFIG
//
//     /* Save SchemeId in the registry */
//     if (pSelectedTheme->Theme != NULL && pSelectedTheme->ThemeActive == FALSE)
//     {
//         StyleName = pSelectedTheme->Color->StyleName;
//         SHSetValueW(HKEY_CURRENT_USER,
//                     g_CPANewSchemes,
//                     g_SelectedStyle,
//                     REG_SZ,
//                     StyleName,
//                     (lstrlenW(StyleName) + 1) * sizeof(WCHAR));
//     }
//
//     if (pSelectedTheme->ThemeActive)
//     {
//         SHSetValueW(HKEY_CURRENT_USER, L"Control Panel\\Appearance", L"Current", REG_SZ, NULL, 0);
//         SHSetValueW(HKEY_CURRENT_USER, L"Control Panel\\Appearance", L"NewCurrent", REG_SZ, NULL, 0);
//     }
//     else if (pSelectedTheme->Color)
//     {
//         PCWSTR ClassicSchemeName = pSelectedTheme->Color->DisplayName;
//         DWORD cb = (lstrlenW(ClassicSchemeName) + 1) * sizeof(WCHAR);
//         SHSetValueW(HKEY_CURRENT_USER, L"Control Panel\\Appearance", L"Current", REG_SZ, ClassicSchemeName, cb); // 95+
//         SHSetValueW(HKEY_CURRENT_USER, L"Control Panel\\Appearance", L"NewCurrent", REG_SZ, ClassicSchemeName, cb); // XP+
//     }
// }
//
// static THEME*
// CreateTheme(LPCWSTR pszName, LPCWSTR pszDisplayName)
// {
//     PTHEME pTheme;
//
//     pTheme = (PTHEME) malloc(sizeof(THEME));
//     if (pTheme == NULL) return NULL;
//
//     pTheme->DisplayName = _wcsdup(pszDisplayName);
//     if (pTheme->DisplayName == NULL)
//     {
//         free(pTheme);
//         return NULL;
//     }
//
//     pTheme->ColoursList = NULL;
//     pTheme->NextTheme = NULL;
//     pTheme->SizesList = NULL;
//
//     if (pszName == NULL)
//     {
//         pTheme->ThemeFileName = NULL;
//         return pTheme;
//     }
//
//     pTheme->ThemeFileName = _wcsdup(pszName);
//     if (pTheme->ThemeFileName == NULL)
//     {
//         free(pTheme->DisplayName);
//         free(pTheme);
//         return NULL;
//     }
//
//     return pTheme;
// }
//
// static PTHEME_STYLE
// CreateStyle(LPCWSTR pszName, LPCWSTR pszDisplayName)
// {
//     PTHEME_STYLE pStyle;
//
//     pStyle = (PTHEME_STYLE) malloc(sizeof(THEME_STYLE));
//     if (pStyle == NULL) return NULL;
//
//     pStyle->StyleName = _wcsdup(pszName);
//     if (pStyle->StyleName == NULL)
//     {
//         free(pStyle);
//         return NULL;
//     }
//
//     pStyle->DisplayName = _wcsdup(pszDisplayName);
//     if (pStyle->DisplayName == NULL)
//     {
//         free(pStyle->StyleName);
//         free(pStyle);
//         return NULL;
//     }
//
//     pStyle->ChildStyle = NULL;
//     pStyle->NextStyle = NULL;
//
//     return pStyle;
// }
//
// static void
// CleanupStyles( PTHEME_STYLE pStylesList)
// {
//     PTHEME_STYLE pStyle, pStyleOld;
//
//     pStyle = pStylesList;
//     while (pStyle)
//     {
//         if (pStyle->ChildStyle) CleanupStyles(pStyle->ChildStyle);
//         if (pStyle->DisplayName) free(pStyle->DisplayName);
//         if (pStyle->StyleName) free(pStyle->StyleName);
//
//         pStyleOld = pStyle;
//         pStyle = pStyle->NextStyle;
//         free(pStyleOld);
//     }
// }
//
// void
// CleanupThemes( PTHEME pThemeList)
// {
//     PTHEME pTheme, pThemeOld;
//
//     pTheme = pThemeList;
//     while (pTheme)
//     {
//         CleanupStyles(pTheme->ColoursList);
//         if (pTheme->SizesList) CleanupStyles(pTheme->SizesList);
//         if (pTheme->DisplayName) free(pTheme->DisplayName);
//         if (pTheme->ThemeFileName) free(pTheme->ThemeFileName);
//
//         pThemeOld = pTheme;
//         pTheme = pTheme->NextTheme;
//         free(pThemeOld);
//     }
// }
//
// static PTHEME_STYLE
// FindStyle( PTHEME_STYLE pStylesList,  PCWSTR StyleName)
// {
//     PTHEME_STYLE pStyle;
//
//     for (pStyle = pStylesList; pStyle; pStyle = pStyle->NextStyle)
//     {
//         if (_wcsicmp(pStyle->StyleName, StyleName) == 0)
//         {
//             return pStyle;
//         }
//     }
//
//     /* If we can't find the style requested, return the first one */
//     return pStylesList;
// }
//
// /*
//  * LoadSchemeSizes: Returns a list of sizes from the registry key of a scheme
//  */
// static PTHEME_STYLE
// LoadSchemeSizes( HKEY hkScheme)
// {
//     HKEY hkSizes, hkSize;
//     INT Result;
//     INT iStyle;
//     WCHAR wstrSizeName[5], wstrDisplayName[50];
//     THEME_STYLE *List = NULL, *pCurrentStyle;
//
//     Result = RegOpenKeyW(hkScheme, L"Sizes",  &hkSizes);
//     if (Result != ERROR_SUCCESS) return NULL;
//
//     iStyle = 0;
//     while ((RegEnumKeyW(hkSizes, iStyle, wstrSizeName, 5) == ERROR_SUCCESS))
//     {
//         iStyle++;
//
//         Result = RegOpenKeyW(hkSizes, wstrSizeName, &hkSize);
//         if (Result != ERROR_SUCCESS) continue;
//
//         Result = RegLoadMUIStringW(hkSize,
//                                    L"DisplayName",
//                                    wstrDisplayName,
//                                    sizeof(wstrDisplayName),
//                                    NULL,
//                                    0,
//                                    NULL);
//         if (Result != ERROR_SUCCESS)
//         {
//             Result = RegLoadMUIStringW(hkSize,
//                                        L"LegacyName",
//                                        wstrDisplayName,
//                                        sizeof(wstrDisplayName),
//                                        NULL,
//                                        0,
//                                        NULL);
//         }
//
//         if (Result == ERROR_SUCCESS)
//             pCurrentStyle = CreateStyle(wstrSizeName, wstrDisplayName);
//         else
//             pCurrentStyle = CreateStyle(wstrSizeName, wstrSizeName);
//
//         if (pCurrentStyle  != NULL)
//         {
//             pCurrentStyle->NextStyle = List;
//             List = pCurrentStyle;
//         }
//
//         RegCloseKey(hkSize);
//     }
//
//     RegCloseKey(hkSizes);
//     return List;
// }
//
// /*
//  * LoadClassicColorSchemes: Returns a list of classic theme colours from the registry key of a scheme
//  */
// static THEME_STYLE*
// LoadClassicColorSchemes(VOID)
// {
//     INT Result;
//     HKEY hkNewSchemes, hkScheme;
//     INT iStyle;
//     WCHAR wstrStyleName[5], wstrDisplayName[50];
//     THEME_STYLE *List = NULL, *pCurrentStyle;
//
//     Result = RegOpenKeyW(HKEY_CURRENT_USER, g_CPANewSchemes,  &hkNewSchemes);
//     if (Result != ERROR_SUCCESS) return NULL;
//
//     iStyle = 0;
//     while ((RegEnumKeyW(hkNewSchemes, iStyle, wstrStyleName, 5) == ERROR_SUCCESS))
//     {
//         iStyle++;
//
//         Result = RegOpenKeyW(hkNewSchemes, wstrStyleName,  &hkScheme);
//         if (Result != ERROR_SUCCESS) continue;
//
//         Result = RegLoadMUIStringW(hkScheme,
//                                    L"DisplayName",
//                                    wstrDisplayName,
//                                    sizeof(wstrDisplayName),
//                                    NULL,
//                                    0,
//                                    NULL);
//         if (Result != ERROR_SUCCESS)
//         {
//             Result = RegLoadMUIStringW(hkScheme,
//                                        L"LegacyName",
//                                        wstrDisplayName,
//                                        sizeof(wstrDisplayName),
//                                        NULL,
//                                        0,
//                                        NULL);
//         }
//
//         if (Result == ERROR_SUCCESS)
//             pCurrentStyle = CreateStyle(wstrStyleName, wstrDisplayName);
//         else
//             pCurrentStyle = CreateStyle(wstrStyleName, wstrStyleName);
//
//         if (pCurrentStyle != NULL)
//         {
//             pCurrentStyle->NextStyle = List;
//             pCurrentStyle->ChildStyle = LoadSchemeSizes(hkScheme);
//             if(pCurrentStyle->ChildStyle == NULL)
//                 CleanupStyles(pCurrentStyle);
//             else
//                 List = pCurrentStyle;
//         }
//
//         RegCloseKey(hkScheme);
//     }
//
//     RegCloseKey(hkNewSchemes);
//     return List;
// }
//
// typedef HRESULT (WINAPI *ENUMTHEMESTYLE) (LPCWSTR, LPWSTR, DWORD, PTHEMENAMES);
//
// static THEME_STYLE*
// EnumThemeStyles( LPCWSTR pszThemeFileName,  ENUMTHEMESTYLE pfnEnumTheme)
// {
//     DWORD index = 0;
//     THEMENAMES names;
//     THEME_STYLE *List = NULL, **ppPrevStyle, *pCurrentStyle;
//
//     ppPrevStyle = &List;
//
//     while (SUCCEEDED(pfnEnumTheme (pszThemeFileName, NULL, index++, &names)))
//     {
//         pCurrentStyle = CreateStyle(names.szName, names.szDisplayName);
//         if(pCurrentStyle == NULL) break;
//
//         *ppPrevStyle = pCurrentStyle;
//         ppPrevStyle = &pCurrentStyle->NextStyle;
//     }
//
//     return List;
// }
//
// PTHEME LoadTheme( LPCWSTR pszThemeFileName, LPCWSTR pszThemeName)
// {
//     PTHEME pTheme = CreateTheme(pszThemeFileName, pszThemeName);
//     if (pTheme == NULL)
//         return NULL;
//
//     pTheme->SizesList = EnumThemeStyles( pszThemeFileName, (ENUMTHEMESTYLE)EnumThemeSizes);
//     pTheme->ColoursList = EnumThemeStyles( pszThemeFileName, (ENUMTHEMESTYLE)EnumThemeColors);
//     if(pTheme->SizesList == NULL || pTheme->ColoursList == NULL)
//     {
//         CleanupThemes(pTheme);
//         return NULL;
//     }
//
//     return pTheme;
// }
//
// BOOL __stdcall
// EnumThemeProc( LPVOID lpReserved,
//                LPCWSTR pszThemeFileName,
//                LPCWSTR pszThemeName,
//                LPCWSTR pszToolTip,
//                LPVOID lpReserved2,
//                 LPVOID lpData)
// {
//     PTHEME *List, pTheme;
//
//     List = (PTHEME*)lpData;
//     pTheme = LoadTheme(pszThemeFileName, pszThemeName);
//     if (pTheme == NULL) return FALSE;
//
//     pTheme->NextTheme = *List;
//     *List = pTheme;
//
//     return TRUE;
// }
//
// /*
//  * LoadThemes: Returns a list that contains tha classic theme and
//  *             the visual styles of the system
//  */
// PTHEME
// LoadThemes(VOID)
// {
//     HRESULT hret;
//     PTHEME pClassicTheme;
//     WCHAR strClassicTheme[40];
//     WCHAR szThemesPath[MAX_PATH], *pszClassicTheme;
//     int res;
//
//     /* Insert the classic theme */
//     res = LoadString(hApplet, IDS_CLASSIC_THEME, strClassicTheme, 40);
//     pszClassicTheme = (res > 0 ? strClassicTheme : L"Classic Theme");
//     pClassicTheme = CreateTheme(NULL, pszClassicTheme);
//     if (pClassicTheme == NULL) return NULL;
//     pClassicTheme->ColoursList = LoadClassicColorSchemes();
//
//     /* Get path to themes folder */
//     ZeroMemory(szThemesPath, sizeof(szThemesPath));
//     hret = SHGetFolderPathW (NULL, CSIDL_RESOURCES, NULL, SHGFP_TYPE_DEFAULT, szThemesPath);
//     if (FAILED(hret)) return pClassicTheme;
//     lstrcatW (szThemesPath, L"\\Themes");
//
//     /* Enumerate themes */
//     hret = EnumThemes( szThemesPath, EnumThemeProc, &pClassicTheme->NextTheme);
//     if (FAILED(hret))
//     {
//         pClassicTheme->NextTheme = NULL;
//         if (pClassicTheme->ColoursList == NULL)
//         {
//             free(pClassicTheme->DisplayName);
//             free(pClassicTheme);
//             return NULL;
//         }
//     }
//
//     return pClassicTheme;
// }
//
// /*
//  * FindSelectedTheme: Finds the specified theme in the list of themes
//  *                    or loads it if it was not loaded already.
//  */
// BOOL
// FindOrAppendTheme( PTHEME pThemeList,
//                    LPCWSTR pwszThemeFileName,
//                    LPCWSTR pwszColorBuff,
//                    LPCWSTR pwszSizeBuff,
//                    PTHEME_SELECTION pSelectedTheme)
// {
//     PTHEME pTheme;
//     PTHEME pFoundTheme = NULL;
//
//     ZeroMemory(pSelectedTheme, sizeof(THEME_SELECTION));
//
//     for (pTheme = pThemeList; pTheme; pTheme = pTheme->NextTheme)
//     {
//         if (pTheme->ThemeFileName &&
//            _wcsicmp(pTheme->ThemeFileName, pwszThemeFileName) == 0)
//         {
//             pFoundTheme = pTheme;
//             break;
//         }
//
//         if (pTheme->NextTheme == NULL)
//             break;
//     }
//
//     if (!pFoundTheme)
//     {
//         pFoundTheme = LoadTheme(pwszThemeFileName, pwszThemeFileName);
//         if (!pFoundTheme)
//             return FALSE;
//
//         pTheme->NextTheme = pFoundTheme;
//     }
//
//     pSelectedTheme->ThemeActive = TRUE;
//     pSelectedTheme->Theme = pFoundTheme;
//     if (pwszColorBuff)
//         pSelectedTheme->Color = FindStyle(pFoundTheme->ColoursList, pwszColorBuff);
//     else
//         pSelectedTheme->Color = pFoundTheme->ColoursList;
//
//     if (pwszSizeBuff)
//         pSelectedTheme->Size = FindStyle(pFoundTheme->SizesList, pwszSizeBuff);
//     else
//         pSelectedTheme->Size = pFoundTheme->SizesList;
//
//     return TRUE;
// }
//
// /*
//  * GetActiveTheme: Gets the active theme and populates pSelectedTheme
//  *                 with entries from the list of loaded themes.
//  */
// BOOL
// GetActiveTheme( PTHEME pThemeList,  PTHEME_SELECTION pSelectedTheme)
// {
//     WCHAR szThemeFileName[MAX_PATH];
//     WCHAR szColorBuff[MAX_PATH];
//     WCHAR szSizeBuff[MAX_PATH];
//     HRESULT hret;
//
//     /* Retrieve the name of the current theme */
//     hret = GetCurrentThemeName(szThemeFileName,
//                                MAX_PATH,
//                                szColorBuff,
//                                MAX_PATH,
//                                szSizeBuff,
//                                MAX_PATH);
//     if (FAILED(hret))
//         return FALSE;
//
//     return FindOrAppendTheme(pThemeList, szThemeFileName, szColorBuff, szSizeBuff, pSelectedTheme);
// }
//
// /*
//  * GetActiveTheme: Gets the active classic theme and populates pSelectedTheme
//  *                 with entries from the list of loaded themes
//  */
// BOOL
// GetActiveClassicTheme( PTHEME pThemeList,  PTHEME_SELECTION pSelectedTheme)
// {
//     INT Result;
//     WCHAR szSelectedClassicScheme[5], szSelectedClassicSize[5];
//     HKEY hkNewSchemes;
//     DWORD dwType, dwDisplayNameLength;
//     PTHEME_STYLE pCurrentStyle, pCurrentSize;
//
//     ZeroMemory(pSelectedTheme, sizeof(THEME_SELECTION));
//
//     /* Assume failure */
//     szSelectedClassicScheme[0] = 0;
//     szSelectedClassicSize[0] = 0;
//
//     Result = RegOpenKeyW(HKEY_CURRENT_USER, g_CPANewSchemes, &hkNewSchemes);
//     if (Result != ERROR_SUCCESS) return FALSE;
//
//     dwType = REG_SZ;
//     dwDisplayNameLength = sizeof(szSelectedClassicScheme);
//     Result = RegQueryValueEx(hkNewSchemes, L"SelectedStyle", NULL, &dwType,
//                              (LPBYTE)&szSelectedClassicScheme, &dwDisplayNameLength);
//     if (Result == ERROR_SUCCESS)
//     {
//         dwType = REG_SZ;
//         dwDisplayNameLength = sizeof(szSelectedClassicSize);
//         Result = SHGetValue(hkNewSchemes, szSelectedClassicScheme, L"SelectedSize",
//                             &dwType, szSelectedClassicSize, &dwDisplayNameLength);
//     }
//
//     RegCloseKey(hkNewSchemes);
//
//     pCurrentStyle = FindStyle(pThemeList->ColoursList, szSelectedClassicScheme);
//     pCurrentSize = FindStyle(pCurrentStyle->ChildStyle, szSelectedClassicSize);
//
//     pSelectedTheme->Theme = pThemeList;
//     pSelectedTheme->Color = pCurrentStyle;
//     pSelectedTheme->Size = pCurrentSize;
//
//     return TRUE;
// }
//
// BOOL
// ActivateTheme( PTHEME_SELECTION pSelectedTheme)
// {
//     HTHEMEFILE hThemeFile = 0;
//     HRESULT hret;
//
//     if (pSelectedTheme->ThemeActive)
//     {
//         hret = OpenThemeFile(pSelectedTheme->Theme->ThemeFileName,
//                              pSelectedTheme->Color->StyleName,
//                              pSelectedTheme->Size->StyleName,
//                              &hThemeFile,
//                              0);
//
//         if (!SUCCEEDED(hret)) return FALSE;
//     }
//
//     hret = ApplyTheme(hThemeFile, 0, 0);
//
//     if (pSelectedTheme->ThemeActive)
//     {
//        CloseThemeFile(hThemeFile);
//     }
//
//     return SUCCEEDED(hret);
// }
//
// BOOL
// LoadSchemeFromTheme( PCOLOR_SCHEME scheme,  PTHEME_SELECTION pSelectedTheme)
// {
//     HTHEMEFILE hThemeFile = 0;
//     HRESULT hret;
//     HTHEME hTheme;
//     int i;
//
//     hret = OpenThemeFile(pSelectedTheme->Theme->ThemeFileName,
//                          pSelectedTheme->Color->StyleName,
//                          pSelectedTheme->Size->StyleName,
//                          &hThemeFile,
//                          0);
//
//     if (!SUCCEEDED(hret)) return FALSE;
//
//     hTheme = OpenThemeDataFromFile(hThemeFile, hCPLWindow, L"WINDOW", 0);
//     if (hTheme == NULL) return FALSE;
//
//     /* Load colors */
//     for (i = 0; i < NUM_COLORS; i++)
//     {
//         scheme->crColor[i] = GetThemeSysColor(hTheme,i);
//     }
//
//     /* Load sizes */
//     /* I wonder why GetThemeSysInt doesn't work here */
//     scheme->ncMetrics.iBorderWidth = GetThemeSysSize(hTheme, SM_CXFRAME);
//     scheme->ncMetrics.iScrollWidth = GetThemeSysSize(hTheme, SM_CXVSCROLL);
//     scheme->ncMetrics.iScrollHeight = GetThemeSysSize(hTheme, SM_CYHSCROLL);
//     scheme->ncMetrics.iCaptionWidth = GetThemeSysSize(hTheme, SM_CXSIZE);
//     scheme->ncMetrics.iCaptionHeight = GetThemeSysSize(hTheme, SM_CYSIZE);
//     scheme->ncMetrics.iSmCaptionWidth = GetThemeSysSize(hTheme, SM_CXSMSIZE);
//     scheme->ncMetrics.iSmCaptionHeight = GetThemeSysSize(hTheme, SM_CYSMSIZE);
//     scheme->ncMetrics.iMenuWidth = GetThemeSysSize(hTheme, SM_CXMENUSIZE);
//     scheme->ncMetrics.iMenuHeight = GetThemeSysSize(hTheme, SM_CYMENUSIZE);
//
//     /* Load fonts */
//     GetThemeSysFont(hTheme, TMT_CAPTIONFONT, &scheme->ncMetrics.lfCaptionFont);
//     GetThemeSysFont(hTheme, TMT_SMALLCAPTIONFONT, &scheme->ncMetrics.lfSmCaptionFont);
//     GetThemeSysFont(hTheme, TMT_MENUFONT, &scheme->ncMetrics.lfMenuFont );
//     GetThemeSysFont(hTheme, TMT_STATUSFONT, &scheme->ncMetrics.lfStatusFont);
//     GetThemeSysFont(hTheme, TMT_MSGBOXFONT, &scheme->ncMetrics.lfMessageFont);
//     GetThemeSysFont(hTheme, TMT_ICONTITLEFONT, &scheme->icMetrics.lfFont);
//
//     scheme->bFlatMenus = GetThemeSysBool(hTheme, TMT_FLATMENUS);
//
//     CloseThemeData(hTheme);
//
//     return TRUE;
// }
//
// BOOL
// DrawThemePreview( HDC hdcMem,  PCOLOR_SCHEME scheme,  PTHEME_SELECTION pSelectedTheme,  PRECT prcWindow)
// {
//     HBRUSH hbrBack;
//     HRESULT hres;
//
//     hbrBack = CreateSolidBrush(scheme->crColor[COLOR_DESKTOP]);
//
//     FillRect(hdcMem, prcWindow, hbrBack);
//     DeleteObject(hbrBack);
//
//     InflateRect(prcWindow, -8, -8);
//     prcWindow->bottom -= 12;
//
//     hres = DrawNCPreview(hdcMem,
//                          DNCP_DRAW_ALL,
//                          prcWindow,
//                          pSelectedTheme->Theme->ThemeFileName,
//                          pSelectedTheme->Color->StyleName,
//                          pSelectedTheme->Size->StyleName,
//                          &scheme->ncMetrics,
//                          scheme->crColor);
//
//     return SUCCEEDED(hres);
// }
//
// BOOL ActivateThemeFile(LPCWSTR pwszFile)
// {
//     PTHEME pThemes;
//     THEME_SELECTION selection;
//     COLOR_SCHEME scheme;
//     BOOL ret = FALSE;
//
//     pThemes = LoadThemes();
//     if (!pThemes)
//         return FALSE;
//
//     LoadCurrentScheme(&scheme);
//
//     if (pwszFile)
//     {
//         ret = FindOrAppendTheme(pThemes, pwszFile, NULL, NULL, &selection);
//         if (!ret)
//             goto cleanup;
//
//         ret = LoadSchemeFromTheme(&scheme, &selection);
//         if (!ret)
//             goto cleanup;
//     }
//     else
//     {
//         ret = GetActiveClassicTheme(pThemes, &selection);
//         if (!ret)
//             goto cleanup;
//
//         ret = LoadSchemeFromReg(&scheme, &selection);
//         if (!ret)
//             goto cleanup;
//     }
//
//     ret = ActivateTheme(&selection);
//     if (!ret)
//         goto cleanup;
//
//     ApplyScheme(&scheme, &selection);
//
//     ret = TRUE;
//
// cleanup:
//     CleanupThemes(pThemes);
//
//     return ret;
// }
//
// /* TODO: */
// INT_PTR __stdcall
// ThemesPageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     LPNMHDR lpnm;
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             break;
//
//         case WM_COMMAND:
//             break;
//
//         case WM_NOTIFY:
//             lpnm = (LPNMHDR)lParam;
//             switch (lpnm->code)
//             {
//                 case PSN_APPLY:
//                     SendMessage(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)TEXT(""));
//                     return TRUE;
//             }
//             break;
//
//         case WM_DESTROY:
//             break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

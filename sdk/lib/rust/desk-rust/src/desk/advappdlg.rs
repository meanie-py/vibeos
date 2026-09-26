// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ASSIGNMENT {
    pub Size: i32,
    pub Size2: i32,
    pub Color1: i32,
    pub Color2: i32,
    pub Font: i32,
    pub FontColor: i32,
}

pub type PASSIGNMENT = *mut ASSIGNMENT;

pub static g_Assignment: [ASSIGNMENT; 18] = [
    ASSIGNMENT { Size: -1, Size2: -1, Color1: COLOR_DESKTOP as i32, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: SIZE_CAPTION_HEIGHT as i32, Size2: -1, Color1: COLOR_INACTIVECAPTION as i32, Color2: COLOR_GRADIENTINACTIVECAPTION as i32, Font: FONT_CAPTION as i32, FontColor: COLOR_INACTIVECAPTIONTEXT as i32 },
    ASSIGNMENT { Size: SIZE_BORDER_WIDTH as i32, Size2: -1, Color1: COLOR_INACTIVEBORDER as i32, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: SIZE_CAPTION_HEIGHT as i32, Size2: -1, Color1: COLOR_ACTIVECAPTION as i32, Color2: COLOR_GRADIENTACTIVECAPTION as i32, Font: FONT_CAPTION as i32, FontColor: COLOR_CAPTIONTEXT as i32 },
    ASSIGNMENT { Size: SIZE_BORDER_WIDTH as i32, Size2: -1, Color1: COLOR_ACTIVEBORDER as i32, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: SIZE_MENU_HEIGHT as i32, Size2: SIZE_MENU_WIDTH as i32, Color1: COLOR_MENU as i32, Color2: COLOR_MENUHILIGHT as i32, Font: FONT_MENU as i32, FontColor: COLOR_MENUTEXT as i32 },
    ASSIGNMENT { Size: SIZE_MENU_HEIGHT as i32, Size2: SIZE_MENU_WIDTH as i32, Color1: COLOR_HIGHLIGHT as i32, Color2: -1, Font: -1, FontColor: COLOR_HIGHLIGHTTEXT as i32 },
    ASSIGNMENT { Size: -1, Size2: -1, Color1: COLOR_WINDOW as i32, Color2: -1, Font: -1, FontColor: COLOR_WINDOWTEXT as i32 },
    ASSIGNMENT { Size: SIZE_SCROLL_WIDTH as i32, Size2: SIZE_SCROLL_HEIGHT as i32, Color1: COLOR_SCROLLBAR as i32, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: -1, Size2: -1, Color1: COLOR_3DFACE as i32, Color2: -1, Font: -1, FontColor: COLOR_BTNTEXT as i32 },
    ASSIGNMENT { Size: SIZE_SM_CAPTION_HEIGHT as i32, Size2: -1, Color1: -1, Color2: -1, Font: FONT_SMCAPTION as i32, FontColor: -1 },
    ASSIGNMENT { Size: SIZE_CAPTION_HEIGHT as i32, Size2: -1, Color1: -1, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: -1, Size2: -1, Color1: -1, Color2: -1, Font: FONT_MESSAGE as i32, FontColor: COLOR_WINDOWTEXT as i32 },
    ASSIGNMENT { Size: -1, Size2: -1, Color1: COLOR_APPWORKSPACE as i32, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: SIZE_ICON_SPACE_X as i32, Size2: -1, Color1: -1, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: SIZE_ICON_SPACE_Y as i32, Size2: -1, Color1: -1, Color2: -1, Font: -1, FontColor: -1 },
    ASSIGNMENT { Size: -1, Size2: -1, Color1: COLOR_INFOBK as i32, Color2: -1, Font: FONT_STATUS as i32, FontColor: COLOR_INFOTEXT as i32 },
    ASSIGNMENT { Size: SIZE_ICON as i32, Size2: -1, Color1: -1, Color2: -1, Font: FONT_ICON as i32, FontColor: -1 },
];
unsafe extern "C" fn UpdateButtonColor(hwndDlg: HWND, g: *mut GLOBALS, ID: i32, nButton: i32, nColor: i32) {
    let mut hdcColorButton: HDC;
    let mut hdcCompat: HDC;
    let mut rect: RECT = core::mem::zeroed();
    let mut hbrush: HBRUSH;
    let mut hwndColorButton: HWND;
    let mut hgdiTmp: HGDIOBJ;

    if nColor != -1
    {
        hwndColorButton = GetDlgItem(hwndDlg, ID);
        hdcColorButton = GetDC(hwndColorButton);
        hdcCompat = CreateCompatibleDC(hdcColorButton);
        ReleaseDC(hwndColorButton, hdcColorButton);

        hgdiTmp = SelectObject(hdcCompat, ((*g).hbmpColor[nButton as usize]).cast());

        rect.left = 2;
        rect.top = 2;
        rect.right = 22;
        rect.bottom = 13;
        hbrush = CreateSolidBrush((*g).SchemeAdv.crColor[nColor as usize]);
        FillRect(hdcCompat, &rect, hbrush);
        DeleteObject((hbrush).cast());

        SelectObject(hdcCompat, hgdiTmp);
        DeleteDC(hdcCompat);

        SendDlgItemMessage(hwndDlg, ID, BM_SETIMAGE, IMAGE_BITMAP as WPARAM, (*g).hbmpColor[nButton as usize] as LPARAM);
        EnableWindow(GetDlgItem(hwndDlg, ID), (TRUE) as BOOL);
    }
    else
    {
        SendDlgItemMessage(hwndDlg, ID, BM_SETIMAGE, IMAGE_BITMAP as WPARAM, core::ptr::null_mut::<core::ffi::c_void>() as LPARAM);
        EnableWindow(GetDlgItem(hwndDlg, ID), (FALSE) as BOOL);
    }
}
unsafe extern "C" fn InitColorButtons(hwndDlg: HWND, g: *mut GLOBALS) {
    let mut i: INT;
    let mut hdcColorButton: HDC;
    let mut hdcCompat: HDC;
    let mut rect: RECT = core::mem::zeroed();
    let mut hbrush: HBRUSH;
    let mut hPen: HPEN;
    let hwndColorButton: HWND;
    let mut hgdiTemp: HGDIOBJ;
    let scheme: *mut COLOR_SCHEME = core::ptr::addr_of_mut!((*g).SchemeAdv);

    let Points: [POINT; 3] = [
        POINT { x: 29, y: 6 },
        POINT { x: 33, y: 6 },
        POINT { x: 31, y: 8 },
    ];

    hwndColorButton = GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_COLOR1_B) as i32);
    hdcColorButton = GetDC(hwndColorButton);

    i = 0;
    while i <= 2 {
        hdcCompat = CreateCompatibleDC(hdcColorButton);

        (*g).hbmpColor[i as usize] = CreateCompatibleBitmap(hdcColorButton, 36, 15);

        hgdiTemp = SelectObject(hdcCompat, (*g).hbmpColor[i as usize] as HGDIOBJ);

        rect.left = 0;
        rect.top = 0;
        rect.right = 36;
        rect.bottom = 15;
        hbrush = CreateSolidBrush((*scheme).crColor[COLOR_BTNFACE as usize]);
        FillRect(hdcCompat, &rect, hbrush);
        DeleteObject(hbrush as HGDIOBJ);

        rect.left = 1;
        rect.top = 1;
        rect.right = 23;
        rect.bottom = 14;
        hbrush = CreateSolidBrush((*scheme).crColor[COLOR_BTNTEXT as usize]);
        FillRect(hdcCompat, &rect, hbrush);
        DeleteObject(hbrush as HGDIOBJ);

        hPen = CreatePen((PS_SOLID) as i32, 1, (*scheme).crColor[COLOR_BTNSHADOW as usize]);
        SelectObject(hdcCompat, hPen as HGDIOBJ);
        MoveToEx(hdcCompat, 26, 1, core::ptr::null_mut());
        LineTo(hdcCompat, 26, 14);
        SelectObject(hdcCompat, GetStockObject((BLACK_PEN) as i32));
        DeleteObject(hPen as HGDIOBJ);

        hPen = CreatePen((PS_SOLID) as i32, 1, (*scheme).crColor[COLOR_BTNHIGHLIGHT as usize]);
        SelectObject(hdcCompat, hPen as HGDIOBJ);
        MoveToEx(hdcCompat, 27, 1, core::ptr::null_mut());
        LineTo(hdcCompat, 27, 14);
        SelectObject(hdcCompat, GetStockObject((BLACK_PEN) as i32));
        DeleteObject(hPen as HGDIOBJ);

        hPen = CreatePen((PS_SOLID) as i32, 1, (*scheme).crColor[COLOR_BTNTEXT as usize]);
        hbrush = CreateSolidBrush((*scheme).crColor[COLOR_BTNTEXT as usize]);
        SelectObject(hdcCompat, hPen as HGDIOBJ);
        SelectObject(hdcCompat, hbrush as HGDIOBJ);
        SetPolyFillMode(hdcCompat, (WINDING) as i32);

        Polygon(hdcCompat, Points.as_ptr(), 3);

        SelectObject(hdcCompat, hgdiTemp);
        DeleteDC(hdcCompat);
        DeleteObject(hPen as HGDIOBJ);
        DeleteObject(hbrush as HGDIOBJ);

        i += 1;
    }

    ReleaseDC(hwndColorButton, hdcColorButton);

    SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_COLOR1_B) as i32, BM_SETIMAGE, IMAGE_BITMAP as WPARAM, (*g).hbmpColor[0 as usize] as LPARAM);
    SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_COLOR2_B) as i32, BM_SETIMAGE, IMAGE_BITMAP as WPARAM, (*g).hbmpColor[1 as usize] as LPARAM);
    SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTCOLOR_B) as i32, BM_SETIMAGE, IMAGE_BITMAP as WPARAM, (*g).hbmpColor[2 as usize] as LPARAM);
}
unsafe extern "system" fn EnumFontFamExProc(
    lpelfe: *mut ENUMLOGFONTEX,
    lpntme: *mut NEWTEXTMETRICEX,
    dwFontType: DWORD,
    lParam: LPARAM,
) -> i32 {
    if SendMessage(lParam as HWND, CB_GETCOUNT, 0, 0) >= 100 {
        return 0;
    }

    if SendMessage(
        lParam as HWND,
        CB_FINDSTRINGEXACT,
        -1i32 as WPARAM,
        core::ptr::addr_of_mut!((*lpelfe).elfLogFont.lfFaceName) as LPARAM,
    ) != (CB_ERR) as LPARAM
    {
        return 2;
    }

    SendMessage(
        lParam as HWND,
        CB_ADDSTRING,
        0,
        ((core::ptr::addr_of_mut!((*lpelfe).elfLogFont.lfFaceName) as WPARAM) as i32) as LPARAM,
    );

    1
}
unsafe extern "C" fn UpdateControls(hwndDlg: HWND, g: *mut GLOBALS) {
    let mut iElement: i32;
    let mut hdcDlg: HDC;

    iElement = (*g).CurrentElement;

    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_E) as i32), (g_Assignment[iElement as usize].Size != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_UD) as i32), (g_Assignment[iElement as usize].Size != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_T) as i32), (g_Assignment[iElement as usize].Size != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_COLOR1_T) as i32), (g_Assignment[iElement as usize].Color1 != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_COLOR2_T) as i32), (g_Assignment[iElement as usize].Color2 != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONT_T) as i32), (g_Assignment[iElement as usize].Font != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONT_C) as i32), (g_Assignment[iElement as usize].Font != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONTSIZE_T) as i32), (g_Assignment[iElement as usize].Font != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONTSIZE_E) as i32), (g_Assignment[iElement as usize].Font != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONTCOLOR_T) as i32), (g_Assignment[iElement as usize].FontColor != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONTBOLD) as i32), (g_Assignment[iElement as usize].Font != -1) as i32);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONTITALIC) as i32), (g_Assignment[iElement as usize].Font != -1) as i32);

    UpdateButtonColor(hwndDlg, g, (IDC_ADVAPPEARANCE_COLOR1_B) as i32, 0, g_Assignment[iElement as usize].Color1);
    UpdateButtonColor(hwndDlg, g, (IDC_ADVAPPEARANCE_COLOR2_B) as i32, 1, g_Assignment[iElement as usize].Color2);
    UpdateButtonColor(hwndDlg, g, (IDC_ADVAPPEARANCE_FONTCOLOR_B) as i32, 2, g_Assignment[iElement as usize].FontColor);

    if g_Assignment[iElement as usize].Size != -1 {
        SetDlgItemInt(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_E) as i32, (SchemeGetMetric(core::ptr::addr_of_mut!((*g).SchemeAdv), g_Assignment[iElement as usize].Size)) as UINT, (FALSE) as BOOL);
    } else {
        SetDlgItemText(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_E) as i32, w!(""));
    }

    hdcDlg = GetDC(hwndDlg);
    if g_Assignment[iElement as usize].Font != -1 {
        let plfFont: PLOGFONTW = SchemeGetFont(core::ptr::addr_of_mut!((*g).SchemeAdv), g_Assignment[iElement as usize].Font);

        SetDlgItemText(hwndDlg, (IDC_ADVAPPEARANCE_FONT_C) as i32, (*plfFont).lfFaceName.as_ptr());
        SetDlgItemInt(hwndDlg, (IDC_ADVAPPEARANCE_FONTSIZE_E) as i32, (-MulDiv((*plfFont).lfHeight, 72, GetDeviceCaps(hdcDlg, (LOGPIXELSY) as i32))) as UINT, (FALSE) as BOOL);
        SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONT_C) as i32, CB_FINDSTRINGEXACT, -1i32 as WPARAM, (((*plfFont).lfFaceName.as_ptr() as WPARAM) as i32) as LPARAM);
        SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTBOLD) as i32, BM_SETCHECK, if (*plfFont).lfWeight == (FW_BOLD) as i32 { 1 } else { 0 }, 0);
        SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTITALIC) as i32, BM_SETCHECK, (*plfFont).lfItalic as WPARAM, 0);
    } else {
        SetDlgItemText(hwndDlg, (IDC_ADVAPPEARANCE_FONT_C) as i32, core::ptr::null_mut());
        SetDlgItemText(hwndDlg, (IDC_ADVAPPEARANCE_FONTSIZE_E) as i32, core::ptr::null_mut());
        SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTBOLD) as i32, BM_SETCHECK, 0, 0);
        SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTITALIC) as i32, BM_SETCHECK, 0, 0);
    }

    ReleaseDC(hwndDlg, hdcDlg);
}
unsafe extern "C" fn SaveCurrentValues(hwndDlg: HWND, g: *mut GLOBALS) {
    let mut bTranslated: i32 = 0;
    let hdcDlg: HDC = GetDC(hwndDlg);

    if g_Assignment[(*g).CurrentElement as usize].Size != -1 {
        SchemeSetMetric(
            core::ptr::addr_of_mut!((*g).SchemeAdv),
            g_Assignment[(*g).CurrentElement as usize].Size,
            (GetDlgItemInt(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_E) as i32, &mut bTranslated, (FALSE) as BOOL)) as i32
        );
    }

    if g_Assignment[(*g).CurrentElement as usize].Font != -1 {
        let plfFont: PLOGFONTW = SchemeGetFont(
            core::ptr::addr_of_mut!((*g).SchemeAdv),
            g_Assignment[(*g).CurrentElement as usize].Font
        );
        (*plfFont).lfHeight = -MulDiv(
            (GetDlgItemInt(hwndDlg, (IDC_ADVAPPEARANCE_FONTSIZE_E) as i32, &mut bTranslated, (FALSE) as BOOL)) as i32,
            GetDeviceCaps(hdcDlg, (LOGPIXELSY) as i32),
            72
        );
        (*plfFont).lfWeight = if SendDlgItemMessage(
            hwndDlg,
            (IDC_ADVAPPEARANCE_FONTBOLD) as i32,
            BM_GETCHECK,
            0,
            0
        ) == 1 {
            (FW_BOLD) as i32
        } else {
            (FW_NORMAL) as i32
        };
        (*plfFont).lfItalic = SendDlgItemMessage(
            hwndDlg,
            (IDC_ADVAPPEARANCE_FONTITALIC) as i32,
            BM_GETCHECK,
            0,
            0
        ) as u8;
        GetDlgItemText(
            hwndDlg,
            (IDC_ADVAPPEARANCE_FONT_C) as i32,
            (*plfFont).lfFaceName.as_mut_ptr(),
            (LF_FACESIZE) as i32
        );
    }

    ReleaseDC(hwndDlg, hdcDlg);
}
unsafe extern "C" fn GetColor(hwndDlg: HWND, g: *mut GLOBALS, nButton: i32) -> i32 {
    let mut cc: CHOOSECOLOR = core::mem::zeroed();
    let mut crColor: COLORREF;
    let mut ID = 0;
    let mut ColorIndex = 0;

    match nButton {
        0 => {
            ColorIndex = g_Assignment[(*g).CurrentElement as usize].Color1;
            ID = IDC_ADVAPPEARANCE_COLOR1_B;
        }

        1 => {
            ColorIndex = g_Assignment[(*g).CurrentElement as usize].Color2;
            ID = IDC_ADVAPPEARANCE_COLOR2_B;
        }

        2 => {
            ColorIndex = g_Assignment[(*g).CurrentElement as usize].FontColor;
            ID = IDC_ADVAPPEARANCE_FONTCOLOR_B;
        }

        _ => {}
    }

    crColor = (*g).SchemeAdv.crColor[ColorIndex as usize];

    cc.lStructSize = core::mem::size_of::<CHOOSECOLOR>() as u32;
    cc.hwndOwner = hwndDlg;
    cc.hInstance = core::ptr::null_mut();
    cc.rgbResult = crColor;
    cc.lpCustColors = (*g).crCustom.as_mut_ptr();
    cc.Flags = CC_ANYCOLOR | CC_FULLOPEN | CC_RGBINIT;
    cc.lCustData = 0;
    cc.lpfnHook = None;
    cc.lpTemplateName = core::ptr::null_mut();

    if ChooseColor(&mut cc) != 0 {
        (*g).SchemeAdv.crColor[ColorIndex as usize] = cc.rgbResult;
        if crColor != cc.rgbResult {
            UpdateButtonColor(hwndDlg, g, (ID) as i32, nButton, ColorIndex);
            SendDlgItemMessage(hwndDlg, (IDC_APPEARANCE_PREVIEW) as i32, PVM_UPDATETHEME, 0, &(*g).SchemeAdv as *const _ as LPARAM);
            return (TRUE) as i32;
        }
    }

    return (FALSE) as i32;
}
unsafe extern "C" fn AdvAppearanceDlg_Init(hwndDlg: HWND, g: *mut GLOBALS) {
    let mut i: INT;
    let mut iElement: INT;
    let mut iListIndex: INT;
    let mut iDeskIndex: INT = 0;
    let mut tstrText: [TCHAR; 80] = [0; 80];
    let mut lfFont: LOGFONT = core::mem::zeroed();
    let mut lfButtonFont: LOGFONT;
    let mut hMyFont: HFONT;
    let hScreenDC: HDC;
    let mut Size: [TCHAR; 4] = [0; 4];

    (*g).SchemeAdv = (*g).Scheme;

    SendDlgItemMessage(hwndDlg, (IDC_APPEARANCE_PREVIEW) as i32, PVM_UPDATETHEME, 0, core::ptr::addr_of_mut!((*g).SchemeAdv) as LPARAM);

    iElement = 0;
    while iElement < (NUM_ELEMENTS) as i32 {
        LoadString(hApplet, IDS_ELEMENT_0 + (iElement) as u32, tstrText.as_mut_ptr() as LPTSTR, 79);
        iListIndex = (SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as i32, CB_ADDSTRING, 0, tstrText.as_ptr() as LPARAM)) as i32;
        SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as i32, CB_SETITEMDATA, iListIndex as WPARAM, iElement as LPARAM);
        iElement += 1;
    }

    iListIndex = 0;
    while iListIndex < (NUM_ELEMENTS) as i32 {
        iElement = SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as i32, CB_GETITEMDATA, iListIndex as WPARAM, 0) as INT;
        if iElement == 0 {
            iDeskIndex = iListIndex;
            break;
        }
        iListIndex += 1;
    }

    SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as i32, CB_SETCURSEL, iDeskIndex as WPARAM, 0);

    lfButtonFont = (*g).Scheme.ncMetrics.lfMessageFont;
    lfButtonFont.lfWeight = (FW_BOLD) as i32;
    lfButtonFont.lfItalic = (FALSE) as u8;
    hMyFont = CreateFontIndirect(core::ptr::addr_of_mut!(lfButtonFont));
    if !hMyFont.is_null() {
        if !(*g).hBoldFont.is_null() {
            DeleteObject(((*g).hBoldFont).cast());
        }
        (*g).hBoldFont = hMyFont;
    }

    lfButtonFont.lfWeight = (FW_REGULAR) as i32;
    lfButtonFont.lfItalic = (TRUE) as u8;
    hMyFont = CreateFontIndirect(core::ptr::addr_of_mut!(lfButtonFont));
    if !hMyFont.is_null() {
        if !(*g).hItalicFont.is_null() {
            DeleteObject(((*g).hItalicFont).cast());
        }
        (*g).hItalicFont = hMyFont;
    }

    SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTBOLD) as i32, WM_SETFONT, (*g).hBoldFont as WPARAM, TRUE as LPARAM);
    SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTITALIC) as i32, WM_SETFONT, (*g).hItalicFont as WPARAM, TRUE as LPARAM);

    InitColorButtons(hwndDlg, g);

    SendMessage(GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_UD) as i32), UDM_SETRANGE, 0, MAKELONG!(200, 1) as LPARAM);

    lfFont.lfCharSet = (DEFAULT_CHARSET) as u8;
    lfFont.lfFaceName[0] = 0 as TCHAR;
    lfFont.lfPitchAndFamily = 0;
    hScreenDC = GetDC((0) as *mut HWND__);
    EnumFontFamiliesEx(hScreenDC, core::ptr::addr_of_mut!(lfFont), Some(core::mem::transmute(EnumFontFamExProc as *const ())), GetDlgItem(hwndDlg, (IDC_ADVAPPEARANCE_FONT_C) as i32) as LPARAM, 0);
    ReleaseDC((0) as *mut HWND__, hScreenDC);

    i = 6;
    while i <= 24 {
        StringCbPrintf!(Size.as_mut_ptr(), core::mem::size_of_val(&Size), w!("%d"), i);
        SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTSIZE_E) as i32, CB_ADDSTRING, 0, Size.as_ptr() as LPARAM);
        i += 1;
    }

    iListIndex = SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as i32, CB_GETCURSEL, 0, 0) as INT;
    (*g).CurrentElement = SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as i32, CB_GETITEMDATA, iListIndex as WPARAM, 0) as INT;
    UpdateControls(hwndDlg, g);
}
unsafe extern "C" fn AdvAppearanceDlg_CleanUp(hwndDlg: HWND, g: *mut GLOBALS) {
    DeleteObject(((*g).hBoldFont).cast());
    DeleteObject(((*g).hItalicFont).cast());
}
unsafe extern "C" fn SelectComboByElement(hwnd: HWND, id: INT, lParam: LPARAM) {
    let nCount: INT;
    let mut i: INT;

    nCount = (SendDlgItemMessage(hwnd, id, CB_GETCOUNT, 0, 0)) as i32;
    if nCount == CB_ERR {
        return;
    }

    i = 0;
    while i < nCount {
        if SendDlgItemMessage(hwnd, id, CB_GETITEMDATA, i as WPARAM, 0) == lParam {
            SendDlgItemMessage(hwnd, id, CB_SETCURSEL, i as WPARAM, 0);
            break;
        }
        i += 1;
    }
}
unsafe extern "C" fn GetSelectedComboText(hwnd: HWND, id: INT, lpStr: LPWSTR) {
    let mut nCount: INT;

    nCount = (SendDlgItemMessage(hwnd, id, CB_GETCURSEL, 0, 0)) as i32;
    if nCount == CB_ERR {
        *lpStr = 0;
        return;
    }

    nCount = (SendDlgItemMessage(hwnd, id, CB_GETLBTEXT, nCount as WPARAM, lpStr as LPARAM)) as i32;
    if nCount == CB_ERR {
        *lpStr = 0;
    }
}
unsafe extern "C" fn GetSelectedComboInt(hwnd: HWND, id: INT) -> INT {
    let mut szBuffer: [TCHAR; 80] = [0; 80];
    let mut nCount: INT;

    nCount = SendDlgItemMessage(hwnd, id, CB_GETCURSEL, 0, 0) as INT;
    if nCount == CB_ERR {
        return 0;
    }

    nCount = SendDlgItemMessage(hwnd, id, CB_GETLBTEXT, nCount as WPARAM, szBuffer.as_mut_ptr() as LPARAM) as INT;
    if nCount == CB_ERR {
        return 0;
    }

    return _ttoi(szBuffer.as_ptr());
}
#[no_mangle]
pub unsafe extern "system" fn AdvAppearanceDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut iSelection: INT;
    let mut iProperty: INT;
    let mut g: *mut GLOBALS;
    let mut plfFont: PLOGFONTW;

    g = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut GLOBALS;

    match uMsg {
        WM_INITDIALOG => {
            g = lParam as *mut GLOBALS;
            SetWindowLongPtr(hwndDlg, DWLP_USER, g as LONG_PTR);
            AdvAppearanceDlg_Init(hwndDlg, g);
        }

        WM_DESTROY => {
            AdvAppearanceDlg_CleanUp(hwndDlg, g);
        }

        WM_COMMAND => {
            if g.is_null() {
                return (TRUE) as LPARAM;
            }

            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    SaveCurrentValues(hwndDlg, g);
                    EndDialog(hwndDlg, (IDOK) as INT_PTR);
                }

                IDCANCEL => {
                    (*g).SchemeAdv = (*g).Scheme;
                    EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
                }

                IDC_APPEARANCE_PREVIEW => {
                    SaveCurrentValues(hwndDlg, g);
                    SelectComboByElement(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as INT, lParam);
                    (*g).CurrentElement = lParam as INT;
                    UpdateControls(hwndDlg, g);
                }

                IDC_ADVAPPEARANCE_ELEMENT => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 {
                        SaveCurrentValues(hwndDlg, g);
                        iSelection = (SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_ELEMENT) as i32, CB_GETCURSEL, 0, 0)) as i32;
                        (*g).CurrentElement = SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT as c_int, CB_GETITEMDATA, iSelection as WPARAM, 0 as LPARAM) as INT;
                        UpdateControls(hwndDlg, g);
                    }
                }

                IDC_ADVAPPEARANCE_SIZE_E => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16 {
                        iProperty = g_Assignment[(*g).CurrentElement as usize].Size;
                        if iProperty == -1 {
                            return (TRUE as INT_PTR) as LPARAM;
                        }

                        iSelection = LOWORD!(SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_SIZE_UD) as i32, UDM_GETPOS, 0, 0)) as INT;
                        SchemeSetMetric(core::ptr::addr_of_mut!((*g).SchemeAdv), iProperty, iSelection);
                        SendDlgItemMessage(hwndDlg, (IDC_APPEARANCE_PREVIEW) as i32, PVM_SETSIZE, iProperty as WPARAM, iSelection as LPARAM);

                        iProperty = g_Assignment[(*g).CurrentElement as usize].Size2;
                        if iProperty != -1 {
                            SchemeSetMetric(core::ptr::addr_of_mut!((*g).SchemeAdv), iProperty, iSelection);
                        }
                    }
                }

                IDC_ADVAPPEARANCE_FONT_C => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 {
                        iProperty = g_Assignment[(*g).CurrentElement as usize].Font;
                        if iProperty == -1 {
                            return TRUE as INT_PTR;
                        }

                        plfFont = SchemeGetFont(core::ptr::addr_of_mut!((*g).SchemeAdv), iProperty);
                        GetSelectedComboText(hwndDlg, (IDC_ADVAPPEARANCE_FONT_C) as INT, ((*plfFont).lfFaceName.as_ptr()) as *mut u16);
                        SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW as c_int, PVM_SETFONT, iProperty as WPARAM, plfFont as LPARAM);
                    }
                }

                IDC_ADVAPPEARANCE_FONTSIZE_E => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 || HIWORD!(wParam) == (CBN_EDITCHANGE) as u16 {
                        let hdcDlg: HDC;

                        iProperty = g_Assignment[(*g).CurrentElement as usize].Font;
                        if iProperty == -1 {
                            return TRUE as INT_PTR;
                        }

                        hdcDlg = GetDC(hwndDlg);

                        plfFont = SchemeGetFont(core::ptr::addr_of_mut!((*g).SchemeAdv), iProperty);
                        iSelection = GetSelectedComboInt(hwndDlg, (IDC_ADVAPPEARANCE_FONTSIZE_E) as INT);
                        (*plfFont).lfHeight = -MulDiv(iSelection, GetDeviceCaps(hdcDlg, (LOGPIXELSY) as i32), 72);
                        SendDlgItemMessage(hwndDlg, (IDC_APPEARANCE_PREVIEW) as i32, PVM_UPDATETHEME, 0, core::ptr::addr_of_mut!((*g).SchemeAdv) as LPARAM);

                        ReleaseDC(hwndDlg, hdcDlg);
                    }
                }

                IDC_ADVAPPEARANCE_FONTBOLD => {
                    if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                        iProperty = g_Assignment[(*g).CurrentElement as usize].Font;
                        if iProperty == -1 {
                            return TRUE as INT_PTR;
                        }

                        plfFont = SchemeGetFont(core::ptr::addr_of_mut!((*g).SchemeAdv), iProperty);
                        iSelection = SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTBOLD) as i32, BM_GETCHECK, 0, 0) as INT;
                        (*plfFont).lfWeight = if iSelection == (BST_CHECKED) as i32 { (FW_BOLD) as i32 } else { (FW_NORMAL) as i32 };
                        SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW as c_int, PVM_SETFONT, iProperty as WPARAM, plfFont as LPARAM);
                    }
                }

                IDC_ADVAPPEARANCE_FONTITALIC => {
                    if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                        iProperty = g_Assignment[(*g).CurrentElement as usize].Font;
                        if iProperty == -1 {
                            return TRUE as INT_PTR;
                        }

                        plfFont = SchemeGetFont(core::ptr::addr_of_mut!((*g).SchemeAdv), iProperty);
                        iSelection = SendDlgItemMessage(hwndDlg, (IDC_ADVAPPEARANCE_FONTITALIC) as i32, BM_GETCHECK, 0, 0) as INT;
                        (*plfFont).lfItalic = if iSelection == (BST_CHECKED) as i32 { (TRUE) as u8 } else { (FALSE) as u8 };
                        SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW as c_int, PVM_SETFONT, iProperty as WPARAM, plfFont as LPARAM);
                    }
                }

                IDC_ADVAPPEARANCE_COLOR1_B => {
                    GetColor(hwndDlg, g, 0);
                }

                IDC_ADVAPPEARANCE_COLOR2_B => {
                    GetColor(hwndDlg, g, 1);
                }

                IDC_ADVAPPEARANCE_FONTCOLOR_B => {
                    GetColor(hwndDlg, g, 2);
                }

                _ => {
                    return FALSE as INT_PTR;
                }
            }
        }

        _ => {
            return FALSE as INT_PTR;
        }
    }

    return TRUE as INT_PTR;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/advappdlg.c
//  * PURPOSE:         Advanced appearance dialog
//  *
//  * PROGRAMMER:      Timo Kreuzer (timo[dot]kreuzer[at]web[dot]de)
//  *
//  */
//
// #include "desk.h"
//
// /******************************************************************************/
//
// typedef struct
// {
//     int Size;
//     int Size2;
//     int Color1;
//     int Color2;
//     int Font;
//     int FontColor;
// } ASSIGNMENT;
//
// /* This const assigns the color and metric numbers to the elements from the elements list */
//
// /*   Size 1 (width)          Size 2 (height)     Color 1                  Color 2                         Font             Fontcolor */
// const ASSIGNMENT g_Assignment[NUM_ELEMENTS] =
// {
//     {-1,                     -1,                 COLOR_DESKTOP,           -1,                             -1,              -1},                 /* Desktop */
//     {SIZE_CAPTION_HEIGHT,    -1,                 COLOR_INACTIVECAPTION,   COLOR_GRADIENTINACTIVECAPTION,  FONT_CAPTION,    COLOR_INACTIVECAPTIONTEXT},/* inactive window caption */
//     {SIZE_BORDER_WIDTH,      -1,                 COLOR_INACTIVEBORDER,    -1,                             -1,              -1},                 /* inactive window border */
//     {SIZE_CAPTION_HEIGHT,    -1,                 COLOR_ACTIVECAPTION,     COLOR_GRADIENTACTIVECAPTION,    FONT_CAPTION,    COLOR_CAPTIONTEXT},  /* active window caption */
//     {SIZE_BORDER_WIDTH,      -1,                 COLOR_ACTIVEBORDER,      -1,                             -1,              -1},                 /* active window border */
//     {SIZE_MENU_HEIGHT,       SIZE_MENU_WIDTH,    COLOR_MENU,              COLOR_MENUHILIGHT,              FONT_MENU,       COLOR_MENUTEXT},     /* menu */
//     {SIZE_MENU_HEIGHT,       SIZE_MENU_WIDTH,    COLOR_HIGHLIGHT,         -1,                             -1,              COLOR_HIGHLIGHTTEXT},/* marked element */
//     {-1,                     -1,                 COLOR_WINDOW,            -1 /*COLOR_WINDOWFRAME*/,       -1,              COLOR_WINDOWTEXT},   /* window */
//     {SIZE_SCROLL_WIDTH,      SIZE_SCROLL_HEIGHT, COLOR_SCROLLBAR,         -1,                             -1,              -1},                 /* scroll bar */
//     {-1,                     -1,                 COLOR_3DFACE,            -1,                             -1,              COLOR_BTNTEXT},      /* 3d objects */
//     {SIZE_SM_CAPTION_HEIGHT, -1,                 -1,                      -1,                             FONT_SMCAPTION,  -1},                 /* palette (small) window caption */
//     {SIZE_CAPTION_HEIGHT,    -1,                 -1,                      -1,                             -1,              -1},                 /* caption bar buttons */
//     {-1,                     -1,                 -1,                      -1,                             FONT_MESSAGE,    COLOR_WINDOWTEXT},   /* dialog */
//     {-1,                     -1,                 COLOR_APPWORKSPACE,      -1,                             -1,              -1},                 /* application background */
//     {SIZE_ICON_SPACE_X,      -1,                 -1,                      -1,                             -1,              -1},                 /* icon distance horiz. */
//     {SIZE_ICON_SPACE_Y,      -1,                 -1,                      -1,                             -1,              -1},                 /* icon distance vert. */
//     {-1,                     -1,                 COLOR_INFOBK,            -1,                             FONT_STATUS,     COLOR_INFOTEXT},     /* quickinfo */
//     {SIZE_ICON,              -1,                 -1,                      -1,                             FONT_ICON,       -1},                 /* icon */
// //  {-1,                     -1,                 -1,                      -1,                             -1,              COLOR_GRAYTEXT},     /* inactive menu item -- FIXME: Access? */
// };
//
// /******************************************************************************/
//
// /* Draw the current color on the color picker buttons */
// static VOID
// UpdateButtonColor(HWND hwndDlg, GLOBALS* g, INT ID, INT nButton, INT nColor)
// {
//     HDC hdcColorButton, hdcCompat;
//     RECT rect;
//     HBRUSH hbrush;
//     HWND hwndColorButton;
//     HGDIOBJ hgdiTmp;
//
//     if (nColor != -1)
//     {
//         /* Create a DC to draw on */
//         hwndColorButton = GetDlgItem(hwndDlg, ID);
//         hdcColorButton = GetDC(hwndColorButton);
//         hdcCompat = CreateCompatibleDC(hdcColorButton);
//         ReleaseDC(hwndColorButton, hdcColorButton);
//
//         /* Select the button image to it */
//         hgdiTmp = SelectObject(hdcCompat, g->hbmpColor[nButton]);
//
//         /* Create a brush and draw the rectangle */
//         rect.left = 2;
//         rect.top = 2;
//         rect.right = 22;
//         rect.bottom = 13;
//         hbrush = CreateSolidBrush(g->SchemeAdv.crColor[nColor]);
//         FillRect(hdcCompat, &rect, hbrush);
//         DeleteObject(hbrush);
//
//         /* hdcCompat is not needed anymore */
//         SelectObject(hdcCompat,hgdiTmp);
//         DeleteDC(hdcCompat);
//
//         SendDlgItemMessage(hwndDlg, ID, BM_SETIMAGE, (WPARAM)IMAGE_BITMAP, (LPARAM)g->hbmpColor[nButton]);
//         EnableWindow(GetDlgItem(hwndDlg, ID), TRUE);
//     }
//     else
//     {
//         SendDlgItemMessage(hwndDlg, ID, BM_SETIMAGE, (WPARAM)IMAGE_BITMAP, (LPARAM)NULL);
//         EnableWindow(GetDlgItem(hwndDlg, ID), FALSE);
//     }
// }
//
//
// /* Create the basic bitmaps for the color picker buttons */
// static VOID
// InitColorButtons(HWND hwndDlg, GLOBALS* g)
// {
//     INT i;
//     HDC hdcColorButton, hdcCompat;
//     RECT rect;
//     HBRUSH hbrush;
//     HPEN hPen;
//     HWND hwndColorButton;
//     HGDIOBJ hgdiTemp;
//     COLOR_SCHEME *scheme = &g->SchemeAdv;
//
//     const POINT Points[3] = {{29,6},{33,6},{31,8}};
//
//     hwndColorButton = GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_COLOR1_B);
//     hdcColorButton = GetDC(hwndColorButton);
//     for (i = 0; i <= 2; i++)
//     {
//         /* Create a DC to draw on */
//         hdcCompat = CreateCompatibleDC(hdcColorButton);
//
//         /* Create the button image */
//         g->hbmpColor[i] = CreateCompatibleBitmap(hdcColorButton, 36, 15);
//
//         /* Select the button image to the DC */
//         hgdiTemp = SelectObject(hdcCompat, g->hbmpColor[i]);
//
//         /* Draw the buttons background color */
//         rect.left = 0;
//         rect.top = 0;
//         rect.right = 36;
//         rect.bottom = 15;
//         hbrush = CreateSolidBrush(scheme->crColor[COLOR_BTNFACE]);
//         FillRect(hdcCompat, &rect, hbrush);
//         DeleteObject(hbrush);
//
//         /* Draw the rectangle */
//         rect.left = 1;
//         rect.top = 1;
//         rect.right = 23;
//         rect.bottom = 14;
//         hbrush = CreateSolidBrush(scheme->crColor[COLOR_BTNTEXT]);
//         FillRect(hdcCompat, &rect, hbrush);
//         DeleteObject(hbrush);
//
//         /* Draw left side of line */
//         hPen = CreatePen(PS_SOLID, 1, scheme->crColor[COLOR_BTNSHADOW]);
//         SelectObject(hdcCompat, hPen);
//         MoveToEx(hdcCompat, 26, 1, NULL);
//         LineTo(hdcCompat, 26, 14);
//         SelectObject(hdcCompat, GetStockObject(BLACK_PEN));
//         DeleteObject(hPen);
//
//         /* Draw right side of line */
//         hPen = CreatePen(PS_SOLID, 1, scheme->crColor[COLOR_BTNHIGHLIGHT]);
//         SelectObject(hdcCompat,hPen);
//         MoveToEx(hdcCompat, 27, 1, NULL);
//         LineTo(hdcCompat, 27, 14);
//         SelectObject(hdcCompat, GetStockObject(BLACK_PEN));
//         DeleteObject(hPen);
//
//         /* Draw triangle */
//         hPen = CreatePen(PS_SOLID, 1, scheme->crColor[COLOR_BTNTEXT]);
//         hbrush = CreateSolidBrush(scheme->crColor[COLOR_BTNTEXT]);
//         SelectObject(hdcCompat, hPen);
//         SelectObject(hdcCompat, hbrush);
//         SetPolyFillMode(hdcCompat, WINDING);
//
//         /* FIXME: HACK, see Points definition */
//         Polygon(hdcCompat, Points, 3);
//
//         /* Cleanup */
//         SelectObject(hdcCompat,hgdiTemp);
//         DeleteDC(hdcCompat);
//         DeleteObject(hPen);
//         DeleteObject(hbrush);
//     }
//
//     ReleaseDC(hwndColorButton, hdcColorButton);
//
//     /* Set the images of the buttons */
//     SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_COLOR1_B, BM_SETIMAGE, (WPARAM)IMAGE_BITMAP, (LPARAM)g->hbmpColor[0]);
//     SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_COLOR2_B, BM_SETIMAGE, (WPARAM)IMAGE_BITMAP, (LPARAM)g->hbmpColor[1]);
//     SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTCOLOR_B, BM_SETIMAGE, (WPARAM)IMAGE_BITMAP, (LPARAM)g->hbmpColor[2]);
// }
//
//
// /* This is the callback function to add the installed fonts to the font combo */
// static int __stdcall
// EnumFontFamExProc(ENUMLOGFONTEX *lpelfe, NEWTEXTMETRICEX *lpntme, DWORD dwFontType, LPARAM lParam)
// {
//     /* Don't enumerate more than 100 fonts */
//     if (SendMessage((HWND)lParam, CB_GETCOUNT, 0, 0) >= 100)
//         return 0;
//
//     /* Only add the string once */
//     if (SendMessage((HWND)lParam, CB_FINDSTRINGEXACT, -1, (WPARAM)&(lpelfe->elfLogFont.lfFaceName)) != CB_ERR)
//         return 2;
//
//     SendMessage((HWND)lParam, CB_ADDSTRING, 0, (WPARAM)&(lpelfe->elfLogFont.lfFaceName));
//
//     return 1;
// }
//
//
// /* Update all the controls with the current values for the selected screen element */
// static VOID
// UpdateControls(HWND hwndDlg, GLOBALS *g)
// {
//     INT iElement;
//     HDC hdcDlg;
//
//     iElement = g->CurrentElement;
//
//     /* First enable / disable the controls */
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_SIZE_E), (g_Assignment[iElement].Size != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_SIZE_UD), (g_Assignment[iElement].Size != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_SIZE_T), (g_Assignment[iElement].Size != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_COLOR1_T), (g_Assignment[iElement].Color1 != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_COLOR2_T), (g_Assignment[iElement].Color2 != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONT_T), (g_Assignment[iElement].Font != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONT_C), (g_Assignment[iElement].Font != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONTSIZE_T), (g_Assignment[iElement].Font != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONTSIZE_E), (g_Assignment[iElement].Font != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONTCOLOR_T), (g_Assignment[iElement].FontColor != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONTBOLD), (g_Assignment[iElement].Font != -1));
//     EnableWindow(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONTITALIC), (g_Assignment[iElement].Font != -1));
//
//     /* Update the colors of the color buttons */
//     UpdateButtonColor(hwndDlg, g, IDC_ADVAPPEARANCE_COLOR1_B, 0, g_Assignment[iElement].Color1);
//     UpdateButtonColor(hwndDlg, g, IDC_ADVAPPEARANCE_COLOR2_B, 1, g_Assignment[iElement].Color2);
//     UpdateButtonColor(hwndDlg, g, IDC_ADVAPPEARANCE_FONTCOLOR_B, 2, g_Assignment[iElement].FontColor);
//
//     if (g_Assignment[iElement].Size != -1)
//         SetDlgItemInt(hwndDlg, IDC_ADVAPPEARANCE_SIZE_E,  SchemeGetMetric(&g->SchemeAdv, g_Assignment[iElement].Size), FALSE);
//     else
//         SetDlgItemText(hwndDlg, IDC_ADVAPPEARANCE_SIZE_E, TEXT(""));
//
//     hdcDlg = GetDC(hwndDlg);
//     if (g_Assignment[iElement].Font != -1)
//     {
//         PLOGFONTW plfFont = SchemeGetFont(&g->SchemeAdv, g_Assignment[iElement].Font);
//
//         SetDlgItemText(hwndDlg, IDC_ADVAPPEARANCE_FONT_C, plfFont->lfFaceName);
//         SetDlgItemInt(hwndDlg, IDC_ADVAPPEARANCE_FONTSIZE_E, -MulDiv(plfFont->lfHeight, 72, GetDeviceCaps(hdcDlg, LOGPIXELSY)), FALSE);
//         SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONT_C, CB_FINDSTRINGEXACT, -1, (WPARAM)plfFont->lfFaceName);
//         SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTBOLD, BM_SETCHECK, plfFont->lfWeight == FW_BOLD?1:0, 0);
//         SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTITALIC, BM_SETCHECK, plfFont->lfItalic, 0);
//     }
//     else
//     {
//         SetDlgItemText(hwndDlg, IDC_ADVAPPEARANCE_FONT_C, NULL);
//         SetDlgItemText(hwndDlg, IDC_ADVAPPEARANCE_FONTSIZE_E, NULL);
//         SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTBOLD, BM_SETCHECK, 0, 0);
//         SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTITALIC, BM_SETCHECK, 0, 0);
//     }
//
//     ReleaseDC(hwndDlg, hdcDlg);
// }
//
//
// static VOID
// SaveCurrentValues(HWND hwndDlg, GLOBALS *g)
// {
//     BOOL bTranslated;
//     HDC hdcDlg = GetDC(hwndDlg);
//
//     if (g_Assignment[g->CurrentElement].Size != -1)
//     {
//         SchemeSetMetric(&g->SchemeAdv, g_Assignment[g->CurrentElement].Size, GetDlgItemInt(hwndDlg, IDC_ADVAPPEARANCE_SIZE_E, &bTranslated, FALSE));
//     }
//
//     if (g_Assignment[g->CurrentElement].Font != -1)
//     {
//         PLOGFONTW plfFont = SchemeGetFont(&g->SchemeAdv, g_Assignment[g->CurrentElement].Font);
//         plfFont->lfHeight = -MulDiv(GetDlgItemInt(hwndDlg, IDC_ADVAPPEARANCE_FONTSIZE_E, &bTranslated, FALSE), GetDeviceCaps(hdcDlg, LOGPIXELSY), 72);
//         plfFont->lfWeight = (SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTBOLD, BM_GETCHECK, 0, 0) == 1) ? FW_BOLD : FW_NORMAL;
//         plfFont->lfItalic = (BYTE)SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTITALIC, BM_GETCHECK, 0, 0);
//         GetDlgItemText(hwndDlg, IDC_ADVAPPEARANCE_FONT_C, plfFont->lfFaceName, LF_FACESIZE);
//     }
//
//     ReleaseDC(hwndDlg, hdcDlg);
// }
//
//
// /* Select a color using a color picker */
// static BOOL
// GetColor(HWND hwndDlg, GLOBALS* g, INT nButton)
// {
//     CHOOSECOLOR cc;
//     COLORREF crColor;
//     INT ID = 0;
//     INT ColorIndex = 0;
//
//     /* Get the color index from the element index and button number */
//     switch (nButton)
//     {
//         case 0:
//             ColorIndex = g_Assignment[g->CurrentElement].Color1;
//             ID = IDC_ADVAPPEARANCE_COLOR1_B;
//             break;
//
//         case 1:
//             ColorIndex = g_Assignment[g->CurrentElement].Color2;
//             ID = IDC_ADVAPPEARANCE_COLOR2_B;
//             break;
//
//         case 2:
//             ColorIndex = g_Assignment[g->CurrentElement].FontColor;
//             ID = IDC_ADVAPPEARANCE_FONTCOLOR_B;
//             break;
//     }
//
//     crColor = g->SchemeAdv.crColor[ColorIndex];
//
//     /* Prepare cc structure */
//     cc.lStructSize = sizeof(CHOOSECOLOR);
//     cc.hwndOwner = hwndDlg;
//     cc.hInstance = NULL;
//     cc.rgbResult = crColor;
//     cc.lpCustColors = g->crCustom;
//     cc.Flags = CC_ANYCOLOR | CC_FULLOPEN | CC_RGBINIT;
//     cc.lCustData = 0;
//     cc.lpfnHook = NULL;
//     cc.lpTemplateName = NULL;
//
//     /* Create the colorpicker */
//     if (ChooseColor(&cc))
//     {
//         g->SchemeAdv.crColor[ColorIndex] = cc.rgbResult;
//         if (crColor != cc.rgbResult)
//         {
//             UpdateButtonColor(hwndDlg, g, ID, nButton, ColorIndex);
//             SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_UPDATETHEME, 0, (LPARAM)&g->SchemeAdv);
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
//
// /* Initialize the advanced appearance dialog */
// static VOID
// AdvAppearanceDlg_Init(HWND hwndDlg, GLOBALS *g)
// {
//     INT i, iElement, iListIndex, iDeskIndex = 0;
//     TCHAR tstrText[80];
//     LOGFONT lfFont;
//     LOGFONT lfButtonFont;
//     HFONT hMyFont;
//     HDC hScreenDC;
//     TCHAR Size[4];
//
//     /* Copy the current theme values */
//     g->SchemeAdv = g->Scheme;
//
//     SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_UPDATETHEME, 0, (LPARAM)&g->SchemeAdv);
//
//     /* Add the elements to the combo */
//     for (iElement = 0; iElement < NUM_ELEMENTS; iElement++)
//     {
//         LoadString(hApplet, IDS_ELEMENT_0 + iElement, (LPTSTR)&tstrText, 79);
//         iListIndex = SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT, CB_ADDSTRING, 0, (LPARAM)&tstrText);
//         SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT, CB_SETITEMDATA, (WPARAM)iListIndex, (LPARAM)iElement);
//     }
//
//     /* Get the list index of the desktop element */
//     for (iListIndex = 0; iListIndex < NUM_ELEMENTS; iListIndex++)
//     {
//         iElement = SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT, CB_GETITEMDATA, (WPARAM)iListIndex, 0);
//         if (iElement == 0)
//         {
//             iDeskIndex = iListIndex;
//             break;
//         }
//     }
//
//     SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT as c_int, CB_SETCURSEL, iDeskIndex as WPARAM, 0 as LPARAM);
//
//     /* Create font for bold button */
//     lfButtonFont = g->Scheme.ncMetrics.lfMessageFont;
//     lfButtonFont.lfWeight = FW_BOLD;
//     lfButtonFont.lfItalic = FALSE;
//     hMyFont = CreateFontIndirect(&lfButtonFont);
//     if (hMyFont)
//     {
//         if (g->hBoldFont)
//             DeleteObject(g->hBoldFont);
//
//         g->hBoldFont = hMyFont;
//     }
//
//     /* Create font for italic button */
//     lfButtonFont.lfWeight = FW_REGULAR;
//     lfButtonFont.lfItalic = TRUE;
//     hMyFont = CreateFontIndirect(&lfButtonFont);
//     if (hMyFont)
//     {
//         if (g->hItalicFont)
//             DeleteObject(g->hItalicFont);
//
//         g->hItalicFont = hMyFont;
//     }
//
//     /* Set the fonts for the font style buttons */
//     SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTBOLD, WM_SETFONT, (WPARAM)g->hBoldFont, (LPARAM)TRUE);
//     SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTITALIC, WM_SETFONT, (WPARAM)g->hItalicFont, (LPARAM)TRUE);
//
//     /* Draw Bitmaps for the colorbuttons */
//     InitColorButtons(hwndDlg, g);
//
//     /* Make the UpDown control count correctly */
//     SendMessage(GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_SIZE_UD), UDM_SETRANGE, 0, MAKELONG (200, 1));
//
//     /* Fill font selection combo */
//     lfFont.lfCharSet = DEFAULT_CHARSET;
//     lfFont.lfFaceName[0] = (TCHAR)0;
//     lfFont.lfPitchAndFamily = 0;
//     hScreenDC = GetDC(0);
//     EnumFontFamiliesEx(hScreenDC, &lfFont, (FONTENUMPROC)EnumFontFamExProc, (LPARAM)GetDlgItem(hwndDlg, IDC_ADVAPPEARANCE_FONT_C), 0);
//     ReleaseDC(0, hScreenDC);
//
//     /* Fill font size combo */
//     for (i = 6; i <= 24; i++)
//     {
//         StringCbPrintf(Size, sizeof(Size), TEXT("%d"), i);
//         SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTSIZE_E, CB_ADDSTRING, 0, (LPARAM)&Size);
//     }
//
//     /* Update the controls */
//     iListIndex = SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT as c_int, CB_GETCURSEL, 0 as WPARAM, 0 as LPARAM);
//     g->CurrentElement = SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT, CB_GETITEMDATA, (WPARAM)iListIndex, 0);
//     UpdateControls(hwndDlg, g);
// }
//
//
// static VOID
// AdvAppearanceDlg_CleanUp(HWND hwndDlg, GLOBALS* g)
// {
//     DeleteObject(g->hBoldFont);
//     DeleteObject(g->hItalicFont);
// }
//
//
// static VOID
// SelectComboByElement(HWND hwnd, INT id, LPARAM lParam)
// {
//     INT nCount;
//     INT i;
//
//     nCount = SendDlgItemMessage(hwnd, id, CB_GETCOUNT, 0, 0);
//     if (nCount == CB_ERR)
//         return;
//
//     for (i = 0; i < nCount; i++)
//     {
//         if (SendDlgItemMessage(hwnd, id, CB_GETITEMDATA, (WPARAM)i, 0) == lParam)
//         {
//             SendDlgItemMessage(hwnd, id, CB_SETCURSEL, (WPARAM)i, 0);
//             break;
//         }
//     }
// }
//
//
// static VOID
// GetSelectedComboText(HWND hwnd, INT id, LPWSTR lpStr)
// {
//     INT nCount;
//
//     nCount = SendDlgItemMessage(hwnd, id, CB_GETCURSEL, 0, 0);
//     if (nCount == CB_ERR)
//     {
//         *lpStr = 0;
//         return;
//     }
//
//     nCount = SendDlgItemMessage(hwnd, id, CB_GETLBTEXT, (WPARAM)nCount, (LPARAM)lpStr);
//     if (nCount == CB_ERR)
//     {
//         *lpStr = 0;
//     }
// }
//
//
// static INT
// GetSelectedComboInt(HWND hwnd, INT id)
// {
//     TCHAR szBuffer[80];
//     INT nCount;
//
//     nCount = SendDlgItemMessage(hwnd, id, CB_GETCURSEL, 0, 0);
//     if (nCount == CB_ERR)
//         return 0;
//
//     nCount = SendDlgItemMessage(hwnd, id, CB_GETLBTEXT, (WPARAM)nCount, (LPARAM)szBuffer);
//     if (nCount == CB_ERR)
//         return 0;
//
//     return _ttoi(szBuffer);
// }
//
// INT_PTR __stdcall
// AdvAppearanceDlgProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     INT iSelection, iProperty;
//     GLOBALS* g;
//     PLOGFONTW plfFont;
//
//     g = (GLOBALS*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             g = (GLOBALS*)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)g);
//             AdvAppearanceDlg_Init(hwndDlg, g);
//             break;
//
//         case WM_DESTROY:
//             AdvAppearanceDlg_CleanUp(hwndDlg, g);
//             break;
//
//         case WM_COMMAND:
//             if (g == NULL)
//                 return TRUE;
//
//             switch(LOWORD(wParam))
//             {
//                 case IDOK:
//                     SaveCurrentValues(hwndDlg, g);
//                     EndDialog(hwndDlg, IDOK);
//                     break;
//
//                 case IDCANCEL:
//                     g->SchemeAdv = g->Scheme;
//                     EndDialog(hwndDlg, IDCANCEL);
//                     break;
//
//                 case IDC_APPEARANCE_PREVIEW:
//                     SaveCurrentValues(hwndDlg, g);
//                     SelectComboByElement(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT, lParam);
//                     g->CurrentElement = (INT)lParam;
//                     UpdateControls(hwndDlg, g);
//                     break;
//
//                 case IDC_ADVAPPEARANCE_ELEMENT:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         SaveCurrentValues(hwndDlg, g);
//                         iSelection = SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT as c_int, CB_GETCURSEL, 0 as WPARAM, 0 as LPARAM);
//                         g->CurrentElement = SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_ELEMENT as c_int, CB_GETITEMDATA, iSelection as WPARAM, 0 as LPARAM);
//                         UpdateControls(hwndDlg, g);
//                     }
//                     break;
//
//                 case IDC_ADVAPPEARANCE_SIZE_E:
//                     if (HIWORD(wParam) == EN_CHANGE)
//                     {
//                         iProperty = g_Assignment[g->CurrentElement].Size;
//                         if (iProperty == -1)
//                             return TRUE;
//
//                         iSelection = LOWORD(SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_SIZE_UD, UDM_GETPOS,0,0));
//                         SchemeSetMetric(&g->SchemeAdv, iProperty, iSelection);
//                         SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW as c_int, PVM_SETSIZE, iProperty as WPARAM, iSelection as LPARAM);
//
//                         iProperty = g_Assignment[g->CurrentElement].Size2;
//                         if (iProperty != -1)
//                             SchemeSetMetric(&g->SchemeAdv, iProperty, iSelection);
//                     }
//                     break;
//
//                 case IDC_ADVAPPEARANCE_FONT_C:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         iProperty = g_Assignment[g->CurrentElement].Font;
//                         if (iProperty == -1)
//                             return TRUE;
//
//                         plfFont = SchemeGetFont(&g->SchemeAdv, iProperty);
//                         GetSelectedComboText(hwndDlg, IDC_ADVAPPEARANCE_FONT_C, plfFont->lfFaceName);
//                         SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_SETFONT, iProperty, (LPARAM)plfFont);
//                     }
//                     break;
//
//                 case IDC_ADVAPPEARANCE_FONTSIZE_E:
//                     if ((HIWORD(wParam) == CBN_SELCHANGE || HIWORD(wParam) == CBN_EDITCHANGE))
//                     {
//                         HDC hdcDlg;
//
//                         iProperty =  g_Assignment[g->CurrentElement].Font;
//                         if (iProperty == -1)
//                             return TRUE;
//
//                         hdcDlg = GetDC(hwndDlg);
//
//                         plfFont = SchemeGetFont(&g->SchemeAdv, iProperty);
//                         iSelection = GetSelectedComboInt(hwndDlg, IDC_ADVAPPEARANCE_FONTSIZE_E);
//                         plfFont->lfHeight = -MulDiv(iSelection , GetDeviceCaps(hdcDlg, LOGPIXELSY), 72);
//                         SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_UPDATETHEME, 0, (LPARAM)&g->SchemeAdv);
//
//                         ReleaseDC(hwndDlg, hdcDlg);
//                     }
//                     break;
//
//                 case IDC_ADVAPPEARANCE_FONTBOLD:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         iProperty = g_Assignment[g->CurrentElement].Font;
//                         if (iProperty == -1)
//                             return TRUE;
//
//                         plfFont = SchemeGetFont(&g->SchemeAdv, iProperty);
//                         iSelection = (INT)SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTBOLD, BM_GETCHECK, 0, 0);
//                         plfFont->lfWeight = (iSelection == BST_CHECKED) ? FW_BOLD : FW_NORMAL;
//                         SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_SETFONT, iProperty, (LPARAM)plfFont);
//                     }
//                     break;
//
//                 case IDC_ADVAPPEARANCE_FONTITALIC:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         iProperty = g_Assignment[g->CurrentElement].Font;
//                         if (iProperty == -1)
//                             return TRUE;
//
//                         plfFont = SchemeGetFont(&g->SchemeAdv, iProperty);
//                         iSelection = (INT)SendDlgItemMessage(hwndDlg, IDC_ADVAPPEARANCE_FONTITALIC, BM_GETCHECK, 0, 0);
//                         plfFont->lfItalic = (iSelection == BST_CHECKED) ? TRUE : FALSE;
//                         SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_SETFONT, iProperty, (LPARAM)plfFont);
//                     }
//                     break;
//
//                 case IDC_ADVAPPEARANCE_COLOR1_B:
//                     GetColor(hwndDlg, g, 0);
//                     break;
//
//                 case IDC_ADVAPPEARANCE_COLOR2_B:
//                     GetColor(hwndDlg, g, 1);
//                     break;
//
//                 case IDC_ADVAPPEARANCE_FONTCOLOR_B:
//                     GetColor(hwndDlg, g, 2);
//                     break;
//
//                 default:
//                     return FALSE;
//             }
//             break;
//
//         default:
//             return FALSE;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

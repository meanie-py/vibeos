// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static szPreviewWndClass: [TCHAR; 17] = [
    b'P' as TCHAR, b'r' as TCHAR, b'e' as TCHAR, b'v' as TCHAR,
    b'i' as TCHAR, b'e' as TCHAR, b'w' as TCHAR, b'W' as TCHAR,
    b'n' as TCHAR, b'd' as TCHAR, b'C' as TCHAR, b'l' as TCHAR,
    b'a' as TCHAR, b's' as TCHAR, b's' as TCHAR, 0 as TCHAR,
    0 as TCHAR,
];

#[repr(C)]
pub struct PREVIEW_DATA {
    pub hdcPreview: HDC,
    pub hwndParent: HWND,
    pub Scheme: COLOR_SCHEME,
    pub hbrScrollbar: HBRUSH,
    pub hbrDesktop: HBRUSH,
    pub hbrWindow: HBRUSH,
    pub cxEdge: INT,
    pub cyEdge: INT,
    pub cySizeFrame: INT,
    pub cyCaption: INT,
    pub cyBorder: INT,
    pub cyMenu: INT,
    pub cxScrollbar: INT,
    pub rcDesktop: RECT,
    pub rcInactiveFrame: RECT,
    pub rcInactiveCaption: RECT,
    pub rcInactiveCaptionButtons: RECT,
    pub rcActiveFrame: RECT,
    pub rcActiveCaption: RECT,
    pub rcActiveCaptionButtons: RECT,
    pub rcActiveMenuBar: RECT,
    pub rcSelectedMenuItem: RECT,
    pub rcActiveClient: RECT,
    pub rcActiveScroll: RECT,
    pub rcDialogFrame: RECT,
    pub rcDialogCaption: RECT,
    pub rcDialogCaptionButtons: RECT,
    pub rcDialogClient: RECT,
    pub rcDialogButton: RECT,
    pub lpInAct: LPTSTR,
    pub lpAct: LPTSTR,
    pub lpWinTxt: LPTSTR,
    pub lpMessBox: LPTSTR,
    pub lpMessText: LPTSTR,
    pub lpButText: LPTSTR,
    pub hCaptionFont: HFONT,
    pub hMenuFont: HFONT,
    pub hMessageFont: HFONT,
    pub hClientFont: HFONT,
    pub hMenu: HMENU,
}

pub type PPREVIEW_DATA = *mut PREVIEW_DATA;
unsafe extern "C" fn UpdatePreviewTheme(hwnd: HWND, pPreviewData: *mut PREVIEW_DATA, scheme: *mut COLOR_SCHEME) {
    if (*pPreviewData).hbrScrollbar != core::ptr::null_mut() {
        DeleteObject(((*pPreviewData).hbrScrollbar).cast());
    }
    (*pPreviewData).hbrScrollbar = CreateSolidBrush((*scheme).crColor[(COLOR_SCROLLBAR) as usize]);
    if (*pPreviewData).hbrDesktop != core::ptr::null_mut() {
        DeleteObject(((*pPreviewData).hbrDesktop).cast());
    }

    (*pPreviewData).hbrDesktop = CreateSolidBrush((*scheme).crColor[(COLOR_DESKTOP) as usize]);
    if (*pPreviewData).hbrWindow != core::ptr::null_mut() {
        DeleteObject(((*pPreviewData).hbrWindow).cast());
    }
    (*pPreviewData).hbrWindow = CreateSolidBrush((*scheme).crColor[(COLOR_WINDOW) as usize]);

    (*pPreviewData).cxEdge = 2;
    (*pPreviewData).cyEdge = 2;

    (*pPreviewData).cySizeFrame = (*scheme).ncMetrics.iBorderWidth;

    (*pPreviewData).cyCaption = (*scheme).ncMetrics.iCaptionHeight + 1;
    (*pPreviewData).cyMenu = (*scheme).ncMetrics.iMenuHeight - 1;
    (*pPreviewData).cxScrollbar = (*scheme).ncMetrics.iScrollWidth;
    (*pPreviewData).cyBorder = (*scheme).ncMetrics.iBorderWidth;

    if (*pPreviewData).hCaptionFont != core::ptr::null_mut() {
        DeleteObject(((*pPreviewData).hCaptionFont).cast());
    }
    (*pPreviewData).hCaptionFont = CreateFontIndirect(core::ptr::addr_of_mut!((*scheme).ncMetrics.lfCaptionFont));

    if (*pPreviewData).hMenuFont != core::ptr::null_mut() {
        DeleteObject(((*pPreviewData).hMenuFont).cast());
    }
    (*pPreviewData).hMenuFont = CreateFontIndirect(core::ptr::addr_of_mut!((*scheme).ncMetrics.lfMenuFont));

    if (*pPreviewData).hMessageFont != core::ptr::null_mut() {
        DeleteObject(((*pPreviewData).hMessageFont).cast());
    }
    (*pPreviewData).hMessageFont = CreateFontIndirect(core::ptr::addr_of_mut!((*scheme).ncMetrics.lfMessageFont));

    (*pPreviewData).Scheme = *scheme;
    InvalidateRect(hwnd, core::ptr::null_mut(), (FALSE) as BOOL);
}
unsafe extern "C" fn OnCreate(hwnd: HWND, pPreviewData: *mut PREVIEW_DATA) {
    let mut scheme: *mut COLOR_SCHEME;

    (*pPreviewData).hClientFont = GetStockObject((DEFAULT_GUI_FONT) as i32) as HFONT;

    (*pPreviewData).hMenu = LoadMenu(hApplet, MAKEINTRESOURCE!(IDR_PREVIEW_MENU));
    EnableMenuItem((*pPreviewData).hMenu,
                   1, MF_BYPOSITION | MF_GRAYED);
    HiliteMenuItem(hwnd, (*pPreviewData).hMenu,
                   2, MF_BYPOSITION | MF_HILITE);

    AllocAndLoadString(core::ptr::addr_of_mut!((*pPreviewData).lpInAct), hApplet, IDS_INACTWIN);
    AllocAndLoadString(core::ptr::addr_of_mut!((*pPreviewData).lpAct), hApplet, IDS_ACTWIN);
    AllocAndLoadString(core::ptr::addr_of_mut!((*pPreviewData).lpWinTxt), hApplet, IDS_WINTEXT);
    AllocAndLoadString(core::ptr::addr_of_mut!((*pPreviewData).lpMessBox), hApplet, IDS_MESSBOX);
    AllocAndLoadString(core::ptr::addr_of_mut!((*pPreviewData).lpMessText), hApplet, IDS_MESSTEXT);
    AllocAndLoadString(core::ptr::addr_of_mut!((*pPreviewData).lpButText), hApplet, IDS_BUTTEXT);

    scheme = core::ptr::addr_of_mut!((*pPreviewData).Scheme);
    LoadCurrentScheme(scheme);

    UpdatePreviewTheme(hwnd, pPreviewData, scheme);
}
unsafe extern "C" fn CalculateItemSize(pPreviewData: *mut PREVIEW_DATA) {
    let mut width: i32;
    let mut height: i32;

    (*pPreviewData).rcInactiveFrame.left = (*pPreviewData).rcDesktop.left + 8;
    (*pPreviewData).rcInactiveFrame.top = (*pPreviewData).rcDesktop.top + 8;
    (*pPreviewData).rcInactiveFrame.right = (*pPreviewData).rcDesktop.right - 25;
    (*pPreviewData).rcInactiveFrame.bottom = (*pPreviewData).rcDesktop.bottom - 30;

    (*pPreviewData).rcInactiveCaption.left = (*pPreviewData).rcInactiveFrame.left + (*pPreviewData).cxEdge + (*pPreviewData).cySizeFrame + 1;
    (*pPreviewData).rcInactiveCaption.top = (*pPreviewData).rcInactiveFrame.top + (*pPreviewData).cyEdge + (*pPreviewData).cySizeFrame + 1;
    (*pPreviewData).rcInactiveCaption.right = (*pPreviewData).rcInactiveFrame.right - (*pPreviewData).cxEdge - (*pPreviewData).cySizeFrame - 1;
    (*pPreviewData).rcInactiveCaption.bottom = (*pPreviewData).rcInactiveCaption.top + (*pPreviewData).cyCaption - (*pPreviewData).cyBorder;

    (*pPreviewData).rcInactiveCaptionButtons.left = (*pPreviewData).rcInactiveCaption.right - 2 - 2 - 3 * 16;
    (*pPreviewData).rcInactiveCaptionButtons.top = (*pPreviewData).rcInactiveCaption.top + 2;
    (*pPreviewData).rcInactiveCaptionButtons.right = (*pPreviewData).rcInactiveCaption.right - 2;
    (*pPreviewData).rcInactiveCaptionButtons.bottom = (*pPreviewData).rcInactiveCaption.bottom - 2;

    (*pPreviewData).rcActiveFrame.left = (*pPreviewData).rcInactiveFrame.left + 3 + (*pPreviewData).cySizeFrame;
    (*pPreviewData).rcActiveFrame.top = (*pPreviewData).rcInactiveCaption.bottom + 1;
    (*pPreviewData).rcActiveFrame.right = (*pPreviewData).rcDesktop.right - 10;
    (*pPreviewData).rcActiveFrame.bottom = (*pPreviewData).rcDesktop.bottom - 25;

    (*pPreviewData).rcActiveCaption.left = (*pPreviewData).rcActiveFrame.left + (*pPreviewData).cxEdge + (*pPreviewData).cySizeFrame + 1;
    (*pPreviewData).rcActiveCaption.top = (*pPreviewData).rcActiveFrame.top + (*pPreviewData).cxEdge + (*pPreviewData).cySizeFrame + 1;
    (*pPreviewData).rcActiveCaption.right = (*pPreviewData).rcActiveFrame.right - (*pPreviewData).cxEdge - (*pPreviewData).cySizeFrame - 1;
    (*pPreviewData).rcActiveCaption.bottom = (*pPreviewData).rcActiveCaption.top + (*pPreviewData).cyCaption - (*pPreviewData).cyBorder;

    (*pPreviewData).rcActiveCaptionButtons.left = (*pPreviewData).rcActiveCaption.right - 2 - 2 - 3 * 16;
    (*pPreviewData).rcActiveCaptionButtons.top = (*pPreviewData).rcActiveCaption.top + 2;
    (*pPreviewData).rcActiveCaptionButtons.right = (*pPreviewData).rcActiveCaption.right - 2;
    (*pPreviewData).rcActiveCaptionButtons.bottom = (*pPreviewData).rcActiveCaption.bottom - 2;

    (*pPreviewData).rcActiveMenuBar.left = (*pPreviewData).rcActiveFrame.left + (*pPreviewData).cxEdge + (*pPreviewData).cySizeFrame + 1;
    (*pPreviewData).rcActiveMenuBar.top = (*pPreviewData).rcActiveCaption.bottom + 1;
    (*pPreviewData).rcActiveMenuBar.right = (*pPreviewData).rcActiveFrame.right - (*pPreviewData).cxEdge - (*pPreviewData).cySizeFrame - 1;
    (*pPreviewData).rcActiveMenuBar.bottom = (*pPreviewData).rcActiveMenuBar.top + (*pPreviewData).cyMenu + 1;

    (*pPreviewData).rcActiveClient.left = (*pPreviewData).rcActiveFrame.left + (*pPreviewData).cxEdge + (*pPreviewData).cySizeFrame + 1;
    (*pPreviewData).rcActiveClient.top = (*pPreviewData).rcActiveMenuBar.bottom;
    (*pPreviewData).rcActiveClient.right = (*pPreviewData).rcActiveFrame.right - (*pPreviewData).cxEdge - (*pPreviewData).cySizeFrame - 1;
    (*pPreviewData).rcActiveClient.bottom = (*pPreviewData).rcActiveFrame.bottom - (*pPreviewData).cyEdge - (*pPreviewData).cySizeFrame - 1;

    (*pPreviewData).rcActiveScroll.left = (*pPreviewData).rcActiveClient.right - 2 - (*pPreviewData).cxScrollbar;
    (*pPreviewData).rcActiveScroll.top = (*pPreviewData).rcActiveClient.top + 2;
    (*pPreviewData).rcActiveScroll.right = (*pPreviewData).rcActiveClient.right - 2;
    (*pPreviewData).rcActiveScroll.bottom = (*pPreviewData).rcActiveClient.bottom - 2;

    (*pPreviewData).rcDialogFrame.left = (*pPreviewData).rcActiveClient.left + 4;
    (*pPreviewData).rcDialogFrame.top = ((*pPreviewData).rcDesktop.bottom * 60) / 100;
    (*pPreviewData).rcDialogFrame.right = ((*pPreviewData).rcDesktop.right * 65) / 100;
    (*pPreviewData).rcDialogFrame.bottom = (*pPreviewData).rcDesktop.bottom - 5;

    (*pPreviewData).rcDialogCaption.left = (*pPreviewData).rcDialogFrame.left + 3;
    (*pPreviewData).rcDialogCaption.top = (*pPreviewData).rcDialogFrame.top + 3;
    (*pPreviewData).rcDialogCaption.right = (*pPreviewData).rcDialogFrame.right - 3;
    (*pPreviewData).rcDialogCaption.bottom = (*pPreviewData).rcDialogFrame.top + (*pPreviewData).cyCaption + 1 + 1;

    (*pPreviewData).rcDialogCaptionButtons.left = (*pPreviewData).rcDialogCaption.right - 2 - 16;
    (*pPreviewData).rcDialogCaptionButtons.top = (*pPreviewData).rcDialogCaption.top + 2;
    (*pPreviewData).rcDialogCaptionButtons.right = (*pPreviewData).rcDialogCaption.right - 2;
    (*pPreviewData).rcDialogCaptionButtons.bottom = (*pPreviewData).rcDialogCaption.bottom - 2;

    (*pPreviewData).rcDialogClient.left = (*pPreviewData).rcDialogFrame.left + 3;
    (*pPreviewData).rcDialogClient.top = (*pPreviewData).rcDialogCaption.bottom + 1;
    (*pPreviewData).rcDialogClient.right = (*pPreviewData).rcDialogFrame.right - 3;
    (*pPreviewData).rcDialogClient.bottom = (*pPreviewData).rcDialogFrame.bottom - 3;

    width = 80;
    height = 28;

    (*pPreviewData).rcDialogButton.left =
        ((*pPreviewData).rcDialogClient.right + (*pPreviewData).rcDialogClient.left - width) / 2;
    (*pPreviewData).rcDialogButton.right = (*pPreviewData).rcDialogButton.left + width;
    (*pPreviewData).rcDialogButton.bottom = (*pPreviewData).rcDialogClient.bottom - 2;
    (*pPreviewData).rcDialogButton.top = (*pPreviewData).rcDialogButton.bottom - height;
}
unsafe extern "C" fn OnSize(cx: i32, cy: i32, pPreviewData: *mut PREVIEW_DATA) {
    (*pPreviewData).rcDesktop.left = 0;
    (*pPreviewData).rcDesktop.top = 0;
    (*pPreviewData).rcDesktop.right = cx;
    (*pPreviewData).rcDesktop.bottom = cy;

    CalculateItemSize(pPreviewData);
}
unsafe extern "C" fn OnPaint(hwnd: HWND, pPreviewData: *mut PREVIEW_DATA) {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hOldFont: HFONT;
    let hdc: HDC;
    let mut rc: RECT = core::mem::zeroed();
    let scheme: *mut COLOR_SCHEME;

    scheme = core::ptr::addr_of_mut!((*pPreviewData).Scheme);

    hdc = BeginPaint(hwnd, &mut ps);

    if !(*pPreviewData).hdcPreview.is_null() {
        BitBlt(hdc, 0, 0, (*pPreviewData).rcDesktop.right, (*pPreviewData).rcDesktop.bottom, (*pPreviewData).hdcPreview, 0, 0, SRCCOPY);
        EndPaint(hwnd, &mut ps);
        return;
    }

    FillRect(hdc, &(*pPreviewData).rcDesktop, (*pPreviewData).hbrDesktop);

    MyDrawEdge(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcInactiveFrame), EDGE_RAISED, BF_RECT | BF_MIDDLE | MY_BF_INACTIVEBORDER, scheme);
    SetTextColor(hdc, (*scheme).crColor[(COLOR_INACTIVECAPTIONTEXT) as usize]);
    MyDrawCaptionTemp(core::ptr::null_mut(), hdc, &(*pPreviewData).rcInactiveCaption, (*pPreviewData).hCaptionFont,
                      core::ptr::null_mut(), (*pPreviewData).lpInAct, DC_GRADIENT | DC_ICON | DC_TEXT, scheme);
    MyDrawCaptionButtons(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcInactiveCaption), (TRUE) as BOOL, (*pPreviewData).cyCaption - 2, scheme);

    MyDrawEdge(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcActiveFrame), EDGE_RAISED, BF_RECT | BF_MIDDLE | MY_BF_ACTIVEBORDER, scheme);
    SetTextColor(hdc, (*scheme).crColor[(COLOR_CAPTIONTEXT) as usize]);
    MyDrawCaptionTemp(core::ptr::null_mut(), hdc, &(*pPreviewData).rcActiveCaption, (*pPreviewData).hCaptionFont,
                      core::ptr::null_mut(), (*pPreviewData).lpAct, DC_ACTIVE | DC_GRADIENT | DC_ICON | DC_TEXT, scheme);
    MyDrawCaptionButtons(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcActiveCaption), (TRUE) as BOOL, (*pPreviewData).cyCaption - 2, scheme);

    MyDrawMenuBarTemp(hwnd, hdc, core::ptr::addr_of_mut!((*pPreviewData).rcActiveMenuBar),
                      (*pPreviewData).hMenu,
                      (*pPreviewData).hMenuFont, scheme);

    CopyRect(&mut rc, &(*pPreviewData).rcActiveClient);
    MyDrawEdge(hdc, &mut rc, EDGE_SUNKEN, BF_RECT | BF_ADJUST, scheme);
    FillRect(hdc, &mut rc, (*pPreviewData).hbrWindow);

    CopyRect(&mut rc, &(*pPreviewData).rcActiveClient);
    rc.left += 4;
    rc.top += 2;
    SetTextColor(hdc, (*scheme).crColor[(COLOR_WINDOWTEXT) as usize]);
    hOldFont = (SelectObject(hdc, ((*pPreviewData).hClientFont).cast())).cast();
    DrawText(hdc, (*pPreviewData).lpWinTxt, -1, &mut rc, DT_LEFT);
    SelectObject(hdc, (hOldFont).cast());

    MyDrawScrollbar(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcActiveScroll), (*pPreviewData).hbrScrollbar, scheme);

    MyDrawEdge(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcDialogFrame), EDGE_RAISED, BF_RECT | BF_MIDDLE, scheme);
    SetTextColor(hdc, (*scheme).crColor[(COLOR_WINDOW) as usize]);
    MyDrawCaptionTemp(core::ptr::null_mut(), hdc, &(*pPreviewData).rcDialogCaption, (*pPreviewData).hCaptionFont,
                      core::ptr::null_mut(), (*pPreviewData).lpMessBox, DC_ACTIVE | DC_GRADIENT | DC_ICON | DC_TEXT, scheme);
    MyDrawCaptionButtons(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcDialogCaption), (FALSE) as BOOL, (*pPreviewData).cyCaption - 2, scheme);

    CopyRect(&mut rc, &(*pPreviewData).rcDialogClient);
    rc.left += 4;
    rc.top += 2;
    SetTextColor(hdc, (*scheme).crColor[(COLOR_WINDOWTEXT) as usize]);
    hOldFont = (SelectObject(hdc, ((*pPreviewData).hMessageFont).cast())).cast();
    DrawText(hdc, (*pPreviewData).lpMessText, -1, &mut rc, DT_LEFT);
    SelectObject(hdc, (hOldFont).cast());

    MyDrawFrameControl(hdc, core::ptr::addr_of_mut!((*pPreviewData).rcDialogButton), DFC_BUTTON, DFCS_BUTTONPUSH, scheme);
    CopyRect(&mut rc, &(*pPreviewData).rcDialogButton);
    SetTextColor(hdc, (*scheme).crColor[(COLOR_BTNTEXT) as usize]);
    hOldFont = (SelectObject(hdc, ((*pPreviewData).hMessageFont).cast())).cast();
    DrawText(hdc, (*pPreviewData).lpButText, -1, &mut rc, DT_VCENTER | DT_CENTER | DT_SINGLELINE);
    SelectObject(hdc, (hOldFont).cast());

    EndPaint(hwnd, &mut ps);
}
unsafe extern "C" fn OnLButtonDown(hwnd: HWND, xPos: i32, yPos: i32, pPreviewData: *mut PREVIEW_DATA) {
    let mut type_: u32 = IDX_DESKTOP;
    let mut pt: POINT = core::mem::zeroed();

    pt.x = xPos;
    pt.y = yPos;

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcInactiveFrame), pt) != 0 {
        type_ = IDX_INACTIVE_BORDER;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcInactiveCaption), pt) != 0 {
        type_ = IDX_INACTIVE_CAPTION;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcInactiveCaptionButtons), pt) != 0 {
        type_ = IDX_CAPTION_BUTTON;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcActiveFrame), pt) != 0 {
        type_ = IDX_ACTIVE_BORDER;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcActiveCaption), pt) != 0 {
        type_ = IDX_ACTIVE_CAPTION;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcActiveCaptionButtons), pt) != 0 {
        type_ = IDX_CAPTION_BUTTON;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcActiveMenuBar), pt) != 0 {
        type_ = IDX_MENU;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcActiveClient), pt) != 0 {
        type_ = IDX_WINDOW;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcActiveScroll), pt) != 0 {
        type_ = IDX_SCROLLBAR;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcDialogFrame), pt) != 0 {
        type_ = IDX_DIALOG;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcDialogCaption), pt) != 0 {
        type_ = IDX_ACTIVE_CAPTION;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcDialogCaptionButtons), pt) != 0 {
        type_ = IDX_CAPTION_BUTTON;
    }

    if PtInRect(core::ptr::addr_of_mut!((*pPreviewData).rcDialogButton), pt) != 0 {
        type_ = IDX_3D_OBJECTS;
    }

    SendMessage(GetParent(hwnd),
                WM_COMMAND,
                MAKEWPARAM!(GetWindowLongPtrW(hwnd, GWLP_ID), 0),
                type_ as LPARAM);
}
unsafe extern "C" fn OnDestroy(pPreviewData: *mut PREVIEW_DATA) {
    DeleteObject(((*pPreviewData).hbrScrollbar).cast());
    DeleteObject(((*pPreviewData).hbrDesktop).cast());
    DeleteObject(((*pPreviewData).hbrWindow).cast());

    DeleteObject(((*pPreviewData).hCaptionFont).cast());
    DeleteObject(((*pPreviewData).hMenuFont).cast());
    DeleteObject(((*pPreviewData).hMessageFont).cast());

    DestroyMenu((*pPreviewData).hMenu);

    LocalFree((*pPreviewData).lpInAct as HLOCAL);
    LocalFree((*pPreviewData).lpAct as HLOCAL);
    LocalFree((*pPreviewData).lpWinTxt as HLOCAL);
    LocalFree((*pPreviewData).lpMessBox as HLOCAL);
    LocalFree((*pPreviewData).lpMessText as HLOCAL);
    LocalFree((*pPreviewData).lpButText as HLOCAL);
}
unsafe extern "system" fn PreviewWndProc(
    hwnd: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut pPreviewData: *mut PREVIEW_DATA;

    pPreviewData = GetWindowLongPtr(hwnd, GWLP_USERDATA) as *mut PREVIEW_DATA;

    match (uMsg) as u32 {
        WM_CREATE => {
            pPreviewData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<PREVIEW_DATA>()) as SIZE_T,
            ) as *mut PREVIEW_DATA;
            if pPreviewData.is_null() {
                return -1;
            }

            SetWindowLongPtr(hwnd, GWLP_USERDATA, ((pPreviewData as isize) as i32) as LPARAM);
            OnCreate(hwnd, pPreviewData);
        }

        WM_SIZE => {
            OnSize((LOWORD!(lParam)) as i32, (HIWORD!(lParam)) as i32, pPreviewData);
        }

        WM_PAINT => {
            OnPaint(hwnd, pPreviewData);
        }

        WM_LBUTTONDOWN => {
            OnLButtonDown(hwnd, (LOWORD!(lParam)) as i32, (HIWORD!(lParam)) as i32, pPreviewData);
        }

        WM_DESTROY => {
            OnDestroy(pPreviewData);
            HeapFree(
                GetProcessHeap(),
                0,
                pPreviewData as *mut _,
            );
        }

        PVM_UPDATETHEME => {
            UpdatePreviewTheme(hwnd, pPreviewData, lParam as *mut COLOR_SCHEME);
            CalculateItemSize(pPreviewData);
            InvalidateRect(hwnd, core::ptr::null_mut(), (FALSE) as BOOL);
        }

        PVM_SETSIZE => {
            SchemeSetMetric(
                core::ptr::addr_of_mut!((*pPreviewData).Scheme),
                (wParam) as i32,
                (lParam) as i32,
            );
            (*pPreviewData).cySizeFrame = (*pPreviewData).Scheme.ncMetrics.iBorderWidth;
            (*pPreviewData).cyCaption =
                (*pPreviewData).Scheme.ncMetrics.iCaptionHeight + 1;
            (*pPreviewData).cyMenu = (*pPreviewData).Scheme.ncMetrics.iMenuHeight - 1;
            (*pPreviewData).cxScrollbar = (*pPreviewData).Scheme.ncMetrics.iScrollWidth;
            (*pPreviewData).cyBorder = (*pPreviewData).Scheme.ncMetrics.iBorderWidth;
            CalculateItemSize(pPreviewData);
            InvalidateRect(hwnd, core::ptr::null_mut(), (FALSE) as BOOL);
        }

        PVM_SETFONT => {
            let plfFont: *mut LOGFONTW;
            let phFont: *mut HFONT;

            phFont = match wParam as UINT {
                FONT_CAPTION => core::ptr::addr_of_mut!((*pPreviewData).hCaptionFont),
                FONT_MENU => core::ptr::addr_of_mut!((*pPreviewData).hMenuFont),
                FONT_MESSAGE => core::ptr::addr_of_mut!((*pPreviewData).hMessageFont),
                _ => return (TRUE) as LPARAM,
            };

            plfFont = SchemeGetFont(
                core::ptr::addr_of_mut!((*pPreviewData).Scheme),
                (wParam) as i32,
            );
            memcpy(
                plfFont as *mut _,
                lParam as *mut _,
                core::mem::size_of::<LOGFONTW>(),
            );

            DeleteObject((*phFont).cast());
            *phFont = CreateFontIndirect(plfFont);

            InvalidateRect(hwnd, core::ptr::null_mut(), (FALSE) as BOOL);
        }

        PVM_SETCOLOR => {
            (*pPreviewData).Scheme.crColor[wParam as usize] = lParam as u32;
            match (wParam) as u32 {
                COLOR_SCROLLBAR => {
                    DeleteObject(((*pPreviewData).hbrScrollbar).cast());
                    (*pPreviewData).hbrScrollbar = CreateSolidBrush(
                        (*pPreviewData).Scheme.crColor[wParam as usize],
                    );
                }
                COLOR_DESKTOP => {
                    DeleteObject(((*pPreviewData).hbrDesktop).cast());
                    (*pPreviewData).hbrDesktop =
                        CreateSolidBrush((*pPreviewData).Scheme.crColor[wParam as usize]);
                }
                COLOR_WINDOW => {
                    DeleteObject(((*pPreviewData).hbrWindow).cast());
                    (*pPreviewData).hbrWindow =
                        CreateSolidBrush((*pPreviewData).Scheme.crColor[wParam as usize]);
                }
                _ => {}
            }

            CalculateItemSize(pPreviewData);
            InvalidateRect(hwnd, core::ptr::null_mut(), (FALSE) as BOOL);
        }

        PVM_GETSIZE => {
            return (((SchemeGetMetric(
                core::ptr::addr_of_mut!((*pPreviewData).Scheme),
                (wParam) as i32,
            )) as i64) as i32) as LPARAM;
        }

        PVM_GETFONT => {
            return SchemeGetFont(
                core::ptr::addr_of_mut!((*pPreviewData).Scheme),
                (wParam) as i32,
            ) as LRESULT;
        }

        PVM_GETCOLOR => {
            return (*pPreviewData).Scheme.crColor[wParam as usize] as LRESULT;
        }

        PVM_SET_HDC_PREVIEW => {
            (*pPreviewData).hdcPreview = lParam as HDC;
            InvalidateRect(hwnd, core::ptr::null_mut(), (FALSE) as BOOL);
        }

        _ => {
            return DefWindowProc(hwnd, uMsg, wParam, lParam);
        }
    }

    (FALSE) as LPARAM
}
#[no_mangle]
pub unsafe extern "C" fn RegisterPreviewControl(hInstance: HINSTANCE) -> u32 {
    let mut wc: WNDCLASSEX = core::mem::zeroed();

    wc.cbSize = core::mem::size_of::<WNDCLASSEX>() as u32;
    wc.lpfnWndProc = Some(core::mem::transmute(PreviewWndProc as *const ()));
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
    wc.hbrBackground = core::ptr::null_mut();
    wc.lpszClassName = (szPreviewWndClass.as_ptr()).cast();

    (RegisterClassEx(&mut wc) != 0 as ATOM) as u32
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterPreviewControl(hInstance: HINSTANCE) {
    UnregisterClass(szPreviewWndClass.as_ptr(), hInstance);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Desktop Control Panel
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/desk/preview.c
//  * PURPOSE:     Draws the preview control
//  * COPYRIGHT:   Copyright 2006, 2007 Eric Kohl
//  */
//
// #include "desk.h"
//
// static const TCHAR szPreviewWndClass[] = TEXT("PreviewWndClass");
//
// typedef struct _PREVIEW_DATA
// {
//     HDC hdcPreview;
//
//     HWND hwndParent;
//
//     COLOR_SCHEME Scheme;
//
//     HBRUSH hbrScrollbar;
//     HBRUSH hbrDesktop;
//     HBRUSH hbrWindow;
//
//     INT cxEdge;
//     INT cyEdge;
//
//     INT cySizeFrame;
//
//     INT cyCaption;
//     INT cyBorder;
//     INT cyMenu;
//     INT cxScrollbar;
//
//     RECT rcDesktop;
//     RECT rcInactiveFrame;
//     RECT rcInactiveCaption;
//     RECT rcInactiveCaptionButtons;
//
//     RECT rcActiveFrame;
//     RECT rcActiveCaption;
//     RECT rcActiveCaptionButtons;
//     RECT rcActiveMenuBar;
//     RECT rcSelectedMenuItem;
//     RECT rcActiveClient;
//     RECT rcActiveScroll;
//
//     RECT rcDialogFrame;
//     RECT rcDialogCaption;
//     RECT rcDialogCaptionButtons;
//     RECT rcDialogClient;
//
//     RECT rcDialogButton;
//
//     LPTSTR lpInAct;
//     LPTSTR lpAct;
//     LPTSTR lpWinTxt;
//     LPTSTR lpMessBox;
//     LPTSTR lpMessText;
//     LPTSTR lpButText;
//
//     HFONT hCaptionFont;
//     HFONT hMenuFont;
//     HFONT hMessageFont;
//     HFONT hClientFont;
//
//     HMENU hMenu;
//
// } PREVIEW_DATA, *PPREVIEW_DATA;
//
//
// static VOID UpdatePreviewTheme(HWND hwnd, PPREVIEW_DATA pPreviewData, COLOR_SCHEME *scheme)
// {
//     if (pPreviewData->hbrScrollbar != NULL)
//         DeleteObject(pPreviewData->hbrScrollbar);
//     pPreviewData->hbrScrollbar = CreateSolidBrush(scheme->crColor[COLOR_SCROLLBAR]);
//     if (pPreviewData->hbrDesktop != NULL)
//         DeleteObject(pPreviewData->hbrDesktop);
//
//     pPreviewData->hbrDesktop = CreateSolidBrush(scheme->crColor[COLOR_DESKTOP]);
//     if (pPreviewData->hbrWindow != NULL)
//         DeleteObject(pPreviewData->hbrWindow);
//     pPreviewData->hbrWindow = CreateSolidBrush(scheme->crColor[COLOR_WINDOW]);
//
//     pPreviewData->cxEdge = 2;                                       /* SM_CXEDGE */
//     pPreviewData->cyEdge = 2;                                       /* SM_CYEDGE */
//
//     pPreviewData->cySizeFrame = scheme->ncMetrics.iBorderWidth;     /* SM_CYSIZEFRAME */
//
//     pPreviewData->cyCaption = scheme->ncMetrics.iCaptionHeight+1;   /* SM_CYCAPTION */
//     pPreviewData->cyMenu = scheme->ncMetrics.iMenuHeight -1;        /* SM_CYMENU */
//     pPreviewData->cxScrollbar = scheme->ncMetrics.iScrollWidth;     /* SM_CXVSCROLL */
//     pPreviewData->cyBorder = scheme->ncMetrics.iBorderWidth;        /* SM_CYBORDER */
//
//     if (pPreviewData->hCaptionFont != NULL)
//         DeleteObject(pPreviewData->hCaptionFont);
//     pPreviewData->hCaptionFont = CreateFontIndirect(&scheme->ncMetrics.lfCaptionFont);
//
//     if (pPreviewData->hMenuFont != NULL)
//         DeleteObject(pPreviewData->hMenuFont);
//     pPreviewData->hMenuFont = CreateFontIndirect(&scheme->ncMetrics.lfMenuFont);
//
//     if (pPreviewData->hMessageFont != NULL)
//         DeleteObject(pPreviewData->hMessageFont);
//     pPreviewData->hMessageFont = CreateFontIndirect(&scheme->ncMetrics.lfMessageFont);
//
//     pPreviewData->Scheme = *scheme;
//     InvalidateRect(hwnd, NULL, FALSE);
// }
//
// static VOID
// OnCreate(HWND hwnd, PPREVIEW_DATA pPreviewData)
// {
//     COLOR_SCHEME *scheme;
//
//     pPreviewData->hClientFont = (HFONT)GetStockObject(DEFAULT_GUI_FONT);
//
//     /* Load and modify the menu */
//     pPreviewData->hMenu = LoadMenu(hApplet, MAKEINTRESOURCE(IDR_PREVIEW_MENU));
//     EnableMenuItem(pPreviewData->hMenu,
//                    1, MF_BYPOSITION | MF_GRAYED);
//     HiliteMenuItem(hwnd, pPreviewData->hMenu,
//                    2, MF_BYPOSITION | MF_HILITE);
//
// //    GetMenuItemRect(hwnd, pPreviewData->hMenu,
// //                    2, &pPreviewData->rcSelectedMenuItem);
//
//     AllocAndLoadString(&pPreviewData->lpInAct, hApplet, IDS_INACTWIN);
//     AllocAndLoadString(&pPreviewData->lpAct, hApplet, IDS_ACTWIN);
//     AllocAndLoadString(&pPreviewData->lpWinTxt, hApplet, IDS_WINTEXT);
//     AllocAndLoadString(&pPreviewData->lpMessBox, hApplet, IDS_MESSBOX);
//     AllocAndLoadString(&pPreviewData->lpMessText, hApplet, IDS_MESSTEXT);
//     AllocAndLoadString(&pPreviewData->lpButText, hApplet, IDS_BUTTEXT);
//
//     scheme = &pPreviewData->Scheme;
//     LoadCurrentScheme(scheme);
//
//     UpdatePreviewTheme(hwnd, pPreviewData, scheme);
// }
//
//
// static VOID
// CalculateItemSize(PPREVIEW_DATA pPreviewData)
// {
//     int width, height;
//
//     /* Calculate the inactive window rectangle */
//     pPreviewData->rcInactiveFrame.left = pPreviewData->rcDesktop.left + 8;
//     pPreviewData->rcInactiveFrame.top = pPreviewData->rcDesktop.top + 8;
//     pPreviewData->rcInactiveFrame.right = pPreviewData->rcDesktop.right - 25;
//     pPreviewData->rcInactiveFrame.bottom = pPreviewData->rcDesktop.bottom - 30;
//
//     /* Calculate the inactive caption rectangle */
//     pPreviewData->rcInactiveCaption.left = pPreviewData->rcInactiveFrame.left + pPreviewData->cxEdge + pPreviewData->cySizeFrame + 1;
//     pPreviewData->rcInactiveCaption.top = pPreviewData->rcInactiveFrame.top + pPreviewData->cyEdge + pPreviewData->cySizeFrame + 1;
//     pPreviewData->rcInactiveCaption.right = pPreviewData->rcInactiveFrame.right - pPreviewData->cxEdge - pPreviewData->cySizeFrame - 1;
//     pPreviewData->rcInactiveCaption.bottom = pPreviewData->rcInactiveCaption.top + pPreviewData->cyCaption - pPreviewData->cyBorder;
//
//     /* Calculate the inactive caption buttons rectangle */
//     pPreviewData->rcInactiveCaptionButtons.left = pPreviewData->rcInactiveCaption.right - 2 - 2 - 3 * 16;
//     pPreviewData->rcInactiveCaptionButtons.top = pPreviewData->rcInactiveCaption.top + 2;
//     pPreviewData->rcInactiveCaptionButtons.right = pPreviewData->rcInactiveCaption.right - 2;
//     pPreviewData->rcInactiveCaptionButtons.bottom = pPreviewData->rcInactiveCaption.bottom - 2;
//
//     /* Calculate the active window rectangle */
//     pPreviewData->rcActiveFrame.left = pPreviewData->rcInactiveFrame.left + 3 + pPreviewData->cySizeFrame;
//     pPreviewData->rcActiveFrame.top = pPreviewData->rcInactiveCaption.bottom + 1;
//     pPreviewData->rcActiveFrame.right = pPreviewData->rcDesktop.right - 10;
//     pPreviewData->rcActiveFrame.bottom = pPreviewData->rcDesktop.bottom - 25;
//
//     /* Calculate the active caption rectangle */
//     pPreviewData->rcActiveCaption.left = pPreviewData->rcActiveFrame.left + pPreviewData->cxEdge + pPreviewData->cySizeFrame + 1;
//     pPreviewData->rcActiveCaption.top = pPreviewData->rcActiveFrame.top + pPreviewData->cxEdge + pPreviewData->cySizeFrame + 1;
//     pPreviewData->rcActiveCaption.right = pPreviewData->rcActiveFrame.right - pPreviewData->cxEdge - pPreviewData->cySizeFrame - 1;
//     pPreviewData->rcActiveCaption.bottom = pPreviewData->rcActiveCaption.top + pPreviewData->cyCaption - pPreviewData->cyBorder;
//
//     /* Calculate the active caption buttons rectangle */
//     pPreviewData->rcActiveCaptionButtons.left = pPreviewData->rcActiveCaption.right - 2 - 2 - 3 * 16;
//     pPreviewData->rcActiveCaptionButtons.top = pPreviewData->rcActiveCaption.top + 2;
//     pPreviewData->rcActiveCaptionButtons.right = pPreviewData->rcActiveCaption.right - 2;
//     pPreviewData->rcActiveCaptionButtons.bottom = pPreviewData->rcActiveCaption.bottom - 2;
//
//     /* Calculate the active menu bar rectangle */
//     pPreviewData->rcActiveMenuBar.left = pPreviewData->rcActiveFrame.left + pPreviewData->cxEdge + pPreviewData->cySizeFrame + 1;
//     pPreviewData->rcActiveMenuBar.top = pPreviewData->rcActiveCaption.bottom + 1;
//     pPreviewData->rcActiveMenuBar.right = pPreviewData->rcActiveFrame.right - pPreviewData->cxEdge - pPreviewData->cySizeFrame - 1;
//     pPreviewData->rcActiveMenuBar.bottom = pPreviewData->rcActiveMenuBar.top + pPreviewData->cyMenu + 1;
//
//     /* Calculate the active client rectangle */
//     pPreviewData->rcActiveClient.left = pPreviewData->rcActiveFrame.left + pPreviewData->cxEdge + pPreviewData->cySizeFrame + 1;
//     pPreviewData->rcActiveClient.top = pPreviewData->rcActiveMenuBar.bottom;
//     pPreviewData->rcActiveClient.right = pPreviewData->rcActiveFrame.right - pPreviewData->cxEdge - pPreviewData->cySizeFrame - 1;
//     pPreviewData->rcActiveClient.bottom = pPreviewData->rcActiveFrame.bottom - pPreviewData->cyEdge - pPreviewData->cySizeFrame - 1;
//
//     /* Calculate the active scroll rectangle */
//     pPreviewData->rcActiveScroll.left = pPreviewData->rcActiveClient.right - 2 - pPreviewData->cxScrollbar;
//     pPreviewData->rcActiveScroll.top = pPreviewData->rcActiveClient.top + 2;
//     pPreviewData->rcActiveScroll.right = pPreviewData->rcActiveClient.right - 2;
//     pPreviewData->rcActiveScroll.bottom = pPreviewData->rcActiveClient.bottom - 2;
//
//
//     /* Dialog window */
//     pPreviewData->rcDialogFrame.left = pPreviewData->rcActiveClient.left + 4;
//     pPreviewData->rcDialogFrame.top = (pPreviewData->rcDesktop.bottom * 60) / 100;
//     pPreviewData->rcDialogFrame.right = (pPreviewData->rcDesktop.right * 65) / 100;
//     pPreviewData->rcDialogFrame.bottom = pPreviewData->rcDesktop.bottom - 5;
//
//     /* Calculate the dialog caption rectangle */
//     pPreviewData->rcDialogCaption.left = pPreviewData->rcDialogFrame.left + 3;
//     pPreviewData->rcDialogCaption.top = pPreviewData->rcDialogFrame.top + 3;
//     pPreviewData->rcDialogCaption.right = pPreviewData->rcDialogFrame.right - 3;
//     pPreviewData->rcDialogCaption.bottom = pPreviewData->rcDialogFrame.top + pPreviewData->cyCaption + 1 + 1;
//
//     /* Calculate the inactive caption buttons rectangle */
//     pPreviewData->rcDialogCaptionButtons.left = pPreviewData->rcDialogCaption.right - 2 - 16;
//     pPreviewData->rcDialogCaptionButtons.top = pPreviewData->rcDialogCaption.top + 2;
//     pPreviewData->rcDialogCaptionButtons.right = pPreviewData->rcDialogCaption.right - 2;
//     pPreviewData->rcDialogCaptionButtons.bottom = pPreviewData->rcDialogCaption.bottom - 2;
//
//     /* Calculate the dialog client rectangle */
//     pPreviewData->rcDialogClient.left = pPreviewData->rcDialogFrame.left + 3;
//     pPreviewData->rcDialogClient.top = pPreviewData->rcDialogCaption.bottom + 1;
//     pPreviewData->rcDialogClient.right = pPreviewData->rcDialogFrame.right - 3;
//     pPreviewData->rcDialogClient.bottom = pPreviewData->rcDialogFrame.bottom - 3;
//
//     /* Calculate the dialog button rectangle */
//     width = 80;
//     height = 28;
//
//     pPreviewData->rcDialogButton.left =
//         (pPreviewData->rcDialogClient.right + pPreviewData->rcDialogClient.left - width) / 2;
//     pPreviewData->rcDialogButton.right = pPreviewData->rcDialogButton.left + width;
//     pPreviewData->rcDialogButton.bottom = pPreviewData->rcDialogClient.bottom - 2;
//     pPreviewData->rcDialogButton.top = pPreviewData->rcDialogButton.bottom - height;
// }
//
//
// static VOID
// OnSize(INT cx, INT cy, PPREVIEW_DATA pPreviewData)
// {
//     /* Get Desktop rectangle */
//     pPreviewData->rcDesktop.left = 0;
//     pPreviewData->rcDesktop.top = 0;
//     pPreviewData->rcDesktop.right = cx;
//     pPreviewData->rcDesktop.bottom = cy;
//
//     CalculateItemSize(pPreviewData);
// }
//
// static VOID
// OnPaint(HWND hwnd, PPREVIEW_DATA pPreviewData)
// {
//     PAINTSTRUCT ps;
//     HFONT hOldFont;
//     HDC hdc;
//     RECT rc;
//     COLOR_SCHEME *scheme;
//
//     scheme = &pPreviewData->Scheme;
//
//     hdc = BeginPaint(hwnd, &ps);
//
//     if (pPreviewData->hdcPreview)
//     {
//         BitBlt(hdc,0,0, pPreviewData->rcDesktop.right, pPreviewData->rcDesktop.bottom, pPreviewData->hdcPreview, 0,0, SRCCOPY);
//         EndPaint(hwnd, &ps);
//         return;
//     }
//
//     /* Desktop */
//     FillRect(hdc, &pPreviewData->rcDesktop, pPreviewData->hbrDesktop);
//
//     /* Inactive Window */
//     MyDrawEdge(hdc, &pPreviewData->rcInactiveFrame, EDGE_RAISED, BF_RECT | BF_MIDDLE | MY_BF_INACTIVEBORDER, scheme);
//     SetTextColor(hdc, scheme->crColor[COLOR_INACTIVECAPTIONTEXT]);
//     MyDrawCaptionTemp(NULL, hdc, &pPreviewData->rcInactiveCaption,  pPreviewData->hCaptionFont,
//                       NULL, pPreviewData->lpInAct, DC_GRADIENT | DC_ICON | DC_TEXT, scheme);
//     MyDrawCaptionButtons(hdc, &pPreviewData->rcInactiveCaption, TRUE, pPreviewData->cyCaption - 2, scheme);
//
//     /* Active Window */
//     MyDrawEdge(hdc, &pPreviewData->rcActiveFrame, EDGE_RAISED, BF_RECT | BF_MIDDLE | MY_BF_ACTIVEBORDER, scheme);
//     SetTextColor(hdc, scheme->crColor[COLOR_CAPTIONTEXT]);
//     MyDrawCaptionTemp(NULL, hdc, &pPreviewData->rcActiveCaption, pPreviewData->hCaptionFont,
//                       NULL, pPreviewData->lpAct, DC_ACTIVE | DC_GRADIENT | DC_ICON | DC_TEXT, scheme);
//     MyDrawCaptionButtons(hdc, &pPreviewData->rcActiveCaption, TRUE, pPreviewData->cyCaption - 2, scheme);
//
//     /* Draw the menu bar */
//     MyDrawMenuBarTemp(hwnd, hdc, &pPreviewData->rcActiveMenuBar,
//                       pPreviewData->hMenu,
//                       pPreviewData->hMenuFont, scheme);
//
//     /* Draw the client area */
//     CopyRect(&rc, &pPreviewData->rcActiveClient);
//     MyDrawEdge(hdc, &rc, EDGE_SUNKEN, BF_RECT | BF_ADJUST, scheme);
//     FillRect(hdc, &rc, pPreviewData->hbrWindow);
//
//     /* Draw the client text */
//     CopyRect(&rc, &pPreviewData->rcActiveClient);
//     rc.left += 4;
//     rc.top += 2;
//     SetTextColor(hdc, scheme->crColor[COLOR_WINDOWTEXT]);
//     hOldFont = SelectObject(hdc, pPreviewData->hClientFont);
//     DrawText(hdc, pPreviewData->lpWinTxt, -1, &rc, DT_LEFT);
//     SelectObject(hdc, hOldFont);
//
//     /* Draw the scroll bar */
//     MyDrawScrollbar(hdc, &pPreviewData->rcActiveScroll, pPreviewData->hbrScrollbar, scheme);
//
//     /* Dialog Window */
//     MyDrawEdge(hdc, &pPreviewData->rcDialogFrame, EDGE_RAISED, BF_RECT | BF_MIDDLE, scheme);
//     SetTextColor(hdc, scheme->crColor[COLOR_WINDOW]);
//     MyDrawCaptionTemp(NULL, hdc, &pPreviewData->rcDialogCaption, pPreviewData->hCaptionFont,
//                       NULL, pPreviewData->lpMessBox, DC_ACTIVE | DC_GRADIENT | DC_ICON | DC_TEXT, scheme);
//     MyDrawCaptionButtons(hdc, &pPreviewData->rcDialogCaption, FALSE, pPreviewData->cyCaption - 2, scheme);
//
//     /* Draw the dialog text */
//     CopyRect(&rc, &pPreviewData->rcDialogClient);
//     rc.left += 4;
//     rc.top += 2;
//     SetTextColor(hdc, scheme->crColor[COLOR_WINDOWTEXT]);
//     hOldFont = SelectObject(hdc, pPreviewData->hMessageFont);
//     DrawText(hdc, pPreviewData->lpMessText, -1, &rc, DT_LEFT);
//     SelectObject(hdc, hOldFont);
//
//     /* Draw Button */
//     MyDrawFrameControl(hdc, &pPreviewData->rcDialogButton, DFC_BUTTON, DFCS_BUTTONPUSH, scheme);
//     CopyRect(&rc, &pPreviewData->rcDialogButton);
//     SetTextColor(hdc, scheme->crColor[COLOR_BTNTEXT]);
//     hOldFont = SelectObject(hdc, pPreviewData->hMessageFont);
//     DrawText(hdc, pPreviewData->lpButText, -1, &rc, DT_VCENTER | DT_CENTER | DT_SINGLELINE);
//     SelectObject(hdc, hOldFont);
//
//     EndPaint(hwnd, &ps);
// }
//
//
// static VOID
// OnLButtonDown(HWND hwnd, int xPos, int yPos, PPREVIEW_DATA pPreviewData)
// {
//     UINT type = IDX_DESKTOP;
//     POINT pt;
//
//     pt.x = xPos;
//     pt.y = yPos;
//
//     if (PtInRect(&pPreviewData->rcInactiveFrame, pt))
//         type = IDX_INACTIVE_BORDER;
//
//     if (PtInRect(&pPreviewData->rcInactiveCaption, pt))
//         type = IDX_INACTIVE_CAPTION;
//
//     if (PtInRect(&pPreviewData->rcInactiveCaptionButtons, pt))
//         type = IDX_CAPTION_BUTTON;
//
//     if (PtInRect(&pPreviewData->rcActiveFrame, pt))
//         type = IDX_ACTIVE_BORDER;
//
//     if (PtInRect(&pPreviewData->rcActiveCaption, pt))
//         type = IDX_ACTIVE_CAPTION;
//
//     if (PtInRect(&pPreviewData->rcActiveCaptionButtons, pt))
//         type = IDX_CAPTION_BUTTON;
//
// //    if (PtInRect(&pPreviewData->rcSelectedMenuItem, pt))
// //        type = IDX_SELECTION;
//
//     if (PtInRect(&pPreviewData->rcActiveMenuBar, pt))
//         type = IDX_MENU;
//
//     if (PtInRect(&pPreviewData->rcActiveClient, pt))
//         type = IDX_WINDOW;
//
//     if (PtInRect(&pPreviewData->rcActiveScroll, pt))
//         type = IDX_SCROLLBAR;
//
//     if (PtInRect(&pPreviewData->rcDialogFrame, pt))
//         type = IDX_DIALOG;
//
//     if (PtInRect(&pPreviewData->rcDialogCaption, pt))
//         type = IDX_ACTIVE_CAPTION;
//
//     if (PtInRect(&pPreviewData->rcDialogCaptionButtons, pt))
//         type = IDX_CAPTION_BUTTON;
//
//     if (PtInRect(&pPreviewData->rcDialogButton, pt))
//         type = IDX_3D_OBJECTS;
//
//     SendMessage(GetParent(hwnd),
//                 WM_COMMAND,
//                 MAKEWPARAM(GetWindowLongPtrW(hwnd, GWLP_ID), 0),
//                 (LPARAM)type);
// }
//
//
// static VOID
// OnDestroy(PPREVIEW_DATA pPreviewData)
// {
//     DeleteObject(pPreviewData->hbrScrollbar);
//     DeleteObject(pPreviewData->hbrDesktop);
//     DeleteObject(pPreviewData->hbrWindow);
//
//     DeleteObject(pPreviewData->hCaptionFont);
//     DeleteObject(pPreviewData->hMenuFont);
//     DeleteObject(pPreviewData->hMessageFont);
//
//     DestroyMenu(pPreviewData->hMenu);
//
//     LocalFree((HLOCAL)pPreviewData->lpInAct);
//     LocalFree((HLOCAL)pPreviewData->lpAct);
//     LocalFree((HLOCAL)pPreviewData->lpWinTxt);
//     LocalFree((HLOCAL)pPreviewData->lpMessBox);
//     LocalFree((HLOCAL)pPreviewData->lpMessText);
//     LocalFree((HLOCAL)pPreviewData->lpButText);
// }
//
//
// static LRESULT __stdcall
// PreviewWndProc(HWND hwnd,
//                UINT uMsg,
//                WPARAM wParam,
//                LPARAM lParam)
// {
//     PPREVIEW_DATA pPreviewData;
//
//     pPreviewData = (PPREVIEW_DATA)GetWindowLongPtr(hwnd, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_CREATE:
//             pPreviewData = (PPREVIEW_DATA)HeapAlloc(GetProcessHeap(),
//                                                     HEAP_ZERO_MEMORY,
//                                                     sizeof(PREVIEW_DATA));
//             if (!pPreviewData)
//                 return -1;
//
//             SetWindowLongPtr(hwnd, GWLP_USERDATA, (LONG_PTR)pPreviewData);
//             OnCreate(hwnd, pPreviewData);
//             break;
//
//         case WM_SIZE:
//             OnSize(LOWORD(lParam), HIWORD(lParam), pPreviewData);
//             break;
//
//         case WM_PAINT:
//             OnPaint(hwnd, pPreviewData);
//             break;
//
//         case WM_LBUTTONDOWN:
//             OnLButtonDown(hwnd, LOWORD(lParam), HIWORD(lParam), pPreviewData);
//             break;
//
//         case WM_DESTROY:
//             OnDestroy(pPreviewData);
//             HeapFree(GetProcessHeap(), 0, pPreviewData);
//             break;
//
//         case PVM_UPDATETHEME:
//             UpdatePreviewTheme(hwnd, pPreviewData, (COLOR_SCHEME *)lParam);
//             CalculateItemSize(pPreviewData);
//             InvalidateRect(hwnd, NULL, FALSE);
//             break;
//
//         case PVM_SETSIZE:
//             SchemeSetMetric(&pPreviewData->Scheme, wParam, lParam);
//             pPreviewData->cySizeFrame = pPreviewData->Scheme.ncMetrics.iBorderWidth;  /* SM_CYSIZEFRAME */
//             pPreviewData->cyCaption = pPreviewData->Scheme.ncMetrics.iCaptionHeight+1;      /* SM_CYCAPTION */
//             pPreviewData->cyMenu = pPreviewData->Scheme.ncMetrics.iMenuHeight -1;            /* SM_CYMENU */
//             pPreviewData->cxScrollbar = pPreviewData->Scheme.ncMetrics.iScrollWidth;     /* SM_CXVSCROLL */
//             pPreviewData->cyBorder = pPreviewData->Scheme.ncMetrics.iBorderWidth;        /* SM_CYBORDER */
//             CalculateItemSize(pPreviewData);
//             InvalidateRect(hwnd, NULL, FALSE);
//             break;
//
//         case PVM_SETFONT:
//         {
//             PLOGFONTW plfFont;
//             HFONT* phFont;
//
//             switch(wParam)
//             {
//             case FONT_CAPTION:   phFont = &pPreviewData->hCaptionFont; break;
//             case FONT_MENU:      phFont = &pPreviewData->hMenuFont; break;
//             case FONT_MESSAGE:   phFont = &pPreviewData->hMessageFont; break;
//             default:  return TRUE;
//             }
//
//             plfFont = SchemeGetFont(&pPreviewData->Scheme, wParam);
//             memcpy(plfFont, (PVOID)lParam, sizeof(LOGFONTW));
//
//             DeleteObject(*phFont);
//             *phFont = CreateFontIndirect(plfFont);
//
//             InvalidateRect(hwnd, NULL, FALSE);
//             break;
//         }
//
//         case PVM_SETCOLOR:
//             pPreviewData->Scheme.crColor[wParam] = lParam;
//             switch(wParam)
//             {
//                 case COLOR_SCROLLBAR:
//                     DeleteObject(pPreviewData->hbrScrollbar);
//                     pPreviewData->hbrScrollbar = CreateSolidBrush(pPreviewData->Scheme.crColor[wParam]);
//                     break;
//                 case COLOR_DESKTOP:
//                     DeleteObject(pPreviewData->hbrDesktop);
//                     pPreviewData->hbrDesktop = CreateSolidBrush(pPreviewData->Scheme.crColor[wParam]);
//                     break;
//                 case COLOR_WINDOW:
//                     DeleteObject(pPreviewData->hbrWindow);
//                     pPreviewData->hbrWindow = CreateSolidBrush(pPreviewData->Scheme.crColor[wParam]);
//                     break;
//             }
//
//             CalculateItemSize(pPreviewData);
//             InvalidateRect(hwnd, NULL, FALSE);
//             break;
//
//         case PVM_GETSIZE:
//             return SchemeGetMetric(&pPreviewData->Scheme, wParam);
//         case PVM_GETFONT:
//             return (LRESULT)SchemeGetFont(&pPreviewData->Scheme, wParam);
//         case PVM_GETCOLOR:
//             return pPreviewData->Scheme.crColor[wParam];
//
//         case PVM_SET_HDC_PREVIEW:
//             pPreviewData->hdcPreview = (HDC)lParam;
//             InvalidateRect(hwnd, NULL, FALSE);
//             break;
//
//         default:
//             return DefWindowProc(hwnd, uMsg, wParam, lParam);
//     }
//     return FALSE;
// }
//
//
// BOOL
// RegisterPreviewControl( HINSTANCE hInstance)
// {
//     WNDCLASSEX wc = {0};
//
//     wc.cbSize = sizeof(wc);
//     wc.lpfnWndProc = PreviewWndProc;
//     wc.hInstance = hInstance;
//     wc.hCursor = LoadCursor(NULL, IDC_ARROW);
//     wc.hbrBackground = (HBRUSH)NULL;
//     wc.lpszClassName = szPreviewWndClass;
//
//     return RegisterClassEx(&wc) != (ATOM)0;
// }
//
//
// VOID
// UnregisterPreviewControl( HINSTANCE hInstance)
// {
//     UnregisterClass(szPreviewWndClass, hInstance);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

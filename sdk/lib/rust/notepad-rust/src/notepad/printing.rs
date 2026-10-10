use crate::notepad::dialog::DIALOG_StringMsgBox;
use crate::notepad::dialog::ShowLastError;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn TextOutW(hdc: HDC, x: INT, y: INT, lpString: LPCTSTR, c: INT) -> BOOL;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PRINT_DATA {
    pub printer: PRINTDLG,
    pub hwndDlg: HWND,
    pub status: INT,
    pub currentPage: INT,
    pub printRect: RECT,
    pub stNow: SYSTEMTIME,
    pub hHeaderFont: HFONT,
    pub hBodyFont: HFONT,
    pub pszText: LPTSTR,
    pub ich: DWORD,
    pub cchText: DWORD,
    pub cyHeader: INT,
    pub cySpacing: INT,
    pub cyFooter: INT,
}

pub type PPRINT_DATA = *mut PRINT_DATA;

pub const BODY_FONT_SIZE: i32 = 10; /* 10pt */

pub const HEADER_FONT_SIZE: i32 = 9; /* 9pt */

pub const SPACING_HEIGHT: i32 = 4; /* 4pt */

pub const PRINTING_MESSAGE: u32 = 0x0400 + 100; // WM_USER + 100


unsafe extern "C" fn AlertPrintError() {
    let mut szUntitled: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    LoadString(
        Globals.hInstance,
        STRING_UNTITLED as UINT,
        core::ptr::addr_of_mut!(szUntitled) as LPTSTR,
        _countof!(szUntitled) as ::core::ffi::c_int,
    );
    DIALOG_StringMsgBox(
        Globals.hMainWnd,
        (STRING_PRINTERROR as i32),
        (if Globals.szFileName[0 as usize] != 0 {
            core::ptr::addr_of_mut!(Globals.szFileName) as *mut TCHAR
        } else {
            core::ptr::addr_of_mut!(szUntitled) as *mut TCHAR
        }) as LPCTSTR,
        (MB_ICONEXCLAMATION | MB_OK) as DWORD,
    );
}

unsafe extern "C" fn GetPrintingRect(mut hdc: HDC, mut pMargins: LPCRECT) -> RECT {
    let mut iLogPixelsX: INT = GetDeviceCaps(hdc, (LOGPIXELSX as i32)) as INT;
    let mut iLogPixelsY: INT = GetDeviceCaps(hdc, (LOGPIXELSY as i32)) as INT;
    let mut iHorzRes: INT = GetDeviceCaps(hdc, (HORZRES as i32)) as INT; /* in pixels */
    let mut iVertRes: INT = GetDeviceCaps(hdc, (VERTRES as i32)) as INT; /* in pixels */
    let mut rcPrintRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcPhysical: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    /* CONVERT_X(x): MulDiv((x), iLogPixelsX, 2540) -- 100th millimeters to pixels */
    /* CONVERT_Y(y): MulDiv((y), iLogPixelsY, 2540) -- 100th millimeters to pixels */
    SetRect(
        core::ptr::addr_of_mut!(rcPrintRect),
        MulDiv((*pMargins).left as INT, iLogPixelsX, 2540),
        MulDiv((*pMargins).top as INT, iLogPixelsY, 2540),
        iHorzRes - MulDiv((*pMargins).right as INT, iLogPixelsX, 2540),
        iVertRes - MulDiv((*pMargins).bottom as INT, iLogPixelsY, 2540),
    );

    rcPhysical.left = GetDeviceCaps(hdc, (PHYSICALOFFSETX as i32)) as LONG;
    rcPhysical.right = rcPhysical.left + GetDeviceCaps(hdc, (PHYSICALWIDTH as i32)) as LONG;
    rcPhysical.top = GetDeviceCaps(hdc, (PHYSICALOFFSETY as i32)) as LONG;
    rcPhysical.bottom = rcPhysical.top + GetDeviceCaps(hdc, (PHYSICALHEIGHT as i32)) as LONG;

    /* Adjust the margin */
    rcPrintRect.left = max(rcPrintRect.left, rcPhysical.left);
    rcPrintRect.top = max(rcPrintRect.top, rcPhysical.top);
    rcPrintRect.right = min(rcPrintRect.right, rcPhysical.right);
    rcPrintRect.bottom = min(rcPrintRect.bottom, rcPhysical.bottom);

    return rcPrintRect;
}

unsafe extern "C" fn GetSelectionTextLength(hWnd: HWND) -> INT {
    let mut dwStart: DWORD = 0 as DWORD;
    let mut dwEnd: DWORD = 0 as DWORD;
    SendMessage(
        hWnd,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(dwStart) as WPARAM,
        core::ptr::addr_of_mut!(dwEnd) as LPARAM,
    );
    return dwEnd.wrapping_sub(dwStart) as INT;
}

unsafe extern "C" fn GetSelectionText(mut hWnd: HWND, mut lpString: LPTSTR, mut nMaxCount: INT) -> INT {
    let mut dwStart: DWORD = 0 as DWORD;
    let mut dwEnd: DWORD = 0 as DWORD;
    let mut cchText: INT = GetWindowTextLength(hWnd) as INT;
    let mut pszText: LPTSTR = core::ptr::null_mut();
    let mut hLocal: HLOCAL = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    SendMessage(
        hWnd,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(dwStart) as WPARAM,
        core::ptr::addr_of_mut!(dwEnd) as LPARAM,
    );
    if lpString.is_null() || dwStart == dwEnd || cchText == 0 as INT {
        return 0 as INT;
    }
    hLocal = SendMessage(
        hWnd,
        EM_GETHANDLE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HLOCAL;
    pszText = LocalLock(hLocal) as LPTSTR;
    if pszText.is_null() {
        return 0 as INT;
    }
    hr = StringCchCopyN(
        lpString as STRSAFE_LPWSTR,
        nMaxCount as size_t,
        pszText.offset(dwStart as isize) as STRSAFE_LPCWSTR,
        dwEnd.wrapping_sub(dwStart) as size_t,
    );
    LocalUnlock(hLocal);

    match hr {
        S_OK => return dwEnd.wrapping_sub(dwStart) as INT,
        STRSAFE_E_INSUFFICIENT_BUFFER => return nMaxCount - 1 as INT,
        _ => return 0 as INT,
    }
}

unsafe extern "C" fn DrawHeaderOrFooter(hDC: HDC, pRect: LPRECT, pszFormat: LPCTSTR, nPageNo: INT, pstNow: *const SYSTEMTIME) {
    let mut szText: [TCHAR; 256] = [0 as TCHAR; 256];
    let mut szField: [TCHAR; 128] = [0 as TCHAR; 128];
    let mut pchFormat: *const TCHAR = core::ptr::null();
    let mut uAlign: UINT = DT_CENTER;
    let mut uFlags: UINT = DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX;
    let mut hOldPen: HGDIOBJ = core::ptr::null_mut();
    let mut hOldBrush: HGDIOBJ = core::ptr::null_mut();

    /* Draw a rectangle */
    hOldPen = SelectObject(hDC, GetStockObject((BLACK_PEN as i32)));
    hOldBrush = SelectObject(hDC, GetStockObject((NULL_BRUSH as i32)));
    Rectangle(hDC, (*pRect).left, (*pRect).top, (*pRect).right, (*pRect).bottom);
    SelectObject(hDC, hOldBrush);
    SelectObject(hDC, hOldPen);

    InflateRect(pRect, -X_POINTS_TO_PIXELS!(hDC, 3), 0); /* Shrink 3pt */

    szText[0] = 0;

    pchFormat = pszFormat;
    'outer: while *pchFormat != 0 {
        'body: {
            if *pchFormat != ('&' as u16) {
                StringCchCatN(szText.as_mut_ptr(), (_countof!(szText)) as _, pchFormat, 1);
                break 'body;
            }

            pchFormat = pchFormat.offset(1);
            if *pchFormat == 0 {
                break 'outer;
            }

            match _totupper(*pchFormat) as i32 { /* Make it uppercase */
                38 => { /* '&': Found double ampersand */
                    StringCchCat(szText.as_mut_ptr(), (_countof!(szText)) as _, (['&' as u16, 0u16].as_ptr()));
                }
                76 => { /* 'L': Left */
                    DrawText(hDC, szText.as_ptr(), -1, pRect, uAlign | uFlags);
                    szText[0] = 0;
                    uAlign = DT_LEFT;
                }
                67 => { /* 'C': Center */
                    DrawText(hDC, szText.as_ptr(), -1, pRect, uAlign | uFlags);
                    szText[0] = 0;
                    uAlign = DT_CENTER;
                }
                82 => { /* 'R': Right */
                    DrawText(hDC, szText.as_ptr(), -1, pRect, uAlign | uFlags);
                    szText[0] = 0;
                    uAlign = DT_RIGHT;
                }
                68 => { /* 'D': Date */
                    GetDateFormat(LOCALE_USER_DEFAULT, 0, pstNow, core::ptr::null_mut(),
                                  szField.as_mut_ptr(), _countof!(szField) as INT);
                    StringCchCat(szText.as_mut_ptr(), (_countof!(szText)) as _, szField.as_ptr());
                }
                84 => { /* 'T': Time */
                    GetTimeFormat(LOCALE_USER_DEFAULT, 0, pstNow, core::ptr::null_mut(),
                                  szField.as_mut_ptr(), _countof!(szField) as INT);
                    StringCchCat(szText.as_mut_ptr(), (_countof!(szText)) as _, szField.as_ptr());
                }
                70 => { /* 'F': Filename */
                    StringCchCat(szText.as_mut_ptr(), (_countof!(szText)) as _, core::ptr::addr_of_mut!(Globals.szFileTitle) as *mut TCHAR);
                }
                80 => { /* 'P': Page number */
                    StringCchPrintf!(szField.as_mut_ptr(), _countof!(szField), (['%' as u16, 'u' as u16, 0u16].as_ptr()), nPageNo);
                    StringCchCat(szText.as_mut_ptr(), (_countof!(szText)) as _, szField.as_ptr());
                }
                _ => { /* Otherwise */
                    szField[0] = ('&' as u16);
                    szField[1] = *pchFormat;
                    szField[2] = 0;
                    StringCchCat(szText.as_mut_ptr(), (_countof!(szText)) as _, szField.as_ptr());
                }
            }
        }
        pchFormat = pchFormat.offset(1);
    }

    DrawText(hDC, szText.as_ptr(), -1, pRect, uAlign | uFlags);
}

unsafe extern "C" fn DoPrintBody(pData: PPRINT_DATA, PageCount: DWORD, bSkipPage: BOOL) -> BOOL {
    let pPrinter: LPPRINTDLG = core::ptr::addr_of_mut!((*pData).printer);
    let printRect: RECT = (*pData).printRect;
    let mut xLeft: INT = printRect.left;
    let mut yTop: INT = printRect.top + (*pData).cyHeader + (*pData).cySpacing;
    let mut xStart: INT = 0;
    let mut tabWidth: INT = 0;
    let mut ichStart: DWORD = 0;
    let mut charMetrics: SIZE = core::mem::zeroed();
    let mut tmText: TEXTMETRIC = core::mem::zeroed();

    /* Calculate a tab width */
    GetTextMetrics((*pPrinter).hDC, core::ptr::addr_of_mut!(tmText));
    tabWidth = 8 * tmText.tmAveCharWidth;

    /* The drawing-body loop */
    ichStart = (*pData).ich;
    xStart = xLeft;
    while (*pData).ich < (*pData).cchText {
        let mut ch: TCHAR = *(*pData).pszText.add((*pData).ich as usize);

        if ch == '\r' as TCHAR {
            /* DO_FLUSH */
            if ichStart < (*pData).ich && bSkipPage == 0 {
                TextOutW(
                    (*pPrinter).hDC,
                    xStart,
                    yTop,
                    (*pData).pszText.add(ichStart as usize),
                    (*pData).ich.wrapping_sub(ichStart) as INT,
                );
            }
            ichStart = (*pData).ich;
            xStart = xLeft;
            if (*pData).status == (STRING_PRINTCANCELING as i32) {
                return (FALSE) as BOOL;
            }

            (*pData).ich = (*pData).ich.wrapping_add(1); /* Next char */
            ichStart = (*pData).ich;
            continue;
        }

        if ch == '\n' as TCHAR {
            /* DO_FLUSH */
            if ichStart < (*pData).ich && bSkipPage == 0 {
                TextOutW(
                    (*pPrinter).hDC,
                    xStart,
                    yTop,
                    (*pData).pszText.add(ichStart as usize),
                    (*pData).ich.wrapping_sub(ichStart) as INT,
                );
            }
            ichStart = (*pData).ich;
            xStart = xLeft;
            if (*pData).status == (STRING_PRINTCANCELING as i32) {
                return (FALSE) as BOOL;
            }

            /* Next line */
            yTop += tmText.tmHeight;
            xLeft = printRect.left;
            xStart = xLeft;
        } else {
            if ch == '\t' as TCHAR {
                let nStepWidth: INT = tabWidth - ((xLeft - printRect.left) % tabWidth);

                /* DO_FLUSH */
                if ichStart < (*pData).ich && bSkipPage == 0 {
                    TextOutW(
                        (*pPrinter).hDC,
                        xStart,
                        yTop,
                        (*pData).pszText.add(ichStart as usize),
                        (*pData).ich.wrapping_sub(ichStart) as INT,
                    );
                }
                ichStart = (*pData).ich;
                xStart = xLeft;
                if (*pData).status == (STRING_PRINTCANCELING as i32) {
                    return (FALSE) as BOOL;
                }

                /* Go to the next tab stop */
                xLeft += nStepWidth;
                xStart = xLeft;
            } else {
                /* Normal char */
                GetTextExtentPoint32(
                    (*pPrinter).hDC,
                    core::ptr::addr_of_mut!(ch),
                    1,
                    core::ptr::addr_of_mut!(charMetrics),
                );
                xLeft += charMetrics.cx;
            }

            /* Insert a line break if the next position reached the right edge */
            if xLeft + charMetrics.cx >= printRect.right {
                if ch != '\t' as TCHAR {
                    /* DO_FLUSH */
                    if ichStart < (*pData).ich && bSkipPage == 0 {
                        TextOutW(
                            (*pPrinter).hDC,
                            xStart,
                            yTop,
                            (*pData).pszText.add(ichStart as usize),
                            (*pData).ich.wrapping_sub(ichStart) as INT,
                        );
                    }
                    ichStart = (*pData).ich;
                    xStart = xLeft;
                    if (*pData).status == (STRING_PRINTCANCELING as i32) {
                        return (FALSE) as BOOL;
                    }
                }

                /* Next line */
                yTop += tmText.tmHeight;
                xLeft = printRect.left;
                xStart = xLeft;
            }
        }

        (*pData).ich = (*pData).ich.wrapping_add(1); /* Next char */
        if ch == '\t' as TCHAR || ch == '\n' as TCHAR {
            ichStart = (*pData).ich;
        }

        if yTop + tmText.tmHeight >= printRect.bottom - (*pData).cyFooter {
            break; /* The next line reached the body bottom */
        }
    }

    /* DO_FLUSH */
    if ichStart < (*pData).ich && bSkipPage == 0 {
        TextOutW(
            (*pPrinter).hDC,
            xStart,
            yTop,
            (*pData).pszText.add(ichStart as usize),
            (*pData).ich.wrapping_sub(ichStart) as INT,
        );
    }
    ichStart = (*pData).ich;
    xStart = xLeft;
    if (*pData).status == (STRING_PRINTCANCELING as i32) {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn DoPrintPage(mut pData: PPRINT_DATA, mut PageCount: DWORD) -> BOOL {
    let mut pPrinter: LPPRINTDLG = core::ptr::addr_of_mut!((*pData).printer);
    let mut bSkipPage: BOOL = 0;
    let mut ret: BOOL = 0;
    let mut hOldFont: HFONT = core::ptr::null_mut::<HFONT__>();

    /* Should we skip this page? */
    bSkipPage = ((((*pPrinter).Flags & (PD_SELECTION as DWORD)) == 0)
        && (((*pPrinter).Flags & (PD_PAGENUMS as DWORD)) != 0)
        && !(((*pPrinter).nFromPage as DWORD) <= PageCount
            && PageCount <= ((*pPrinter).nToPage as DWORD))) as ::core::ffi::c_int
        as BOOL;

    /* The prologue of a page */
    if bSkipPage == 0 {
        if StartPage((*pPrinter).hDC) <= 0 {
            (*pData).status = (STRING_PRINTFAILED as i32);
            return (FALSE) as BOOL;
        }

        if (*pData).cyHeader > 0 {
            /* Draw the page header */
            let mut rc: RECT = (*pData).printRect;
            rc.bottom = rc.top + (*pData).cyHeader as LONG;

            hOldFont = SelectObject((*pPrinter).hDC, (*pData).hHeaderFont as HGDIOBJ) as HFONT;
            DrawHeaderOrFooter(
                (*pPrinter).hDC,
                core::ptr::addr_of_mut!(rc),
                core::ptr::addr_of_mut!(Globals.szHeader) as *mut TCHAR as LPCTSTR,
                PageCount as INT,
                core::ptr::addr_of_mut!((*pData).stNow),
            );
            SelectObject((*pPrinter).hDC, hOldFont as HGDIOBJ); /* De-select the font */
        }
    }

    hOldFont = SelectObject((*pPrinter).hDC, (*pData).hBodyFont as HGDIOBJ) as HFONT;
    ret = DoPrintBody(pData, PageCount, bSkipPage);
    SelectObject((*pPrinter).hDC, hOldFont as HGDIOBJ);
    if ret == 0 {
        return (FALSE) as BOOL; /* Canceled */
    }

    /* The epilogue of a page */
    if bSkipPage == 0 {
        if (*pData).cyFooter > 0 {
            /* Draw the page footer */
            let mut rc_0: RECT = (*pData).printRect;
            rc_0.top = rc_0.bottom - (*pData).cyFooter as LONG;

            hOldFont = SelectObject((*pPrinter).hDC, (*pData).hHeaderFont as HGDIOBJ) as HFONT;
            DrawHeaderOrFooter(
                (*pPrinter).hDC,
                core::ptr::addr_of_mut!(rc_0),
                core::ptr::addr_of_mut!(Globals.szFooter) as *mut TCHAR as LPCTSTR,
                PageCount as INT,
                core::ptr::addr_of_mut!((*pData).stNow),
            );
            SelectObject((*pPrinter).hDC, hOldFont as HGDIOBJ);
        }

        if EndPage((*pPrinter).hDC) <= 0 {
            (*pData).status = (STRING_PRINTFAILED as i32);
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn DoCreatePrintFonts(pPrinter: LPPRINTDLG, pPrintData: PPRINT_DATA) -> BOOL {
    let mut lfBody: LOGFONT = core::mem::zeroed();
    let mut lfHeader: LOGFONT = core::mem::zeroed();

    /* Create the main text font for printing */
    lfBody = Globals.lfFont;
    lfBody.lfHeight = -Y_POINTS_TO_PIXELS!((*pPrinter).hDC, BODY_FONT_SIZE) as LONG;
    (*pPrintData).hBodyFont = CreateFontIndirect(core::ptr::addr_of_mut!(lfBody));
    if (*pPrintData).hBodyFont.is_null() {
        return (FALSE) as BOOL;
    }

    /* Create the header/footer font */
    lfHeader = Globals.lfFont;
    lfHeader.lfHeight = -Y_POINTS_TO_PIXELS!((*pPrinter).hDC, HEADER_FONT_SIZE) as LONG;
    lfHeader.lfWeight = FW_BOLD as LONG;
    (*pPrintData).hHeaderFont = CreateFontIndirect(core::ptr::addr_of_mut!(lfHeader));
    if (*pPrintData).hHeaderFont.is_null() {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn DoPrintDocument(printData: PPRINT_DATA) -> BOOL {
    let mut docInfo: DOCINFO = core::mem::zeroed();
    let pPrinter: LPPRINTDLG = core::ptr::addr_of_mut!((*printData).printer);
    let mut CopyCount: DWORD = 0;
    let mut PageCount: DWORD = 0;
    let mut tmHeader: TEXTMETRIC = core::mem::zeroed();
    let mut ret: BOOL = (FALSE) as i32;
    let mut hOldFont: HFONT = core::ptr::null_mut();

    GetLocalTime(core::ptr::addr_of_mut!((*printData).stNow));

    (*printData).printRect = GetPrintingRect((*pPrinter).hDC, core::ptr::addr_of_mut!(Globals.lMargins) as LPCRECT);

    'quit: {
        if DoCreatePrintFonts(pPrinter, printData) == 0 {
            (*printData).status = (STRING_PRINTFAILED as i32);
            break 'quit;
        }

        if ((*pPrinter).Flags & PD_SELECTION) != 0 {
            (*printData).cchText = GetSelectionTextLength(Globals.hEdit) as DWORD;
        } else {
            (*printData).cchText = GetWindowTextLength(Globals.hEdit) as DWORD;
        }

        /* Allocate a buffer for the text */
        (*printData).pszText = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*printData).cchText.wrapping_add(1) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
        ) as LPTSTR;
        if (*printData).pszText.is_null() {
            (*printData).status = (STRING_PRINTFAILED as i32);
            break 'quit;
        }

        if ((*pPrinter).Flags & PD_SELECTION) != 0 {
            GetSelectionText(
                Globals.hEdit,
                (*printData).pszText,
                ((*printData).cchText.wrapping_add(1)) as INT,
            );
        } else {
            GetWindowText(
                Globals.hEdit,
                (*printData).pszText,
                ((*printData).cchText.wrapping_add(1)) as INT,
            );
        }

        /* Start a document */
        ZeroMemory!(
            core::ptr::addr_of_mut!(docInfo) as *mut core::ffi::c_void,
            core::mem::size_of::<DOCINFO>() as SIZE_T,
        );
        docInfo.cbSize = (core::mem::size_of::<DOCINFO>() as DWORD) as i32;
        docInfo.lpszDocName = core::ptr::addr_of_mut!(Globals.szFileTitle) as *mut TCHAR as LPCWSTR;
        if StartDoc((*pPrinter).hDC, core::ptr::addr_of_mut!(docInfo)) <= 0 {
            (*printData).status = (STRING_PRINTFAILED as i32);
            break 'quit;
        }

        /* Calculate the header and footer heights */
        hOldFont = SelectObject((*pPrinter).hDC, (*printData).hHeaderFont as HGDIOBJ) as HFONT;
        GetTextMetrics((*pPrinter).hDC, core::ptr::addr_of_mut!(tmHeader));
        (*printData).cyFooter = 2 * tmHeader.tmHeight;
        (*printData).cyHeader = (*printData).cyFooter;
        (*printData).cySpacing = Y_POINTS_TO_PIXELS!((*pPrinter).hDC, SPACING_HEIGHT);
        SelectObject((*pPrinter).hDC, hOldFont as HGDIOBJ); /* De-select the font */
        if Globals.szHeader[0 as usize] == 0 {
            (*printData).cyHeader = 0;
            (*printData).cySpacing = 0;
        }
        if Globals.szFooter[0 as usize] == 0 {
            (*printData).cyFooter = 0;
        }

        /* The printing-copies loop */
        CopyCount = 1;
        while CopyCount <= (*pPrinter).nCopies as DWORD {
            /* The printing-pages loop */
            PageCount = 1;
            (*printData).ich = 0;
            while (*printData).ich < (*printData).cchText {
                (*printData).currentPage = PageCount as INT;
                PostMessage(
                    (*printData).hwndDlg,
                    PRINTING_MESSAGE,
                    0,
                    0,
                );

                if DoPrintPage(printData, PageCount) == 0 {
                    AbortDoc((*pPrinter).hDC); /* Cancel printing */
                    break 'quit;
                }
                PageCount = PageCount.wrapping_add(1);
            }
            CopyCount = CopyCount.wrapping_add(1);
        }

        if EndDoc((*pPrinter).hDC) <= 0 {
            (*printData).status = (STRING_PRINTFAILED as i32);
            break 'quit;
        }

        ret = (TRUE) as i32;
        (*printData).status = (STRING_PRINTCOMPLETE as i32);
    }

    /* Quit: */
    DeleteObject((*printData).hHeaderFont as HGDIOBJ);
    DeleteObject((*printData).hBodyFont as HGDIOBJ);
    if !(*printData).pszText.is_null() {
        HeapFree(GetProcessHeap(), 0, (*printData).pszText as PVOID);
    }
    if (*printData).status == (STRING_PRINTCANCELING as i32) {
        (*printData).status = (STRING_PRINTCANCELED as i32);
    }
    PostMessage(
        (*printData).hwndDlg,
        PRINTING_MESSAGE,
        0,
        0,
    );
    return ret;
}

unsafe extern "system" fn PrintThreadFunc(arg: LPVOID) -> DWORD {
    let pData: PPRINT_DATA = arg as PPRINT_DATA;
    (*pData).currentPage = 1 as INT;
    (*pData).status = STRING_NOWPRINTING as INT;
    PostMessage(
        (*pData).hwndDlg,
        PRINTING_MESSAGE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    return DoPrintDocument(pData) as DWORD;
}

unsafe extern "system" fn DIALOG_Printing_DialogProc(mut hwnd: HWND, mut uMsg: UINT, mut wParam: WPARAM, mut lParam: LPARAM) -> INT_PTR {
    let mut szText: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    static mut s_szPage: [TCHAR; 64] = [0; 64];
    static mut s_pData: PPRINT_DATA = core::ptr::null_mut();
    static mut s_hThread: HANDLE = core::ptr::null_mut();

    match uMsg {
        WM_INITDIALOG => {
            s_pData = lParam as PPRINT_DATA;
            (*s_pData).hwndDlg = hwnd;
            SetDlgItemText(hwnd, (IDC_PRINTING_FILENAME as i32), core::ptr::addr_of_mut!(Globals.szFileTitle) as *mut TCHAR as LPCTSTR);
            GetDlgItemText(hwnd, (IDC_PRINTING_PAGE as i32), core::ptr::addr_of_mut!(s_szPage) as *mut TCHAR as LPTSTR, _countof!(s_szPage) as INT);
            SetDlgItemText(hwnd, (IDC_PRINTING_PAGE as i32), core::ptr::null_mut());

            s_hThread = CreateThread(core::ptr::null_mut(), 0, Some(PrintThreadFunc as unsafe extern "system" fn(LPVOID) -> DWORD), s_pData as LPVOID, 0, core::ptr::null_mut());
            if s_hThread.is_null() {
                (*s_pData).status = STRING_PRINTFAILED as INT;
                EndDialog(hwnd, (IDABORT) as INT_PTR);
            }
            return (TRUE) as INT_PTR;
        }

        PRINTING_MESSAGE => {
            match ((*s_pData).status) as u32 {
                STRING_NOWPRINTING | STRING_PRINTCANCELING => {
                    StringCchPrintf!(core::ptr::addr_of_mut!(szText) as *mut TCHAR, _countof!(szText), core::ptr::addr_of_mut!(s_szPage) as *mut TCHAR, (*s_pData).currentPage);
                    SetDlgItemText(hwnd, (IDC_PRINTING_PAGE as i32), core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPCTSTR);

                    LoadString(Globals.hInstance, (*s_pData).status as UINT, core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPTSTR, _countof!(szText) as INT);
                    SetDlgItemText(hwnd, (IDC_PRINTING_STATUS as i32), core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPCTSTR);
                }
                STRING_PRINTCOMPLETE | STRING_PRINTCANCELED | STRING_PRINTFAILED => {
                    LoadString(Globals.hInstance, (*s_pData).status as UINT, core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPTSTR, _countof!(szText) as INT);
                    SetDlgItemText(hwnd, (IDC_PRINTING_STATUS as i32), core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPCTSTR);

                    if (*s_pData).status == (STRING_PRINTCOMPLETE as i32) {
                        EndDialog(hwnd, (IDOK) as INT_PTR);
                    } else if (*s_pData).status == (STRING_PRINTFAILED as i32) {
                        EndDialog(hwnd, (IDABORT) as INT_PTR);
                    } else {
                        EndDialog(hwnd, (IDCANCEL) as INT_PTR);
                    }
                }
                _ => {}
            }
        }

        WM_COMMAND => {
            if (LOWORD!(wParam) as INT) as u32 == IDCANCEL && (*s_pData).status == (STRING_NOWPRINTING as i32) {
                EnableWindow(GetDlgItem(hwnd, (IDCANCEL) as i32), (FALSE) as BOOL);
                (*s_pData).status = STRING_PRINTCANCELING as INT;
                PostMessage(hwnd, PRINTING_MESSAGE, 0, 0);
            }
        }

        WM_DESTROY => {
            if !s_hThread.is_null() {
                CloseHandle(s_hThread);
            }
            DeleteDC((*s_pData).printer.hDC);
            s_pData = LocalFree(s_pData as HLOCAL) as PPRINT_DATA;
        }

        _ => {}
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FilePrint() {
    let mut ret: BOOL;
    let mut printer: LPPRINTDLG;
    let printData: PPRINT_DATA = LocalAlloc(
        LPTR as UINT,
        core::mem::size_of::<PRINT_DATA>() as SIZE_T,
    ) as PPRINT_DATA;
    if printData.is_null() {
        ShowLastError();
        return;
    }

    printer = core::ptr::addr_of_mut!((*printData).printer) as LPPRINTDLG;
    (*printer).lStructSize = core::mem::size_of::<PRINTDLG>() as DWORD;
    (*printer).hwndOwner = Globals.hMainWnd;
    (*printer).Flags = (PD_RETURNDC | PD_SELECTION) as DWORD;

    /* Disable the selection radio button if there is no text selected */
    if GetSelectionTextLength(Globals.hEdit) == 0 {
        (*printer).Flags |= PD_NOSELECTION as DWORD;
    }

    (*printer).nFromPage = 1 as WORD;
    (*printer).nToPage = MAXWORD as WORD;
    (*printer).nMinPage = 1 as WORD;
    (*printer).nMaxPage = MAXWORD as WORD;

    (*printer).hDevMode = Globals.hDevMode;
    (*printer).hDevNames = Globals.hDevNames;

    ret = PrintDlg(printer);
    /* NOTE: Even if PrintDlg returns FALSE, hDevMode and hDevNames may have changed. */
    Globals.hDevMode = (*printer).hDevMode;
    Globals.hDevNames = (*printer).hDevNames;

    if ret == 0 {
        LocalFree(printData as HLOCAL);
        return; /* The user canceled printing */
    }
    assert!(!(*printer).hDC.is_null());

    /* Ensure that each logical unit maps to one pixel */
    SetMapMode((*printer).hDC, MM_TEXT as INT);

    if DialogBoxParam(
        Globals.hInstance,
        MAKEINTRESOURCE!(DIALOG_PRINTING),
        Globals.hMainWnd,
        Some(DIALOG_Printing_DialogProc),
        printer as LPARAM,
    ) == IDABORT as INT_PTR
    {
        AlertPrintError();
    }
}

unsafe extern "system" fn DIALOG_PAGESETUP_Hook(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> UINT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            /* fetch last user input prior to display dialog */
            SetDlgItemText(hDlg, 0x141, core::ptr::addr_of_mut!(Globals.szHeader) as LPCTSTR);
            SetDlgItemText(hDlg, 0x143, core::ptr::addr_of_mut!(Globals.szFooter) as LPCTSTR);
        }

        WM_COMMAND => 'block: {
            if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                match (LOWORD!(wParam)) as u32 {
                    IDOK => {
                        /* save user input and close dialog */
                        GetDlgItemText(hDlg, 0x141, core::ptr::addr_of_mut!(Globals.szHeader) as LPTSTR, (_countof!(Globals.szHeader)) as i32);
                        GetDlgItemText(hDlg, 0x143, core::ptr::addr_of_mut!(Globals.szFooter) as LPTSTR, (_countof!(Globals.szFooter)) as i32);
                        return (FALSE) as UINT_PTR;
                    }

                    IDCANCEL => {
                        /* discard user input and close dialog */
                        return (FALSE) as UINT_PTR;
                    }

                    IDHELP => {
                        /* FIXME: Bring this to work */
                        static sorry: [TCHAR; 25] = [
                            'S' as TCHAR, 'o' as TCHAR, 'r' as TCHAR, 'r' as TCHAR, 'y' as TCHAR,
                            ',' as TCHAR, ' ' as TCHAR, 'n' as TCHAR, 'o' as TCHAR, ' ' as TCHAR,
                            'h' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'p' as TCHAR, ' ' as TCHAR,
                            'a' as TCHAR, 'v' as TCHAR, 'a' as TCHAR, 'i' as TCHAR, 'l' as TCHAR,
                            'a' as TCHAR, 'b' as TCHAR, 'l' as TCHAR, 'e' as TCHAR, 0 as TCHAR,
                        ];
                        static help: [TCHAR; 5] = [
                            'H' as TCHAR, 'e' as TCHAR, 'l' as TCHAR, 'p' as TCHAR, 0 as TCHAR,
                        ];
                        MessageBox(
                            Globals.hMainWnd,
                            core::ptr::addr_of!(sorry) as LPCTSTR,
                            core::ptr::addr_of!(help) as LPCTSTR,
                            MB_ICONEXCLAMATION,
                        );
                        return (TRUE) as UINT_PTR;
                    }

                    _ => {}
                }
            }
            break 'block;
        }

        _ => {}
    }

    return (FALSE) as UINT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FilePageSetup() {
    let mut page: PAGESETUPDLG = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(page) as *mut core::ffi::c_void,
        core::mem::size_of::<PAGESETUPDLG>(),
    );
    page.lStructSize = core::mem::size_of::<PAGESETUPDLG>() as DWORD;
    page.hwndOwner = Globals.hMainWnd;
    page.Flags = PSD_ENABLEPAGESETUPTEMPLATE | PSD_ENABLEPAGESETUPHOOK | PSD_MARGINS;
    page.hInstance = Globals.hInstance;
    page.rtMargin = Globals.lMargins;
    page.hDevMode = Globals.hDevMode;
    page.hDevNames = Globals.hDevNames;
    page.lpPageSetupTemplateName = MAKEINTRESOURCE!(DIALOG_PAGESETUP);
    page.lpfnPageSetupHook = Some(
        DIALOG_PAGESETUP_Hook,
    );

    PageSetupDlg(core::ptr::addr_of_mut!(page));

    /* NOTE: Even if PageSetupDlg returns FALSE, the following members may have changed */
    Globals.hDevMode = page.hDevMode;
    Globals.hDevNames = page.hDevNames;
    Globals.lMargins = page.rtMargin;
}

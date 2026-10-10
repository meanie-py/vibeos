use crate::clipbrd::cliputils::GetLineExtentA;
use crate::clipbrd::cliputils::GetLineExtentW;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ShowLastWin32Error(hwndParent: HWND) {
    let mut dwError: DWORD;
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();

    dwError = GetLastError();
    if dwError == ERROR_SUCCESS {
        return;
    }

    if FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER |
        FORMAT_MESSAGE_FROM_SYSTEM |
        FORMAT_MESSAGE_IGNORE_INSERTS,
        core::ptr::null(),
        dwError,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(lpMsgBuf) as LPWSTR,
        0,
        core::ptr::null_mut(),
    ) == 0
    {
        return;
    }

    MessageBoxW(hwndParent, lpMsgBuf as LPCWSTR, core::ptr::null(), MB_OK | MB_ICONERROR);
    LocalFree((lpMsgBuf).cast());
}
#[no_mangle]
pub unsafe extern "C" fn BringWindowToFront(hWnd: HWND) {
    if IsIconic(hWnd) != 0 {
        ShowWindow(hWnd, (SW_RESTORE) as i32);
        SetForegroundWindow(hWnd);
    } else {
        SetForegroundWindow(hWnd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MessageBoxRes(hWnd: HWND, hInstance: HINSTANCE, uText: UINT, uCaption: UINT, uType: UINT) -> ::core::ffi::c_int {
    let mut mb: MSGBOXPARAMSW = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(mb) as *mut core::ffi::c_void, core::mem::size_of::<MSGBOXPARAMSW>() as usize);
    mb.cbSize = core::mem::size_of::<MSGBOXPARAMSW>() as UINT;
    mb.hwndOwner = hWnd;
    mb.hInstance = hInstance;
    mb.lpszText = MAKEINTRESOURCEW!(uText) as LPCWSTR;
    mb.lpszCaption = MAKEINTRESOURCEW!(uCaption) as LPCWSTR;
    mb.dwStyle = uType as DWORD;
    mb.dwLanguageId = MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD;

    return MessageBoxIndirectW(core::ptr::addr_of_mut!(mb));
}
#[no_mangle]
pub unsafe extern "C" fn DrawTextFromResource(hInstance: HINSTANCE, uID: UINT, hDC: HDC, lpRect: LPRECT, uFormat: UINT) {
    let mut lpBuffer: LPWSTR = core::ptr::null_mut();
    let mut nCount: i32 = 0;
    nCount = LoadStringW(hInstance, uID, core::ptr::addr_of_mut!(lpBuffer) as LPWSTR, 0);
    if nCount != 0 {
        DrawTextW(hDC, lpBuffer as LPCWSTR, nCount, lpRect, uFormat);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DrawTextFromClipboard(uFormat: UINT, ps: PAINTSTRUCT, state: SCROLLSTATE) {
    let mut ptOrg: POINT = POINT { x: 0, y: 0 };
    let mut hGlobal: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    let mut lpText: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ptr: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut lineSize: SIZE_T = 0;
    let mut FirstLine: INT = 0;
    let mut LastLine: INT = 0;

    hGlobal = GetClipboardData(uFormat) as HGLOBAL;
    if hGlobal.is_null() {
        return;
    }

    lpText = GlobalLock(hGlobal) as PVOID;
    if lpText.is_null() {
        return;
    }

    /* Find the first and last line indices to display (Note that CurrentX/Y are in pixels!) */
    FirstLine = max(0, (state.CurrentY + ps.rcPaint.top) / Globals.CharHeight) as INT;
    // LastLine = min(LINES - 1, (state.CurrentY + ps.rcPaint.bottom) / Globals.CharHeight);
    // NOTE: Can be less or greater than the actual number of lines in the text.
    LastLine = ((state.CurrentY + ps.rcPaint.bottom) / Globals.CharHeight) as INT;

    /* Find the first text line to display */
    while FirstLine > 0 {
        if uFormat == CF_UNICODETEXT {
            if *(lpText as LPCWSTR) == UNICODE_NULL {
                break;
            }
            GetLineExtentW(lpText as LPCWSTR, core::ptr::addr_of_mut!(ptr) as *mut LPCWSTR);
        } else {
            if *(lpText as LPCSTR) == ANSI_NULL {
                break;
            }
            GetLineExtentA(lpText as LPCSTR, core::ptr::addr_of_mut!(ptr) as *mut LPCSTR);
        }

        FirstLine -= 1;
        LastLine -= 1;

        lpText = ptr;
    }

    ptOrg.x = ps.rcPaint.left;
    ptOrg.y = /* FirstLine */ max(0, (state.CurrentY + ps.rcPaint.top) / Globals.CharHeight)
        * Globals.CharHeight - state.CurrentY;

    /* Display each line from the current one up to the last one */
    LastLine += 1;
    while LastLine >= 0 {
        if uFormat == CF_UNICODETEXT {
            if *(lpText as LPCWSTR) == UNICODE_NULL {
                break;
            }
            lineSize = GetLineExtentW(lpText as LPCWSTR, core::ptr::addr_of_mut!(ptr) as *mut LPCWSTR) as SIZE_T;
            TabbedTextOutW(
                ps.hdc,
                /*ptOrg.x*/ 0 - state.CurrentX,
                ptOrg.y,
                lpText as LPCWSTR,
                lineSize as INT,
                0,
                core::ptr::null::<INT>(),
                /*ptOrg.x*/ 0 - state.CurrentX,
            );
        } else {
            if *(lpText as LPCSTR) == ANSI_NULL {
                break;
            }
            lineSize = GetLineExtentA(lpText as LPCSTR, core::ptr::addr_of_mut!(ptr) as *mut LPCSTR) as SIZE_T;
            TabbedTextOutA(
                ps.hdc,
                /*ptOrg.x*/ 0 - state.CurrentX,
                ptOrg.y,
                lpText as LPCSTR,
                lineSize as INT,
                0,
                core::ptr::null::<INT>(),
                /*ptOrg.x*/ 0 - state.CurrentX,
            );
        }

        LastLine -= 1;

        ptOrg.y += Globals.CharHeight;
        lpText = ptr;
    }

    GlobalUnlock(hGlobal);
}
#[no_mangle]
pub unsafe extern "C" fn BitBltFromClipboard(ps: PAINTSTRUCT, state: SCROLLSTATE, dwRop: DWORD) {
    let mut hdcMem: HDC = core::ptr::null_mut();
    let mut hBitmap: HBITMAP = core::ptr::null_mut();
    let mut bmp: BITMAP = core::mem::zeroed();
    let mut bmWidth: LONG;
    let mut bmHeight: LONG;

    hdcMem = CreateCompatibleDC(ps.hdc);
    if hdcMem.is_null() {
        return;
    }

    hBitmap = GetClipboardData(CF_BITMAP as UINT) as HBITMAP;
    GetObjectW(
        hBitmap as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bmp) as LPVOID,
    );

    SelectObject(hdcMem, hBitmap as HGDIOBJ);

    bmWidth = min(
        ps.rcPaint.right - ps.rcPaint.left,
        bmp.bmWidth - ps.rcPaint.left - state.CurrentX as LONG,
    );
    bmHeight = min(
        ps.rcPaint.bottom - ps.rcPaint.top,
        bmp.bmHeight - ps.rcPaint.top - state.CurrentY as LONG,
    );

    BitBlt(
        ps.hdc,
        ps.rcPaint.left as INT,
        ps.rcPaint.top as INT,
        bmWidth as INT,
        bmHeight as INT,
        hdcMem,
        (ps.rcPaint.left + state.CurrentX as LONG) as INT,
        (ps.rcPaint.top + state.CurrentY as LONG) as INT,
        dwRop,
    );

    DeleteDC(hdcMem);
}
#[no_mangle]
pub unsafe extern "C" fn SetDIBitsToDeviceFromClipboard(uFormat: UINT, ps: PAINTSTRUCT, state: SCROLLSTATE, fuColorUse: UINT) {
    let mut hGlobal: HGLOBAL;
    let mut lpInfoHeader: LPBITMAPINFOHEADER;
    let mut lpBits: LPBYTE;
    let mut bmWidth: LONG;
    let mut bmHeight: LONG;
    let mut dwPalSize: DWORD = 0;

    hGlobal = GetClipboardData(uFormat) as HGLOBAL;
    if hGlobal.is_null() {
        return;
    }

    lpInfoHeader = GlobalLock(hGlobal) as LPBITMAPINFOHEADER;
    if lpInfoHeader.is_null() {
        return;
    }

    if (*lpInfoHeader).biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        let lpCoreHeader: LPBITMAPCOREHEADER = lpInfoHeader as LPBITMAPCOREHEADER;

        dwPalSize = 0;

        if (*lpCoreHeader).bcBitCount as c_int <= 8 {
            dwPalSize = (1 << ((*lpCoreHeader).bcBitCount as u32)) as DWORD;

            if fuColorUse == DIB_RGB_COLORS {
                dwPalSize = dwPalSize.wrapping_mul(core::mem::size_of::<RGBTRIPLE>() as DWORD);
            } else {
                dwPalSize = dwPalSize.wrapping_mul(core::mem::size_of::<WORD>() as DWORD);
            }
        }

        bmWidth = (*lpCoreHeader).bcWidth as LONG;
        bmHeight = (*lpCoreHeader).bcHeight as LONG;
    } else if (*lpInfoHeader).biSize == core::mem::size_of::<BITMAPINFOHEADER>() as DWORD
        || (*lpInfoHeader).biSize == core::mem::size_of::<BITMAPV4HEADER>() as DWORD
        || (*lpInfoHeader).biSize == core::mem::size_of::<BITMAPV5HEADER>() as DWORD
    {
        dwPalSize = (*lpInfoHeader).biClrUsed;

        if (dwPalSize == 0) && ((*lpInfoHeader).biBitCount as c_int <= 8) {
            dwPalSize = (1 << ((*lpInfoHeader).biBitCount as u32)) as DWORD;
        }

        if fuColorUse == DIB_RGB_COLORS {
            dwPalSize = dwPalSize.wrapping_mul(core::mem::size_of::<RGBQUAD>() as DWORD);
        } else {
            dwPalSize = dwPalSize.wrapping_mul(core::mem::size_of::<WORD>() as DWORD);
        }

        if /*(lpInfoHeader->biSize == sizeof(core::mem::size_of::<BITMAPINFOHEADER>())) &&*/
            (*lpInfoHeader).biCompression == BI_BITFIELDS
        {
            dwPalSize = dwPalSize.wrapping_add((3 as DWORD).wrapping_mul(core::mem::size_of::<DWORD>() as DWORD));
        }

        /*
         * This is a (disabled) hack for Windows, when uFormat == CF_DIB
         * it needs yet another extra 3 DWORDs, in addition to the
         * ones already taken into account in via the compression.
         * This problem doesn't happen when uFormat == CF_DIBV5
         * (in that case, when compression is taken into account,
         * everything is nice).
         *
         * NOTE 1: This fix is only for us, because when one pastes DIBs
         * directly in apps, the bitmap offset problem is still present.
         *
         * NOTE 2: The problem can be seen with Windows' clipbrd.exe if
         * one copies a CF_DIB image in the clipboard. By default Windows'
         * clipbrd.exe works with CF_DIBV5 and CF_BITMAP, so the problem
         * is unseen, and the clipboard doesn't have to convert to CF_DIB.
         *
         * FIXME: investigate!!
         * ANSWER: this is a Windows bug; part of the answer is there:
         * https://social.msdn.microsoft.com/Forums/windowsdesktop/en-US/ac7ab3b5-8609-4478-b86a-976dab44c271/bug-clipboard-format-conversions-cfdib-cfdibv5-cfdib (DEAD_LINK)
         * May be related:
         * https://blog.talosintelligence.com/dangerous-clipboard/
         */

        bmWidth = (*lpInfoHeader).biWidth;
        /* NOTE: biHeight < 0 for bottom-up DIBs, or > 0 for top-down DIBs */
        bmHeight = (*lpInfoHeader).biHeight;
    } else {
        /* Invalid format */
        GlobalUnlock(hGlobal);
        return;
    }

    lpBits = (lpInfoHeader as LPBYTE)
        .add((*lpInfoHeader).biSize as usize + dwPalSize as usize);

    /*
     * The seventh parameter (YSrc) of SetDIBitsToDevice always designates
     * the Y-coordinate of the "lower-left corner" of the image, be the DIB
     * in bottom-up or top-down mode.
     */
    SetDIBitsToDevice(
        ps.hdc,
        -(state.CurrentX as c_int), // ps.rcPaint.left,
        -(state.CurrentY as c_int), // ps.rcPaint.top,
        bmWidth as DWORD,
        bmHeight as DWORD,
        0, // ps.rcPaint.left + state.CurrentX,
        0, // -(ps.rcPaint.top  + state.CurrentY),
        0, // uStartScan,
        bmHeight as UINT,
        lpBits as *const c_void,
        lpInfoHeader as LPBITMAPINFO as *const BITMAPINFO,
        fuColorUse,
    );

    GlobalUnlock(hGlobal);
}
#[no_mangle]
pub unsafe extern "C" fn PlayMetaFileFromClipboard(mut hdc: HDC, mut lpRect: *const RECT) {
    let mut mp: LPMETAFILEPICT = core::ptr::null_mut::<tagMETAFILEPICT>();
    let mut hGlobal: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    hGlobal = GetClipboardData(CF_METAFILEPICT as UINT) as HGLOBAL;
    if hGlobal.is_null() {
        return;
    }
    mp = GlobalLock(hGlobal) as LPMETAFILEPICT;
    if mp.is_null() {
        return;
    }
    SetMapMode(hdc, (*mp).mm as core::ffi::c_int);
    SetViewportExtEx(
        hdc,
        (*lpRect).right as core::ffi::c_int,
        (*lpRect).bottom as core::ffi::c_int,
        core::ptr::null_mut::<tagSIZE>(),
    );
    SetViewportOrgEx(
        hdc,
        (*lpRect).left as core::ffi::c_int,
        (*lpRect).top as core::ffi::c_int,
        core::ptr::null_mut::<tagPOINT>(),
    );
    PlayMetaFile(hdc, (*mp).hMF);
    GlobalUnlock(hGlobal);
}
#[no_mangle]
pub unsafe extern "C" fn PlayEnhMetaFileFromClipboard(hdc: HDC, lpRect: *const RECT) {
    let mut hEmf: HENHMETAFILE;
    hEmf = GetClipboardData(CF_ENHMETAFILE as UINT) as HENHMETAFILE;
    PlayEnhMetaFile(hdc, hEmf, lpRect);
}

unsafe extern "C" fn AllocStrCat(mut psz: LPWSTR, mut cat: LPCWSTR) -> LPWSTR {
    let mut cch: INT = 0;
    let mut pszNew: LPWSTR = core::ptr::null_mut();
    if psz.is_null() {
        return _wcsdup(cat as *const WCHAR) as LPWSTR;
    }
    cch = (lstrlenW(psz as LPCWSTR) + lstrlenW(cat) + 1) as INT;
    pszNew = realloc(
        psz as *mut core::ffi::c_void,
        (cch as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as LPWSTR;
    if pszNew.is_null() {
        return psz;
    }
    lstrcatW(pszNew, cat);
    return pszNew;
}
#[no_mangle]
pub unsafe extern "C" fn HDropFromClipboard(hdc: HDC, lpRect: *const RECT) {
    let mut pszAlloc: LPWSTR = core::ptr::null_mut();
    let mut szFile: [WCHAR; MAX_PATH as usize + 2 as usize] = [0; MAX_PATH as usize + 2 as usize];
    let mut hDrop: HDROP = GetClipboardData(CF_HDROP as UINT) as HDROP;
    let mut iFile: UINT = 0;
    let mut cFiles: UINT = DragQueryFileW(hDrop, 0xFFFFFFFF as UINT, core::ptr::null_mut(), 0 as UINT) as UINT;
    let mut rc: RECT = *lpRect;

    FillRect(hdc, core::ptr::addr_of_mut!(rc), (COLOR_WINDOW + 1) as HBRUSH);

    iFile = 0 as UINT;
    while iFile < cFiles {
        DragQueryFileW(hDrop, iFile, core::ptr::addr_of_mut!(szFile) as LPWSTR, _countof!(szFile) as UINT);
        lstrcatW(core::ptr::addr_of_mut!(szFile) as LPWSTR, RUSTCONV_WSTR_80E09B2F645F.as_ptr());
        pszAlloc = AllocStrCat(pszAlloc, core::ptr::addr_of_mut!(szFile) as LPCWSTR);
        iFile += 1;
    }

    DrawTextW(hdc, pszAlloc as LPCWSTR, -(1 as ::core::ffi::c_int), core::ptr::addr_of_mut!(rc),
              (DT_LEFT | DT_NOPREFIX | DT_EXTERNALLEADING | DT_WORD_ELLIPSIS) as UINT);
    free(pszAlloc as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn RealizeClipboardPalette(mut hdc: HDC) -> BOOL {
    let mut Success: BOOL;
    let mut hPalette: HPALETTE;
    let mut hOldPalette: HPALETTE;

    if IsClipboardFormatAvailable(CF_PALETTE) == 0 {
        return (FALSE) as BOOL;
    }

    hPalette = GetClipboardData(CF_PALETTE) as HPALETTE;
    if hPalette.is_null() {
        return (FALSE) as BOOL;
    }

    hOldPalette = SelectPalette(hdc, hPalette, (FALSE) as BOOL);
    if hOldPalette.is_null() {
        return (FALSE) as BOOL;
    }

    Success = (RealizePalette(hdc) != GDI_ERROR) as BOOL;

    SelectPalette(hdc, hOldPalette, (FALSE) as BOOL);

    return Success;
}

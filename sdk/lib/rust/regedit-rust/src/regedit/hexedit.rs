// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00DD366A589D: [u16; 3] = [0x0025u16, 0x0043, 0x0000];
pub static RUSTCONV_WSTR_03818682B74D: [u16; 5] = [0x0025u16, 0x0030, 0x0034, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_1E8D48188E3D: [u16; 5] = [0x0025u16, 0x0030, 0x0032, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_DE4DB6B4EC92: [u16; 6] = [0x0025u16, 0x0030, 0x0034, 0x006C, 0x0058, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const HEX_EDIT_CLASS_NAME: [::core::ffi::c_ushort; 10] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_ushort; 10]>(
        *b"H\0e\0x\0E\0d\0i\0t\x003\x002\0\0\0",
    )
};

static mut ClipboardFormatName: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"R\0e\0g\0E\0d\0i\0t\0_\0H\0e\0x\0D\0a\0t\0a\0\0\0",
    )
};

static mut ClipboardFormatID: UINT = 0 as UINT;

#[repr(C)]
pub struct HEXEDIT_DATA {
    pub hWndSelf: HWND,
    pub hWndParent: HWND,
    pub hBuffer: HLOCAL,
    pub style: DWORD,
    pub MaxBuffer: DWORD,
    pub ColumnsPerLine: INT,
    pub nLines: INT,
    pub nVisibleLinesComplete: INT,
    pub nVisibleLines: INT,
    pub Index: INT,
    pub LineHeight: INT,
    pub CharWidth: INT,
    pub hFont: HFONT,
    pub SbVisible: BOOL,

    pub LeftMargin: INT,
    pub AddressSpacing: INT,
    pub SplitSpacing: INT,

    pub EditingField: BOOL,
    pub CaretCol: INT,
    pub CaretLine: INT,
    pub InMid: BOOL,

    pub SelStart: INT,
    pub SelEnd: INT,
}
pub type PHEXEDIT_DATA = *mut HEXEDIT_DATA;

pub const HEHT_LEFTMARGIN: u32 = 0x1;
pub const HEHT_ADDRESS: u32 = 0x2;
pub const HEHT_ADDRESSSPACING: u32 = 0x3;
pub const HEHT_HEXDUMP: u32 = 0x4;
pub const HEHT_HEXDUMPSPACING: u32 = 0x5;
pub const HEHT_ASCIIDUMP: u32 = 0x6;
pub const HEHT_RIGHTMARGIN: u32 = 0x7;

#[no_mangle]
pub unsafe extern "system" fn RegisterHexEditorClass(hInstance: HINSTANCE) -> ATOM {
    let mut WndClass: WNDCLASSEXW = core::mem::zeroed();

    ClipboardFormatID = RegisterClipboardFormatW(ClipboardFormatName.as_ptr());

    ZeroMemory!(
        core::ptr::addr_of_mut!(WndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSEXW>(),
    );
    WndClass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    WndClass.style = CS_DBLCLKS;
    WndClass.lpfnWndProc = Some(HexEditWndProc);
    WndClass.cbWndExtra = core::mem::size_of::<PHEXEDIT_DATA>() as core::ffi::c_int;
    WndClass.hInstance = hInstance;
    WndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_IBEAM as LPCWSTR,
    );
    WndClass.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    WndClass.lpszClassName = (HEX_EDIT_CLASS_NAME).as_ptr();

    return RegisterClassEx(core::ptr::addr_of_mut!(WndClass));
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterHexEditorClass(hInstance: HINSTANCE) -> BOOL {
    return UnregisterClassW(HEX_EDIT_CLASS_NAME.as_ptr() as LPCWSTR, hInstance);
}

unsafe extern "C" fn HEXEDIT_MoveCaret(hed: PHEXEDIT_DATA, Scroll: BOOL) {
    let mut si: SCROLLINFO = core::mem::zeroed();

    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = SIF_POS as UINT;
    GetScrollInfo((*hed).hWndSelf, (SB_VERT) as i32, core::ptr::addr_of_mut!(si));

    if Scroll != 0 {
        if si.nPos > (*hed).CaretLine {
            si.nPos = (*hed).CaretLine;
            SetScrollInfo(
                (*hed).hWndSelf,
                (SB_VERT) as i32,
                core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
                (TRUE) as BOOL,
            );
            GetScrollInfo(
                (*hed).hWndSelf,
                (SB_VERT) as i32,
                core::ptr::addr_of_mut!(si),
            );
            InvalidateRect(
                (*hed).hWndSelf,
                core::ptr::null::<RECT>(),
                (TRUE) as BOOL,
            );
        } else if (*hed).CaretLine >= (*hed).nVisibleLinesComplete + si.nPos {
            si.nPos = (*hed).CaretLine - (*hed).nVisibleLinesComplete + 1;
            SetScrollInfo(
                (*hed).hWndSelf,
                (SB_VERT) as i32,
                core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
                (TRUE) as BOOL,
            );
            GetScrollInfo(
                (*hed).hWndSelf,
                (SB_VERT) as i32,
                core::ptr::addr_of_mut!(si),
            );
            InvalidateRect(
                (*hed).hWndSelf,
                core::ptr::null::<RECT>(),
                (TRUE) as BOOL,
            );
        }
    }

    if (*hed).EditingField != 0 {
        SetCaretPos(
            (*hed).LeftMargin
                + ((4
                    + (*hed).AddressSpacing
                    + (3 * (*hed).CaretCol)
                    + (*hed).InMid * 2)
                    * (*hed).CharWidth)
                - 1,
            ((*hed).CaretLine - si.nPos) * (*hed).LineHeight,
        );
    } else {
        SetCaretPos(
            (*hed).LeftMargin
                + ((4
                    + (*hed).AddressSpacing
                    + (*hed).SplitSpacing
                    + (3 * (*hed).ColumnsPerLine)
                    + (*hed).CaretCol)
                    * (*hed).CharWidth)
                - 2,
            ((*hed).CaretLine - si.nPos) * (*hed).LineHeight,
        );
    }
}

unsafe extern "C" fn HEXEDIT_Update(hed: PHEXEDIT_DATA) {
    let mut si: SCROLLINFO = core::mem::zeroed();
    let mut rcClient: RECT = core::mem::zeroed();
    let mut SbVisible: BOOL;
    let mut bufsize: INT;
    let mut cvislines: INT;

    GetClientRect((*hed).hWndSelf, core::ptr::addr_of_mut!(rcClient));
    (*hed).style = GetWindowLongPtr((*hed).hWndSelf, GWL_STYLE) as DWORD;

    bufsize = if !(*hed).hBuffer.is_null() {
        LocalSize((*hed).hBuffer) as INT
    } else {
        0
    };
    (*hed).nLines = max!(bufsize / (*hed).ColumnsPerLine, 1);
    if bufsize > (*hed).ColumnsPerLine && (bufsize % (*hed).ColumnsPerLine) > 0
    {
        (*hed).nLines += 1;
    }

    if (*hed).LineHeight > 0 {
        cvislines = rcClient.bottom / (*hed).LineHeight;
        (*hed).nVisibleLinesComplete = cvislines;
        (*hed).nVisibleLines = (*hed).nVisibleLinesComplete;
        if rcClient.bottom % (*hed).LineHeight != 0 {
            (*hed).nVisibleLines += 1;
        }
    } else {
        cvislines = 0;
        (*hed).nVisibleLines = cvislines;
    }

    SbVisible = (bufsize > 0 && cvislines < (*hed).nLines) as BOOL;
    ShowScrollBar((*hed).hWndSelf, (SB_VERT) as i32, SbVisible);

    /* update scrollbar */
    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = (SIF_RANGE | SIF_PAGE) as UINT;
    si.nMin = 0;
    si.nMax = if bufsize > 0 {
        (*hed).nLines - 1
    } else {
        0
    };
    si.nPage = (if (*hed).LineHeight > 0 {
        rcClient.bottom / (*hed).LineHeight
    } else {
        0
    }) as UINT;
    SetScrollInfo(
        (*hed).hWndSelf,
        (SB_VERT) as i32,
        core::ptr::addr_of_mut!(si),
        (TRUE) as BOOL,
    );

    if IsWindowVisible((*hed).hWndSelf) != 0 && SbVisible != (*hed).SbVisible {
        InvalidateRect(
            (*hed).hWndSelf,
            core::ptr::null_mut(),
            (TRUE) as BOOL,
        );
    }

    (*hed).SbVisible = SbVisible;
}

unsafe extern "C" fn HEXEDIT_GetFixedFont() -> HFONT {
    let mut lf: LOGFONT = core::mem::zeroed();
    GetObject(GetStockObject((ANSI_FIXED_FONT as i32)) as HANDLE, core::mem::size_of::<LOGFONT>() as ::core::ffi::c_int, core::ptr::addr_of_mut!(lf) as LPVOID);
    return CreateFontIndirect(core::ptr::addr_of_mut!(lf));
}

unsafe extern "C" fn HEXEDIT_PaintLines(
    hed: PHEXEDIT_DATA,
    hDC: HDC,
    ScrollPos: DWORD,
    mut First: DWORD,
    mut Last: DWORD,
    rc: *mut RECT,
) {
    let mut dx: DWORD = 0;
    let mut dy: DWORD = 0;
    let mut linestart: DWORD = 0;
    let mut i: INT = 0;
    let mut isave: INT = 0;
    let mut i0: INT = 0;
    let mut i1: INT = 0;
    let mut x: INT = 0;
    let mut buf: PBYTE = core::ptr::null_mut();
    let mut current: PBYTE = core::ptr::null_mut();
    let mut end: PBYTE = core::ptr::null_mut();
    let mut line: PBYTE = core::ptr::null_mut();
    let mut bufsize: usize = 0;
    let mut hex: [WCHAR; 3] = [0; 3];
    let mut addr: [WCHAR; 17] = [0; 17];
    let mut rct: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rct2: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    FillRect(
        hDC,
        rc as LPCRECT,
        (COLOR_WINDOW + 1) as HBRUSH,
    );
    SetTextColor(
        hDC,
        GetSysColor((COLOR_WINDOWTEXT) as i32) as COLORREF,
    );

    if (*hed).SelStart < (*hed).SelEnd {
        i0 = (*hed).SelStart;
        i1 = (*hed).SelEnd;
    } else {
        i0 = (*hed).SelEnd;
        i1 = (*hed).SelStart;
    }

    if !(*hed).hBuffer.is_null() {
        bufsize = LocalSize((*hed).hBuffer) as usize;
        buf = LocalLock((*hed).hBuffer) as PBYTE;
    } else {
        buf = core::ptr::null_mut();
        bufsize = 0 as usize;

        if ScrollPos.wrapping_add(First) == 0 as DWORD {
            wsprintf(
                addr.as_mut_ptr(),
                RUSTCONV_WSTR_03818682B74D.as_ptr(),
                0 as ::core::ffi::c_int,
            );
            TextOutW(
                hDC,
                (*hed).LeftMargin,
                First.wrapping_mul((*hed).LineHeight as DWORD) as INT,
                addr.as_mut_ptr() as LPCWSTR,
                4 as INT,
            );
        }
    }

    if !buf.is_null() {
        end = buf.add(bufsize);
        dy = First.wrapping_mul((*hed).LineHeight as DWORD);
        linestart = ScrollPos
            .wrapping_add(First)
            .wrapping_mul((*hed).ColumnsPerLine as DWORD);
        i = linestart as INT;
        current = buf.add(linestart as usize);
        Last = min!(
            ((*hed).nLines as DWORD).wrapping_sub(ScrollPos),
            Last,
        );

        SetBkMode(hDC, (TRANSPARENT as i32));
        while First <= Last && current < end {
            let mut dh: DWORD = 0;

            dx = (*hed).LeftMargin as DWORD;

            /* draw address */
            wsprintf(
                addr.as_mut_ptr(),
                RUSTCONV_WSTR_DE4DB6B4EC92.as_ptr(),
                linestart as ::core::ffi::c_int,
            );
            TextOutW(
                hDC,
                dx as INT,
                dy as INT,
                addr.as_mut_ptr() as LPCWSTR,
                4 as INT,
            );

            dx = dx.wrapping_add(
                (((4 + (*hed).AddressSpacing) * (*hed).CharWidth)) as DWORD,
            );
            dh = ((3 * (*hed).CharWidth)) as DWORD;

            rct.left = dx as LONG;
            rct.top = dy as LONG;
            rct.right = (rct.left as DWORD).wrapping_add(dh) as LONG;
            rct.bottom = dy.wrapping_add((*hed).LineHeight as DWORD) as LONG;

            /* draw hex map */
            dx = dx.wrapping_add(((*hed).CharWidth / 2) as DWORD);
            line = current;
            isave = i;
            x = 0 as INT;
            while x < (*hed).ColumnsPerLine && current < end {
                rct.left = (rct.left as DWORD).wrapping_add(dh) as LONG;
                rct.right = (rct.right as DWORD).wrapping_add(dh) as LONG;

                let fresh34 = current;
                current = current.add(1);
                wsprintf(
                    hex.as_mut_ptr(),
                    RUSTCONV_WSTR_1E8D48188E3D.as_ptr(),
                    *fresh34 as ::core::ffi::c_int,
                );
                if i0 <= i && i < i1 {
                    rct2.left = dx as LONG;
                    rct2.top = dy as LONG;
                    rct2.right = dx
                        .wrapping_add(((*hed).CharWidth * 2) as DWORD)
                        .wrapping_add(1 as DWORD) as LONG;
                    rct2.bottom = dy.wrapping_add((*hed).LineHeight as DWORD) as LONG;
                    InflateRect(
                        core::ptr::addr_of_mut!(rct2),
                        (*hed).CharWidth / 2,
                        0,
                    );
                    FillRect(
                        hDC,
                        core::ptr::addr_of_mut!(rct2),
                        (COLOR_HIGHLIGHT + 1) as HBRUSH,
                    );
                    SetTextColor(
                        hDC,
                        GetSysColor((COLOR_HIGHLIGHTTEXT) as i32) as COLORREF,
                    );
                    ExtTextOutW(
                        hDC,
                        dx as INT,
                        dy as INT,
                        0 as UINT,
                        core::ptr::addr_of_mut!(rct),
                        hex.as_mut_ptr() as LPCWSTR,
                        2 as UINT,
                        core::ptr::null::<INT>(),
                    );
                    SetTextColor(
                        hDC,
                        GetSysColor((COLOR_WINDOWTEXT) as i32) as COLORREF,
                    );
                } else {
                    ExtTextOutW(
                        hDC,
                        dx as INT,
                        dy as INT,
                        ETO_OPAQUE as UINT,
                        core::ptr::addr_of_mut!(rct),
                        hex.as_mut_ptr() as LPCWSTR,
                        2 as UINT,
                        core::ptr::null::<INT>(),
                    );
                }
                dx = dx.wrapping_add(dh);
                i += 1;
                x += 1;
            }

            /* draw ascii map */
            dx = ((4
                + (*hed).AddressSpacing
                + (*hed).SplitSpacing
                + ((*hed).ColumnsPerLine * 3))
                * (*hed).CharWidth) as DWORD;
            current = line;
            i = isave;
            x = 0 as INT;
            while x < (*hed).ColumnsPerLine && current < end {
                let fresh35 = current;
                current = current.add(1);
                wsprintf(
                    hex.as_mut_ptr(),
                    RUSTCONV_WSTR_00DD366A589D.as_ptr(),
                    *fresh35 as ::core::ffi::c_int,
                );
                hex[0 as usize] = (if (hex[0 as usize]
                    & 0x007Fu16)
                    >= 0x0020u16
                {
                    hex[0 as usize]
                } else {
                    0x002Eu16
                }) as WCHAR;
                if i0 <= i && i < i1 {
                    rct2.left = dx as LONG;
                    rct2.top = dy as LONG;
                    rct2.right = dx.wrapping_add((*hed).CharWidth as DWORD) as LONG;
                    rct2.bottom = dy.wrapping_add((*hed).LineHeight as DWORD) as LONG;
                    FillRect(
                        hDC,
                        core::ptr::addr_of_mut!(rct2),
                        (COLOR_HIGHLIGHT + 1) as HBRUSH,
                    );
                    SetTextColor(
                        hDC,
                        GetSysColor((COLOR_HIGHLIGHTTEXT) as i32) as COLORREF,
                    );
                    TextOutW(
                        hDC,
                        dx as INT,
                        dy as INT,
                        hex.as_mut_ptr() as LPCWSTR,
                        1 as INT,
                    );
                    SetTextColor(
                        hDC,
                        GetSysColor((COLOR_WINDOWTEXT) as i32) as COLORREF,
                    );
                } else {
                    TextOutW(
                        hDC,
                        dx as INT,
                        dy as INT,
                        hex.as_mut_ptr() as LPCWSTR,
                        1 as INT,
                    );
                }
                dx = dx.wrapping_add((*hed).CharWidth as DWORD);
                i += 1;
                x += 1;
            }

            dy = dy.wrapping_add((*hed).LineHeight as DWORD);
            linestart = linestart.wrapping_add((*hed).ColumnsPerLine as DWORD);
            First = First.wrapping_add(1);
        }
    }

    LocalUnlock((*hed).hBuffer);
}

unsafe extern "C" fn HEXEDIT_HitRegionTest(mut hed: PHEXEDIT_DATA, mut pt: POINTS) -> DWORD {
    let mut d: INT = 0;

    if pt.x as INT <= (*hed).LeftMargin {
        return HEHT_LEFTMARGIN as DWORD;
    }

    pt.x = (pt.x as INT - (*hed).LeftMargin as INT) as SHORT;
    d = (4 as INT * (*hed).CharWidth) as INT;
    if pt.x as INT <= d {
        return HEHT_ADDRESS as DWORD;
    }

    pt.x = (pt.x as INT - d) as SHORT;
    d = ((*hed).AddressSpacing * (*hed).CharWidth) as INT;
    if pt.x as INT <= d {
        return HEHT_ADDRESSSPACING as DWORD;
    }

    pt.x = (pt.x as INT - d) as SHORT;
    d = ((3 as INT * (*hed).ColumnsPerLine + 1 as INT) * (*hed).CharWidth) as INT;
    if pt.x as INT <= d {
        return HEHT_HEXDUMP as DWORD;
    }

    pt.x = (pt.x as INT - d) as SHORT;
    d = (((*hed).SplitSpacing - 1 as INT) * (*hed).CharWidth) as INT;
    if pt.x as INT <= d {
        return HEHT_HEXDUMPSPACING as DWORD;
    }

    pt.x = (pt.x as INT - d) as SHORT;
    d = ((*hed).ColumnsPerLine * (*hed).CharWidth) as INT;
    if pt.x as INT <= d {
        return HEHT_ASCIIDUMP as DWORD;
    }

    return HEHT_RIGHTMARGIN as DWORD;
}

unsafe extern "C" fn HEXEDIT_IndexFromPoint(hed: PHEXEDIT_DATA, mut pt: POINTS, Hit: DWORD, EditPos: *mut POINT, EditField: *mut BOOL) -> DWORD {
    let mut si: SCROLLINFO = core::mem::zeroed();
    let mut Index: DWORD = 0;
    let mut bufsize: DWORD = 0;

    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = SIF_POS as UINT;
    GetScrollInfo((*hed).hWndSelf, SB_VERT as INT, core::ptr::addr_of_mut!(si));

    (*EditPos).x = 0 as LONG;

    if (*hed).LineHeight > 0 as INT {
        (*EditPos).y = min!(si.nPos + (pt.y as INT / (*hed).LineHeight), (*hed).nLines - 1);
    } else {
        (*EditPos).y = si.nPos as LONG;
    }

    match Hit {
        HEHT_LEFTMARGIN | HEHT_ADDRESS | HEHT_ADDRESSSPACING | HEHT_HEXDUMP => {
            pt.x = (pt.x as INT
                - ((*hed).LeftMargin as SHORT as INT
                    + (4 + (*hed).AddressSpacing) * (*hed).CharWidth)) as SHORT;
            *EditField = TRUE as BOOL;
        }
        _ => {
            pt.x = (pt.x as INT
                - ((*hed).LeftMargin
                    + (4 + (*hed).AddressSpacing + (*hed).SplitSpacing + (3 * (*hed).ColumnsPerLine))
                        * (*hed).CharWidth)) as SHORT;
            *EditField = FALSE as BOOL;
        }
    }

    if pt.x as INT > 0 as INT {
        let BlockWidth: INT = if *EditField != 0 {
            (*hed).CharWidth * 3
        } else {
            (*hed).CharWidth
        };
        (*EditPos).x = min!((*hed).ColumnsPerLine, (pt.x as INT + BlockWidth / 2) / BlockWidth) as LONG;
    }

    bufsize = if !(*hed).hBuffer.is_null() {
        LocalSize((*hed).hBuffer) as DWORD
    } else {
        0 as DWORD
    };
    Index = (((*EditPos).y * (*hed).ColumnsPerLine as LONG) + (*EditPos).x) as DWORD;
    if Index > bufsize {
        let tmp: INT = (bufsize % (*hed).ColumnsPerLine as DWORD) as INT;
        Index = bufsize;
        (*EditPos).x = (if tmp == 0 as INT {
            (*hed).ColumnsPerLine
        } else {
            tmp
        }) as LONG;
    }
    return Index;
}

unsafe extern "C" fn HEXEDIT_Copy(hed: PHEXEDIT_DATA) {
    let mut pb: PBYTE = core::ptr::null_mut();
    let mut buf: PBYTE = core::ptr::null_mut();
    let mut cb: UINT = 0;
    let mut i0: INT = 0;
    let mut i1: INT = 0;
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();

    if (*hed).SelStart < (*hed).SelEnd {
        i0 = (*hed).SelStart;
        i1 = (*hed).SelEnd;
    } else {
        i0 = (*hed).SelEnd;
        i1 = (*hed).SelStart;
    }

    cb = (i1 - i0) as UINT;
    if cb == 0 {
        return;
    }

    hGlobal = GlobalAlloc(
        (GHND | GMEM_SHARE) as UINT,
        (cb as SIZE_T).wrapping_add(core::mem::size_of::<DWORD>() as SIZE_T),
    );
    if hGlobal.is_null() {
        return;
    }

    pb = GlobalLock(hGlobal) as PBYTE;
    if !pb.is_null() {
        *(pb as PDWORD) = cb as DWORD;
        pb = pb.offset(core::mem::size_of::<DWORD>() as usize as isize);
        buf = LocalLock((*hed).hBuffer) as PBYTE;
        if !buf.is_null() {
            CopyMemory!(pb, buf.offset(i0 as isize), cb as usize);
            LocalUnlock((*hed).hBuffer);
        }
        GlobalUnlock(hGlobal);

        if OpenClipboard((*hed).hWndSelf) != 0 {
            EmptyClipboard();
            SetClipboardData(ClipboardFormatID, hGlobal);
            CloseClipboard();
        }
    } else {
        GlobalFree(hGlobal);
    }
}

unsafe extern "C" fn HEXEDIT_Delete(hed: PHEXEDIT_DATA) {
    let mut buf: PBYTE = ::core::ptr::null_mut::<BYTE>();
    let mut i0: INT = 0;
    let mut i1: INT = 0;
    let mut bufsize: UINT = 0;
    if (*hed).SelStart < (*hed).SelEnd {
        i0 = (*hed).SelStart;
        i1 = (*hed).SelEnd;
    } else {
        i0 = (*hed).SelEnd;
        i1 = (*hed).SelStart;
    }
    if i0 != i1 {
        bufsize = (if !(*hed).hBuffer.is_null() {
            LocalSize((*hed).hBuffer) as UINT
        } else {
            0 as UINT
        }) as UINT;
        buf = LocalLock((*hed).hBuffer) as PBYTE;
        if !buf.is_null() {
            MoveMemory!(
                buf.offset(i0 as isize) as *mut ::core::ffi::c_void,
                buf.offset(i1 as isize) as *const ::core::ffi::c_void,
                bufsize.wrapping_sub(i1 as UINT) as SIZE_T,
            );
            LocalUnlock((*hed).hBuffer);
        }
        HexEdit_SetMaxBufferSize!(
            (*hed).hWndSelf,
            bufsize.wrapping_sub((i1 - i0) as UINT),
        );
        (*hed).InMid = FALSE as BOOL;
        (*hed).SelEnd = i0;
        (*hed).SelStart = (*hed).SelEnd;
        (*hed).Index = (*hed).SelStart;
        (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
        (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
        InvalidateRect(
            (*hed).hWndSelf,
            ::core::ptr::null::<RECT>(),
            TRUE as BOOL,
        );
        HEXEDIT_MoveCaret(hed, TRUE as BOOL);
    }
}

unsafe extern "C" fn HEXEDIT_Paste(hed: PHEXEDIT_DATA) {
    let mut hGlobal: HGLOBAL;
    let mut bufsize: UINT;
    let mut pb: PBYTE;
    let mut buf: PBYTE;
    let mut cb: DWORD;

    HEXEDIT_Delete(hed);
    bufsize = (if !(*hed).hBuffer.is_null() {
        LocalSize((*hed).hBuffer)
    } else {
        0
    }) as UINT;

    if OpenClipboard((*hed).hWndSelf) != 0 {
        hGlobal = GetClipboardData(ClipboardFormatID) as HGLOBAL;
        if !hGlobal.is_null() {
            pb = GlobalLock(hGlobal) as PBYTE;
            cb = *(pb as PDWORD);
            pb = pb.add(::core::mem::size_of::<DWORD>());
            HexEdit_SetMaxBufferSize!((*hed).hWndSelf, (bufsize as DWORD).wrapping_add(cb));
            buf = LocalLock((*hed).hBuffer) as PBYTE;
            if !buf.is_null() {
                MoveMemory!(
                    buf.offset((*hed).Index as isize).offset(cb as isize),
                    buf.offset((*hed).Index as isize),
                    bufsize.wrapping_sub((*hed).Index as UINT) as usize,
                );
                CopyMemory!(
                    buf.offset((*hed).Index as isize),
                    pb,
                    cb as usize,
                );
                LocalUnlock((*hed).hBuffer);
            }
            GlobalUnlock(hGlobal);
        }
        CloseClipboard();
    }
    InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
    HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
}

unsafe extern "C" fn HEXEDIT_Cut(hed: PHEXEDIT_DATA) {
    HEXEDIT_Copy(hed);
    HEXEDIT_Delete(hed);
}

unsafe extern "C" fn HEXEDIT_SelectAll(mut hed: PHEXEDIT_DATA) {
    let mut bufsize: INT = 0;
    bufsize = (if !(*hed).hBuffer.is_null() {
        LocalSize((*hed).hBuffer) as INT
    } else {
        0 as INT
    }) as INT;
    (*hed).Index = 0 as INT;
    (*hed).SelStart = (*hed).Index;
    (*hed).SelEnd = bufsize;
    InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), TRUE as _);
    HEXEDIT_MoveCaret(hed, TRUE as _);
}

unsafe extern "C" fn HEXEDIT_HEM_LOADBUFFER(
    mut hed: PHEXEDIT_DATA,
    mut Buffer: PVOID,
    mut Size: DWORD,
) -> LRESULT {
    if !Buffer.is_null() && Size > 0 {
        let mut buf: LPVOID = ::core::ptr::null_mut::<::core::ffi::c_void>();
        if (*hed).MaxBuffer > 0 && Size > (*hed).MaxBuffer {
            Size = (*hed).MaxBuffer;
        }
        if !(*hed).hBuffer.is_null() {
            if Size > 0 {
                if LocalSize((*hed).hBuffer) != Size as SIZE_T {
                    (*hed).hBuffer = LocalReAlloc(
                        (*hed).hBuffer,
                        Size as SIZE_T,
                        (LMEM_MOVEABLE | LMEM_ZEROINIT) as UINT,
                    );
                }
            } else {
                (*hed).hBuffer = LocalFree((*hed).hBuffer);
                (*hed).Index = 0 as INT;
                HEXEDIT_Update(hed);

                return 0 as LRESULT;
            }
        } else if Size > 0 {
            (*hed).hBuffer = LocalAlloc(LHND as UINT, Size as SIZE_T);
        }
        if Size > 0 {
            buf = LocalLock((*hed).hBuffer) as LPVOID;
            if !buf.is_null() {
                memcpy(
                    buf as *mut ::core::ffi::c_void,
                    Buffer as *const ::core::ffi::c_void,
                    (Size as SIZE_T) as _,
                );
            } else {
                Size = 0;
            }
            LocalUnlock((*hed).hBuffer);
        }

        (*hed).Index = 0 as INT;
        HEXEDIT_Update(hed);
        return Size as LRESULT;
    } else if !(*hed).hBuffer.is_null() {
        (*hed).Index = 0 as INT;
        (*hed).hBuffer = LocalFree((*hed).hBuffer);
        HEXEDIT_Update(hed);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEXEDIT_HEM_COPYBUFFER(hed: PHEXEDIT_DATA, Buffer: PVOID, Size: DWORD) -> LRESULT {
    let mut nCpy: size_t;

    if (*hed).hBuffer.is_null() {
        return 0;
    }

    if !Buffer.is_null() && Size > 0 {
        nCpy = min!(Size as SIZE_T, LocalSize((*hed).hBuffer)) as size_t;
        if nCpy > 0 {
            let mut buf: PVOID = core::ptr::null_mut::<core::ffi::c_void>();

            buf = LocalLock((*hed).hBuffer);
            if !buf.is_null() {
                memcpy(
                    Buffer as *mut core::ffi::c_void,
                    buf as *const core::ffi::c_void,
                    nCpy,
                );
            } else {
                nCpy = 0;
            }
            LocalUnlock((*hed).hBuffer);
        }
        return nCpy as LRESULT;
    }

    return LocalSize((*hed).hBuffer) as LRESULT;
}

unsafe extern "C" fn HEXEDIT_HEM_SETMAXBUFFERSIZE(mut hed: PHEXEDIT_DATA, mut nMaxSize: DWORD) -> LRESULT {
    (*hed).MaxBuffer = nMaxSize;
    if (*hed).MaxBuffer == 0 as DWORD {
        (*hed).hBuffer = LocalFree((*hed).hBuffer);
        return 0 as LRESULT;
    }
    if !(*hed).hBuffer.is_null() {
        (*hed).hBuffer = LocalReAlloc(
            (*hed).hBuffer,
            (*hed).MaxBuffer as SIZE_T,
            LMEM_MOVEABLE as UINT,
        );
    } else {
        (*hed).hBuffer = LocalAlloc(
            LMEM_MOVEABLE as UINT,
            (*hed).MaxBuffer as SIZE_T,
        );
    }
    HEXEDIT_Update(hed);
    return 0 as LRESULT;
}

unsafe extern "C" fn HEXEDIT_WM_NCCREATE(hWnd: HWND, cs: *mut CREATESTRUCT) -> LRESULT {
    let mut hed: PHEXEDIT_DATA = core::ptr::null_mut::<HEXEDIT_DATA>();

    hed = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<HEXEDIT_DATA>() as SIZE_T,
    ) as PHEXEDIT_DATA;
    if hed.is_null() {
        return FALSE as LRESULT;
    }

    (*hed).hWndSelf = hWnd;
    (*hed).hWndParent = (*cs).hwndParent;
    (*hed).style = (*cs).style as DWORD;

    (*hed).ColumnsPerLine = 8 as INT;
    (*hed).LeftMargin = 2 as INT;
    (*hed).AddressSpacing = 2 as INT;
    (*hed).SplitSpacing = 2 as INT;
    (*hed).EditingField = TRUE as BOOL; // in hexdump field

    SetWindowLongPtr(hWnd, 0 as ::core::ffi::c_int, hed as DWORD_PTR as LONG_PTR);
    HEXEDIT_Update(hed);

    return TRUE as LRESULT;
}

unsafe extern "C" fn HEXEDIT_WM_NCDESTROY(mut hed: PHEXEDIT_DATA) -> LRESULT {
    if !(*hed).hBuffer.is_null() {
        //while(LocalUnlock(hed->hBuffer));
        LocalFree((*hed).hBuffer);
    }

    if !(*hed).hFont.is_null() {
        DeleteObject(((*hed).hFont).cast());
    }

    SetWindowLongPtr((*hed).hWndSelf, 0, 0 as LONG_PTR);
    HeapFree(GetProcessHeap(), 0, hed as PVOID);

    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_CREATE(hed: PHEXEDIT_DATA) -> LRESULT {
    UNREFERENCED_PARAMETER!(hed);
    return 1 as LRESULT;
}
unsafe extern "C" fn HEXEDIT_WM_SETFOCUS(mut hed: PHEXEDIT_DATA) -> LRESULT {
    CreateCaret((*hed).hWndSelf, core::ptr::null_mut(), 1, (*hed).LineHeight);
    HEXEDIT_MoveCaret(hed, (FALSE) as BOOL);
    ShowCaret((*hed).hWndSelf);
    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_KILLFOCUS(hed: PHEXEDIT_DATA) -> LRESULT {
    UNREFERENCED_PARAMETER!(hed);
    DestroyCaret();
    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_VSCROLL(hed: PHEXEDIT_DATA, ThumbPosition: WORD, SbCmd: WORD) -> LRESULT {
    let mut ScrollY: ::core::ffi::c_int = 0;
    let mut si: SCROLLINFO = tagSCROLLINFO {
        cbSize: 0,
        fMask: 0,
        nMin: 0,
        nMax: 0,
        nPage: 0,
        nPos: 0,
        nTrackPos: 0,
    };

    UNREFERENCED_PARAMETER!(ThumbPosition);

    ZeroMemory!(
        core::ptr::addr_of_mut!(si) as *mut ::core::ffi::c_void,
        core::mem::size_of::<SCROLLINFO>() as size_t,
    );
    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = SIF_ALL as UINT;
    GetScrollInfo(
        (*hed).hWndSelf,
        SB_VERT as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(si),
    );

    ScrollY = si.nPos;
    match (SbCmd as ::core::ffi::c_int) as u32 {
        SB_TOP => {
            si.nPos = si.nMin;
        }
        SB_BOTTOM => {
            si.nPos = si.nMax;
        }
        SB_LINEUP => {
            si.nPos -= 1;
        }
        SB_LINEDOWN => {
            si.nPos += 1;
        }
        SB_PAGEUP => {
            si.nPos = (si.nPos as UINT).wrapping_sub(si.nPage) as ::core::ffi::c_int;
        }
        SB_PAGEDOWN => {
            si.nPos = (si.nPos as UINT).wrapping_add(si.nPage) as ::core::ffi::c_int;
        }
        SB_THUMBTRACK => {
            si.nPos = si.nTrackPos;
        }
        _ => {}
    }

    si.fMask = SIF_POS as UINT;
    SetScrollInfo(
        (*hed).hWndSelf,
        SB_VERT as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
        (TRUE) as BOOL,
    );
    GetScrollInfo(
        (*hed).hWndSelf,
        SB_VERT as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(si),
    );

    if si.nPos != ScrollY {
        ScrollWindow(
            (*hed).hWndSelf,
            0 as ::core::ffi::c_int,
            (ScrollY - si.nPos) * (*hed).LineHeight as ::core::ffi::c_int,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        UpdateWindow((*hed).hWndSelf);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEXEDIT_WM_SETFONT(mut hed: PHEXEDIT_DATA, mut hNewFont: HFONT, mut bRedraw: BOOL) -> LRESULT {
    let mut hDC: HDC = core::ptr::null_mut();
    let mut tm: TEXTMETRIC = core::mem::zeroed();
    let mut hOldFont: HFONT = core::ptr::null_mut();

    if hNewFont == core::ptr::null_mut() {
        hNewFont = HEXEDIT_GetFixedFont();
    }

    (*hed).hFont = hNewFont;
    hDC = GetDC((*hed).hWndSelf);
    if !hNewFont.is_null() {
        hOldFont = SelectObject(hDC, hNewFont as HGDIOBJ) as HFONT;
    }
    GetTextMetrics(hDC, &raw mut tm);
    (*hed).LineHeight = tm.tmHeight as INT;
    (*hed).CharWidth = tm.tmAveCharWidth as INT;
    if !hOldFont.is_null() {
        SelectObject(hDC, hOldFont as HGDIOBJ);
    }
    ReleaseDC((*hed).hWndSelf, hDC);

    if bRedraw != 0 {
        InvalidateRect((*hed).hWndSelf, core::ptr::null::<RECT>(), (TRUE) as BOOL);
    }

    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_GETFONT(hed: PHEXEDIT_DATA) -> LRESULT {
    return (*hed).hFont as LRESULT;
}

unsafe extern "C" fn HEXEDIT_WM_PAINT(hed: PHEXEDIT_DATA) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut si: SCROLLINFO = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();
    let mut hbmp: HBITMAP = core::ptr::null_mut();
    let mut hbmpold: HBITMAP = core::ptr::null_mut();
    let mut nLines: INT = 0;
    let mut nFirst: INT = 0;
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut hTempDC: HDC = core::ptr::null_mut();
    let mut height: DWORD = 0;

    if GetUpdateRect((*hed).hWndSelf, core::ptr::addr_of_mut!(rc), (FALSE) as BOOL) != 0
        && (*hed).LineHeight > 0
    {
        ZeroMemory!(
            core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void,
            core::mem::size_of::<SCROLLINFO>() as usize,
        );
        si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
        si.fMask = SIF_POS as UINT;
        GetScrollInfo((*hed).hWndSelf, SB_VERT as _, core::ptr::addr_of_mut!(si));

        height = (rc.bottom - rc.top) as DWORD;
        nLines = height.wrapping_div((*hed).LineHeight as DWORD) as INT;
        if (height.wrapping_rem((*hed).LineHeight as DWORD)) > 0 as DWORD {
            nLines += 1;
        }
        if nLines > (*hed).nLines - si.nPos {
            nLines = (*hed).nLines - si.nPos;
        }
        nFirst = rc.top / (*hed).LineHeight;

        BeginPaint((*hed).hWndSelf, core::ptr::addr_of_mut!(ps));
        hTempDC = CreateCompatibleDC(ps.hdc);
        'epaint: {
            if hTempDC.is_null() {
                FillRect(
                    ps.hdc,
                    core::ptr::addr_of_mut!(rc),
                    (COLOR_WINDOW + 1) as HBRUSH,
                );
                break 'epaint;
            }
            hbmp = CreateCompatibleBitmap(ps.hdc, ps.rcPaint.right, ps.rcPaint.bottom);
            if hbmp.is_null() {
                FillRect(
                    ps.hdc,
                    core::ptr::addr_of_mut!(rc),
                    (COLOR_WINDOW + 1) as HBRUSH,
                );
                DeleteDC(hTempDC);
                break 'epaint;
            }
            hbmpold = SelectObject(hTempDC, hbmp as HGDIOBJ) as HBITMAP;
            hOldFont = SelectObject(hTempDC, (*hed).hFont as HGDIOBJ) as HFONT;
            HEXEDIT_PaintLines(
                hed,
                hTempDC,
                si.nPos as DWORD,
                nFirst as DWORD,
                (nFirst + nLines) as DWORD,
                core::ptr::addr_of_mut!(ps.rcPaint),
            );
            BitBlt(
                ps.hdc,
                rc.left,
                rc.top,
                rc.right - rc.left,
                rc.bottom - rc.top,
                hTempDC,
                rc.left,
                rc.top,
                SRCCOPY as DWORD,
            );
            SelectObject(hTempDC, hOldFont as HGDIOBJ);
            SelectObject(hTempDC, hbmpold as HGDIOBJ);
            DeleteObject(hbmp as HGDIOBJ);
            DeleteDC(hTempDC);
        }
        EndPaint((*hed).hWndSelf, core::ptr::addr_of_mut!(ps));
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEXEDIT_WM_MOUSEWHEEL(
    mut hed: PHEXEDIT_DATA,
    mut cyMoveLines: ::core::ffi::c_int,
    mut ButtonsDown: WORD,
    mut MousePos: LPPOINTS,
) -> LRESULT {
    let mut si: SCROLLINFO = tagSCROLLINFO {
        cbSize: 0,
        fMask: 0,
        nMin: 0,
        nMax: 0,
        nPage: 0,
        nPos: 0,
        nTrackPos: 0,
    };
    let mut ScrollY: ::core::ffi::c_int = 0;

    UNREFERENCED_PARAMETER!(ButtonsDown);
    UNREFERENCED_PARAMETER!(MousePos);

    SetFocus((*hed).hWndSelf);

    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = SIF_ALL as UINT;
    GetScrollInfo((*hed).hWndSelf, SB_VERT as ::core::ffi::c_int, core::ptr::addr_of_mut!(si));

    ScrollY = si.nPos;

    si.fMask = SIF_POS as UINT;
    si.nPos += cyMoveLines;
    SetScrollInfo(
        (*hed).hWndSelf,
        SB_VERT as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
        TRUE as BOOL,
    );

    GetScrollInfo((*hed).hWndSelf, SB_VERT as ::core::ffi::c_int, core::ptr::addr_of_mut!(si));
    if si.nPos != ScrollY {
        ScrollWindow(
            (*hed).hWndSelf,
            0 as ::core::ffi::c_int,
            (ScrollY - si.nPos) * (*hed).LineHeight as ::core::ffi::c_int,
            core::ptr::null::<RECT>(),
            core::ptr::null::<RECT>(),
        );
        UpdateWindow((*hed).hWndSelf);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEXEDIT_WM_GETDLGCODE(Msg: LPMSG) -> LRESULT {
    UNREFERENCED_PARAMETER!(Msg);
    return (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
}

unsafe extern "C" fn HEXEDIT_WM_LBUTTONDOWN(mut hed: PHEXEDIT_DATA, mut Buttons: INT, mut Pt: POINTS) -> LRESULT {
    let mut NewField: BOOL = 0;
    let mut EditPos: POINT = POINT { x: 0, y: 0 };
    let mut Hit: DWORD = 0;

    UNREFERENCED_PARAMETER!(Buttons);
    SetFocus((*hed).hWndSelf);

    if (GetAsyncKeyState(VK_SHIFT as ::core::ffi::c_int) as ::core::ffi::c_int) < 0 {
        if (*hed).EditingField != 0 {
            (*hed).Index = HEXEDIT_IndexFromPoint(
                hed,
                Pt,
                HEHT_HEXDUMP,
                core::ptr::addr_of_mut!(EditPos),
                core::ptr::addr_of_mut!(NewField),
            ) as INT;
        } else {
            (*hed).Index = HEXEDIT_IndexFromPoint(
                hed,
                Pt,
                HEHT_ASCIIDUMP,
                core::ptr::addr_of_mut!(EditPos),
                core::ptr::addr_of_mut!(NewField),
            ) as INT;
        }
        (*hed).SelEnd = (*hed).Index;
    } else {
        Hit = HEXEDIT_HitRegionTest(hed, Pt);
        (*hed).Index = HEXEDIT_IndexFromPoint(
            hed,
            Pt,
            Hit,
            core::ptr::addr_of_mut!(EditPos),
            core::ptr::addr_of_mut!(NewField),
        ) as INT;
        (*hed).SelEnd = (*hed).Index;
        (*hed).SelStart = (*hed).SelEnd;
        (*hed).EditingField = NewField;
        SetCapture((*hed).hWndSelf);
    }
    (*hed).CaretCol = EditPos.x as INT;
    (*hed).CaretLine = EditPos.y as INT;
    (*hed).InMid = (FALSE) as i32;
    InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (FALSE) as BOOL);
    HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);

    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_LBUTTONUP(hed: PHEXEDIT_DATA, Buttons: INT, Pt: POINTS) -> LRESULT {
    let mut NewField: BOOL = 0;
    let mut EditPos: POINT = POINT { x: 0, y: 0 };
    if GetCapture() == (*hed).hWndSelf {
        if (*hed).EditingField != 0 {
            (*hed).Index = HEXEDIT_IndexFromPoint(hed, Pt, HEHT_HEXDUMP, &raw mut EditPos, &raw mut NewField) as INT;
        } else {
            (*hed).Index = HEXEDIT_IndexFromPoint(hed, Pt, HEHT_ASCIIDUMP, &raw mut EditPos, &raw mut NewField) as INT;
        }
        (*hed).CaretCol = EditPos.x as INT;
        (*hed).CaretLine = EditPos.y as INT;
        (*hed).SelEnd = (*hed).Index;
        ReleaseCapture();
        InvalidateRect((*hed).hWndSelf, core::ptr::null_mut::<RECT>(), (FALSE) as BOOL);
        HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
    }
    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_MOUSEMOVE(mut hed: PHEXEDIT_DATA, mut Buttons: INT, mut Pt: POINTS) -> LRESULT {
    let mut NewField: BOOL = 0;
    let mut EditPos: POINT = POINT { x: 0, y: 0 };
    if GetCapture() == (*hed).hWndSelf {
        if (*hed).EditingField != 0 {
            (*hed).Index = HEXEDIT_IndexFromPoint(
                hed,
                Pt,
                HEHT_HEXDUMP,
                core::ptr::addr_of_mut!(EditPos),
                core::ptr::addr_of_mut!(NewField),
            ) as INT;
        } else {
            (*hed).Index = HEXEDIT_IndexFromPoint(
                hed,
                Pt,
                HEHT_ASCIIDUMP,
                core::ptr::addr_of_mut!(EditPos),
                core::ptr::addr_of_mut!(NewField),
            ) as INT;
        }
        (*hed).CaretCol = EditPos.x as INT;
        (*hed).CaretLine = EditPos.y as INT;
        (*hed).SelEnd = (*hed).Index;
        InvalidateRect((*hed).hWndSelf, core::ptr::null(), (FALSE) as BOOL);
        HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
    }
    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_KEYDOWN(mut hed: PHEXEDIT_DATA, mut VkCode: INT) -> BOOL {
    let mut bufsize: size_t = 0;
    let mut buf: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut i0: INT = 0;
    let mut i1: INT = 0;

    if ((GetKeyState(VK_MENU as INT) as INT) & 0x8000) != 0 {
        return (FALSE) as BOOL;
    }

    bufsize = if !(*hed).hBuffer.is_null() {
        LocalSize((*hed).hBuffer) as size_t
    } else {
        0 as size_t
    };

    if (*hed).SelStart < (*hed).SelEnd {
        i0 = (*hed).SelStart;
        i1 = (*hed).SelEnd;
    } else {
        i0 = (*hed).SelEnd;
        i1 = (*hed).SelStart;
    }

    match (VkCode) as u32 {
        88 => {
            if (GetAsyncKeyState(VK_SHIFT as INT) as INT) >= 0
                && (GetAsyncKeyState(VK_CONTROL as INT) as INT) < 0
                && (*hed).SelStart != (*hed).SelEnd
            {
                HEXEDIT_Cut(hed);
            } else {
                return (TRUE) as BOOL;
            }
        }
        67 => {
            if (GetAsyncKeyState(VK_SHIFT as INT) as INT) >= 0
                && (GetAsyncKeyState(VK_CONTROL as INT) as INT) < 0
                && (*hed).SelStart != (*hed).SelEnd
            {
                HEXEDIT_Copy(hed);
            } else {
                return (TRUE) as BOOL;
            }
        }
        86 => {
            if (GetAsyncKeyState(VK_SHIFT as INT) as INT) >= 0
                && (GetAsyncKeyState(VK_CONTROL as INT) as INT) < 0
            {
                HEXEDIT_Paste(hed);
            } else {
                return (TRUE) as BOOL;
            }
        }
        65 => {
            if (GetAsyncKeyState(VK_SHIFT as INT) as INT) >= 0
                && (GetAsyncKeyState(VK_CONTROL as INT) as INT) < 0
            {
                HEXEDIT_SelectAll(hed);
            } else {
                return (TRUE) as BOOL;
            }
        }
        VK_INSERT => {
            if (*hed).SelStart != (*hed).SelEnd {
                if (GetAsyncKeyState(VK_SHIFT as INT) as INT) >= 0
                    && (GetAsyncKeyState(VK_CONTROL as INT) as INT) < 0
                {
                    HEXEDIT_Copy(hed);
                }
            }
            if ((GetAsyncKeyState(VK_SHIFT as INT) as INT) < 0)
                && ((GetAsyncKeyState(VK_CONTROL as INT) as INT) >= 0)
            {
                HEXEDIT_Paste(hed);
            }
        }
        VK_DELETE => {
            if ((GetAsyncKeyState(VK_SHIFT as INT) as INT) < 0)
                && ((GetAsyncKeyState(VK_CONTROL as INT) as INT) >= 0)
                && (*hed).SelStart != (*hed).SelEnd
            {
                HEXEDIT_Copy(hed);
            }
            if i0 != i1 {
                buf = LocalLock((*hed).hBuffer) as PBYTE;
                if !buf.is_null() {
                    MoveMemory!(
                        buf.offset(i0 as isize) as LPVOID,
                        buf.offset(i1 as isize) as LPVOID,
                        bufsize.wrapping_sub(i1 as size_t),
                    );
                    LocalUnlock((*hed).hBuffer);
                }
                HexEdit_SetMaxBufferSize!(
                    (*hed).hWndSelf,
                    bufsize.wrapping_sub((i1 - i0) as size_t),
                );
                (*hed).InMid = (FALSE) as i32;
                (*hed).SelEnd = i0;
                (*hed).SelStart = (*hed).SelEnd;
                (*hed).Index = (*hed).SelStart;
                (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
                (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
            } else {
                if (*hed).InMid != 0 && (*hed).EditingField != 0 {
                    buf = LocalLock((*hed).hBuffer) as PBYTE;
                    if !buf.is_null() {
                        MoveMemory!(
                            buf.offset((*hed).Index as isize) as LPVOID,
                            buf.offset(((*hed).Index + 1) as isize) as LPVOID,
                            bufsize
                                .wrapping_sub((*hed).Index as size_t)
                                .wrapping_sub(1 as size_t),
                        );
                        LocalUnlock((*hed).hBuffer);
                    }
                    HexEdit_SetMaxBufferSize!(
                        (*hed).hWndSelf,
                        bufsize.wrapping_sub(1 as size_t),
                    );
                    (*hed).InMid = (FALSE) as i32;
                } else if ((*hed).Index as size_t) < bufsize {
                    buf = LocalLock((*hed).hBuffer) as PBYTE;
                    if !buf.is_null() {
                        MoveMemory!(
                            buf.offset((*hed).Index as isize) as LPVOID,
                            buf.offset(((*hed).Index + 1) as isize) as LPVOID,
                            bufsize
                                .wrapping_sub((*hed).Index as size_t)
                                .wrapping_sub(1 as size_t),
                        );
                        LocalUnlock((*hed).hBuffer);
                    }
                    HexEdit_SetMaxBufferSize!(
                        (*hed).hWndSelf,
                        bufsize.wrapping_sub(1 as size_t),
                    );
                }
            }
            InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
            HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
        }
        VK_BACK => {
            if i0 != i1 {
                buf = LocalLock((*hed).hBuffer) as PBYTE;
                if !buf.is_null() {
                    MoveMemory!(
                        buf.offset(i0 as isize) as LPVOID,
                        buf.offset(i1 as isize) as LPVOID,
                        bufsize.wrapping_sub(i1 as size_t),
                    );
                    LocalUnlock((*hed).hBuffer);
                }
                HexEdit_SetMaxBufferSize!(
                    (*hed).hWndSelf,
                    bufsize.wrapping_sub((i1 - i0) as size_t),
                );
                (*hed).InMid = (FALSE) as i32;
                (*hed).SelEnd = i0;
                (*hed).SelStart = (*hed).SelEnd;
                (*hed).Index = (*hed).SelStart;
                (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
                (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
            } else {
                if (*hed).InMid != 0 && (*hed).EditingField != 0 {
                    buf = LocalLock((*hed).hBuffer) as PBYTE;
                    if !buf.is_null() {
                        MoveMemory!(
                            buf.offset((*hed).Index as isize) as LPVOID,
                            buf.offset(((*hed).Index + 1) as isize) as LPVOID,
                            bufsize
                                .wrapping_sub((*hed).Index as size_t)
                                .wrapping_sub(1 as size_t),
                        );
                        LocalUnlock((*hed).hBuffer);
                    }
                } else if (*hed).Index > 0 {
                    buf = LocalLock((*hed).hBuffer) as PBYTE;
                    if !buf.is_null() {
                        MoveMemory!(
                            buf.offset(((*hed).Index - 1) as isize) as LPVOID,
                            buf.offset((*hed).Index as isize) as LPVOID,
                            bufsize.wrapping_sub((*hed).Index as size_t),
                        );
                        LocalUnlock((*hed).hBuffer);
                    }
                    (*hed).Index -= 1;
                    (*hed).SelEnd = (*hed).Index;
                    (*hed).SelStart = (*hed).SelEnd;
                    (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
                    (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
                } else {
                    return (TRUE) as BOOL;
                }
                HexEdit_SetMaxBufferSize!(
                    (*hed).hWndSelf,
                    bufsize.wrapping_sub(1 as size_t),
                );
                (*hed).InMid = (FALSE) as i32;
            }
            InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
            HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
        }
        VK_LEFT => {
            if (*hed).Index > 0 {
                (*hed).Index -= 1;
                if (GetAsyncKeyState(VK_SHIFT as INT) as INT) < 0 {
                    (*hed).SelEnd = (*hed).Index;
                } else {
                    (*hed).SelEnd = (*hed).Index;
                    (*hed).SelStart = (*hed).SelEnd;
                }
                (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
                (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
                (*hed).InMid = (FALSE) as i32;
                InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
                HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
            }
        }
        VK_RIGHT => {
            if (*hed).Index < (bufsize as INT) {
                (*hed).Index += 1;
                if (GetAsyncKeyState(VK_SHIFT as INT) as INT) < 0 {
                    (*hed).SelEnd = (*hed).Index;
                } else {
                    (*hed).SelEnd = (*hed).Index;
                    (*hed).SelStart = (*hed).SelEnd;
                }
                (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
                (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
                (*hed).InMid = (FALSE) as i32;
                InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
                HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
            }
        }
        VK_UP => {
            if (*hed).Index >= (*hed).ColumnsPerLine {
                (*hed).Index -= (*hed).ColumnsPerLine;
                if (GetAsyncKeyState(VK_SHIFT as INT) as INT) < 0 {
                    (*hed).SelEnd = (*hed).Index;
                } else {
                    (*hed).SelEnd = (*hed).Index;
                    (*hed).SelStart = (*hed).SelEnd;
                }
                (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
                (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
                (*hed).InMid = (FALSE) as i32;
                InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
                HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
            }
        }
        VK_DOWN => {
            if (*hed).Index + (*hed).ColumnsPerLine <= (bufsize as INT) {
                (*hed).Index += (*hed).ColumnsPerLine;
            } else {
                (*hed).Index = bufsize as INT;
            }
            (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
            (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
            if (GetAsyncKeyState(VK_SHIFT as INT) as INT) < 0 {
                (*hed).SelEnd = (*hed).Index;
            } else {
                (*hed).SelStart = (*hed).SelEnd;
                (*hed).SelEnd = (*hed).Index;
                (*hed).SelStart = (*hed).SelEnd;
            }
            (*hed).InMid = (FALSE) as i32;
            InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
            HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
        }
        _ => return (TRUE) as i32,
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn HEXEDIT_WM_CHAR(hed: PHEXEDIT_DATA, wch: WCHAR) -> BOOL {
    let mut bufsize: usize = 0;
    let ch: CHAR = wch as CHAR; // keep the lowest octet.
    let mut buf: PBYTE = core::ptr::null_mut();
    let mut i0: INT = 0;
    let mut i1: INT = 0;

    bufsize = if !(*hed).hBuffer.is_null() {
        LocalSize((*hed).hBuffer) as usize
    } else {
        0 as usize
    };
    if (*hed).SelStart < (*hed).SelEnd {
        i0 = (*hed).SelStart;
        i1 = (*hed).SelEnd;
    } else {
        i0 = (*hed).SelEnd;
        i1 = (*hed).SelStart;
    }
    if (*hed).EditingField == 0 {
        if (0x20 <= (ch as i32)) && ((ch as i32) <= 0xFF) {
            if (*hed).SelStart != (*hed).SelEnd {
                buf = LocalLock((*hed).hBuffer) as PBYTE;
                if !buf.is_null() {
                    MoveMemory!(
                        buf.offset(i0 as isize) as *mut core::ffi::c_void,
                        buf.offset(i1 as isize) as *const core::ffi::c_void,
                        bufsize.wrapping_sub(i1 as usize),
                    );
                    LocalUnlock((*hed).hBuffer);
                }
                HexEdit_SetMaxBufferSize!(
                    (*hed).hWndSelf,
                    bufsize.wrapping_sub((i1 - i0) as usize) as LPARAM,
                );
                (*hed).InMid = (FALSE) as i32;
                bufsize = if !(*hed).hBuffer.is_null() {
                    LocalSize((*hed).hBuffer) as usize
                } else {
                    0 as usize
                };
                (*hed).SelEnd = i0;
                (*hed).SelStart = (*hed).SelEnd;
                (*hed).Index = (*hed).SelStart;
            }
            HexEdit_SetMaxBufferSize!(
                (*hed).hWndSelf,
                bufsize.wrapping_add(1 as usize) as LPARAM,
            );
            buf = LocalLock((*hed).hBuffer) as PBYTE;
            if !buf.is_null() {
                MoveMemory!(
                    buf.offset((*hed).Index as isize + 1 as isize)
                        as *mut core::ffi::c_void,
                    buf.offset((*hed).Index as isize) as *const core::ffi::c_void,
                    bufsize.wrapping_sub((*hed).Index as usize),
                );
                *buf.offset((*hed).Index as isize) = ch as BYTE;
                LocalUnlock((*hed).hBuffer);
            }
            (*hed).Index += 1;
            (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
            (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
            InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
            HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
            return (FALSE) as BOOL;
        }
    } else {
        if (('0' as i32 <= (ch as i32)) && ((ch as i32) <= '9' as i32))
            || (('A' as i32 <= (ch as i32)) && ((ch as i32) <= 'F' as i32))
            || (('a' as i32 <= (ch as i32)) && ((ch as i32) <= 'f' as i32))
        {
            if (*hed).SelStart != (*hed).SelEnd {
                buf = LocalLock((*hed).hBuffer) as PBYTE;
                if !buf.is_null() {
                    MoveMemory!(
                        buf.offset(i0 as isize) as *mut core::ffi::c_void,
                        buf.offset(i1 as isize) as *const core::ffi::c_void,
                        bufsize.wrapping_sub(i1 as usize),
                    );
                    LocalUnlock((*hed).hBuffer);
                }
                HexEdit_SetMaxBufferSize!(
                    (*hed).hWndSelf,
                    bufsize.wrapping_sub((i1 - i0) as usize) as LPARAM,
                );
                (*hed).InMid = (FALSE) as i32;
                bufsize = if !(*hed).hBuffer.is_null() {
                    LocalSize((*hed).hBuffer) as usize
                } else {
                    0 as usize
                };
                (*hed).SelEnd = i0;
                (*hed).SelStart = (*hed).SelEnd;
                (*hed).Index = (*hed).SelStart;
            }
            if (*hed).InMid != 0 {
                buf = LocalLock((*hed).hBuffer) as PBYTE;
                if !buf.is_null() {
                    if ('0' as i32 <= (ch as i32)) && ((ch as i32) <= '9' as i32) {
                        *buf.offset((*hed).Index as isize) |= ((ch as i32) - ('0' as i32)) as BYTE;
                    } else if ('A' as i32 <= (ch as i32)) && ((ch as i32) <= 'F' as i32) {
                        *buf.offset((*hed).Index as isize) |=
                            ((ch as i32) + 10 - ('A' as i32)) as BYTE;
                    } else if ('a' as i32 <= (ch as i32)) && ((ch as i32) <= 'f' as i32) {
                        *buf.offset((*hed).Index as isize) |=
                            ((ch as i32) + 10 - ('a' as i32)) as BYTE;
                    }
                    LocalUnlock((*hed).hBuffer);
                }
                (*hed).InMid = (FALSE) as i32;
                (*hed).Index += 1;
            } else {
                HexEdit_SetMaxBufferSize!(
                    (*hed).hWndSelf,
                    bufsize.wrapping_add(1 as usize) as LPARAM,
                );
                buf = LocalLock((*hed).hBuffer) as PBYTE;
                if !buf.is_null() {
                    MoveMemory!(
                        buf.offset((*hed).Index as isize + 1 as isize)
                            as *mut core::ffi::c_void,
                        buf.offset((*hed).Index as isize) as *const core::ffi::c_void,
                        bufsize.wrapping_sub((*hed).Index as usize),
                    );
                    if ('0' as i32 <= (ch as i32)) && ((ch as i32) <= '9' as i32) {
                        *buf.offset((*hed).Index as isize) =
                            (((ch as i32) - ('0' as i32)) << 4) as BYTE;
                    } else if ('A' as i32 <= (ch as i32)) && ((ch as i32) <= 'F' as i32) {
                        *buf.offset((*hed).Index as isize) =
                            (((ch as i32) + 10 - ('A' as i32))
                                << 4) as BYTE;
                    } else if ('a' as i32 <= (ch as i32)) && ((ch as i32) <= 'f' as i32) {
                        *buf.offset((*hed).Index as isize) =
                            (((ch as i32) + 10 - ('a' as i32))
                                << 4) as BYTE;
                    }
                    LocalUnlock((*hed).hBuffer);
                }
                (*hed).InMid = (TRUE) as i32;
            }
            (*hed).CaretCol = (*hed).Index % (*hed).ColumnsPerLine;
            (*hed).CaretLine = (*hed).Index / (*hed).ColumnsPerLine;
            InvalidateRect((*hed).hWndSelf, core::ptr::null_mut(), (TRUE) as BOOL);
            HEXEDIT_MoveCaret(hed, (TRUE) as BOOL);
            return (FALSE) as BOOL;
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn HEXEDIT_WM_SIZE(hed: PHEXEDIT_DATA, sType: DWORD, NewWidth: WORD, NewHeight: WORD) -> LRESULT {
    UNREFERENCED_PARAMETER!(sType);
    UNREFERENCED_PARAMETER!(NewHeight);
    UNREFERENCED_PARAMETER!(NewWidth);
    HEXEDIT_Update(hed);
    return 0;
}

unsafe extern "C" fn HEXEDIT_WM_CONTEXTMENU(mut hed: PHEXEDIT_DATA, mut x: INT, mut y: INT) {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();

    if x == -(1 as INT) && y == -(1 as INT) {
        GetWindowRect((*hed).hWndSelf, core::ptr::addr_of_mut!(rc));
        x = rc.left as INT;
        y = rc.top as INT;
    }

    hMenu = GetSubMenu(hPopupMenus, (PM_HEXEDIT as UINT) as i32);
    if (*hed).SelStart == (*hed).SelEnd {
        EnableMenuItem(hMenu, ID_HEXEDIT_CUT as UINT, MF_GRAYED as UINT);
        EnableMenuItem(hMenu, ID_HEXEDIT_COPY as UINT, MF_GRAYED as UINT);
        EnableMenuItem(hMenu, ID_HEXEDIT_PASTE as UINT, MF_GRAYED as UINT);
        EnableMenuItem(hMenu, ID_HEXEDIT_DELETE as UINT, MF_GRAYED as UINT);
    } else {
        EnableMenuItem(hMenu, ID_HEXEDIT_CUT as UINT, MF_ENABLED as UINT);
        EnableMenuItem(hMenu, ID_HEXEDIT_COPY as UINT, MF_ENABLED as UINT);
        EnableMenuItem(hMenu, ID_HEXEDIT_PASTE as UINT, MF_ENABLED as UINT);
        EnableMenuItem(hMenu, ID_HEXEDIT_DELETE as UINT, MF_ENABLED as UINT);
    }

    SetForegroundWindow((*hed).hWndSelf);
    TrackPopupMenu(
        hMenu,
        TPM_RIGHTBUTTON as UINT,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        (*hed).hWndSelf,
        core::ptr::null_mut(),
    );
    PostMessageW(
        (*hed).hWndSelf,
        WM_NULL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HexEditWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut hed: PHEXEDIT_DATA = core::ptr::null_mut();
    let mut p: POINTS = POINTS { x: 0, y: 0 };

    hed = GetWindowLongPtr(hWnd, 0 as DWORD_PTR as ::core::ffi::c_int) as PHEXEDIT_DATA;
    match uMsg {
        WM_ERASEBKGND => {
            return TRUE as LRESULT;
        }
        WM_PAINT => {
            return HEXEDIT_WM_PAINT(hed);
        }
        WM_KEYDOWN => {
            return HEXEDIT_WM_KEYDOWN(hed, wParam as INT) as LRESULT;
        }
        WM_CHAR => {
            return HEXEDIT_WM_CHAR(hed, wParam as WCHAR) as LRESULT;
        }
        WM_VSCROLL => {
            return HEXEDIT_WM_VSCROLL(hed, HIWORD!(wParam), LOWORD!(wParam));
        }
        WM_SIZE => {
            return HEXEDIT_WM_SIZE(hed, wParam as DWORD, LOWORD!(lParam), HIWORD!(lParam));
        }
        WM_LBUTTONDOWN => {
            p.x = LOWORD!(lParam) as SHORT;
            p.y = HIWORD!(lParam) as SHORT;
            return HEXEDIT_WM_LBUTTONDOWN(hed, wParam as INT, p);
        }
        WM_LBUTTONUP => {
            p.x = LOWORD!(lParam) as SHORT;
            p.y = HIWORD!(lParam) as SHORT;
            return HEXEDIT_WM_LBUTTONUP(hed, wParam as INT, p);
        }
        WM_MOUSEMOVE => {
            p.x = LOWORD!(lParam) as SHORT;
            p.y = HIWORD!(lParam) as SHORT;
            return HEXEDIT_WM_MOUSEMOVE(hed, wParam as INT, p);
        }
        WM_MOUSEWHEEL => {
            let mut nScrollLines: UINT = 3 as UINT;
            let mut delta: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

            SystemParametersInfoW(
                SPI_GETWHEELSCROLLLINES as UINT,
                0 as UINT,
                core::ptr::addr_of_mut!(nScrollLines) as PVOID,
                0 as UINT,
            );
            delta -= HIWORD!(wParam) as SHORT as ::core::ffi::c_int;
            if abs(delta) >= WHEEL_DELTA as ::core::ffi::c_int
                && nScrollLines != 0 as UINT
            {
                p.x = LOWORD!(lParam) as SHORT;
                p.y = HIWORD!(lParam) as SHORT;
                return HEXEDIT_WM_MOUSEWHEEL(
                    hed,
                    nScrollLines.wrapping_mul((delta / WHEEL_DELTA as ::core::ffi::c_int) as UINT) as ::core::ffi::c_int,
                    LOWORD!(wParam),
                    core::ptr::addr_of_mut!(p),
                );
            }
        }
        HEM_LOADBUFFER => {
            return HEXEDIT_HEM_LOADBUFFER(hed, wParam as PVOID, lParam as DWORD);
        }
        HEM_COPYBUFFER => {
            return HEXEDIT_HEM_COPYBUFFER(hed, wParam as PVOID, lParam as DWORD);
        }
        HEM_SETMAXBUFFERSIZE => {
            return HEXEDIT_HEM_SETMAXBUFFERSIZE(hed, lParam as DWORD);
        }
        WM_SETFOCUS => {
            return HEXEDIT_WM_SETFOCUS(hed);
        }
        WM_KILLFOCUS => {
            return HEXEDIT_WM_KILLFOCUS(hed);
        }
        WM_GETDLGCODE => {
            return HEXEDIT_WM_GETDLGCODE(lParam as LPMSG);
        }
        WM_SETFONT => {
            return HEXEDIT_WM_SETFONT(hed, wParam as HFONT, LOWORD!(lParam) as BOOL);
        }
        WM_GETFONT => {
            return HEXEDIT_WM_GETFONT(hed);
        }
        WM_CREATE => {
            return HEXEDIT_WM_CREATE(hed);
        }
        WM_NCCREATE => {
            if hed.is_null() {
                return HEXEDIT_WM_NCCREATE(hWnd, lParam as *mut CREATESTRUCT);
            }
        }
        WM_NCDESTROY => {
            if !hed.is_null() {
                return HEXEDIT_WM_NCDESTROY(hed);
            }
        }
        WM_CONTEXTMENU => {
            HEXEDIT_WM_CONTEXTMENU(
                hed,
                LOWORD!(lParam) as SHORT as INT,
                HIWORD!(lParam) as SHORT as INT,
            );
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                ID_HEXEDIT_CUT => {
                    HEXEDIT_Cut(hed);
                }
                ID_HEXEDIT_COPY => {
                    HEXEDIT_Copy(hed);
                }
                ID_HEXEDIT_PASTE => {
                    HEXEDIT_Paste(hed);
                }
                ID_HEXEDIT_DELETE => {
                    HEXEDIT_Delete(hed);
                }
                ID_HEXEDIT_SELECT_ALL => {
                    HEXEDIT_SelectAll(hed);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return DefWindowProcW(hWnd, uMsg, wParam, lParam);
}

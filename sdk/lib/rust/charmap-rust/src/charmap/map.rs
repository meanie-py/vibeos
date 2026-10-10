use crate::charmap::lrgcell::LrgCellWndProc;
use crate::charmap::charmap::UpdateStatusBar;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut codePages: [UINT; 31] = [
    864 as ::core::ffi::c_int as UINT,
    775 as ::core::ffi::c_int as UINT,
    863 as ::core::ffi::c_int as UINT,
    855 as ::core::ffi::c_int as UINT,
    737 as ::core::ffi::c_int as UINT,
    856 as ::core::ffi::c_int as UINT,
    862 as ::core::ffi::c_int as UINT,
    861 as ::core::ffi::c_int as UINT,
    852 as ::core::ffi::c_int as UINT,
    869 as ::core::ffi::c_int as UINT,
    850 as ::core::ffi::c_int as UINT,
    858 as ::core::ffi::c_int as UINT,
    865 as ::core::ffi::c_int as UINT,
    860 as ::core::ffi::c_int as UINT,
    866 as ::core::ffi::c_int as UINT,
    857 as ::core::ffi::c_int as UINT,
    437 as ::core::ffi::c_int as UINT,
    1256 as ::core::ffi::c_int as UINT,
    1257 as ::core::ffi::c_int as UINT,
    1250 as ::core::ffi::c_int as UINT,
    1251 as ::core::ffi::c_int as UINT,
    1253 as ::core::ffi::c_int as UINT,
    1255 as ::core::ffi::c_int as UINT,
    932 as ::core::ffi::c_int as UINT,
    949 as ::core::ffi::c_int as UINT,
    1252 as ::core::ffi::c_int as UINT,
    936 as ::core::ffi::c_int as UINT,
    874 as ::core::ffi::c_int as UINT,
    950 as ::core::ffi::c_int as UINT,
    1254 as ::core::ffi::c_int as UINT,
    1258 as ::core::ffi::c_int as UINT,
];

static mut szMapWndClass: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"F\0o\0n\0t\0M\0a\0p\0W\0n\0d\0\0\0")
};

static mut szLrgCellWndClass: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"L\0r\0g\0C\0e\0l\0l\0W\0n\0d\0\0\0")
};

// C: #define MAX_ROWS (0xFFFF / XCELLS) + 1 - YCELLS
// Depends on XCELLS and YCELLS, which are not defined in this preamble; they must be in scope.
pub const MAX_ROWS: i32 = ((0xFFFF / XCELLS) + 1 - YCELLS) as i32;

unsafe extern "C" fn SetGrid(mut infoPtr: PMAP) {
    let mut x: INT;
    let mut y: INT;
    let mut Cell: PCELL;
    y = 0;
    while y < (YCELLS as i32) {
        x = 0;
        while x < (XCELLS as i32) {
            Cell = &raw mut (*infoPtr).Cells[y as usize][x as usize];
            (*Cell).CellExt.left = x as LONG * (*infoPtr).CellSize.cx + 1;
            (*Cell).CellExt.top = y as LONG * (*infoPtr).CellSize.cy + 1;
            (*Cell).CellExt.right = (x as LONG + 1) * (*infoPtr).CellSize.cx + 2;
            (*Cell).CellExt.bottom = (y as LONG + 1) * (*infoPtr).CellSize.cy + 2;

            (*Cell).CellInt = (*Cell).CellExt;

            InflateRect(&raw mut (*Cell).CellInt, -1, -1);
            x += 1;
        }
        y += 1;
    }
}

unsafe extern "C" fn UpdateCells(mut infoPtr: PMAP) {
    let mut x: INT;
    let mut y: INT;
    let mut i: INT = (XCELLS * ((*infoPtr).iYStart) as u32) as i32;
    let mut ch: WCHAR;
    let mut Cell: PCELL;
    y = 0;
    while y < (YCELLS as i32) {
        x = 0;
        while x < (XCELLS as i32) {
            if i < (*infoPtr).NumValidGlyphs as INT {
                ch = (*infoPtr).ValidGlyphs[i as usize];
            } else {
                ch = 0xFFFF;
            }
            Cell = &raw mut (*infoPtr).Cells[y as usize][x as usize];
            (*Cell).ch = ch;
            x += 1;
            i += 1;
        }
        y += 1;
    }
}

unsafe extern "C" fn FillGrid(infoPtr: PMAP, ps: *mut PAINTSTRUCT) {
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut Cell: PCELL = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut hOldBrush: HBRUSH = core::ptr::null_mut();
    let mut hbrGray: HBRUSH = GetStockObject((LTGRAY_BRUSH as i32)) as HBRUSH;
    let mut hOldPen: HPEN = core::ptr::null_mut();
    let mut hPenGray: HPEN = CreatePen(
        (PS_SOLID as i32),
        1,
        RGB!(140, 140, 140),
    ) as HPEN;

    UpdateCells(infoPtr);

    hOldFont = SelectObject((*ps).hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
    hOldPen = SelectObject(
        (*ps).hdc,
        GetStockObject((BLACK_PEN as i32)) as HGDIOBJ,
    ) as HPEN;
    hOldBrush = SelectObject(
        (*ps).hdc,
        GetStockObject((WHITE_BRUSH as i32)) as HGDIOBJ,
    ) as HBRUSH;

    i = (XCELLS * ((*infoPtr).iYStart) as u32) as i32;

    y = 0;
    while y < (YCELLS as i32) {
        x = 0;
        while x < (XCELLS as i32) {
            Cell = &raw mut (*infoPtr).Cells[y as usize][x as usize] as PCELL;
            if IntersectRect(
                &raw mut rc,
                &raw mut (*ps).rcPaint,
                &raw mut (*Cell).CellExt,
            ) != 0
            {
                rc = (*Cell).CellExt;
                Rectangle((*ps).hdc, rc.left, rc.top, rc.right, rc.bottom);

                if i < (*infoPtr).NumValidGlyphs as INT {
                    DrawTextW(
                        (*ps).hdc,
                        &raw mut (*Cell).ch,
                        1,
                        &raw mut (*Cell).CellInt,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                    if Cell == (*infoPtr).pActiveCell {
                        rc = (*Cell).CellInt;

                        /* Draw gray box */
                        SelectObject(
                            (*ps).hdc,
                            GetStockObject((NULL_BRUSH as i32)) as HGDIOBJ,
                        );
                        SelectObject((*ps).hdc, hPenGray as HGDIOBJ);
                        Rectangle((*ps).hdc, rc.left, rc.top, rc.right, rc.bottom);
                        SelectObject((*ps).hdc, hOldPen as HGDIOBJ);
                        SelectObject((*ps).hdc, hOldBrush as HGDIOBJ);

                        if GetFocus() == (*infoPtr).hMapWnd {
                            /* Draw focus rectangle */
                            InflateRect(
                                &raw mut rc,
                                -1,
                                -1,
                            );
                            DrawFocusRect((*ps).hdc, &raw mut rc);
                        }
                    }
                } else {
                    FillRect(
                        (*ps).hdc,
                        &raw mut (*Cell).CellInt,
                        hbrGray,
                    );
                }
            }
            x += 1;
            i += 1;
        }
        y += 1;
    }

    SelectObject((*ps).hdc, hOldFont as HGDIOBJ);
    SelectObject((*ps).hdc, hOldPen as HGDIOBJ);
    SelectObject((*ps).hdc, hOldBrush as HGDIOBJ);
    DeleteObject(hPenGray as HGDIOBJ);
}

unsafe extern "C" fn CreateLargeCell(infoPtr: PMAP) -> BOOL {
    let mut rLarge: RECT = (*(*infoPtr).pActiveCell).CellExt;

    MapWindowPoints(
        (*infoPtr).hMapWnd,
        (*infoPtr).hParent,
        core::ptr::addr_of_mut!(rLarge) as LPPOINT,
        2 as UINT,
    );

    InflateRect(
        core::ptr::addr_of_mut!(rLarge),
        (XLARGE - XCELLS) as i32,
        (YLARGE - YCELLS) as i32,
    );

    (*infoPtr).hLrgWnd = CreateWindowExW(
        0 as DWORD,
        core::ptr::addr_of!(szLrgCellWndClass) as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        (WS_CHILDWINDOW | WS_VISIBLE) as DWORD,
        rLarge.left as i32,
        rLarge.top as i32,
        (rLarge.right - rLarge.left) as i32,
        (rLarge.bottom - rLarge.top) as i32,
        (*infoPtr).hParent,
        core::ptr::null_mut(),
        hInstance,
        infoPtr as LPVOID,
    );
    if (*infoPtr).hLrgWnd.is_null() {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn MoveLargeCell(infoPtr: PMAP) {
    let mut rLarge: RECT = (*(*infoPtr).pActiveCell).CellExt;

    MapWindowPoints((*infoPtr).hMapWnd, (*infoPtr).hParent, &raw mut rLarge as LPPOINT, 2);

    InflateRect(&raw mut rLarge, (XLARGE - XCELLS) as i32, (YLARGE - YCELLS) as i32);

    MoveWindow(
        (*infoPtr).hLrgWnd,
        rLarge.left as i32,
        rLarge.top as i32,
        (rLarge.right - rLarge.left) as i32,
        (rLarge.bottom - rLarge.top) as i32,
        (TRUE) as BOOL,
    );

    InvalidateRect((*infoPtr).hLrgWnd, core::ptr::null::<RECT>(), (TRUE) as BOOL);
}

unsafe extern "C" fn GetPossibleCharacters(ch: *mut WCHAR, chLen: INT, codePageIdx: INT) {
    let mut i: INT = 0;
    let mut j: INT = 0;
    ZeroMemory!(
        ch as *mut core::ffi::c_void,
        core::mem::size_of_val(&*ch.add(0 as usize)) * (chLen as usize),
    );
    if codePageIdx <= 0 || codePageIdx > SIZEOF!(codePages) as INT {
        /* this is unicode, so just load up the first MAX_GLYPHS characters
           start at 0x21 to bypass whitespace characters */
        let len: INT = min(MAX_GLYPHS as INT, chLen);
        i = 0x21 as INT;
        j = 0 as INT;
        while i < len {
            *ch.offset(j as isize) = i as WCHAR;
            j += 1;
            i += 1;
        }
    } else {
        /* This is a codepage, so use NLS to translate the first 256 characters */
        let mut multiByteString: [CHAR; 256 as usize] =
            [0 as CHAR; 256 as usize];
        i = 0x21 as INT;
        while i < SIZEOF!(multiByteString) as INT {
            multiByteString[i as usize] = i as CHAR;
            i += 1;
        }

        if MultiByteToWideChar(
            codePages[(codePageIdx - 1) as usize],
            0 as DWORD,
            multiByteString.as_mut_ptr() as LPCSTR,
            (256 as usize * core::mem::size_of::<CHAR>()) as INT,
            ch as LPWSTR,
            chLen,
        ) == 0
        {
            /* Failed for some reason, so clear the array */
            memset(
                ch as *mut core::ffi::c_void,
                0 as core::ffi::c_int,
                core::mem::size_of_val(&*ch.add(0 as usize)) * (chLen as usize),
            );
        }
    }
}

unsafe extern "C" fn SetFont(mut infoPtr: PMAP, mut lpFontName: LPWSTR) {
    let mut hdc: HDC = core::ptr::null_mut::<HDC__>();
    let mut ch: [WCHAR; (MAX_GLYPHS) as _] = [0; (MAX_GLYPHS) as _];
    let mut out: [WORD; (MAX_GLYPHS) as _] = [0; (MAX_GLYPHS) as _];
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;

    /* Destroy Zoom window, since it was created with older font */
    DestroyWindow((*infoPtr).hLrgWnd);
    (*infoPtr).hLrgWnd = core::ptr::null_mut::<HWND__>();

    if !(*infoPtr).hFont.is_null() {
        DeleteObject((*infoPtr).hFont as HGDIOBJ);
    }

    ZeroMemory!(
        &raw mut (*infoPtr).CurrentFont as *mut core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>() as usize,
    );

    hdc = GetDC((*infoPtr).hMapWnd);
    (*infoPtr).CurrentFont.lfHeight =
        (GetDeviceCaps(hdc, (LOGPIXELSY as i32)) / 5) as LONG;

    (*infoPtr).CurrentFont.lfCharSet = DEFAULT_CHARSET as BYTE;
    lstrcpynW(
        &raw mut (*infoPtr).CurrentFont.lfFaceName as LPWSTR,
        lpFontName as LPCWSTR,
        (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>()) as INT,
    );

    (*infoPtr).hFont = CreateFontIndirectW(&raw mut (*infoPtr).CurrentFont);

    InvalidateRect(
        (*infoPtr).hMapWnd,
        core::ptr::null::<RECT>(),
        (TRUE) as BOOL,
    );

    // Get all the valid glyphs in this font

    SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ);

    // Get the code page associated with the selected 'character set'
    GetPossibleCharacters(
        &raw mut ch as *mut WCHAR,
        (MAX_GLYPHS) as INT,
        (*infoPtr).CharMap,
    );

    if GetGlyphIndicesW(
        hdc,
        &raw mut ch as *mut WCHAR as LPCWSTR,
        (MAX_GLYPHS as i32),
        &raw mut out as LPWORD,
        GGI_MARK_NONEXISTING_GLYPHS,
    ) != GDI_ERROR as DWORD
    {
        j = 0;
        i = 0;
        while i < MAX_GLYPHS {
            if out[i as usize] as INT != 0xffff
                && out[i as usize] as INT != 0x0000
                && ch[i as usize] as INT != 0x0000
            {
                (*infoPtr).ValidGlyphs[j as usize] = ch[i as usize] as USHORT;
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        (*infoPtr).NumValidGlyphs = j as USHORT;
    }

    ReleaseDC((*infoPtr).hMapWnd, hdc);

    (*infoPtr).NumRows = (((*infoPtr).NumValidGlyphs as INT) / ((XCELLS)) as i32) as INT;
    if ((*infoPtr).NumValidGlyphs as INT) % ((XCELLS) as i32) != 0 {
        (*infoPtr).NumRows += 1;
    }
    (*infoPtr).NumRows = (if (*infoPtr).NumRows > (YCELLS as i32) {
        (*infoPtr).NumRows - (YCELLS as i32)
    } else {
        0
    }) as INT;

    SetScrollRange(
        (*infoPtr).hMapWnd,
        (SB_VERT) as i32,
        0,
        (*infoPtr).NumRows,
        (FALSE) as BOOL,
    );
    SetScrollPos(
        (*infoPtr).hMapWnd,
        (SB_VERT) as i32,
        0,
        (TRUE) as BOOL,
    );
    (*infoPtr).iYStart = 0;
}

unsafe extern "C" fn NotifyParentOfSelection(
    mut infoPtr: PMAP,
    mut code: UINT,
    mut ch: WCHAR,
) -> LRESULT {
    let mut Ret: LRESULT = 0 as LRESULT;

    if !(*infoPtr).hParent.is_null() {
        let mut dwIdc: DWORD = GetWindowLongPtr((*infoPtr).hMapWnd, GWLP_ID as core::ffi::c_int) as DWORD;
        /*
         * Push directly into the event queue instead of waiting
         * the parent to be unlocked.
         * High word of LPARAM is still available for future needs...
         */
        Ret = PostMessage(
            (*infoPtr).hParent,
            WM_COMMAND as UINT,
            MAKELPARAM!((dwIdc as WORD), (code as WORD)) as WPARAM,
            LOWORD!(ch as DWORD) as LPARAM,
        ) as LRESULT;
    }

    return Ret;
}

unsafe extern "C" fn LimitCaretXY(infoPtr: PMAP, pX: *mut INT, pY: *mut INT) {
    let mut i: INT = 0;
    let mut X: INT = *pX;
    let mut Y: INT = *pY;
    let mut iYStart: INT = (*infoPtr).iYStart;

    i = (XCELLS * ((iYStart + Y)) as u32 + (X) as u32) as i32;
    while i >= (*infoPtr).NumValidGlyphs as INT {
        if X > 0 {
            X -= 1;
        } else {
            X = (XCELLS - 1) as i32;
            Y -= 1;
        }
        i = (XCELLS * ((iYStart + Y)) as u32 + (X) as u32) as i32;
    }

    *pX = X;
    *pY = Y;
}

unsafe extern "C" fn SetCaretXY(
    mut infoPtr: PMAP,
    mut X: INT,
    mut Y: INT,
    mut bLarge: BOOL,
    mut bInvalidateAll: BOOL,
) {
    if bInvalidateAll == 0 {
        InvalidateRect(
            (*infoPtr).hMapWnd,
            &raw mut (*(*infoPtr).pActiveCell).CellInt as LPCRECT,
            (FALSE) as BOOL,
        );
    }
    LimitCaretXY(infoPtr, &raw mut X, &raw mut Y);
    (*infoPtr).CaretX = X;
    (*infoPtr).CaretY = Y;
    UpdateCells(infoPtr);
    (*infoPtr).pActiveCell = (&raw mut *(&raw mut (*infoPtr).Cells as *mut [CELL; 20])
        .offset(Y as isize) as *mut CELL)
        .offset(X as isize) as *mut CELL as PCELL;
    if bInvalidateAll == 0 {
        InvalidateRect(
            (*infoPtr).hMapWnd,
            &raw mut (*(*infoPtr).pActiveCell).CellInt as LPCRECT,
            (FALSE) as BOOL,
        );
    }
    if bLarge != 0 {
        if !(*infoPtr).hLrgWnd.is_null() {
            MoveLargeCell(infoPtr);
        } else {
            CreateLargeCell(infoPtr);
        }
    } else if !(*infoPtr).hLrgWnd.is_null() {
        DestroyWindow((*infoPtr).hLrgWnd);
        (*infoPtr).hLrgWnd = ::core::ptr::null_mut::<HWND__>();
    }
    if bInvalidateAll != 0 {
        InvalidateRect((*infoPtr).hMapWnd, ::core::ptr::null::<RECT>(), (FALSE) as BOOL);
    }
    UpdateStatusBar((*(*infoPtr).pActiveCell).ch);
}

unsafe extern "C" fn OnClick(infoPtr: PMAP, ptx: WORD, pty: WORD) {
    let x: INT = min((XCELLS - 1) as i32, (ptx as INT) / max(1, (*infoPtr).CellSize.cx));
    let y: INT = min((YCELLS - 1) as i32, (pty as INT) / max(1, (*infoPtr).CellSize.cy));

    SetCaretXY(infoPtr, x, y, (TRUE) as BOOL, (FALSE) as BOOL);
}

unsafe extern "C" fn MapOnCreate(mut infoPtr: PMAP, mut hwnd: HWND, mut hParent: HWND) -> BOOL {
    let mut rc: RECT = core::mem::zeroed();

    infoPtr = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, core::mem::size_of::<MAP>() as SIZE_T) as PMAP;
    if infoPtr.is_null() {
        return (FALSE) as BOOL;
    }

    SetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int, infoPtr as LONG_PTR);

    (*infoPtr).hMapWnd = hwnd;
    (*infoPtr).hParent = hParent;

    GetClientRect(hwnd, &raw mut rc);
    (*infoPtr).ClientSize.cx = rc.right;
    (*infoPtr).ClientSize.cy = rc.bottom;
    (*infoPtr).CellSize.cx = (*infoPtr).ClientSize.cx / XCELLS as LONG;
    (*infoPtr).CellSize.cy = (*infoPtr).ClientSize.cy / YCELLS as LONG;

    (*infoPtr).pActiveCell = &raw mut (*infoPtr).Cells[0 as usize][0 as usize] as PCELL;

    SetGrid(infoPtr);

    SetScrollPos((*infoPtr).hParent, (SB_VERT) as i32, 0, (TRUE) as BOOL);
    return (TRUE) as BOOL;
}

// The C parameter `Pos` would shadow ntdll-sys's `Pos` (_BUS_DATA_TYPE_Pos) in Rust
// (E0530): renamed.
unsafe extern "C" fn OnVScroll(infoPtr: PMAP, Value: INT, mut Pos_: INT) {
    let mut iYDiff: INT = 0;
    let mut iOldYStart: INT = (*infoPtr).iYStart;
    let mut X: INT = 0;
    let mut Y: INT = 0;

    match (Value) as u32 {
        SB_LINEUP => {
            (*infoPtr).iYStart -= 1;
        }
        SB_LINEDOWN => {
            (*infoPtr).iYStart += 1;
        }
        SB_PAGEUP => {
            (*infoPtr).iYStart -= (YCELLS as i32);
        }
        SB_PAGEDOWN => {
            (*infoPtr).iYStart += (YCELLS as i32);
        }
        SB_THUMBTRACK => {
            (*infoPtr).iYStart = Pos_;
        }
        SB_TOP => {
            (*infoPtr).iYStart = 0;
            SetCaretXY(infoPtr, 0, 0, (FALSE) as BOOL, (TRUE) as BOOL);
            return;
        }
        SB_BOTTOM => {
            (*infoPtr).iYStart = (*infoPtr).NumRows;
            SetCaretXY(infoPtr, (XCELLS - 1) as INT, (YCELLS - 1) as INT, (FALSE) as BOOL, (TRUE) as BOOL);
        }
        _ => {}
    }

    (*infoPtr).iYStart = max(0, (*infoPtr).iYStart);
    (*infoPtr).iYStart = min((*infoPtr).iYStart, (*infoPtr).NumRows);

    UpdateCells(infoPtr);

    X = (*infoPtr).CaretX;
    Y = (*infoPtr).CaretY;
    LimitCaretXY(infoPtr, core::ptr::addr_of_mut!(X), core::ptr::addr_of_mut!(Y));
    SetCaretXY(infoPtr, X, Y, IsWindow((*infoPtr).hLrgWnd), (FALSE) as BOOL);

    iYDiff = iOldYStart - (*infoPtr).iYStart;
    if iYDiff != 0 {
        if !(*infoPtr).hLrgWnd.is_null() {
            ShowWindow((*infoPtr).hLrgWnd, (SW_HIDE) as i32);
        }

        SetScrollPos(
            (*infoPtr).hMapWnd,
            (SB_VERT) as i32,
            (*infoPtr).iYStart,
            (TRUE) as BOOL,
        );

        if (abs(iYDiff) < (YCELLS as i32)) {
            let mut rect: RECT = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };

            /* Invalidate the rect around the active cell since a new cell will become active */
            if !(*infoPtr).pActiveCell.is_null() {
                InvalidateRect(
                    (*infoPtr).hMapWnd,
                    core::ptr::addr_of_mut!((*(*infoPtr).pActiveCell).CellExt),
                    (TRUE) as BOOL,
                );
            }

            GetClientRect((*infoPtr).hMapWnd, core::ptr::addr_of_mut!(rect));
            rect.top += 2;
            rect.bottom -= 2;
            ScrollWindowEx(
                (*infoPtr).hMapWnd,
                0,
                iYDiff * (*infoPtr).CellSize.cy,
                core::ptr::addr_of_mut!(rect),
                core::ptr::addr_of_mut!(rect),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                SW_INVALIDATE,
            );
        } else {
            InvalidateRect(
                (*infoPtr).hMapWnd,
                core::ptr::null(),
                (TRUE) as BOOL,
            );
        }

        if !(*infoPtr).hLrgWnd.is_null() {
            ShowWindow((*infoPtr).hLrgWnd, (SW_SHOW) as i32);
        }
    }

    UpdateStatusBar((*(*infoPtr).pActiveCell).ch);
}

unsafe extern "C" fn OnPaint(infoPtr: PMAP, wParam: WPARAM) {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();

    if wParam != 0 {
        if GetUpdateRect((*infoPtr).hMapWnd, core::ptr::addr_of_mut!(ps.rcPaint), (TRUE) as BOOL) == 0 {
            return;
        }

        ps.hdc = wParam as HDC;
    } else {
        hdc = BeginPaint((*infoPtr).hMapWnd, core::ptr::addr_of_mut!(ps));
        if hdc.is_null() {
            return;
        }
    }

    FillGrid(infoPtr, core::ptr::addr_of_mut!(ps));

    if wParam == 0 {
        EndPaint((*infoPtr).hMapWnd, core::ptr::addr_of_mut!(ps));
    }
}

unsafe extern "C" fn MoveUpDown(infoPtr: PMAP, DY: INT) {
    let mut Y: INT = (*infoPtr).CaretY;

    if DY < 0 /* Move Up */ {
        if Y <= 0 {
            SendMessageW((*infoPtr).hMapWnd, WM_VSCROLL, MAKEWPARAM!(SB_LINEUP, 0), 0);
            return;
        }

        Y -= 1;
    } else if DY > 0 /* Move Down */ {
        if Y + 1 >= (YCELLS as i32) {
            SendMessageW((*infoPtr).hMapWnd, WM_VSCROLL, MAKEWPARAM!(SB_LINEDOWN, 0), 0);
            return;
        }

        Y += 1;
    }

    SetCaretXY(infoPtr, (*infoPtr).CaretX, Y, IsWindow((*infoPtr).hLrgWnd), (FALSE) as BOOL);
}

unsafe extern "C" fn MoveLeftRight(mut infoPtr: PMAP, mut DX: INT) {
    let mut X: INT = (*infoPtr).CaretX;
    let mut Y: INT = (*infoPtr).CaretY;

    if DX < 0 {
        if X <= 0 {
            if Y <= 0 {
                Y = 0;
                if (*infoPtr).iYStart > 0 {
                    X = (XCELLS - 1) as i32;
                }
                SendMessageW(
                    (*infoPtr).hMapWnd,
                    WM_VSCROLL,
                    MAKEWPARAM!(SB_LINEUP, 0),
                    0,
                );
            } else {
                X = (XCELLS - 1) as i32;
                Y -= 1;
            }
        } else {
            X -= 1;
        }
    } else if DX > 0 {
        if X + 1 >= (XCELLS as i32) {
            if Y + 1 >= (YCELLS as i32) {
                Y = (YCELLS - 1) as i32;
                if (*infoPtr).iYStart < (*infoPtr).NumRows {
                    X = 0;
                }
                SendMessageW(
                    (*infoPtr).hMapWnd,
                    WM_VSCROLL,
                    MAKEWPARAM!(SB_LINEDOWN, 0),
                    0,
                );
            } else {
                X = 0;
                Y += 1;
            }
        } else {
            X += 1;
        }
    }

    SetCaretXY(
        infoPtr,
        X,
        Y,
        IsWindow((*infoPtr).hLrgWnd),
        (FALSE) as BOOL,
    );
}

unsafe extern "C" fn OnKeyDown(mut infoPtr: PMAP, mut wParam: WPARAM, mut lParam: LPARAM) {
    let mut bCtrlDown: BOOL = (GetKeyState(VK_CONTROL as ::core::ffi::c_int) < 0) as BOOL;

    match wParam as UINT {
        VK_UP => {
            if bCtrlDown != 0 {
                SetCaretXY(infoPtr, (*infoPtr).CaretX, 0, (FALSE) as BOOL, (FALSE) as BOOL);
            } else {
                MoveUpDown(infoPtr, -(1 as INT));
            }
        }
        VK_DOWN => {
            if bCtrlDown != 0 {
                SetCaretXY(infoPtr, (*infoPtr).CaretX, (YCELLS - 1) as INT, (FALSE) as BOOL, (FALSE) as BOOL);
            } else {
                MoveUpDown(infoPtr, 1 as INT);
            }
        }
        VK_LEFT => {
            if bCtrlDown != 0 {
                SetCaretXY(infoPtr, 0, (*infoPtr).CaretY, (FALSE) as BOOL, (FALSE) as BOOL);
            } else {
                MoveLeftRight(infoPtr, -(1 as INT));
            }
        }
        VK_RIGHT => {
            if bCtrlDown != 0 {
                SetCaretXY(infoPtr, (XCELLS - 1) as INT, (*infoPtr).CaretY, (FALSE) as BOOL, (FALSE) as BOOL);
            } else {
                MoveLeftRight(infoPtr, 1 as INT);
            }
        }
        VK_PRIOR => {
            /* Page Up */
            SendMessageW((*infoPtr).hMapWnd, WM_VSCROLL as UINT, MAKEWPARAM!(SB_PAGEUP, 0) as WPARAM, 0 as LPARAM);
        }
        VK_NEXT => {
            /* Page Down */
            SendMessageW((*infoPtr).hMapWnd, WM_VSCROLL as UINT, MAKEWPARAM!(SB_PAGEDOWN, 0) as WPARAM, 0 as LPARAM);
        }
        VK_HOME => {
            if bCtrlDown != 0 {
                SendMessageW((*infoPtr).hMapWnd, WM_VSCROLL as UINT, MAKEWPARAM!(SB_TOP, 0) as WPARAM, 0 as LPARAM);
            } else {
                SetCaretXY(infoPtr, 0, (*infoPtr).CaretY, (FALSE) as BOOL, (FALSE) as BOOL);
            }
        }
        VK_END => {
            if bCtrlDown != 0 {
                SendMessageW((*infoPtr).hMapWnd, WM_VSCROLL as UINT, MAKEWPARAM!(SB_BOTTOM, 0) as WPARAM, 0 as LPARAM);
            } else {
                SetCaretXY(infoPtr, (XCELLS - 1) as INT, (*infoPtr).CaretY, (FALSE) as BOOL, (FALSE) as BOOL);
            }
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn MapWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let infoPtr: PMAP = GetWindowLongPtrW(hwnd, 0) as PMAP;
    let mut Ret: LRESULT = 0 as LRESULT;
    let mut lfFaceName: [WCHAR; (LF_FACESIZE as usize)] = [0; (LF_FACESIZE as usize)];

    match uMsg {
        WM_CREATE => {
            if MapOnCreate(
                infoPtr,
                hwnd,
                (*(lParam as LPCREATESTRUCTW)).hwndParent,
            ) == 0
            {
                return -(1 as LRESULT);
            }
        }

        WM_KEYDOWN => {
            OnKeyDown(infoPtr, wParam, lParam);
        }

        WM_LBUTTONDOWN => {
            SetFocus(hwnd);
            OnClick(infoPtr, LOWORD!(lParam), HIWORD!(lParam));
        }

        WM_MOUSEMOVE => {
            if (wParam & (MK_LBUTTON as WPARAM)) != 0 {
                OnClick(infoPtr, LOWORD!(lParam), HIWORD!(lParam));
            }
        }

        WM_LBUTTONDBLCLK => {
            if !((*infoPtr).pActiveCell.is_null() || GetFocus() != hwnd) {
                NotifyParentOfSelection(
                    infoPtr,
                    FM_SETCHAR as UINT,
                    (*(*infoPtr).pActiveCell).ch,
                );

                if !(*infoPtr).hLrgWnd.is_null() {
                    DestroyWindow((*infoPtr).hLrgWnd);
                    (*infoPtr).hLrgWnd = core::ptr::null_mut::<HWND__>();
                }
            }
        }

        WM_VSCROLL => {
            OnVScroll(infoPtr, (LOWORD!(wParam)) as INT, (HIWORD!(wParam)) as INT);
        }

        FM_SETCHARMAP => {
            (*infoPtr).iYStart = 0 as INT;
            (*infoPtr).CaretY = (*infoPtr).iYStart;
            (*infoPtr).CaretX = (*infoPtr).CaretY;
            (*infoPtr).CharMap = LOWORD!(wParam) as INT;
            wcsncpy(
                core::ptr::addr_of_mut!(lfFaceName) as *mut WCHAR,
                core::ptr::addr_of_mut!((*infoPtr).CurrentFont.lfFaceName) as *mut WCHAR,
                SIZEOF!(lfFaceName), // already a WCHAR count; the port halved it again
            );
            SetFont(infoPtr, core::ptr::addr_of_mut!(lfFaceName) as LPWSTR);
            SetCaretXY(
                infoPtr,
                (*infoPtr).CaretX,
                (*infoPtr).CaretY,
                (FALSE) as BOOL,
                (TRUE) as BOOL,
            );
        }

        FM_SETFONT => {
            (*infoPtr).iYStart = 0 as INT;
            (*infoPtr).CaretY = (*infoPtr).iYStart;
            (*infoPtr).CaretX = (*infoPtr).CaretY;
            SetFont(infoPtr, lParam as LPWSTR);
            SetCaretXY(
                infoPtr,
                (*infoPtr).CaretX,
                (*infoPtr).CaretY,
                (FALSE) as BOOL,
                (TRUE) as BOOL,
            );
        }

        FM_GETCHAR => {
            if (*infoPtr).pActiveCell.is_null() {
                return 0 as LRESULT;
            }
            return (*(*infoPtr).pActiveCell).ch as LRESULT;
        }

        FM_GETHFONT => {
            return (*infoPtr).hFont as LRESULT;
        }

        WM_PAINT => {
            OnPaint(infoPtr, wParam);
        }

        WM_DESTROY => {
            DeleteObject((*infoPtr).hFont as HGDIOBJ);
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                infoPtr as PVOID,
            );
            SetWindowLongPtrW(hwnd, 0, 0 as LONG_PTR);
        }

        WM_GETDLGCODE => {
            return DLGC_WANTARROWS as LRESULT;
        }

        WM_SETFOCUS | WM_KILLFOCUS => {
            if (*infoPtr).hLrgWnd.is_null() {
                InvalidateRect(
                    hwnd,
                    core::ptr::addr_of_mut!((*(*infoPtr).pActiveCell).CellInt) as LPCRECT,
                    (FALSE) as BOOL,
                );
            }
        }

        _ => {
            Ret = DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterMapClasses(hinst: HINSTANCE) -> BOOL {
    let mut wc: WNDCLASSW = core::mem::zeroed();

    wc.style = CS_DBLCLKS as UINT;
    wc.lpfnWndProc =
        Some(MapWndProc);
    wc.cbWndExtra = core::mem::size_of::<PMAP>() as core::ffi::c_int;
    wc.hInstance = hinst;
    wc.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as usize as LPWSTR,
    );
    wc.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    wc.lpszClassName = core::ptr::addr_of!(szMapWndClass) as *const WCHAR as LPCWSTR;

    if RegisterClassW(core::ptr::addr_of_mut!(wc)) != 0 {
        wc.lpfnWndProc =
            Some(LrgCellWndProc);
        wc.cbWndExtra = 0 as core::ffi::c_int;
        wc.lpszClassName = core::ptr::addr_of!(szLrgCellWndClass) as *const WCHAR as LPCWSTR;

        return (RegisterClassW(core::ptr::addr_of_mut!(wc)) != 0)
            as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterMapClasses(hInst: HINSTANCE) {
    UnregisterClassW(core::ptr::addr_of!(szMapWndClass) as *const u16, hInst);
    UnregisterClassW(core::ptr::addr_of!(szLrgCellWndClass) as *const u16, hInst);
}

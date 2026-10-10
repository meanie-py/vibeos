use crate::taskmgr::perfpage::PerformancePageCpuUsageHistoryGraph;
use crate::taskmgr::perfpage::PerformancePageMemUsageHistoryGraph;
use crate::taskmgr::perfpage::hPerformancePageCpuUsageHistoryGraph;
use crate::taskmgr::perfpage::hPerformancePageMemUsageHistoryGraph;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut OldGraphCtrlWndProc: WNDPROC = None;
#[no_mangle]
pub unsafe extern "C" fn GraphCtrl_Create(inst: PTM_GRAPH_CONTROL, hWnd: HWND, hParentWnd: HWND, fmt: PTM_FORMAT) -> BOOL {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hdcg: HDC = core::ptr::null_mut();
    let mut hbmOld: HBITMAP = core::ptr::null_mut();
    let mut Size: UINT = 0;
    let mut p: INT = 0;
    let mut rc: RECT = core::mem::zeroed();

    (*inst).hParentWnd = hParentWnd;
    (*inst).hWnd = hWnd;

    Size = GetSystemMetrics((SM_CXSCREEN) as i32) as UINT;
    (*inst).BitmapWidth = Size as INT;
    Size /= PLOT_SHIFT as UINT;
    (*inst).PointBuffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        Size.wrapping_mul(NUM_PLOTS as UINT) as SIZE_T,
    ) as PBYTE;
    'fail: {
        if (*inst).PointBuffer.is_null() {
            break 'fail;
        }

        (*inst).NumberOfPoints = Size as UINT32;
        (*inst).CurrIndex = 0 as UINT32;

        /* Styling */
        (*inst).hPenGrid = CreatePen(
            (PS_SOLID as i32),
            0 as ::core::ffi::c_int,
            (*fmt).clrGrid,
        );
        (*inst).hPen0 = CreatePen(
            (PS_SOLID as i32),
            0 as ::core::ffi::c_int,
            (*fmt).clrPlot0,
        );
        (*inst).hPen1 = CreatePen(
            (PS_SOLID as i32),
            0 as ::core::ffi::c_int,
            (*fmt).clrPlot1,
        );
        (*inst).hBrushBack = CreateSolidBrush((*fmt).clrBack);

        if (*inst).hPenGrid.is_null()
            || (*inst).hPen0.is_null()
            || (*inst).hPen1.is_null()
            || (*inst).hBrushBack.is_null()
        {
            break 'fail;
        }

        if ((*fmt).GridCellWidth) as u32 >= (PLOT_SHIFT << 2) {
            (*inst).GridCellWidth = (*fmt).GridCellWidth;
        } else {
            (*inst).GridCellWidth = (PLOT_SHIFT << 2) as INT;
        }
        if ((*fmt).GridCellHeight) as u32 >= (PLOT_SHIFT << 2) {
            (*inst).GridCellHeight = (*fmt).GridCellHeight;
        } else {
            (*inst).GridCellHeight = (PLOT_SHIFT << 2) as INT;
        }

        (*inst).DrawSecondaryPlot = (*fmt).DrawSecondaryPlot;

        GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
        (*inst).BitmapHeight = rc.bottom as INT;
        (*inst).ftPixelsPerPercent =
            ((*inst).BitmapHeight as FLOAT) / (100 as FLOAT);

        hdc = GetDC(hParentWnd);
        if hdc.is_null() {
            break 'fail;
        }

        hdcg = CreateCompatibleDC(hdc);
        (*inst).hdcGraph = hdcg;
        (*inst).hbmGraph =
            CreateCompatibleBitmap(hdc, (*inst).BitmapWidth, (*inst).BitmapHeight);
        ReleaseDC(hParentWnd, hdc);
        if hdcg.is_null() || (*inst).hbmGraph.is_null() {
            break 'fail;
        }

        hbmOld = SelectObject(hdcg, (*inst).hbmGraph as HGDIOBJ) as HBITMAP;
        DeleteObject(hbmOld as HGDIOBJ);

        SetBkColor(hdcg, (*fmt).clrBack);
        rc.right = (*inst).BitmapWidth as LONG;
        FillRect(
            hdcg,
            core::ptr::addr_of_mut!(rc) as LPCRECT,
            (*inst).hBrushBack,
        );

        (*inst).CurrShift = 0 as INT;
        SelectObject(hdcg, (*inst).hPenGrid as HGDIOBJ);
        p = (*inst).GridCellHeight - 1;
        while p < (*inst).BitmapHeight {
            MoveToEx(
                hdcg,
                0 as ::core::ffi::c_int,
                p as ::core::ffi::c_int,
                core::ptr::null_mut(),
            );
            LineTo(
                hdcg,
                (*inst).BitmapWidth as ::core::ffi::c_int,
                p as ::core::ffi::c_int,
            );
            p += (*inst).GridCellHeight;
        }
        p = (*inst).BitmapWidth - 1;
        while p > 0 {
            MoveToEx(
                hdcg,
                p as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                core::ptr::null_mut(),
            );
            LineTo(
                hdcg,
                p as ::core::ffi::c_int,
                (*inst).BitmapHeight as ::core::ffi::c_int,
            );
            p -= (*inst).GridCellWidth;
        }
        SelectObject(hdcg, (*inst).hPen0 as HGDIOBJ);

        return (TRUE) as BOOL;
    }

    // fail:
    GraphCtrl_Dispose(inst);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GraphCtrl_Dispose(inst: PTM_GRAPH_CONTROL) {
    if !(*inst).PointBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, (*inst).PointBuffer as PVOID);
    }

    if !(*inst).hdcGraph.is_null() {
        DeleteDC((*inst).hdcGraph);
    }

    if !(*inst).hPenGrid.is_null() {
        DeleteObject((*inst).hPenGrid as HGDIOBJ);
    }

    if !(*inst).hPen0.is_null() {
        DeleteObject((*inst).hPen0 as HGDIOBJ);
    }

    if !(*inst).hPen1.is_null() {
        DeleteObject((*inst).hPen1 as HGDIOBJ);
    }

    if !(*inst).hBrushBack.is_null() {
        DeleteObject((*inst).hBrushBack as HGDIOBJ);
    }

    if !(*inst).hbmGraph.is_null() {
        DeleteObject((*inst).hbmGraph as HGDIOBJ);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GraphCtrl_AddPoint(inst: PTM_GRAPH_CONTROL, val0: BYTE, val1: BYTE) {
    let mut hdcg: HDC = core::ptr::null_mut();
    let mut t: PBYTE = core::ptr::null_mut();
    let mut rcDirt: RECT = core::mem::zeroed();

    let mut Prev0: UINT = 0;
    let mut Prev1: UINT = 0;
    let mut RetainingWidth: UINT = 0;
    let mut PrevY: INT = 0;
    let mut CurrY: INT = 0;
    let mut p: INT = 0;
    let mut v: INT = 0;

    hdcg = (*inst).hdcGraph;
    RetainingWidth = ((*inst).BitmapWidth as UINT).wrapping_sub(PLOT_SHIFT as UINT);
    t = (*inst).PointBuffer;
    Prev0 = *t.offset((*inst).CurrIndex as isize) as UINT;
    Prev1 = *t.offset((*inst).CurrIndex as isize + (*inst).NumberOfPoints as isize) as UINT;
    if (*inst).CurrIndex < (*inst).NumberOfPoints.wrapping_sub(1 as UINT32) {
        (*inst).CurrIndex = (*inst).CurrIndex.wrapping_add(1);
    } else {
        (*inst).CurrIndex = 0 as UINT32;
    }
    *t.offset((*inst).CurrIndex as isize) = val0;
    *t.offset((*inst).CurrIndex as isize + (*inst).NumberOfPoints as isize) = val1;

    /* Drawing points, first shifting the plot left */
    BitBlt(
        hdcg,
        0 as INT,
        0 as INT,
        RetainingWidth as INT,
        (*inst).BitmapHeight as INT,
        hdcg,
        PLOT_SHIFT as INT,
        0 as INT,
        SRCCOPY as DWORD,
    );

    rcDirt.left = RetainingWidth as LONG;
    rcDirt.top = 0 as LONG;
    rcDirt.right = (*inst).BitmapWidth as LONG;
    rcDirt.bottom = (*inst).BitmapHeight as LONG;
    FillRect(
        hdcg,
        core::ptr::addr_of_mut!(rcDirt) as LPCRECT,
        (*inst).hBrushBack,
    );

    SelectObject(hdcg, (*inst).hPenGrid as HGDIOBJ);
    p = (*inst).GridCellHeight as INT - 1 as INT;
    while p < (*inst).BitmapHeight as INT {
        MoveToEx(hdcg, RetainingWidth as INT, p, core::ptr::null_mut());
        LineTo(hdcg, (*inst).BitmapWidth as INT, p);
        p += (*inst).GridCellHeight as INT;
    }
    v = (*inst).CurrShift as INT + PLOT_SHIFT as INT;
    if v >= (*inst).GridCellWidth as INT {
        v -= (*inst).GridCellWidth as INT;
        p = (*inst).BitmapWidth as INT - v - 1 as INT;
        MoveToEx(hdcg, p, 0 as INT, core::ptr::null_mut());
        LineTo(hdcg, p, (*inst).BitmapHeight as INT);
    }
    (*inst).CurrShift = v;

    if (*inst).DrawSecondaryPlot != 0 {
        SelectObject((*inst).hdcGraph, (*inst).hPen1 as HGDIOBJ);

        PrevY = ((*inst).BitmapHeight as FLOAT - Prev1 as FLOAT * (*inst).ftPixelsPerPercent) as INT;
        MoveToEx(
            (*inst).hdcGraph,
            RetainingWidth as INT - 1 as INT,
            PrevY,
            core::ptr::null_mut(),
        );
        CurrY = ((*inst).BitmapHeight as FLOAT - val1 as FLOAT * (*inst).ftPixelsPerPercent) as INT;
        LineTo(
            (*inst).hdcGraph,
            (*inst).BitmapWidth as INT - 1 as INT,
            CurrY,
        );
    }

    SelectObject((*inst).hdcGraph, (*inst).hPen0 as HGDIOBJ);
    PrevY = ((*inst).BitmapHeight as FLOAT - Prev0 as FLOAT * (*inst).ftPixelsPerPercent) as INT;
    MoveToEx(
        (*inst).hdcGraph,
        RetainingWidth as INT - 1 as INT,
        PrevY,
        core::ptr::null_mut(),
    );
    CurrY = ((*inst).BitmapHeight as FLOAT - val0 as FLOAT * (*inst).ftPixelsPerPercent) as INT;
    LineTo(
        (*inst).hdcGraph,
        (*inst).BitmapWidth as INT - 1 as INT,
        CurrY,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GraphCtrl_RedrawBitmap(mut inst: PTM_GRAPH_CONTROL, mut h: INT) {
    let mut hdcg: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut t: PBYTE = ::core::ptr::null_mut::<BYTE>();
    let mut rc: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut i: INT = 0;
    let mut j: INT = 0;
    let mut y: INT = 0;
    let mut x: INT = 0;
    let mut p: INT = 0;
    let mut coef: FLOAT = 0.;
    hdcg = (*inst).hdcGraph;
    rc.left = 0 as LONG;
    rc.top = 0 as LONG;
    rc.right = (*inst).BitmapWidth as LONG;
    rc.bottom = h as LONG;
    FillRect(hdcg, &raw mut rc as LPCRECT, (*inst).hBrushBack);
    SelectObject(hdcg, (*inst).hPenGrid as HGDIOBJ);
    p = ((*inst).GridCellHeight as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;
    while p < (*inst).BitmapHeight {
        MoveToEx(
            hdcg,
            0 as ::core::ffi::c_int,
            p as ::core::ffi::c_int,
            ::core::ptr::null_mut::<tagPOINT>(),
        );
        LineTo(
            hdcg,
            (*inst).BitmapWidth as ::core::ffi::c_int,
            p as ::core::ffi::c_int,
        );
        p += (*inst).GridCellHeight;
    }
    p = ((*inst).BitmapWidth as ::core::ffi::c_int
        - (*inst).CurrShift as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as INT;
    while p > 0 as INT {
        MoveToEx(
            hdcg,
            p as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<tagPOINT>(),
        );
        LineTo(
            hdcg,
            p as ::core::ffi::c_int,
            (*inst).BitmapHeight as ::core::ffi::c_int,
        );
        p -= (*inst).GridCellWidth;
    }
    coef = (*inst).ftPixelsPerPercent;
    if (*inst).DrawSecondaryPlot != 0 {
        SelectObject(hdcg, (*inst).hPen1 as HGDIOBJ);
        t = (*inst).PointBuffer.offset((*inst).NumberOfPoints as isize);
        x = ((*inst).BitmapWidth as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;
        j = (*inst).CurrIndex as INT;
        y = (h as FLOAT - *t.offset(j as isize) as ::core::ffi::c_int as FLOAT * coef) as INT;
        MoveToEx(
            hdcg,
            x as ::core::ffi::c_int,
            y as ::core::ffi::c_int,
            ::core::ptr::null_mut::<tagPOINT>(),
        );
        i = 0 as INT;
        while (i as UINT32) < (*inst).NumberOfPoints {
            j = (if j != 0 {
                j as UINT32
            } else {
                (*inst).NumberOfPoints
            })
            .wrapping_sub(1 as UINT32) as INT;
            y = (h as FLOAT - *t.offset(j as isize) as ::core::ffi::c_int as FLOAT * coef) as INT;
            x -= PLOT_SHIFT as INT;
            LineTo(hdcg, x as ::core::ffi::c_int, y as ::core::ffi::c_int);
            i += 1;
        }
    }
    SelectObject(hdcg, (*inst).hPen0 as HGDIOBJ);
    t = (*inst).PointBuffer;
    x = ((*inst).BitmapWidth as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;
    j = (*inst).CurrIndex as INT;
    y = (h as FLOAT - *t.offset(j as isize) as ::core::ffi::c_int as FLOAT * coef) as INT;
    MoveToEx(
        hdcg,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        ::core::ptr::null_mut::<tagPOINT>(),
    );
    i = 0 as INT;
    while (i as UINT32) < (*inst).NumberOfPoints {
        j = (if j != 0 {
            j as UINT32
        } else {
            (*inst).NumberOfPoints
        })
        .wrapping_sub(1 as UINT32) as INT;
        y = (h as FLOAT - *t.offset(j as isize) as ::core::ffi::c_int as FLOAT * coef) as INT;
        x -= PLOT_SHIFT as INT;
        LineTo(hdcg, x as ::core::ffi::c_int, y as ::core::ffi::c_int);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GraphCtrl_RedrawOnHeightChange(inst: PTM_GRAPH_CONTROL, nh: INT) {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hbmOld: HBITMAP = core::ptr::null_mut();

    (*inst).BitmapHeight = nh;
    (*inst).ftPixelsPerPercent = (nh as FLOAT) / (100 as FLOAT);

    hdc = GetDC((*inst).hParentWnd);
    hbmOld = (*inst).hbmGraph;
    (*inst).hbmGraph = CreateCompatibleBitmap(hdc, (*inst).BitmapWidth, nh);
    SelectObject((*inst).hdcGraph, (*inst).hbmGraph as HGDIOBJ);
    DeleteObject(hbmOld as HGDIOBJ);
    ReleaseDC((*inst).hParentWnd, hdc);

    GraphCtrl_RedrawBitmap(inst, nh);
}
#[no_mangle]
pub unsafe extern "system" fn GraphCtrl_WndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut graph: PTM_GRAPH_CONTROL = core::ptr::null_mut();

    match message {
        WM_ERASEBKGND => {
            return (TRUE) as INT_PTR;
        }
        /*
         *  Filter out mouse  & keyboard messages
         */
        // case WM_APPCOMMAND:
        WM_CAPTURECHANGED
        | WM_LBUTTONDBLCLK
        | WM_LBUTTONDOWN
        | WM_LBUTTONUP
        | WM_MBUTTONDBLCLK
        | WM_MBUTTONDOWN
        | WM_MBUTTONUP
        | WM_MOUSEACTIVATE
        | WM_MOUSEHOVER
        | WM_MOUSELEAVE
        | WM_MOUSEMOVE
        // case WM_MOUSEWHEEL:
        | WM_NCHITTEST
        | WM_NCLBUTTONDBLCLK
        | WM_NCLBUTTONDOWN
        | WM_NCLBUTTONUP
        | WM_NCMBUTTONDBLCLK
        | WM_NCMBUTTONDOWN
        | WM_NCMBUTTONUP
        // case WM_NCMOUSEHOVER:
        // case WM_NCMOUSELEAVE:
        | WM_NCMOUSEMOVE
        | WM_NCRBUTTONDBLCLK
        | WM_NCRBUTTONDOWN
        | WM_NCRBUTTONUP
        // case WM_NCXBUTTONDBLCLK:
        // case WM_NCXBUTTONDOWN:
        // case WM_NCXBUTTONUP:
        | WM_RBUTTONDBLCLK
        | WM_RBUTTONDOWN
        | WM_RBUTTONUP
        // case WM_XBUTTONDBLCLK:
        // case WM_XBUTTONDOWN:
        // case WM_XBUTTONUP:
        | WM_ACTIVATE
        | WM_CHAR
        | WM_DEADCHAR
        | WM_GETHOTKEY
        | WM_HOTKEY
        | WM_KEYDOWN
        | WM_KEYUP
        | WM_KILLFOCUS
        | WM_SETFOCUS
        | WM_SETHOTKEY
        | WM_SYSCHAR
        | WM_SYSDEADCHAR
        | WM_SYSKEYDOWN
        | WM_SYSKEYUP => {
            return 0;
        }

        WM_NCCALCSIZE => {
            return 0;
        }

        WM_SIZE => {
            if hWnd == hPerformancePageCpuUsageHistoryGraph {
                graph = core::ptr::addr_of_mut!(PerformancePageCpuUsageHistoryGraph);
            } else if hWnd == hPerformancePageMemUsageHistoryGraph {
                graph = core::ptr::addr_of_mut!(PerformancePageMemUsageHistoryGraph);
            } else {
                return 0;
            }

            if HIWORD!(lParam) as INT != (*graph).BitmapHeight {
                GraphCtrl_RedrawOnHeightChange(graph, HIWORD!(lParam) as INT);
            }
            InvalidateRect(hWnd, core::ptr::null_mut(), (FALSE) as BOOL);

            return 0;
        }

        WM_PAINT => {
            let mut rcClient: RECT = core::mem::zeroed();
            let mut hdc: HDC = core::ptr::null_mut();
            let mut ps: PAINTSTRUCT = core::mem::zeroed();

            if hWnd == hPerformancePageCpuUsageHistoryGraph {
                graph = core::ptr::addr_of_mut!(PerformancePageCpuUsageHistoryGraph);
            } else if hWnd == hPerformancePageMemUsageHistoryGraph {
                graph = core::ptr::addr_of_mut!(PerformancePageMemUsageHistoryGraph);
            } else {
                return 0;
            }

            hdc = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
            GetClientRect(hWnd, core::ptr::addr_of_mut!(rcClient));
            BitBlt(
                hdc,
                0,
                0,
                rcClient.right,
                rcClient.bottom,
                (*graph).hdcGraph,
                (*graph).BitmapWidth - rcClient.right,
                0,
                SRCCOPY,
            );
            EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
            return 0;
        }
        _ => {}
    }

    /*
     *  We pass on all non-handled messages
     */
    return CallWindowProcW(OldGraphCtrlWndProc, hWnd, message, wParam, lParam);
}

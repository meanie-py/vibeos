use crate::taskmgr::taskmgr::FillSolidRect;
use crate::taskmgr::perfdata::PerfDataGetCommitChargeLimitK;
use crate::taskmgr::perfdata::PerfDataGetCommitChargeTotalK;
use crate::taskmgr::perfdata::PerfDataGetProcessorSystemUsage;
use crate::taskmgr::perfdata::PerfDataGetProcessorUsage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_23CC5F20442A: [u16; 6] = [0x0025u16, 0x0064, 0x0020, 0x004D, 0x0042, 0x0000];
pub static RUSTCONV_WSTR_BD0EF5CBF459: [u16; 5] = [0x0025u16, 0x0064, 0x0020, 0x004B, 0x0000];
pub static RUSTCONV_WSTR_C5D4AD672797: [u16; 5] = [0x0025u16, 0x0064, 0x0025, 0x0025, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut nlastBarsUsed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[no_mangle]
pub static mut OldGraphWndProc: WNDPROC = None;
#[no_mangle]
pub unsafe extern "system" fn Graph_WndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut WindowId: LONG = 0;

    match message {
        WM_ERASEBKGND => {
            return TRUE as INT_PTR;
        }
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
        | WM_NCHITTEST
        | WM_NCLBUTTONDBLCLK
        | WM_NCLBUTTONDOWN
        | WM_NCLBUTTONUP
        | WM_NCMBUTTONDBLCLK
        | WM_NCMBUTTONDOWN
        | WM_NCMBUTTONUP
        | WM_NCMOUSEMOVE
        | WM_NCRBUTTONDBLCLK
        | WM_NCRBUTTONDOWN
        | WM_NCRBUTTONUP
        | WM_RBUTTONDBLCLK
        | WM_RBUTTONDOWN
        | WM_RBUTTONUP
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
        | WM_SYSKEYUP
        | WM_NCCALCSIZE => {
            return 0 as INT_PTR;
        }
        WM_PAINT => {
            hdc = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));

            WindowId = GetWindowLongPtrW(hWnd, GWLP_ID) as LONG;

            match (WindowId) as u32 {
                IDC_CPU_USAGE_GRAPH => {
                    Graph_DrawCpuUsageGraph(hdc, hWnd);
                }
                IDC_MEM_USAGE_GRAPH => {
                    Graph_DrawMemUsageGraph(hdc, hWnd);
                }
                IDC_MEM_USAGE_HISTORY_GRAPH => {
                    Graph_DrawMemUsageHistoryGraph(hdc, hWnd);
                }
                _ => {}
            }

            EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
            return 0 as INT_PTR;
        }
        _ => {}
    }

    return CallWindowProcW(OldGraphWndProc, hWnd, message, wParam, lParam) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn Graph_DrawCpuUsageGraph(hDC: HDC, hWnd: HWND) {
    let mut rcClient: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcBarLeft: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcBarRight: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcText: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut crPrevForeground: COLORREF = 0;
    let mut Text: [WCHAR; 260] = [0; 260];
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut CpuUsage: ULONG = 0;
    let mut CpuKernelUsage: ULONG = 0;
    let mut nBars: core::ffi::c_int = 0;
    let mut nBarsUsed: core::ffi::c_int = 0;
    /* Bottom bars that are "used", i.e. are bright green, representing used cpu time */
    let mut nBarsUsedKernel: core::ffi::c_int = 0;
    /* Bottom bars that are "used", i.e. are bright green, representing used cpu kernel time */
    let mut nBarsFree: core::ffi::c_int = 0;
    /* Top bars that are "unused", i.e. are dark green, representing free cpu time */
    let mut i: core::ffi::c_int = 0;

    /*
     * Get the client area rectangle
     */
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rcClient));

    /*
     * Fill it with blackness
     */
    FillSolidRect(hDC, core::ptr::addr_of_mut!(rcClient), RGB!(0, 0, 0));

    /*
     * Get the CPU usage
     */
    CpuUsage = PerfDataGetProcessorUsage();

    wsprintfW(core::ptr::addr_of_mut!(Text) as *mut WCHAR, RUSTCONV_WSTR_C5D4AD672797.as_ptr(), CpuUsage as core::ffi::c_int);

    /*
     * Draw the font text onto the graph
     */
    rcText = rcClient;
    InflateRect(core::ptr::addr_of_mut!(rcText), -2, -2);
    crPrevForeground = SetTextColor(hDC, RGB!(0, 255, 0));
    hOldFont = SelectObject(hDC, GetStockObject((DEFAULT_GUI_FONT as i32))) as HFONT;
    DrawTextW(hDC, addr_of_mut!(Text) as *mut WCHAR as LPCWSTR, -1, core::ptr::addr_of_mut!(rcText), (DT_BOTTOM | DT_CENTER | DT_NOPREFIX | DT_SINGLELINE) as UINT);
    SelectObject(hDC, hOldFont as HGDIOBJ);
    SetTextColor(hDC, crPrevForeground);

    /*
     * Draw the graph. So first find out how many bars we can fit
     */
    nBars = (((rcClient.bottom - rcClient.top) - 25) / 3) as core::ffi::c_int;
    nBarsUsed = ((nBars as ULONG).wrapping_mul(CpuUsage) / 100) as core::ffi::c_int;
    if (CpuUsage != 0) && (nBarsUsed == 0) {
        nBarsUsed = 1;
    }
    nBarsFree = nBars - (if nlastBarsUsed > nBarsUsed { nlastBarsUsed } else { nBarsUsed });

    if TaskManagerSettings.ShowKernelTimes != 0 {
        CpuKernelUsage = PerfDataGetProcessorSystemUsage();
        nBarsUsedKernel = ((nBars as ULONG).wrapping_mul(CpuKernelUsage) / 100) as core::ffi::c_int;
    } else {
        nBarsUsedKernel = 0;
    }

    /*
     * Draw the bar graph
     */
    rcBarLeft.left = (((rcClient.right - rcClient.left) - 33) / 2) as LONG;
    rcBarLeft.right = rcBarLeft.left + 16;
    rcBarRight.left = rcBarLeft.left + 17;
    rcBarRight.right = rcBarLeft.right + 17;
    rcBarLeft.top = 5;
    rcBarRight.top = rcBarLeft.top;
    rcBarLeft.bottom = 7;
    rcBarRight.bottom = rcBarLeft.bottom;

    if nBarsUsed < 0 {
        nBarsUsed = 0;
    }
    if nBarsUsed > nBars {
        nBarsUsed = nBars;
    }

    if nBarsFree < 0 {
        nBarsFree = 0;
    }
    if nBarsFree > nBars {
        nBarsFree = nBars;
    }

    if nBarsUsedKernel < 0 {
        nBarsUsedKernel = 0;
    }
    if nBarsUsedKernel > nBars {
        nBarsUsedKernel = nBars;
    }

    /*
     * Draw the "free" bars
     */
    i = 0;
    while i < nBarsFree {
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarLeft), DARK_GREEN);
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarRight), DARK_GREEN);

        rcBarLeft.top += 3;
        rcBarLeft.bottom += 3;

        rcBarRight.top += 3;
        rcBarRight.bottom += 3;
        i += 1;
    }

    /*
     * Draw the last "used" bars
     */
    if (nlastBarsUsed - nBarsUsed) > 0 {
        i = 0;
        while i < (nlastBarsUsed - nBarsUsed) {
            if nlastBarsUsed > 5000 {
                nlastBarsUsed = 5000;
            }

            FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarLeft), MEDIUM_GREEN);
            FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarRight), MEDIUM_GREEN);

            rcBarLeft.top += 3;
            rcBarLeft.bottom += 3;

            rcBarRight.top += 3;
            rcBarRight.bottom += 3;
            i += 1;
        }
    }
    nlastBarsUsed = nBarsUsed;
    /*
     * Draw the "used" bars
     */
    i = 0;
    while i < nBarsUsed {
        if nBarsUsed > 5000 {
            nBarsUsed = 5000;
        }

        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarLeft), BRIGHT_GREEN);
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarRight), BRIGHT_GREEN);

        rcBarLeft.top += 3;
        rcBarLeft.bottom += 3;

        rcBarRight.top += 3;
        rcBarRight.bottom += 3;
        i += 1;
    }

    /*
     * Draw the "used" kernel bars
     */

    rcBarLeft.top -= 3;
    rcBarLeft.bottom -= 3;

    rcBarRight.top -= 3;
    rcBarRight.bottom -= 3;

    i = 0;
    while i < nBarsUsedKernel {
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarLeft), RED);
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarRight), RED);

        rcBarLeft.top -= 3;
        rcBarLeft.bottom -= 3;

        rcBarRight.top -= 3;
        rcBarRight.bottom -= 3;
        i += 1;
    }

    SelectObject(hDC, hOldFont as HGDIOBJ);
}
#[no_mangle]
pub unsafe extern "C" fn Graph_DrawMemUsageGraph(hDC: HDC, hWnd: HWND) {
    let mut rcClient: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcBarLeft: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcBarRight: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcText: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut crPrevForeground: COLORREF = 0;
    let mut Text: [WCHAR; 260] = [0; 260];
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut CommitChargeTotal: ULONGLONG = 0;
    let mut CommitChargeLimit: ULONGLONG = 0;
    let mut nBars: i32 = 0;
    let mut nBarsUsed: i32 = 0;
    let mut nBarsFree: i32 = 0;
    let mut i: i32 = 0;

    /*
     * Get the client area rectangle
     */
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rcClient));

    /*
     * Fill it with blackness
     */
    FillSolidRect(hDC, core::ptr::addr_of_mut!(rcClient), RGB!(0, 0, 0));

    /*
     * Get the memory usage
     */
    CommitChargeTotal = PerfDataGetCommitChargeTotalK() as ULONGLONG;
    CommitChargeLimit = PerfDataGetCommitChargeLimitK() as ULONGLONG;

    if CommitChargeTotal > 1024 {
        wsprintfW(core::ptr::addr_of_mut!(Text) as LPWSTR, RUSTCONV_WSTR_23CC5F20442A.as_ptr(), (CommitChargeTotal.wrapping_div(1024)) as i32);
    } else {
        wsprintfW(core::ptr::addr_of_mut!(Text) as LPWSTR, RUSTCONV_WSTR_BD0EF5CBF459.as_ptr(), CommitChargeTotal as i32);
    }
    /*
     * Draw the font text onto the graph
     */
    rcText = rcClient;
    InflateRect(core::ptr::addr_of_mut!(rcText), -2, -2);
    crPrevForeground = SetTextColor(hDC, RGB!(0, 255, 0));
    hOldFont = SelectObject(hDC, GetStockObject((DEFAULT_GUI_FONT as i32))) as HFONT;
    DrawTextW(hDC, core::ptr::addr_of_mut!(Text) as *mut WCHAR as LPCWSTR, -1, core::ptr::addr_of_mut!(rcText), (DT_BOTTOM | DT_CENTER | DT_NOPREFIX | DT_SINGLELINE) as UINT);
    SelectObject(hDC, hOldFont as HGDIOBJ);
    SetTextColor(hDC, crPrevForeground);

    /*
     * Draw the graph. So first find out how many bars we can fit
     */
    nBars = ((rcClient.bottom - rcClient.top) - 25) / 3;
    if CommitChargeLimit != 0 {
        nBarsUsed = (nBars * ((CommitChargeTotal.wrapping_mul(100)).wrapping_div(CommitChargeLimit)) as i32) / 100;
    }
    nBarsFree = nBars - nBarsUsed;

    if nBarsUsed < 0 {
        nBarsUsed = 0;
    }
    if nBarsUsed > nBars {
        nBarsUsed = nBars;
    }

    if nBarsFree < 0 {
        nBarsFree = 0;
    }
    if nBarsFree > nBars {
        nBarsFree = nBars;
    }

    /*
     * Draw the bar graph
     */
    rcBarLeft.left = ((rcClient.right - rcClient.left) - 33) / 2;
    rcBarLeft.right = rcBarLeft.left + 16;
    rcBarRight.left = rcBarLeft.left + 17;
    rcBarRight.right = rcBarLeft.right + 17;
    rcBarRight.top = 5;
    rcBarLeft.top = rcBarRight.top;
    rcBarRight.bottom = 7;
    rcBarLeft.bottom = rcBarRight.bottom;

    /*
     * Draw the "free" bars
     */
    i = 0;
    while i < nBarsFree {
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarLeft), DARK_GREEN);
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarRight), DARK_GREEN);

        rcBarLeft.top += 3;
        rcBarLeft.bottom += 3;

        rcBarRight.top += 3;
        rcBarRight.bottom += 3;
        i += 1;
    }

    /*
     * Draw the "used" bars
     */
    i = 0;
    while i < nBarsUsed {
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarLeft), BRIGHT_GREEN);
        FillSolidRect(hDC, core::ptr::addr_of_mut!(rcBarRight), BRIGHT_GREEN);

        rcBarLeft.top += 3;
        rcBarLeft.bottom += 3;

        rcBarRight.top += 3;
        rcBarRight.bottom += 3;
        i += 1;
    }

    SelectObject(hDC, hOldFont as HGDIOBJ);
}
#[no_mangle]
pub unsafe extern "C" fn Graph_DrawMemUsageHistoryGraph(hDC: HDC, hWnd: HWND) {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut i: ::core::ffi::c_int;
    static mut offset: ::core::ffi::c_int = 0;

    let fresh31 = offset;
    offset = offset + 1;
    if fresh31 >= 10 {
        offset = 0;
    }

    /*
     * Get the client area rectangle
     */
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rcClient));

    /*
     * Fill it with blackness
     */
    FillSolidRect(
        hDC,
        core::ptr::addr_of_mut!(rcClient),
        RGB!(0, 0, 0),
    );

    /*
     * Draw the graph background and horizontal bars
     */
    i = 0;
    while (i as LONG) < rcClient.bottom {
        if (i % 11) == 0 {
            // FillSolidRect2(hDC, 0, i, rcClient.right, 1, DARK_GREEN);
        }
        i += 1;
    }

    /*
     * Draw the vertical bars
     */
    i = 11;
    while (i as LONG) < rcClient.right + offset as LONG {
        if (i % 11) == 0 {
            // FillSolidRect2(hDC, i - offset, 0, 1, rcClient.bottom, DARK_GREEN);
        }
        i += 1;
    }

    /*
     * Draw the memory usage
     */
    i = rcClient.right as ::core::ffi::c_int;
    while i >= 0 {
        i -= 1;
    }
}

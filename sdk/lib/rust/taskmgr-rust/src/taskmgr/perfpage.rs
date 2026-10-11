use crate::taskmgr::taskmgr::EndLocalThread;
use crate::taskmgr::taskmgr::GetRefreshMessage;
use crate::taskmgr::graphctl::GraphCtrl_AddPoint;
use crate::taskmgr::graphctl::GraphCtrl_Create;
use crate::taskmgr::graphctl::GraphCtrl_Dispose;
use crate::taskmgr::graphctl::GraphCtrl_RedrawBitmap;
use crate::taskmgr::graphctl::GraphCtrl_WndProc;
use crate::taskmgr::graph::Graph_WndProc;
use crate::taskmgr::perfdata::PerfDataGetCommitChargeLimitK;
use crate::taskmgr::perfdata::PerfDataGetCommitChargePeakK;
use crate::taskmgr::perfdata::PerfDataGetCommitChargeTotalK;
use crate::taskmgr::perfdata::PerfDataGetKernelMemoryNonPagedK;
use crate::taskmgr::perfdata::PerfDataGetKernelMemoryPagedK;
use crate::taskmgr::perfdata::PerfDataGetKernelMemoryTotalK;
use crate::taskmgr::perfdata::PerfDataGetPhysicalMemoryAvailableK;
use crate::taskmgr::perfdata::PerfDataGetPhysicalMemorySystemCacheK;
use crate::taskmgr::perfdata::PerfDataGetPhysicalMemoryTotalK;
use crate::taskmgr::perfdata::PerfDataGetProcessCount;
use crate::taskmgr::perfdata::PerfDataGetProcessorSystemUsage;
use crate::taskmgr::perfdata::PerfDataGetProcessorUsage;
use crate::taskmgr::perfdata::PerfDataGetSystemHandleCount;
use crate::taskmgr::perfdata::PerfDataGetTotalThreadCount;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    static mut bTrackMenu: BOOL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PerformancePageCpuUsageHistoryGraph: TM_GRAPH_CONTROL = _TM_GRAPH_CONTROL {
    hParentWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hdcGraph: ::core::ptr::null::<HDC__>() as *mut HDC__,
    hbmGraph: ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__,
    hPenGrid: ::core::ptr::null::<HPEN__>() as *mut HPEN__,
    hPen0: ::core::ptr::null::<HPEN__>() as *mut HPEN__,
    hPen1: ::core::ptr::null::<HPEN__>() as *mut HPEN__,
    hBrushBack: ::core::ptr::null::<HBRUSH__>() as *mut HBRUSH__,
    BitmapWidth: 0,
    BitmapHeight: 0,
    GridCellWidth: 0,
    GridCellHeight: 0,
    CurrShift: 0,
    PointBuffer: ::core::ptr::null::<BYTE>() as *mut BYTE,
    NumberOfPoints: 0,
    CurrIndex: 0,
    ftPixelsPerPercent: 0.,
    DrawSecondaryPlot: 0,
};

#[no_mangle]
pub static mut PerformancePageMemUsageHistoryGraph: TM_GRAPH_CONTROL = _TM_GRAPH_CONTROL {
    hParentWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hdcGraph: ::core::ptr::null::<HDC__>() as *mut HDC__,
    hbmGraph: ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__,
    hPenGrid: ::core::ptr::null::<HPEN__>() as *mut HPEN__,
    hPen0: ::core::ptr::null::<HPEN__>() as *mut HPEN__,
    hPen1: ::core::ptr::null::<HPEN__>() as *mut HPEN__,
    hBrushBack: ::core::ptr::null::<HBRUSH__>() as *mut HBRUSH__,
    BitmapWidth: 0,
    BitmapHeight: 0,
    GridCellWidth: 0,
    GridCellHeight: 0,
    CurrShift: 0,
    PointBuffer: ::core::ptr::null::<BYTE>() as *mut BYTE,
    NumberOfPoints: 0,
    CurrIndex: 0,
    ftPixelsPerPercent: 0.,
    DrawSecondaryPlot: 0,
};

#[no_mangle]
pub static mut hPerformancePage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hCpuUsageGraph: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hMemUsageGraph: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hPerformancePageCpuUsageHistoryGraph: HWND =
    ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hPerformancePageMemUsageHistoryGraph: HWND =
    ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hTotalsFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hCommitChargeFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hKernelMemoryFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hPhysicalMemoryFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hCpuUsageFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hMemUsageFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hCpuUsageHistoryFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hMemUsageHistoryFrame: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hCommitChargeTotalEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hCommitChargeLimitEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hCommitChargePeakEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hKernelMemoryTotalEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hKernelMemoryPagedEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hKernelMemoryNonPagedEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hPhysicalMemoryTotalEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hPhysicalMemoryAvailableEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hPhysicalMemorySystemCacheEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hTotalsHandleCountEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hTotalsProcessCountEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hTotalsThreadCountEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hPerformanceThread: HANDLE = NULL;

static mut dwPerformanceThread: DWORD = 0;

static mut nPerformancePageWidth: ::core::ffi::c_int = 0;

static mut nPerformancePageHeight: ::core::ffi::c_int = 0;

static mut lastX: ::core::ffi::c_int = 0;

static mut lastY: ::core::ffi::c_int = 0;

// #ifdef RUN_PERF_PAGE
// #endif
#[no_mangle]
pub unsafe extern "C" fn AdjustFrameSize(hCntrl: HWND, hDlg: HWND, nXDifference: i32, nYDifference: i32, pos: i32) {
    let mut rc: RECT = core::mem::zeroed();
    let mut cx: i32 = 0;
    let mut cy: i32 = 0;
    let mut sx: i32 = 0;
    let mut sy: i32 = 0;
    GetClientRect(hCntrl, core::ptr::addr_of_mut!(rc));
    MapWindowPoints(
        hCntrl,
        hDlg,
        core::ptr::addr_of_mut!(rc) as PRECT as LPPOINT,
        (core::mem::size_of::<RECT>() / (core::mem::size_of::<POINT>() as usize)) as UINT,
    );
    if pos != 0 {
        cx = rc.left;
        cy = rc.top;
        sx = rc.right - rc.left;
        match pos {
            _ if pos == 1 => {}
            _ if pos == 2 => {
                cy += nYDifference / 2;
            }
            _ if pos == 3 => {
                sx += nXDifference;
            }
            _ if pos == 4 => {
                cy += nYDifference / 2;
                sx += nXDifference;
            }
            _ => {}
        }
        sy = rc.bottom - rc.top + nYDifference / 2;
        SetWindowPos(
            hCntrl,
            core::ptr::null_mut(),
            cx,
            cy,
            sx,
            sy,
            (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER) as UINT,
        );
    } else {
        cx = rc.left + nXDifference;
        cy = rc.top + nYDifference;
        SetWindowPos(
            hCntrl,
            core::ptr::null_mut(),
            cx,
            cy,
            0,
            0,
            (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
        );
    }
    InvalidateRect(hCntrl, core::ptr::null_mut(), (TRUE) as BOOL);
}

unsafe extern "C" fn AdjustControlPosition(hCntrl: HWND, hDlg: HWND, nXDifference: i32, nYDifference: i32) {
    AdjustFrameSize(hCntrl, hDlg, nXDifference, nYDifference, 0);
}

unsafe extern "C" fn AdjustCntrlPos(ctrl_id: ::core::ffi::c_int, hDlg: HWND, nXDifference: ::core::ffi::c_int, nYDifference: ::core::ffi::c_int) {
    AdjustFrameSize(GetDlgItem(hDlg, ctrl_id), hDlg, nXDifference, nYDifference, 0);
}

#[no_mangle]
pub unsafe extern "system" fn PerformancePageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut rc: RECT = core::mem::zeroed();

    match message {
        WM_DESTROY => {
            /* Stop the refresh thread BEFORE freeing what it draws into: it calls
             * GraphCtrl_AddPoint on every tick, and disposing first let a tick write the
             * CPU percentage into the freed PointBuffer (heap corruption, then a crash in
             * the heap; seen as 1 run in 3 under x86-64 emulation, where ticks are slow
             * and the CPU reads 100). */
            EndLocalThread(core::ptr::addr_of_mut!(hPerformanceThread), dwPerformanceThread);
            GraphCtrl_Dispose(core::ptr::addr_of_mut!(PerformancePageCpuUsageHistoryGraph));
            GraphCtrl_Dispose(core::ptr::addr_of_mut!(PerformancePageMemUsageHistoryGraph));
        }

        WM_INITDIALOG => {
            let mut bGraph: BOOL = 0;
            let mut fmt: TM_FORMAT = core::mem::zeroed();

            /* Save the width and height */
            GetClientRect(hDlg, core::ptr::addr_of_mut!(rc));
            nPerformancePageWidth = rc.right as i32;
            nPerformancePageHeight = rc.bottom as i32;

            /* Update window position */
            SetWindowPos(
                hDlg,
                core::ptr::null_mut(),
                15,
                30,
                0,
                0,
                (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
            );

            /*
             * Get handles to all the controls
             */
            hTotalsFrame = GetDlgItem(hDlg, (IDC_TOTALS_FRAME as i32));
            hCommitChargeFrame = GetDlgItem(hDlg, (IDC_COMMIT_CHARGE_FRAME as i32));
            hKernelMemoryFrame = GetDlgItem(hDlg, (IDC_KERNEL_MEMORY_FRAME as i32));
            hPhysicalMemoryFrame = GetDlgItem(hDlg, (IDC_PHYSICAL_MEMORY_FRAME as i32));

            hCpuUsageFrame = GetDlgItem(hDlg, (IDC_CPU_USAGE_FRAME as i32));
            hMemUsageFrame = GetDlgItem(hDlg, (IDC_MEM_USAGE_FRAME as i32));
            hCpuUsageHistoryFrame = GetDlgItem(hDlg, (IDC_CPU_USAGE_HISTORY_FRAME as i32));
            hMemUsageHistoryFrame = GetDlgItem(hDlg, (IDC_MEMORY_USAGE_HISTORY_FRAME as i32));

            hCommitChargeTotalEdit = GetDlgItem(hDlg, (IDC_COMMIT_CHARGE_TOTAL as i32));
            hCommitChargeLimitEdit = GetDlgItem(hDlg, (IDC_COMMIT_CHARGE_LIMIT as i32));
            hCommitChargePeakEdit = GetDlgItem(hDlg, (IDC_COMMIT_CHARGE_PEAK as i32));
            hKernelMemoryTotalEdit = GetDlgItem(hDlg, (IDC_KERNEL_MEMORY_TOTAL as i32));
            hKernelMemoryPagedEdit = GetDlgItem(hDlg, (IDC_KERNEL_MEMORY_PAGED as i32));
            hKernelMemoryNonPagedEdit = GetDlgItem(hDlg, (IDC_KERNEL_MEMORY_NONPAGED as i32));
            hPhysicalMemoryTotalEdit = GetDlgItem(hDlg, (IDC_PHYSICAL_MEMORY_TOTAL as i32));
            hPhysicalMemoryAvailableEdit = GetDlgItem(hDlg, (IDC_PHYSICAL_MEMORY_AVAILABLE as i32));
            hPhysicalMemorySystemCacheEdit = GetDlgItem(hDlg, (IDC_PHYSICAL_MEMORY_SYSTEM_CACHE as i32));
            hTotalsHandleCountEdit = GetDlgItem(hDlg, (IDC_TOTALS_HANDLE_COUNT as i32));
            hTotalsProcessCountEdit = GetDlgItem(hDlg, (IDC_TOTALS_PROCESS_COUNT as i32));
            hTotalsThreadCountEdit = GetDlgItem(hDlg, (IDC_TOTALS_THREAD_COUNT as i32));

            hCpuUsageGraph = GetDlgItem(hDlg, (IDC_CPU_USAGE_GRAPH as i32));
            hMemUsageGraph = GetDlgItem(hDlg, (IDC_MEM_USAGE_GRAPH as i32));
            hPerformancePageMemUsageHistoryGraph = GetDlgItem(hDlg, (IDC_MEM_USAGE_HISTORY_GRAPH as i32));
            hPerformancePageCpuUsageHistoryGraph = GetDlgItem(hDlg, (IDC_CPU_USAGE_HISTORY_GRAPH as i32));

            /* Create the controls */
            fmt.clrBack = RGB!(0, 0, 0);
            fmt.clrGrid = RGB!(0, 128, 64);
            fmt.clrPlot0 = RGB!(0, 255, 0);
            fmt.clrPlot1 = RGB!(255, 0, 0);
            fmt.GridCellHeight = 12;
            fmt.GridCellWidth = fmt.GridCellHeight;
            fmt.DrawSecondaryPlot = TaskManagerSettings.ShowKernelTimes;
            bGraph = GraphCtrl_Create(
                core::ptr::addr_of_mut!(PerformancePageCpuUsageHistoryGraph),
                hPerformancePageCpuUsageHistoryGraph,
                hDlg,
                core::ptr::addr_of_mut!(fmt),
            );
            if bGraph == 0 {
                EndDialog(hDlg, 0);
                return (FALSE) as INT_PTR;
            }

            fmt.clrPlot0 = RGB!(255, 255, 0);
            fmt.clrPlot1 = RGB!(100, 255, 255);
            fmt.DrawSecondaryPlot = (TRUE) as i32;
            bGraph = GraphCtrl_Create(
                core::ptr::addr_of_mut!(PerformancePageMemUsageHistoryGraph),
                hPerformancePageMemUsageHistoryGraph,
                hDlg,
                core::ptr::addr_of_mut!(fmt),
            );
            if bGraph == 0 {
                EndDialog(hDlg, 0);
                return (FALSE) as INT_PTR;
            }

            /* Start our refresh thread */
            hPerformanceThread = CreateThread(
                core::ptr::null_mut(),
                0,
                Some(PerformancePageRefreshThread),
                core::ptr::null_mut(),
                0,
                core::ptr::addr_of_mut!(dwPerformanceThread),
            );

            /*
             * Subclass graph buttons
             */
            OldGraphWndProc = ::core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtrW(
                hCpuUsageGraph,
                GWLP_WNDPROC,
                Graph_WndProc as LONG_PTR,
            ));
            SetWindowLongPtrW(hMemUsageGraph, GWLP_WNDPROC, Graph_WndProc as LONG_PTR);
            OldGraphCtrlWndProc = ::core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtrW(
                hPerformancePageMemUsageHistoryGraph,
                GWLP_WNDPROC,
                GraphCtrl_WndProc as LONG_PTR,
            ));
            SetWindowLongPtrW(
                hPerformancePageCpuUsageHistoryGraph,
                GWLP_WNDPROC,
                GraphCtrl_WndProc as LONG_PTR,
            );
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {}

        WM_SIZE => {
            let mut cx: i32 = 0;
            let mut cy: i32 = 0;
            let mut nXDifference: i32 = 0;
            let mut nYDifference: i32 = 0;

            if wParam == SIZE_MINIMIZED as WPARAM {
                return 0;
            }

            cx = LOWORD!(lParam) as i32;
            cy = HIWORD!(lParam) as i32;
            nXDifference = cx - nPerformancePageWidth;
            nYDifference = cy - nPerformancePageHeight;
            nPerformancePageWidth = cx;
            nPerformancePageHeight = cy;

            /* Reposition the performance page's controls */
            AdjustFrameSize(hTotalsFrame, hDlg, 0, nYDifference, 0);
            AdjustFrameSize(hCommitChargeFrame, hDlg, 0, nYDifference, 0);
            AdjustFrameSize(hKernelMemoryFrame, hDlg, 0, nYDifference, 0);
            AdjustFrameSize(hPhysicalMemoryFrame, hDlg, 0, nYDifference, 0);
            AdjustCntrlPos((IDS_COMMIT_CHARGE_TOTAL as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_COMMIT_CHARGE_LIMIT as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_COMMIT_CHARGE_PEAK as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_KERNEL_MEMORY_TOTAL as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_KERNEL_MEMORY_PAGED as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_KERNEL_MEMORY_NONPAGED as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_PHYSICAL_MEMORY_TOTAL as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_PHYSICAL_MEMORY_AVAILABLE as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_PHYSICAL_MEMORY_SYSTEM_CACHE as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_TOTALS_HANDLE_COUNT as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_TOTALS_PROCESS_COUNT as i32), hDlg, 0, nYDifference);
            AdjustCntrlPos((IDS_TOTALS_THREAD_COUNT as i32), hDlg, 0, nYDifference);

            AdjustControlPosition(hCommitChargeTotalEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hCommitChargeLimitEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hCommitChargePeakEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hKernelMemoryTotalEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hKernelMemoryPagedEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hKernelMemoryNonPagedEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hPhysicalMemoryTotalEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hPhysicalMemoryAvailableEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hPhysicalMemorySystemCacheEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hTotalsHandleCountEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hTotalsProcessCountEdit, hDlg, 0, nYDifference);
            AdjustControlPosition(hTotalsThreadCountEdit, hDlg, 0, nYDifference);

            nXDifference += lastX;
            nYDifference += lastY;
            lastY = 0;
            lastX = lastY;
            if nXDifference % 2 != 0 {
                if nXDifference > 0 {
                    nXDifference -= 1;
                    lastX += 1;
                } else {
                    nXDifference += 1;
                    lastX -= 1;
                }
            }
            if nYDifference % 2 != 0 {
                if nYDifference > 0 {
                    nYDifference -= 1;
                    lastY += 1;
                } else {
                    nYDifference += 1;
                    lastY -= 1;
                }
            }
            AdjustFrameSize(hCpuUsageFrame, hDlg, nXDifference, nYDifference, 1);
            AdjustFrameSize(hMemUsageFrame, hDlg, nXDifference, nYDifference, 2);
            AdjustFrameSize(hCpuUsageHistoryFrame, hDlg, nXDifference, nYDifference, 3);
            AdjustFrameSize(hMemUsageHistoryFrame, hDlg, nXDifference, nYDifference, 4);
            AdjustFrameSize(hCpuUsageGraph, hDlg, nXDifference, nYDifference, 1);
            AdjustFrameSize(hMemUsageGraph, hDlg, nXDifference, nYDifference, 2);
            AdjustFrameSize(hPerformancePageCpuUsageHistoryGraph, hDlg, nXDifference, nYDifference, 3);
            AdjustFrameSize(hPerformancePageMemUsageHistoryGraph, hDlg, nXDifference, nYDifference, 4);
        }

        _ => {}
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn RefreshPerformancePage() {
    PostThreadMessage(
        dwPerformanceThread,
        WM_TIMER as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PerformancePageRefreshThread(Parameter: PVOID) -> DWORD {
    let mut CommitChargeTotal: ULONGLONG = 0;
    let mut CommitChargeLimit: ULONGLONG = 0;
    let mut CommitChargePeak: ULONGLONG = 0;

    let mut CpuUsage: ULONG = 0;
    let mut CpuKernelUsage: ULONG = 0;

    let mut KernelMemoryTotal: ULONGLONG = 0;
    let mut KernelMemoryPaged: ULONGLONG = 0;
    let mut KernelMemoryNonPaged: ULONGLONG = 0;

    let mut PhysicalMemoryTotal: ULONGLONG = 0;
    let mut PhysicalMemoryAvailable: ULONGLONG = 0;
    let mut PhysicalMemorySystemCache: ULONGLONG = 0;

    let mut TotalHandles: ULONG = 0;
    let mut TotalThreads: ULONG = 0;
    let mut TotalProcesses: ULONG = 0;

    let mut msg: MSG = core::mem::zeroed();

    let mut Text: [WCHAR; 260] = [0; 260];
    let mut szMemUsage: [WCHAR; 256] = [0; 256];
    let mut szCpuUsage: [WCHAR; 256] = [0; 256];
    let mut szProcesses: [WCHAR; 256] = [0; 256];

    LoadStringW(
        hInst,
        IDS_STATUS_CPUUSAGE as UINT,
        core::ptr::addr_of_mut!(szCpuUsage) as *mut WCHAR,
        _countof!(szCpuUsage) as core::ffi::c_int,
    );
    LoadStringW(
        hInst,
        IDS_STATUS_MEMUSAGE as UINT,
        core::ptr::addr_of_mut!(szMemUsage) as *mut WCHAR,
        _countof!(szMemUsage) as core::ffi::c_int,
    );
    LoadStringW(
        hInst,
        IDS_STATUS_PROCESSES as UINT,
        core::ptr::addr_of_mut!(szProcesses) as *mut WCHAR,
        _countof!(szProcesses) as core::ffi::c_int,
    );

    while 1 != 0 {

        let mut nBarsUsed1: core::ffi::c_int = 0;
        let mut nBarsUsed2: core::ffi::c_int = 0;

        let mut szChargeTotalFormat: [WCHAR; 256] = [0; 256];
        let mut szChargeLimitFormat: [WCHAR; 256] = [0; 256];

        /* Wait for an the event or application close */
        if GetRefreshMessage(core::ptr::addr_of_mut!(msg)) == 0 {
            return 0 as DWORD;
        }

        if msg.message == WM_TIMER as UINT {
            /*
             * Update the commit charge info
             */
            CommitChargeTotal = PerfDataGetCommitChargeTotalK() as ULONGLONG;
            CommitChargeLimit = PerfDataGetCommitChargeLimitK() as ULONGLONG;
            CommitChargePeak = PerfDataGetCommitChargePeakK() as ULONGLONG;
            _ultow(
                CommitChargeTotal as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hCommitChargeTotalEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                CommitChargeLimit as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hCommitChargeLimitEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                CommitChargePeak as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hCommitChargePeakEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );

            StrFormatByteSizeW(
                CommitChargeTotal.wrapping_mul(1024 as ULONGLONG) as LONGLONG,
                core::ptr::addr_of_mut!(szChargeTotalFormat) as *mut WCHAR,
                _countof!(szChargeTotalFormat) as UINT,
            );

            StrFormatByteSizeW(
                CommitChargeLimit.wrapping_mul(1024 as ULONGLONG) as LONGLONG,
                core::ptr::addr_of_mut!(szChargeLimitFormat) as *mut WCHAR,
                _countof!(szChargeLimitFormat) as UINT,
            );

            if bTrackMenu == 0 {
                wsprintfW(
                    core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                    core::ptr::addr_of_mut!(szMemUsage) as *mut WCHAR as *const WCHAR,
                    core::ptr::addr_of_mut!(szChargeTotalFormat) as *mut WCHAR,
                    core::ptr::addr_of_mut!(szChargeLimitFormat) as *mut WCHAR,
                    (if CommitChargeLimit != 0 {
                        CommitChargeTotal
                            .wrapping_mul(100 as ULONGLONG)
                            .wrapping_div(CommitChargeLimit)
                    } else {
                        0 as ULONGLONG
                    }),
                );
                SendMessageW(
                    hStatusWnd,
                    SB_SETTEXT as UINT,
                    2 as WPARAM,
                    core::ptr::addr_of_mut!(Text) as *mut WCHAR as LPARAM,
                );
            }

            /*
             * Update the kernel memory info
             */
            KernelMemoryTotal = PerfDataGetKernelMemoryTotalK() as ULONGLONG;
            KernelMemoryPaged = PerfDataGetKernelMemoryPagedK() as ULONGLONG;
            KernelMemoryNonPaged = PerfDataGetKernelMemoryNonPagedK() as ULONGLONG;
            _ultow(
                KernelMemoryTotal as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hKernelMemoryTotalEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                KernelMemoryPaged as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hKernelMemoryPagedEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                KernelMemoryNonPaged as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hKernelMemoryNonPagedEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );

            /*
             * Update the physical memory info
             */
            PhysicalMemoryTotal = PerfDataGetPhysicalMemoryTotalK() as ULONGLONG;
            PhysicalMemoryAvailable = PerfDataGetPhysicalMemoryAvailableK() as ULONGLONG;
            PhysicalMemorySystemCache = PerfDataGetPhysicalMemorySystemCacheK() as ULONGLONG;
            _ultow(
                PhysicalMemoryTotal as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hPhysicalMemoryTotalEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                PhysicalMemoryAvailable as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hPhysicalMemoryAvailableEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                PhysicalMemorySystemCache as ULONG,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hPhysicalMemorySystemCacheEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );

            /*
             * Update the totals info
             */
            TotalHandles = PerfDataGetSystemHandleCount() as ULONG;
            TotalThreads = PerfDataGetTotalThreadCount() as ULONG;
            TotalProcesses = PerfDataGetProcessCount() as ULONG;
            _ultow(
                TotalHandles,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hTotalsHandleCountEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                TotalThreads,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hTotalsThreadCountEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            _ultow(
                TotalProcesses,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                10 as core::ffi::c_int,
            );
            SetWindowTextW(
                hTotalsProcessCountEdit,
                core::ptr::addr_of_mut!(Text) as *mut WCHAR as *const WCHAR,
            );
            if bTrackMenu == 0 {
                wsprintfW(
                    core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                    core::ptr::addr_of_mut!(szProcesses) as *mut WCHAR as *const WCHAR,
                    TotalProcesses,
                );
                SendMessageW(
                    hStatusWnd,
                    SB_SETTEXT as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(Text) as *mut WCHAR as LPARAM,
                );
            }

            /*
             * Redraw the graphs
             */
            InvalidateRect(
                hCpuUsageGraph,
                core::ptr::null::<RECT>(),
                FALSE as BOOL,
            );
            InvalidateRect(
                hMemUsageGraph,
                core::ptr::null::<RECT>(),
                FALSE as BOOL,
            );

            /*
             * Get the CPU usage
             */
            CpuUsage = PerfDataGetProcessorUsage() as ULONG;
            CpuKernelUsage = PerfDataGetProcessorSystemUsage() as ULONG;

            if bTrackMenu == 0 {
                wsprintfW(
                    core::ptr::addr_of_mut!(Text) as *mut WCHAR,
                    core::ptr::addr_of_mut!(szCpuUsage) as *mut WCHAR as *const WCHAR,
                    CpuUsage,
                );
                SendMessageW(
                    hStatusWnd,
                    SB_SETTEXT as UINT,
                    1 as WPARAM,
                    core::ptr::addr_of_mut!(Text) as *mut WCHAR as LPARAM,
                );
            }

            /*
             * Get the memory usage
             */
            CommitChargeTotal = PerfDataGetCommitChargeTotalK() as ULONGLONG;
            CommitChargeLimit = PerfDataGetCommitChargeLimitK() as ULONGLONG;
            nBarsUsed1 = (if CommitChargeLimit != 0 {
                CommitChargeTotal
                    .wrapping_mul(100 as ULONGLONG)
                    .wrapping_div(CommitChargeLimit)
            } else {
                0 as ULONGLONG
            }) as core::ffi::c_int;

            PhysicalMemoryTotal = PerfDataGetPhysicalMemoryTotalK() as ULONGLONG;
            PhysicalMemoryAvailable = PerfDataGetPhysicalMemoryAvailableK() as ULONGLONG;
            nBarsUsed2 = (if PhysicalMemoryTotal != 0 {
                PhysicalMemoryAvailable
                    .wrapping_mul(100 as ULONGLONG)
                    .wrapping_div(PhysicalMemoryTotal)
            } else {
                0 as ULONGLONG
            }) as core::ffi::c_int;

            GraphCtrl_AddPoint(
                core::ptr::addr_of_mut!(PerformancePageCpuUsageHistoryGraph),
                CpuUsage as BYTE,
                CpuKernelUsage as BYTE,
            );
            GraphCtrl_AddPoint(
                core::ptr::addr_of_mut!(PerformancePageMemUsageHistoryGraph),
                nBarsUsed1 as BYTE,
                nBarsUsed2 as BYTE,
            );
            InvalidateRect(
                hPerformancePageMemUsageHistoryGraph,
                core::ptr::null::<RECT>(),
                FALSE as BOOL,
            );
            InvalidateRect(
                hPerformancePageCpuUsageHistoryGraph,
                core::ptr::null::<RECT>(),
                FALSE as BOOL,
            );
        }
    }
    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn PerformancePage_OnViewShowKernelTimes() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hViewMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hViewMenu = GetSubMenu(hMenu, 2 as ::core::ffi::c_int);

    /* Check or uncheck the show 16-bit tasks menu item */
    if GetMenuState(hViewMenu, ID_VIEW_SHOWKERNELTIMES as UINT, MF_BYCOMMAND as UINT) & MF_CHECKED as UINT != 0
    {
        CheckMenuItem(
            hViewMenu,
            ID_VIEW_SHOWKERNELTIMES as UINT,
            (MF_BYCOMMAND | MF_UNCHECKED) as UINT,
        );
        TaskManagerSettings.ShowKernelTimes = FALSE as BOOL;
        PerformancePageCpuUsageHistoryGraph.DrawSecondaryPlot = FALSE as BOOL;
    }
    else
    {
        CheckMenuItem(
            hViewMenu,
            ID_VIEW_SHOWKERNELTIMES as UINT,
            (MF_BYCOMMAND | MF_CHECKED) as UINT,
        );
        TaskManagerSettings.ShowKernelTimes = TRUE as BOOL;
        PerformancePageCpuUsageHistoryGraph.DrawSecondaryPlot = TRUE as BOOL;
    }

    GraphCtrl_RedrawBitmap(
        core::ptr::addr_of_mut!(PerformancePageCpuUsageHistoryGraph),
        PerformancePageCpuUsageHistoryGraph.BitmapHeight,
    );
    RefreshPerformancePage();
}
#[no_mangle]
pub unsafe extern "C" fn PerformancePage_OnViewCPUHistoryOneGraphAll() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hViewMenu: HMENU = core::ptr::null_mut();
    let mut hCPUHistoryMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hViewMenu = GetSubMenu(hMenu, 2);
    hCPUHistoryMenu = GetSubMenu(hViewMenu, 3);

    TaskManagerSettings.CPUHistory_OneGraphPerCPU = (FALSE) as i32;
    CheckMenuRadioItem(hCPUHistoryMenu, ID_VIEW_CPUHISTORY_ONEGRAPHALL, ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU, ID_VIEW_CPUHISTORY_ONEGRAPHALL, MF_BYCOMMAND);
}
#[no_mangle]
pub unsafe extern "C" fn PerformancePage_OnViewCPUHistoryOneGraphPerCPU() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hViewMenu: HMENU = core::ptr::null_mut();
    let mut hCPUHistoryMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hViewMenu = GetSubMenu(hMenu, 2 as ::core::ffi::c_int);
    hCPUHistoryMenu = GetSubMenu(hViewMenu, 3 as ::core::ffi::c_int);

    TaskManagerSettings.CPUHistory_OneGraphPerCPU = TRUE as BOOL;
    CheckMenuRadioItem(
        hCPUHistoryMenu,
        ID_VIEW_CPUHISTORY_ONEGRAPHALL as UINT,
        ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU as UINT,
        ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU as UINT,
        MF_BYCOMMAND as UINT,
    );
}

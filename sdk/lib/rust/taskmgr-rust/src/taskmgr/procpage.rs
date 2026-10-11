use crate::taskmgr::column::AddColumns;
use crate::taskmgr::taskmgr::EndLocalThread;
use crate::taskmgr::taskmgr::GetRefreshMessage;
use crate::taskmgr::perfdata::PerfDataGetBasePriority;
use crate::taskmgr::perfdata::PerfDataGetCPUTime;
use crate::taskmgr::perfdata::PerfDataGetCPUUsage;
use crate::taskmgr::perfdata::PerfDataGetCommandLine;
use crate::taskmgr::perfdata::PerfDataGetGDIObjectCount;
use crate::taskmgr::perfdata::PerfDataGetHandleCount;
use crate::taskmgr::perfdata::PerfDataGetIOCounters;
use crate::taskmgr::perfdata::PerfDataGetImageName;
use crate::taskmgr::perfdata::PerfDataGetNonPagedPoolUsagePages;
use crate::taskmgr::perfdata::PerfDataGetPageFaultCount;
use crate::taskmgr::perfdata::PerfDataGetPageFaultCountDelta;
use crate::taskmgr::perfdata::PerfDataGetPagedPoolUsagePages;
use crate::taskmgr::perfdata::PerfDataGetPeakWorkingSetSizeBytes;
use crate::taskmgr::perfdata::PerfDataGetProcessCount;
use crate::taskmgr::perfdata::PerfDataGetProcessId;
use crate::taskmgr::perfdata::PerfDataGetProcessIndex;
use crate::taskmgr::perfdata::PerfDataGetSessionId;
use crate::taskmgr::perfdata::PerfDataGetThreadCount;
use crate::taskmgr::perfdata::PerfDataGetUSERObjectCount;
use crate::taskmgr::perfdata::PerfDataGetUserName;
use crate::taskmgr::perfdata::PerfDataGetVirtualMemorySizeBytes;
use crate::taskmgr::perfdata::PerfDataGetWorkingSetSizeBytes;
use crate::taskmgr::perfdata::PerfDataGetWorkingSetSizeDelta;
use crate::taskmgr::proclist::ProcessListWndProc;
use crate::taskmgr::endproc::ProcessPage_OnEndProcess;
use crate::taskmgr::column::SaveColumnSettings;
use crate::taskmgr::column::UpdateColumnDataHints;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_48D9276C533E: [u16; 10] = [0x0050u16, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_5968F39826A0: [u16; 16] = [
    0x0066u16, 0x0069, 0x006C, 0x0065, 0x0062, 0x0072, 0x006F, 0x0077, 0x0073, 0x0065,
    0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_82DB442E7A60: [u16; 5] = [0x006Fu16, 0x0070, 0x0065, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_84FA0582B45C: [u16; 3] = [0x0020u16, 0x004B, 0x0000];
pub static RUSTCONV_WSTR_958F461AF768: [u16; 9] = [0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9B349295E8D5: [u16; 11] = [
    0x0070u16, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_9D0AE7D0581B: [u16; 6] = [0x0025u16, 0x0030, 0x0032, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_AB68B18B313D: [u16; 13] = [
    0x0065u16, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065,
    0x0078, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_BA75F9F8779D: [u16; 16] = [
    0x0025u16, 0x006C, 0x0075, 0x003A, 0x0025, 0x0030, 0x0032, 0x006C, 0x0075, 0x003A,
    0x0025, 0x0030, 0x0032, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_D7CB07DBF8E2: [u16; 9] = [0x0044u16, 0x0065, 0x0062, 0x0075, 0x0067, 0x0067, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_DEB4EFA22C30: [u16; 53] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0041, 0x0065, 0x0044, 0x0065, 0x0062,
    0x0075, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_E2CD87D16D16: [u16; 5] = [0x0025u16, 0x0073, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_EAFE2DCE91A2: [u16; 6] = [0x0025u16, 0x0049, 0x0036, 0x0034, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_F8FDF9BFDBA4: [u16; 9] = [0x0044u16, 0x0052, 0x0057, 0x0054, 0x0053, 0x004E, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_FB47CCC40FE6: [u16; 3] = [0x0025u16, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hProcessPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hProcessPageListCtrl: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hProcessPageHeaderCtrl: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hProcessPageEndProcessButton: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hProcessPageShowAllProcessesButton: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut bProcessPageSelectionMade: BOOL = (FALSE) as i32;

static mut nProcessPageWidth: ::core::ffi::c_int = 0;

static mut nProcessPageHeight: ::core::ffi::c_int = 0;

static mut hProcessThread: HANDLE = NULL;

static mut dwProcessThread: DWORD = 0;

#[repr(C)]
pub struct PROCESS_PAGE_LIST_ITEM {
    pub ProcessId: ULONG,
}
pub type LPPROCESS_PAGE_LIST_ITEM = *mut PROCESS_PAGE_LIST_ITEM;

// Function prototypes (not definitions; emitted by the per-function pipeline):
// ProcessPageCompareFunc, AddProcess, UpdateProcesses, gethmsfromlargeint,
// ProcessPageOnNotify, ProcessPageShowContextMenu, PerfDataGetText,
// ProcessPageRefreshThread, ProcessRunning
#[no_mangle]
pub unsafe extern "C" fn Cleanup() {
    let mut i: c_int = 0;
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut pData: LPPROCESS_PAGE_LIST_ITEM = core::ptr::null_mut();
    while i < ListView_GetItemCount!(hProcessPageListCtrl) {
        memset(
            core::ptr::addr_of_mut!(item) as *mut c_void,
            0 as c_int,
            core::mem::size_of::<LV_ITEM>() as size_t,
        );
        item.mask = LVIF_PARAM as UINT;
        item.iItem = i;
        let _ = ListView_GetItem!(hProcessPageListCtrl, core::ptr::addr_of_mut!(item));
        pData = item.lParam as LPPROCESS_PAGE_LIST_ITEM;
        HeapFree(GetProcessHeap(), 0 as DWORD, pData as PVOID);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ProcGetIndexByProcessId(dwProcessId: DWORD) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut item: LVITEM = core::mem::zeroed();
    let mut pData: LPPROCESS_PAGE_LIST_ITEM = core::ptr::null_mut();
    while i < ListView_GetItemCount!(hProcessPageListCtrl) {
        memset(
            core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<LV_ITEM>() as usize,
        );
        item.mask = LVIF_PARAM as UINT;
        item.iItem = i;
        let _ = ListView_GetItem!(hProcessPageListCtrl, core::ptr::addr_of_mut!(item));
        pData = item.lParam as LPPROCESS_PAGE_LIST_ITEM;
        if (*pData).ProcessId == dwProcessId {
            return i;
        }
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn GetSelectedProcessId() -> DWORD {
    let mut Index: ::core::ffi::c_int;
    let mut lvitem: LVITEM = core::mem::zeroed();

    if ListView_GetSelectedCount!(hProcessPageListCtrl) == 1 {
        Index = ListView_GetSelectionMark!(hProcessPageListCtrl);

        memset(
            core::ptr::addr_of_mut!(lvitem) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<LVITEM>() as size_t,
        );

        lvitem.mask = LVIF_PARAM;
        lvitem.iItem = Index;

        let _ = ListView_GetItem!(hProcessPageListCtrl, core::ptr::addr_of_mut!(lvitem));

        if lvitem.lParam != 0 {
            return (*(lvitem.lParam as LPPROCESS_PAGE_LIST_ITEM)).ProcessId;
        }
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPageUpdate() {
    if ListView_GetSelectedCount!(hProcessPageListCtrl) != 0 {
        EnableWindow(hProcessPageEndProcessButton, (TRUE) as BOOL);
    } else {
        EnableWindow(hProcessPageEndProcessButton, (FALSE) as BOOL);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ProcessPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut rc: RECT = core::mem::zeroed();
    let mut nXDifference: core::ffi::c_int;
    let mut nYDifference: core::ffi::c_int;
    let mut cx: core::ffi::c_int;
    let mut cy: core::ffi::c_int;

    match message {
        WM_INITDIALOG => {
            /*
             * Save the width and height
             */
            GetClientRect(hDlg, core::ptr::addr_of_mut!(rc));
            nProcessPageWidth = rc.right;
            nProcessPageHeight = rc.bottom;

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
             * Get handles to the controls
             */
            hProcessPageListCtrl = GetDlgItem(hDlg, (IDC_PROCESSLIST as i32));
            hProcessPageHeaderCtrl = ListView_GetHeader!(hProcessPageListCtrl);
            hProcessPageEndProcessButton = GetDlgItem(hDlg, (IDC_ENDPROCESS as i32));
            hProcessPageShowAllProcessesButton = GetDlgItem(hDlg, (IDC_SHOWALLPROCESSES as i32));

            /*
             * Set the title, and extended window styles for the list control
             */
            SetWindowTextW(hProcessPageListCtrl, RUSTCONV_WSTR_48D9276C533E.as_ptr());
            let _ = ListView_SetExtendedListViewStyle!(
                hProcessPageListCtrl,
                ListView_GetExtendedListViewStyle!(hProcessPageListCtrl) | LVS_EX_FULLROWSELECT | LVS_EX_HEADERDRAGDROP,
            );

            AddColumns();

            /*
             * Subclass the process list control so we can intercept WM_ERASEBKGND
             */
            OldProcessListWndProc = core::mem::transmute::<LONG_PTR, WNDPROC>(
                SetWindowLongPtrW(
                    hProcessPageListCtrl,
                    GWLP_WNDPROC,
                    (ProcessListWndProc as usize) as LONG_PTR,
                ),
            );

            /* Start our refresh thread */
            hProcessThread = CreateThread(
                core::ptr::null_mut(),
                0,
                Some(ProcessPageRefreshThread),
                core::ptr::null_mut(),
                0,
                core::ptr::addr_of_mut!(dwProcessThread),
            );

            /* Refresh page */
            ProcessPageUpdate();

            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            /* Close the event handle, this will make the */
            /* refresh thread exit when the wait fails */
            EndLocalThread(core::ptr::addr_of_mut!(hProcessThread), dwProcessThread);
            SaveColumnSettings();
            Cleanup();
            // break
        }

        WM_COMMAND => {
            /* Handle the button clicks */
            match (LOWORD!(wParam)) as u32 {
                IDC_ENDPROCESS => {
                    ProcessPage_OnEndProcess();
                }
                _ => {}
            }
            // break
        }

        WM_SIZE => {
            if wParam == (SIZE_MINIMIZED) as _ {
                return 0 as INT_PTR;
            }

            cx = LOWORD!(lParam) as core::ffi::c_int;
            cy = HIWORD!(lParam) as core::ffi::c_int;
            nXDifference = cx - nProcessPageWidth;
            nYDifference = cy - nProcessPageHeight;
            nProcessPageWidth = cx;
            nProcessPageHeight = cy;

            /* Reposition the application page's controls */
            GetWindowRect(hProcessPageListCtrl, core::ptr::addr_of_mut!(rc));
            cx = (rc.right - rc.left) + nXDifference;
            cy = (rc.bottom - rc.top) + nYDifference;
            SetWindowPos(
                hProcessPageListCtrl,
                core::ptr::null_mut(),
                0,
                0,
                cx,
                cy,
                (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOZORDER) as UINT,
            );
            InvalidateRect(
                hProcessPageListCtrl,
                core::ptr::null_mut(),
                (TRUE) as BOOL,
            );

            GetClientRect(hProcessPageEndProcessButton, core::ptr::addr_of_mut!(rc));
            MapWindowPoints(
                hProcessPageEndProcessButton,
                hDlg,
                core::ptr::addr_of_mut!(rc) as PRECT as LPPOINT,
                (core::mem::size_of::<RECT>() / (core::mem::size_of::<POINT>() as usize)) as UINT,
            );
            cx = rc.left + nXDifference;
            cy = rc.top + nYDifference;
            SetWindowPos(
                hProcessPageEndProcessButton,
                core::ptr::null_mut(),
                cx,
                cy,
                0,
                0,
                (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
            );
            InvalidateRect(
                hProcessPageEndProcessButton,
                core::ptr::null_mut(),
                (TRUE) as BOOL,
            );

            GetClientRect(hProcessPageShowAllProcessesButton, core::ptr::addr_of_mut!(rc));
            MapWindowPoints(
                hProcessPageShowAllProcessesButton,
                hDlg,
                core::ptr::addr_of_mut!(rc) as PRECT as LPPOINT,
                (core::mem::size_of::<RECT>() / (core::mem::size_of::<POINT>() as usize)) as UINT,
            );
            cx = rc.left;
            cy = rc.top + nYDifference;
            SetWindowPos(
                hProcessPageShowAllProcessesButton,
                core::ptr::null_mut(),
                cx,
                cy,
                0,
                0,
                (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
            );
            InvalidateRect(
                hProcessPageShowAllProcessesButton,
                core::ptr::null_mut(),
                (TRUE) as BOOL,
            );
            // break
        }

        WM_NOTIFY => {
            ProcessPageOnNotify(wParam, lParam);
            // break
        }

        WM_KEYDOWN => {
            if wParam == (VK_DELETE) as _ {
                ProcessPage_OnEndProcess();
            }
            // break
        }

        _ => {}
    }

    return 0 as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPageOnNotify(mut wParam: WPARAM, mut lParam: LPARAM) {
    let mut pnmh: LPNMHDR = core::ptr::null_mut::<tagNMHDR>();
    let mut pnmdi: *mut NMLVDISPINFO = core::ptr::null_mut::<NMLVDISPINFO>();
    let mut pnmhdr: LPNMHEADER = core::ptr::null_mut::<tagNMHEADERW>();
    let mut Index: ULONG = 0;
    let mut ColumnIndex: ULONG = 0;
    let mut pData: LPPROCESS_PAGE_LIST_ITEM = core::ptr::null_mut::<PROCESS_PAGE_LIST_ITEM>();

    pnmh = lParam as LPNMHDR;
    pnmdi = lParam as *mut NMLVDISPINFO;
    pnmhdr = lParam as LPNMHEADER;

    if (*pnmh).hwndFrom == hProcessPageListCtrl {
        match (((*pnmh).code) as i32) as u32 {
            __c if __c == LVN_ITEMCHANGED as u32 => {
                ProcessPageUpdate();
            }
            LVN_GETDISPINFO => {
                if ((*pnmdi).item.mask & LVIF_TEXT) != 0 {
                    pData = (*pnmdi).item.lParam as LPPROCESS_PAGE_LIST_ITEM;
                    Index = PerfDataGetProcessIndex((*pData).ProcessId);
                    ColumnIndex = (*pnmdi).item.iSubItem as ULONG;

                    PerfDataGetText(
                        Index,
                        ColumnIndex,
                        (*pnmdi).item.pszText as LPTSTR,
                        (*pnmdi).item.cchTextMax as ULONG,
                    );
                }
            }
            NM_RCLICK => {
                ProcessPageShowContextMenu(GetSelectedProcessId());
            }
            __c if __c == LVN_KEYDOWN as u32 => {
                if ((*(lParam as LPNMLVKEYDOWN)).wVKey as ::core::ffi::c_int) as u32 == VK_DELETE {
                    ProcessPage_OnEndProcess();
                }
            }
            _ => {}
        }
    } else if (*pnmh).hwndFrom == hProcessPageHeaderCtrl {
        match (((*pnmh).code) as i32) as u32 {
            HDN_ITEMCLICK => {
                TaskManagerSettings.SortColumn =
                    ColumnDataHints[(*pnmhdr).iItem as usize] as ::core::ffi::c_int;
                TaskManagerSettings.SortAscending =
                    (TaskManagerSettings.SortAscending == 0) as ::core::ffi::c_int as BOOL;
                ListView_SortItems!(
                    hProcessPageListCtrl,
                    Some(ProcessPageCompareFunc),
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }
            HDN_ITEMCHANGED => {
                UpdateColumnDataHints();
            }
            __c if __c == HDN_ENDDRAG as u32 => {
                UpdateColumnDataHints();
            }
            _ => {}
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SH_FormatInteger(
    Num: LONGLONG,
    pwszResult: LPWSTR,
    cchResultMax: UINT,
) -> UINT {
    let mut nf: NUMBERFMTW = core::mem::zeroed();
    let mut i: INT;
    let mut cchGrouping: INT;
    let mut cchResult: INT;
    let mut wszNumber: [WCHAR; 24] = [0; 24];
    let mut wszDecimalSep: [WCHAR; 8] = [0; 8];
    let mut wszThousandSep: [WCHAR; 8] = [0; 8];
    let mut wszGrouping: [WCHAR; 12] = [0; 12];

    /* Print the number in uniform mode */
    StringCchPrintfW!(
        wszNumber.as_mut_ptr(),
        _countof!(wszNumber) as INT,
        RUSTCONV_WSTR_EAFE2DCE91A2.as_ptr(),
        Num,
    );

    /* Get system strings for decimal and thousand separators */
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_SDECIMAL,
        wszDecimalSep.as_mut_ptr(),
        _countof!(wszDecimalSep) as INT,
    );
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_STHOUSAND,
        wszThousandSep.as_mut_ptr(),
        _countof!(wszThousandSep) as INT,
    );

    /* Initialize format for printing the number in bytes */
    ZeroMemory!(
        core::ptr::addr_of_mut!(nf) as *mut core::ffi::c_void,
        core::mem::size_of::<NUMBERFMTW>(),
    );
    nf.lpDecimalSep = wszDecimalSep.as_mut_ptr();
    nf.lpThousandSep = wszThousandSep.as_mut_ptr();

    /* Get system string for groups separator */
    cchGrouping = GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_SGROUPING,
        wszGrouping.as_mut_ptr(),
        _countof!(wszGrouping) as INT,
    );

    /* Convert grouping specs from string to integer */
    i = 0;
    while i < cchGrouping {
        let wch: WCHAR = wszGrouping[i as usize];

        if wch >= 0x0030u16 && wch <= 0x0039u16 {
            nf.Grouping = nf
                .Grouping
                .wrapping_mul(10)
                .wrapping_add((wch - 0x0030u16) as UINT);
        } else if wch != 0x003Bu16 {
            break;
        }
        i += 1;
    }

    if (nf.Grouping % 10) == 0 {
        nf.Grouping /= 10;
    } else {
        nf.Grouping = nf.Grouping.wrapping_mul(10);
    }

    /* Format the number */
    cchResult = GetNumberFormatW(
        LOCALE_USER_DEFAULT,
        0,
        wszNumber.as_mut_ptr(),
        core::ptr::addr_of_mut!(nf),
        pwszResult,
        cchResultMax as INT,
    );
    if cchResult == 0 {
        return 0;
    }

    /* GetNumberFormatW returns number of characters including UNICODE_NULL */
    return (cchResult - 1) as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPageShowContextMenu(mut dwProcessId: DWORD) {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hSubMenu: HMENU = core::ptr::null_mut();
    let mut hPriorityMenu: HMENU = core::ptr::null_mut();
    let mut pt: POINT = core::mem::zeroed();
    let mut si: SYSTEM_INFO = core::mem::zeroed();
    let mut hProcess: HANDLE = core::ptr::null_mut();
    let mut dwProcessPriorityClass: DWORD = 0;
    let mut strDebugger: [WCHAR; 260] = [0 as WCHAR; 260];
    let mut dwDebuggerSize: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    if dwProcessId == 0 {
        return;
    }

    memset(
        core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<SYSTEM_INFO>() as size_t,
    );

    GetCursorPos(core::ptr::addr_of_mut!(pt));
    GetSystemInfo(core::ptr::addr_of_mut!(si));

    hMenu = LoadMenuW(hInst, MAKEINTRESOURCEW!(IDR_PROCESS_PAGE_CONTEXT));
    hSubMenu = GetSubMenu(hMenu, 0);
    hPriorityMenu = GetSubMenu(hSubMenu, 4);

    hProcess = OpenProcess(PROCESS_QUERY_INFORMATION, (FALSE) as BOOL, dwProcessId);
    dwProcessPriorityClass = GetPriorityClass(hProcess);
    CloseHandle(hProcess);

    if si.dwNumberOfProcessors < 2 {
        RemoveMenu(hSubMenu, ID_PROCESS_PAGE_SETAFFINITY, MF_BYCOMMAND);
    }

    match dwProcessPriorityClass {
        REALTIME_PRIORITY_CLASS => {
            CheckMenuRadioItem(hPriorityMenu, ID_PROCESS_PAGE_SETPRIORITY_REALTIME, ID_PROCESS_PAGE_SETPRIORITY_LOW, ID_PROCESS_PAGE_SETPRIORITY_REALTIME, MF_BYCOMMAND);
        }
        HIGH_PRIORITY_CLASS => {
            CheckMenuRadioItem(hPriorityMenu, ID_PROCESS_PAGE_SETPRIORITY_REALTIME, ID_PROCESS_PAGE_SETPRIORITY_LOW, ID_PROCESS_PAGE_SETPRIORITY_HIGH, MF_BYCOMMAND);
        }
        ABOVE_NORMAL_PRIORITY_CLASS => {
            CheckMenuRadioItem(hPriorityMenu, ID_PROCESS_PAGE_SETPRIORITY_REALTIME, ID_PROCESS_PAGE_SETPRIORITY_LOW, ID_PROCESS_PAGE_SETPRIORITY_ABOVENORMAL, MF_BYCOMMAND);
        }
        NORMAL_PRIORITY_CLASS => {
            CheckMenuRadioItem(hPriorityMenu, ID_PROCESS_PAGE_SETPRIORITY_REALTIME, ID_PROCESS_PAGE_SETPRIORITY_LOW, ID_PROCESS_PAGE_SETPRIORITY_NORMAL, MF_BYCOMMAND);
        }
        BELOW_NORMAL_PRIORITY_CLASS => {
            CheckMenuRadioItem(hPriorityMenu, ID_PROCESS_PAGE_SETPRIORITY_REALTIME, ID_PROCESS_PAGE_SETPRIORITY_LOW, ID_PROCESS_PAGE_SETPRIORITY_BELOWNORMAL, MF_BYCOMMAND);
        }
        IDLE_PRIORITY_CLASS => {
            CheckMenuRadioItem(hPriorityMenu, ID_PROCESS_PAGE_SETPRIORITY_REALTIME, ID_PROCESS_PAGE_SETPRIORITY_LOW, ID_PROCESS_PAGE_SETPRIORITY_LOW, MF_BYCOMMAND);
        }
        _ => {}
    }

    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, RUSTCONV_WSTR_DEB4EFA22C30.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS
    {
        dwDebuggerSize = core::mem::size_of::<[WCHAR; 260]>() as DWORD;
        if (RegQueryValueExW(hKey, RUSTCONV_WSTR_D7CB07DBF8E2.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), strDebugger.as_mut_ptr() as LPBYTE, core::ptr::addr_of_mut!(dwDebuggerSize))) as u32 == ERROR_SUCCESS
        {
            CharUpper(strDebugger.as_mut_ptr());
            if wcsstr(strDebugger.as_mut_ptr(), RUSTCONV_WSTR_F8FDF9BFDBA4.as_ptr()).is_null() == false {
                EnableMenuItem(hSubMenu, ID_PROCESS_PAGE_DEBUG, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            }
        } else {
            EnableMenuItem(hSubMenu, ID_PROCESS_PAGE_DEBUG, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        }

        RegCloseKey(hKey);
    } else {
        EnableMenuItem(hSubMenu, ID_PROCESS_PAGE_DEBUG, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
    }
    TrackPopupMenu(hSubMenu, TPM_LEFTALIGN | TPM_TOPALIGN | TPM_LEFTBUTTON, pt.x, pt.y, 0, hMainWnd, core::ptr::null_mut());
    DestroyMenu(hMenu);
}
#[no_mangle]
pub unsafe extern "C" fn RefreshProcessPage() {
    PostThreadMessage(dwProcessThread, WM_TIMER as UINT, 0 as WPARAM, 0 as LPARAM);
}
#[no_mangle]
pub unsafe extern "system" fn ProcessPageRefreshThread(mut lpParameter: *mut core::ffi::c_void) -> DWORD {
    let mut msg: MSG = core::mem::zeroed();

    while 1 != 0 {
        /*  Wait for an the event or application close */
        if GetRefreshMessage(core::ptr::addr_of_mut!(msg)) == 0 {
            return 0;
        }

        if msg.message == WM_TIMER as UINT {

            UpdateProcesses();

            if IsWindowVisible(hProcessPage) != 0 {
                InvalidateRect(hProcessPageListCtrl, core::ptr::null::<RECT>(), (FALSE) as BOOL);
            }

            ProcessPageUpdate();
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn UpdateProcesses() {
    let mut i: ::core::ffi::c_int;
    let mut l: ULONG;
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut pData: LPPROCESS_PAGE_LIST_ITEM = core::ptr::null_mut();

    SendMessage(
        hProcessPageListCtrl,
        WM_SETREDRAW as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );

    /* Remove old processes */
    i = 0 as ::core::ffi::c_int;
    while i < ListView_GetItemCount!(hProcessPageListCtrl) {
        memset(
            core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<LV_ITEM>(),
        );
        item.mask = LVIF_PARAM as UINT;
        item.iItem = i;
        ListView_GetItem!(hProcessPageListCtrl, core::ptr::addr_of_mut!(item));
        pData = item.lParam as LPPROCESS_PAGE_LIST_ITEM;
        if ProcessRunning((*pData).ProcessId) == 0 {
            ListView_DeleteItem!(hProcessPageListCtrl, i);
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pData as LPVOID,
            );
        }
        i += 1;
    }

    /* Check for difference in listview process and performance process counts */
    if ListView_GetItemCount!(hProcessPageListCtrl) as ULONG != PerfDataGetProcessCount() {
        /* Add new processes by checking against the current items */
        l = 0 as ULONG;
        while l < PerfDataGetProcessCount() {
            AddProcess(l);
            l = l.wrapping_add(1);
        }
    }

    if TaskManagerSettings.SortColumn != -(1 as ::core::ffi::c_int) {
        ListView_SortItems!(
            hProcessPageListCtrl,
            Some(ProcessPageCompareFunc),
            core::ptr::null_mut::<::core::ffi::c_void>() as LPARAM,
        );
    }

    SendMessage(
        hProcessPageListCtrl,
        WM_SETREDRAW as UINT,
        TRUE as WPARAM,
        0 as LPARAM,
    );

    /* Select first item if any */
    if (ListView_GetNextItem!(
        hProcessPageListCtrl,
        -(1 as ::core::ffi::c_int),
        (LVNI_FOCUSED | LVNI_SELECTED) as UINT,
    ) == -(1 as ::core::ffi::c_int))
        && (ListView_GetItemCount!(hProcessPageListCtrl) > 0 as ::core::ffi::c_int)
        && bProcessPageSelectionMade == 0
    {
        ListView_SetItemState!(
            hProcessPageListCtrl,
            0 as ::core::ffi::c_int,
            (LVIS_FOCUSED | LVIS_SELECTED) as UINT,
            (LVIS_FOCUSED | LVIS_SELECTED) as UINT,
        );
        bProcessPageSelectionMade = TRUE as BOOL;
    }
    /*
    else
    {
        bProcessPageSelectionMade = FALSE;
    }
    */
}
#[no_mangle]
pub unsafe extern "C" fn ProcessRunning(ProcessId: ULONG) -> BOOL {
    let mut hProcess: HANDLE;
    let mut exitCode: DWORD = core::mem::zeroed();

    if ProcessId == 0 {
        return (TRUE) as BOOL;
    }

    hProcess = OpenProcess(PROCESS_ALL_ACCESS, (FALSE) as BOOL, ProcessId);
    if hProcess.is_null() {
        return (FALSE) as BOOL;
    }

    if GetExitCodeProcess(hProcess, core::ptr::addr_of_mut!(exitCode)) != 0 {
        CloseHandle(hProcess);
        return (exitCode == STILL_ACTIVE) as BOOL;
    }

    CloseHandle(hProcess);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AddProcess(Index: ULONG) {
    let mut pData: LPPROCESS_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut i: ::core::ffi::c_int;
    let mut item: LVITEMW = core::mem::zeroed();
    let mut bAlreadyInList: BOOL = (FALSE) as i32;
    let mut pid: ULONG;

    pid = PerfDataGetProcessId(Index);

    /* Check to see if it's already in our list */
    i = 0;
    while i < ListView_GetItemCount!(hProcessPageListCtrl) {
        memset(core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, ::core::mem::size_of::<LV_ITEM>() as size_t);
        item.mask = LVIF_PARAM;
        item.iItem = i;
        let _ = ListView_GetItem!(hProcessPageListCtrl, core::ptr::addr_of_mut!(item));
        pData = item.lParam as LPPROCESS_PAGE_LIST_ITEM;
        if (*pData).ProcessId == pid {
            bAlreadyInList = (TRUE) as i32;
            break;
        }
        i += 1;
    }
    if bAlreadyInList == 0 {
        /* Add */
        pData = HeapAlloc(GetProcessHeap(), 0 as DWORD, ::core::mem::size_of::<PROCESS_PAGE_LIST_ITEM>() as SIZE_T) as LPPROCESS_PAGE_LIST_ITEM;
        (*pData).ProcessId = pid;

        /* Add the item to the list */
        memset(core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, ::core::mem::size_of::<LV_ITEM>() as size_t);
        item.mask = LVIF_TEXT | LVIF_PARAM;
        item.pszText = LPSTR_TEXTCALLBACK;
        item.iItem = ListView_GetItemCount!(hProcessPageListCtrl);
        item.lParam = pData as LPARAM;
        let _ = ListView_InsertItem!(hProcessPageListCtrl, core::ptr::addr_of_mut!(item));
    }
}
#[no_mangle]
pub unsafe extern "C" fn PerfDataGetText(Index: ULONG, ColumnIndex: ULONG, lpText: LPTSTR, nMaxCount: ULONG) -> BOOL {
    let mut iocounters: IO_COUNTERS = core::mem::zeroed();

    match ColumnDataHints[ColumnIndex as usize] {
        COLUMN_IMAGENAME => {
            PerfDataGetImageName(Index, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_PID => {
            StringCchPrintfW!(lpText, nMaxCount, RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), PerfDataGetProcessId(Index));
            return (TRUE) as BOOL;
        }

        COLUMN_USERNAME => {
            PerfDataGetUserName(Index, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_COMMANDLINE => {
            PerfDataGetCommandLine(Index, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_SESSIONID => {
            StringCchPrintfW!(lpText, nMaxCount, RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), PerfDataGetSessionId(Index));
            return (TRUE) as BOOL;
        }

        COLUMN_CPUUSAGE => {
            StringCchPrintfW!(lpText, nMaxCount, RUSTCONV_WSTR_9D0AE7D0581B.as_ptr(), PerfDataGetCPUUsage(Index));
            return (TRUE) as BOOL;
        }

        COLUMN_CPUTIME => {
            let mut time: LARGE_INTEGER = core::mem::zeroed();
            let mut dwHours: DWORD = 0;
            let mut dwMinutes: DWORD = 0;
            let mut dwSeconds: DWORD = 0;

            time = PerfDataGetCPUTime(Index);
            gethmsfromlargeint(
                time,
                core::ptr::addr_of_mut!(dwHours),
                core::ptr::addr_of_mut!(dwMinutes),
                core::ptr::addr_of_mut!(dwSeconds),
            );
            StringCchPrintfW!(lpText, nMaxCount, RUSTCONV_WSTR_BA75F9F8779D.as_ptr(), dwHours, dwMinutes, dwSeconds);
            return (TRUE) as BOOL;
        }

        COLUMN_MEMORYUSAGE => {
            SH_FormatInteger((PerfDataGetWorkingSetSizeBytes(Index) / 1024) as LONGLONG, lpText, nMaxCount);
            StringCchCatW(lpText, (nMaxCount) as _, RUSTCONV_WSTR_84FA0582B45C.as_ptr());
            return (TRUE) as BOOL;
        }

        COLUMN_PEAKMEMORYUSAGE => {
            SH_FormatInteger((PerfDataGetPeakWorkingSetSizeBytes(Index) / 1024) as LONGLONG, lpText, nMaxCount);
            StringCchCatW(lpText, (nMaxCount) as _, RUSTCONV_WSTR_84FA0582B45C.as_ptr());
            return (TRUE) as BOOL;
        }

        COLUMN_MEMORYUSAGEDELTA => {
            SH_FormatInteger((PerfDataGetWorkingSetSizeDelta(Index) / 1024) as LONGLONG, lpText, nMaxCount);
            StringCchCatW(lpText, (nMaxCount) as _, RUSTCONV_WSTR_84FA0582B45C.as_ptr());
            return (TRUE) as BOOL;
        }

        COLUMN_PAGEFAULTS => {
            SH_FormatInteger((PerfDataGetPageFaultCount(Index)) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_PAGEFAULTSDELTA => {
            SH_FormatInteger((PerfDataGetPageFaultCountDelta(Index)) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_VIRTUALMEMORYSIZE => {
            SH_FormatInteger((PerfDataGetVirtualMemorySizeBytes(Index) / 1024) as LONGLONG, lpText, nMaxCount);
            StringCchCatW(lpText, (nMaxCount) as _, RUSTCONV_WSTR_84FA0582B45C.as_ptr());
            return (TRUE) as BOOL;
        }

        COLUMN_PAGEDPOOL => {
            SH_FormatInteger((PerfDataGetPagedPoolUsagePages(Index) / 1024) as LONGLONG, lpText, nMaxCount);
            StringCchCatW(lpText, (nMaxCount) as _, RUSTCONV_WSTR_84FA0582B45C.as_ptr());
            return (TRUE) as BOOL;
        }

        COLUMN_NONPAGEDPOOL => {
            SH_FormatInteger((PerfDataGetNonPagedPoolUsagePages(Index) / 1024) as LONGLONG, lpText, nMaxCount);
            StringCchCatW(lpText, (nMaxCount) as _, RUSTCONV_WSTR_84FA0582B45C.as_ptr());
            return (TRUE) as BOOL;
        }

        COLUMN_BASEPRIORITY => {
            StringCchPrintfW!(lpText, nMaxCount, RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), PerfDataGetBasePriority(Index));
            return (TRUE) as BOOL;
        }

        COLUMN_HANDLECOUNT => {
            SH_FormatInteger((PerfDataGetHandleCount(Index)) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_THREADCOUNT => {
            SH_FormatInteger((PerfDataGetThreadCount(Index)) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_USEROBJECTS => {
            SH_FormatInteger((PerfDataGetUSERObjectCount(Index)) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_GDIOBJECTS => {
            SH_FormatInteger((PerfDataGetGDIObjectCount(Index)) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_IOREADS => {
            PerfDataGetIOCounters(Index, core::ptr::addr_of_mut!(iocounters));
            SH_FormatInteger((iocounters.ReadOperationCount) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_IOWRITES => {
            PerfDataGetIOCounters(Index, core::ptr::addr_of_mut!(iocounters));
            SH_FormatInteger((iocounters.WriteOperationCount) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_IOOTHER => {
            PerfDataGetIOCounters(Index, core::ptr::addr_of_mut!(iocounters));
            SH_FormatInteger((iocounters.OtherOperationCount) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_IOREADBYTES => {
            PerfDataGetIOCounters(Index, core::ptr::addr_of_mut!(iocounters));
            SH_FormatInteger((iocounters.ReadTransferCount) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_IOWRITEBYTES => {
            PerfDataGetIOCounters(Index, core::ptr::addr_of_mut!(iocounters));
            SH_FormatInteger((iocounters.WriteTransferCount) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        COLUMN_IOOTHERBYTES => {
            PerfDataGetIOCounters(Index, core::ptr::addr_of_mut!(iocounters));
            SH_FormatInteger((iocounters.OtherTransferCount) as LONGLONG, lpText, nMaxCount);
            return (TRUE) as BOOL;
        }

        _ => {}
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn gethmsfromlargeint(
    largeint: LARGE_INTEGER,
    dwHours: *mut DWORD,
    dwMinutes: *mut DWORD,
    dwSeconds: *mut DWORD,
) {
    if cfg!(target_env = "msvc") {
        *dwHours = (largeint.QuadPart / 36000000000) as DWORD;
        *dwMinutes = ((largeint.QuadPart % 36000000000) / 600000000) as DWORD;
        *dwSeconds = (((largeint.QuadPart % 36000000000) % 600000000) / 10000000) as DWORD;
    } else {
        *dwHours = (largeint.QuadPart / 36000000000) as DWORD;
        *dwMinutes = ((largeint.QuadPart % 36000000000) / 600000000) as DWORD;
        *dwSeconds = (((largeint.QuadPart % 36000000000) % 600000000) / 10000000) as DWORD;
    }
}
#[no_mangle]
pub unsafe extern "C" fn largeintcmp(l1: LARGE_INTEGER, l2: LARGE_INTEGER) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0;
    let mut dwHours1: DWORD;
    let mut dwMinutes1: DWORD;
    let mut dwSeconds1: DWORD;
    let mut dwHours2: DWORD;
    let mut dwMinutes2: DWORD;
    let mut dwSeconds2: DWORD;
    dwHours1 = 0;
    dwMinutes1 = 0;
    dwSeconds1 = 0;
    dwHours2 = 0;
    dwMinutes2 = 0;
    dwSeconds2 = 0;

    gethmsfromlargeint(
        l1,
        core::ptr::addr_of_mut!(dwHours1),
        core::ptr::addr_of_mut!(dwMinutes1),
        core::ptr::addr_of_mut!(dwSeconds1),
    );
    gethmsfromlargeint(
        l2,
        core::ptr::addr_of_mut!(dwHours2),
        core::ptr::addr_of_mut!(dwMinutes2),
        core::ptr::addr_of_mut!(dwSeconds2),
    );
    ret = CMP!(dwHours1, dwHours2);
    if ret == 0 {
        ret = CMP!(dwMinutes1, dwMinutes2);
        if ret == 0 {
            ret = CMP!(dwSeconds1, dwSeconds2);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn ProcessPageCompareFunc(lParam1: LPARAM, lParam2: LPARAM, lParamSort: LPARAM) -> c_int {
    let mut ret: c_int = 0;
    let mut Param1: LPPROCESS_PAGE_LIST_ITEM;
    let mut Param2: LPPROCESS_PAGE_LIST_ITEM;
    let mut IndexParam1: ULONG;
    let mut IndexParam2: ULONG;
    let mut text1: [WCHAR; 260] = [0; 260];
    let mut text2: [WCHAR; 260] = [0; 260];
    let mut l1: ULONG = 0;
    let mut l2: ULONG = 0;
    let mut time1: LARGE_INTEGER = core::mem::zeroed();
    let mut time2: LARGE_INTEGER = core::mem::zeroed();
    let mut iocounters1: IO_COUNTERS = core::mem::zeroed();
    let mut iocounters2: IO_COUNTERS = core::mem::zeroed();
    let mut ull1: ULONGLONG = 0;
    let mut ull2: ULONGLONG = 0;

    if TaskManagerSettings.SortAscending != 0 {
        Param1 = lParam1 as LPPROCESS_PAGE_LIST_ITEM;
        Param2 = lParam2 as LPPROCESS_PAGE_LIST_ITEM;
    } else {
        Param1 = lParam2 as LPPROCESS_PAGE_LIST_ITEM;
        Param2 = lParam1 as LPPROCESS_PAGE_LIST_ITEM;
    }
    IndexParam1 = PerfDataGetProcessIndex((*Param1).ProcessId);
    IndexParam2 = PerfDataGetProcessIndex((*Param2).ProcessId);

    if TaskManagerSettings.SortColumn == (COLUMN_IMAGENAME as i32)
    {
        PerfDataGetImageName(IndexParam1, core::ptr::addr_of_mut!(text1) as LPWSTR, (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ULONG);
        PerfDataGetImageName(IndexParam2, core::ptr::addr_of_mut!(text2) as LPWSTR, (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ULONG);
        ret = _wcsicmp(core::ptr::addr_of_mut!(text1) as *mut WCHAR, core::ptr::addr_of_mut!(text2) as *mut WCHAR);
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_PID as i32)
    {
        l1 = (*Param1).ProcessId;
        l2 = (*Param2).ProcessId;
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_USERNAME as i32)
    {
        PerfDataGetUserName(IndexParam1, core::ptr::addr_of_mut!(text1) as LPWSTR, (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ULONG);
        PerfDataGetUserName(IndexParam2, core::ptr::addr_of_mut!(text2) as LPWSTR, (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ULONG);
        ret = _wcsicmp(core::ptr::addr_of_mut!(text1) as *mut WCHAR, core::ptr::addr_of_mut!(text2) as *mut WCHAR);
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_COMMANDLINE as i32)
    {
        PerfDataGetCommandLine(IndexParam1, core::ptr::addr_of_mut!(text1) as LPWSTR, (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ULONG);
        PerfDataGetCommandLine(IndexParam2, core::ptr::addr_of_mut!(text2) as LPWSTR, (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as ULONG);
        ret = _wcsicmp(core::ptr::addr_of_mut!(text1) as *mut WCHAR, core::ptr::addr_of_mut!(text2) as *mut WCHAR);
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_SESSIONID as i32)
    {
        l1 = PerfDataGetSessionId(IndexParam1);
        l2 = PerfDataGetSessionId(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_CPUUSAGE as i32)
    {
        l1 = PerfDataGetCPUUsage(IndexParam1);
        l2 = PerfDataGetCPUUsage(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_CPUTIME as i32)
    {
        time1 = PerfDataGetCPUTime(IndexParam1);
        time2 = PerfDataGetCPUTime(IndexParam2);
        ret = largeintcmp(time1, time2);
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_MEMORYUSAGE as i32)
    {
        l1 = PerfDataGetWorkingSetSizeBytes(IndexParam1);
        l2 = PerfDataGetWorkingSetSizeBytes(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_PEAKMEMORYUSAGE as i32)
    {
        l1 = PerfDataGetPeakWorkingSetSizeBytes(IndexParam1);
        l2 = PerfDataGetPeakWorkingSetSizeBytes(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_MEMORYUSAGEDELTA as i32)
    {
        l1 = PerfDataGetWorkingSetSizeDelta(IndexParam1);
        l2 = PerfDataGetWorkingSetSizeDelta(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_PAGEFAULTS as i32)
    {
        l1 = PerfDataGetPageFaultCount(IndexParam1);
        l2 = PerfDataGetPageFaultCount(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_PAGEFAULTSDELTA as i32)
    {
        l1 = PerfDataGetPageFaultCountDelta(IndexParam1);
        l2 = PerfDataGetPageFaultCountDelta(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_VIRTUALMEMORYSIZE as i32)
    {
        l1 = PerfDataGetVirtualMemorySizeBytes(IndexParam1);
        l2 = PerfDataGetVirtualMemorySizeBytes(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_PAGEDPOOL as i32)
    {
        l1 = PerfDataGetPagedPoolUsagePages(IndexParam1);
        l2 = PerfDataGetPagedPoolUsagePages(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_NONPAGEDPOOL as i32)
    {
        l1 = PerfDataGetNonPagedPoolUsagePages(IndexParam1);
        l2 = PerfDataGetNonPagedPoolUsagePages(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_BASEPRIORITY as i32)
    {
        l1 = PerfDataGetBasePriority(IndexParam1);
        l2 = PerfDataGetBasePriority(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_HANDLECOUNT as i32)
    {
        l1 = PerfDataGetHandleCount(IndexParam1);
        l2 = PerfDataGetHandleCount(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_THREADCOUNT as i32)
    {
        l1 = PerfDataGetThreadCount(IndexParam1);
        l2 = PerfDataGetThreadCount(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_USEROBJECTS as i32)
    {
        l1 = PerfDataGetUSERObjectCount(IndexParam1);
        l2 = PerfDataGetUSERObjectCount(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_GDIOBJECTS as i32)
    {
        l1 = PerfDataGetGDIObjectCount(IndexParam1);
        l2 = PerfDataGetGDIObjectCount(IndexParam2);
        ret = if l1 < l2 { -1 } else if l1 > l2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_IOREADS as i32)
    {
        PerfDataGetIOCounters(IndexParam1, core::ptr::addr_of_mut!(iocounters1));
        PerfDataGetIOCounters(IndexParam2, core::ptr::addr_of_mut!(iocounters2));
        ull1 = iocounters1.ReadOperationCount;
        ull2 = iocounters2.ReadOperationCount;
        ret = if ull1 < ull2 { -1 } else if ull1 > ull2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_IOWRITES as i32)
    {
        PerfDataGetIOCounters(IndexParam1, core::ptr::addr_of_mut!(iocounters1));
        PerfDataGetIOCounters(IndexParam2, core::ptr::addr_of_mut!(iocounters2));
        ull1 = iocounters1.WriteOperationCount;
        ull2 = iocounters2.WriteOperationCount;
        ret = if ull1 < ull2 { -1 } else if ull1 > ull2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_IOOTHER as i32)
    {
        PerfDataGetIOCounters(IndexParam1, core::ptr::addr_of_mut!(iocounters1));
        PerfDataGetIOCounters(IndexParam2, core::ptr::addr_of_mut!(iocounters2));
        ull1 = iocounters1.OtherOperationCount;
        ull2 = iocounters2.OtherOperationCount;
        ret = if ull1 < ull2 { -1 } else if ull1 > ull2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_IOREADBYTES as i32)
    {
        PerfDataGetIOCounters(IndexParam1, core::ptr::addr_of_mut!(iocounters1));
        PerfDataGetIOCounters(IndexParam2, core::ptr::addr_of_mut!(iocounters2));
        ull1 = iocounters1.ReadTransferCount;
        ull2 = iocounters2.ReadTransferCount;
        ret = if ull1 < ull2 { -1 } else if ull1 > ull2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_IOWRITEBYTES as i32)
    {
        PerfDataGetIOCounters(IndexParam1, core::ptr::addr_of_mut!(iocounters1));
        PerfDataGetIOCounters(IndexParam2, core::ptr::addr_of_mut!(iocounters2));
        ull1 = iocounters1.WriteTransferCount;
        ull2 = iocounters2.WriteTransferCount;
        ret = if ull1 < ull2 { -1 } else if ull1 > ull2 { 1 } else { 0 };
    }
    else if TaskManagerSettings.SortColumn == (COLUMN_IOOTHERBYTES as i32)
    {
        PerfDataGetIOCounters(IndexParam1, core::ptr::addr_of_mut!(iocounters1));
        PerfDataGetIOCounters(IndexParam2, core::ptr::addr_of_mut!(iocounters2));
        ull1 = iocounters1.OtherTransferCount;
        ull2 = iocounters2.OtherTransferCount;
        ret = if ull1 < ull2 { -1 } else if ull1 > ull2 { 1 } else { 0 };
    }
    return ret;
}

unsafe extern "C" fn DevicePathToDosPath(
    lpDevicePath: LPCWSTR,
    lpDosPath: LPWSTR,
    dwLength: DWORD,
) -> DWORD {
    let mut dwRet: DWORD = 0;
    let mut szDrive: [WCHAR; 3] = ['?' as WCHAR, ':' as WCHAR, 0];
    let mut szDeviceName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Check if lpDevicePath is a device path */
    if _wcsnicmp(lpDevicePath, RUSTCONV_WSTR_958F461AF768.as_ptr(), CONST_STR_LEN!(RUSTCONV_WSTR_958F461AF768) as usize) != 0
    {
        return 0;
    }

    szDrive[0] = 0x0041u16;
    while szDrive[0] <= 0x0060u16
    {
        if QueryDosDeviceW(
            core::ptr::addr_of_mut!(szDrive) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(szDeviceName) as *mut WCHAR as LPWSTR,
            _countof!(szDeviceName) as DWORD,
        ) != 0
        {
            let mut len: size_t = wcslen(core::ptr::addr_of_mut!(szDeviceName) as *mut WCHAR) as size_t;

            if _wcsnicmp(lpDevicePath, core::ptr::addr_of_mut!(szDeviceName) as *mut WCHAR, len) == 0
            {
                /* Get the required length, including the NULL terminator */
                dwRet = (_countof!(szDrive) as DWORD).wrapping_add(
                    wcslen(lpDevicePath.add(len as usize)) as DWORD,
                );

                if !lpDosPath.is_null() && (dwLength >= dwRet)
                {
                    StringCchPrintfW!(
                        lpDosPath,
                        dwLength,
                        RUSTCONV_WSTR_E2CD87D16D16.as_ptr(),
                        core::ptr::addr_of_mut!(szDrive) as *mut WCHAR,
                        lpDevicePath.add(len as usize),
                    );
                }

                break;
            }
        }
        szDrive[0] = szDrive[0].wrapping_add(1);
    }

    return dwRet;
}

unsafe extern "C" fn GetProcessExecutablePath(
    hProcess: HANDLE,
    lpExePath: LPWSTR,
    dwLength: DWORD,
) -> DWORD {
    let mut dwRet: DWORD = 0;
    let mut Status: NTSTATUS;
    let mut StaticBuffer: [BYTE; 536] = [0; 536];
    let mut DynamicBuffer: PVOID = core::ptr::null_mut();
    let mut ExePath: PUNICODE_STRING = core::ptr::null_mut();
    let mut SizeNeeded: ULONG = 0;

    Status = NtQueryInformationProcess(
        hProcess,
        ProcessImageFileName,
        StaticBuffer.as_mut_ptr() as PVOID,
        /* Reserve a NULL terminator */
        (core::mem::size_of::<[BYTE; 536]>() - core::mem::size_of::<WCHAR>()) as ULONG,
        core::ptr::addr_of_mut!(SizeNeeded),
    );
    'cleanup: {
        if NT_SUCCESS!(Status) {
            ExePath = StaticBuffer.as_mut_ptr() as PUNICODE_STRING;
        } else if Status == STATUS_INFO_LENGTH_MISMATCH {
            /* Allocate the buffer, reserving space for a NULL terminator */
            DynamicBuffer = HeapAlloc(
                GetProcessHeap(),
                0,
                (SizeNeeded as SIZE_T).wrapping_add(core::mem::size_of::<WCHAR>() as SIZE_T),
            );
            if DynamicBuffer.is_null() {
                return 0;
            }

            Status = NtQueryInformationProcess(
                hProcess,
                ProcessImageFileName,
                DynamicBuffer,
                SizeNeeded,
                core::ptr::addr_of_mut!(SizeNeeded),
            );
            if !NT_SUCCESS!(Status) {
                break 'cleanup;
            }

            ExePath = DynamicBuffer as PUNICODE_STRING;
        } else {
            return 0;
        }

        /* Manually NULL-terminate */
        *(*ExePath).Buffer.offset(
            ((*ExePath).Length as usize / core::mem::size_of::<WCHAR>()) as isize,
        ) = UNICODE_NULL;

        /* HACK: Convert device path format into Win32 path format.
         * Use ProcessImageFileNameWin32 instead if the kernel supports it. */
        dwRet = DevicePathToDosPath((*ExePath).Buffer as LPCWSTR, lpExePath, dwLength);
    }

    HeapFree(GetProcessHeap(), 0, DynamicBuffer);

    return dwRet;
}

unsafe extern "C" fn GetProcessExecutablePathById(
    dwProcessId: DWORD,
    lpExePath: LPWSTR,
    dwLength: DWORD,
) -> DWORD {
    let mut dwRet: DWORD = 0;

    if dwProcessId == 0 {
        return 0;
    }

    /* PID = 4 ("System") */
    if dwProcessId == 4 {
        static szKernelExe: [WCHAR; 14] = [
    0x005Cu16, 0x006E, 0x0074, 0x006F, 0x0073, 0x006B, 0x0072, 0x006E, 0x006C, 0x002E,
    0x0065, 0x0078, 0x0065, 0x0000,
];
        let mut pszSystemDir: LPWSTR;
        let mut uLength: UINT;

        uLength = GetSystemDirectoryW(core::ptr::null_mut(), 0);
        if uLength == 0 {
            return 0;
        }

        pszSystemDir = HeapAlloc(
            GetProcessHeap(),
            0,
            (uLength as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if pszSystemDir.is_null() {
            return 0;
        }

        if GetSystemDirectoryW(pszSystemDir, uLength) != 0 {
            /* Get the required length, including the NULL terminator */
            dwRet = uLength + CONST_STR_LEN!(szKernelExe);

            if !lpExePath.is_null() && (dwLength >= dwRet) {
                StringCchPrintfW!(
                    lpExePath,
                    dwLength,
                    RUSTCONV_WSTR_E2CD87D16D16.as_ptr(),
                    pszSystemDir,
                    core::ptr::addr_of!(szKernelExe) as *const WCHAR,
                );
            }
        }

        HeapFree(GetProcessHeap(), 0, pszSystemDir as PVOID);
    } else {
        let mut hProcess: HANDLE;

        hProcess = OpenProcess(PROCESS_QUERY_INFORMATION, (FALSE) as BOOL, dwProcessId);
        if !hProcess.is_null() {
            dwRet = GetProcessExecutablePath(hProcess, lpExePath, dwLength);
            CloseHandle(hProcess);
        }
    }

    return dwRet;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPage_OnProperties() {
    let mut dwProcessId: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut pszExePath: LPWSTR = core::ptr::null_mut();
    let mut info: SHELLEXECUTEINFOW = core::mem::zeroed();

    'cleanup: {
        dwProcessId = GetSelectedProcessId();

        /* Retrieve the image path length */
        dwLength = GetProcessExecutablePathById(dwProcessId, core::ptr::null_mut(), 0);
        if dwLength == 0 {
            return;
        }

        /* Allocate and retrieve the image path */
        pszExePath = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwLength * (core::mem::size_of::<WCHAR>() as DWORD)) as SIZE_T,
        ) as LPWSTR;
        if pszExePath.is_null() {
            return;
        }

        if GetProcessExecutablePathById(dwProcessId, pszExePath, dwLength) == 0 {
            break 'cleanup;
        }

        /* Call the shell to display the file properties */
        info.cbSize = (core::mem::size_of::<SHELLEXECUTEINFOW>()) as _;
        info.fMask = SEE_MASK_INVOKEIDLIST;
        info.hwnd = core::ptr::null_mut();
        info.lpVerb = RUSTCONV_WSTR_9B349295E8D5.as_ptr();
        info.lpFile = pszExePath as LPCWSTR;
        info.lpParameters = RUSTCONV_WSTR_96A296D224F2.as_ptr();
        info.lpDirectory = core::ptr::null();
        info.nShow = (SW_SHOW) as i32;
        info.hInstApp = core::ptr::null_mut();

        ShellExecuteExW(&raw mut info);
    }

    HeapFree(GetProcessHeap(), 0, pszExePath as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPage_OnOpenFileLocation() {
    let mut dwProcessId: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut pszExePath: LPWSTR = core::ptr::null_mut();
    static szCmdFormat: [u16; 13] = [
    0x002Fu16, 0x0073, 0x0065, 0x006C, 0x0065, 0x0063, 0x0074, 0x002C, 0x0022, 0x0025,
    0x0073, 0x0022, 0x0000,
];
    let mut pszCmdLine: LPWSTR = core::ptr::null_mut();

    'cleanup: {
        dwProcessId = GetSelectedProcessId();

        /* Retrieve the image path length */
        dwLength = GetProcessExecutablePathById(dwProcessId, core::ptr::null_mut(), 0);
        if dwLength == 0 {
            return;
        }

        /* Allocate and retrieve the image path */
        pszExePath = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if pszExePath.is_null() {
            return;
        }

        if GetProcessExecutablePathById(dwProcessId, pszExePath, dwLength) == 0 {
            break 'cleanup;
        }

        /* Build the shell command line */
        dwLength = dwLength.wrapping_add(
            (CONST_STR_LEN!(szCmdFormat) - CONST_STR_LEN!(RUSTCONV_WSTR_FB47CCC40FE6)) as DWORD,
        );
        pszCmdLine = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if pszCmdLine.is_null() {
            break 'cleanup;
        }

        StringCchPrintfW!(
            pszCmdLine,
            dwLength,
            core::ptr::addr_of!(szCmdFormat) as *const WCHAR,
            pszExePath,
        );

        /* Call the shell to open the file location and select it. If Explorer shell
         * is not available, use ReactOS's alternative file browser instead. */
        ShellExecuteW(
            core::ptr::null_mut(),
            RUSTCONV_WSTR_82DB442E7A60.as_ptr(),
            if GetShellWindow().is_null() {
                RUSTCONV_WSTR_5968F39826A0.as_ptr()
            } else {
                RUSTCONV_WSTR_AB68B18B313D.as_ptr()
            },
            pszCmdLine,
            core::ptr::null_mut(),
            (SW_SHOWNORMAL) as INT,
        );
    }

    /* Cleanup: */
    HeapFree(GetProcessHeap(), 0, pszCmdLine as PVOID);
    HeapFree(GetProcessHeap(), 0, pszExePath as PVOID);
}

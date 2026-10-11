use crate::taskmgr::applpage::ApplicationPageWndProc;
use crate::taskmgr::applpage::ApplicationPage_OnEndTask;
use crate::taskmgr::applpage::ApplicationPage_OnGotoProcess;
use crate::taskmgr::applpage::ApplicationPage_OnSwitchTo;
use crate::taskmgr::applpage::ApplicationPage_OnView;
use crate::taskmgr::applpage::ApplicationPage_OnWindowsBringToFront;
use crate::taskmgr::applpage::ApplicationPage_OnWindowsCascade;
use crate::taskmgr::applpage::ApplicationPage_OnWindowsMaximize;
use crate::taskmgr::applpage::ApplicationPage_OnWindowsMinimize;
use crate::taskmgr::applpage::ApplicationPage_OnWindowsTile;
use crate::taskmgr::priority::DoSetPriority;
use crate::taskmgr::about::OnAbout;
use crate::taskmgr::perfdata::PerfDataDeallocCommandLineCache;
use crate::taskmgr::perfdata::PerfDataInitialize;
use crate::taskmgr::perfdata::PerfDataRefresh;
use crate::taskmgr::perfdata::PerfDataUninitialize;
use crate::taskmgr::perfpage::PerformancePageWndProc;
use crate::taskmgr::perfpage::PerformancePage_OnViewCPUHistoryOneGraphAll;
use crate::taskmgr::perfpage::PerformancePage_OnViewCPUHistoryOneGraphPerCPU;
use crate::taskmgr::perfpage::PerformancePage_OnViewShowKernelTimes;
use crate::taskmgr::procpage::ProcessPageWndProc;
use crate::taskmgr::debug::ProcessPage_OnDebug;
use crate::taskmgr::endproc::ProcessPage_OnEndProcess;
use crate::taskmgr::endproc::ProcessPage_OnEndProcessTree;
use crate::taskmgr::procpage::ProcessPage_OnOpenFileLocation;
use crate::taskmgr::procpage::ProcessPage_OnProperties;
use crate::taskmgr::affinity::ProcessPage_OnSetAffinity;
use crate::taskmgr::column::ProcessPage_OnViewSelectColumns;
use crate::taskmgr::applpage::RefreshApplicationPage;
use crate::taskmgr::perfpage::RefreshPerformancePage;
use crate::taskmgr::procpage::RefreshProcessPage;
use crate::taskmgr::shutdown::ShutDown_Disconnect;
use crate::taskmgr::shutdown::ShutDown_EjectComputer;
use crate::taskmgr::shutdown::ShutDown_Hibernate;
use crate::taskmgr::shutdown::ShutDown_LockComputer;
use crate::taskmgr::shutdown::ShutDown_LogOffUser;
use crate::taskmgr::shutdown::ShutDown_PowerOff;
use crate::taskmgr::shutdown::ShutDown_Reboot;
use crate::taskmgr::shutdown::ShutDown_StandBy;
use crate::taskmgr::shutdown::ShutDown_SwitchUser;
use crate::taskmgr::run::TaskManager_OnFileNew;
use crate::taskmgr::optnmenu::TaskManager_OnOptionsAlwaysOnTop;
use crate::taskmgr::optnmenu::TaskManager_OnOptionsHideWhenMinimized;
use crate::taskmgr::optnmenu::TaskManager_OnOptionsMinimizeOnUse;
use crate::taskmgr::optnmenu::TaskManager_OnOptionsShow16BitTasks;
use crate::taskmgr::trayicon::TrayIcon_AddIcon;
use crate::taskmgr::trayicon::TrayIcon_RemoveIcon;
use crate::taskmgr::trayicon::TrayIcon_UpdateIcon;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_6AF5D4B374C5: [u16; 4] = [0x006Eu16, 0x002F, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_9568924DB153: [u16; 11] = [
    0x0074u16, 0x0061, 0x0073, 0x006B, 0x006D, 0x0067, 0x0072, 0x0072, 0x006F, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9B4959AB0A72: [u16; 12] = [
    0x0050u16, 0x0072, 0x0065, 0x0066, 0x0065, 0x0072, 0x0065, 0x006E, 0x0063, 0x0065,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_FCAAC800CA6D: [u16; 10] = [0x0025u16, 0x0073, 0x0020, 0x0028, 0x0030, 0x0078, 0x0025, 0x0078, 0x0029, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hMainWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hStatusWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hTabWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hWindowMenu: HMENU = ::core::ptr::null::<HMENU__>() as *mut HMENU__;

#[no_mangle]
pub static mut nMinimumWidth: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut nMinimumHeight: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut nOldWidth: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut nOldHeight: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut bTrackMenu: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bWasKeyboardInput: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut TaskManagerSettings: TASKMANAGER_SETTINGS = TASKMANAGER_SETTINGS {
    Maximized: 0,
    Left: 0,
    Top: 0,
    Right: 0,
    Bottom: 0,
    ActiveTabPage: 0,
    AlwaysOnTop: 0,
    MinimizeOnUse: 0,
    HideWhenMinimized: 0,
    Show16BitTasks: 0,
    UpdateSpeed: 0,
    ViewMode: 0,
    ShowProcessesFromAllUsers: 0,
    Columns: [0; 26],
    ColumnOrderArray: [0; 26],
    ColumnSizeArray: [0; 26],
    SortColumn: 0,
    SortAscending: 0,
    CPUHistory_OneGraphPerCPU: 0,
    ShowKernelTimes: 0,
};


pub const ETDT_DISABLE: u32 = 0x00000001;
pub const ETDT_ENABLE: u32 = 0x00000002;
pub const ETDT_USETABTEXTURE: u32 = 0x00000004;
pub const ETDT_ENABLETAB: u32 = ETDT_ENABLE | ETDT_USETABTEXTURE;

pub type ETDTProc = unsafe extern "system" fn(HWND, DWORD) -> HRESULT;

#[no_mangle]
pub unsafe extern "C" fn EnableDialogTheme(hwnd: HWND) -> BOOL {
    let mut hUXTheme: HMODULE;
    let mut fnEnableThemeDialogTexture: Option<ETDTProc>;

    hUXTheme = LoadLibraryA(b"uxtheme.dll\0".as_ptr() as LPCSTR) as HMODULE;

    if !hUXTheme.is_null() {
        fnEnableThemeDialogTexture = ::core::mem::transmute::<FARPROC, Option<ETDTProc>>(
            GetProcAddress(hUXTheme as HINSTANCE, b"EnableThemeDialogTexture\0".as_ptr() as LPCSTR),
        );

        if fnEnableThemeDialogTexture.is_some() {
            (fnEnableThemeDialogTexture.unwrap_unchecked())(hwnd, ETDT_ENABLETAB as DWORD);

            FreeLibrary(hUXTheme);
            return (TRUE) as BOOL;
        } else {
            // Failed to locate API!
            FreeLibrary(hUXTheme);
            return (FALSE) as BOOL;
        }
    } else {
        // Not running under XP? Just fail gracefully
        return (FALSE) as BOOL;
    }
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32 {
    let mut hProcess: HANDLE = core::ptr::null_mut();
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut tkp: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut hMutex: HANDLE = core::ptr::null_mut();

    /* check wether we're already running or not */
    hMutex = CreateMutexW(core::ptr::null_mut(), (TRUE) as BOOL, RUSTCONV_WSTR_9568924DB153.as_ptr());
    if !hMutex.is_null() && GetLastError() == ERROR_ALREADY_EXISTS {
        /* Restore existing taskmanager and bring window to front */
        /* Relies on the fact that the application title string and window title are the same */
        let mut hTaskMgr: HWND;
        let mut szTaskmgr: [TCHAR; 128] = core::mem::zeroed();

        LoadString(hInst, IDS_APP_TITLE, core::ptr::addr_of_mut!(szTaskmgr) as LPWSTR, (_countof!(szTaskmgr)) as i32);
        hTaskMgr = FindWindow(core::ptr::null_mut(), core::ptr::addr_of_mut!(szTaskmgr) as LPCWSTR);

        if !hTaskMgr.is_null() {
            SendMessage(hTaskMgr, WM_SYSCOMMAND, (SC_RESTORE) as _, 0);
            SetForegroundWindow(hTaskMgr);
        }

        CloseHandle(hMutex);
        return 0;
    } else if hMutex.is_null() {
        return 1;
    }

    /* Initialize global variables */
    hInst = hInstance;

    /* Change our priority class to HIGH */
    hProcess = OpenProcess(PROCESS_ALL_ACCESS, (FALSE) as BOOL, GetCurrentProcessId());
    SetPriorityClass(hProcess, HIGH_PRIORITY_CLASS);
    CloseHandle(hProcess);

    /* Now lets get the SE_DEBUG_NAME privilege
     * so that we can debug processes
     */

    /* Get a token for this process. */
    if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, core::ptr::addr_of_mut!(hToken)) != 0 {
        /* Get the LUID for the debug privilege. */
        if LookupPrivilegeValueW(core::ptr::null_mut(), SE_DEBUG_NAME, core::ptr::addr_of_mut!(tkp.Privileges[0].Luid)) != 0 {
            tkp.PrivilegeCount = 1; /* one privilege to set */
            tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

            /* Get the debug privilege for this process. */
            AdjustTokenPrivileges(hToken, (FALSE) as BOOL, core::ptr::addr_of_mut!(tkp), 0, core::ptr::null_mut() as PTOKEN_PRIVILEGES, core::ptr::null_mut());
        }
        CloseHandle(hToken);
    }

    /* Load our settings from the registry */
    LoadSettings();

    /* Initialize perf data */
    if PerfDataInitialize() == 0 {
        return -1;
    }

    /*
     * Set our shutdown parameters: we want to shutdown the very last,
     * without displaying any end task dialog if needed.
     */
    SetProcessShutdownParameters(1, SHUTDOWN_NORETRY);

    DialogBoxW!(hInst, IDD_TASKMGR_DIALOG as usize as LPCWSTR, core::ptr::null_mut(), Some(TaskManagerWndProc));

    /* Save our settings to the registry */
    SaveSettings();
    PerfDataUninitialize();
    CloseHandle(hMutex);
    if !hWindowMenu.is_null() {
        DestroyMenu(hWindowMenu);
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn TaskManagerWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pRC: LPRECT = core::ptr::null_mut();
    let mut pnmh: LPNMHDR = core::ptr::null_mut();
    let mut wp: WINDOWPLACEMENT = core::mem::zeroed();

    match message {
        WM_INITDIALOG => {
            // For now, the Help dialog menu item is disabled because of lacking of HTML Help support
            EnableMenuItem(GetMenu(hDlg), ID_HELP_TOPICS, MF_BYCOMMAND | MF_GRAYED);
            hMainWnd = hDlg;
            return OnCreate(hDlg) as INT_PTR;
        }
        WM_COMMAND => {
            if (LOWORD!(wParam)) as u32 == IDOK || (LOWORD!(wParam)) as u32 == IDCANCEL {
                EndDialog(hDlg, (LOWORD!(wParam)) as INT_PTR);
                return (TRUE) as INT_PTR;
            }
            /* Process menu commands */
            match (LOWORD!(wParam)) as u32 {
                ID_FILE_NEW => {
                    TaskManager_OnFileNew();
                }
                ID_OPTIONS_ALWAYSONTOP => {
                    TaskManager_OnOptionsAlwaysOnTop();
                }
                ID_OPTIONS_MINIMIZEONUSE => {
                    TaskManager_OnOptionsMinimizeOnUse();
                }
                ID_OPTIONS_HIDEWHENMINIMIZED => {
                    TaskManager_OnOptionsHideWhenMinimized();
                }
                ID_OPTIONS_SHOW16BITTASKS => {
                    TaskManager_OnOptionsShow16BitTasks();
                }
                ID_RESTORE => {
                    TaskManager_OnRestoreMainWindow();
                }
                ID_VIEW_LARGE | ID_VIEW_SMALL | ID_VIEW_DETAILS => {
                    ApplicationPage_OnView((LOWORD!(wParam)) as DWORD);
                }
                ID_VIEW_SHOWKERNELTIMES => {
                    PerformancePage_OnViewShowKernelTimes();
                }
                ID_VIEW_CPUHISTORY_ONEGRAPHALL => {
                    PerformancePage_OnViewCPUHistoryOneGraphAll();
                }
                ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU => {
                    PerformancePage_OnViewCPUHistoryOneGraphPerCPU();
                }
                ID_VIEW_UPDATESPEED_HIGH | ID_VIEW_UPDATESPEED_NORMAL | ID_VIEW_UPDATESPEED_LOW | ID_VIEW_UPDATESPEED_PAUSED => {
                    TaskManager_OnViewUpdateSpeed((LOWORD!(wParam)) as DWORD);
                }
                ID_VIEW_SELECTCOLUMNS => {
                    ProcessPage_OnViewSelectColumns();
                }
                ID_VIEW_REFRESH => {
                    PostMessageW(hDlg, WM_TIMER, 0, 0);
                }
                ID_WINDOWS_TILEHORIZONTALLY => {
                    ApplicationPage_OnWindowsTile(MDITILE_HORIZONTAL);
                }
                ID_WINDOWS_TILEVERTICALLY => {
                    ApplicationPage_OnWindowsTile(MDITILE_VERTICAL);
                }
                ID_WINDOWS_MINIMIZE => {
                    ApplicationPage_OnWindowsMinimize();
                }
                ID_WINDOWS_MAXIMIZE => {
                    ApplicationPage_OnWindowsMaximize();
                }
                ID_WINDOWS_CASCADE => {
                    ApplicationPage_OnWindowsCascade();
                }
                ID_WINDOWS_BRINGTOFRONT => {
                    ApplicationPage_OnWindowsBringToFront();
                }
                ID_APPLICATION_PAGE_SWITCHTO => {
                    ApplicationPage_OnSwitchTo();
                }
                ID_APPLICATION_PAGE_ENDTASK => {
                    ApplicationPage_OnEndTask();
                }
                ID_APPLICATION_PAGE_GOTOPROCESS => {
                    ApplicationPage_OnGotoProcess();
                }
                ID_PROCESS_PAGE_ENDPROCESS => {
                    ProcessPage_OnEndProcess();
                }
                ID_PROCESS_PAGE_ENDPROCESSTREE => {
                    ProcessPage_OnEndProcessTree();
                }
                ID_PROCESS_PAGE_DEBUG => {
                    ProcessPage_OnDebug();
                }
                ID_PROCESS_PAGE_SETAFFINITY => {
                    ProcessPage_OnSetAffinity();
                }
                ID_PROCESS_PAGE_SETPRIORITY_REALTIME => {
                    DoSetPriority(REALTIME_PRIORITY_CLASS);
                }
                ID_PROCESS_PAGE_SETPRIORITY_HIGH => {
                    DoSetPriority(HIGH_PRIORITY_CLASS);
                }
                ID_PROCESS_PAGE_SETPRIORITY_ABOVENORMAL => {
                    DoSetPriority(ABOVE_NORMAL_PRIORITY_CLASS);
                }
                ID_PROCESS_PAGE_SETPRIORITY_NORMAL => {
                    DoSetPriority(NORMAL_PRIORITY_CLASS);
                }
                ID_PROCESS_PAGE_SETPRIORITY_BELOWNORMAL => {
                    DoSetPriority(BELOW_NORMAL_PRIORITY_CLASS);
                }
                ID_PROCESS_PAGE_SETPRIORITY_LOW => {
                    DoSetPriority(IDLE_PRIORITY_CLASS);
                }
                ID_PROCESS_PAGE_PROPERTIES => {
                    ProcessPage_OnProperties();
                }
                ID_PROCESS_PAGE_OPENFILELOCATION => {
                    ProcessPage_OnOpenFileLocation();
                }

                /* ShutDown items */
                ID_SHUTDOWN_STANDBY => {
                    ShutDown_StandBy();
                }
                ID_SHUTDOWN_HIBERNATE => {
                    ShutDown_Hibernate();
                }
                ID_SHUTDOWN_POWEROFF => {
                    ShutDown_PowerOff();
                }
                ID_SHUTDOWN_REBOOT => {
                    ShutDown_Reboot();
                }
                ID_SHUTDOWN_LOGOFF => {
                    ShutDown_LogOffUser();
                }
                ID_SHUTDOWN_SWITCHUSER => {
                    ShutDown_SwitchUser();
                }
                ID_SHUTDOWN_LOCKCOMPUTER => {
                    ShutDown_LockComputer();
                }
                ID_SHUTDOWN_DISCONNECT => {
                    ShutDown_Disconnect();
                }
                ID_SHUTDOWN_EJECT_COMPUTER => {
                    ShutDown_EjectComputer();
                }

                ID_HELP_ABOUT => {
                    OnAbout();
                }
                ID_FILE_EXIT => {
                    EndDialog(hDlg, (IDOK) as INT_PTR);
                }
                _ => {}
            }
        }

        WM_ONTRAYICON => {
            match (lParam) as u32 {
                WM_RBUTTONDOWN => {
                    let mut pt: POINT = core::mem::zeroed();
                    let mut OnTop: BOOL = 0;
                    let mut hMenu: HMENU = core::ptr::null_mut();
                    let mut hPopupMenu: HMENU = core::ptr::null_mut();

                    GetCursorPos(core::ptr::addr_of_mut!(pt));

                    OnTop = ((GetWindowLongPtrW(hMainWnd, GWL_EXSTYLE) & (WS_EX_TOPMOST) as LONG_PTR) != 0) as BOOL;

                    hMenu = LoadMenuW(hInst, MAKEINTRESOURCEW!(IDR_TRAY_POPUP));
                    hPopupMenu = GetSubMenu(hMenu, 0);

                    if IsWindowVisible(hMainWnd) != 0 {
                        DeleteMenu(hPopupMenu, ID_RESTORE, MF_BYCOMMAND);
                    } else {
                        SetMenuDefaultItem(hPopupMenu, ID_RESTORE, FALSE);
                    }

                    if OnTop != 0 {
                        CheckMenuItem(hPopupMenu, ID_OPTIONS_ALWAYSONTOP, MF_BYCOMMAND | MF_CHECKED);
                    } else {
                        CheckMenuItem(hPopupMenu, ID_OPTIONS_ALWAYSONTOP, MF_BYCOMMAND | MF_UNCHECKED);
                    }

                    SetForegroundWindow(hMainWnd);
                    TrackPopupMenuEx(hPopupMenu, 0, pt.x, pt.y, hMainWnd, core::ptr::null_mut());

                    DestroyMenu(hMenu);
                }
                WM_LBUTTONDBLCLK => {
                    TaskManager_OnRestoreMainWindow();
                }
                _ => {}
            }
        }

        WM_NOTIFY => {
            pnmh = lParam as LPNMHDR;
            if (*pnmh).hwndFrom == hTabWnd && (*pnmh).idFrom == (IDC_TAB as UINT_PTR) {
                match (((*pnmh).code) as i32) as u32 {
                    __c if __c == TCN_SELCHANGE as u32 => {
                        TaskManager_OnTabWndSelChange();
                    }
                    __c if __c == TCN_KEYDOWN as u32 => {
                        bWasKeyboardInput = (TRUE) as i32;
                    }
                    NM_CLICK => {
                        bWasKeyboardInput = (FALSE) as i32;
                    }
                    _ => {}
                }
            }
        }

        WM_SIZING => {
            /* Make sure the user is sizing the dialog */
            /* in an acceptable range */
            pRC = lParam as LPRECT;
            if wParam == (WMSZ_LEFT) as _ || wParam == (WMSZ_TOPLEFT) as _ || wParam == (WMSZ_BOTTOMLEFT) as _ {
                /* If the width is too small enlarge it to the minimum */
                if nMinimumWidth as LONG > (*pRC).right - (*pRC).left {
                    (*pRC).left = (*pRC).right - nMinimumWidth as LONG;
                }
            } else {
                /* If the width is too small enlarge it to the minimum */
                if nMinimumWidth as LONG > (*pRC).right - (*pRC).left {
                    (*pRC).right = (*pRC).left + nMinimumWidth as LONG;
                }
            }
            if wParam == (WMSZ_TOP) as _ || wParam == (WMSZ_TOPLEFT) as _ || wParam == (WMSZ_TOPRIGHT) as _ {
                /* If the height is too small enlarge it to the minimum */
                if nMinimumHeight as LONG > (*pRC).bottom - (*pRC).top {
                    (*pRC).top = (*pRC).bottom - nMinimumHeight as LONG;
                }
            } else {
                /* If the height is too small enlarge it to the minimum */
                if nMinimumHeight as LONG > (*pRC).bottom - (*pRC).top {
                    (*pRC).bottom = (*pRC).top + nMinimumHeight as LONG;
                }
            }
            return (TRUE) as INT_PTR;
        }

        WM_SIZE => {
            /* Handle the window sizing in it's own function */
            OnSize(wParam, (LOWORD!(lParam)) as _, (HIWORD!(lParam)) as _);
        }

        WM_MOVE => {
            /* Handle the window moving in it's own function */
            OnMove(wParam, (LOWORD!(lParam)) as _, (HIWORD!(lParam)) as _);
        }

        WM_DESTROY => {
            ShowWindow(hDlg, (SW_HIDE) as i32);
            TrayIcon_RemoveIcon();
            wp.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
            GetWindowPlacement(hDlg, core::ptr::addr_of_mut!(wp));
            TaskManagerSettings.Left = wp.rcNormalPosition.left as ::core::ffi::c_int;
            TaskManagerSettings.Top = wp.rcNormalPosition.top as ::core::ffi::c_int;
            TaskManagerSettings.Right = wp.rcNormalPosition.right as ::core::ffi::c_int;
            TaskManagerSettings.Bottom = wp.rcNormalPosition.bottom as ::core::ffi::c_int;
            if IsZoomed(hDlg) != 0 || (wp.flags & WPF_RESTORETOMAXIMIZED) != 0 {
                TaskManagerSettings.Maximized = (TRUE) as i32;
            } else {
                TaskManagerSettings.Maximized = (FALSE) as i32;
            }
            /* Get rid of the allocated command line cache, if any */
            PerfDataDeallocCommandLineCache();
            if !hWindowMenu.is_null() {
                DestroyMenu(hWindowMenu);
            }
            return DefWindowProcW(hDlg, message, wParam, lParam) as INT_PTR;
        }

        WM_TIMER => {
            /* Refresh the performance data */
            PerfDataRefresh();
            RefreshApplicationPage();
            RefreshProcessPage();
            RefreshPerformancePage();
            TrayIcon_UpdateIcon();
        }

        WM_MENUSELECT => {
            TaskManager_OnMenuSelect(hDlg, (LOWORD!(wParam)) as UINT, (HIWORD!(wParam)) as UINT, lParam as HMENU);
        }

        WM_SYSCOLORCHANGE => {
            /* Forward WM_SYSCOLORCHANGE to common controls */
            SendMessage(hApplicationPageListCtrl, WM_SYSCOLORCHANGE, 0, 0);
            SendMessage(hProcessPageListCtrl, WM_SYSCOLORCHANGE, 0, 0);
            SendMessage(hProcessPageHeaderCtrl, WM_SYSCOLORCHANGE, 0, 0);
        }
        _ => {}
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn FillSolidRect(hDC: HDC, lpRect: LPCRECT, clr: COLORREF) {
    SetBkColor(hDC, clr);
    ExtTextOutW(hDC, 0, 0, ETO_OPAQUE, lpRect, core::ptr::null(), 0, core::ptr::null());
}

unsafe extern "C" fn SetUpdateSpeed(hWnd: HWND) {
    match TaskManagerSettings.UpdateSpeed {
        ID_VIEW_UPDATESPEED_HIGH => {
            SetTimer(hWnd, 1, 500, None);
        }
        ID_VIEW_UPDATESPEED_NORMAL => {
            SetTimer(hWnd, 1, 2000, None);
        }
        ID_VIEW_UPDATESPEED_LOW => {
            SetTimer(hWnd, 1, 4000, None);
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn OnCreate(hWnd: HWND) -> BOOL {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hEditMenu: HMENU = core::ptr::null_mut();
    let mut hViewMenu: HMENU = core::ptr::null_mut();
    let mut hShutMenu: HMENU = core::ptr::null_mut();
    let mut hUpdateSpeedMenu: HMENU = core::ptr::null_mut();
    let mut hCPUHistoryMenu: HMENU = core::ptr::null_mut();
    let mut nActivePage: ::core::ffi::c_int = 0;
    let mut nParts: [::core::ffi::c_int; 3] = [0; 3];
    let mut rc: RECT = core::mem::zeroed();
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut szLogOffItem: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpUserName: LPWSTR = core::ptr::null_mut();
    let mut item: TCITEM = core::mem::zeroed();
    let mut len: DWORD = 0;

    SendMessageW(
        hMainWnd,
        WM_SETICON,
        (ICON_BIG) as WPARAM,
        LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_TASKMANAGER)) as LPARAM,
    );

    /* Initialize the Windows Common Controls DLL */
    InitCommonControls();

    /* Get the minimum window sizes */
    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rc));
    nMinimumWidth = (rc.right - rc.left) as ::core::ffi::c_int;
    nMinimumHeight = (rc.bottom - rc.top) as ::core::ffi::c_int;

    /* Create the status bar */
    hStatusWnd = CreateStatusWindow(
        (WS_VISIBLE | WS_CHILD | WS_CLIPSIBLINGS | SBT_NOBORDERS) as i32,
        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
        hWnd,
        STATUS_WINDOW,
    );
    if hStatusWnd.is_null() {
        return (FALSE) as BOOL;
    }

    /* Create the status bar panes */
    nParts[0] = (STATUS_SIZE1 as i32);
    nParts[1] = (STATUS_SIZE2 as i32);
    nParts[2] = (STATUS_SIZE3 as i32);
    SendMessageW(
        hStatusWnd,
        SB_SETPARTS,
        (_countof!(nParts)) as WPARAM,
        core::ptr::addr_of_mut!(nParts) as *mut ::core::ffi::c_int as LPARAM,
    );

    /* Create tab pages */
    hTabWnd = GetDlgItem(hWnd, (IDC_TAB as i32));
    hApplicationPage = CreateDialogW!(
        hInst,
        MAKEINTRESOURCEW!(IDD_APPLICATION_PAGE),
        hWnd,
        Some(ApplicationPageWndProc),
    );
    EnableDialogTheme(hApplicationPage);
    hProcessPage = CreateDialogW!(
        hInst,
        MAKEINTRESOURCEW!(IDD_PROCESS_PAGE),
        hWnd,
        Some(ProcessPageWndProc),
    );
    EnableDialogTheme(hProcessPage);
    hPerformancePage = CreateDialogW!(
        hInst,
        MAKEINTRESOURCEW!(IDD_PERFORMANCE_PAGE),
        hWnd,
        Some(PerformancePageWndProc),
    );
    EnableDialogTheme(hPerformancePage);

    /* Insert tabs */
    LoadStringW(hInst, IDS_TAB_APPS, szTemp.as_mut_ptr(), 256);
    memset(
        core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void,
        0,
        core::mem::size_of::<TCITEM>(),
    );
    item.mask = TCIF_TEXT;
    item.pszText = szTemp.as_mut_ptr();
    let _ = TabCtrl_InsertItem!(hTabWnd, 0, core::ptr::addr_of_mut!(item));
    LoadStringW(hInst, IDS_TAB_PROCESSES, szTemp.as_mut_ptr(), 256);
    memset(
        core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void,
        0,
        core::mem::size_of::<TCITEM>(),
    );
    item.mask = TCIF_TEXT;
    item.pszText = szTemp.as_mut_ptr();
    let _ = TabCtrl_InsertItem!(hTabWnd, 1, core::ptr::addr_of_mut!(item));
    LoadStringW(hInst, IDS_TAB_PERFORMANCE, szTemp.as_mut_ptr(), 256);
    memset(
        core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void,
        0,
        core::mem::size_of::<TCITEM>(),
    );
    item.mask = TCIF_TEXT;
    item.pszText = szTemp.as_mut_ptr();
    let _ = TabCtrl_InsertItem!(hTabWnd, 2, core::ptr::addr_of_mut!(item));

    /* Size everything correctly */
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
    nOldWidth = rc.right as ::core::ffi::c_int;
    nOldHeight = rc.bottom as ::core::ffi::c_int;
    /* nOldStartX = rc.left; */
    /*nOldStartY = rc.top;  */

    /* #define PAGE_OFFSET_LEFT    17
       #define PAGE_OFFSET_TOP     72
       #define PAGE_OFFSET_WIDTH   (PAGE_OFFSET_LEFT*2)
       #define PAGE_OFFSET_HEIGHT  (PAGE_OFFSET_TOP+32) */

    if (TaskManagerSettings.Left != 0)
        || (TaskManagerSettings.Top != 0)
        || (TaskManagerSettings.Right != 0)
        || (TaskManagerSettings.Bottom != 0)
    {
        MoveWindow(
            hWnd,
            TaskManagerSettings.Left,
            TaskManagerSettings.Top,
            TaskManagerSettings.Right - TaskManagerSettings.Left,
            TaskManagerSettings.Bottom - TaskManagerSettings.Top,
            (TRUE) as BOOL,
        );
        // Preprocessor block `#ifdef __GNUC__TEST__` is excluded from the build; kept as dead code.
        if false {
            MoveWindow(
                hApplicationPage,
                TaskManagerSettings.Left + PAGE_OFFSET_LEFT,
                TaskManagerSettings.Top + PAGE_OFFSET_TOP,
                TaskManagerSettings.Right - TaskManagerSettings.Left - PAGE_OFFSET_WIDTH,
                TaskManagerSettings.Bottom - TaskManagerSettings.Top - PAGE_OFFSET_HEIGHT,
                (FALSE) as BOOL,
            );
            MoveWindow(
                hProcessPage,
                TaskManagerSettings.Left + PAGE_OFFSET_LEFT,
                TaskManagerSettings.Top + PAGE_OFFSET_TOP,
                TaskManagerSettings.Right - TaskManagerSettings.Left - PAGE_OFFSET_WIDTH,
                TaskManagerSettings.Bottom - TaskManagerSettings.Top - PAGE_OFFSET_HEIGHT,
                (FALSE) as BOOL,
            );
            MoveWindow(
                hPerformancePage,
                TaskManagerSettings.Left + PAGE_OFFSET_LEFT,
                TaskManagerSettings.Top + PAGE_OFFSET_TOP,
                TaskManagerSettings.Right - TaskManagerSettings.Left - PAGE_OFFSET_WIDTH,
                TaskManagerSettings.Bottom - TaskManagerSettings.Top - PAGE_OFFSET_HEIGHT,
                (FALSE) as BOOL,
            );
        }
    }
    if TaskManagerSettings.Maximized != 0 {
        ShowWindow(hWnd, (SW_MAXIMIZE) as i32);
    }

    /* Set the always on top style */
    hMenu = GetMenu(hWnd);
    hEditMenu = GetSubMenu(hMenu, 1);
    hViewMenu = GetSubMenu(hMenu, 2);
    hShutMenu = GetSubMenu(hMenu, 4);
    hUpdateSpeedMenu = GetSubMenu(hViewMenu, 1);
    hCPUHistoryMenu = GetSubMenu(hViewMenu, 7);

    /* Check or uncheck the always on top menu item */
    if TaskManagerSettings.AlwaysOnTop != 0 {
        CheckMenuItem(hEditMenu, ID_OPTIONS_ALWAYSONTOP, MF_BYCOMMAND | MF_CHECKED);
        SetWindowPos(
            hWnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
    } else {
        CheckMenuItem(hEditMenu, ID_OPTIONS_ALWAYSONTOP, MF_BYCOMMAND | MF_UNCHECKED);
        SetWindowPos(
            hWnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
    }

    /* Check or uncheck the minimize on use menu item */
    if TaskManagerSettings.MinimizeOnUse != 0 {
        CheckMenuItem(hEditMenu, ID_OPTIONS_MINIMIZEONUSE, MF_BYCOMMAND | MF_CHECKED);
    } else {
        CheckMenuItem(hEditMenu, ID_OPTIONS_MINIMIZEONUSE, MF_BYCOMMAND | MF_UNCHECKED);
    }

    /* Check or uncheck the hide when minimized menu item */
    if TaskManagerSettings.HideWhenMinimized != 0 {
        CheckMenuItem(hEditMenu, ID_OPTIONS_HIDEWHENMINIMIZED, MF_BYCOMMAND | MF_CHECKED);
    } else {
        CheckMenuItem(hEditMenu, ID_OPTIONS_HIDEWHENMINIMIZED, MF_BYCOMMAND | MF_UNCHECKED);
    }

    /* Check or uncheck the show 16-bit tasks menu item */
    if TaskManagerSettings.Show16BitTasks != 0 {
        CheckMenuItem(hEditMenu, ID_OPTIONS_SHOW16BITTASKS, MF_BYCOMMAND | MF_CHECKED);
    } else {
        CheckMenuItem(hEditMenu, ID_OPTIONS_SHOW16BITTASKS, MF_BYCOMMAND | MF_UNCHECKED);
    }

    /* Set the view mode */
    CheckMenuRadioItem(
        hViewMenu,
        ID_VIEW_LARGE,
        ID_VIEW_DETAILS,
        TaskManagerSettings.ViewMode,
        MF_BYCOMMAND,
    );

    if TaskManagerSettings.ShowKernelTimes != 0 {
        CheckMenuItem(hViewMenu, ID_VIEW_SHOWKERNELTIMES, MF_BYCOMMAND | MF_CHECKED);
    } else {
        CheckMenuItem(hViewMenu, ID_VIEW_SHOWKERNELTIMES, MF_BYCOMMAND | MF_UNCHECKED);
    }

    CheckMenuRadioItem(
        hUpdateSpeedMenu,
        ID_VIEW_UPDATESPEED_HIGH,
        ID_VIEW_UPDATESPEED_PAUSED,
        TaskManagerSettings.UpdateSpeed,
        MF_BYCOMMAND,
    );

    if TaskManagerSettings.CPUHistory_OneGraphPerCPU != 0 {
        CheckMenuRadioItem(
            hCPUHistoryMenu,
            ID_VIEW_CPUHISTORY_ONEGRAPHALL,
            ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU,
            ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU,
            MF_BYCOMMAND,
        );
    } else {
        CheckMenuRadioItem(
            hCPUHistoryMenu,
            ID_VIEW_CPUHISTORY_ONEGRAPHALL,
            ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU,
            ID_VIEW_CPUHISTORY_ONEGRAPHALL,
            MF_BYCOMMAND,
        );
    }

    nActivePage = TaskManagerSettings.ActiveTabPage;
    TabCtrl_SetCurFocus!(hTabWnd, 0);
    TabCtrl_SetCurFocus!(hTabWnd, 1);
    TabCtrl_SetCurFocus!(hTabWnd, 2);
    TabCtrl_SetCurFocus!(hTabWnd, nActivePage);

    /* Set the username in the "Log Off %s" item of the Shutdown menu */

    /* 1- Get the menu item text and store it temporarily */
    GetMenuStringW(
        hShutMenu,
        ID_SHUTDOWN_LOGOFF,
        szTemp.as_mut_ptr(),
        256,
        MF_BYCOMMAND,
    );

    /* 2- Retrieve the username length first, then allocate a buffer for it and call it again */
    if GetUserNameW(core::ptr::null_mut(), core::ptr::addr_of_mut!(len)) == 0
        && GetLastError() == ERROR_INSUFFICIENT_BUFFER
    {
        lpUserName = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !lpUserName.is_null()
            && GetUserNameW(lpUserName, core::ptr::addr_of_mut!(len)) != 0
        {
            _snwprintf(
                szLogOffItem.as_mut_ptr(),
                (_countof!(szLogOffItem)) as _,
                szTemp.as_mut_ptr(),
                lpUserName,
            );
            szLogOffItem[(_countof!(szLogOffItem) - 1) as usize] = UNICODE_NULL;
        } else {
            _snwprintf(
                szLogOffItem.as_mut_ptr(),
                (_countof!(szLogOffItem)) as _,
                szTemp.as_mut_ptr(),
                RUSTCONV_WSTR_6AF5D4B374C5.as_ptr(),
            );
        }

        if !lpUserName.is_null() {
            HeapFree(GetProcessHeap(), 0, lpUserName as PVOID);
        }
    } else {
        _snwprintf(
            szLogOffItem.as_mut_ptr(),
            (_countof!(szLogOffItem)) as _,
            szTemp.as_mut_ptr(),
            RUSTCONV_WSTR_6AF5D4B374C5.as_ptr(),
        );
    }

    /* 3- Set the menu item text to its formatted counterpart */
    ModifyMenuW(
        hShutMenu,
        ID_SHUTDOWN_LOGOFF,
        MF_BYCOMMAND | MF_STRING,
        (ID_SHUTDOWN_LOGOFF) as UINT_PTR,
        szLogOffItem.as_mut_ptr(),
    );

    /* Setup update speed */
    SetUpdateSpeed(hWnd);

    /*
     * Refresh the performance data
     * Sample it twice so we can establish
     * the delta values & cpu usage
     */
    PerfDataRefresh();
    PerfDataRefresh();

    RefreshApplicationPage();
    RefreshProcessPage();
    RefreshPerformancePage();

    TrayIcon_AddIcon();

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn OnMove(mut nType: WPARAM, mut cx: ::core::ffi::c_int, mut cy: ::core::ffi::c_int) {
    let _ = (nType, cx, cy);
    if false {
        MoveWindow(hApplicationPage, TaskManagerSettings.Left + PAGE_OFFSET_LEFT, TaskManagerSettings.Top + PAGE_OFFSET_TOP, TaskManagerSettings.Right - TaskManagerSettings.Left - PAGE_OFFSET_WIDTH, TaskManagerSettings.Bottom - TaskManagerSettings.Top - PAGE_OFFSET_HEIGHT, (FALSE) as BOOL);
        MoveWindow(hProcessPage, TaskManagerSettings.Left + PAGE_OFFSET_LEFT, TaskManagerSettings.Top + PAGE_OFFSET_TOP, TaskManagerSettings.Right - TaskManagerSettings.Left - PAGE_OFFSET_WIDTH, TaskManagerSettings.Bottom - TaskManagerSettings.Top - PAGE_OFFSET_HEIGHT, (FALSE) as BOOL);
        MoveWindow(hPerformancePage, TaskManagerSettings.Left + PAGE_OFFSET_LEFT, TaskManagerSettings.Top + PAGE_OFFSET_TOP, TaskManagerSettings.Right - TaskManagerSettings.Left - PAGE_OFFSET_WIDTH, TaskManagerSettings.Bottom - TaskManagerSettings.Top - PAGE_OFFSET_HEIGHT, (FALSE) as BOOL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn OnSize(mut nType: WPARAM, mut cx: c_int, mut cy: c_int) {
    let mut nParts: [c_int; 3] = [0; 3];
    let mut nXDifference: c_int = 0;
    let mut nYDifference: c_int = 0;
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if nType == (SIZE_MINIMIZED) as _ {
        if TaskManagerSettings.HideWhenMinimized != 0 {
            ShowWindow(hMainWnd, (SW_HIDE) as i32);
        }
        return;
    }

    nXDifference = cx - nOldWidth;
    nYDifference = cy - nOldHeight;
    nOldWidth = cx;
    nOldHeight = cy;

    /* Update the status bar size */
    GetWindowRect(hStatusWnd, core::ptr::addr_of_mut!(rc));
    SendMessageW(
        hStatusWnd,
        WM_SIZE,
        nType,
        MAKELPARAM!(cx, rc.bottom - rc.top),
    );

    /* Update the status bar pane sizes */
    nParts[0] = (STATUS_SIZE1 as i32);
    nParts[1] = (STATUS_SIZE2 as i32);
    nParts[2] = cx;
    SendMessageW(
        hStatusWnd,
        SB_SETPARTS,
        (_countof!(nParts)) as WPARAM,
        core::ptr::addr_of_mut!(nParts) as *mut c_int as LPARAM,
    );

    /* Resize the tab control */
    GetWindowRect(hTabWnd, core::ptr::addr_of_mut!(rc));
    cx = (rc.right - rc.left) + nXDifference;
    cy = (rc.bottom - rc.top) + nYDifference;
    SetWindowPos(
        hTabWnd,
        core::ptr::null_mut(),
        0,
        0,
        cx,
        cy,
        (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOZORDER) as UINT,
    );

    /* Resize the application page */
    GetWindowRect(hApplicationPage, core::ptr::addr_of_mut!(rc));
    cx = (rc.right - rc.left) + nXDifference;
    cy = (rc.bottom - rc.top) + nYDifference;
    SetWindowPos(
        hApplicationPage,
        core::ptr::null_mut(),
        0,
        0,
        cx,
        cy,
        (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOZORDER) as UINT,
    );

    /* Resize the process page */
    GetWindowRect(hProcessPage, core::ptr::addr_of_mut!(rc));
    cx = (rc.right - rc.left) + nXDifference;
    cy = (rc.bottom - rc.top) + nYDifference;
    SetWindowPos(
        hProcessPage,
        core::ptr::null_mut(),
        0,
        0,
        cx,
        cy,
        (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOZORDER) as UINT,
    );

    /* Resize the performance page */
    GetWindowRect(hPerformancePage, core::ptr::addr_of_mut!(rc));
    cx = (rc.right - rc.left) + nXDifference;
    cy = (rc.bottom - rc.top) + nYDifference;
    SetWindowPos(
        hPerformancePage,
        core::ptr::null_mut(),
        0,
        0,
        cx,
        cy,
        (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOZORDER) as UINT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn LoadSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szSubKey: [WCHAR; 29] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0052,
    0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005C, 0x0054, 0x0061, 0x0073,
    0x006B, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000,
];
    let mut i: c_int;
    let mut dwSize: DWORD;

    /* Window size & position settings */
    TaskManagerSettings.Maximized = (FALSE) as i32;
    TaskManagerSettings.Left = 0;
    TaskManagerSettings.Top = 0;
    TaskManagerSettings.Right = 0;
    TaskManagerSettings.Bottom = 0;

    /* Tab settings */
    TaskManagerSettings.ActiveTabPage = 0;

    /* Options menu settings */
    TaskManagerSettings.AlwaysOnTop = (TRUE) as i32;
    TaskManagerSettings.MinimizeOnUse = (TRUE) as i32;
    TaskManagerSettings.HideWhenMinimized = (FALSE) as i32;
    TaskManagerSettings.Show16BitTasks = (TRUE) as i32;

    /* Update speed settings */
    TaskManagerSettings.UpdateSpeed = ID_VIEW_UPDATESPEED_NORMAL;

    /* Applications page settings */
    TaskManagerSettings.ViewMode = ID_VIEW_DETAILS;

    /* Processes page settings */
    TaskManagerSettings.ShowProcessesFromAllUsers = (FALSE) as i32; /* It's the default */

    i = 0;
    while i < (COLUMN_NMAX as i32) {
        TaskManagerSettings.Columns[i as usize] = ColumnPresets[i as usize].bDefaults;
        TaskManagerSettings.ColumnOrderArray[i as usize] = i;
        TaskManagerSettings.ColumnSizeArray[i as usize] = ColumnPresets[i as usize].size;
        i += 1;
    }

    TaskManagerSettings.SortColumn = (COLUMN_IMAGENAME as i32);
    TaskManagerSettings.SortAscending = (TRUE) as i32;

    /* Performance page settings */
    TaskManagerSettings.CPUHistory_OneGraphPerCPU = (TRUE) as i32;
    TaskManagerSettings.ShowKernelTimes = (FALSE) as i32;

    /* Open the key */
    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        szSubKey.as_ptr() as LPCWSTR,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    )) as u32 != ERROR_SUCCESS
    {
        return;
    }
    /* Read the settings */
    dwSize = core::mem::size_of::<TASKMANAGER_SETTINGS>() as DWORD;
    RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_9B4959AB0A72.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(TaskManagerSettings) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );

    /*
     * ATM, the 'ImageName' column is always visible
     * (and grayed in configuration dialog)
     * This will avoid troubles if the registry gets corrupted.
     */
    // column.h: `#define Column_ImageName Columns[COLUMN_IMAGENAME]`, a member-access macro.
    TaskManagerSettings.Columns[COLUMN_IMAGENAME as usize] = (TRUE) as i32;

    /* Close the key */
    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn SaveSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szSubKey: [u16; 29] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0052,
    0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005C, 0x0054, 0x0061, 0x0073,
    0x006B, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000,
];

    /* Open (or create) the key */
    if (RegCreateKeyExW(HKEY_CURRENT_USER, szSubKey.as_mut_ptr(), 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE, KEY_WRITE, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
        return;
    }
    /* Save the settings */
    RegSetValueExW(hKey, RUSTCONV_WSTR_9B4959AB0A72.as_ptr(), 0, REG_BINARY, core::ptr::addr_of_mut!(TaskManagerSettings) as LPBYTE, core::mem::size_of::<TASKMANAGER_SETTINGS>() as u32);
    /* Close the key */
    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnRestoreMainWindow() {
    let mut OnTop: BOOL;

    OnTop = ((GetWindowLongPtrW(hMainWnd, GWL_EXSTYLE) & (WS_EX_TOPMOST) as LONG_PTR) != 0) as BOOL;

    OpenIcon(hMainWnd);
    SetForegroundWindow(hMainWnd);
    SetWindowPos(hMainWnd, (if OnTop != 0 { HWND_TOPMOST } else { HWND_TOP }), 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE | SWP_SHOWWINDOW);
}

#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnMenuSelect(hWnd: HWND, nItemID: UINT, nFlags: UINT, hSysMenu: HMENU) {
    let mut str: [WCHAR; 100] = [0; 100];

    /*
     * Reset the status bar if we close the current menu, or
     * we open the system menu or hover above a menu separator.
     * Adapted from comctl32!MenuHelp().
     */
    if (LOWORD!(nFlags) == 0xFFFF && hSysMenu.is_null())
        || ((nFlags & (MF_SEPARATOR | MF_SYSMENU)) != 0)
    {
        /* Set the status bar for multiple-parts output */
        SendMessageW(
            hStatusWnd,
            SB_SIMPLE as UINT,
            (FALSE) as WPARAM,
            (0) as LPARAM,
        );
        bTrackMenu = FALSE as BOOL;

        /* Trigger update of status bar columns and performance page asynchronously */
        RefreshPerformancePage();
        return;
    }

    /* Otherwise, retrieve the appropriate menu hint string */
    if LoadStringW(
        hInst,
        nItemID,
        core::ptr::addr_of_mut!(str) as LPWSTR,
        _countof!(str) as i32,
    ) != 0
    {
        /* First newline terminates actual string */
        let mut lpsz: LPWSTR = wcschr(core::ptr::addr_of_mut!(str) as *mut WCHAR, '\n' as WCHAR) as LPWSTR;
        if !lpsz.is_null() {
            *lpsz = '\0' as WCHAR;
        }
    }

    /* Set the status bar for single-part output, if needed... */
    if bTrackMenu == 0 {
        SendMessageW(
            hStatusWnd,
            SB_SIMPLE as UINT,
            (TRUE) as WPARAM,
            (0) as LPARAM,
        );
    }
    bTrackMenu = TRUE as BOOL;

    /* ... and display the menu hint */
    SendMessageW(
        hStatusWnd,
        SB_SETTEXT as UINT,
        (SB_SIMPLEID | SBT_NOBORDERS) as WPARAM,
        core::ptr::addr_of_mut!(str) as *mut WCHAR as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnViewUpdateSpeed(dwSpeed: DWORD) {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hViewMenu: HMENU = core::ptr::null_mut();
    let mut hUpdateSpeedMenu: HMENU = core::ptr::null_mut();

    hMenu = GetMenu(hMainWnd);
    hViewMenu = GetSubMenu(hMenu, 2);
    hUpdateSpeedMenu = GetSubMenu(hViewMenu, 1);

    TaskManagerSettings.UpdateSpeed = dwSpeed;
    CheckMenuRadioItem(hUpdateSpeedMenu, ID_VIEW_UPDATESPEED_HIGH, ID_VIEW_UPDATESPEED_PAUSED, dwSpeed, MF_BYCOMMAND);

    KillTimer(hMainWnd, 1);

    SetUpdateSpeed(hMainWnd);
}
#[no_mangle]
pub unsafe extern "C" fn TaskManager_OnTabWndSelChange() {
    let mut i: ::core::ffi::c_int = 0;
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hOptionsMenu: HMENU = core::ptr::null_mut();
    let mut hViewMenu: HMENU = core::ptr::null_mut();
    let mut hSubMenu: HMENU = core::ptr::null_mut();
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut sysInfo: SYSTEM_INFO = core::mem::zeroed();

    hMenu = GetMenu(hMainWnd);
    hViewMenu = GetSubMenu(hMenu, 2);
    hOptionsMenu = GetSubMenu(hMenu, 1);
    TaskManagerSettings.ActiveTabPage = TabCtrl_GetCurSel!(hTabWnd);
    i = GetMenuItemCount(hViewMenu) - 1;
    while i > 2 {
        hSubMenu = GetSubMenu(hViewMenu, i);
        if !hSubMenu.is_null() {
            DestroyMenu(hSubMenu);
        }
        RemoveMenu(hViewMenu, i as UINT, MF_BYPOSITION as UINT);
        i -= 1;
    }
    RemoveMenu(hOptionsMenu, 3 as UINT, MF_BYPOSITION as UINT);
    if !hWindowMenu.is_null() {
        DestroyMenu(hWindowMenu);
    }

    let sel = TaskManagerSettings.ActiveTabPage;
    if sel == 0 {
        ShowWindow(hApplicationPage, (SW_SHOW) as i32);
        ShowWindow(hProcessPage, (SW_HIDE) as i32);
        ShowWindow(hPerformancePage, (SW_HIDE) as i32);
        BringWindowToTop(hApplicationPage);

        LoadStringW(hInst, IDS_MENU_LARGEICONS as UINT, szTemp.as_mut_ptr(), 256);
        AppendMenuW(hViewMenu, MF_STRING as UINT, ID_VIEW_LARGE as UINT_PTR, szTemp.as_ptr());

        LoadStringW(hInst, IDS_MENU_SMALLICONS as UINT, szTemp.as_mut_ptr(), 256);
        AppendMenuW(hViewMenu, MF_STRING as UINT, ID_VIEW_SMALL as UINT_PTR, szTemp.as_ptr());

        LoadStringW(hInst, IDS_MENU_DETAILS as UINT, szTemp.as_mut_ptr(), 256);
        AppendMenuW(hViewMenu, MF_STRING as UINT, ID_VIEW_DETAILS as UINT_PTR, szTemp.as_ptr());

        if GetMenuItemCount(hMenu) <= 5 {
            hWindowMenu = LoadMenuW(hInst, MAKEINTRESOURCEW!(IDR_WINDOWSMENU));

            LoadStringW(hInst, IDS_MENU_WINDOWS as UINT, szTemp.as_mut_ptr(), 256);
            InsertMenuW(hMenu, 3 as UINT, (MF_BYPOSITION | MF_POPUP) as UINT, hWindowMenu as UINT_PTR, szTemp.as_ptr());

            DrawMenuBar(hMainWnd);
        }
        CheckMenuRadioItem(hViewMenu, ID_VIEW_LARGE as UINT, ID_VIEW_DETAILS as UINT, TaskManagerSettings.ViewMode as UINT, MF_BYCOMMAND as UINT);

        /*
         * Give the application list control focus
         */
        if bWasKeyboardInput == 0 {
            SetFocus(hApplicationPageListCtrl);
        }
    } else if sel == 1 {
        ShowWindow(hApplicationPage, (SW_HIDE) as i32);
        ShowWindow(hProcessPage, (SW_SHOW) as i32);
        ShowWindow(hPerformancePage, (SW_HIDE) as i32);
        BringWindowToTop(hProcessPage);

        LoadStringW(hInst, IDS_MENU_SELECTCOLUMNS as UINT, szTemp.as_mut_ptr(), 256);
        AppendMenuW(hViewMenu, MF_STRING as UINT, ID_VIEW_SELECTCOLUMNS as UINT_PTR, szTemp.as_ptr());

        LoadStringW(hInst, IDS_MENU_16BITTASK as UINT, szTemp.as_mut_ptr(), 256);
        AppendMenuW(hOptionsMenu, MF_STRING as UINT, ID_OPTIONS_SHOW16BITTASKS as UINT_PTR, szTemp.as_ptr());

        if TaskManagerSettings.Show16BitTasks != 0 {
            CheckMenuItem(hOptionsMenu, ID_OPTIONS_SHOW16BITTASKS as UINT, (MF_BYCOMMAND | MF_CHECKED) as UINT);
        }
        if GetMenuItemCount(hMenu) > 5 {
            DeleteMenu(hMenu, 3 as UINT, MF_BYPOSITION as UINT);
            DrawMenuBar(hMainWnd);
        }
        /*
         * Give the process list control focus
         */
        if bWasKeyboardInput == 0 {
            SetFocus(hProcessPageListCtrl);
        }
    } else if sel == 2 {
        ShowWindow(hApplicationPage, (SW_HIDE) as i32);
        ShowWindow(hProcessPage, (SW_HIDE) as i32);
        ShowWindow(hPerformancePage, (SW_SHOW) as i32);
        BringWindowToTop(hPerformancePage);
        if GetMenuItemCount(hMenu) > 5 {
            DeleteMenu(hMenu, 3 as UINT, MF_BYPOSITION as UINT);
            DrawMenuBar(hMainWnd);
        }

        GetSystemInfo(core::ptr::addr_of_mut!(sysInfo));

        /* Hide CPU graph options on single CPU systems */
        if sysInfo.dwNumberOfProcessors > 1 {
            hSubMenu = CreatePopupMenu();

            LoadStringW(hInst, IDS_MENU_ONEGRAPHALLCPUS as UINT, szTemp.as_mut_ptr(), 256);
            AppendMenuW(hSubMenu, MF_STRING as UINT, ID_VIEW_CPUHISTORY_ONEGRAPHALL as UINT_PTR, szTemp.as_ptr());

            LoadStringW(hInst, IDS_MENU_ONEGRAPHPERCPU as UINT, szTemp.as_mut_ptr(), 256);
            AppendMenuW(hSubMenu, MF_STRING as UINT, ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU as UINT_PTR, szTemp.as_ptr());

            LoadStringW(hInst, IDS_MENU_CPUHISTORY as UINT, szTemp.as_mut_ptr(), 256);
            AppendMenuW(hViewMenu, (MF_STRING | MF_POPUP) as UINT, hSubMenu as UINT_PTR, szTemp.as_ptr());

            if TaskManagerSettings.CPUHistory_OneGraphPerCPU != 0 {
                CheckMenuRadioItem(hSubMenu, ID_VIEW_CPUHISTORY_ONEGRAPHALL as UINT, ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU as UINT, ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU as UINT, MF_BYCOMMAND as UINT);
            } else {
                CheckMenuRadioItem(hSubMenu, ID_VIEW_CPUHISTORY_ONEGRAPHALL as UINT, ID_VIEW_CPUHISTORY_ONEGRAPHPERCPU as UINT, ID_VIEW_CPUHISTORY_ONEGRAPHALL as UINT, MF_BYCOMMAND as UINT);
            }
        }

        LoadStringW(hInst, IDS_MENU_SHOWKERNELTIMES as UINT, szTemp.as_mut_ptr(), 256);
        AppendMenuW(hViewMenu, MF_STRING as UINT, ID_VIEW_SHOWKERNELTIMES as UINT_PTR, szTemp.as_ptr());

        if TaskManagerSettings.ShowKernelTimes != 0 {
            CheckMenuItem(hViewMenu, ID_VIEW_SHOWKERNELTIMES as UINT, (MF_BYCOMMAND | MF_CHECKED) as UINT);
        } else {
            CheckMenuItem(hViewMenu, ID_VIEW_SHOWKERNELTIMES as UINT, (MF_BYCOMMAND | MF_UNCHECKED) as UINT);
        }

        /*
         * Give the tab control focus
         */
        if bWasKeyboardInput == 0 {
            SetFocus(hTabWnd);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ConfirmMessageBox(mut hWnd: HWND, mut Text: LPCWSTR, mut Title: LPCWSTR, mut Type: UINT) -> BOOL {
    let mut positive: UINT = (if (Type & 0xF) <= MB_OKCANCEL {
        IDOK
    } else {
        IDYES
    }) as UINT;
    if (GetKeyState(VK_SHIFT as ::core::ffi::c_int) as ::core::ffi::c_int) < 0 {
        return (TRUE) as BOOL;
    }
    return (MessageBoxW(hWnd, Text, Title, Type) as UINT == positive) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ShowWin32Error(dwError: DWORD) {
    let mut lpMessageBuffer: LPWSTR = core::ptr::null_mut();

    if FormatMessageW((FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM) as DWORD,
                       core::ptr::null(),
                       dwError,
                       MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
                       core::ptr::addr_of_mut!(lpMessageBuffer) as LPWSTR,
                       0 as DWORD,
                       core::ptr::null_mut()) != 0 as DWORD
    {
        MessageBoxW(hMainWnd, lpMessageBuffer as LPCWSTR, core::ptr::null(), (MB_OK | MB_ICONERROR) as UINT);
        if !lpMessageBuffer.is_null() {
            LocalFree(lpMessageBuffer as HLOCAL);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetLastErrorText(lpszBuf: LPTSTR, dwSize: DWORD) -> LPTSTR {
    let mut dwRet: DWORD;
    let mut lpszTemp: LPWSTR = core::ptr::null_mut();
    dwRet = FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY) as DWORD,
        core::ptr::null_mut(),
        GetLastError(),
        LANG_NEUTRAL as DWORD,
        core::ptr::addr_of_mut!(lpszTemp) as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    );

    /* supplied buffer is not long enough */
    if dwRet == 0
        || ((dwSize as core::ffi::c_long)
            < (dwRet as core::ffi::c_long + 14 as core::ffi::c_long))
    {
        *lpszBuf.offset(0 as isize) = 0x0000u16;
    } else {
        /*remove cr and newline character */
        *lpszTemp.offset(
            (lstrlenW(lpszTemp as LPCWSTR) - 2) as isize,
        ) = 0x0000u16;
        wsprintfW(
            lpszBuf,
            RUSTCONV_WSTR_FCAAC800CA6D.as_ptr(),
            lpszTemp,
            GetLastError() as core::ffi::c_int,
        );
    }
    if !lpszTemp.is_null() {
        LocalFree(lpszTemp as HLOCAL);
    }
    return lpszBuf;
}
#[no_mangle]
pub unsafe extern "C" fn GetRefreshMessage(msg: *mut MSG) -> BOOL {
    let mut extra: MSG = core::mem::zeroed();
    if GetMessage(msg, core::ptr::null_mut(), 0, 0) <= 0 {
        return (FALSE) as BOOL;
    }
    if (*msg).message == WM_TIMER {
        while PeekMessage(core::ptr::addr_of_mut!(extra), core::ptr::null_mut(), WM_TIMER, WM_TIMER, PM_REMOVE) != 0 {
        }
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EndLocalThread(hThread: *mut HANDLE, dwThread: DWORD) -> DWORD {
    let mut dwExitCodeThread: DWORD = 0;

    if !(*hThread).is_null() {
        PostThreadMessage(dwThread, WM_QUIT, 0, 0);
        loop {
            let mut msg: MSG = core::mem::zeroed();

            /* The refresh threads SendMessage into our windows. A plain
             * WaitForSingleObject services none of those, so each send used
             * to stall until the next 500ms poll, and a pass of dozens of
             * ListView sends turned closing the window into minutes. Wake on
             * input of any kind instead. */
            if WAIT_OBJECT_0 == MsgWaitForMultipleObjects(1, hThread, (FALSE) as BOOL,
                                                           INFINITE, QS_ALLINPUT)
            {
                break;
            }
            while PeekMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(core::ptr::addr_of_mut!(msg));
                DispatchMessage(core::ptr::addr_of_mut!(msg));
            }
        }
        GetExitCodeThread(*hThread, core::ptr::addr_of_mut!(dwExitCodeThread));
        CloseHandle(*hThread);
        *hThread = core::ptr::null_mut();
    }
    return dwExitCodeThread;
}

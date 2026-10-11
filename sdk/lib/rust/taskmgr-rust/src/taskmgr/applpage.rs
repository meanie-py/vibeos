use crate::taskmgr::taskmgr::EndLocalThread;
use crate::taskmgr::taskmgr::GetRefreshMessage;
use crate::taskmgr::procpage::ProcGetIndexByProcessId;
use crate::taskmgr::endproc::ProcessPage_OnEndProcess;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1C71252208A9: [u16; 11] = [
    0x0055u16, 0x0053, 0x0045, 0x0052, 0x0033, 0x0032, 0x002E, 0x0044, 0x004C, 0x004C,
    0x0000,
];
pub static RUSTCONV_WSTR_5E587F4D45D2: [u16; 6] = [0x0054u16, 0x0061, 0x0073, 0x006B, 0x0073, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn SendMessageTimeoutW( hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, fuFlags: UINT, uTimeout: UINT, lpdwResult: PDWORD_PTR, ) -> LRESULT;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplicationPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hApplicationPageListCtrl: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hApplicationPageEndTaskButton: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hApplicationPageSwitchToButton: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hApplicationPageNewTaskButton: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut nApplicationPageWidth: ::core::ffi::c_int = 0;

static mut nApplicationPageHeight: ::core::ffi::c_int = 0;

static mut bSortAscending: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut noApps: BOOL = 0;

#[no_mangle]
pub static mut bApplicationPageSelectionMade: BOOL = (FALSE) as i32;

static mut hApplicationThread: HANDLE = NULL;

static mut dwApplicationThread: DWORD = 0;

#[repr(C)]
pub struct APPLICATION_PAGE_LIST_ITEM {
    pub hWnd: HWND,
    pub szTitle: [WCHAR; 260],
    pub hIcon: HICON,
    pub bHung: BOOL,
}
pub type LPAPPLICATION_PAGE_LIST_ITEM = *mut APPLICATION_PAGE_LIST_ITEM;

// #endif

unsafe extern "C" fn GetSystemColorDepth() -> INT {
    let mut pDevMode: DEVMODE = core::mem::zeroed();
    let mut ColorDepth: INT = 0;

    pDevMode.dmSize = core::mem::size_of::<DEVMODE>() as WORD;
    pDevMode.dmDriverExtra = 0 as WORD;

    if EnumDisplaySettings(
        core::ptr::null_mut(),
        ENUM_CURRENT_SETTINGS,
        core::ptr::addr_of_mut!(pDevMode),
    ) == 0
    {
        return ILC_COLOR as INT;
    }

    match pDevMode.dmBitsPerPel {
        32 => {
            ColorDepth = ILC_COLOR32 as INT;
        }
        24 => {
            ColorDepth = ILC_COLOR24 as INT;
        }
        16 => {
            ColorDepth = ILC_COLOR16 as INT;
        }
        8 => {
            ColorDepth = ILC_COLOR8 as INT;
        }
        4 => {
            ColorDepth = ILC_COLOR4 as INT;
        }
        _ => {
            ColorDepth = ILC_COLOR as INT;
        }
    }

    return ColorDepth;
}
#[no_mangle]
pub unsafe extern "C" fn AppPageCleanup() {
    let mut i: c_int = 0;
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut pData: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) as c_int {
        memset(
            core::ptr::addr_of_mut!(item) as *mut c_void,
            0,
            core::mem::size_of::<LV_ITEM>() as size_t,
        );
        item.mask = LVIF_PARAM;
        item.iItem = i;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));
        pData = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
        HeapFree(
            GetProcessHeap(),
            0,
            pData as PVOID,
        );
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ApplicationPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut rc: RECT = core::mem::zeroed();
    let mut nXDifference: ::core::ffi::c_int;
    let mut nYDifference: ::core::ffi::c_int;
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut cx: ::core::ffi::c_int;
    let mut cy: ::core::ffi::c_int;

    match message {
        WM_INITDIALOG => {
            /* Save the width and height */
            GetClientRect(hDlg, core::ptr::addr_of_mut!(rc));
            nApplicationPageWidth = rc.right;
            nApplicationPageHeight = rc.bottom;

            /* Update window position */
            SetWindowPos(hDlg, core::ptr::null_mut(), 15, 30, 0, 0, SWP_NOACTIVATE|SWP_NOOWNERZORDER|SWP_NOSIZE|SWP_NOZORDER);

            /* Get handles to the controls */
            hApplicationPageListCtrl = GetDlgItem(hDlg, (IDC_APPLIST as i32));
            hApplicationPageEndTaskButton = GetDlgItem(hDlg, (IDC_ENDTASK as i32));
            hApplicationPageSwitchToButton = GetDlgItem(hDlg, (IDC_SWITCHTO as i32));
            hApplicationPageNewTaskButton = GetDlgItem(hDlg, (IDC_NEWTASK as i32));

            SetWindowTextW(hApplicationPageListCtrl, RUSTCONV_WSTR_5E587F4D45D2.as_ptr());

            /* Initialize the application page's controls */
            column.mask = LVCF_TEXT|LVCF_WIDTH;

            LoadStringW(hInst, IDS_TAB_TASK, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 250;
            let _ = ListView_InsertColumn!(hApplicationPageListCtrl, 0, core::ptr::addr_of_mut!(column));    /* Add the "Task" column */
            column.mask = LVCF_TEXT|LVCF_WIDTH;
            LoadStringW(hInst, IDS_TAB_STATUS, szTemp.as_mut_ptr(), 256);
            column.pszText = szTemp.as_mut_ptr();
            column.cx = 95;
            let _ = ListView_InsertColumn!(hApplicationPageListCtrl, 1, core::ptr::addr_of_mut!(column));    /* Add the "Status" column */

            let _ = ListView_SetImageList!(hApplicationPageListCtrl, ImageList_Create(16, 16, (GetSystemColorDepth() | (ILC_MASK as i32)) as UINT, 0, 1), LVSIL_SMALL);
            let _ = ListView_SetImageList!(hApplicationPageListCtrl, ImageList_Create(32, 32, (GetSystemColorDepth() | (ILC_MASK as i32)) as UINT, 0, 1), LVSIL_NORMAL);

            UpdateApplicationListControlViewSetting();

            /* Start our refresh thread */
            hApplicationThread = CreateThread(core::ptr::null_mut(), 0, Some(ApplicationPageRefreshThread), core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(dwApplicationThread));

            /* Refresh page */
            ApplicationPageUpdate();

            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            /* Close refresh thread */
            EndLocalThread(core::ptr::addr_of_mut!(hApplicationThread), dwApplicationThread);
            AppPageCleanup();
        }

        WM_COMMAND => {
            /* Handle the button clicks */
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_ENDTASK => {
                    ApplicationPage_OnEndTask();
                }
                IDC_SWITCHTO => {
                    ApplicationPage_OnSwitchTo();
                }
                IDC_NEWTASK => {
                    SendMessageW(hMainWnd, WM_COMMAND, MAKEWPARAM!(ID_FILE_NEW, 0), 0);
                }
                _ => {}
            }
        }

        WM_SIZE => {
            if wParam == SIZE_MINIMIZED as WPARAM {
                return 0 as INT_PTR;
            }

            cx = LOWORD!(lParam) as ::core::ffi::c_int;
            cy = HIWORD!(lParam) as ::core::ffi::c_int;
            nXDifference = cx - nApplicationPageWidth;
            nYDifference = cy - nApplicationPageHeight;
            nApplicationPageWidth = cx;
            nApplicationPageHeight = cy;

            /* Reposition the application page's controls */
            GetWindowRect(hApplicationPageListCtrl, core::ptr::addr_of_mut!(rc));
            cx = (rc.right - rc.left) + nXDifference;
            cy = (rc.bottom - rc.top) + nYDifference;
            SetWindowPos(hApplicationPageListCtrl, core::ptr::null_mut(), 0, 0, cx, cy, SWP_NOACTIVATE|SWP_NOOWNERZORDER|SWP_NOMOVE|SWP_NOZORDER);
            InvalidateRect(hApplicationPageListCtrl, core::ptr::null(), (TRUE) as BOOL);

            GetClientRect(hApplicationPageEndTaskButton, core::ptr::addr_of_mut!(rc));
            MapWindowPoints(hApplicationPageEndTaskButton, hDlg, core::ptr::addr_of_mut!(rc) as PRECT as LPPOINT, (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT);
            cx = rc.left + nXDifference;
            cy = rc.top + nYDifference;
            SetWindowPos(hApplicationPageEndTaskButton, core::ptr::null_mut(), cx, cy, 0, 0, SWP_NOACTIVATE|SWP_NOOWNERZORDER|SWP_NOSIZE|SWP_NOZORDER);
            InvalidateRect(hApplicationPageEndTaskButton, core::ptr::null(), (TRUE) as BOOL);

            GetClientRect(hApplicationPageSwitchToButton, core::ptr::addr_of_mut!(rc));
            MapWindowPoints(hApplicationPageSwitchToButton, hDlg, core::ptr::addr_of_mut!(rc) as PRECT as LPPOINT, (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT);
            cx = rc.left + nXDifference;
            cy = rc.top + nYDifference;
            SetWindowPos(hApplicationPageSwitchToButton, core::ptr::null_mut(), cx, cy, 0, 0, SWP_NOACTIVATE|SWP_NOOWNERZORDER|SWP_NOSIZE|SWP_NOZORDER);
            InvalidateRect(hApplicationPageSwitchToButton, core::ptr::null(), (TRUE) as BOOL);

            GetClientRect(hApplicationPageNewTaskButton, core::ptr::addr_of_mut!(rc));
            MapWindowPoints(hApplicationPageNewTaskButton, hDlg, core::ptr::addr_of_mut!(rc) as PRECT as LPPOINT, (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT);
            cx = rc.left + nXDifference;
            cy = rc.top + nYDifference;
            SetWindowPos(hApplicationPageNewTaskButton, core::ptr::null_mut(), cx, cy, 0, 0, SWP_NOACTIVATE|SWP_NOOWNERZORDER|SWP_NOSIZE|SWP_NOZORDER);
            InvalidateRect(hApplicationPageNewTaskButton, core::ptr::null(), (TRUE) as BOOL);
        }

        WM_NOTIFY => {
            ApplicationPageOnNotify(wParam, lParam);
        }

        WM_KEYDOWN => {
            if wParam == VK_DELETE as WPARAM {
                ProcessPage_OnEndProcess();
            }
        }

        _ => {}
    }

    return 0 as INT_PTR;
}

#[no_mangle]
pub unsafe extern "C" fn RefreshApplicationPage() {
    PostThreadMessage(dwApplicationThread, WM_TIMER as UINT, 0 as WPARAM, 0 as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn UpdateApplicationListControlViewSetting() {
    let mut dwStyle: DWORD = GetWindowLongPtrW(hApplicationPageListCtrl, GWL_STYLE) as DWORD;
    dwStyle &= !((LVS_REPORT | LVS_ICON | LVS_LIST | LVS_SMALLICON) as DWORD);
    if TaskManagerSettings.ViewMode == ID_VIEW_LARGE {
        dwStyle |= LVS_ICON as DWORD;
    } else if TaskManagerSettings.ViewMode == ID_VIEW_SMALL {
        dwStyle |= LVS_SMALLICON as DWORD;
    } else if TaskManagerSettings.ViewMode == ID_VIEW_DETAILS {
        dwStyle |= LVS_REPORT as DWORD;
    }
    SetWindowLongPtrW(hApplicationPageListCtrl, GWL_STYLE, dwStyle as LONG_PTR);

    RefreshApplicationPage();
}
#[no_mangle]
pub unsafe extern "system" fn ApplicationPageRefreshThread(lpParameter: *mut core::ffi::c_void) -> DWORD {
    let mut msg: MSG = core::mem::zeroed();
    let mut i: INT = 0;
    let mut bItemRemoved: BOOL = FALSE as BOOL;
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut hImageListLarge: HIMAGELIST = core::ptr::null_mut();
    let mut hImageListSmall: HIMAGELIST = core::ptr::null_mut();

    /* If we couldn't create the event then exit the thread */
    while 1 != 0 {
        /*  Wait for an the event or application close */
        if GetRefreshMessage(core::ptr::addr_of_mut!(msg)) == 0 {
            return 0 as DWORD;
        }

        if msg.message == WM_TIMER as UINT {
            // FIXME: Should this be EnumDesktopWindows() instead?
            noApps = TRUE as BOOL;
            EnumWindows(
                Some(EnumWindowsProc),
                0 as LPARAM,
            );
            if noApps != 0 {
                let _ = ListView_DeleteAllItems!(hApplicationPageListCtrl);
                bApplicationPageSelectionMade = FALSE as BOOL;
            }

            /* Get the image lists */
            hImageListLarge = ListView_GetImageList!(hApplicationPageListCtrl, LVSIL_NORMAL as UINT);
            hImageListSmall = ListView_GetImageList!(hApplicationPageListCtrl, LVSIL_SMALL as UINT);

            /* Check to see if we need to remove any items from the list */
            i = (ListView_GetItemCount!(hApplicationPageListCtrl) - 1 as INT) as INT;
            while i >= 0 as INT {
                memset(
                    core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
                    0 as core::ffi::c_int,
                    core::mem::size_of::<LV_ITEM>() as size_t,
                );
                item.mask = (LVIF_IMAGE | LVIF_PARAM) as UINT;
                item.iItem = i as core::ffi::c_int;
                let _ = ListView_GetItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));

                pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
                if IsWindow((*pAPLI).hWnd) == 0
                    || (wcslen(core::ptr::addr_of_mut!((*pAPLI).szTitle) as *mut WCHAR)
                        <= 0 as size_t)
                    || IsWindowVisible((*pAPLI).hWnd) == 0
                    || !GetParent((*pAPLI).hWnd).is_null()
                    || !GetWindow((*pAPLI).hWnd, GW_OWNER as UINT).is_null()
                    || ((GetWindowLongPtr((*pAPLI).hWnd, GWL_EXSTYLE as core::ffi::c_int)
                        & (WS_EX_TOOLWINDOW as LONG_PTR))
                        != 0)
                {
                    ImageList_Remove(hImageListLarge, item.iItem);
                    ImageList_Remove(hImageListSmall, item.iItem);

                    let _ = ListView_DeleteItem!(hApplicationPageListCtrl, item.iItem);
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        pAPLI as *mut core::ffi::c_void,
                    );
                    bItemRemoved = TRUE as BOOL;
                }
                i -= 1;
            }

            /*
             * If an item was removed from the list then
             * we need to resync all the items with the
             * image list
             */
            if bItemRemoved != 0 {
                i = 0 as INT;
                while i < ListView_GetItemCount!(hApplicationPageListCtrl) {
                    memset(
                        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
                        0 as core::ffi::c_int,
                        core::mem::size_of::<LV_ITEM>() as size_t,
                    );
                    item.mask = LVIF_IMAGE as UINT;
                    item.iItem = i as core::ffi::c_int;
                    item.iImage = i as core::ffi::c_int;
                    let _ = ListView_SetItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));
                    i += 1;
                }
                bItemRemoved = FALSE as BOOL;
            }

            ApplicationPageUpdate();
        }
    }
    0 as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn EnumWindowsProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    let mut hIcon: HICON;
    let mut szText: [WCHAR; 260] = [0; 260];
    let mut bLargeIcon: BOOL;
    let mut bHung: BOOL = (FALSE) as i32;
    let mut bAlive: LRESULT;

    type IsHungAppWindowProc = Option<unsafe extern "system" fn(HWND) -> i32>;
    let mut IsHungAppWindow: IsHungAppWindowProc;

    /* Skip our window */
    if hWnd == hMainWnd {
        return (TRUE) as BOOL;
    }

    bLargeIcon = (TaskManagerSettings.ViewMode == ID_VIEW_LARGE) as BOOL;

    GetWindowTextW(
        hWnd,
        core::ptr::addr_of_mut!(szText) as LPWSTR,
        260 as i32,
    ); /* Get the window text */

    /* Check and see if this is a top-level app window */
    if (wcslen(core::ptr::addr_of_mut!(szText) as *mut WCHAR)
        <= 0)
        || (IsWindowVisible(hWnd) == 0)
        || (!GetParent(hWnd).is_null())
        || (!GetWindow(hWnd, GW_OWNER as UINT).is_null())
        || ((GetWindowLongPtrW(hWnd, GWL_EXSTYLE as i32)
            & (WS_EX_TOOLWINDOW as LONG_PTR))
            != 0)
    {
        return (TRUE) as BOOL; /* Skip this window */
    }

    noApps = (FALSE) as i32;

    /* Get the icon for this window (GET_ICON expanded inline) */
    hIcon = core::ptr::null_mut();
    bAlive = SendMessageTimeoutW(
        hWnd,
        0x007F, /* WM_GETICON */
        (if bLargeIcon != 0 {
            ICON_BIG
        } else {
            ICON_SMALL
        }) as WPARAM,
        0,
        0x0002, /* SMTO_ABORTIFHUNG */
        100,
        core::ptr::addr_of_mut!(hIcon) as PDWORD_PTR,
    );
    if hIcon.is_null() {
        /* We failed, try to retrieve other icons... */
        if hIcon.is_null() && bAlive != 0 {
            SendMessageTimeoutW(
                hWnd,
                0x007F, /* WM_GETICON */
                (if bLargeIcon != 0 {
                    ICON_SMALL
                } else {
                    ICON_BIG
                }) as WPARAM,
                0,
                0x0002, /* SMTO_ABORTIFHUNG */
                100,
                core::ptr::addr_of_mut!(hIcon) as PDWORD_PTR,
            );
        }
        if hIcon.is_null() {
            hIcon = (GetClassLongPtrW(
                hWnd,
                (if bLargeIcon != 0 {
                    GCL_HICON
                } else {
                    GCL_HICONSM
                }) as i32,
            ) as LONG_PTR) as HICON;
        }
        if hIcon.is_null() {
            hIcon = (GetClassLongPtrW(
                hWnd,
                (if bLargeIcon != 0 {
                    GCL_HICONSM
                } else {
                    GCL_HICON
                }) as i32,
            ) as LONG_PTR) as HICON;
        }

        /* If we still do not have any icon, load the default one */
        if hIcon.is_null() {
            hIcon = LoadIconW(
                hInst,
                (if bLargeIcon != 0 {
                    MAKEINTRESOURCEW!(IDI_WINDOW)
                } else {
                    MAKEINTRESOURCEW!(IDI_WINDOWSM)
                }) as LPCWSTR,
            );
        }
    }

    bHung = (FALSE) as i32;

    IsHungAppWindow = ::core::mem::transmute::<FARPROC, IsHungAppWindowProc>(
        GetProcAddress(
            GetModuleHandleW(RUSTCONV_WSTR_1C71252208A9.as_ptr()),
            b"IsHungAppWindow\0".as_ptr() as LPCSTR,
        ),
    );

    if let Some(IsHungAppWindow) = IsHungAppWindow {
        bHung = IsHungAppWindow(hWnd) as BOOL;
    }

    AddOrUpdateHwnd(
        hWnd,
        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
        hIcon,
        bHung,
    );

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AddOrUpdateHwnd(hWnd: HWND, szTitle: *mut WCHAR, hIcon: HICON, bHung: BOOL) {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut hImageListLarge: HIMAGELIST;
    let mut hImageListSmall: HIMAGELIST;
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: i32;
    let mut bAlreadyInList: BOOL = (FALSE) as i32;

    memset(
        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<LV_ITEM>() as usize,
    );

    /* Get the image lists */
    hImageListLarge = ListView_GetImageList!(hApplicationPageListCtrl, LVSIL_NORMAL) as HIMAGELIST;
    hImageListSmall = ListView_GetImageList!(hApplicationPageListCtrl, LVSIL_SMALL) as HIMAGELIST;

    /* Check to see if it's already in our list */
    i = 0;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) {
        memset(
            core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<LV_ITEM>() as usize,
        );
        item.mask = LVIF_IMAGE | LVIF_PARAM;
        item.iItem = i;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));

        pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
        if (*pAPLI).hWnd == hWnd {
            bAlreadyInList = (TRUE) as i32;
            break;
        }
        i += 1;
    }

    /* If it is already in the list then update it if necessary */
    if bAlreadyInList != 0 {
        /* Check to see if anything needs updating */
        if ((*pAPLI).hIcon != hIcon)
            || (_wcsicmp(
                core::ptr::addr_of_mut!((*pAPLI).szTitle) as *mut WCHAR,
                szTitle,
            ) != 0)
            || ((*pAPLI).bHung != bHung)
        {
            /* Update the structure */
            (*pAPLI).hIcon = hIcon;
            (*pAPLI).bHung = bHung;
            wcscpy(
                core::ptr::addr_of_mut!((*pAPLI).szTitle) as *mut WCHAR,
                szTitle,
            );

            /* Update the image list */
            ImageList_ReplaceIcon(hImageListLarge, item.iItem, hIcon);
            ImageList_ReplaceIcon(hImageListSmall, item.iItem, hIcon);

            /* Update the list view */
            let _ = ListView_RedrawItems!(
                hApplicationPageListCtrl,
                0,
                ListView_GetItemCount!(hApplicationPageListCtrl),
            );
            /* UpdateWindow(hApplicationPageListCtrl); */
            InvalidateRect(hApplicationPageListCtrl, core::ptr::null_mut(), 0);
        }
    } else {
        // It is not already in the list so add it
        pAPLI = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<APPLICATION_PAGE_LIST_ITEM>()) as SIZE_T,
        ) as LPAPPLICATION_PAGE_LIST_ITEM;

        (*pAPLI).hWnd = hWnd;
        (*pAPLI).hIcon = hIcon;
        (*pAPLI).bHung = bHung;
        wcscpy(
            core::ptr::addr_of_mut!((*pAPLI).szTitle) as *mut WCHAR,
            szTitle,
        );

        /* Add the item to the list */
        memset(
            core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<LV_ITEM>() as usize,
        );
        item.mask = LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM;
        ImageList_AddIcon!(hImageListLarge, hIcon);
        item.iImage = ImageList_AddIcon!(hImageListSmall, hIcon);
        item.pszText = LPSTR_TEXTCALLBACK;
        item.iItem = ListView_GetItemCount!(hApplicationPageListCtrl);
        item.lParam = pAPLI as LPARAM;
        let _ = ListView_InsertItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));
    }

    /* Select first item if any */
    if (ListView_GetNextItem!(
        hApplicationPageListCtrl,
        -1,
        LVNI_FOCUSED | LVNI_SELECTED,
    ) == -1)
        && (ListView_GetItemCount!(hApplicationPageListCtrl) > 0)
        && (bApplicationPageSelectionMade == 0)
    {
        ListView_SetItemState!(
            hApplicationPageListCtrl,
            0,
            LVIS_FOCUSED | LVIS_SELECTED,
            LVIS_FOCUSED | LVIS_SELECTED,
        );
        bApplicationPageSelectionMade = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPageUpdate() {
    /* Enable or disable the "End Task" & "Switch To" buttons */
    if ListView_GetSelectedCount!(hApplicationPageListCtrl) != 0 {
        EnableWindow(hApplicationPageEndTaskButton, (TRUE) as BOOL);
    } else {
        EnableWindow(hApplicationPageEndTaskButton, (FALSE) as BOOL);
    }

    /* Enable "Switch To" button only if only one app is selected */
    EnableWindow(
        hApplicationPageSwitchToButton,
        (ListView_GetSelectedCount!(hApplicationPageListCtrl) == 1) as i32,
    );

    /* If we are on the applications tab the windows menu will be */
    /* present on the menu bar so enable & disable the menu items */
    if TabCtrl_GetCurSel!(hTabWnd) == 0 {
        let mut hMenu: HMENU = core::ptr::null_mut();
        let mut hWindowsMenu: HMENU = core::ptr::null_mut();

        hMenu = GetMenu(hMainWnd);
        hWindowsMenu = GetSubMenu(hMenu, 3);

        if ListView_GetSelectedCount!(hApplicationPageListCtrl) == 1 {
            // Only one item selected
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_TILEHORIZONTALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_TILEVERTICALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_MINIMIZE, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_MAXIMIZE, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_CASCADE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_BRINGTOFRONT, MF_BYCOMMAND | MF_ENABLED);
        } else if ListView_GetSelectedCount!(hApplicationPageListCtrl) > 1 {
            // More than one item selected
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_TILEHORIZONTALLY, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_TILEVERTICALLY, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_MINIMIZE, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_MAXIMIZE, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_CASCADE, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_BRINGTOFRONT, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        } else {
            // No items selected
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_TILEHORIZONTALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_TILEVERTICALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_MINIMIZE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_MAXIMIZE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_CASCADE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
            EnableMenuItem(hWindowsMenu, ID_WINDOWS_BRINGTOFRONT, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPageOnNotify(mut wParam: WPARAM, mut lParam: LPARAM) {
    let mut pnmh: LPNMHDR = core::ptr::null_mut();
    let mut pnmdi: *mut LV_DISPINFO = core::ptr::null_mut();
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut szMsg: [WCHAR; 256] = [0; 256];

    pnmh = lParam as LPNMHDR;
    pnmdi = lParam as *mut LV_DISPINFO;

    if (*pnmh).hwndFrom == hApplicationPageListCtrl {
        match (((*pnmh).code) as i32) as u32 {
            __c if __c == LVN_ITEMCHANGED as u32 => {
                ApplicationPageUpdate();
            }
            LVN_GETDISPINFO => {
                pAPLI = (*pnmdi).item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;

                /* Update the item text */
                if (*pnmdi).item.iSubItem == 0 {
                    wcsncpy(
                        (*pnmdi).item.pszText,
                        core::ptr::addr_of_mut!((*pAPLI).szTitle) as *mut WCHAR,
                        (*pnmdi).item.cchTextMax as usize,
                    );
                }
                /* Update the item status */
                else if (*pnmdi).item.iSubItem == 1 {
                    if (*pAPLI).bHung != 0 {
                        LoadStringW(
                            GetModuleHandleW(core::ptr::null::<WCHAR>()) as HINSTANCE,
                            IDS_NOT_RESPONDING as UINT,
                            core::ptr::addr_of_mut!(szMsg) as LPWSTR,
                            _countof!(szMsg) as core::ffi::c_int,
                        );
                    } else {
                        LoadStringW(
                            GetModuleHandleW(core::ptr::null::<WCHAR>()) as HINSTANCE,
                            IDS_RUNNING as UINT,
                            core::ptr::addr_of_mut!(szMsg) as LPWSTR,
                            _countof!(szMsg) as core::ffi::c_int,
                        );
                    }
                    wcsncpy(
                        (*pnmdi).item.pszText,
                        core::ptr::addr_of_mut!(szMsg) as *mut WCHAR,
                        (*pnmdi).item.cchTextMax as usize,
                    );
                }

                // break
            }
            NM_RCLICK => {
                if ListView_GetSelectedCount!(hApplicationPageListCtrl) < 1 {
                    ApplicationPageShowContextMenu1();
                } else {
                    ApplicationPageShowContextMenu2();
                }
            }
            NM_DBLCLK => {
                ApplicationPage_OnSwitchTo();
            }
            __c if __c == LVN_KEYDOWN as u32 => {
                if ((*(lParam as LPNMLVKEYDOWN)).wVKey as core::ffi::c_int) as u32 == VK_DELETE {
                    ApplicationPage_OnEndTask();
                }
            }
            _ => {}
        }
    } else if (*pnmh).hwndFrom == ListView_GetHeader!(hApplicationPageListCtrl) {
        match (*pnmh).code {
            NM_RCLICK => {
                if ListView_GetSelectedCount!(hApplicationPageListCtrl) < 1 {
                    ApplicationPageShowContextMenu1();
                } else {
                    ApplicationPageShowContextMenu2();
                }
            }
            HDN_ITEMCLICK => {
                let _ = ListView_SortItems!(
                    hApplicationPageListCtrl,
                    Some(ApplicationPageCompareFunc),
                    0,
                );
                bSortAscending = (bSortAscending == 0) as core::ffi::c_int as BOOL;
            }
            _ => {}
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPageShowContextMenu1() {
    let mut hMenu: HMENU;
    let mut hSubMenu: HMENU;
    let mut pt: POINT = core::mem::zeroed::<POINT>();

    GetCursorPos(core::ptr::addr_of_mut!(pt));

    hMenu = LoadMenuW(hInst, MAKEINTRESOURCEW!(IDR_APPLICATION_PAGE_CONTEXT1));
    hSubMenu = GetSubMenu(hMenu, 0);

    CheckMenuRadioItem(hSubMenu, ID_VIEW_LARGE, ID_VIEW_DETAILS, TaskManagerSettings.ViewMode, MF_BYCOMMAND);

    TrackPopupMenu(hSubMenu, TPM_LEFTALIGN | TPM_TOPALIGN | TPM_LEFTBUTTON, pt.x, pt.y, 0, hMainWnd, core::ptr::null_mut());

    DestroyMenu(hMenu);
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPageShowContextMenu2() {
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut hSubMenu: HMENU = core::ptr::null_mut();
    let mut pt: POINT = core::mem::zeroed();

    GetCursorPos(core::ptr::addr_of_mut!(pt));

    hMenu = LoadMenuW(hInst, MAKEINTRESOURCEW!(IDR_APPLICATION_PAGE_CONTEXT2));
    hSubMenu = GetSubMenu(hMenu, 0);

    if ListView_GetSelectedCount!(hApplicationPageListCtrl) == 1 {
        EnableMenuItem(hSubMenu, ID_APPLICATION_PAGE_SWITCHTO, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_TILEHORIZONTALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_TILEVERTICALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_MINIMIZE, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_MAXIMIZE, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_CASCADE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_BRINGTOFRONT, MF_BYCOMMAND | MF_ENABLED);
    } else if ListView_GetSelectedCount!(hApplicationPageListCtrl) > 1 {
        EnableMenuItem(hSubMenu, ID_APPLICATION_PAGE_SWITCHTO, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_TILEHORIZONTALLY, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_TILEVERTICALLY, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_MINIMIZE, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_MAXIMIZE, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_CASCADE, MF_BYCOMMAND | MF_ENABLED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_BRINGTOFRONT, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
    } else {
        EnableMenuItem(hSubMenu, ID_APPLICATION_PAGE_SWITCHTO, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_TILEHORIZONTALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_TILEVERTICALLY, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_MINIMIZE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_MAXIMIZE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_CASCADE, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
        EnableMenuItem(hSubMenu, ID_WINDOWS_BRINGTOFRONT, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
    }

    SetMenuDefaultItem(hSubMenu, ID_APPLICATION_PAGE_SWITCHTO, MF_BYCOMMAND);

    TrackPopupMenu(hSubMenu, TPM_LEFTALIGN | TPM_TOPALIGN | TPM_LEFTBUTTON, pt.x, pt.y, 0, hMainWnd, core::ptr::null_mut());

    DestroyMenu(hMenu);
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnView(dwMode: DWORD) {
    let mut hMenu: HMENU;
    let mut hViewMenu: HMENU;

    hMenu = GetMenu(hMainWnd);
    hViewMenu = GetSubMenu(hMenu, 2);

    TaskManagerSettings.ViewMode = dwMode;
    CheckMenuRadioItem(hViewMenu, ID_VIEW_LARGE, ID_VIEW_DETAILS, dwMode, MF_BYCOMMAND);

    UpdateApplicationListControlViewSetting();
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnWindowsTile(mut dwMode: DWORD) {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: core::ffi::c_int = 0;
    let mut hWndArray: *mut HWND = core::ptr::null_mut();
    let mut nWndCount: core::ffi::c_int = 0;

    hWndArray = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::size_of::<HWND>() as SIZE_T).wrapping_mul(
            ListView_GetItemCount!(hApplicationPageListCtrl) as core::ffi::c_int as SIZE_T,
        ),
    ) as *mut HWND;
    nWndCount = 0 as core::ffi::c_int;

    i = 0 as core::ffi::c_int;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) as core::ffi::c_int {
        memset(
            &raw mut item as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<LV_ITEM>() as size_t,
        );
        item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.stateMask = -(1 as core::ffi::c_int) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, &raw mut item);

        if item.state & LVIS_SELECTED as UINT != 0 {
            pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
            if !pAPLI.is_null() {
                let ref mut fresh31 = *hWndArray.offset(nWndCount as isize);
                *fresh31 = (*pAPLI).hWnd;
                nWndCount += 1;
            }
        }
        i += 1;
    }

    TileWindows(
        core::ptr::null_mut(),
        dwMode as UINT,
        core::ptr::null(),
        nWndCount as UINT,
        hWndArray,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        hWndArray as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnWindowsMinimize() {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: core::ffi::c_int;

    i = 0 as core::ffi::c_int;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) {
        memset(
            &raw mut item as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<LV_ITEM>() as size_t,
        );
        item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.stateMask = (-(1 as core::ffi::c_int)) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, &raw mut item);
        if (item.state & LVIS_SELECTED as UINT) != 0 {
            pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
            if !pAPLI.is_null() {
                ShowWindowAsync((*pAPLI).hWnd, (SW_MINIMIZE) as i32);
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnWindowsMaximize() {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: core::ffi::c_int;

    i = 0 as core::ffi::c_int;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) {
        memset(
            &raw mut item as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<LV_ITEM>(),
        );
        item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.stateMask = (-(1 as core::ffi::c_int)) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, &raw mut item);
        if (item.state & LVIS_SELECTED as UINT) != 0 {
            pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
            if !pAPLI.is_null() {
                ShowWindowAsync((*pAPLI).hWnd, (SW_MAXIMIZE) as i32);
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnWindowsCascade() {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: c_int;
    let mut hWndArray: *mut HWND;
    let mut nWndCount: c_int;

    hWndArray = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::size_of::<HWND>() as SIZE_T).wrapping_mul(
            ListView_GetItemCount!(hApplicationPageListCtrl) as c_int as SIZE_T,
        ),
    ) as *mut HWND;
    nWndCount = 0 as c_int;

    i = 0 as c_int;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) as c_int {
        memset(
            core::ptr::addr_of_mut!(item) as *mut c_void,
            0 as c_int,
            core::mem::size_of::<LV_ITEM>() as size_t,
        );
        item.mask = ((LVIF_STATE as UINT) | (LVIF_PARAM as UINT)) as UINT;
        item.iItem = i;
        item.stateMask = (-(1 as i32)) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));
        if (item.state & (LVIS_SELECTED as UINT)) != 0 {
            pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
            if !pAPLI.is_null() {
                *hWndArray.offset(nWndCount as isize) = (*pAPLI).hWnd;
                nWndCount += 1;
            }
        }
        i += 1;
    }
    CascadeWindows(
        core::ptr::null_mut(),
        0 as UINT,
        core::ptr::null(),
        nWndCount as UINT,
        hWndArray,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        hWndArray as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnWindowsBringToFront() {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: ::core::ffi::c_int = 0;

    i = 0 as ::core::ffi::c_int;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) as ::core::ffi::c_int {
        memset(
            core::ptr::addr_of_mut!(item) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<LV_ITEM>() as size_t,
        );
        item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.stateMask = (-(1 as ::core::ffi::c_int)) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));
        if (item.state & (LVIS_SELECTED as UINT)) != 0 {
            pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
            break;
        }
        i += 1;
    }
    if !pAPLI.is_null() {
        SwitchToThisWindow((*pAPLI).hWnd, TRUE as BOOL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnSwitchTo() {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: core::ffi::c_int;

    i = 0;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) {
        memset(
            core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<LV_ITEM>(),
        );
        item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.stateMask = (-(1) as core::ffi::c_int) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, core::ptr::addr_of_mut!(item));

        if (item.state & LVIS_SELECTED as UINT) != 0 {
            pAPLI = item.lParam as usize as LPAPPLICATION_PAGE_LIST_ITEM;
            break;
        }
        i += 1;
    }
    if !pAPLI.is_null() {
        SwitchToThisWindow((*pAPLI).hWnd, (TRUE) as BOOL);
        if TaskManagerSettings.MinimizeOnUse != 0 {
            ShowWindowAsync(hMainWnd, (SW_MINIMIZE) as i32);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnEndTask() {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: core::ffi::c_int;

    /* Trick: on Windows, pressing the CTRL key forces the task to be ended */
    let ForceEndTask: BOOL = ((((GetKeyState(VK_CONTROL as core::ffi::c_int) as core::ffi::c_int)
        & (0x8000 as core::ffi::c_int))
        != 0) as BOOL);

    i = 0 as core::ffi::c_int;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) {
        memset(
            &raw mut item as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<LV_ITEM>() as size_t,
        );
        item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.stateMask = (-(1 as core::ffi::c_int)) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, &raw mut item);
        if (item.state & (LVIS_SELECTED as UINT)) != 0 {
            pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
            if !pAPLI.is_null() {
                EndTask((*pAPLI).hWnd, 0 as BOOL, ForceEndTask);
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ApplicationPage_OnGotoProcess() {
    let mut pAPLI: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut i: c_int;

    i = 0;
    while i < ListView_GetItemCount!(hApplicationPageListCtrl) {
        memset(&raw mut item as *mut c_void, 0, core::mem::size_of::<LV_ITEM>());
        item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.stateMask = (-(1 as c_int)) as UINT;
        let _ = ListView_GetItem!(hApplicationPageListCtrl, &raw mut item);
        if (item.state & (LVIS_SELECTED as UINT)) != 0 {
            pAPLI = item.lParam as LPAPPLICATION_PAGE_LIST_ITEM;
            break;
        }
        i = i + 1;
    }
    if !pAPLI.is_null() {
        let mut dwProcessId: DWORD = 0;

        GetWindowThreadProcessId((*pAPLI).hWnd, &raw mut dwProcessId);
        /*
         * Switch to the process tab
         */
        TabCtrl_SetCurFocus!(hTabWnd, 1);
        /*
         * Select the process item in the list
         */
        i = ProcGetIndexByProcessId(dwProcessId);
        if i != -(1) {
            ListView_SetItemState!(
                hProcessPageListCtrl,
                i,
                (LVIS_SELECTED | LVIS_FOCUSED),
                (LVIS_SELECTED | LVIS_FOCUSED),
            );
            let _ = ListView_EnsureVisible!(hProcessPageListCtrl, i, FALSE);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn ApplicationPageCompareFunc(lParam1: LPARAM, lParam2: LPARAM, lParamSort: LPARAM) -> i32 {
    let mut Param1: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    let mut Param2: LPAPPLICATION_PAGE_LIST_ITEM = core::ptr::null_mut();
    if bSortAscending != 0 {
        Param1 = lParam1 as LPAPPLICATION_PAGE_LIST_ITEM;
        Param2 = lParam2 as LPAPPLICATION_PAGE_LIST_ITEM;
    } else {
        Param1 = lParam2 as LPAPPLICATION_PAGE_LIST_ITEM;
        Param2 = lParam1 as LPAPPLICATION_PAGE_LIST_ITEM;
    }
    return wcscmp(
        core::ptr::addr_of_mut!((*Param1).szTitle) as *mut WCHAR,
        core::ptr::addr_of_mut!((*Param2).szTitle) as *mut WCHAR,
    );
}

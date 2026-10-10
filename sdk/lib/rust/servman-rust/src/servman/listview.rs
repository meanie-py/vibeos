use crate::servman::misc::AllocAndLoadString;
use crate::servman::query::GetSelectedService;
use crate::servman::query::GetServiceConfig;
use crate::servman::query::GetServiceDescription;
use crate::servman::query::GetServiceList;
use crate::servman::mainwnd::UpdateServiceCount;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_351CAA9D073B: [u16; 28] = [
    0x0043u16, 0x006F, 0x0075, 0x006C, 0x0064, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020,
    0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x004C, 0x0069, 0x0073,
    0x0074, 0x0020, 0x0056, 0x0069, 0x0065, 0x0077, 0x002E, 0x0000,
];
pub static RUSTCONV_WSTR_6C49BEACD7CB: [u16; 6] = [0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_LISTVIEWW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"S\0y\0s\0L\0i\0s\0t\0V\0i\0e\0w\x003\x002\0\0\0",
    )
};

static mut Columns: [COLUMN_LIST; 5] = [
    _COLUMN_LIST {
        iSubItem: (LVNAME as i32),
        cx: 150 as ::core::ffi::c_int,
        idsText: IDS_FIRSTCOLUMN as UINT,
    },
    _COLUMN_LIST {
        iSubItem: (LVDESC as i32),
        cx: 240 as ::core::ffi::c_int,
        idsText: IDS_SECONDCOLUMN as UINT,
    },
    _COLUMN_LIST {
        iSubItem: (LVSTATUS as i32),
        cx: 55 as ::core::ffi::c_int,
        idsText: IDS_THIRDCOLUMN as UINT,
    },
    _COLUMN_LIST {
        iSubItem: (LVSTARTUP as i32),
        cx: 80 as ::core::ffi::c_int,
        idsText: IDS_FOURTHCOLUMN as UINT,
    },
    _COLUMN_LIST {
        iSubItem: (LVLOGONAS as i32),
        cx: 100 as ::core::ffi::c_int,
        idsText: IDS_FITHCOLUMN as UINT,
    },
];

pub type PCOLUMN_LIST = *mut COLUMN_LIST;

// LVNAME, LVDESC, LVSTATUS, LVSTARTUP, LVLOGONAS (i32) and
// IDS_FIRSTCOLUMN .. IDS_FITHCOLUMN (UINT) are defined in the header,
// not in this preamble; the caller must provide them.
#[no_mangle]
pub unsafe extern "C" fn SetListViewStyle(hListView: HWND, View: DWORD) {
    let mut Style: DWORD = GetWindowLongPtr(hListView, GWL_STYLE) as DWORD;

    if (Style & LVS_TYPEMASK as DWORD) != View {
        SetWindowLongPtr(
            hListView,
            GWL_STYLE,
            ((Style & !(LVS_TYPEMASK as DWORD)) | View) as LONG_PTR,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ListViewSelectionChanged(Info: PMAIN_WND_INFO, pnmv: LPNMLISTVIEW) {
    let mut hMainMenu: HMENU = core::ptr::null_mut();

    /* get handle to menu */
    hMainMenu = GetMenu((*Info).hMainWnd);

    /* activate properties menu item, if not already */
    if GetMenuState(
        hMainMenu,
        ID_PROP as UINT,
        MF_BYCOMMAND as UINT,
    ) != MF_ENABLED as UINT
    {
        EnableMenuItem(
            hMainMenu,
            ID_PROP as UINT,
            MF_ENABLED as UINT,
        );
        EnableMenuItem(
            GetSubMenu((*Info).hShortcutMenu, 0 as core::ffi::c_int),
            ID_PROP as UINT,
            MF_ENABLED as UINT,
        );
        SetMenuDefaultItem(
            GetSubMenu((*Info).hShortcutMenu, 0 as core::ffi::c_int),
            ID_PROP as UINT,
            MF_BYCOMMAND as UINT,
        );
    }

    /* activate delete menu item, if not already */
    if GetMenuState(
        hMainMenu,
        ID_DELETE as UINT,
        MF_BYCOMMAND as UINT,
    ) != MF_ENABLED as UINT
    {
        EnableMenuItem(
            hMainMenu,
            ID_DELETE as UINT,
            MF_ENABLED as UINT,
        );
        EnableMenuItem(
            GetSubMenu((*Info).hShortcutMenu, 0 as core::ffi::c_int),
            ID_DELETE as UINT,
            MF_ENABLED as UINT,
        );
    }

    /* set selected service */
    (*Info).SelectedItem = (*pnmv).iItem as INT;

    /* get pointer to selected service */
    (*Info).pCurrentService = GetSelectedService(Info);

    /* set current selected service in the status bar */
    SendMessage(
        (*Info).hStatus,
        SB_SETTEXT as UINT,
        1 as WPARAM,
        (*(*Info).pCurrentService).lpDisplayName as LPARAM,
    );

    /* show the properties button */
    SendMessage(
        (*Info).hTool,
        TB_SETSTATE as UINT,
        ID_PROP as WPARAM,
        MAKELONG!(TBSTATE_ENABLED as WORD, 0 as WORD) as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ChangeListViewText(Info: PMAIN_WND_INFO, pService: *mut ENUM_SERVICE_STATUS_PROCESS, Column: UINT) {
    let mut lvfi: LVFINDINFOW = core::mem::zeroed();
    let mut lvItem: LVITEMW = core::mem::zeroed();
    let mut index: INT;

    lvfi.flags = LVFI_PARAM as UINT;
    lvfi.lParam = pService as LPARAM;
    index = ListView_FindItem!(
        (*Info).hListView,
        -(1 as INT),
        core::ptr::addr_of_mut!(lvfi),
    );
    if index != -(1 as INT) {
        lvItem.iItem = index;
        lvItem.iSubItem = Column as INT;

        match Column {
            LVNAME => {
                let mut lpServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();

                lpServiceConfig = GetServiceConfig((*pService).lpServiceName);
                if !lpServiceConfig.is_null() {
                    lvItem.pszText = (*lpServiceConfig).lpDisplayName;
                    SendMessage(
                        (*Info).hListView,
                        LVM_SETITEMTEXT as UINT,
                        lvItem.iItem as WPARAM,
                        core::ptr::addr_of_mut!(lvItem) as LPARAM,
                    );

                    HeapFree(
                        ProcessHeap,
                        0 as DWORD,
                        lpServiceConfig as LPVOID,
                    );
                }
            }

            LVDESC => {
                let mut lpDescription: LPWSTR;

                lpDescription = GetServiceDescription((*pService).lpServiceName);

                lvItem.pszText = lpDescription;
                SendMessage(
                    (*Info).hListView,
                    LVM_SETITEMTEXTW as UINT,
                    lvItem.iItem as WPARAM,
                    core::ptr::addr_of_mut!(lvItem) as LPARAM,
                );

                HeapFree(
                    ProcessHeap,
                    0 as DWORD,
                    lpDescription as LPVOID,
                );
            }

            LVSTATUS => {
                let mut szStatus: [WCHAR; 64] = [0; 64];

                if (*pService).ServiceStatusProcess.dwCurrentState == SERVICE_RUNNING as DWORD {
                    LoadStringW(
                        hInstance,
                        IDS_SERVICES_STARTED as UINT,
                        core::ptr::addr_of_mut!(szStatus) as LPWSTR,
                        (core::mem::size_of::<[WCHAR; 64]>()
                            / core::mem::size_of::<WCHAR>()) as INT,
                    );
                } else {
                    szStatus[0 as usize] = 0 as WCHAR;
                }

                lvItem.pszText = core::ptr::addr_of_mut!(szStatus) as *mut WCHAR as LPWSTR;
                SendMessageW(
                    (*Info).hListView,
                    LVM_SETITEMTEXT as UINT,
                    lvItem.iItem as WPARAM,
                    core::ptr::addr_of_mut!(lvItem) as LPARAM,
                );
            }

            LVSTARTUP => {
                let mut lpServiceConfig: LPQUERY_SERVICE_CONFIGW = core::ptr::null_mut();
                let mut lpStartup: LPWSTR = core::ptr::null_mut();
                let mut StringId: DWORD = 0 as DWORD;

                lpServiceConfig = GetServiceConfig((*pService).lpServiceName)
                    as LPQUERY_SERVICE_CONFIGW;

                if !lpServiceConfig.is_null() {
                    match (*lpServiceConfig).dwStartType {
                        2 => {
                            StringId = IDS_SERVICES_AUTO as DWORD;
                        }
                        3 => {
                            StringId = IDS_SERVICES_MAN as DWORD;
                        }
                        4 => {
                            StringId = IDS_SERVICES_DIS as DWORD;
                        }
                        _ => {}
                    }
                }

                if StringId != 0 {
                    AllocAndLoadString(
                        core::ptr::addr_of_mut!(lpStartup),
                        hInstance,
                        StringId as UINT,
                    );
                }

                lvItem.pszText = lpStartup;
                SendMessageW(
                    (*Info).hListView,
                    LVM_SETITEMTEXTW as UINT,
                    lvItem.iItem as WPARAM,
                    core::ptr::addr_of_mut!(lvItem) as LPARAM,
                );

                LocalFree(lpStartup as HLOCAL);
                HeapFree(
                    ProcessHeap,
                    0 as DWORD,
                    lpServiceConfig as LPVOID,
                );
            }

            LVLOGONAS => {
                let mut lpServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();

                lpServiceConfig = GetServiceConfig((*pService).lpServiceName);
                if !lpServiceConfig.is_null() {
                    lvItem.pszText = (*lpServiceConfig).lpServiceStartName;
                    SendMessageW(
                        (*Info).hListView,
                        LVM_SETITEMTEXT as UINT,
                        lvItem.iItem as WPARAM,
                        core::ptr::addr_of_mut!(lvItem) as LPARAM,
                    );

                    HeapFree(
                        ProcessHeap,
                        0 as DWORD,
                        lpServiceConfig as LPVOID,
                    );
                }
            }

            _ => {}
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RefreshServiceList(mut Info: PMAIN_WND_INFO) -> BOOL {
    let mut pService: *mut ENUM_SERVICE_STATUS_PROCESS = core::ptr::null_mut();
    let mut lvItem: LVITEMW = core::mem::zeroed();
    let mut Index: DWORD = 0;

    SendMessage(
        (*Info).hListView,
        WM_SETREDRAW as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );

    let _ = ListView_DeleteAllItems!((*Info).hListView);

    if GetServiceList(Info) != 0 {
        Index = 0 as DWORD;
        while Index < (*Info).NumServices {
            let mut i: INT = 0;

            pService = (*Info).pAllServices.add(Index as usize);

            /* set the display name */
            ZeroMemory!(
                core::ptr::addr_of_mut!(lvItem) as *mut core::ffi::c_void,
                core::mem::size_of::<LVITEMW>(),
            );
            lvItem.mask = (LVIF_TEXT | LVIF_PARAM) as UINT;
            lvItem.pszText = (*pService).lpDisplayName;

            /* Add the service pointer */
            lvItem.lParam = pService as LPARAM;

            /* add it to the listview */
            lvItem.iItem = ListView_InsertItem!(
                (*Info).hListView,
                core::ptr::addr_of_mut!(lvItem),
            );

            /* fill out all the column data */
            i = LVDESC as INT;
            while i <= LVLOGONAS as INT {
                ChangeListViewText(Info, pService, i as UINT);
                i += 1;
            }
            Index = Index.wrapping_add(1);
        }

        UpdateServiceCount(Info);
    }

    /* turn redraw flag on. */
    SendMessageW(
        (*Info).hListView,
        WM_SETREDRAW as UINT,
        TRUE as WPARAM,
        0 as LPARAM,
    );

    return TRUE as BOOL;
}

unsafe extern "C" fn InitListViewImage(Info: PMAIN_WND_INFO) {
    let mut hSmIconItem: HICON;
    let mut hLgIconItem: HICON;
    let mut hSmall: HIMAGELIST;
    let mut hLarge: HIMAGELIST;

    hSmall = ImageList_Create(
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        (ILC_MASK | ILC_COLOR32) as UINT,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if !hSmall.is_null() {
        hSmIconItem = LoadImageW(
            hInstance,
            MAKEINTRESOURCE!(IDI_SM_ICON as WORD) as LPCWSTR,
            IMAGE_ICON as UINT,
            16 as ::core::ffi::c_int,
            16 as ::core::ffi::c_int,
            0 as UINT,
        ) as HICON;
        if !hSmIconItem.is_null() {
            ImageList_AddIcon!(hSmall, hSmIconItem);
            let _ = ListView_SetImageList!(
                (*Info).hListView,
                hSmall,
                LVSIL_SMALL as WPARAM,
            );

            DestroyIcon(hSmIconItem);
        }
    }

    hLarge = ImageList_Create(
        GetSystemMetrics((SM_CXICON) as i32),
        GetSystemMetrics((SM_CYICON) as i32),
        (ILC_MASK | ILC_COLOR32) as UINT,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if !hLarge.is_null() {
        hLgIconItem = LoadImageW(
            hInstance,
            MAKEINTRESOURCE!(IDI_SM_ICON as WORD) as LPCWSTR,
            IMAGE_ICON as UINT,
            32 as ::core::ffi::c_int,
            32 as ::core::ffi::c_int,
            0 as UINT,
        ) as HICON;
        if !hLgIconItem.is_null() {
            ImageList_AddIcon!(hLarge, hLgIconItem);
            let _ = ListView_SetImageList!(
                (*Info).hListView,
                hLarge,
                LVSIL_NORMAL as WPARAM,
            );
            DestroyIcon(hLgIconItem);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreateListView(Info: PMAIN_WND_INFO) -> BOOL {
    let mut lvc: LVCOLUMNW = core::mem::zeroed();
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut hdi: HDITEM = core::mem::zeroed();
    let mut i: core::ffi::c_int = 0;
    let mut n: core::ffi::c_int = 0;

    (*Info).hListView = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        (WC_LISTVIEWW).as_ptr(),
        core::ptr::null(),
        WS_CHILD | WS_VISIBLE | LVS_REPORT | WS_BORDER |
            LBS_NOTIFY | LVS_SORTASCENDING | LBS_NOREDRAW,
        0,
        0,
        0,
        0,
        (*Info).hMainWnd,
        IDC_SERVLIST as HMENU,
        hInstance,
        core::ptr::null_mut(),
    );
    if (*Info).hListView.is_null() {
        MessageBoxW(
            (*Info).hMainWnd,
            RUSTCONV_WSTR_351CAA9D073B.as_ptr(),
            RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
        return (FALSE) as BOOL;
    }

    (*Info).hHeader = ListView_GetHeader!((*Info).hListView);

    let _ = ListView_SetExtendedListViewStyle!(
        (*Info).hListView,
        LVS_EX_FULLROWSELECT | LVS_EX_HEADERDRAGDROP,
    );

    lvc.mask = LVCF_TEXT | LVCF_SUBITEM | LVCF_WIDTH | LVCF_FMT;
    lvc.fmt = (LVCFMT_LEFT as i32);
    lvc.pszText = core::ptr::addr_of_mut!(szTemp) as *mut WCHAR;

    /* Add columns to the list-view */
    n = 0;
    while (n as usize) < (core::mem::size_of::<[COLUMN_LIST; 5]>() / core::mem::size_of::<COLUMN_LIST>()) {
        lvc.iSubItem = Columns[n as usize].iSubItem;
        lvc.cx = Columns[n as usize].cx;

        LoadStringW(
            hInstance,
            Columns[n as usize].idsText,
            core::ptr::addr_of_mut!(szTemp) as *mut WCHAR,
            (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
        );

        i = ListView_InsertColumn!((*Info).hListView, Columns[n as usize].iSubItem, core::ptr::addr_of_mut!(lvc));

        hdi.mask = HDI_LPARAM;
        hdi.lParam = ORD_ASCENDING as LPARAM;
        let _ = Header_SetItem!((*Info).hHeader, i, core::ptr::addr_of_mut!(hdi));
        n += 1;
    }

    InitListViewImage(Info);

    /* check the details view menu item */
    CheckMenuRadioItem(
        GetMenu((*Info).hMainWnd),
        ID_VIEW_LARGE,
        ID_VIEW_DETAILS,
        ID_VIEW_DETAILS,
        MF_BYCOMMAND,
    );

    return (TRUE) as BOOL;
}

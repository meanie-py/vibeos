use crate::servman::misc::AllocAndLoadString;
use crate::servman::listview::ChangeListViewText;
use crate::servman::create::CreateDialogProc;
use crate::servman::listview::CreateListView;
use crate::servman::delete::DeleteDialogProc;
use crate::servman::misc::DisplayString;
use crate::servman::export::ExportFile;
use crate::servman::query::FreeServiceList;
use crate::servman::misc::GetError;
use crate::servman::query::GetServiceConfig;
use crate::servman::misc::InitImageList;
use crate::servman::listview::ListViewSelectionChanged;
use crate::servman::propsheet::OpenPropSheet;
use crate::servman::listview::RefreshServiceList;
use crate::servman::progress::RunActionWithProgress;
use crate::servman::listview::SetListViewStyle;
use crate::servman::misc::StatusBarLoadString;
use crate::servman::query::UpdateServiceStatus;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8DED1C60686E: [u16; 25] = [
    0x0065u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061,
    0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x006C, 0x0069, 0x0073, 0x0074, 0x0020,
    0x0076, 0x0069, 0x0065, 0x0077, 0x0000,
];
pub static RUSTCONV_WSTR_8F3A7D4C9443: [u16; 29] = [
    0x0048u16, 0x0065, 0x006C, 0x0070, 0x0020, 0x0069, 0x0073, 0x0020, 0x006E, 0x006F,
    0x0074, 0x0020, 0x0079, 0x0065, 0x0074, 0x0020, 0x0069, 0x006D, 0x0070, 0x006C,
    0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_BB541F34C7CE: [u16; 26] = [
    0x0065u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061,
    0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0073, 0x0074, 0x0061, 0x0074, 0x0075,
    0x0073, 0x0020, 0x0062, 0x0061, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_CBE6669ECA10: [u16; 23] = [
    0x0065u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061,
    0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0074, 0x006F, 0x006F, 0x006C, 0x0062,
    0x0061, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_E872F95E8F59: [u16; 6] = [0x004Eu16, 0x006F, 0x0074, 0x0065, 0x0021, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szMainWndClass: [WCHAR; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [WCHAR; 16]>(
        *b"S\0e\0r\0v\0M\0a\0n\0W\0n\0d\0C\0l\0a\0s\0s\0\0\0",
    )
};

static mut Buttons: [TBBUTTON; 11] = [
    _TBBUTTON {
        iBitmap: (TBICON_PROP as i32),
        idCommand: (ID_PROP as i32),
        fsState: TBSTATE_INDETERMINATE as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_REFRESH as i32),
        idCommand: (ID_REFRESH as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_EXPORT as i32),
        idCommand: (ID_EXPORT as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: 15 as ::core::ffi::c_int,
        idCommand: 0 as ::core::ffi::c_int,
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_SEP as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_CREATE as i32),
        idCommand: (ID_CREATE as i32),
        fsState: TBSTATE_INDETERMINATE as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_DELETE as i32),
        idCommand: (ID_DELETE as i32),
        fsState: TBSTATE_INDETERMINATE as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: 15 as ::core::ffi::c_int,
        idCommand: 0 as ::core::ffi::c_int,
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_SEP as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_START as i32),
        idCommand: (ID_START as i32),
        fsState: TBSTATE_INDETERMINATE as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_STOP as i32),
        idCommand: (ID_STOP as i32),
        fsState: TBSTATE_INDETERMINATE as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_PAUSE as i32),
        idCommand: (ID_PAUSE as i32),
        fsState: TBSTATE_INDETERMINATE as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_RESTART as i32),
        idCommand: (ID_RESTART as i32),
        fsState: TBSTATE_INDETERMINATE as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
];

static mut MainMenuHintTable: [MENU_HINT; 20] = [
    _MENU_HINT {
        CmdId: ID_EXPORT as WORD,
        HintId: IDS_HINT_EXPORT as UINT,
    },
    _MENU_HINT {
        CmdId: ID_EXIT as WORD,
        HintId: IDS_HINT_EXIT as UINT,
    },
    _MENU_HINT {
        CmdId: ID_CONNECT as WORD,
        HintId: IDS_HINT_CONNECT as UINT,
    },
    _MENU_HINT {
        CmdId: ID_START as WORD,
        HintId: IDS_HINT_START as UINT,
    },
    _MENU_HINT {
        CmdId: ID_STOP as WORD,
        HintId: IDS_HINT_STOP as UINT,
    },
    _MENU_HINT {
        CmdId: ID_PAUSE as WORD,
        HintId: IDS_HINT_PAUSE as UINT,
    },
    _MENU_HINT {
        CmdId: ID_RESUME as WORD,
        HintId: IDS_HINT_RESUME as UINT,
    },
    _MENU_HINT {
        CmdId: ID_RESTART as WORD,
        HintId: IDS_HINT_RESTART as UINT,
    },
    _MENU_HINT {
        CmdId: ID_REFRESH as WORD,
        HintId: IDS_HINT_REFRESH as UINT,
    },
    _MENU_HINT {
        CmdId: ID_EDIT as WORD,
        HintId: IDS_HINT_EDIT as UINT,
    },
    _MENU_HINT {
        CmdId: ID_CREATE as WORD,
        HintId: IDS_HINT_CREATE as UINT,
    },
    _MENU_HINT {
        CmdId: ID_DELETE as WORD,
        HintId: IDS_HINT_DELETE as UINT,
    },
    _MENU_HINT {
        CmdId: ID_PROP as WORD,
        HintId: IDS_HINT_PROP as UINT,
    },
    _MENU_HINT {
        CmdId: ID_VIEW_LARGE as WORD,
        HintId: IDS_HINT_LARGE as UINT,
    },
    _MENU_HINT {
        CmdId: ID_VIEW_SMALL as WORD,
        HintId: IDS_HINT_SMALL as UINT,
    },
    _MENU_HINT {
        CmdId: ID_VIEW_LIST as WORD,
        HintId: IDS_HINT_LIST as UINT,
    },
    _MENU_HINT {
        CmdId: ID_VIEW_DETAILS as WORD,
        HintId: IDS_HINT_DETAILS as UINT,
    },
    _MENU_HINT {
        CmdId: ID_VIEW_CUST as WORD,
        HintId: IDS_HINT_CUST as UINT,
    },
    _MENU_HINT {
        CmdId: ID_HELP as WORD,
        HintId: IDS_HINT_HELP as UINT,
    },
    _MENU_HINT {
        CmdId: ID_ABOUT as WORD,
        HintId: IDS_HINT_ABOUT as UINT,
    },
];

static mut SystemMenuHintTable: [MENU_HINT; 6] = [
    _MENU_HINT {
        CmdId: SC_RESTORE as WORD,
        HintId: IDS_HINT_SYS_RESTORE as UINT,
    },
    _MENU_HINT {
        CmdId: SC_MOVE as WORD,
        HintId: IDS_HINT_SYS_MOVE as UINT,
    },
    _MENU_HINT {
        CmdId: SC_SIZE as WORD,
        HintId: IDS_HINT_SYS_SIZE as UINT,
    },
    _MENU_HINT {
        CmdId: SC_MINIMIZE as WORD,
        HintId: IDS_HINT_SYS_MINIMIZE as UINT,
    },
    _MENU_HINT {
        CmdId: SC_MAXIMIZE as WORD,
        HintId: IDS_HINT_SYS_MAXIMIZE as UINT,
    },
    _MENU_HINT {
        CmdId: SC_CLOSE as WORD,
        HintId: IDS_HINT_SYS_CLOSE as UINT,
    },
];

// Field names of MENU_HINT assumed to be `id` and `ids` (C struct: {command, hint string id}).

unsafe extern "C" fn MainWndMenuHint(
    Info: PMAIN_WND_INFO,
    CmdId: WORD,
    mut HintArray: *const MENU_HINT,
    HintsCount: DWORD,
    DefHintId: UINT,
) -> BOOL {
    let mut Found: BOOL = (FALSE) as i32;
    let mut LastHint: *const MENU_HINT;
    let mut HintId: UINT = DefHintId;

    LastHint = HintArray.add(HintsCount as usize);
    while HintArray != LastHint {
        if (*HintArray).CmdId == CmdId {
            HintId = (*HintArray).HintId;
            Found = (TRUE) as i32;
            break;
        }
        HintArray = HintArray.add(1);
    }

    StatusBarLoadString(
        (*Info).hStatus,
        (SB_SIMPLEID) as INT,
        hInstance,
        HintId,
    );

    return Found;
}

unsafe extern "C" fn UpdateMainStatusBar(mut Info: PMAIN_WND_INFO) {
    if !(*Info).hStatus.is_null() {
        SendMessage(
            (*Info).hStatus,
            SB_SIMPLE as UINT,
            (*Info).bInMenuLoop as WPARAM,
            0 as LPARAM,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn UpdateServiceCount(Info: PMAIN_WND_INFO) {
    let mut lpNumServices: LPWSTR = core::ptr::null_mut();
    if AllocAndLoadString(
        core::ptr::addr_of_mut!(lpNumServices),
        hInstance,
        IDS_NUM_SERVICES,
    ) != 0
    {
        let mut szNumServices: [WCHAR; 32] = [0; 32];

        let NumListedServ: INT = ListView_GetItemCount!((*Info).hListView);

        _snwprintf(
            core::ptr::addr_of_mut!(szNumServices) as *mut WCHAR,
            31,
            lpNumServices as *const WCHAR,
            NumListedServ,
        );

        SendMessage(
            (*Info).hStatus,
            (SB_SETTEXT as u32),
            0,
            core::ptr::addr_of_mut!(szNumServices) as *mut WCHAR as LPARAM,
        );

        LocalFree(lpNumServices as HLOCAL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetMenuAndButtonStates(Info: PMAIN_WND_INFO) {
    let mut hMainMenu: HMENU = core::ptr::null_mut();
    let mut i: UINT;

    /* get handle to menu */
    hMainMenu = GetMenu((*Info).hMainWnd);

    /* set all to greyed */
    i = ID_START;
    while i <= ID_RESTART {
        EnableMenuItem(hMainMenu, i, MF_GRAYED);
        EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), i, MF_GRAYED);
        SendMessage((*Info).hTool, TB_SETSTATE, (i) as _,
                    (MAKELONG!(TBSTATE_INDETERMINATE, 0)) as LPARAM);
        i += 1;
    }

    if (*Info).SelectedItem != NO_ITEM_SELECTED {
        let mut lpServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
        let mut Flags: DWORD;
        let mut State: DWORD;

        /* allow user to delete service */
        if (*Info).bIsUserAnAdmin != 0 {
            SendMessage((*Info).hTool, TB_SETSTATE, (ID_DELETE as WPARAM),
                       (MAKELONG!(TBSTATE_ENABLED, 0)) as LPARAM);
            EnableMenuItem(hMainMenu, ID_DELETE, MF_ENABLED);
            EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), ID_DELETE, MF_ENABLED);
        }

        Flags = (*(*Info).pCurrentService).ServiceStatusProcess.dwControlsAccepted;
        State = (*(*Info).pCurrentService).ServiceStatusProcess.dwCurrentState;

        lpServiceConfig = GetServiceConfig((*(*Info).pCurrentService).lpServiceName);

        if !lpServiceConfig.is_null() && (*lpServiceConfig).dwStartType != SERVICE_DISABLED {
            if State == SERVICE_STOPPED {
                EnableMenuItem(hMainMenu, ID_START, MF_ENABLED);
                EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), ID_START, MF_ENABLED);
                SendMessage((*Info).hTool, TB_SETSTATE, (ID_START as WPARAM),
                       (MAKELONG!(TBSTATE_ENABLED, 0)) as LPARAM);
            }

            if (Flags & SERVICE_ACCEPT_STOP) != 0 && (State == SERVICE_RUNNING) {
                EnableMenuItem(hMainMenu, ID_RESTART, MF_ENABLED);
                EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), ID_RESTART, MF_ENABLED);
                SendMessage((*Info).hTool, TB_SETSTATE, (ID_RESTART as WPARAM),
                       (MAKELONG!(TBSTATE_ENABLED, 0)) as LPARAM);
            }
        }

        if !lpServiceConfig.is_null() {
            HeapFree(GetProcessHeap(), 0, lpServiceConfig as PVOID);
        }

        if (Flags & SERVICE_ACCEPT_STOP) != 0 && (State == SERVICE_RUNNING) {
            EnableMenuItem(hMainMenu, ID_STOP, MF_ENABLED);
            EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), ID_STOP, MF_ENABLED);
            SendMessage((*Info).hTool, TB_SETSTATE, (ID_STOP as WPARAM),
                   (MAKELONG!(TBSTATE_ENABLED, 0)) as LPARAM);
        }

        if (Flags & SERVICE_ACCEPT_PAUSE_CONTINUE) != 0 && (State == SERVICE_RUNNING) {
            EnableMenuItem(hMainMenu, ID_PAUSE, MF_ENABLED);
            EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), ID_PAUSE, MF_ENABLED);
            SendMessage((*Info).hTool, TB_SETSTATE, (ID_PAUSE as WPARAM),
                   (MAKELONG!(TBSTATE_ENABLED, 0)) as LPARAM);
        }
    } else {
        /* disable tools which rely on a selected service */
        EnableMenuItem(hMainMenu, ID_PROP, MF_GRAYED);
        EnableMenuItem(hMainMenu, ID_DELETE, MF_GRAYED);
        EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), ID_PROP, MF_GRAYED);
        EnableMenuItem(GetSubMenu((*Info).hShortcutMenu, 0), ID_DELETE, MF_GRAYED);
        SendMessage((*Info).hTool, TB_SETSTATE, (ID_PROP as WPARAM),
                   (MAKELONG!(TBSTATE_INDETERMINATE, 0)) as LPARAM);
        SendMessage((*Info).hTool, TB_SETSTATE, (ID_DELETE as WPARAM),
                   (MAKELONG!(TBSTATE_INDETERMINATE, 0)) as LPARAM);
    }
}

unsafe extern "system" fn CompareFunc(lParam1: LPARAM, lParam2: LPARAM, lParamSort: LPARAM) -> INT {
    let Info: PMAIN_WND_INFO = lParamSort as PMAIN_WND_INFO;
    let mut Item1: [WCHAR; 256] = [0; 256];
    let mut Item2: [WCHAR; 256] = [0; 256];

    ListView_GetItemText!(
        (*Info).hListView,
        lParam1,
        (*Info).SortSelection,
        Item1.as_mut_ptr(),
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );
    ListView_GetItemText!(
        (*Info).hListView,
        lParam2,
        (*Info).SortSelection,
        Item2.as_mut_ptr(),
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );

    return wcscmp(Item1.as_mut_ptr(), Item2.as_mut_ptr()) as INT * (*Info).SortDirection;
}

unsafe extern "C" fn pCreateToolbar(Info: PMAIN_WND_INFO) -> BOOL {
    let mut numButtons: INT = (core::mem::size_of_val(&Buttons) / core::mem::size_of_val(&Buttons[0])) as INT;

    (*Info).hTool = CreateWindowEx(0,
                                 TOOLBARCLASSNAME,
                                 core::ptr::null_mut(),
                                 WS_CHILD | WS_VISIBLE | TBSTYLE_FLAT | TBSTYLE_TOOLTIPS,
                                 0, 0, 0, 0,
                                 (*Info).hMainWnd,
                                 core::ptr::null_mut(),
                                 hInstance,
                                 core::ptr::null_mut());
    if !(*Info).hTool.is_null() {
        let mut hImageList: HIMAGELIST = core::ptr::null_mut();

        SendMessage((*Info).hTool,
                    TB_SETEXTENDEDSTYLE,
                    0,
                    TBSTYLE_EX_HIDECLIPPEDBUTTONS as LPARAM);

        SendMessage((*Info).hTool,
                    TB_BUTTONSTRUCTSIZE,
                    core::mem::size_of_val(&Buttons[0]) as WPARAM,
                    0);

        hImageList = InitImageList(IDB_PROP,
                                   IDB_RESTART,
                                   (GetSystemMetrics((SM_CXSMICON) as i32)) as UINT,
                                   (GetSystemMetrics((SM_CXSMICON) as i32)) as UINT,
                                   IMAGE_BITMAP);
        if hImageList.is_null() {
            return (FALSE) as BOOL;
        }

        ImageList_Destroy(SendMessage((*Info).hTool,
                                                  TB_SETIMAGELIST,
                                                  0,
                                                  hImageList as LPARAM) as HIMAGELIST);

        SendMessage((*Info).hTool,
                    (TB_ADDBUTTONS as u32),
                    numButtons as WPARAM,
                    (&raw const Buttons) as LPARAM);

        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn CreateStatusBar(mut Info: PMAIN_WND_INFO) -> BOOL {
    let mut StatWidths: [INT; 2] = [130, -1];
    (*Info).hStatus = CreateWindowEx(
        0,
        STATUSCLASSNAME,
        core::ptr::null_mut(),
        WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
        0,
        0,
        0,
        0,
        (*Info).hMainWnd,
        IDC_STATUSBAR as HMENU,
        hInstance,
        core::ptr::null_mut(),
    );
    if (*Info).hStatus.is_null() {
        return (FALSE) as BOOL;
    }
    SendMessage(
        (*Info).hStatus,
        SB_SETPARTS,
        (core::mem::size_of::<[INT; 2]>() / core::mem::size_of::<INT>()) as WPARAM,
        core::ptr::addr_of_mut!(StatWidths) as LPARAM,
    );
    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitMainWnd(Info: PMAIN_WND_INFO) -> BOOL {
    if pCreateToolbar(Info) == 0 {
        DisplayString(RUSTCONV_WSTR_CBE6669ECA10.as_ptr().cast_mut());
        return (FALSE) as BOOL;
    }

    if CreateListView(Info) == 0 {
        DisplayString(RUSTCONV_WSTR_8DED1C60686E.as_ptr().cast_mut());
        return (FALSE) as BOOL;
    }

    if CreateStatusBar(Info) == 0 {
        DisplayString(RUSTCONV_WSTR_BB541F34C7CE.as_ptr().cast_mut());
    }

    /* Create Popup Menu */
    (*Info).hShortcutMenu = LoadMenu(
        hInstance,
        MAKEINTRESOURCE!(IDR_POPUP),
    );

    (*Info).bIsUserAnAdmin = (TRUE) as i32; // IsUserAnAdmin();
    if (*Info).bIsUserAnAdmin != 0 {
        let mut hMainMenu: HMENU = GetMenu((*Info).hMainWnd);

        SendMessage(
            (*Info).hTool,
            TB_SETSTATE,
            (ID_CREATE as WPARAM),
            MAKELONG!(TBSTATE_ENABLED, 0) as LPARAM,
        );
        if !hMainMenu.is_null() {
            EnableMenuItem(
                hMainMenu,
                ID_CREATE,
                MF_ENABLED,
            );
        }
        EnableMenuItem(
            GetSubMenu((*Info).hShortcutMenu, 0),
            ID_CREATE,
            MF_ENABLED,
        );
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn MainWndCommand(Info: PMAIN_WND_INFO, CmdId: WORD, hControl: HWND) {
    let mut szAppName: [WCHAR; 256] = [0; 256];
    let mut szAppAuthors: [WCHAR; 256] = [0; 256];

    UNREFERENCED_PARAMETER!(hControl);

    match (CmdId as i32) as u32 {
        _ if ((CmdId as i32)) as u32 == ID_PROP => {
            if (*Info).SelectedItem != NO_ITEM_SELECTED {
                (*Info).bDlgOpen = (TRUE) as i32;
                OpenPropSheet(Info);
                (*Info).bDlgOpen = (FALSE) as i32;
                SetMenuAndButtonStates(Info);
            }
        }

        _ if ((CmdId as i32)) as u32 == ID_REFRESH => {
            RefreshServiceList(Info);
            (*Info).SelectedItem = NO_ITEM_SELECTED;

            /* disable menus and buttons */
            SetMenuAndButtonStates(Info);

            /* clear the service in the status bar */
            SendMessage(
                (*Info).hStatus,
                (SB_SETTEXT as u32),
                1,
                (0x0000u16) as _,
            );
        }

        _ if ((CmdId as i32)) as u32 == ID_EXPORT => {
            ExportFile(Info);
            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_CREATE => {
            let mut ret: INT;

            ret = (DialogBoxParam(
                hInstance,
                MAKEINTRESOURCE!(IDD_DLG_CREATE),
                (*Info).hMainWnd,
                Some(CreateDialogProc),
                Info as LPARAM,
            )) as i32;
            if (ret) as u32 == IDOK {
                RefreshServiceList(Info);
            }

            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_DELETE => {
            if (*(*Info).pCurrentService).ServiceStatusProcess.dwCurrentState != SERVICE_RUNNING {
                DialogBoxParam(
                    hInstance,
                    MAKEINTRESOURCE!(IDD_DLG_DELETE),
                    (*Info).hMainWnd,
                    Some(DeleteDialogProc),
                    Info as LPARAM,
                );
            } else {
                let mut Buf: [WCHAR; 60] = [0; 60];
                LoadString(
                    hInstance,
                    IDS_DELETE_STOP,
                    core::ptr::addr_of_mut!(Buf) as LPWSTR,
                    (core::mem::size_of::<[WCHAR; 60]>()
                        / core::mem::size_of::<WCHAR>()) as INT,
                );
                DisplayString(core::ptr::addr_of_mut!(Buf) as LPWSTR);
            }

            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_START => {
            RunActionWithProgress(
                (*Info).hMainWnd,
                (*(*Info).pCurrentService).lpServiceName,
                (*(*Info).pCurrentService).lpDisplayName,
                ACTION_START,
                core::ptr::null_mut(),
            ); //FIXME: Add start params

            UpdateServiceStatus((*Info).pCurrentService);
            ChangeListViewText(Info, (*Info).pCurrentService, LVSTATUS);
            SetMenuAndButtonStates(Info);
            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_STOP => {
            RunActionWithProgress(
                (*Info).hMainWnd,
                (*(*Info).pCurrentService).lpServiceName,
                (*(*Info).pCurrentService).lpDisplayName,
                ACTION_STOP,
                core::ptr::null_mut(),
            );

            UpdateServiceStatus((*Info).pCurrentService);
            ChangeListViewText(Info, (*Info).pCurrentService, LVSTATUS);
            SetMenuAndButtonStates(Info);
            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_PAUSE => {
            RunActionWithProgress(
                (*Info).hMainWnd,
                (*(*Info).pCurrentService).lpServiceName,
                (*(*Info).pCurrentService).lpDisplayName,
                ACTION_PAUSE,
                core::ptr::null_mut(),
            );

            UpdateServiceStatus((*Info).pCurrentService);
            ChangeListViewText(Info, (*Info).pCurrentService, LVSTATUS);
            SetMenuAndButtonStates(Info);
            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_RESUME => {
            RunActionWithProgress(
                (*Info).hMainWnd,
                (*(*Info).pCurrentService).lpServiceName,
                (*(*Info).pCurrentService).lpDisplayName,
                ACTION_RESUME,
                core::ptr::null_mut(),
            );

            UpdateServiceStatus((*Info).pCurrentService);
            ChangeListViewText(Info, (*Info).pCurrentService, LVSTATUS);
            SetMenuAndButtonStates(Info);
            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_RESTART => {
            RunActionWithProgress(
                (*Info).hMainWnd,
                (*(*Info).pCurrentService).lpServiceName,
                (*(*Info).pCurrentService).lpDisplayName,
                ACTION_RESTART,
                core::ptr::null_mut(),
            );

            UpdateServiceStatus((*Info).pCurrentService);
            ChangeListViewText(Info, (*Info).pCurrentService, LVSTATUS);
            SetMenuAndButtonStates(Info);
            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_HELP => {
            MessageBoxW(
                core::ptr::null_mut(),
                RUSTCONV_WSTR_8F3A7D4C9443.as_ptr(),
                RUSTCONV_WSTR_E872F95E8F59.as_ptr(),
                MB_OK | MB_ICONINFORMATION,
            );
            SetFocus((*Info).hListView);
        }

        _ if ((CmdId as i32)) as u32 == ID_EXIT => {
            PostMessage(
                (*Info).hMainWnd,
                WM_CLOSE,
                0,
                0,
            );
        }

        _ if ((CmdId as i32)) as u32 == ID_VIEW_LARGE => {
            SetListViewStyle((*Info).hListView, LVS_ICON);
            ListView_Arrange!((*Info).hListView, LVA_DEFAULT);

            CheckMenuRadioItem(
                GetMenu((*Info).hMainWnd),
                ID_VIEW_LARGE,
                ID_VIEW_DETAILS,
                ID_VIEW_LARGE,
                MF_BYCOMMAND,
            );
        }

        _ if ((CmdId as i32)) as u32 == ID_VIEW_SMALL => {
            SetListViewStyle((*Info).hListView, LVS_SMALLICON);
            ListView_Arrange!((*Info).hListView, LVA_DEFAULT);

            CheckMenuRadioItem(
                GetMenu((*Info).hMainWnd),
                ID_VIEW_LARGE,
                ID_VIEW_DETAILS,
                ID_VIEW_SMALL,
                MF_BYCOMMAND,
            );
        }

        _ if ((CmdId as i32)) as u32 == ID_VIEW_LIST => {
            SetListViewStyle((*Info).hListView, LVS_LIST);
            CheckMenuRadioItem(
                GetMenu((*Info).hMainWnd),
                ID_VIEW_LARGE,
                ID_VIEW_DETAILS,
                ID_VIEW_LIST,
                MF_BYCOMMAND,
            );
        }

        _ if ((CmdId as i32)) as u32 == ID_VIEW_DETAILS => {
            SetListViewStyle((*Info).hListView, LVS_REPORT);
            CheckMenuRadioItem(
                GetMenu((*Info).hMainWnd),
                ID_VIEW_LARGE,
                ID_VIEW_DETAILS,
                ID_VIEW_DETAILS,
                MF_BYCOMMAND,
            );
        }

        _ if ((CmdId as i32)) as u32 == ID_VIEW_CUST => {}

        _ if ((CmdId as i32)) as u32 == ID_ABOUT => {
            LoadStringW(
                hInstance,
                IDS_APPNAME,
                core::ptr::addr_of_mut!(szAppName) as LPWSTR,
                (_countof!(szAppName)) as i32,
            );
            LoadStringW(
                hInstance,
                IDS_APPAUTHORS,
                core::ptr::addr_of_mut!(szAppAuthors) as LPWSTR,
                (_countof!(szAppAuthors)) as i32,
            );

            ShellAboutW(
                (*Info).hMainWnd,
                core::ptr::addr_of_mut!(szAppName) as LPWSTR,
                core::ptr::addr_of_mut!(szAppAuthors) as LPWSTR,
                LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_SM_ICON)),
            );
        }

        _ => {}
    }
}
unsafe extern "system" fn MainWndResize(Info: PMAIN_WND_INFO, cx: WORD, cy: WORD) {
    let mut rcClient: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcTool: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcStatus: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut lvHeight: i32;
    let mut iToolHeight: i32;
    let mut iStatusHeight: i32;

    /* Size toolbar and get height */
    SendMessage((*Info).hTool, TB_AUTOSIZE, 0, 0);
    GetWindowRect((*Info).hTool, core::ptr::addr_of_mut!(rcTool));
    iToolHeight = rcTool.bottom - rcTool.top;

    /* Size status bar and get height */
    SendMessage((*Info).hStatus, WM_SIZE, 0, 0);
    GetWindowRect((*Info).hStatus, core::ptr::addr_of_mut!(rcStatus));
    iStatusHeight = rcStatus.bottom - rcStatus.top;

    /* Calculate remaining height and size list view */
    GetClientRect((*Info).hMainWnd, core::ptr::addr_of_mut!(rcClient));
    lvHeight = rcClient.bottom - iToolHeight - iStatusHeight;
    SetWindowPos(
        (*Info).hListView,
        core::ptr::null_mut(),
        0,
        iToolHeight,
        rcClient.right,
        lvHeight,
        SWP_NOZORDER,
    );
}

unsafe extern "system" fn MainWndProc(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Info: PMAIN_WND_INFO;
    let mut Ret: LRESULT = 0;
    let mut goto_default: bool = false;

    /* Get the window context */
    Info = GetWindowLongPtr(hwnd, GWLP_USERDATA) as PMAIN_WND_INFO;
    if Info.is_null() && msg != WM_CREATE {
        goto_default = true;
    } else {
        match msg {
            WM_CREATE => {
                Info = (*(lParam as LPCREATESTRUCT)).lpCreateParams as PMAIN_WND_INFO;

                /* Initialize the main window context */
                (*Info).hMainWnd = hwnd;
                (*Info).SelectedItem = NO_ITEM_SELECTED;

                SetWindowLongPtr(hwnd, GWLP_USERDATA, Info as LONG_PTR);

                if InitMainWnd(Info) == 0 {
                    return -(1) as LRESULT;
                }

                /* Fill the list-view before showing the main window */
                RefreshServiceList(Info);

                /* Show the window */
                ShowWindow(hwnd, (*Info).nCmdShow);

                SetFocus((*Info).hListView);
            }

            WM_SIZE => {
                MainWndResize(Info, LOWORD!(lParam), HIWORD!(lParam));
            }

            WM_NOTIFY => {
                let pnmhdr: LPNMHDR = lParam as LPNMHDR;

                match (((*pnmhdr).code) as i32) as u32 {
                    NM_DBLCLK => {
                        let mut pt: POINT = core::mem::zeroed();
                        let mut rect: RECT = core::mem::zeroed();

                        GetCursorPos(core::ptr::addr_of_mut!(pt));
                        GetWindowRect((*Info).hListView, core::ptr::addr_of_mut!(rect));

                        if PtInRect(core::ptr::addr_of_mut!(rect), pt) != 0 {
                            SendMessage(
                                hwnd,
                                WM_COMMAND,
                                // ID_PROP,
                                MAKEWPARAM!(ID_PROP as WORD, 0 as WORD),
                                0,
                            );
                        }

                        //OpenPropSheet(Info);
                    }

                    NM_RETURN => {
                        SendMessage(
                            hwnd,
                            WM_COMMAND,
                            // ID_PROP,
                            MAKEWPARAM!(ID_PROP as WORD, 0 as WORD),
                            0,
                        );
                    }

                    __c if __c == LVN_COLUMNCLICK as u32 => {
                        let pnmv: LPNMLISTVIEW = lParam as LPNMLISTVIEW;
                        let mut hdi: HDITEM = core::mem::zeroed();

                        /* get pending sort direction for clicked column */
                        hdi.mask = HDI_LPARAM;
                        let _ = Header_GetItem!((*Info).hHeader, (*pnmv).iSubItem, core::ptr::addr_of_mut!(hdi));

                        /* get new sort parameters */
                        (*Info).SortSelection = (*pnmv).iSubItem as INT;
                        (*Info).SortDirection = hdi.lParam as INT;

                        /* set new sort direction and save */
                        hdi.lParam = (if hdi.lParam == ORD_ASCENDING as LPARAM {
                            ORD_DESCENDING
                        } else {
                            (ORD_ASCENDING as i32)
                        }) as LPARAM;

                        let _ = Header_SetItem!((*Info).hHeader, (*pnmv).iSubItem, core::ptr::addr_of_mut!(hdi));

                        let _ = ListView_SortItemsEx!((*Info).hListView, CompareFunc, Info as LPARAM);
                    }

                    __c if __c == LVN_ITEMCHANGED as u32 => {
                        let pnmv: LPNMLISTVIEW = lParam as LPNMLISTVIEW;

                        if (*pnmv).uNewState != 0 as UINT {
                            ListViewSelectionChanged(Info, pnmv);
                            SetMenuAndButtonStates(Info);
                        }
                    }

                    TTN_GETDISPINFO => {
                        let lpttt: LPTOOLTIPTEXT = lParam as LPTOOLTIPTEXT;
                        let mut idButton: UINT;

                        /* Specify the resource identifier of the descriptive
                         * text for the given button. */
                        idButton = (*lpttt).hdr.idFrom as UINT;
                        match idButton {
                            ID_PROP => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_PROP);
                            }
                            ID_REFRESH => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_REFRESH);
                            }
                            ID_EXPORT => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_EXPORT);
                            }
                            ID_CREATE => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_CREATE);
                            }
                            ID_DELETE => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_DELETE);
                            }
                            ID_START => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_START);
                            }
                            ID_STOP => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_STOP);
                            }
                            ID_PAUSE => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_PAUSE);
                            }
                            ID_RESTART => {
                                (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_RESTART);
                            }
                            _ => {}
                        }
                    }

                    _ => {}
                }
            }

            WM_CONTEXTMENU => {
                let mut pt: POINT = core::mem::zeroed();
                let mut lvRect: RECT = core::mem::zeroed();

                let xPos: INT = GET_X_LPARAM!(lParam);
                let yPos: INT = GET_Y_LPARAM!(lParam);

                GetCursorPos(core::ptr::addr_of_mut!(pt));

                /* display popup when cursor is in the list view */
                GetWindowRect((*Info).hListView, core::ptr::addr_of_mut!(lvRect));
                if PtInRect(core::ptr::addr_of_mut!(lvRect), pt) != 0 {
                    TrackPopupMenuEx(
                        GetSubMenu((*Info).hShortcutMenu, 0),
                        TPM_RIGHTBUTTON,
                        xPos,
                        yPos,
                        (*Info).hMainWnd,
                        core::ptr::null_mut(),
                    );
                }
            }

            WM_COMMAND => {
                MainWndCommand(Info, LOWORD!(wParam), lParam as HWND);
                goto_default = true;
            }

            WM_MENUSELECT => {
                if !(*Info).hStatus.is_null() {
                    if MainWndMenuHint(
                        Info,
                        LOWORD!(wParam),
                        core::ptr::addr_of!(MainMenuHintTable) as *const MENU_HINT,
                        (core::mem::size_of_val(&MainMenuHintTable)
                            / core::mem::size_of_val(&MainMenuHintTable[0])) as DWORD,
                        IDS_HINT_BLANK,
                    ) == 0
                    {
                        MainWndMenuHint(
                            Info,
                            LOWORD!(wParam),
                            core::ptr::addr_of!(SystemMenuHintTable) as *const MENU_HINT,
                            (core::mem::size_of_val(&SystemMenuHintTable)
                                / core::mem::size_of_val(&SystemMenuHintTable[0])) as DWORD,
                            IDS_HINT_BLANK,
                        );
                    }
                }
            }

            WM_ENTERMENULOOP => {
                (*Info).bInMenuLoop = TRUE as BOOL;
                UpdateMainStatusBar(Info);
            }

            WM_EXITMENULOOP => {
                (*Info).bInMenuLoop = FALSE as BOOL;
                UpdateMainStatusBar(Info);
            }

            WM_CLOSE => {
                FreeServiceList(Info);
                DestroyMenu((*Info).hShortcutMenu);
                DestroyWindow(hwnd);
            }

            WM_DESTROY => {
                HeapFree(ProcessHeap, 0, Info as PVOID);
                SetWindowLongPtr(hwnd, GWLP_USERDATA, 0);

                PostQuitMessage(0);
            }

            _ => {
                goto_default = true;
            }
        }
    }

    if goto_default {
        /* HandleDefaultMessage: */
        Ret = DefWindowProc(hwnd, msg, wParam, lParam);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn CreateMainWindow(mut lpCaption: LPCTSTR, mut nCmdShow: ::core::ffi::c_int) -> HWND {
    let mut Info: PMAIN_WND_INFO = ::core::ptr::null_mut::<_MAIN_WND_INFO>();
    let mut hMainWnd: HWND = ::core::ptr::null_mut::<HWND__>();
    Info = HeapAlloc(
        ProcessHeap,
        HEAP_ZERO_MEMORY as DWORD,
        ::core::mem::size_of::<MAIN_WND_INFO>() as SIZE_T,
    ) as *mut MAIN_WND_INFO as PMAIN_WND_INFO;
    if !Info.is_null() {
        (*Info).nCmdShow = nCmdShow;
        hMainWnd = CreateWindowEx(
            WS_EX_WINDOWEDGE as DWORD,
            ::core::ptr::addr_of!(szMainWndClass) as LPCWSTR,
            lpCaption as LPCWSTR,
            (WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS) as DWORD,
            CW_USEDEFAULT as ::core::ffi::c_int,
            CW_USEDEFAULT as ::core::ffi::c_int,
            680 as ::core::ffi::c_int,
            450 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<HWND__>(),
            ::core::ptr::null_mut::<HMENU__>(),
            hInstance,
            Info as LPVOID,
        );
        if hMainWnd.is_null() {
            GetError();
            HeapFree(
                ProcessHeap,
                0 as DWORD,
                Info as PVOID,
            );
        }
    }
    return hMainWnd;
}
#[no_mangle]
pub unsafe extern "C" fn InitMainWindowImpl() -> BOOL {
    let mut wc: WNDCLASSEX = core::mem::zeroed();

    wc.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    wc.lpfnWndProc = Some(MainWndProc);
    wc.hInstance = hInstance;
    wc.hIcon = LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_SM_ICON));
    wc.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
    wc.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wc.lpszMenuName = MAKEINTRESOURCE!(IDR_MAINMENU);
    wc.lpszClassName = core::ptr::addr_of!(szMainWndClass) as *const WCHAR as LPCTSTR;
    wc.hIconSm = LoadImage(
        hInstance,
        MAKEINTRESOURCE!(IDI_SM_ICON),
        IMAGE_ICON,
        16,
        16,
        LR_SHARED,
    ) as HICON;

    return (RegisterClassEx(core::ptr::addr_of_mut!(wc))
        != 0 as ATOM) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UninitMainWindowImpl() {
    UnregisterClass(core::ptr::addr_of!(szMainWndClass) as LPCTSTR, hInstance);
}

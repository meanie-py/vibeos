use crate::regedit::treeview::CreateNewKey;
use crate::regedit::edit::DeleteKey;
use crate::regedit::treeview::DeleteNode;
use crate::regedit::find::FindDialog;
use crate::regedit::find::FindNextMessageBox;
use crate::regedit::treeview::GetItemPath;
use crate::regedit::edit::GetKeyName;
use crate::regedit::listview::GetValueName;
use crate::regedit::edit::ModifyValue;
use crate::regedit::listview::RefreshListView;
use crate::regedit::treeview::RefreshTreeView;
use crate::regedit::security::RegKeyEditPermissions;
use crate::regedit::settings::SaveSettings;
use crate::regedit::treeview::SelectNode;
use crate::regedit::about::ShowAboutBox;
use crate::regedit::childwnd::Suggestions;
use crate::regedit::regproc::export_registry_key;
use crate::regedit::childwnd::get_root_key_name;
use crate::regedit::regproc::import_registry_file;
use crate::regedit::txtproc::txt_export_registry_key;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_69D435449EDE: [u16; 7] = [0x0025u16, 0x0073, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_76CFE90FF2FC: [u16; 3] = [0x0072u16, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_E89B9A83645C: [u16; 8] = [0x0072u16, 0x0065, 0x0067, 0x0065, 0x0064, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_EED2BF96A061: [u16; 4] = [0x0072u16, 0x0065, 0x0067, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut s_szFavoritesRegKey: [WCHAR; 68] = unsafe {
    ::core::mem::transmute::<
        [u8; 136],
        [WCHAR; 68],
    >(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0A\0p\0p\0l\0e\0t\0s\0\\\0R\0e\0g\0e\0d\0i\0t\0\\\0F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0",
    )
};

static mut bInMenuLoop: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut FileNameBuffer: [WCHAR; 260] = [0; 260];

pub const FAVORITES_MENU_POSITION: i32 = 3;


#[repr(C)]
pub struct FILTERPAIR {
    pub DisplayID: UINT,
    pub FilterID: UINT,
}
pub type PFILTERPAIR = *mut FILTERPAIR;

pub const LOADHIVE_KEYNAMELENGTH: i32 = 128;

unsafe extern "C" fn ErrorBox(hWnd: HWND, Error: UINT) -> UINT {
    let mut buf: [WCHAR; 400] = [0; 400];
    if FormatMessageW(
        (FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD,
        core::ptr::null(),
        Error as DWORD,
        0 as DWORD,
        core::ptr::addr_of_mut!(buf) as LPWSTR,
        _countof!(buf) as DWORD,
        core::ptr::null_mut(),
    ) == 0
    {
        *(core::ptr::addr_of_mut!(buf) as *mut UINT) = 0x003Fu16 as UINT;
    }
    MessageBoxW(
        hWnd,
        core::ptr::addr_of_mut!(buf) as LPCWSTR,
        core::ptr::null(),
        MB_ICONSTOP as UINT,
    );
    return Error;
}

unsafe extern "C" fn resize_frame_rect(hWnd: HWND, prect: PRECT) {
    if IsWindowVisible(hStatusBar) != 0 {
        let mut rt: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        SetupStatusBar(hWnd, (TRUE) as BOOL);
        GetWindowRect(hStatusBar, core::ptr::addr_of_mut!(rt));
        (*prect).bottom -= rt.bottom - rt.top;
    }
    MoveWindow(
        (*g_pChildWnd).hWnd,
        (*prect).left,
        (*prect).top,
        (*prect).right,
        (*prect).bottom,
        (TRUE) as BOOL,
    );
}

unsafe extern "C" fn resize_frame_client(hWnd: HWND) {
    let mut rect: RECT = core::mem::zeroed();
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
    resize_frame_rect(hWnd, core::ptr::addr_of_mut!(rect));
}

unsafe extern "C" fn OnInitMenu(hWnd: HWND) {
    let mut lResult: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwIndex: DWORD;
    let mut cbValueName: DWORD;
    let mut cbValueData: DWORD;
    let mut dwType: DWORD = core::mem::zeroed();
    let mut szValueName: [WCHAR; 256] = [0; 256];
    let mut abValueData: [BYTE; 256] = [0; 256];
    static mut s_nFavoriteMenuSubPos: i32 = -1;
    let mut hMenu: HMENU;
    let mut bDisplayedAny: BOOL = (FALSE) as i32;
    let mut hSelTreeItem: HTREEITEM;
    let mut bCanAddFav: BOOL;

    'done: {
        /* Find Favorites menu and clear it out */
        hMenu = GetSubMenu(GetMenu(hWnd), FAVORITES_MENU_POSITION);
        if hMenu.is_null() {
            break 'done;
        }
        if s_nFavoriteMenuSubPos < 0 {
            s_nFavoriteMenuSubPos = GetMenuItemCount(hMenu);
        } else {
            while RemoveMenu(hMenu, s_nFavoriteMenuSubPos as UINT, MF_BYPOSITION as UINT) != 0 {}
        }

        hSelTreeItem = TreeView_GetSelection!((*g_pChildWnd).hTreeWnd);
        bCanAddFav = (TreeView_GetParent!((*g_pChildWnd).hTreeWnd, hSelTreeItem) != core::ptr::null_mut()) as BOOL;
        EnableMenuItem(
            GetMenu(hWnd),
            ID_FAVOURITES_ADDTOFAVOURITES as UINT,
            (MF_BYCOMMAND | (if bCanAddFav != 0 { MF_ENABLED } else { MF_GRAYED })) as UINT,
        );

        lResult = RegOpenKeyW(HKEY_CURRENT_USER, core::ptr::addr_of_mut!(s_szFavoritesRegKey) as LPCWSTR, core::ptr::addr_of_mut!(hKey));
        if (lResult) as u32 != ERROR_SUCCESS {
            break 'done;
        }

        dwIndex = 0;
        loop {
            cbValueName = ARRAY_SIZE!(szValueName) as DWORD;
            cbValueData = core::mem::size_of::<[BYTE; 256]>() as DWORD;
            lResult = RegEnumValueW(
                hKey,
                dwIndex,
                core::ptr::addr_of_mut!(szValueName) as *mut WCHAR as LPWSTR,
                core::ptr::addr_of_mut!(cbValueName),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwType),
                core::ptr::addr_of_mut!(abValueData) as *mut BYTE as LPBYTE,
                core::ptr::addr_of_mut!(cbValueData),
            );
            if ((lResult) as u32 == ERROR_SUCCESS) && (dwType == REG_SZ) {
                if bDisplayedAny == 0 {
                    AppendMenu(hMenu, MF_SEPARATOR as UINT, 0 as UINT_PTR, core::ptr::null_mut());
                    bDisplayedAny = (TRUE) as i32;
                }
                AppendMenu(
                    hMenu,
                    0 as UINT,
                    (ID_FAVORITES_MIN + (GetMenuItemCount(hMenu)) as u32) as UINT_PTR,
                    core::ptr::addr_of_mut!(szValueName) as *mut WCHAR as LPCWSTR,
                );
            }
            dwIndex = dwIndex.wrapping_add(1);
            if !((lResult) as u32 == ERROR_SUCCESS) {
                break;
            }
        }
    }

    // done:
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn OnEnterMenuLoop(hWnd: HWND) {
    let mut nParts: i32;
    UNREFERENCED_PARAMETER!(hWnd);

    /* Update the status bar pane sizes */
    nParts = -1;
    SendMessageW(hStatusBar, SB_SETPARTS, 1, &mut nParts as *mut i32 as LPARAM);
    bInMenuLoop = (TRUE) as i32;
    SendMessageW(hStatusBar, SB_SETTEXTW, 0 as WPARAM, RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPARAM);
}

unsafe extern "C" fn OnExitMenuLoop(hWnd: HWND) {
    bInMenuLoop = (FALSE) as i32;
    SetupStatusBar(hWnd, (TRUE) as BOOL);
    UpdateStatusBar();
}

unsafe extern "C" fn OnMenuSelect(
    mut hWnd: HWND,
    mut nItemID: UINT,
    mut nFlags: UINT,
    mut hSysMenu: HMENU,
) {
    let mut str: [WCHAR; 100] = [0; 100];

    str[0 as usize] = UNICODE_NULL;
    if (nFlags & MF_POPUP) != 0 {
        if hSysMenu != GetMenu(hWnd) {
            if nItemID == 2 {
                nItemID = 5;
            }
        }
    }
    if LoadStringW(
        hInst,
        nItemID,
        core::ptr::addr_of_mut!(str) as LPWSTR,
        100,
    ) != 0
    {
        /* load appropriate string*/
        let mut lpsz: LPWSTR = core::ptr::addr_of_mut!(str) as LPWSTR;
        /* first newline terminates actual string*/
        lpsz = wcschr(lpsz as *const WCHAR, 0x000Au16) as LPWSTR;
        if !lpsz.is_null() {
            *lpsz = 0x0000u16;
        }
    }
    SendMessageW(
        hStatusBar,
        SB_SETTEXTW,
        0 as WPARAM,
        core::ptr::addr_of_mut!(str) as *mut WCHAR as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SetupStatusBar(hWnd: HWND, bResize: BOOL) {
    let mut rc: RECT = core::mem::zeroed();
    let mut nParts: i32 = 0;
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
    nParts = rc.right;
    /*    nParts = -1;*/
    if bResize != 0 {
        SendMessageW(hStatusBar, WM_SIZE as UINT, 0 as WPARAM, 0 as LPARAM);
    }
    SendMessageW(hStatusBar, SB_SETPARTS as UINT, 1 as WPARAM, core::ptr::addr_of_mut!(nParts) as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn UpdateStatusBar() {
    let mut hKeyRoot: HKEY = core::ptr::null_mut();
    let mut pszKeyPath: LPCWSTR = core::ptr::null();
    let mut pszRootName: LPCWSTR = core::ptr::null();
    let mut pszFullPath: LPWSTR = core::ptr::null_mut();
    let mut dwCbFullPath: DWORD = 0;

    pszKeyPath = GetItemPath((*g_pChildWnd).hTreeWnd, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKeyRoot));
    if pszKeyPath.is_null() {
        return;
    }

    pszRootName = get_root_key_name(hKeyRoot);
    dwCbFullPath = ((wcslen(pszRootName) as DWORD) + 1 + (wcslen(pszKeyPath) as DWORD) + 1) * (core::mem::size_of::<WCHAR>() as DWORD);
    pszFullPath = malloc(dwCbFullPath as usize) as LPWSTR;
    if pszFullPath.is_null() {
        return;
    }

    if *pszKeyPath.offset(0 as isize) != UNICODE_NULL {
        StringCbPrintfW!(
            pszFullPath,
            dwCbFullPath as usize,
            RUSTCONV_WSTR_69D435449EDE.as_ptr(),
            pszRootName,
            (if *pszKeyPath.offset(0 as isize) == 0x005Cu16 {
                RUSTCONV_WSTR_96A296D224F2.as_ptr()
            } else {
                RUSTCONV_WSTR_B7FC02907B1F.as_ptr()
            }),
            pszKeyPath,
        );
    } else {
        StringCbCopyW(pszFullPath, dwCbFullPath as usize, pszRootName);
    }

    SendMessageW(hStatusBar, SB_SETTEXTW, 0, pszFullPath as LPARAM);
    free(pszFullPath as *mut core::ffi::c_void);
}

unsafe extern "C" fn toggle_child(hWnd: HWND, cmd: UINT, hchild: HWND) {
    let vis: BOOL = IsWindowVisible(hchild);
    let hMenuView: HMENU = GetSubMenu(hMenuFrame, (ID_VIEW_MENU as i32));

    CheckMenuItem(
        hMenuView,
        cmd,
        (if vis != 0 {
            MF_BYCOMMAND
        } else {
            MF_BYCOMMAND | MF_CHECKED
        }) as UINT,
    );
    ShowWindow(
        hchild,
        if vis != 0 { (SW_HIDE) as i32 } else { (SW_SHOW) as i32 },
    );
    resize_frame_client(hWnd);
}

unsafe extern "C" fn CheckCommDlgError(mut hWnd: HWND) -> BOOL {
    let mut dwErrorCode: DWORD = CommDlgExtendedError();
    UNREFERENCED_PARAMETER!(hWnd);
    match dwErrorCode {
        CDERR_DIALOGFAILURE => {}
        CDERR_FINDRESFAILURE => {}
        CDERR_NOHINSTANCE => {}
        CDERR_INITIALIZATION => {}
        CDERR_NOHOOK => {}
        CDERR_LOCKRESFAILURE => {}
        CDERR_NOTEMPLATE => {}
        CDERR_LOADRESFAILURE => {}
        CDERR_STRUCTSIZE => {}
        CDERR_LOADSTRFAILURE => {}
        FNERR_BUFFERTOOSMALL => {}
        CDERR_MEMALLOCFAILURE => {}
        FNERR_INVALIDFILENAME => {}
        CDERR_MEMLOCKFAILURE => {}
        FNERR_SUBCLASSFAILURE => {}
        _ => {}
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn BuildFilterStrings(
    Filter: *mut WCHAR,
    Pairs: PFILTERPAIR,
    PairCount: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int;
    let mut c: ::core::ffi::c_int;
    c = 0;
    i = 0;
    while i < PairCount {
        c += LoadStringW(
            hInst,
            (*Pairs.offset(i as isize)).DisplayID,
            Filter.offset(c as isize) as LPWSTR,
            255,
        );
        c += 1;
        *Filter.offset(c as isize) = 0x0000u16;
        c += LoadStringW(
            hInst,
            (*Pairs.offset(i as isize)).FilterID,
            Filter.offset(c as isize) as LPWSTR,
            255,
        );
        c += 1;
        *Filter.offset(c as isize) = 0x0000u16;
        i += 1;
    }
    c += 1;
    *Filter.offset(c as isize) = 0x0000u16;
}

unsafe extern "C" fn InitOpenFileName(hWnd: HWND, pofn: *mut OPENFILENAME, bSave: BOOL) -> BOOL {
    let mut FilterPairs: [FILTERPAIR; 5] = [
        FILTERPAIR { DisplayID: 0, FilterID: 0 },
        FILTERPAIR { DisplayID: 0, FilterID: 0 },
        FILTERPAIR { DisplayID: 0, FilterID: 0 },
        FILTERPAIR { DisplayID: 0, FilterID: 0 },
        FILTERPAIR { DisplayID: 0, FilterID: 0 },
    ];
    static mut Filter: [WCHAR; 1024] = [0; 1024];

    memset(
        pofn as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<OPENFILENAME>() as size_t,
    );
    (*pofn).lStructSize = ::core::mem::size_of::<OPENFILENAME>() as DWORD;
    (*pofn).hwndOwner = hWnd;
    (*pofn).hInstance = hInst;

    /* create filter string */
    FilterPairs[0 as usize].DisplayID = IDS_FLT_REGFILES as UINT;
    FilterPairs[0 as usize].FilterID = IDS_FLT_REGFILES_FLT as UINT;
    FilterPairs[1 as usize].DisplayID = IDS_FLT_HIVFILES as UINT;
    FilterPairs[1 as usize].FilterID = IDS_FLT_HIVFILES_FLT as UINT;
    FilterPairs[2 as usize].DisplayID = IDS_FLT_REGEDIT4 as UINT;
    FilterPairs[2 as usize].FilterID = IDS_FLT_REGEDIT4_FLT as UINT;
    if bSave != 0 {
        FilterPairs[3 as usize].DisplayID = IDS_FLT_TXTFILES as UINT;
        FilterPairs[3 as usize].FilterID = IDS_FLT_TXTFILES_FLT as UINT;
        FilterPairs[4 as usize].DisplayID = IDS_FLT_ALLFILES as UINT;
        FilterPairs[4 as usize].FilterID = IDS_FLT_ALLFILES_FLT as UINT;
    } else {
        FilterPairs[3 as usize].DisplayID = IDS_FLT_ALLFILES as UINT;
        FilterPairs[3 as usize].FilterID = IDS_FLT_ALLFILES_FLT as UINT;
    }

    BuildFilterStrings(
        core::ptr::addr_of_mut!(Filter) as *mut WCHAR,
        core::ptr::addr_of_mut!(FilterPairs) as PFILTERPAIR,
        ARRAY_SIZE!(FilterPairs) as ::core::ffi::c_int - (bSave == 0) as ::core::ffi::c_int,
    );

    (*pofn).lpstrFilter = core::ptr::addr_of_mut!(Filter) as *mut WCHAR as LPCWSTR;
    (*pofn).lpstrFile = core::ptr::addr_of_mut!(FileNameBuffer) as *mut WCHAR as LPWSTR;
    (*pofn).nMaxFile = _countof!(FileNameBuffer) as DWORD;
    (*pofn).Flags = (OFN_EXPLORER | OFN_HIDEREADONLY) as DWORD;
    (*pofn).lpstrDefExt = RUSTCONV_WSTR_EED2BF96A061.as_ptr();
    if bSave != 0 {
        (*pofn).Flags |= (OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST) as DWORD;
    } else {
        (*pofn).Flags |= OFN_FILEMUSTEXIST as DWORD;
    }

    return TRUE as BOOL;
}

unsafe extern "system" fn LoadHive_KeyNameInHookProc(hWndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut sKey: LPWSTR = core::ptr::null_mut();
    match uMsg {
        WM_INITDIALOG => {
            sKey = lParam as LPWSTR;
        }
        WM_COMMAND => {
            match LOWORD!(wParam) as UINT {
                IDOK => {
                    if GetDlgItemTextW(hWndDlg, (IDC_EDIT_KEY as i32), sKey, LOADHIVE_KEYNAMELENGTH) != 0 {
                        return EndDialog(hWndDlg, -1) as INT_PTR;
                    } else {
                        return EndDialog(hWndDlg, 0) as INT_PTR;
                    }
                }
                IDCANCEL => {
                    return EndDialog(hWndDlg, 0) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "C" fn EnablePrivilege(lpszPrivilegeName: LPCWSTR, lpszSystemName: LPCWSTR, bEnablePrivilege: BOOL) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;
    let mut hToken: HANDLE = core::ptr::null_mut();

    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES,
        core::ptr::addr_of_mut!(hToken),
    ) != 0
    {
        let mut tp: TOKEN_PRIVILEGES = core::mem::zeroed();

        tp.PrivilegeCount = 1;
        tp.Privileges[0].Attributes = if bEnablePrivilege != 0 {
            SE_PRIVILEGE_ENABLED
        } else {
            0
        };

        if LookupPrivilegeValueW(
            lpszSystemName,
            lpszPrivilegeName,
            core::ptr::addr_of_mut!(tp.Privileges[0].Luid),
        ) != 0
        {
            bRet = AdjustTokenPrivileges(
                hToken,
                (FALSE) as BOOL,
                core::ptr::addr_of_mut!(tp),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
                bRet = (FALSE) as i32;
            }
        }

        CloseHandle(hToken);
    }

    return bRet;
}

unsafe extern "C" fn LoadHive(mut hWnd: HWND) -> BOOL {
    let mut ofn: OPENFILENAME = core::mem::zeroed();
    let mut Caption: [WCHAR; 128] = [0; 128];
    let mut pszKeyPath: LPCWSTR = core::ptr::null::<WCHAR>();
    let mut xPath: [WCHAR; (LOADHIVE_KEYNAMELENGTH as usize)] = [0; (LOADHIVE_KEYNAMELENGTH as usize)];
    let mut hRootKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut Filter: [WCHAR; 1024] = [0; 1024];
    let mut filter: FILTERPAIR = FILTERPAIR {
        DisplayID: 0,
        FilterID: 0,
    };
    /* get the item key to load the hive in */
    pszKeyPath = GetItemPath(
        (*g_pChildWnd).hTreeWnd,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hRootKey),
    );
    /* initialize the "open file" dialog */
    InitOpenFileName(hWnd, core::ptr::addr_of_mut!(ofn), (FALSE) as BOOL);
    /* build the "All Files" filter up */
    filter.DisplayID = IDS_FLT_ALLFILES as UINT;
    filter.FilterID = IDS_FLT_ALLFILES_FLT as UINT;
    BuildFilterStrings(
        core::ptr::addr_of_mut!(Filter) as *mut WCHAR,
        core::ptr::addr_of_mut!(filter),
        1 as ::core::ffi::c_int,
    );
    ofn.lpstrFilter = core::ptr::addr_of_mut!(Filter) as *mut WCHAR as LPCWSTR;
    /* load and set the caption and flags for dialog */
    LoadStringW(
        hInst,
        IDS_LOAD_HIVE as UINT,
        core::ptr::addr_of_mut!(Caption) as LPWSTR,
        ARRAY_SIZE!(Caption) as ::core::ffi::c_int,
    );
    ofn.lpstrTitle = core::ptr::addr_of_mut!(Caption) as *mut WCHAR as LPCWSTR;
    ofn.Flags |= OFN_ENABLESIZING as DWORD;

    /* now load the hive */
    if GetOpenFileName(core::ptr::addr_of_mut!(ofn)) != 0 {
        if DialogBoxParamW(
            hInst,
            MAKEINTRESOURCEW!(IDD_LOADHIVE),
            hWnd,
            Some(
                LoadHive_KeyNameInHookProc,
            ),
            core::ptr::addr_of_mut!(xPath) as *mut WCHAR as LPARAM,
        ) != 0
        {
            let mut regLoadResult: LONG = 0;

            /* Enable the 'restore' privilege, load the hive, disable the privilege */
            EnablePrivilege(
                SE_RESTORE_NAME,
                core::ptr::null::<WCHAR>(),
                (TRUE) as BOOL,
            );
            regLoadResult = RegLoadKeyW(
                hRootKey,
                core::ptr::addr_of_mut!(xPath) as *mut WCHAR as LPCWSTR,
                ofn.lpstrFile as LPCWSTR,
            ) as LONG;
            EnablePrivilege(
                SE_RESTORE_NAME,
                core::ptr::null::<WCHAR>(),
                (FALSE) as BOOL,
            );

            if regLoadResult == ERROR_SUCCESS as LONG {
                /* refresh tree and list views */
                RefreshTreeView((*g_pChildWnd).hTreeWnd);
                pszKeyPath = GetItemPath(
                    (*g_pChildWnd).hTreeWnd,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(hRootKey),
                );
                RefreshListView(
                    (*g_pChildWnd).hListWnd,
                    hRootKey,
                    pszKeyPath,
                    (TRUE) as BOOL,
                );
            } else {
                ErrorMessageBox(
                    hWnd,
                    core::ptr::addr_of_mut!(Caption) as *mut WCHAR as LPCWSTR,
                    regLoadResult as DWORD,
                );
                return (FALSE) as BOOL;
            }
        }
    } else {
        CheckCommDlgError(hWnd);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn UnloadHive(hWnd: HWND) -> BOOL {
    let mut Caption: [WCHAR; 128] = [0; 128];
    let mut pszKeyPath: LPCWSTR = core::ptr::null();
    let mut hRootKey: HKEY = core::ptr::null_mut();
    let mut regUnloadResult: LONG = 0;

    /* get the item key to unload */
    pszKeyPath = GetItemPath(
        (*g_pChildWnd).hTreeWnd,
        core::ptr::null_mut(),
        &raw mut hRootKey,
    );
    /* load and set the caption and flags for dialog */
    LoadStringW(
        hInst,
        IDS_UNLOAD_HIVE as UINT,
        core::ptr::addr_of_mut!(Caption) as LPWSTR,
        ARRAY_SIZE!(Caption) as ::core::ffi::c_int,
    );

    /* Enable the 'restore' privilege, unload the hive, disable the privilege */
    EnablePrivilege(
        SE_RESTORE_NAME,
        core::ptr::null_mut(),
        (TRUE) as BOOL,
    );
    regUnloadResult = RegUnLoadKeyW(hRootKey, pszKeyPath) as LONG;
    EnablePrivilege(
        SE_RESTORE_NAME,
        core::ptr::null_mut(),
        (FALSE) as BOOL,
    );

    if regUnloadResult == ERROR_SUCCESS as LONG {
        /* refresh tree and list views */
        RefreshTreeView((*g_pChildWnd).hTreeWnd);
        pszKeyPath = GetItemPath(
            (*g_pChildWnd).hTreeWnd,
            core::ptr::null_mut(),
            &raw mut hRootKey,
        );
        RefreshListView(
            (*g_pChildWnd).hListWnd,
            hRootKey,
            pszKeyPath,
            (TRUE) as BOOL,
        );
    } else {
        ErrorMessageBox(
            hWnd,
            core::ptr::addr_of_mut!(Caption) as *mut WCHAR as LPCWSTR,
            regUnloadResult as DWORD,
        );
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn ImportRegistryFile(hWnd: HWND) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;
    let mut ofn: OPENFILENAME = core::mem::zeroed();
    let mut Caption: [WCHAR; 128] = [0; 128];
    let mut szTitle_local: [WCHAR; 512] = [0; 512];
    let mut szText: [WCHAR; 512] = [0; 512];
    let mut hKeyRoot: HKEY = core::ptr::null_mut();
    let mut pszKeyPath: LPCWSTR = core::ptr::null();

    /* Figure out in which key path we are importing */
    pszKeyPath = GetItemPath(
        (*g_pChildWnd).hTreeWnd,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKeyRoot),
    );

    InitOpenFileName(hWnd, core::ptr::addr_of_mut!(ofn), (FALSE) as BOOL);
    LoadStringW(
        hInst,
        IDS_IMPORT_REG_FILE,
        core::ptr::addr_of_mut!(Caption) as LPWSTR,
        ARRAY_SIZE!(Caption) as core::ffi::c_int,
    );
    ofn.lpstrTitle = core::ptr::addr_of_mut!(Caption) as *mut WCHAR as LPCWSTR;
    ofn.Flags |= OFN_ENABLESIZING;

    if GetOpenFileName(core::ptr::addr_of_mut!(ofn)) != 0 {
        /* Look at the extension of the file to determine its type */
        if (ofn.nFileExtension as core::ffi::c_int) >= 1
            && _wcsicmp(
                ofn.lpstrFile.offset(ofn.nFileExtension as isize) as *const u16,
                RUSTCONV_WSTR_EED2BF96A061.as_ptr(),
            ) == 0
        {
            /* Open the file */
            let mut fp: *mut FILE =
                (_wfopen(ofn.lpstrFile, RUSTCONV_WSTR_76CFE90FF2FC.as_ptr())).cast();

            /* Import it */
            if fp.is_null() || import_registry_file(fp) == 0 {
                /* Error opening the file */
                LoadStringW(
                    hInst,
                    IDS_APP_TITLE,
                    core::ptr::addr_of_mut!(szTitle_local) as LPWSTR,
                    ARRAY_SIZE!(szTitle_local) as core::ffi::c_int,
                );
                LoadStringW(
                    hInst,
                    IDS_IMPORT_ERROR,
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    ARRAY_SIZE!(szText) as core::ffi::c_int,
                );
                InfoMessageBox(
                    hWnd,
                    MB_OK | MB_ICONERROR,
                    core::ptr::addr_of_mut!(szTitle_local) as *mut WCHAR as LPCWSTR,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR,
                    ofn.lpstrFile,
                );
                bRet = (FALSE) as i32;
            } else {
                /* Show successful import */
                LoadStringW(
                    hInst,
                    IDS_APP_TITLE,
                    core::ptr::addr_of_mut!(szTitle_local) as LPWSTR,
                    ARRAY_SIZE!(szTitle_local) as core::ffi::c_int,
                );
                LoadStringW(
                    hInst,
                    IDS_IMPORT_OK,
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    ARRAY_SIZE!(szText) as core::ffi::c_int,
                );
                InfoMessageBox(
                    hWnd,
                    MB_OK | MB_ICONINFORMATION,
                    core::ptr::addr_of_mut!(szTitle_local) as *mut WCHAR as LPCWSTR,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR,
                    ofn.lpstrFile,
                );
                bRet = (TRUE) as i32;
            }

            /* Close the file */
            if !fp.is_null() {
                fclose(fp);
            }
        } else {
            /* Registry Hive Files */
            LoadStringW(
                hInst,
                IDS_QUERY_IMPORT_HIVE_CAPTION,
                core::ptr::addr_of_mut!(szTitle_local) as LPWSTR,
                ARRAY_SIZE!(szTitle_local) as core::ffi::c_int,
            );
            LoadStringW(
                hInst,
                IDS_QUERY_IMPORT_HIVE_MSG,
                core::ptr::addr_of_mut!(szText) as LPWSTR,
                ARRAY_SIZE!(szText) as core::ffi::c_int,
            );

            /* Display a confirmation message */
            if (MessageBoxW(
                (*g_pChildWnd).hWnd,
                core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR,
                core::ptr::addr_of_mut!(szTitle_local) as *mut WCHAR as LPCWSTR,
                MB_ICONWARNING | MB_YESNO,
            )) as u32 == IDYES
            {
                let mut lResult: LONG = 0;
                let mut hSubKey: HKEY = core::ptr::null_mut();

                /* Open the subkey */
                lResult = RegOpenKeyExW(
                    hKeyRoot,
                    pszKeyPath,
                    0,
                    KEY_WRITE,
                    core::ptr::addr_of_mut!(hSubKey),
                );
                if (lResult) as u32 == ERROR_SUCCESS {
                    /* Enable the 'restore' privilege, restore the hive then disable the privilege */
                    EnablePrivilege(
                        SE_RESTORE_NAME,
                        core::ptr::null(),
                        (TRUE) as BOOL,
                    );
                    lResult = RegRestoreKey(
                        hSubKey,
                        ofn.lpstrFile,
                        REG_FORCE_RESTORE,
                    );
                    EnablePrivilege(
                        SE_RESTORE_NAME,
                        core::ptr::null(),
                        (FALSE) as BOOL,
                    );

                    /* Flush the subkey and close it */
                    RegFlushKey(hSubKey);
                    RegCloseKey(hSubKey);
                }

                /* Set the return value */
                bRet = ((lResult) as u32 == ERROR_SUCCESS) as core::ffi::c_int as BOOL;

                /* Display error, if any */
                if bRet == 0 {
                    ErrorMessageBox(
                        hWnd,
                        core::ptr::addr_of_mut!(Caption) as *mut WCHAR as LPCWSTR,
                        (lResult) as DWORD,
                    );
                }
            }
        }
    } else {
        CheckCommDlgError(hWnd);
    }

    /* refresh tree and list views */
    RefreshTreeView((*g_pChildWnd).hTreeWnd);
    pszKeyPath = GetItemPath(
        (*g_pChildWnd).hTreeWnd,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKeyRoot),
    );
    RefreshListView(
        (*g_pChildWnd).hListWnd,
        hKeyRoot,
        pszKeyPath,
        (TRUE) as BOOL,
    );

    return bRet;
}

unsafe extern "system" fn ExportRegistryFile_OFNHookProc(
    mut hdlg: HWND,
    mut uiMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> UINT_PTR {
    let mut hwndExportAll: HWND = core::ptr::null_mut();
    let mut hwndExportBranch: HWND = core::ptr::null_mut();
    let mut hwndExportBranchText: HWND = core::ptr::null_mut();
    let mut iResult: UINT_PTR = 0;
    let mut pOfn: *mut OPENFILENAME = core::ptr::null_mut();
    let mut pszSelectedKey: LPWSTR = core::ptr::null_mut();
    let mut pOfnNotify: *mut OFNOTIFY = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(wParam);

    match uiMsg {
        WM_INITDIALOG => {
            pOfn = lParam as *mut OPENFILENAME;
            pszSelectedKey = (*pOfn).lCustData as LPWSTR;

            hwndExportAll = GetDlgItem(hdlg, (IDC_EXPORT_ALL as i32));
            if !hwndExportAll.is_null() {
                SendMessageW(
                    hwndExportAll,
                    BM_SETCHECK as UINT,
                    (if !pszSelectedKey.is_null() {
                        BST_UNCHECKED
                    } else {
                        BST_CHECKED
                    }) as WPARAM,
                    0 as LPARAM,
                );
            }

            hwndExportBranch = GetDlgItem(hdlg, (IDC_EXPORT_BRANCH as i32));
            if !hwndExportBranch.is_null() {
                SendMessageW(
                    hwndExportBranch,
                    BM_SETCHECK as UINT,
                    (if !pszSelectedKey.is_null() {
                        BST_CHECKED
                    } else {
                        BST_UNCHECKED
                    }) as WPARAM,
                    0 as LPARAM,
                );
            }

            hwndExportBranchText = GetDlgItem(hdlg, (IDC_EXPORT_BRANCH_TEXT as i32));
            if !hwndExportBranchText.is_null() {
                SetWindowTextW(hwndExportBranchText, pszSelectedKey);
            }
        }
        WM_NOTIFY => {
            if (*(lParam as *mut NMHDR)).code == (CDN_FILEOK) as u32 {
                pOfnNotify = lParam as *mut OFNOTIFY;
                pszSelectedKey = (*(*pOfnNotify).lpOFN).lCustData as LPWSTR;

                hwndExportBranch = GetDlgItem(hdlg, (IDC_EXPORT_BRANCH as i32));
                hwndExportBranchText = GetDlgItem(hdlg, (IDC_EXPORT_BRANCH_TEXT as i32));
                if !hwndExportBranch.is_null()
                    && !hwndExportBranchText.is_null()
                    && (SendMessageW(
                        hwndExportBranch,
                        BM_GETCHECK as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) == BST_CHECKED as LRESULT)
                {
                    GetWindowTextW(
                        hwndExportBranchText,
                        pszSelectedKey,
                        (_MAX_PATH as i32),
                    );
                } else if !pszSelectedKey.is_null() {
                    *pszSelectedKey.offset(0 as isize) = 0x0000u16;
                }
            }
        }
        _ => {}
    }
    return iResult;
}
#[no_mangle]
pub unsafe extern "C" fn ExportRegistryFile(hWnd: HWND) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;
    let mut ofn: OPENFILENAME = core::mem::zeroed();
    let mut ExportKeyPath: [WCHAR; (_MAX_PATH) as _] = [0; (_MAX_PATH) as _];
    let mut Caption: [WCHAR; 128] = [0; 128];
    let mut szTitle_local: [WCHAR; 512] = [0; 512];
    let mut szText: [WCHAR; 512] = [0; 512];
    let mut hKeyRoot: HKEY = core::ptr::null_mut();
    let mut pszKeyPath: LPCWSTR = core::ptr::null();

    /* Figure out which key path we are exporting */
    pszKeyPath = GetItemPath(
        (*g_pChildWnd).hTreeWnd,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKeyRoot),
    );
    GetKeyName(
        core::ptr::addr_of_mut!(ExportKeyPath) as *mut WCHAR,
        ARRAY_SIZE!(ExportKeyPath),
        hKeyRoot,
        pszKeyPath,
    );

    InitOpenFileName(hWnd, core::ptr::addr_of_mut!(ofn), (TRUE) as BOOL);
    LoadStringW(
        hInst,
        IDS_EXPORT_REG_FILE,
        core::ptr::addr_of_mut!(Caption) as *mut WCHAR,
        (ARRAY_SIZE!(Caption)) as i32,
    );
    ofn.lpstrTitle = core::ptr::addr_of_mut!(Caption) as *mut WCHAR as LPCWSTR;

    /* Only set the path if a key (not the root node) is selected */
    if hKeyRoot != core::ptr::null_mut() {
        ofn.lCustData = core::ptr::addr_of_mut!(ExportKeyPath) as *mut WCHAR as LPARAM;
    }
    ofn.Flags |= OFN_ENABLETEMPLATE | OFN_ENABLEHOOK;
    ofn.lpfnHook = Some(ExportRegistryFile_OFNHookProc);
    ofn.lpTemplateName = MAKEINTRESOURCEW!(IDD_EXPORTRANGE) as LPCWSTR;
    if GetSaveFileName(core::ptr::addr_of_mut!(ofn)) != 0 {
        match ofn.nFilterIndex {
            2 => {
                /* Registry Hive Files */
                let mut lResult: LONG;
                let mut hSubKey: HKEY = core::ptr::null_mut();

                /* Open the subkey */
                lResult = RegOpenKeyExW(
                    hKeyRoot,
                    pszKeyPath,
                    0,
                    KEY_READ,
                    core::ptr::addr_of_mut!(hSubKey),
                );
                if (lResult) as u32 == ERROR_SUCCESS {
                    /* Enable the 'backup' privilege, save the hive then disable the privilege */
                    EnablePrivilege(
                        SE_BACKUP_NAME,
                        core::ptr::null(),
                        (TRUE) as BOOL,
                    );
                    lResult = RegSaveKeyW(hSubKey, ofn.lpstrFile, core::ptr::null_mut());
                    if (lResult) as u32 == ERROR_ALREADY_EXISTS {
                        /*
                         * We are here, that means that we already said "yes" to the confirmation dialog.
                         * So we absolutely want to replace the hive file.
                         */
                        if DeleteFileW(ofn.lpstrFile) != 0 {
                            /* Try again */
                            lResult = RegSaveKeyW(hSubKey, ofn.lpstrFile, core::ptr::null_mut());
                        }
                    }
                    EnablePrivilege(
                        SE_BACKUP_NAME,
                        core::ptr::null(),
                        (FALSE) as BOOL,
                    );

                    if (lResult) as u32 != ERROR_SUCCESS {
                        /*
                         * If we are here, it's because RegSaveKeyW has failed for any reason.
                         * The problem is that even if it has failed, it has created or
                         * replaced the exported hive file with a new empty file. We don't
                         * want to keep this file, so we delete it.
                         */
                        DeleteFileW(ofn.lpstrFile);
                    }

                    /* Close the subkey */
                    RegCloseKey(hSubKey);
                }

                /* Set the return value */
                bRet = ((lResult) as u32 == ERROR_SUCCESS) as BOOL;

                /* Display error, if any */
                if bRet == 0 {
                    ErrorMessageBox(
                        hWnd,
                        core::ptr::addr_of_mut!(Caption) as *mut WCHAR,
                        (lResult) as DWORD,
                    );
                }

            }

            // C: `case 1: case 3: default:` sits ABOVE `case 4:`, and C still sends 4 to its
            // own case. A Rust `_` here would take 4 too (Text File export written as .reg).
            0..=1 | 3 | 5.. => {
                /* Windows Registry Editor Version 5.00 / REGEDIT4 / All files ==> use Windows Registry Editor Version 5.00 */
                if export_registry_key(
                    ofn.lpstrFile,
                    core::ptr::addr_of_mut!(ExportKeyPath) as *mut WCHAR,
                    (if ofn.nFilterIndex == 3 {
                        REG_FORMAT_4
                    } else {
                        REG_FORMAT_5
                    }),
                ) == 0
                {
                    /* Error creating the file */
                    LoadStringW(
                        hInst,
                        IDS_APP_TITLE,
                        core::ptr::addr_of_mut!(szTitle_local) as *mut WCHAR,
                        (ARRAY_SIZE!(szTitle_local)) as i32,
                    );
                    LoadStringW(
                        hInst,
                        IDS_EXPORT_ERROR,
                        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
                        (ARRAY_SIZE!(szText)) as i32,
                    );
                    InfoMessageBox(
                        hWnd,
                        MB_OK | MB_ICONERROR,
                        core::ptr::addr_of_mut!(szTitle_local) as *mut WCHAR,
                        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
                        ofn.lpstrFile,
                    );
                    bRet = (FALSE) as i32;
                } else {
                    bRet = (TRUE) as i32;
                }

            }

            4 => {
                /* Text File */
                bRet = txt_export_registry_key(
                    ofn.lpstrFile,
                    core::ptr::addr_of_mut!(ExportKeyPath) as *mut WCHAR,
                );
                if bRet == 0 {
                    /* Error creating the file */
                    LoadStringW(
                        hInst,
                        IDS_APP_TITLE,
                        core::ptr::addr_of_mut!(szTitle_local) as *mut WCHAR,
                        (ARRAY_SIZE!(szTitle_local)) as i32,
                    );
                    LoadStringW(
                        hInst,
                        IDS_EXPORT_ERROR,
                        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
                        (ARRAY_SIZE!(szText)) as i32,
                    );
                    InfoMessageBox(
                        hWnd,
                        MB_OK | MB_ICONERROR,
                        core::ptr::addr_of_mut!(szTitle_local) as *mut WCHAR,
                        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
                        ofn.lpstrFile,
                    );
                }
            }
        }
    } else {
        CheckCommDlgError(hWnd);
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn PrintRegistryHive(mut hWnd: HWND, mut path: LPWSTR) -> BOOL {
    let mut pd: PRINTDLG = core::mem::zeroed();
    UNREFERENCED_PARAMETER!(path);

    ZeroMemory!(
        &raw mut pd as *mut core::ffi::c_void,
        core::mem::size_of::<PRINTDLG>() as usize,
    );
    pd.lStructSize = core::mem::size_of::<PRINTDLG>() as DWORD;
    pd.hwndOwner = hWnd;
    pd.hDevMode = core::ptr::null_mut();
    pd.hDevNames = core::ptr::null_mut();
    pd.Flags = (PD_USEDEVMODECOPIESANDCOLLATE | PD_RETURNDC) as DWORD;
    pd.nCopies = 1 as WORD;
    pd.nFromPage = 0xFFFF as WORD;
    pd.nToPage = 0xFFFF as WORD;
    pd.nMinPage = 1 as WORD;
    pd.nMaxPage = 0xFFFF as WORD;
    if PrintDlg(&raw mut pd) != 0 {
        /* GDI calls to render output. */
        DeleteDC(pd.hDC); /* Delete DC when done.*/
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn ChooseFavorite(pszFavorite: LPCWSTR) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szFavoritePath: [WCHAR; 512] = [0; 512];
    let mut cbData: DWORD;
    let mut dwType: DWORD;

    'done: {
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            core::ptr::addr_of_mut!(s_szFavoritesRegKey) as *mut WCHAR as LPCWSTR,
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hKey),
        ) != ERROR_SUCCESS as LSTATUS
        {
            break 'done;
        }

        cbData = core::mem::size_of::<[WCHAR; 512]>() as DWORD;
        memset(
            core::ptr::addr_of_mut!(szFavoritePath) as *mut WCHAR as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<[WCHAR; 512]>() as size_t,
        );
        dwType = 0;
        if RegQueryValueExW(
            hKey,
            pszFavorite,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(szFavoritePath) as *mut WCHAR as LPBYTE,
            core::ptr::addr_of_mut!(cbData),
        ) != ERROR_SUCCESS as LSTATUS
        {
            break 'done;
        }

        if dwType == REG_SZ as DWORD {
            SelectNode(
                (*g_pChildWnd).hTreeWnd,
                core::ptr::addr_of_mut!(szFavoritePath) as *mut WCHAR as LPCWSTR,
            );
        }
    }
    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn GetItemFullPath(hTI: HTREEITEM) -> LPWSTR {
    let mut hRoot: HKEY = core::ptr::null_mut();
    let mut rootname: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut buffer: *mut WCHAR;
    let mut rootlen: SIZE_T;
    let mut subkeylen: SIZE_T;
    let subkey: LPCWSTR = GetItemPath((*g_pChildWnd).hTreeWnd, hTI, core::ptr::addr_of_mut!(hRoot));
    if subkey.is_null() || hRoot.is_null() {
        return core::ptr::null_mut();
    }
    if GetKeyName(
        core::ptr::addr_of_mut!(rootname) as LPWSTR,
        ARRAY_SIZE!(rootname),
        hRoot,
        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
    ) == 0
    {
        return core::ptr::null_mut();
    }
    rootlen = (lstrlenW(core::ptr::addr_of_mut!(rootname) as LPCWSTR) + 1) as SIZE_T;
    subkeylen = lstrlenW(subkey) as SIZE_T;
    buffer = malloc(
        ((rootlen + subkeylen + 1) * (core::mem::size_of::<WCHAR>() as SIZE_T)) as _,
    ) as *mut WCHAR;
    if !buffer.is_null() {
        lstrcpyW(buffer as LPWSTR, core::ptr::addr_of_mut!(rootname) as LPCWSTR);
        *buffer.offset((rootlen - 1) as isize) = '\\' as i32 as WCHAR;
        lstrcpyW(buffer.offset(rootlen as isize) as LPWSTR, subkey);
    }
    return buffer as LPWSTR;
}
unsafe extern "system" fn AddToFavoritesDlgProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut hName: HWND = GetDlgItem(hWnd, (IDC_FAVORITENAME as i32));
    let mut name: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    match uMsg {
        WM_INITDIALOG => {
            let mut tvi: TVITEM = core::mem::zeroed();
            tvi.mask = (TVIF_HANDLE | TVIF_TEXT) as UINT;
            tvi.hItem = TreeView_GetSelection!((*g_pChildWnd).hTreeWnd);
            tvi.pszText = core::ptr::addr_of_mut!(name) as LPWSTR;
            tvi.cchTextMax = _countof!(name) as core::ffi::c_int;
            if TreeView_GetItem!((*g_pChildWnd).hTreeWnd, core::ptr::addr_of_mut!(tvi)) == 0 {
                *tvi.pszText.offset(0 as isize) = UNICODE_NULL as WCHAR;
            }
            SetWindowTextW(hName, tvi.pszText as LPCWSTR);
            SendMessageW(
                hName,
                EM_LIMITTEXT as UINT,
                (_countof!(name) as usize - 1) as WPARAM,
                0 as LPARAM,
            );
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as i32) as u32 {
                IDOK => {
                    let mut path: LPWSTR = core::ptr::null_mut();
                    let mut hKey: HKEY = core::ptr::null_mut();
                    let mut err: DWORD = 0;
                    'tail: {
                        'fail: {
                            if GetWindowTextW(
                                hName,
                                core::ptr::addr_of_mut!(name) as LPWSTR,
                                _countof!(name) as core::ffi::c_int,
                            ) == 0
                            {
                                err = GetLastError();
                                break 'fail;
                            }
                            path = GetItemFullPath(core::ptr::null_mut());
                            if path.is_null() {
                                err = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                                break 'fail;
                            }
                            err = RegCreateKeyExW(
                                HKEY_CURRENT_USER,
                                core::ptr::addr_of!(s_szFavoritesRegKey) as LPCWSTR,
                                0,
                                core::ptr::null_mut(),
                                0,
                                KEY_SET_VALUE,
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(hKey),
                                core::ptr::null_mut(),
                            ) as DWORD;
                            if err != 0 {
                                break 'fail;
                            }
                            err = RegSetValueExW(
                                hKey,
                                core::ptr::addr_of_mut!(name) as LPCWSTR,
                                0,
                                REG_SZ,
                                path as *mut BYTE,
                                ((lstrlenW(path as LPCWSTR) + 1) as usize)
                                    .wrapping_mul(core::mem::size_of::<WCHAR>())
                                    as DWORD,
                            ) as DWORD;
                            RegCloseKey(hKey);
                            if err == 0 {
                                break 'tail;
                            }
                        }
                        ErrorBox(hWnd, err as UINT);
                    }
                    free(path as *mut core::ffi::c_void);
                    return EndDialog(hWnd, err as INT_PTR) as INT_PTR;
                }
                IDCANCEL => {
                    return EndDialog(hWnd, (ERROR_CANCELLED) as INT_PTR) as INT_PTR;
                }
                IDC_FAVORITENAME => {
                    if HIWORD!(wParam) as UINT == EN_UPDATE {
                        EnableWindow(
                            GetDlgItem(hWnd, (IDOK) as i32),
                            (GetWindowTextLengthW(hName) != 0) as core::ffi::c_int,
                        );
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn CopyKeyName(hWnd: HWND, hRootKey: HKEY, keyName: LPCWSTR) -> BOOL {
    let mut bClipboardOpened: BOOL = (FALSE) as i32;
    let mut bSuccess: BOOL = (FALSE) as i32;
    let mut szBuffer: [WCHAR; 512] = [0; 512];
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();
    let mut s: LPWSTR = core::ptr::null_mut();
    let mut cbGlobal: SIZE_T = 0;
    'done: {
        if OpenClipboard(hWnd) == 0 {
            break 'done;
        }
        bClipboardOpened = (TRUE) as i32;

        if EmptyClipboard() == 0 {
            break 'done;
        }

        if GetKeyName(
            core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
            ARRAY_SIZE!(szBuffer),
            hRootKey,
            keyName,
        ) == 0
        {
            break 'done;
        }

        cbGlobal = ((wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR) as SIZE_T)
            + 1 as SIZE_T)
            * core::mem::size_of::<WCHAR>() as SIZE_T;
        hGlobal = GlobalAlloc(GMEM_MOVEABLE as UINT, cbGlobal);
        if hGlobal.is_null() {
            break 'done;
        }

        s = GlobalLock(hGlobal) as LPWSTR;
        StringCbCopyW(
            s as STRSAFE_LPWSTR,
            cbGlobal as size_t,
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as STRSAFE_LPCWSTR,
        );
        GlobalUnlock(hGlobal);

        SetClipboardData(CF_UNICODETEXT as UINT, hGlobal as HANDLE);
        bSuccess = (TRUE) as i32;
    }
    if bClipboardOpened != 0 {
        CloseClipboard();
    }
    return bSuccess;
}

unsafe extern "C" fn CreateNewValue(hRootKey: HKEY, pszKeyPath: LPCWSTR, dwType: DWORD) -> BOOL {
    let mut szNewValueFormat: [WCHAR; 128] = [0; 128];
    let mut szNewValue: [WCHAR; 128] = [0; 128];
    let mut iIndex: core::ffi::c_int = 1;
    let mut data: [BYTE; 128] = [0; 128];
    let mut dwExistingType: DWORD = 0;
    let mut cbData: DWORD = 0;
    let mut lResult: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lvfi: LVFINDINFOW = core::mem::zeroed();

    if RegOpenKeyExW(
        hRootKey,
        pszKeyPath,
        0,
        (KEY_QUERY_VALUE | KEY_SET_VALUE) as REGSAM,
        &raw mut hKey,
    ) != ERROR_SUCCESS as LONG
    {
        return (FALSE) as BOOL;
    }

    LoadStringW(
        hInst,
        IDS_NEW_VALUE as UINT,
        szNewValueFormat.as_mut_ptr(),
        ARRAY_SIZE!(szNewValueFormat) as core::ffi::c_int,
    );

    loop {
        let fresh_index = iIndex;
        iIndex += 1;
        wsprintf(
            szNewValue.as_mut_ptr(),
            szNewValueFormat.as_ptr(),
            fresh_index,
        );
        cbData = core::mem::size_of_val(&data) as DWORD;
        lResult = RegQueryValueExW(
            hKey,
            szNewValue.as_ptr(),
            core::ptr::null_mut(),
            &raw mut dwExistingType,
            data.as_mut_ptr(),
            &raw mut cbData,
        );
        if !(lResult == ERROR_SUCCESS as LONG) {
            break;
        }
    }

    match dwType {
        _ if dwType == REG_DWORD => {
            cbData = core::mem::size_of::<DWORD>() as DWORD;
        }
        _ if dwType == REG_SZ || dwType == REG_EXPAND_SZ => {
            cbData = core::mem::size_of::<WCHAR>() as DWORD;
        }
        _ if dwType == REG_MULTI_SZ => {
            /*
             * WARNING: An empty multi-string has only one null char.
             * Indeed, multi-strings are built in the following form:
             * str1\0str2\0...strN\0\0
             * where each strI\0 is a null-terminated string, and it
             * ends with a terminating empty string.
             * Therefore an empty multi-string contains only the terminating
             * empty string, that is, one null char.
             */
            cbData = core::mem::size_of::<WCHAR>() as DWORD;
        }
        _ if dwType == REG_QWORD => {
            /* REG_QWORD_LITTLE_ENDIAN */
            cbData = core::mem::size_of::<DWORDLONG>() as DWORD; // == sizeof(DWORD) * 2;
        }
        _ => {
            cbData = 0 as DWORD;
        }
    }

    memset(
        &raw mut data as *mut core::ffi::c_void,
        0,
        cbData as usize,
    );
    lResult = RegSetValueExW(
        hKey,
        szNewValue.as_ptr(),
        0 as DWORD,
        dwType,
        data.as_mut_ptr(),
        cbData,
    );
    RegCloseKey(hKey);
    if lResult != ERROR_SUCCESS as LONG {
        return (FALSE) as BOOL;
    }

    RefreshListView(
        (*g_pChildWnd).hListWnd,
        hRootKey,
        pszKeyPath,
        (TRUE) as BOOL,
    );

    /* locate the newly added value, and get ready to rename it */
    memset(
        &raw mut lvfi as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<LVFINDINFOW>(),
    );
    lvfi.flags = LVFI_STRING as UINT;
    lvfi.psz = szNewValue.as_ptr();
    iIndex = ListView_FindItem!(
        (*g_pChildWnd).hListWnd,
        -1,
        &raw mut lvfi,
    );
    if iIndex >= 0 {
        ListView_SetItemState!(
            (*g_pChildWnd).hListWnd,
            iIndex,
            (LVIS_SELECTED | LVIS_FOCUSED) as UINT,
            (LVIS_SELECTED | LVIS_FOCUSED) as UINT,
        );
        ListView_EnsureVisible!(
            (*g_pChildWnd).hListWnd,
            iIndex,
            FALSE,
        );
        let _ = ListView_EditLabel!((*g_pChildWnd).hListWnd, iIndex);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitializeRemoteRegistryPicker(
    pDsObjectPicker: *mut *mut IDsObjectPicker,
) -> HRESULT {
    let mut hRet: HRESULT;

    *pDsObjectPicker = core::ptr::null_mut();

    hRet = CoCreateInstance(
        &raw const CLSID_DsObjectPicker,
        core::ptr::null_mut::<IUnknown>(),
        CLSCTX_INPROC_SERVER as DWORD,
        &raw const IID_IDsObjectPicker,
        pDsObjectPicker as *mut LPVOID,
    );
    if hRet >= 0 {
        let mut InitInfo: DSOP_INIT_INFO = DSOP_INIT_INFO {
            cbSize: 0,
            pwzTargetComputer: core::ptr::null::<WCHAR>(),
            cDsScopeInfos: 0,
            aDsScopeInfos: core::ptr::null_mut::<DSOP_SCOPE_INIT_INFO>(),
            flOptions: 0,
            cAttributesToFetch: 0,
            apwzAttributeNames: core::ptr::null_mut::<PCWSTR>(),
        };
        static mut Scopes: [DSOP_SCOPE_INIT_INFO; 1] = [DSOP_SCOPE_INIT_INFO {
            cbSize: core::mem::size_of::<DSOP_SCOPE_INIT_INFO>() as ULONG,
            flType: (DSOP_SCOPE_TYPE_USER_ENTERED_UPLEVEL_SCOPE
                | DSOP_SCOPE_TYPE_USER_ENTERED_DOWNLEVEL_SCOPE
                | DSOP_SCOPE_TYPE_GLOBAL_CATALOG
                | DSOP_SCOPE_TYPE_EXTERNAL_UPLEVEL_DOMAIN
                | DSOP_SCOPE_TYPE_EXTERNAL_DOWNLEVEL_DOMAIN
                | DSOP_SCOPE_TYPE_WORKGROUP
                | DSOP_SCOPE_TYPE_UPLEVEL_JOINED_DOMAIN
                | DSOP_SCOPE_TYPE_DOWNLEVEL_JOINED_DOMAIN) as ULONG,
            flScope: 0 as ULONG,
            FilterFlags: _DSOP_FILTER_FLAGS {
                Uplevel: _DSOP_UPLEVEL_FILTER_FLAGS {
                    flBothModes: DSOP_FILTER_COMPUTERS as ULONG,
                    flMixedModeOnly: 0 as ULONG,
                    flNativeModeOnly: 0 as ULONG,
                },
                flDownlevel: DSOP_DOWNLEVEL_FILTER_COMPUTERS as ULONG,
            },
            pwzDcName: core::ptr::null::<WCHAR>(),
            pwzADsPath: core::ptr::null::<WCHAR>(),
            hr: S_OK,
        }];

        InitInfo.cbSize = core::mem::size_of::<DSOP_INIT_INFO>() as ULONG;
        InitInfo.pwzTargetComputer = core::ptr::null::<WCHAR>();
        InitInfo.cDsScopeInfos = ARRAY_SIZE!(Scopes) as ULONG;
        InitInfo.aDsScopeInfos = &raw mut Scopes as *mut DSOP_SCOPE_INIT_INFO;
        InitInfo.flOptions = 0 as ULONG;
        InitInfo.cAttributesToFetch = 0 as ULONG;
        InitInfo.apwzAttributeNames = core::ptr::null_mut::<PCWSTR>();

        hRet = (*(**pDsObjectPicker).lpVtbl)
            .Initialize
            .unwrap_unchecked()(*pDsObjectPicker, &raw mut InitInfo);

        if hRet < 0 {
            /* delete the object picker in case initialization failed! */
            (*(**pDsObjectPicker).lpVtbl)
                .Release
                .unwrap_unchecked()(*pDsObjectPicker);
        }
    }

    return hRet;
}

unsafe extern "C" fn InvokeRemoteRegistryPickerDialog(
    mut pDsObjectPicker: *mut IDsObjectPicker,
    mut hwndParent: HWND,
    mut lpBuffer: LPWSTR,
    mut uSize: UINT,
) -> HRESULT {
    let mut pdo: *mut IDataObject = core::ptr::null_mut::<IDataObject>();
    let mut hRet: HRESULT = 0;

    hRet = ((*(*pDsObjectPicker).lpVtbl).InvokeDialog.unwrap_unchecked())(
        pDsObjectPicker,
        hwndParent,
        core::ptr::addr_of_mut!(pdo),
    );
    if hRet == S_OK {
        let mut stm: STGMEDIUM = core::mem::zeroed();
        let mut fe: FORMATETC = core::mem::zeroed();

        fe.cfFormat = RegisterClipboardFormatW(CFSTR_DSOP_DS_SELECTION_LIST) as CLIPFORMAT;
        fe.ptd = core::ptr::null_mut::<DVTARGETDEVICE>();
        fe.dwAspect = DVASPECT_CONTENT as DWORD;
        fe.lindex = -(1 as LONG);
        fe.tymed = TYMED_HGLOBAL as DWORD;

        hRet = ((*(*pdo).lpVtbl).GetData.unwrap_unchecked())(
            pdo,
            core::ptr::addr_of_mut!(fe),
            core::ptr::addr_of_mut!(stm),
        );
        if hRet >= 0 {
            let mut SelectionList: PDS_SELECTION_LIST =
                GlobalLock(stm.__bindgen_anon_1.hGlobal) as PDS_SELECTION_LIST;
            if !SelectionList.is_null() {
                if (*SelectionList).cItems == 1 as ULONG {
                    let mut nlen: size_t = wcslen(
                        (*(*SelectionList).aDsSelection.as_ptr().add(0 as usize)).pwzName,
                    ) as size_t;
                    if nlen >= uSize as size_t {
                        nlen = (uSize as size_t) - (1 as size_t);
                    }

                    memcpy(
                        lpBuffer as *mut core::ffi::c_void,
                        (*(*SelectionList).aDsSelection.as_ptr().add(0 as usize)).pwzName
                            as *const core::ffi::c_void,
                        nlen * (core::mem::size_of::<WCHAR>() as size_t),
                    );

                    *lpBuffer.add(nlen) = 0x0000u16;
                }

                GlobalUnlock(stm.__bindgen_anon_1.hGlobal);
            }

            ReleaseStgMedium(core::ptr::addr_of_mut!(stm));
        }

        ((*(*pdo).lpVtbl).Release.unwrap_unchecked())(pdo);
    }

    return hRet;
}

unsafe extern "C" fn FreeObjectPicker(pDsObjectPicker: *mut IDsObjectPicker) {
    (((*(*pDsObjectPicker).lpVtbl).Release).unwrap_unchecked())(pDsObjectPicker);
}

unsafe extern "C" fn _CmdWndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    let mut hKeyRoot: HKEY = 0 as HKEY;
    let mut hKey: HKEY = 0 as HKEY;
    let mut keyPath: LPCWSTR = core::ptr::null::<WCHAR>();
    let mut valueName: LPCWSTR = core::ptr::null::<WCHAR>();
    let mut result: BOOL = TRUE as BOOL;
    let mut regsam: REGSAM = KEY_READ as REGSAM;
    let mut item: ::core::ffi::c_int = 0;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(message);

    match (LOWORD!(wParam)) as u32 {
        ID_REGISTRY_LOADHIVE => {
            LoadHive(hWnd);
            return (TRUE) as BOOL;
        }
        ID_REGISTRY_UNLOADHIVE => {
            UnloadHive(hWnd);
            return (TRUE) as BOOL;
        }
        ID_REGISTRY_IMPORTREGISTRYFILE => {
            ImportRegistryFile(hWnd);
            return (TRUE) as BOOL;
        }
        ID_REGISTRY_EXPORTREGISTRYFILE => {
            ExportRegistryFile(hWnd);
            return (TRUE) as BOOL;
        }
        ID_REGISTRY_CONNECTNETWORKREGISTRY => {
            let mut ObjectPicker: *mut IDsObjectPicker = core::ptr::null_mut::<IDsObjectPicker>();
            let mut szComputerName: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as _] =
                [0; (MAX_COMPUTERNAME_LENGTH + 1) as _];
            let mut hRet: HRESULT = 0;

            hRet = CoInitialize(core::ptr::null_mut());
            if ((hRet) >= 0) {
                hRet = InitializeRemoteRegistryPicker(core::ptr::addr_of_mut!(ObjectPicker));
                if ((hRet) >= 0) {
                    hRet = InvokeRemoteRegistryPickerDialog(
                        ObjectPicker,
                        hWnd,
                        core::ptr::addr_of_mut!(szComputerName) as LPWSTR,
                        (ARRAY_SIZE!(szComputerName)) as UINT,
                    );
                    if hRet == S_OK {
                        // FIXME - connect to the registry
                    }

                    FreeObjectPicker(ObjectPicker);
                }

                CoUninitialize();
            }

            return (TRUE) as BOOL;
        }
        ID_REGISTRY_DISCONNECTNETWORKREGISTRY => {
            return (TRUE) as BOOL;
        }
        ID_REGISTRY_PRINT => {
            PrintRegistryHive(hWnd, RUSTCONV_WSTR_96A296D224F2.as_ptr().cast_mut());
            return (TRUE) as BOOL;
        }
        ID_REGISTRY_EXIT => {
            DestroyWindow(hWnd);
            return (TRUE) as BOOL;
        }
        ID_VIEW_STATUSBAR => {
            toggle_child(hWnd, LOWORD!(wParam) as UINT, hStatusBar);
            return (TRUE) as BOOL;
        }
        ID_FAVOURITES_ADDTOFAVOURITES => {
            DialogBoxW!(hInst, MAKEINTRESOURCEW!(IDD_ADDFAVORITES), hWnd, Some(AddToFavoritesDlgProc));
            return (TRUE) as BOOL;
        }
        ID_HELP_HELPTOPICS => {
            WinHelpW(hWnd, RUSTCONV_WSTR_E89B9A83645C.as_ptr(), HELP_FINDER, 0);
            return (TRUE) as BOOL;
        }
        ID_HELP_ABOUT => {
            ShowAboutBox(hWnd);
            return (TRUE) as BOOL;
        }
        ID_VIEW_SPLIT => {
            let mut rt: RECT = core::mem::zeroed();
            let mut pt: POINT = core::mem::zeroed();
            let mut pts: POINT = core::mem::zeroed();
            GetClientRect((*g_pChildWnd).hWnd, core::ptr::addr_of_mut!(rt));
            pt.x = rt.left + (*g_pChildWnd).nSplitPos as LONG;
            pt.y = (rt.bottom / 2);
            pts = pt;
            if ClientToScreen((*g_pChildWnd).hWnd, core::ptr::addr_of_mut!(pts)) != 0 {
                SetCursorPos(pts.x, pts.y);
                SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_SIZEWE as usize as LPCWSTR));
                SendMessageW((*g_pChildWnd).hWnd, WM_LBUTTONDOWN, 0, MAKELPARAM!(pt.x, pt.y));
            }
            return (TRUE) as BOOL;
        }
        ID_EDIT_RENAME | ID_EDIT_MODIFY | ID_EDIT_MODIFY_BIN | ID_EDIT_DELETE => {
            regsam |= KEY_WRITE as REGSAM;
        }
        _ => {}
    }

    keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKeyRoot));
    valueName = GetValueName((*g_pChildWnd).hListWnd, -1);
    if !keyPath.is_null() {
        if (RegOpenKeyExW(hKeyRoot, keyPath, 0, regsam, core::ptr::addr_of_mut!(hKey))) as u32 != ERROR_SUCCESS {
            hKey = 0 as HKEY;
        }
    }

    match (LOWORD!(wParam)) as u32 {
        ID_EDIT_MODIFY => {
            if !valueName.is_null() && ModifyValue(hWnd, hKey, valueName, (FALSE) as BOOL) != 0 {
                RefreshListView((*g_pChildWnd).hListWnd, hKeyRoot, keyPath, (FALSE) as BOOL);
            }
        }
        ID_EDIT_MODIFY_BIN => {
            if !valueName.is_null() && ModifyValue(hWnd, hKey, valueName, (TRUE) as BOOL) != 0 {
                RefreshListView((*g_pChildWnd).hListWnd, hKeyRoot, keyPath, (FALSE) as BOOL);
            }
        }
        ID_EDIT_RENAME => {
            if GetFocus() == (*g_pChildWnd).hListWnd {
                if ListView_GetSelectedCount!((*g_pChildWnd).hListWnd) == 1 {
                    item = ListView_GetNextItem!((*g_pChildWnd).hListWnd, -1, LVNI_SELECTED);
                    if item > -1 {
                        ListView_EditLabel!((*g_pChildWnd).hListWnd, item);
                    }
                }
            } else if GetFocus() == (*g_pChildWnd).hTreeWnd {
                /* Get focused entry of treeview (if any) */
                let mut hItem: HTREEITEM = TreeView_GetSelection!((*g_pChildWnd).hTreeWnd);
                if !hItem.is_null() {
                    TreeView_EditLabel!((*g_pChildWnd).hTreeWnd, hItem);
                }
            }
        }
        ID_EDIT_DELETE => {
            if GetFocus() == (*g_pChildWnd).hListWnd && !hKey.is_null() {
                let mut nSelected: UINT = ListView_GetSelectedCount!((*g_pChildWnd).hListWnd);
                if nSelected >= 1 {
                    let mut msg: [WCHAR; 128] = [0; 128];
                    let mut caption: [WCHAR; 128] = [0; 128];
                    LoadStringW(hInst, IDS_QUERY_DELETE_CONFIRM, core::ptr::addr_of_mut!(caption) as LPWSTR, (ARRAY_SIZE!(caption)) as i32);
                    LoadStringW(
                        hInst,
                        (if nSelected == 1 { IDS_QUERY_DELETE_ONE } else { IDS_QUERY_DELETE_MORE }),
                        core::ptr::addr_of_mut!(msg) as LPWSTR,
                        (ARRAY_SIZE!(msg)) as i32,
                    );
                    if (MessageBoxW(
                        (*g_pChildWnd).hWnd,
                        core::ptr::addr_of_mut!(msg) as LPCWSTR,
                        core::ptr::addr_of_mut!(caption) as LPCWSTR,
                        MB_ICONQUESTION | MB_YESNO,
                    )) as u32 == IDYES
                    {
                        let mut ni: ::core::ffi::c_int = 0;
                        let mut errs: ::core::ffi::c_int = 0;

                        item = -1;
                        errs = 0;
                        loop {
                            ni = ListView_GetNextItem!((*g_pChildWnd).hListWnd, item, LVNI_SELECTED);
                            if !(ni > -1) {
                                break;
                            }
                            valueName = GetValueName((*g_pChildWnd).hListWnd, item);
                            if (RegDeleteValueW(hKey, valueName)) as u32 != ERROR_SUCCESS {
                                errs += 1;
                            }
                            item = ni;
                        }

                        RefreshListView((*g_pChildWnd).hListWnd, hKeyRoot, keyPath, (FALSE) as BOOL);
                        if errs > 0 {
                            LoadStringW(hInst, IDS_ERR_DELVAL_CAPTION, core::ptr::addr_of_mut!(caption) as LPWSTR, (ARRAY_SIZE!(caption)) as i32);
                            LoadStringW(hInst, IDS_ERR_DELETEVALUE, core::ptr::addr_of_mut!(msg) as LPWSTR, (ARRAY_SIZE!(msg)) as i32);
                            MessageBoxW(
                                (*g_pChildWnd).hWnd,
                                core::ptr::addr_of_mut!(msg) as LPCWSTR,
                                core::ptr::addr_of_mut!(caption) as LPCWSTR,
                                MB_ICONSTOP,
                            );
                        }
                    }
                }
            } else if GetFocus() == (*g_pChildWnd).hTreeWnd {
                if keyPath.is_null() || *keyPath == UNICODE_NULL {
                    MessageBeep(MB_ICONHAND);
                } else if DeleteKey(hWnd, hKeyRoot, keyPath) != 0 {
                    DeleteNode((*g_pChildWnd).hTreeWnd, core::ptr::null_mut());
                    RefreshTreeView((*g_pChildWnd).hTreeWnd);
                }
            }
        }
        ID_EDIT_NEW_STRINGVALUE => {
            CreateNewValue(hKeyRoot, keyPath, REG_SZ);
        }
        ID_EDIT_NEW_BINARYVALUE => {
            CreateNewValue(hKeyRoot, keyPath, REG_BINARY);
        }
        ID_EDIT_NEW_DWORDVALUE => {
            CreateNewValue(hKeyRoot, keyPath, REG_DWORD);
        }
        ID_EDIT_NEW_MULTISTRINGVALUE => {
            CreateNewValue(hKeyRoot, keyPath, REG_MULTI_SZ);
        }
        ID_EDIT_NEW_EXPANDABLESTRINGVALUE => {
            CreateNewValue(hKeyRoot, keyPath, REG_EXPAND_SZ);
        }
        ID_EDIT_FIND => {
            FindDialog(hWnd);
        }
        ID_EDIT_FINDNEXT => {
            FindNextMessageBox(hWnd);
        }
        ID_EDIT_COPYKEYNAME => {
            CopyKeyName(hWnd, hKeyRoot, keyPath);
        }
        ID_EDIT_PERMISSIONS => {
            RegKeyEditPermissions(hWnd, hKeyRoot, core::ptr::null::<WCHAR>(), keyPath);
        }
        ID_VIEW_REFRESH => {
            RefreshTreeView((*g_pChildWnd).hTreeWnd);
            keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKeyRoot));
            RefreshListView((*g_pChildWnd).hListWnd, hKeyRoot, keyPath, (TRUE) as BOOL);
        }
        //case ID_OPTIONS_TOOLBAR:
        //    toggle_child(hWnd, LOWORD(wParam), hToolBar);
        //    break;
        ID_EDIT_NEW_KEY => {
            CreateNewKey((*g_pChildWnd).hTreeWnd, TreeView_GetSelection!((*g_pChildWnd).hTreeWnd));
        }
        ID_TREE_EXPANDBRANCH => {
            TreeView_Expand!((*g_pChildWnd).hTreeWnd, TreeView_GetSelection!((*g_pChildWnd).hTreeWnd), TVE_EXPAND);
        }
        ID_TREE_COLLAPSEBRANCH => {
            TreeView_Expand!((*g_pChildWnd).hTreeWnd, TreeView_GetSelection!((*g_pChildWnd).hTreeWnd), TVE_COLLAPSE);
        }
        ID_TREE_RENAME => {
            SetFocus((*g_pChildWnd).hTreeWnd);
            TreeView_EditLabel!((*g_pChildWnd).hTreeWnd, TreeView_GetSelection!((*g_pChildWnd).hTreeWnd));
        }
        ID_TREE_DELETE => {
            keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, TreeView_GetSelection!((*g_pChildWnd).hTreeWnd), core::ptr::addr_of_mut!(hKeyRoot));
            if keyPath.is_null() || *keyPath == 0 {
                MessageBeep(MB_ICONHAND);
            } else if DeleteKey(hWnd, hKeyRoot, keyPath) != 0 {
                DeleteNode((*g_pChildWnd).hTreeWnd, core::ptr::null_mut());
            }
        }
        ID_TREE_EXPORT => {
            ExportRegistryFile((*g_pChildWnd).hTreeWnd);
        }
        ID_TREE_PERMISSIONS => {
            keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, TreeView_GetSelection!((*g_pChildWnd).hTreeWnd), core::ptr::addr_of_mut!(hKeyRoot));
            RegKeyEditPermissions(hWnd, hKeyRoot, core::ptr::null::<WCHAR>(), keyPath);
        }
        ID_SWITCH_PANELS => {
            let mut bShiftDown: BOOL = (GetKeyState(VK_SHIFT as ::core::ffi::c_int) < 0) as BOOL;
            let mut hwndItem: HWND = GetNextDlgTabItem((*g_pChildWnd).hWnd, GetFocus(), bShiftDown);
            if hwndItem == (*g_pChildWnd).hAddressBarWnd {
                PostMessageW(hwndItem, EM_SETSEL, 0, -1);
            }
            SetFocus(hwndItem);
        }
        ID_ADDRESS_FOCUS => {
            SendMessageW((*g_pChildWnd).hAddressBarWnd, EM_SETSEL, 0, -1);
            SetFocus((*g_pChildWnd).hAddressBarWnd);
        }
        _ => {
            if LOWORD!(wParam) >= (ID_FAVORITES_MIN as u16) && LOWORD!(wParam) <= (ID_FAVORITES_MAX as u16) {
                let mut hMenu: HMENU = core::ptr::null_mut::<HMENU__>();
                let mut mii: MENUITEMINFOW = core::mem::zeroed();
                let mut szFavorite: [WCHAR; 512] = [0; 512];

                hMenu = GetSubMenu(GetMenu(hWnd), FAVORITES_MENU_POSITION);

                memset((core::ptr::addr_of_mut!(mii)).cast(), 0, core::mem::size_of::<MENUITEMINFOW>());
                mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
                mii.fMask = MIIM_TYPE;
                mii.fType = MFT_STRING;
                mii.dwTypeData = core::ptr::addr_of_mut!(szFavorite) as LPWSTR;
                mii.cch = ARRAY_SIZE!(szFavorite) as UINT;

                if GetMenuItemInfo(
                    hMenu,
                    (LOWORD!(wParam) as ::core::ffi::c_int - (ID_FAVORITES_MIN as i32)) as UINT,
                    TRUE as BOOL,
                    core::ptr::addr_of_mut!(mii),
                ) != 0
                {
                    ChooseFavorite(core::ptr::addr_of_mut!(szFavorite) as LPCWSTR);
                }
            } else if LOWORD!(wParam) >= (ID_TREE_SUGGESTION_MIN as u16) && LOWORD!(wParam) <= (ID_TREE_SUGGESTION_MAX as u16) {
                let mut wID: WORD = LOWORD!(wParam);
                let mut s: LPCWSTR = core::ptr::addr_of_mut!(Suggestions) as *mut WCHAR as LPCWSTR;
                while wID > (ID_TREE_SUGGESTION_MIN as u16) {
                    if *s != 0 {
                        s = s.add((wcslen(s) + 1) as usize);
                    }
                    wID = wID.wrapping_sub(1);
                }
                SelectNode((*g_pChildWnd).hTreeWnd, s);
            } else {
                result = FALSE as BOOL;
            }
        }
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    return result;
}

#[no_mangle]
pub unsafe extern "system" fn FrameWndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut rc: RECT = core::mem::zeroed();
    match message {
        WM_CREATE => {
            // For now, the Help dialog item is disabled because of lacking of HTML Help support
            EnableMenuItem(
                GetMenu(hWnd),
                ID_HELP_HELPTOPICS,
                MF_BYCOMMAND | MF_GRAYED,
            );
            GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
            CreateWindowExW(
                0,
                core::ptr::addr_of_mut!(szChildClass) as LPCWSTR,
                core::ptr::null_mut(),
                WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                rc.left,
                rc.top,
                rc.right - rc.left,
                rc.bottom - rc.top,
                hWnd,
                0 as HMENU,
                hInst,
                core::ptr::null_mut(),
            );
        }
        WM_COMMAND => {
            if _CmdWndProc(hWnd, message, wParam, lParam) == 0 {
                return DefWindowProcW(hWnd, message, wParam, lParam);
            }
        }
        WM_ACTIVATE => {
            if (LOWORD!(wParam)) as u32 != WA_INACTIVE && !g_pChildWnd.is_null() {
                SetFocus((*g_pChildWnd).hWnd);
            }
        }
        WM_SIZE => {
            resize_frame_client(hWnd);
        }
        WM_INITMENU => {
            OnInitMenu(hWnd);
        }
        WM_ENTERMENULOOP => {
            OnEnterMenuLoop(hWnd);
        }
        WM_EXITMENULOOP => {
            OnExitMenuLoop(hWnd);
        }
        WM_MENUSELECT => {
            OnMenuSelect(hWnd, (LOWORD!(wParam)) as UINT, (HIWORD!(wParam)) as UINT, lParam as HMENU);
        }
        WM_SYSCOLORCHANGE => {
            // Forward WM_SYSCOLORCHANGE to common controls
            SendMessageW(
                (*g_pChildWnd).hListWnd,
                WM_SYSCOLORCHANGE,
                0,
                0,
            );
            SendMessageW(
                (*g_pChildWnd).hTreeWnd,
                WM_SYSCOLORCHANGE,
                0,
                0,
            );
        }
        WM_DESTROY => {
            WinHelpW(
                hWnd,
                RUSTCONV_WSTR_E89B9A83645C.as_ptr(),
                HELP_QUIT,
                0,
            );
            SaveSettings();
            PostQuitMessage(0);
            // fall through to default
            return DefWindowProcW(hWnd, message, wParam, lParam);
        }
        _ => {
            return DefWindowProcW(hWnd, message, wParam, lParam);
        }
    }
    return 0;
}

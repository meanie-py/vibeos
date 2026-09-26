//! `dll/win32/shell32/dialogs/general.cpp`, translated and FLIPPED — the C is commented out
//! of the CMake source list and this module is what shell32 links. It was the first shell32
//! TU to take its substrate from the bindgen island (`crate::prelude`) instead of the
//! hand-written `types.rs`/`win32.rs`; see prelude.rs for why that is "instead of", not
//! "as well as". A PAID artifact — do not delete it to tidy up.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_079544FA595D: [WCHAR; 42] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0065, 0x0074, 0x0020, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x004D, 0x0061, 0x0069, 0x006E, 0x0000];
pub static CPP_COM_WSTR_09815BC5A072: [WCHAR; 17] = [0x0041, 0x006E, 0x0063, 0x0068, 0x006F, 0x0072, 0x0020, 0x0055, 0x006E, 0x0064, 0x0065, 0x0072, 0x006C, 0x0069, 0x006E, 0x0065, 0x0000];
pub static CPP_COM_WSTR_1CA266DE5730: [WCHAR; 14] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0055, 0x006E, 0x0064, 0x0065, 0x0072, 0x006C, 0x0069, 0x006E, 0x0065, 0x0000];
pub static CPP_COM_WSTR_203E168CD5DB: [WCHAR; 11] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0053, 0x0074, 0x0061, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_833AD7EF6992: [WCHAR; 3] = [0x006E, 0x006F, 0x0000];
pub static CPP_COM_WSTR_9CF479D14C5E: [WCHAR; 6] = [0x0068, 0x006F, 0x0076, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_9F585864DA1D: [WCHAR; 15] = [0x005C, 0x0049, 0x0063, 0x006F, 0x006E, 0x0055, 0x006E, 0x0064, 0x0065, 0x0072, 0x006C, 0x0069, 0x006E, 0x0065, 0x0000];

use core::ffi::c_void;
use crate::prelude::*;

// ---- file-scope preamble (translated) ----
pub const UNDERLINE_ON: u32 = 0;
pub const UNDERLINE_OFF: u32 = 1;
pub const UNDERLINE_HOVER: u32 = 2;
pub const UNDERLINE_IE: u32 = 3;

// general.cpp:26 `static const LPCWSTR s_pszExplorerKey = REGSTR_PATH_EXPLORER;`
// Held as the array rather than an `LPCWSTR`: a `static` of raw-pointer type is not `Sync`,
// so the pointer is taken at each use site instead.
static s_pszExplorerKey: &[WCHAR; 51] = REGSTR_PATH_EXPLORER;

/// general.cpp:104 `REGSTR_PATH_EXPLORER L"\\IconUnderline"` — C adjacent-string-literal
/// CONCATENATION, which is one string, not two expressions. The translation emitted the two
/// operands juxtaposed with no operator at all.
/// The array below is the CONCATENATED result, so state it in full — quoting only the second
/// operand leaves the array with no complete specification to be checked against:
/// L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\IconUnderline"
static s_pszIconUnderlineKey: [WCHAR; 65] = [
    0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063,
    0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
    0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065,
    0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072,
    0x0065, 0x0072, 0x005C, 0x0049, 0x0063, 0x006F, 0x006E, 0x0055, 0x006E, 0x0064, 0x0065, 0x0072,
    0x006C, 0x0069, 0x006E, 0x0065, 0x0000,
];

#[repr(C)]
pub struct GENERAL_DIALOG {
    pub hTaskIcon: HICON,
    pub hFolderIcon: HICON,
    pub hClickIcon: HICON,
}
pub type PGENERAL_DIALOG = *mut GENERAL_DIALOG;

#[no_mangle]
pub unsafe extern "C" fn SHELL32_GetDefaultShellState(pss: LPSHELLSTATE) {
    ZeroMemory(pss, core::mem::size_of::<SHELLSTATE>());
    (*pss).set_fShowAllObjects((TRUE) as i32);
    (*pss).set_fShowExtensions((TRUE) as i32);
    (*pss).set_fShowCompColor((TRUE) as i32);
    (*pss).set_fDoubleClickInWebView((TRUE) as i32);
    (*pss).set_fShowAttribCol((TRUE) as i32);
    (*pss).set_fShowInfoTip((TRUE) as i32);
    (*pss).set_fShowSuperHidden((FALSE) as i32);
    (*pss).lParamSort = (SHFSF_COL_NAME) as i32;
    (*pss).iSortDirection = 1;
    (*pss).version = REGSHELLSTATE_VERSION;
    (*pss).set_fSepProcess((FALSE) as i32);
    (*pss).set_fStartPanelOn((FALSE) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_WriteRegShellState(prss: PREGSHELLSTATE) -> LSTATUS {
    (*prss).dwSize = REGSHELLSTATE_SIZE;
    (*prss).ss.version = REGSHELLSTATE_VERSION;
    return (SHSetValueW(
        HKEY_CURRENT_USER,
        s_pszExplorerKey.as_ptr(),
        CPP_COM_WSTR_203E168CD5DB.as_ptr(),
        REG_BINARY,
        prss as *const core::ffi::c_void,
        REGSHELLSTATE_SIZE,
    )) as i32;
}
unsafe extern "C" fn IntSetShellStateSettings(bDoubleClick: i32, bUseCommonTasks: i32) -> i32 {
    let mut shellstate: SHELLSTATE = core::mem::zeroed();
    shellstate.set_fDoubleClickInWebView(if bDoubleClick != 0 { 1 } else { 0 });
    shellstate.set_fWebView(if bUseCommonTasks != 0 { 1 } else { 0 });
    SHGetSetSettings(core::ptr::addr_of_mut!(shellstate), SSF_DOUBLECLICKINWEBVIEW | SSF_WEBVIEW, (TRUE) as i32);
    (TRUE) as i32
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_ReadRegShellState(prss: PREGSHELLSTATE) -> BOOL {
    let mut dwSize: u32 = core::mem::size_of::<REGSHELLSTATE>() as u32;
    let err = SHGetValueW(
        HKEY_CURRENT_USER,
        s_pszExplorerKey.as_ptr(),
        CPP_COM_WSTR_203E168CD5DB.as_ptr(),
        core::ptr::null_mut(),
        (prss).cast(),
        core::ptr::addr_of_mut!(dwSize),
    );
    ((err == ERROR_SUCCESS && (*prss).dwSize >= REGSHELLSTATE_SIZE)) as BOOL
}
unsafe extern "C" fn IntSetUnderlineState(bUnderlineHover: i32) -> i32 {
    let dwValue: u32 = if bUnderlineHover != 0 { UNDERLINE_HOVER } else { UNDERLINE_IE };
    let status = SHRegSetUSValue(s_pszExplorerKey.as_ptr(), CPP_COM_WSTR_1CA266DE5730.as_ptr(), REG_NONE, core::ptr::addr_of!(dwValue) as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, SHREGSET_FORCE_HKCU | SHREGSET_HKLM);
    if status != (ERROR_SUCCESS) as i32 {
        return (FALSE) as i32;
    }
    SHSendMessageBroadcastW(WM_SETTINGCHANGE, 0, s_pszIconUnderlineKey.as_ptr() as LPARAM);
    (TRUE) as i32
}
unsafe extern "C" fn IntGetRawIconUnderlineValue() -> u32 {
    let mut dwValue: u32 = 0;
    let mut dwDefault: u32 = UNDERLINE_ON;
    let mut dwSize: u32 = core::mem::size_of::<u32>() as u32;
    SHRegGetUSValue(
        s_pszExplorerKey.as_ptr(),
        CPP_COM_WSTR_1CA266DE5730.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwValue) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(dwSize),
        (FALSE) as i32,
        core::ptr::addr_of_mut!(dwDefault) as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
    );
    dwValue
}
unsafe extern "C" fn SHELL_GetIconUnderlineMode() -> u32 {
    let mut mode = IntGetRawIconUnderlineValue();
    if mode < UNDERLINE_IE {
        return mode;
    }

    let mut buf: [u16; 7] = [0u16; 7];
    buf[0] = UNICODE_NULL;
    let mut cb: u32 = core::mem::size_of::<[u16; 7]>() as u32;
    SHRegGetUSValueW(
        CPP_COM_WSTR_079544FA595D.as_ptr(),
        CPP_COM_WSTR_09815BC5A072.as_ptr(),
        core::ptr::null_mut(),
        (buf.as_mut_ptr()).cast(),
        core::ptr::addr_of_mut!(cb),
        (FALSE) as i32,
        buf.as_mut_ptr().cast(),
        cb,
    );

    if lstrcmpiW(buf.as_ptr(), CPP_COM_WSTR_833AD7EF6992.as_ptr()) == 0i32 {
        return UNDERLINE_OFF;
    }
    if lstrcmpiW(buf.as_ptr(), CPP_COM_WSTR_9CF479D14C5E.as_ptr()) == 0i32 {
        return UNDERLINE_HOVER;
    }

    UNDERLINE_ON
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetIconUnderlineFlags() -> u32 {
    let mode = SHELL_GetIconUnderlineMode();
    if mode == UNDERLINE_HOVER {
        return LVS_EX_UNDERLINEHOT;
    }
    if mode == UNDERLINE_ON {
        return LVS_EX_UNDERLINEHOT | LVS_EX_UNDERLINECOLD;
    }
    return 0;
}
unsafe extern "C" fn IntSetNewWindowMode(bNewWindowMode: i32) -> i32 {
    let mut cs: CABINETSTATE = core::mem::zeroed();
    let size = core::mem::size_of::<CABINETSTATE>() as u32;
    if ReadCabinetState(core::ptr::addr_of_mut!(cs), (size) as i32) == 0 {
        return E_FAIL;
    }
    let new_bool = bNewWindowMode != 0;
    let old_bool = cs.fNewWindowMode() != 0;
    let changed = (old_bool != new_bool) as i32;
    cs.set_fNewWindowMode(new_bool as i32);
    if WriteCabinetState(core::ptr::addr_of_mut!(cs)) != 0 {
        if changed != 0 {
            S_OK
        } else {
            S_FALSE
        }
    } else {
        E_FAIL
    }
}
unsafe extern "C" fn IntGetNewWindowMode() -> u8 {
    let mut cs: CABINETSTATE = core::mem::zeroed();
    if ReadCabinetState(core::ptr::addr_of_mut!(cs), (core::mem::size_of::<CABINETSTATE>()) as i32) == 0 {
        return (FALSE) as u8;
    }
    (cs.fNewWindowMode() != 0) as u8
}
unsafe extern "C" fn GeneralDlg_UpdateIcons(hDlg: HWND, nCtrlID: UINT, pGeneral: PGENERAL_DIALOG) {
    let mut hTaskIcon: HICON = core::ptr::null_mut() as HICON;
    let mut hFolderIcon: HICON = core::ptr::null_mut() as HICON;
    let mut hClickIcon: HICON = core::ptr::null_mut() as HICON;
    let mut lpTaskIconName: LPTSTR = core::ptr::null_mut() as LPTSTR;
    let mut lpFolderIconName: LPTSTR = core::ptr::null_mut() as LPTSTR;
    let mut lpClickIconName: LPTSTR = core::ptr::null_mut() as LPTSTR;

    if IsDlgButtonChecked(hDlg, (IDC_FOLDER_OPTIONS_COMMONTASKS) as i32) == BST_CHECKED {
        lpTaskIconName = MAKEINTRESOURCE(IDI_SHELL_SHOW_COMMON_TASKS);
    } else if IsDlgButtonChecked(hDlg, (IDC_FOLDER_OPTIONS_CLASSICFOLDERS) as i32) == BST_CHECKED {
        lpTaskIconName = MAKEINTRESOURCE(IDI_SHELL_CLASSIC_FOLDERS);
    }

    if !lpTaskIconName.is_null() {
        hTaskIcon = LoadImage(shell32_hInstance, lpTaskIconName,
                                     IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR) as HICON;
        if !hTaskIcon.is_null() {
            let hwndTaskIcon = GetDlgItem(hDlg, (IDC_FOLDER_OPTIONS_TASKICON) as i32);
            SendMessage(hwndTaskIcon, STM_SETIMAGE, IMAGE_ICON as WPARAM, hTaskIcon as LPARAM);
        }
    }

    if IsDlgButtonChecked(hDlg, (IDC_FOLDER_OPTIONS_SAMEWINDOW) as i32) == BST_CHECKED {
        lpFolderIconName = MAKEINTRESOURCE(IDI_SHELL_OPEN_IN_SOME_WINDOW);
    } else if IsDlgButtonChecked(hDlg, (IDC_FOLDER_OPTIONS_OWNWINDOW) as i32) == BST_CHECKED {
        lpFolderIconName = MAKEINTRESOURCE(IDI_SHELL_OPEN_IN_NEW_WINDOW);
    }

    if !lpFolderIconName.is_null() {
        hFolderIcon = LoadImage(shell32_hInstance, lpFolderIconName,
                                       IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR) as HICON;
        if !hFolderIcon.is_null() {
            let hwndFolderIcon = GetDlgItem(hDlg, (IDC_FOLDER_OPTIONS_FOLDERICON) as i32);
            SendMessage(hwndFolderIcon, STM_SETIMAGE, IMAGE_ICON as WPARAM, hFolderIcon as LPARAM);
        }
    }

    if IsDlgButtonChecked(hDlg, (IDC_FOLDER_OPTIONS_SINGLECLICK) as i32) == BST_CHECKED {
        lpClickIconName = MAKEINTRESOURCE(IDI_SHELL_SINGLE_CLICK_TO_OPEN);
    } else if IsDlgButtonChecked(hDlg, (IDC_FOLDER_OPTIONS_DOUBLECLICK) as i32) == BST_CHECKED {
        lpClickIconName = MAKEINTRESOURCE(IDI_SHELL_DOUBLE_CLICK_TO_OPEN);
    }

    if !lpClickIconName.is_null() {
        hClickIcon = LoadImage(shell32_hInstance, lpClickIconName,
                                      IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR) as HICON;
        if !hClickIcon.is_null() {
            let hwndClickIcon = GetDlgItem(hDlg, (IDC_FOLDER_OPTIONS_CLICKICON) as i32);
            SendMessage(hwndClickIcon, STM_SETIMAGE, IMAGE_ICON as WPARAM, hClickIcon as LPARAM);
        }
    }

    if !hTaskIcon.is_null() {
        DestroyIcon((*pGeneral).hTaskIcon);
        (*pGeneral).hTaskIcon = hTaskIcon;
    }
    if !hFolderIcon.is_null() {
        DestroyIcon((*pGeneral).hFolderIcon);
        (*pGeneral).hFolderIcon = hFolderIcon;
    }
    if !hClickIcon.is_null() {
        DestroyIcon((*pGeneral).hClickIcon);
        (*pGeneral).hClickIcon = hClickIcon;
    }

    if nCtrlID == IDC_FOLDER_OPTIONS_SINGLECLICK {
        EnableWindow(GetDlgItem(hDlg, (IDC_FOLDER_OPTIONS_ULBROWSER) as i32), (TRUE) as i32);
        EnableWindow(GetDlgItem(hDlg, (IDC_FOLDER_OPTIONS_ULPOINT) as i32), (TRUE) as i32);
    }

    if nCtrlID == IDC_FOLDER_OPTIONS_DOUBLECLICK {
        EnableWindow(GetDlgItem(hDlg, (IDC_FOLDER_OPTIONS_ULBROWSER) as i32), (FALSE) as i32);
        EnableWindow(GetDlgItem(hDlg, (IDC_FOLDER_OPTIONS_ULPOINT) as i32), (FALSE) as i32);
    }
}
unsafe extern "C" fn GeneralDlg_StoreToUI(
    hwndDlg: HWND,
    mut bDoubleClick: BOOL,
    mut bUseCommonTasks: BOOL,
    bUnderlineHover: BOOL,
    bNewWindowMode: BOOL,
    _pGeneral: *mut GENERAL_DIALOG,
) {
    if SHRestricted(REST_CLASSICSHELL) != 0 {
        EnableWindow(
            GetDlgItem(hwndDlg, (IDC_FOLDER_OPTIONS_SINGLECLICK) as i32),
            (FALSE) as i32,
        );
        bDoubleClick = (TRUE) as i32;
        bUseCommonTasks = (FALSE) as i32;
    }

    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_FOLDER_OPTIONS_COMMONTASKS) as i32),
        bUseCommonTasks,
    );

    if bUseCommonTasks != 0 {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_COMMONTASKS) as i32, (IDC_FOLDER_OPTIONS_CLASSICFOLDERS) as i32, (IDC_FOLDER_OPTIONS_COMMONTASKS) as i32);
    } else {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_COMMONTASKS) as i32, (IDC_FOLDER_OPTIONS_CLASSICFOLDERS) as i32, (IDC_FOLDER_OPTIONS_CLASSICFOLDERS) as i32);
    }

    if bDoubleClick != 0 {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_SINGLECLICK) as i32, (IDC_FOLDER_OPTIONS_DOUBLECLICK) as i32, (IDC_FOLDER_OPTIONS_DOUBLECLICK) as i32);
    } else {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_SINGLECLICK) as i32, (IDC_FOLDER_OPTIONS_DOUBLECLICK) as i32, (IDC_FOLDER_OPTIONS_SINGLECLICK) as i32);
    }

    if bNewWindowMode != 0 {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_SAMEWINDOW) as i32, (IDC_FOLDER_OPTIONS_OWNWINDOW) as i32, (IDC_FOLDER_OPTIONS_OWNWINDOW) as i32);
    } else {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_SAMEWINDOW) as i32, (IDC_FOLDER_OPTIONS_OWNWINDOW) as i32, (IDC_FOLDER_OPTIONS_SAMEWINDOW) as i32);
    }

    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_FOLDER_OPTIONS_ULBROWSER) as i32),
        (bDoubleClick == 0) as BOOL,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, (IDC_FOLDER_OPTIONS_ULPOINT) as i32),
        (bDoubleClick == 0) as BOOL,
    );

    if bUnderlineHover != 0 {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_ULBROWSER) as i32, (IDC_FOLDER_OPTIONS_ULPOINT) as i32, (IDC_FOLDER_OPTIONS_ULPOINT) as i32);
    } else {
        CheckRadioButton(hwndDlg, (IDC_FOLDER_OPTIONS_ULBROWSER) as i32, (IDC_FOLDER_OPTIONS_ULPOINT) as i32, (IDC_FOLDER_OPTIONS_ULBROWSER) as i32);
    }
}
unsafe extern "C" fn GeneralDlg_OnInitDialog(hwndDlg: HWND, pGeneral: PGENERAL_DIALOG) -> BOOL {
    let mut ss: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings(core::ptr::addr_of_mut!(ss), SSF_DOUBLECLICKINWEBVIEW | SSF_WEBVIEW, (FALSE) as i32);
    let bDoubleClick = (ss.fDoubleClickInWebView() != 0) as u8;
    let bUseCommonTasks = (ss.fWebView() != 0) as u8;
    let bUnderlineHover = (IntGetRawIconUnderlineValue() == UNDERLINE_HOVER) as u8;
    let bNewWindowMode = IntGetNewWindowMode() as u8;
    GeneralDlg_StoreToUI(hwndDlg, (bDoubleClick) as i32, (bUseCommonTasks) as i32, (bUnderlineHover) as i32, (bNewWindowMode) as i32, pGeneral);
    GeneralDlg_UpdateIcons(hwndDlg, 0, pGeneral);
    (TRUE) as i32
}
unsafe extern "C" fn GeneralDlg_OnRestoreDefaults(hwndDlg: HWND, pGeneral: PGENERAL_DIALOG) {
    let bDoubleClick: u8 = (TRUE) as u8;
    let bUseCommonTasks: u8 = (TRUE) as u8;
    let bUnderlineHover: u8 = (FALSE) as u8;
    let bNewWindowMode: u8 = if _WIN32_WINNT < _WIN32_WINNT_WIN2K { 1u8 } else { 0u8 };
    GeneralDlg_StoreToUI(hwndDlg, (bDoubleClick) as i32, (bUseCommonTasks) as i32, (bUnderlineHover) as i32, (bNewWindowMode) as i32, pGeneral);
    GeneralDlg_UpdateIcons(hwndDlg, 0, pGeneral);
}
unsafe extern "C" fn GeneralDlg_OnApply(hwndDlg: HWND, pGeneral: PGENERAL_DIALOG) -> BOOL {
    let bDoubleClick: BOOL = (IsDlgButtonChecked(hwndDlg, (IDC_FOLDER_OPTIONS_SINGLECLICK) as i32) != BST_CHECKED) as i32;
    let bUseCommonTasks: BOOL = (IsDlgButtonChecked(hwndDlg, (IDC_FOLDER_OPTIONS_COMMONTASKS) as i32) == BST_CHECKED) as i32;
    let bUnderlineHover: BOOL = (IsDlgButtonChecked(hwndDlg, (IDC_FOLDER_OPTIONS_ULPOINT) as i32) == BST_CHECKED) as i32;
    let bNewWindowMode: BOOL = (IsDlgButtonChecked(hwndDlg, (IDC_FOLDER_OPTIONS_SAMEWINDOW) as i32) != BST_CHECKED) as i32;

    IntSetUnderlineState(bUnderlineHover);
    let updateCabinets: BOOL = (IntSetNewWindowMode(bNewWindowMode) == S_OK) as i32;
    IntSetShellStateSettings(bDoubleClick, bUseCommonTasks);
    if updateCabinets != 0 {
        PostCabinetMessage(CWM_STATECHANGE, 0, 0);
    }
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn FolderOptionsGeneralDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    static mut general: GENERAL_DIALOG = GENERAL_DIALOG {
        hTaskIcon: core::ptr::null_mut(),
        hFolderIcon: core::ptr::null_mut(),
        hClickIcon: core::ptr::null_mut(),
    };
    match uMsg {
        WM_INITDIALOG => {
            general.hTaskIcon = core::ptr::null_mut();
            general.hFolderIcon = core::ptr::null_mut();
            general.hClickIcon = core::ptr::null_mut();
            GeneralDlg_OnInitDialog(hwndDlg, core::ptr::addr_of_mut!(general)) as INT_PTR
        }
        WM_COMMAND => {
            match LOWORD((wParam) as u32) as u32 {
                IDC_FOLDER_OPTIONS_COMMONTASKS
                | IDC_FOLDER_OPTIONS_CLASSICFOLDERS
                | IDC_FOLDER_OPTIONS_SAMEWINDOW
                | IDC_FOLDER_OPTIONS_OWNWINDOW
                | IDC_FOLDER_OPTIONS_SINGLECLICK
                | IDC_FOLDER_OPTIONS_DOUBLECLICK
                | IDC_FOLDER_OPTIONS_ULBROWSER
                | IDC_FOLDER_OPTIONS_ULPOINT => {
                    if HIWORD((wParam) as u32) == (BN_CLICKED) as u16 {
                        GeneralDlg_UpdateIcons(
                            hwndDlg,
                            (LOWORD((wParam) as u32)) as u32,
                            core::ptr::addr_of_mut!(general),
                        );
                        PropSheet_Changed(
                            GetParent(hwndDlg),
                            hwndDlg,
                        );
                    }
                }
                IDC_FOLDER_OPTIONS_RESTORE => {
                    if HIWORD((wParam) as u32) == (BN_CLICKED) as u16 {
                        GeneralDlg_OnRestoreDefaults(
                            hwndDlg,
                            core::ptr::addr_of_mut!(general),
                        );
                        PropSheet_Changed(
                            GetParent(hwndDlg),
                            hwndDlg,
                        );
                    }
                }
                _ => {}
            }
            FALSE as INT_PTR
        }
        WM_NOTIFY => {
            let pnmh: PNMHDR = lParam as PNMHDR;
            match (*pnmh).code as i32 {
                PSN_SETACTIVE => {}
                PSN_APPLY => {
                    return GeneralDlg_OnApply(hwndDlg, core::ptr::addr_of_mut!(general)) as INT_PTR;
                }
                _ => {}
            }
            FALSE as INT_PTR
        }
        WM_DESTROY => {
            DestroyIcon(general.hTaskIcon);
            DestroyIcon(general.hFolderIcon);
            DestroyIcon(general.hClickIcon);
            FALSE as INT_PTR
        }
        _ => FALSE as INT_PTR,
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *     'General' tab property sheet of Folder Options
//  *
//  * Copyright 2007 Johannes Anderwald <johannes.anderwald@reactos.org>
//  * Copyright 2016-2018 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL (fprop);
//
// static const LPCWSTR s_pszExplorerKey = REGSTR_PATH_EXPLORER;
//
// enum { UNDERLINE_ON, UNDERLINE_OFF, UNDERLINE_HOVER, UNDERLINE_IE };
//
// /////////////////////////////////////////////////////////////////////////////
// // Shell settings
//
// extern "C" void
// SHELL32_GetDefaultShellState(LPSHELLSTATE pss)
// {
//     ZeroMemory(pss, sizeof(*pss));
//     pss->fShowAllObjects = TRUE;
//     pss->fShowExtensions = TRUE;
//     pss->fShowCompColor = TRUE;
//     pss->fDoubleClickInWebView = TRUE;
//     pss->fShowAttribCol = TRUE; // ROS defaults to Details view with this column on
//     pss->fShowInfoTip = TRUE;
//     pss->fShowSuperHidden = FALSE;
//     pss->lParamSort = SHFSF_COL_NAME;
//     pss->iSortDirection = 1;
//     pss->version = REGSHELLSTATE_VERSION;
//     pss->fSepProcess = FALSE;
//     pss->fStartPanelOn = FALSE; // Note: This should be changed to TRUE when the modern start menu is implemented
// }
//
// extern "C" LSTATUS
// SHELL32_WriteRegShellState(PREGSHELLSTATE prss)
// {
//     prss->dwSize = REGSHELLSTATE_SIZE;
//     prss->ss.version = REGSHELLSTATE_VERSION;
//     return SHSetValueW(HKEY_CURRENT_USER, s_pszExplorerKey, L"ShellState",
//                        REG_BINARY, prss, prss->dwSize);
// }
//
// // bDoubleClick is TRUE if "Double-click to open an item (single-click to select)".
// // bDoubleClick is FALSE if "Single-click to open an item (point to select)".
// //////////////////////////////////////////////////////////////////////////////
// // API Monitor:
// // SHLWAPI.dll	RegOpenKeyExW ( 0x000000c8, NULL, 0, MAXIMUM_ALLOWED, 0x00def234 )	ERROR_SUCCESS
// // SHLWAPI.dll	RegSetValueExW ( 0x000003e8, "ShellState", 0, REG_BINARY, 0x000c2050, 36 )	ERROR_SUCCESS
// // SHLWAPI.dll	RegCloseKey ( 0x000003e8 )	ERROR_SUCCESS
// // Explorer.EXE	SHSettingsChanged ( 0, "ShellState" ) (WM_SETTINGCHANGE handling)
// static BOOL
// IntSetShellStateSettings(BOOL bDoubleClick, BOOL bUseCommonTasks)
// {
//     SHELLSTATE shellstate;
//     shellstate.fDoubleClickInWebView = !!bDoubleClick;
//     shellstate.fWebView = !!bUseCommonTasks;
//     SHGetSetSettings(&shellstate, SSF_DOUBLECLICKINWEBVIEW | SSF_WEBVIEW, TRUE);
//     return TRUE;
// }
//
// extern "C" BOOL
// SHELL32_ReadRegShellState(PREGSHELLSTATE prss)
// {
//     DWORD dwSize = sizeof(REGSHELLSTATE);
//     LSTATUS err = SHGetValueW(HKEY_CURRENT_USER, s_pszExplorerKey,
//                               L"ShellState", NULL, prss, &dwSize);
//     return err == ERROR_SUCCESS && prss->dwSize >= REGSHELLSTATE_SIZE;
// }
//
// // bUnderlineHover is TRUE if "Underline icon titles only when I point at them".
// // bUnderlineHover is FALSE if "Underline icon titles consistent with my browser".
// //////////////////////////////////////////////////////////////////////////////
// // API Monitor:
// // SHELL32.dll	SHRegGetUSValueW ( "Software\Microsoft\Windows\CurrentVersion\Explorer", "IconUnderline", NULL, 0x00d2f324, 0x00d2f328, FALSE, 0x00d2f32c, 4 )	ERROR_SUCCESS
// // SHELL32.dll	IsDlgButtonChecked ( 0x0005009e, 30104 )	BST_CHECKED
// // SHELL32.dll	SHRegSetUSValueW ( "Software\Microsoft\Windows\CurrentVersion\Explorer", "IconUnderline", 0, 0x00d2f314, 4, 6 )	ERROR_SUCCESS
// // Explorer.EXE	SHSettingsChanged ( 0, "Software\Microsoft\Windows\CurrentVersion\Explorer\IconUnderline" ) (WM_SETTINGCHANGE handling)
// static BOOL IntSetUnderlineState(BOOL bUnderlineHover)
// {
//     LSTATUS Status;
//     DWORD dwValue = bUnderlineHover ? UNDERLINE_HOVER : UNDERLINE_IE;
//     Status = SHRegSetUSValue(s_pszExplorerKey, L"IconUnderline", REG_NONE,
//                              &dwValue, sizeof(dwValue), SHREGSET_FORCE_HKCU | SHREGSET_HKLM);
//     if (Status != ERROR_SUCCESS)
//         return FALSE;
//
//     SHSendMessageBroadcastW(WM_SETTINGCHANGE, 0, (LPARAM)(REGSTR_PATH_EXPLORER L"\\IconUnderline"));
//     return TRUE;
// }
//
// static UINT IntGetRawIconUnderlineValue()
// {
//     DWORD dwValue, dwDefault = UNDERLINE_ON;
//     DWORD dwSize = sizeof(dwValue);
//     SHRegGetUSValue(s_pszExplorerKey, L"IconUnderline", NULL, &dwValue, &dwSize, FALSE, &dwDefault, sizeof(dwDefault));
//     return dwValue;
// }
//
// static UINT SHELL_GetIconUnderlineMode()
// {
//     UINT mode = IntGetRawIconUnderlineValue();
//     if (mode < UNDERLINE_IE)
//         return mode;
//
//     WCHAR buf[sizeof("hoverX")];
//     *buf = UNICODE_NULL;
//     DWORD cb = sizeof(buf);
//     SHRegGetUSValueW(L"Software\\Microsoft\\Internet Explorer\\Main",
//                      L"Anchor Underline", NULL, buf, &cb, FALSE, buf, cb);
//     if (!lstrcmpiW(buf, L"no"))
//         return UNDERLINE_OFF;
//     if (!lstrcmpiW(buf, L"hover"))
//         return UNDERLINE_HOVER;
//     return UNDERLINE_ON;
// }
//
// UINT SHELL_GetIconUnderlineFlags()
// {
//     UINT mode = SHELL_GetIconUnderlineMode();
//     if (mode == UNDERLINE_HOVER)
//         return LVS_EX_UNDERLINEHOT;
//     if (mode == UNDERLINE_ON)
//         return LVS_EX_UNDERLINEHOT | LVS_EX_UNDERLINECOLD;
//     return 0;
// }
//
// // bNewWindowMode is TRUE if "Open each folder in its own window".
// // bNewWindowMode is FALSE if "Open each folder in the same window".
// //////////////////////////////////////////////////////////////////////////////
// // API Monitor:
// // SHELL32.dll	RegCreateKeyExW ( HKEY_CURRENT_USER, "Software\Microsoft\Windows\CurrentVersion\Explorer\CabinetState", 0, NULL, 0, KEY_SET_VALUE, NULL, 0x00d2f2b8, NULL )	ERROR_SUCCESS		0.0000455
// // SHELL32.dll	RegSetValueExW ( 0x00000854, "Settings", 0, REG_BINARY, 0x0210f170, 12 )	ERROR_SUCCESS		0.0001472
// // SHELL32.dll	RegSetValueExW ( 0x00000854, "FullPath", 0, REG_DWORD, 0x00d2f2ac, 4 )	ERROR_SUCCESS		0.0000168
// // SHELL32.dll	RegCloseKey ( 0x00000854 )	ERROR_SUCCESS		0.0000000
// static HRESULT IntSetNewWindowMode(BOOL bNewWindowMode)
// {
//     CABINETSTATE cs;
//     if (!ReadCabinetState(&cs, sizeof(cs)))
//         return E_FAIL;
//
//     BOOL changed = !!cs.fNewWindowMode != !!bNewWindowMode;
//     cs.fNewWindowMode = !!bNewWindowMode;
//     return WriteCabinetState(core::ptr::addr_of_mut!(cs)) ? (changed ? S_OK : S_FALSE) : E_FAIL;
// }
//
// static BOOL IntGetNewWindowMode(VOID)
// {
//     CABINETSTATE cs;
//     if (!ReadCabinetState(&cs, sizeof(cs)))
//         return FALSE;
//
//     return !!cs.fNewWindowMode;
// }
//
// /////////////////////////////////////////////////////////////////////////////
// // GeneralDlg
//
// typedef struct GENERAL_DIALOG
// {
//     HICON hTaskIcon;
//     HICON hFolderIcon;
//     HICON hClickIcon;
// } GENERAL_DIALOG, *PGENERAL_DIALOG;
//
// static VOID
// GeneralDlg_UpdateIcons(HWND hDlg, UINT nCtrlID, PGENERAL_DIALOG pGeneral)
// {
//     HICON hTaskIcon = NULL, hFolderIcon = NULL, hClickIcon = NULL;
//     LPTSTR lpTaskIconName = NULL, lpFolderIconName = NULL, lpClickIconName = NULL;
//
//     // Show task setting icon.
//     if (IsDlgButtonChecked(hDlg, IDC_FOLDER_OPTIONS_COMMONTASKS) == BST_CHECKED)
//         lpTaskIconName = MAKEINTRESOURCE(IDI_SHELL_SHOW_COMMON_TASKS);
//     else if (IsDlgButtonChecked(hDlg, IDC_FOLDER_OPTIONS_CLASSICFOLDERS) == BST_CHECKED)
//         lpTaskIconName = MAKEINTRESOURCE(IDI_SHELL_CLASSIC_FOLDERS);
//
//     if (lpTaskIconName)
//     {
//         hTaskIcon = (HICON)LoadImage(shell32_hInstance, lpTaskIconName,
//                                      IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR);
//         if (hTaskIcon)
//         {
//             HWND hwndTaskIcon = GetDlgItem(hDlg, IDC_FOLDER_OPTIONS_TASKICON);
//             SendMessage(hwndTaskIcon, STM_SETIMAGE, IMAGE_ICON, (LPARAM)hTaskIcon);
//         }
//     }
//
//     // Show Folder setting icons
//     if (IsDlgButtonChecked(hDlg, IDC_FOLDER_OPTIONS_SAMEWINDOW) == BST_CHECKED)
//         lpFolderIconName = MAKEINTRESOURCE(IDI_SHELL_OPEN_IN_SOME_WINDOW);
//     else if (IsDlgButtonChecked(hDlg, IDC_FOLDER_OPTIONS_OWNWINDOW) == BST_CHECKED)
//         lpFolderIconName = MAKEINTRESOURCE(IDI_SHELL_OPEN_IN_NEW_WINDOW);
//
//     if (lpFolderIconName)
//     {
//         hFolderIcon = (HICON)LoadImage(shell32_hInstance, lpFolderIconName,
//                                        IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR);
//         if (hFolderIcon)
//         {
//             HWND hwndFolderIcon = GetDlgItem(hDlg, IDC_FOLDER_OPTIONS_FOLDERICON);
//             SendMessage(hwndFolderIcon, STM_SETIMAGE, IMAGE_ICON, (LPARAM)hFolderIcon);
//         }
//     }
//
//     // Show click setting icon
//     if (IsDlgButtonChecked(hDlg, IDC_FOLDER_OPTIONS_SINGLECLICK) == BST_CHECKED)
//         lpClickIconName = MAKEINTRESOURCE(IDI_SHELL_SINGLE_CLICK_TO_OPEN);
//     else if (IsDlgButtonChecked(hDlg, IDC_FOLDER_OPTIONS_DOUBLECLICK) == BST_CHECKED)
//         lpClickIconName = MAKEINTRESOURCE(IDI_SHELL_DOUBLE_CLICK_TO_OPEN);
//
//     if (lpClickIconName)
//     {
//         hClickIcon = (HICON)LoadImage(shell32_hInstance, lpClickIconName,
//                                       IMAGE_ICON, 0, 0, LR_DEFAULTCOLOR);
//         if (hClickIcon)
//         {
//             HWND hwndClickIcon = GetDlgItem(hDlg, IDC_FOLDER_OPTIONS_CLICKICON);
//             SendMessage(hwndClickIcon, STM_SETIMAGE, IMAGE_ICON, (LPARAM)hClickIcon);
//         }
//     }
//
//     // Replace icons
//     if (hTaskIcon)
//     {
//         DestroyIcon(pGeneral->hTaskIcon);
//         pGeneral->hTaskIcon = hTaskIcon;
//     }
//     if (hFolderIcon)
//     {
//         DestroyIcon(pGeneral->hFolderIcon);
//         pGeneral->hFolderIcon = hFolderIcon;
//     }
//     if (hClickIcon)
//     {
//         DestroyIcon(pGeneral->hClickIcon);
//         pGeneral->hClickIcon = hClickIcon;
//     }
//
//     if (nCtrlID == IDC_FOLDER_OPTIONS_SINGLECLICK)
//     {
//         EnableWindow(GetDlgItem(hDlg, IDC_FOLDER_OPTIONS_ULBROWSER), TRUE);
//         EnableWindow(GetDlgItem(hDlg, IDC_FOLDER_OPTIONS_ULPOINT), TRUE);
//     }
//
//     if (nCtrlID == IDC_FOLDER_OPTIONS_DOUBLECLICK)
//     {
//         EnableWindow(GetDlgItem(hDlg, IDC_FOLDER_OPTIONS_ULBROWSER), FALSE);
//         EnableWindow(GetDlgItem(hDlg, IDC_FOLDER_OPTIONS_ULPOINT), FALSE);
//     }
// }
//
// static void
// GeneralDlg_StoreToUI(HWND hwndDlg, BOOL bDoubleClick, BOOL bUseCommonTasks,
//                      BOOL bUnderlineHover, BOOL bNewWindowMode, PGENERAL_DIALOG pGeneral)
// {
//     if (SHRestricted(REST_CLASSICSHELL))
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_FOLDER_OPTIONS_SINGLECLICK), FALSE);
//         bDoubleClick = TRUE;
//         bUseCommonTasks = FALSE;
//     }
//
//     EnableWindow(GetDlgItem(hwndDlg, IDC_FOLDER_OPTIONS_COMMONTASKS), bUseCommonTasks); // TODO: ROS DefView does not support WebView nor the tasks pane
//
//     if (bUseCommonTasks)
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_COMMONTASKS, IDC_FOLDER_OPTIONS_CLASSICFOLDERS, IDC_FOLDER_OPTIONS_COMMONTASKS);
//     else
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_COMMONTASKS, IDC_FOLDER_OPTIONS_CLASSICFOLDERS, IDC_FOLDER_OPTIONS_CLASSICFOLDERS);
//
//     if (bDoubleClick)
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_SINGLECLICK, IDC_FOLDER_OPTIONS_DOUBLECLICK, IDC_FOLDER_OPTIONS_DOUBLECLICK);
//     else
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_SINGLECLICK, IDC_FOLDER_OPTIONS_DOUBLECLICK, IDC_FOLDER_OPTIONS_SINGLECLICK);
//
//     if (bNewWindowMode)
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_SAMEWINDOW, IDC_FOLDER_OPTIONS_OWNWINDOW, IDC_FOLDER_OPTIONS_OWNWINDOW);
//     else
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_SAMEWINDOW, IDC_FOLDER_OPTIONS_OWNWINDOW, IDC_FOLDER_OPTIONS_SAMEWINDOW);
//
//     EnableWindow(GetDlgItem(hwndDlg, IDC_FOLDER_OPTIONS_ULBROWSER), !bDoubleClick);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_FOLDER_OPTIONS_ULPOINT), !bDoubleClick);
//     if (bUnderlineHover)
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_ULBROWSER, IDC_FOLDER_OPTIONS_ULPOINT, IDC_FOLDER_OPTIONS_ULPOINT);
//     else
//         CheckRadioButton(hwndDlg, IDC_FOLDER_OPTIONS_ULBROWSER, IDC_FOLDER_OPTIONS_ULPOINT, IDC_FOLDER_OPTIONS_ULBROWSER);
// }
//
// static BOOL
// GeneralDlg_OnInitDialog(HWND hwndDlg, PGENERAL_DIALOG pGeneral)
// {
//     SHELLSTATE ss;
//     SHGetSetSettings(&ss, SSF_DOUBLECLICKINWEBVIEW | SSF_WEBVIEW, FALSE);
//     BOOL bDoubleClick = !!ss.fDoubleClickInWebView;
//     BOOL bUseCommonTasks = !!ss.fWebView;
//     BOOL bUnderlineHover = IntGetRawIconUnderlineValue() == UNDERLINE_HOVER;
//     BOOL bNewWindowMode = IntGetNewWindowMode();
//
//     GeneralDlg_StoreToUI(hwndDlg, bDoubleClick, bUseCommonTasks, bUnderlineHover, bNewWindowMode, pGeneral);
//     GeneralDlg_UpdateIcons(hwndDlg, 0, pGeneral);
//
//     return TRUE;
// }
//
// static void
// GeneralDlg_OnRestoreDefaults(HWND hwndDlg, PGENERAL_DIALOG pGeneral)
// {
//     // default values
//     BOOL bDoubleClick = TRUE;
//     BOOL bUseCommonTasks = TRUE;
//     BOOL bUnderlineHover = FALSE;
//     BOOL bNewWindowMode = (_WIN32_WINNT < _WIN32_WINNT_WIN2K);
//
//     GeneralDlg_StoreToUI(hwndDlg, bDoubleClick, bUseCommonTasks, bUnderlineHover, bNewWindowMode, pGeneral);
//     GeneralDlg_UpdateIcons(hwndDlg, 0, pGeneral);
// }
//
// static BOOL
// GeneralDlg_OnApply(HWND hwndDlg, PGENERAL_DIALOG pGeneral)
// {
//     BOOL bDoubleClick = !(IsDlgButtonChecked(hwndDlg, IDC_FOLDER_OPTIONS_SINGLECLICK) == BST_CHECKED);
//     BOOL bUseCommonTasks = (IsDlgButtonChecked(hwndDlg, IDC_FOLDER_OPTIONS_COMMONTASKS) == BST_CHECKED);
//     BOOL bUnderlineHover = (IsDlgButtonChecked(hwndDlg, IDC_FOLDER_OPTIONS_ULPOINT) == BST_CHECKED);
//     BOOL bNewWindowMode = !(IsDlgButtonChecked(hwndDlg, IDC_FOLDER_OPTIONS_SAMEWINDOW) == BST_CHECKED);
//
//     IntSetUnderlineState(bUnderlineHover);
//     BOOL updateCabinets = IntSetNewWindowMode(bNewWindowMode) == S_OK;
//     IntSetShellStateSettings(bDoubleClick, bUseCommonTasks);
//     if (updateCabinets)
//         PostCabinetMessage(CWM_STATECHANGE, 0, 0);
//     return TRUE;
// }
//
// INT_PTR __stdcall
// FolderOptionsGeneralDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     static GENERAL_DIALOG general;
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             general.hTaskIcon = NULL;
//             general.hFolderIcon = NULL;
//             general.hClickIcon = NULL;
//             return GeneralDlg_OnInitDialog(hwndDlg, &general);
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_FOLDER_OPTIONS_COMMONTASKS:
//                 case IDC_FOLDER_OPTIONS_CLASSICFOLDERS:
//                 case IDC_FOLDER_OPTIONS_SAMEWINDOW:
//                 case IDC_FOLDER_OPTIONS_OWNWINDOW:
//                 case IDC_FOLDER_OPTIONS_SINGLECLICK:
//                 case IDC_FOLDER_OPTIONS_DOUBLECLICK:
//                 case IDC_FOLDER_OPTIONS_ULBROWSER:
//                 case IDC_FOLDER_OPTIONS_ULPOINT:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         GeneralDlg_UpdateIcons(hwndDlg, LOWORD(wParam), &general);
//
//                         // Enable the 'Apply' button
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//                 case IDC_FOLDER_OPTIONS_RESTORE:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         GeneralDlg_OnRestoreDefaults(hwndDlg, &general);
//
//                         // Enable the 'Apply' button
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//             }
//             break;
//
//         case WM_NOTIFY:
//         {
//             LPNMHDR pnmh = (LPNMHDR)lParam;
//
//             switch (pnmh->code)
//             {
//                 case PSN_SETACTIVE:
//                     break;
//
//                 case PSN_APPLY:
//                     return GeneralDlg_OnApply(hwndDlg, &general);
//             }
//             break;
//         }
//
//         case WM_DESTROY:
//             DestroyIcon(general.hTaskIcon);
//             DestroyIcon(general.hFolderIcon);
//             DestroyIcon(general.hClickIcon);
//             break;
//
//         default:
//              return FALSE;
//     }
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_593117FB0799: [u16; 14] = [
    0x004Cu16, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0048, 0x006F, 0x0074,
    0x006B, 0x0065, 0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_7C846DD5631B: [u16; 16] = [
    0x004Bu16, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0020, 0x004C,
    0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_82A16CE4AE7E: [u16; 23] = [
    0x004Bu16, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0020, 0x004C,
    0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x005C, 0x0054, 0x006F, 0x0067, 0x0067,
    0x006C, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A38E93E48CB3: [u16; 11] = [
    0x0041u16, 0x0074, 0x0074, 0x0072, 0x0069, 0x0062, 0x0075, 0x0074, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_D59C317D44DA: [u16; 7] = [0x0048u16, 0x006F, 0x0074, 0x006B, 0x0065, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_F315F9787974: [u16; 16] = [
    0x004Cu16, 0x0061, 0x006E, 0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0020, 0x0048,
    0x006F, 0x0074, 0x006B, 0x0065, 0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut _KeySettings: KEY_SETTINGS = KEY_SETTINGS {
    dwAttributes: 0 as DWORD,
    dwLanguage: 0,
    dwLayout: 0,
};
#[no_mangle]
pub unsafe extern "C" fn ReadAttributes() -> DWORD {
    let mut dwAttributes: DWORD = 0 as DWORD;
    let mut hKey: HKEY = core::ptr::null_mut::<HKEY__>();

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_7C846DD5631B.as_ptr(),
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        &raw mut hKey,
    ) == ERROR_SUCCESS as LSTATUS
    {
        let mut dwSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;

        RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_A38E93E48CB3.as_ptr(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            &raw mut dwAttributes as LPBYTE,
            &raw mut dwSize,
        );

        RegCloseKey(hKey);
    }

    return dwAttributes;
}

unsafe extern "C" fn ReadKeysSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();

    _KeySettings.dwAttributes = ReadAttributes();

    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_82A16CE4AE7E.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    )) as u32 == ERROR_SUCCESS
    {
        let mut szBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut dwSize: DWORD;

        dwSize = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;

        if (RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_F315F9787974.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        )) as u32 == ERROR_SUCCESS
        {
            _KeySettings.dwLanguage = _wtoi(addr_of_mut!(szBuffer) as *mut WCHAR) as DWORD;
        }

        dwSize = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;

        if (RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_593117FB0799.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        )) as u32 == ERROR_SUCCESS
        {
            _KeySettings.dwLayout = _wtoi(addr_of_mut!(szBuffer) as *mut WCHAR) as DWORD;
        }

        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn WriteKeysSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();

    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_7C846DD5631B.as_ptr(),
        0,
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(hKey),
    )) as u32 == ERROR_SUCCESS
    {
        RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_A38E93E48CB3.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(_KeySettings.dwAttributes) as LPBYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );

        RegCloseKey(hKey);
    }

    if (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_82A16CE4AE7E.as_ptr(),
        0,
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(hKey),
    )) as u32 == ERROR_SUCCESS
    {
        let mut szBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        StringCchPrintfW!(
            szBuffer.as_mut_ptr(),
            ARRAYSIZE!(szBuffer),
            RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
            _KeySettings.dwLanguage,
        );

        RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_D59C317D44DA.as_ptr(),
            0,
            REG_SZ,
            szBuffer.as_mut_ptr() as LPBYTE,
            ((wcslen(szBuffer.as_mut_ptr()) + 1)
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );

        RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_F315F9787974.as_ptr(),
            0,
            REG_SZ,
            szBuffer.as_mut_ptr() as LPBYTE,
            ((wcslen(szBuffer.as_mut_ptr()) + 1)
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );

        StringCchPrintfW!(
            szBuffer.as_mut_ptr(),
            ARRAYSIZE!(szBuffer),
            RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
            _KeySettings.dwLayout,
        );

        RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_593117FB0799.as_ptr(),
            0,
            REG_SZ,
            szBuffer.as_mut_ptr() as LPBYTE,
            ((wcslen(szBuffer.as_mut_ptr()) + 1)
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );

        RegCloseKey(hKey);
    }

    /* Notice system of change hotkeys parameters */
    SystemParametersInfoW(
        SPI_SETLANGTOGGLE,
        0,
        core::ptr::null_mut(),
        0,
    );

    /* Notice system of change CapsLock mode parameters */
    ActivateKeyboardLayout(
        GetKeyboardLayout(0),
        KLF_RESET | _KeySettings.dwAttributes,
    );
}

unsafe extern "C" fn UpdateKeySettingsListView(hwndList: HWND) {
    let mut szBuffer: [WCHAR; (MAX_STR_LEN) as _] = [0; (MAX_STR_LEN) as _];
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut iItemIndex: INT;

    ListView_DeleteAllItems!(hwndList);

    ZeroMemory!(
        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
        core::mem::size_of::<LV_ITEM>(),
    );

    LoadStringW(
        hApplet,
        IDS_SWITCH_BET_INLANG,
        szBuffer.as_mut_ptr(),
        ARRAYSIZE!(szBuffer) as core::ffi::c_int,
    );
    item.mask = (LVIF_TEXT | LVIF_PARAM | LVIF_STATE) as UINT;
    item.pszText = szBuffer.as_mut_ptr();
    item.iItem = 0 as core::ffi::c_int;

    iItemIndex = ListView_InsertItem!(hwndList, core::ptr::addr_of_mut!(item));

    if _KeySettings.dwLanguage == 1 {
        LoadStringW(
            hApplet,
            IDS_LEFT_ALT_SHIFT,
            szBuffer.as_mut_ptr(),
            ARRAYSIZE!(szBuffer) as core::ffi::c_int,
        );
    } else if _KeySettings.dwLanguage == 2 {
        LoadStringW(
            hApplet,
            IDS_CTRL_SHIFT,
            szBuffer.as_mut_ptr(),
            ARRAYSIZE!(szBuffer) as core::ffi::c_int,
        );
    } else {
        LoadStringW(
            hApplet,
            IDS_NONE,
            szBuffer.as_mut_ptr(),
            ARRAYSIZE!(szBuffer) as core::ffi::c_int,
        );
    }

    ListView_SetItemText!(
        hwndList,
        iItemIndex,
        1,
        szBuffer.as_mut_ptr(),
    );
}

unsafe extern "C" fn OnInitKeySettingsDialog(hwndDlg: HWND) {
    let mut column: LVCOLUMNW = core::mem::zeroed();
    let mut hwndList: HWND = core::ptr::null_mut();

    ReadKeysSettings();

    if _KeySettings.dwAttributes & KLF_SHIFTLOCK != 0 {
        CheckDlgButton(hwndDlg, (IDC_PRESS_SHIFT_KEY_RB as i32), BST_CHECKED);
        CheckDlgButton(hwndDlg, (IDC_PRESS_CL_KEY_RB as i32), BST_UNCHECKED);
    } else {
        CheckDlgButton(hwndDlg, (IDC_PRESS_SHIFT_KEY_RB as i32), BST_UNCHECKED);
        CheckDlgButton(hwndDlg, (IDC_PRESS_CL_KEY_RB as i32), BST_CHECKED);
    }

    hwndList = GetDlgItem(hwndDlg, (IDC_KEY_LISTVIEW as i32));

    ListView_SetExtendedListViewStyle!(hwndList, LVS_EX_FULLROWSELECT);

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(column) as *mut u8,
        0,
        core::mem::size_of::<LVCOLUMNW>(),
    );

    column.mask = LVCF_FMT | LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM;

    column.fmt = (LVCFMT_LEFT as i32);
    column.iSubItem = 0;
    column.pszText = RUSTCONV_WSTR_96A296D224F2.as_ptr() as *mut u16;
    column.cx = 210;
    ListView_InsertColumn!(hwndList, 0, core::ptr::addr_of!(column));

    column.fmt = (LVCFMT_RIGHT as i32);
    column.cx = 145;
    column.iSubItem = 1;
    column.pszText = RUSTCONV_WSTR_96A296D224F2.as_ptr() as *mut u16;
    ListView_InsertColumn!(hwndList, 1, core::ptr::addr_of!(column));

    UpdateKeySettingsListView(hwndList);
}
#[no_mangle]
pub unsafe extern "system" fn KeySettingsDialogProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            OnInitKeySettingsDialog(hwndDlg);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_CHANGE_KEY_SEQ_BTN => {
                    if DialogBoxParamW(
                        hApplet,
                        MAKEINTRESOURCEW!(IDD_CHANGE_KEY_SEQ),
                        hwndDlg,
                        Some(ChangeKeySeqDialogProc),
                        core::ptr::addr_of_mut!(_KeySettings) as LPARAM,
                    ) == IDOK as INT_PTR
                    {
                        UpdateKeySettingsListView(GetDlgItem(hwndDlg, (IDC_KEY_LISTVIEW as i32)));
                    }
                }
                IDOK => {
                    if IsDlgButtonChecked(hwndDlg, (IDC_PRESS_CL_KEY_RB as i32)) == BST_CHECKED as UINT {
                        _KeySettings.dwAttributes &= !(KLF_SHIFTLOCK as DWORD);
                    } else {
                        _KeySettings.dwAttributes |= KLF_SHIFTLOCK as DWORD;
                    }

                    WriteKeysSettings();
                    EndDialog(hwndDlg, LOWORD!(wParam) as INT_PTR);
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD!(wParam) as INT_PTR);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

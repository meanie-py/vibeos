// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0B087FAD2DEC: [u16; 29] = [
    0x0044u16, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0020, 0x0054, 0x0068,
    0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0049, 0x006E, 0x0070, 0x0075, 0x0074,
    0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_EAEBEC3E6F8D: [u16; 23] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x0054, 0x0046, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_bTextServiceIsOff: BOOL = (FALSE) as i32;
#[no_mangle]
pub unsafe extern "C" fn LoadAdvancedSettings(hwndDlg: HWND) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LRESULT = 0;
    let mut dwType: DWORD = 0;
    let mut dwValue: DWORD = 0;
    let mut cbValue: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    error = (RegOpenKeyExW(HKEY_CURRENT_USER, RUSTCONV_WSTR_EAEBEC3E6F8D.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as _;
    if (error) as u32 != (ERROR_SUCCESS) as _ {
        return (FALSE) as BOOL;
    }

    error = (RegQueryValueExW(hKey,
                             RUSTCONV_WSTR_0B087FAD2DEC.as_ptr(),
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!(dwType),
                             core::ptr::addr_of_mut!(dwValue) as LPBYTE,
                             core::ptr::addr_of_mut!(cbValue))) as _;
    if ((error) as u32 != (ERROR_SUCCESS) as _) || (dwType != REG_DWORD) || (cbValue != core::mem::size_of::<DWORD>() as DWORD) {
        dwValue = FALSE; /* Default */
    }

    RegCloseKey(hKey);

    CheckDlgButton(hwndDlg, (IDC_TURNOFFTEXTSVCS_CB as i32), (if dwValue != 0 { BST_CHECKED } else { BST_UNCHECKED }) as UINT);
    g_bTextServiceIsOff = (dwValue != 0) as BOOL;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SaveAdvancedSettings(hwndDlg: HWND) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LRESULT = 0;
    let dwValue: DWORD = g_bTextServiceIsOff as DWORD;
    let cbValue: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    error = RegOpenKeyExW(HKEY_CURRENT_USER, RUSTCONV_WSTR_EAEBEC3E6F8D.as_ptr(), 0, KEY_WRITE, core::ptr::addr_of_mut!(hKey)) as LRESULT;
    if error != ERROR_SUCCESS as LRESULT {
        return (FALSE) as BOOL;
    }

    error = RegSetValueExW(hKey, RUSTCONV_WSTR_0B087FAD2DEC.as_ptr(), 0, REG_DWORD,
                           core::ptr::addr_of!(dwValue) as *const BYTE, cbValue) as LRESULT;

    RegCloseKey(hKey);
    return (error == ERROR_SUCCESS as LRESULT) as BOOL;
}

unsafe extern "C" fn OnNotifyAdvancedSettingsPage(hwndDlg: HWND, lParam: LPARAM) -> INT_PTR {
    let header: LPNMHDR = lParam as LPNMHDR;

    match ((*header).code) as i32 {
        PSN_APPLY => {
            let bOff: BOOL = (IsDlgButtonChecked(hwndDlg, (IDC_TURNOFFTEXTSVCS_CB as i32)) == BST_CHECKED) as BOOL;
            g_bRebootNeeded |= (g_bTextServiceIsOff != 0 && bOff == 0) as BOOL;
            g_bTextServiceIsOff = bOff;

            /* Write advanced settings */
            SaveAdvancedSettings(hwndDlg);
        }
        _ => {}
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn AdvancedSettingsPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    if uMsg == WM_INITDIALOG {
        LoadAdvancedSettings(hwndDlg);
        return (TRUE) as INT_PTR;
    } else if uMsg == WM_NOTIFY {
        return OnNotifyAdvancedSettingsPage(hwndDlg, lParam);
    } else if uMsg == WM_COMMAND {
        if LOWORD!(wParam) == (IDC_TURNOFFTEXTSVCS_CB as u16) {
            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
        }
    }

    return 0;
}

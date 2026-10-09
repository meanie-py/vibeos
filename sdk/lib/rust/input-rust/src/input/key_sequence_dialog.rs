use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn ChangeKeySeqDialogProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            let mut keySettings: *mut KEY_SETTINGS = lParam as *mut KEY_SETTINGS;

            if !keySettings.is_null() {
                SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, keySettings as LONG_PTR);

                if (*keySettings).dwLanguage == 3 {
                    CheckDlgButton(hwndDlg, (IDC_CTRL_LANG as i32), BST_CHECKED);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_CTRL_LANG as i32)), (FALSE) as BOOL);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_LEFT_ALT_LANG as i32)), (FALSE) as BOOL);
                } else {
                    CheckDlgButton(hwndDlg, (IDC_SWITCH_INPUT_LANG_CB as i32), BST_CHECKED);

                    if (*keySettings).dwLanguage == 1 {
                        CheckDlgButton(hwndDlg, (IDC_LEFT_ALT_LANG as i32), BST_CHECKED);
                    } else {
                        CheckDlgButton(hwndDlg, (IDC_CTRL_LANG as i32), BST_CHECKED);
                    }
                }

                if (*keySettings).dwLayout == 3 {
                    CheckDlgButton(hwndDlg, (IDC_LEFT_ALT_LAYOUT as i32), BST_CHECKED);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_CTRL_LAYOUT as i32)), (FALSE) as BOOL);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_LEFT_ALT_LAYOUT as i32)), (FALSE) as BOOL);
                } else {
                    CheckDlgButton(hwndDlg, (IDC_SWITCH_KBLAYOUTS_CB as i32), BST_CHECKED);

                    if (*keySettings).dwLayout == 1 {
                        CheckDlgButton(hwndDlg, (IDC_LEFT_ALT_LAYOUT as i32), BST_CHECKED);
                    } else {
                        CheckDlgButton(hwndDlg, (IDC_CTRL_LAYOUT as i32), BST_CHECKED);
                    }
                }
            }
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_SWITCH_INPUT_LANG_CB => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        if IsDlgButtonChecked(hwndDlg, (IDC_SWITCH_INPUT_LANG_CB as i32)) == BST_CHECKED {
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_CTRL_LANG as i32)), (TRUE) as BOOL);
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_LEFT_ALT_LANG as i32)), (TRUE) as BOOL);
                        } else {
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_CTRL_LANG as i32)), (FALSE) as BOOL);
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_LEFT_ALT_LANG as i32)), (FALSE) as BOOL);
                        }
                    }
                }

                IDC_SWITCH_KBLAYOUTS_CB => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        if IsDlgButtonChecked(hwndDlg, (IDC_SWITCH_KBLAYOUTS_CB as i32)) == BST_CHECKED {
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_CTRL_LAYOUT as i32)), (TRUE) as BOOL);
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_LEFT_ALT_LAYOUT as i32)), (TRUE) as BOOL);
                        } else {
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_CTRL_LAYOUT as i32)), (FALSE) as BOOL);
                            EnableWindow(GetDlgItem(hwndDlg, (IDC_LEFT_ALT_LAYOUT as i32)), (FALSE) as BOOL);
                        }
                    }
                }

                IDC_CTRL_LANG => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        CheckDlgButton(hwndDlg, (IDC_LEFT_ALT_LAYOUT as i32), BST_CHECKED);
                        CheckDlgButton(hwndDlg, (IDC_CTRL_LAYOUT as i32), BST_UNCHECKED);
                    }
                }

                IDC_LEFT_ALT_LANG => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        CheckDlgButton(hwndDlg, (IDC_CTRL_LAYOUT as i32), BST_CHECKED);
                        CheckDlgButton(hwndDlg, (IDC_LEFT_ALT_LAYOUT as i32), BST_UNCHECKED);
                    }
                }

                IDC_CTRL_LAYOUT => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        CheckDlgButton(hwndDlg, (IDC_LEFT_ALT_LANG as i32), BST_CHECKED);
                        CheckDlgButton(hwndDlg, (IDC_CTRL_LANG as i32), BST_UNCHECKED);
                    }
                }

                IDC_LEFT_ALT_LAYOUT => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        CheckDlgButton(hwndDlg, (IDC_CTRL_LANG as i32), BST_CHECKED);
                        CheckDlgButton(hwndDlg, (IDC_LEFT_ALT_LANG as i32), BST_UNCHECKED);
                    }
                }

                IDOK => {
                    let mut keySettings: *mut KEY_SETTINGS;

                    keySettings = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as *mut KEY_SETTINGS;

                    if !keySettings.is_null() {
                        if IsDlgButtonChecked(hwndDlg, (IDC_SWITCH_INPUT_LANG_CB as i32)) == BST_CHECKED {
                            if IsDlgButtonChecked(hwndDlg, (IDC_CTRL_LANG as i32)) == BST_CHECKED {
                                (*keySettings).dwLanguage = 2;
                            } else {
                                (*keySettings).dwLanguage = 1;
                            }
                        } else {
                            (*keySettings).dwLanguage = 3;
                        }

                        if IsDlgButtonChecked(hwndDlg, (IDC_SWITCH_KBLAYOUTS_CB as i32)) == BST_CHECKED {
                            if IsDlgButtonChecked(hwndDlg, (IDC_CTRL_LAYOUT as i32)) == BST_CHECKED {
                                (*keySettings).dwLayout = 2;
                            } else {
                                (*keySettings).dwLayout = 1;
                            }
                        } else {
                            (*keySettings).dwLayout = 3;
                        }
                    }

                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}

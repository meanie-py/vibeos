// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_A06143DD154F: [u16; 8] = [0x0053u16, 0x0065, 0x0063, 0x006F, 0x006E, 0x0064, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut nBounceArray: [UINT; 5] = [
    500 as ::core::ffi::c_int as UINT,
    700 as ::core::ffi::c_int as UINT,
    1000 as ::core::ffi::c_int as UINT,
    1500 as ::core::ffi::c_int as UINT,
    2000 as ::core::ffi::c_int as UINT,
];

static mut nDelayArray: [UINT; 5] = [
    300 as ::core::ffi::c_int as UINT,
    700 as ::core::ffi::c_int as UINT,
    1000 as ::core::ffi::c_int as UINT,
    1500 as ::core::ffi::c_int as UINT,
    2000 as ::core::ffi::c_int as UINT,
];

static mut nRepeatArray: [UINT; 6] = [
    300 as ::core::ffi::c_int as UINT,
    500 as ::core::ffi::c_int as UINT,
    700 as ::core::ffi::c_int as UINT,
    1000 as ::core::ffi::c_int as UINT,
    1500 as ::core::ffi::c_int as UINT,
    2000 as ::core::ffi::c_int as UINT,
];

static mut nWaitArray: [UINT; 10] = [
    0 as ::core::ffi::c_int as UINT,
    300 as ::core::ffi::c_int as UINT,
    500 as ::core::ffi::c_int as UINT,
    700 as ::core::ffi::c_int as UINT,
    1000 as ::core::ffi::c_int as UINT,
    1500 as ::core::ffi::c_int as UINT,
    2000 as ::core::ffi::c_int as UINT,
    5000 as ::core::ffi::c_int as UINT,
    10000 as ::core::ffi::c_int as UINT,
    20000 as ::core::ffi::c_int as UINT,
];

pub const BOUNCETICKS: usize = 5;

pub const DELAYTICKS: usize = 5;

pub const REPEATTICKS: usize = 6;

pub const WAITTICKS: usize = 10;

unsafe extern "C" fn EnableFilterKeysTest(pGlobalData: PGLOBAL_DATA) {
    (*pGlobalData).filterKeys.dwFlags |= FKF_FILTERKEYSON as DWORD;
    (*pGlobalData).filterKeys.dwFlags &= !(FKF_INDICATOR as DWORD);

    SystemParametersInfo(
        SPI_SETFILTERKEYS as UINT,
        core::mem::size_of::<FILTERKEYS>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).filterKeys) as PVOID,
        0 as UINT,
    );
}

unsafe extern "C" fn DisableFilterKeysTest(mut pGlobalData: PGLOBAL_DATA) {
    if (*pGlobalData).oldFilterKeys.dwFlags & FKF_FILTERKEYSON as DWORD != 0 {
        (*pGlobalData).filterKeys.dwFlags |= FKF_FILTERKEYSON as DWORD;
    } else {
        (*pGlobalData).filterKeys.dwFlags &= !(FKF_FILTERKEYSON as DWORD);
    }

    if (*pGlobalData).oldFilterKeys.dwFlags & FKF_INDICATOR as DWORD != 0 {
        (*pGlobalData).filterKeys.dwFlags |= FKF_INDICATOR as DWORD;
    } else {
        (*pGlobalData).filterKeys.dwFlags &= !(FKF_INDICATOR as DWORD);
    }

    SystemParametersInfo(
        SPI_SETFILTERKEYS as UINT,
        core::mem::size_of::<FILTERKEYS>() as UINT,
        &raw mut (*pGlobalData).filterKeys as PVOID,
        0 as UINT,
    );
}
#[no_mangle]
pub unsafe extern "system" fn StickyKeysDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            memcpy(
                core::ptr::addr_of_mut!((*pGlobalData).oldStickyKeys) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*pGlobalData).stickyKeys) as *const core::ffi::c_void,
                core::mem::size_of::<STICKYKEYS>() as size_t,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_STICKY_ACTIVATE_CHECK as i32),
                (if (*pGlobalData).stickyKeys.dwFlags & SKF_HOTKEYACTIVE as DWORD != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_STICKY_LOCK_CHECK as i32),
                (if (*pGlobalData).stickyKeys.dwFlags & SKF_TRISTATE as DWORD != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_STICKY_UNLOCK_CHECK as i32),
                (if (*pGlobalData).stickyKeys.dwFlags & SKF_TWOKEYSOFF as DWORD != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_STICKY_SOUND_CHECK as i32),
                (if (*pGlobalData).stickyKeys.dwFlags & SKF_AUDIBLEFEEDBACK as DWORD != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_STICKY_STATUS_CHECK as i32),
                (if (*pGlobalData).stickyKeys.dwFlags & SKF_INDICATOR as DWORD != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_STICKY_ACTIVATE_CHECK => {
                    (*pGlobalData).stickyKeys.dwFlags ^= SKF_HOTKEYACTIVE as DWORD;
                }

                IDC_STICKY_LOCK_CHECK => {
                    (*pGlobalData).stickyKeys.dwFlags ^= SKF_TRISTATE as DWORD;
                }

                IDC_STICKY_UNLOCK_CHECK => {
                    (*pGlobalData).stickyKeys.dwFlags ^= SKF_TWOKEYSOFF as DWORD;
                }

                IDC_STICKY_SOUND_CHECK => {
                    (*pGlobalData).stickyKeys.dwFlags ^= SKF_AUDIBLEFEEDBACK as DWORD;
                }

                IDC_STICKY_STATUS_CHECK => {
                    (*pGlobalData).stickyKeys.dwFlags ^= SKF_INDICATOR as DWORD;
                }

                IDOK => {
                    EndDialog(
                        hwndDlg,
                        ((*pGlobalData).stickyKeys.dwFlags != (*pGlobalData).oldStickyKeys.dwFlags)
                            as ::core::ffi::c_int as INT_PTR,
                    );
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, FALSE as INT_PTR);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn AddComboBoxTime(hwnd: HWND, nId: INT, nTimeMs: INT) {
    let mut szBuffer: [TCHAR; 32] = [0; 32];
    let mut szSeconds: [TCHAR; 20] = [0; 20];

    if LoadString(hApplet, IDS_SECONDS, (core::ptr::addr_of_mut!(szSeconds) as LPTSTR).cast(), 20) == 0 {
        lstrcpy((core::ptr::addr_of_mut!(szSeconds) as LPTSTR).cast(), RUSTCONV_WSTR_A06143DD154F.as_ptr());
    }

    _stprintf(
        (core::ptr::addr_of_mut!(szBuffer) as *mut TCHAR).cast(),
        (&[0x0025, 0x0064, 0x002e, 0x0025, 0x0064, 0x0020, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T "%d.%d %s" */,
        nTimeMs / 1000,
        (nTimeMs % 1000) / 100,
        core::ptr::addr_of_mut!(szSeconds) as *mut TCHAR,
    );

    SendDlgItemMessage(hwnd, nId, CB_ADDSTRING, 0, core::ptr::addr_of_mut!(szBuffer) as *mut TCHAR as LPARAM);
}

#[no_mangle]
pub unsafe extern "system" fn BounceKeysDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut n: INT = 0;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            /* Determine the current bounce time */
            if (*pGlobalData).filterKeys.iBounceMSec == 0 {
                (*pGlobalData).filterKeys.iBounceMSec = nBounceArray[0 as usize];
            }

            n = 0;
            while n < (BOUNCETICKS as i32) {
                if (*pGlobalData).filterKeys.iBounceMSec < nBounceArray[n as usize] {
                    break;
                }
                n += 1;
            }
            n -= 1;

            i = 0;
            while i < (BOUNCETICKS as i32) {
                AddComboBoxTime(hwndDlg, (IDC_BOUNCE_TIME_COMBO) as INT, nBounceArray[i as usize] as INT);
                i += 1;
            }

            SendDlgItemMessage(hwndDlg, (IDC_BOUNCE_TIME_COMBO as i32), CB_SETCURSEL, n as WPARAM, 0);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as INT) as u32 {
                IDC_BOUNCE_TIME_COMBO => {
                    if (HIWORD!(wParam) as INT) as u32 == CBN_SELCHANGE {
                        i = SendDlgItemMessage(hwndDlg, (IDC_BOUNCE_TIME_COMBO as i32), CB_GETCURSEL, 0, 0) as INT;
                        if i != CB_ERR {
                            (*pGlobalData).filterKeys.iBounceMSec = nBounceArray[i as usize];
                        }
                    }
                }

                IDC_BOUNCE_TEST_EDIT => {
                    match (HIWORD!(wParam) as INT) as u32 {
                        EN_SETFOCUS => {
                            EnableFilterKeysTest(pGlobalData);
                        }

                        EN_KILLFOCUS => {
                            DisableFilterKeysTest(pGlobalData);
                        }

                        _ => {}
                    }
                }

                IDOK => {
                    EndDialog(hwndDlg, (TRUE) as INT_PTR);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, (FALSE) as INT_PTR);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn RepeatKeysDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut n: INT = 0;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            CheckRadioButton(
                hwndDlg,
                (IDC_REPEAT_NOREPEAT_RADIO as i32),
                (IDC_REPEAT_REPEAT_RADIO as i32),
                if (*pGlobalData).filterKeys.iDelayMSec == 0 as DWORD {
                    (IDC_REPEAT_NOREPEAT_RADIO as i32)
                } else {
                    (IDC_REPEAT_REPEAT_RADIO as i32)
                },
            );

            /* Initialize the delay combobox */
            n = 0 as INT;
            while n < DELAYTICKS as INT {
                if (*pGlobalData).filterKeys.iDelayMSec < nDelayArray[n as usize] {
                    break;
                }
                n += 1;
            }
            n -= 1;

            i = 0 as INT;
            while i < DELAYTICKS as INT {
                AddComboBoxTime(hwndDlg, (IDC_REPEAT_DELAY_COMBO) as INT, nDelayArray[i as usize] as INT);
                i += 1;
            }

            SendDlgItemMessage(
                hwndDlg,
                (IDC_REPEAT_DELAY_COMBO as UINT) as i32,
                CB_SETCURSEL,
                n as WPARAM,
                0 as LPARAM,
            );

            /* Initialize the repeat combobox */
            n = 0 as INT;
            while n < REPEATTICKS as INT {
                if (*pGlobalData).filterKeys.iRepeatMSec < nRepeatArray[n as usize] {
                    break;
                }
                n += 1;
            }
            n -= 1;

            i = 0 as INT;
            while i < REPEATTICKS as INT {
                AddComboBoxTime(hwndDlg, (IDC_REPEAT_REPEAT_COMBO) as INT, nRepeatArray[i as usize] as INT);
                i += 1;
            }

            SendDlgItemMessage(
                hwndDlg,
                (IDC_REPEAT_REPEAT_COMBO as UINT) as i32,
                CB_SETCURSEL,
                n as WPARAM,
                0 as LPARAM,
            );

            /* Disable the delay and repeat comboboxes if needed */
            if (*pGlobalData).filterKeys.iDelayMSec == 0 as DWORD {
                EnableWindow(GetDlgItem(hwndDlg, (IDC_REPEAT_DELAY_COMBO as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_REPEAT_REPEAT_COMBO as i32)), (FALSE) as BOOL);
            }

            /* Initialize the wait combobox */
            n = 0 as INT;
            while n < WAITTICKS as INT {
                if (*pGlobalData).filterKeys.iWaitMSec < nWaitArray[n as usize] {
                    break;
                }
                n += 1;
            }
            n -= 1;

            i = 0 as INT;
            while i < WAITTICKS as INT {
                AddComboBoxTime(hwndDlg, (IDC_REPEAT_WAIT_COMBO) as INT, nWaitArray[i as usize] as INT);
                i += 1;
            }

            SendDlgItemMessage(
                hwndDlg,
                (IDC_REPEAT_WAIT_COMBO as UINT) as i32,
                CB_SETCURSEL,
                n as WPARAM,
                0 as LPARAM,
            );
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as INT) as u32 {
                IDC_REPEAT_NOREPEAT_RADIO => {
                    (*pGlobalData).filterKeys.iDelayMSec = 0 as DWORD;
                    (*pGlobalData).filterKeys.iRepeatMSec = 0 as DWORD;
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_REPEAT_DELAY_COMBO as i32)), (FALSE) as BOOL);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_REPEAT_REPEAT_COMBO as i32)), (FALSE) as BOOL);
                }

                IDC_REPEAT_REPEAT_RADIO => {
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_REPEAT_DELAY_COMBO as i32)), (TRUE) as BOOL);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_REPEAT_REPEAT_COMBO as i32)), (TRUE) as BOOL);
                }

                IDC_REPEAT_DELAY_COMBO => {
                    if (HIWORD!(wParam) as INT) == CBN_SELCHANGE as INT {
                        i = SendDlgItemMessage(
                            hwndDlg,
                            (IDC_REPEAT_DELAY_COMBO as UINT) as i32,
                            CB_GETCURSEL,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) as INT;
                        if i != CB_ERR as INT {
                            (*pGlobalData).filterKeys.iDelayMSec = nDelayArray[i as usize] as DWORD;
                        }
                    }
                }

                IDC_REPEAT_REPEAT_COMBO => {
                    if (HIWORD!(wParam) as INT) == CBN_SELCHANGE as INT {
                        i = SendDlgItemMessage(
                            hwndDlg,
                            (IDC_REPEAT_REPEAT_COMBO as UINT) as i32,
                            CB_GETCURSEL,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) as INT;
                        if i != CB_ERR as INT {
                            (*pGlobalData).filterKeys.iRepeatMSec = nRepeatArray[i as usize] as DWORD;
                        }
                    }
                }

                IDC_REPEAT_WAIT_COMBO => {
                    if (HIWORD!(wParam) as INT) == CBN_SELCHANGE as INT {
                        i = SendDlgItemMessage(
                            hwndDlg,
                            (IDC_REPEAT_WAIT_COMBO as UINT) as i32,
                            CB_GETCURSEL,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) as INT;
                        if i != CB_ERR as INT {
                            (*pGlobalData).filterKeys.iWaitMSec = nWaitArray[i as usize] as DWORD;
                        }
                    }
                }

                IDC_REPEAT_TEST_EDIT => {
                    match (HIWORD!(wParam) as INT) as u32 {
                        EN_SETFOCUS => {
                            EnableFilterKeysTest(pGlobalData);
                        }

                        EN_KILLFOCUS => {
                            DisableFilterKeysTest(pGlobalData);
                        }

                        _ => {}
                    }
                }

                IDOK => {
                    EndDialog(hwndDlg, TRUE as INT_PTR);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, FALSE as INT_PTR);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn FilterKeysDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            memcpy(
                core::ptr::addr_of_mut!((*pGlobalData).oldFilterKeys) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*pGlobalData).filterKeys) as *const core::ffi::c_void,
                core::mem::size_of::<FILTERKEYS>() as usize,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_FILTER_ACTIVATE_CHECK as i32),
                if ((*pGlobalData).filterKeys.dwFlags & FKF_HOTKEYACTIVE) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                },
            );

            if (*pGlobalData).filterKeys.iBounceMSec != 0 {
                CheckRadioButton(hwndDlg, (IDC_FILTER_BOUNCE_RADIO as i32), (IDC_FILTER_REPEAT_RADIO as i32), (IDC_FILTER_BOUNCE_RADIO as i32));
                EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_BOUNCE_BUTTON as i32)), (TRUE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_REPEAT_BUTTON as i32)), (FALSE) as BOOL);
            } else {
                CheckRadioButton(hwndDlg, (IDC_FILTER_BOUNCE_RADIO as i32), (IDC_FILTER_REPEAT_RADIO as i32), (IDC_FILTER_REPEAT_RADIO as i32));
                EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_BOUNCE_BUTTON as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_REPEAT_BUTTON as i32)), (TRUE) as BOOL);
            }

            CheckDlgButton(
                hwndDlg,
                (IDC_FILTER_SOUND_CHECK as i32),
                if ((*pGlobalData).filterKeys.dwFlags & FKF_CLICKON) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                },
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_FILTER_STATUS_CHECK as i32),
                if ((*pGlobalData).filterKeys.dwFlags & FKF_INDICATOR) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                },
            );
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_FILTER_ACTIVATE_CHECK => {
                    (*pGlobalData).filterKeys.dwFlags ^= FKF_HOTKEYACTIVE;
                }

                IDC_FILTER_BOUNCE_RADIO => {
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_BOUNCE_BUTTON as i32)), (TRUE) as BOOL);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_REPEAT_BUTTON as i32)), (FALSE) as BOOL);
                }

                IDC_FILTER_REPEAT_RADIO => {
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_BOUNCE_BUTTON as i32)), (FALSE) as BOOL);
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_FILTER_REPEAT_BUTTON as i32)), (TRUE) as BOOL);
                }

                IDC_FILTER_BOUNCE_BUTTON => {
                    DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_BOUNCEKEYSOPTIONS),
                        hwndDlg,
                        Some(BounceKeysDlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
                        pGlobalData as LPARAM,
                    );
                }

                IDC_FILTER_TEST_EDIT => {
                    match (HIWORD!(wParam)) as u32 {
                        EN_SETFOCUS => {
                            EnableFilterKeysTest(pGlobalData);
                        }

                        EN_KILLFOCUS => {
                            DisableFilterKeysTest(pGlobalData);
                        }

                        _ => {}
                    }
                }

                IDC_FILTER_SOUND_CHECK => {
                    (*pGlobalData).filterKeys.dwFlags ^= FKF_CLICKON;
                }

                IDC_FILTER_REPEAT_BUTTON => {
                    DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_REPEATKEYSOPTIONS),
                        hwndDlg,
                        Some(RepeatKeysDlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
                        pGlobalData as LPARAM,
                    );
                }

                IDC_FILTER_STATUS_CHECK => {
                    (*pGlobalData).filterKeys.dwFlags ^= FKF_INDICATOR;
                }

                IDOK => {
                    EndDialog(
                        hwndDlg,
                        ((*pGlobalData).filterKeys.dwFlags != (*pGlobalData).oldFilterKeys.dwFlags) as i32 as INT_PTR,
                    );
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, (FALSE) as INT_PTR);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn ToggleKeysDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            memcpy(
                core::ptr::addr_of_mut!((*pGlobalData).oldToggleKeys) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*pGlobalData).toggleKeys) as *const core::ffi::c_void,
                core::mem::size_of::<TOGGLEKEYS>() as usize,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_TOGGLE_ACTIVATE_CHECK as i32),
                if (*pGlobalData).toggleKeys.dwFlags & TKF_HOTKEYACTIVE != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                },
            );
        }

        WM_COMMAND => match (LOWORD!(wParam)) as u32 {
            IDC_TOGGLE_ACTIVATE_CHECK => {
                (*pGlobalData).toggleKeys.dwFlags ^= TKF_HOTKEYACTIVE;
            }

            IDOK => {
                EndDialog(
                    hwndDlg,
                    (((*pGlobalData).toggleKeys.dwFlags != (*pGlobalData).oldToggleKeys.dwFlags)) as INT_PTR,
                );
            }

            IDCANCEL => {
                EndDialog(hwndDlg, (FALSE) as INT_PTR);
            }

            _ => {}
        },

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn KeyboardPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut lppsn: LPPSHNOTIFY = core::ptr::null_mut();

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    'sw: {
        if uMsg == WM_INITDIALOG {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBAL_DATA;
            if pGlobalData.is_null() {
                return FALSE as INT_PTR;
            }

            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            CheckDlgButton(
                hwndDlg,
                (IDC_STICKY_BOX as i32),
                (if ((*pGlobalData).stickyKeys.dwFlags & SKF_STICKYKEYSON) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_FILTER_BOX as i32),
                (if ((*pGlobalData).filterKeys.dwFlags & FKF_FILTERKEYSON) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_TOGGLE_BOX as i32),
                (if ((*pGlobalData).toggleKeys.dwFlags & TKF_TOGGLEKEYSON) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_KEYBOARD_EXTRA as i32),
                (if (*pGlobalData).bKeyboardPref != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );
            return TRUE as INT_PTR;
        }

        if uMsg == WM_COMMAND {
            'inner: {
                let cmd = LOWORD!(wParam);
                if cmd == (IDC_STICKY_BOX as u16) {
                    (*pGlobalData).stickyKeys.dwFlags ^= SKF_STICKYKEYSON;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    break 'inner;
                } else if cmd == (IDC_STICKY_BUTTON as u16) {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_STICKYKEYSOPTIONS) as LPCWSTR,
                        hwndDlg,
                        Some(StickyKeysDlgProc),
                        pGlobalData as LPARAM,
                    ) != 0
                    {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    break 'inner;
                } else if cmd == (IDC_FILTER_BOX as u16) {
                    (*pGlobalData).filterKeys.dwFlags ^= FKF_FILTERKEYSON;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    break 'inner;
                } else if cmd == (IDC_FILTER_BUTTON as u16) {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_FILTERKEYSOPTIONS) as LPCWSTR,
                        hwndDlg,
                        Some(FilterKeysDlgProc),
                        pGlobalData as LPARAM,
                    ) != 0
                    {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    break 'inner;
                } else if cmd == (IDC_TOGGLE_BOX as u16) {
                    (*pGlobalData).toggleKeys.dwFlags ^= TKF_TOGGLEKEYSON;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    break 'inner;
                } else if cmd == (IDC_TOGGLE_BUTTON as u16) {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_TOGGLEKEYSOPTIONS) as LPCWSTR,
                        hwndDlg,
                        Some(ToggleKeysDlgProc),
                        pGlobalData as LPARAM,
                    ) != 0
                    {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    break 'inner;
                } else if cmd == (IDC_KEYBOARD_EXTRA as u16) {
                    // C `!pGlobalData->bKeyboardPref` is a LOGICAL not; Rust's `!` on a BOOL is
                    // bitwise (1 -> -2, still true), so the preference could never be switched off.
                    (*pGlobalData).bKeyboardPref = ((*pGlobalData).bKeyboardPref == 0) as BOOL;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    break 'inner;
                } else {
                    break 'inner;
                }
            }
            break 'sw;
        }

        if uMsg == WM_NOTIFY {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == (PSN_APPLY) as u32 {
                SystemParametersInfo(
                    SPI_SETSTICKYKEYS,
                    core::mem::size_of::<STICKYKEYS>() as UINT,
                    &raw mut (*pGlobalData).stickyKeys as PVOID,
                    (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT /*0*/,
                );

                SystemParametersInfo(
                    SPI_SETFILTERKEYS,
                    core::mem::size_of::<FILTERKEYS>() as UINT,
                    &raw mut (*pGlobalData).filterKeys as PVOID,
                    (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT /*0*/,
                );

                SystemParametersInfo(
                    SPI_SETTOGGLEKEYS,
                    core::mem::size_of::<TOGGLEKEYS>() as UINT,
                    &raw mut (*pGlobalData).toggleKeys as PVOID,
                    (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT /*0*/,
                );

                SystemParametersInfo(
                    SPI_SETKEYBOARDPREF,
                    (*pGlobalData).bKeyboardPref as UINT,
                    core::ptr::null_mut(),
                    (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT /*0*/,
                );

                return TRUE as INT_PTR;
            }
            break 'sw;
        }
    }

    return FALSE as INT_PTR;
}

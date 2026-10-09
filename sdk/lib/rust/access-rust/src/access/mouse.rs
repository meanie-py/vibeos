use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut nSpeedArray: [UINT; 9] = [
    10 as ::core::ffi::c_int as UINT,
    20 as ::core::ffi::c_int as UINT,
    30 as ::core::ffi::c_int as UINT,
    40 as ::core::ffi::c_int as UINT,
    60 as ::core::ffi::c_int as UINT,
    80 as ::core::ffi::c_int as UINT,
    120 as ::core::ffi::c_int as UINT,
    180 as ::core::ffi::c_int as UINT,
    360 as ::core::ffi::c_int as UINT,
];

pub const SPEEDTICKS: usize = 9;

pub const ACCELTICKS: usize = 9;

#[no_mangle]
pub unsafe extern "system" fn MouseKeysDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut i: INT = 0;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            CheckDlgButton(
                hwndDlg,
                (IDC_MOUSEKEYS_ACTIVATE_CHECK as i32),
                (if ((*pGlobalData).mouseKeys.dwFlags & MKF_HOTKEYACTIVE) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            /* Set the number of ticks for the speed trackbar */
            SendDlgItemMessage(
                hwndDlg,
                (IDC_MOUSEKEYS_SPEED_TRACK as i32),
                TBM_SETRANGE,
                (TRUE) as _,
                MAKELONG!(0, SPEEDTICKS - 1) as LPARAM,
            );

            /* Calculate the matching tick */
            i = 0;
            while i < (SPEEDTICKS as i32) {
                if (*pGlobalData).mouseKeys.iMaxSpeed <= nSpeedArray[i as usize] {
                    break;
                }
                i += 1;
            }

            /* Set the thumb */
            SendDlgItemMessage(hwndDlg, (IDC_MOUSEKEYS_SPEED_TRACK as i32), TBM_SETPOS, (TRUE) as _, i as LPARAM);

            /* Set the number of ticks for the acceleration trackbar */
            SendDlgItemMessage(
                hwndDlg,
                (IDC_MOUSEKEYS_ACCEL_TRACK as i32),
                TBM_SETRANGE,
                (TRUE) as _,
                MAKELONG!(0, ACCELTICKS - 1) as LPARAM,
            );

            /* Calculate the matching tick */
            i = ((ACCELTICKS + 1) as DWORD).wrapping_sub(
                (*pGlobalData).mouseKeys.iTimeToMaxSpeed / (500 as DWORD),
            ) as INT;
            if i > (ACCELTICKS - 1) as i32 {
                i = (ACCELTICKS - 1) as i32;
            }

            if i < 0 {
                i = 0;
            }

            /* Set the thumb */
            SendDlgItemMessage(hwndDlg, (IDC_MOUSEKEYS_ACCEL_TRACK as i32), TBM_SETPOS, (TRUE) as _, i as LPARAM);

            CheckDlgButton(
                hwndDlg,
                (IDC_MOUSEKEYS_SPEED_CHECK as i32),
                (if ((*pGlobalData).mouseKeys.dwFlags & MKF_MODIFIERS) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            CheckRadioButton(
                hwndDlg,
                (IDC_MOUSEKEYS_ON_RADIO as i32),
                (IDC_MOUSEKEYS_OFF_RADIO as i32),
                ((if ((*pGlobalData).mouseKeys.dwFlags & MKF_REPLACENUMBERS) != 0 {
                    IDC_MOUSEKEYS_ON_RADIO
                } else {
                    IDC_MOUSEKEYS_OFF_RADIO
                }) as UINT) as i32,
            );

            CheckDlgButton(
                hwndDlg,
                (IDC_MOUSEKEYS_STATUS_CHECK as i32),
                (if ((*pGlobalData).mouseKeys.dwFlags & MKF_INDICATOR) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );
        }

        WM_HSCROLL => {
            match (GetWindowLongPtr(lParam as HWND, GWL_ID)) as u32 {
                IDC_MOUSEKEYS_SPEED_TRACK => {
                    i = SendDlgItemMessage(hwndDlg, (IDC_MOUSEKEYS_SPEED_TRACK as i32), TBM_GETPOS, 0, 0) as INT;
                    if i >= 0 && i < (SPEEDTICKS as i32) {
                        (*pGlobalData).mouseKeys.iMaxSpeed = nSpeedArray[i as usize] as DWORD;
                    }
                }

                IDC_MOUSEKEYS_ACCEL_TRACK => {
                    i = SendDlgItemMessage(hwndDlg, (IDC_MOUSEKEYS_ACCEL_TRACK as i32), TBM_GETPOS, 0, 0) as INT;
                    if i >= 0 && i < (ACCELTICKS as i32) {
                        (*pGlobalData).mouseKeys.iTimeToMaxSpeed =
                            ((ACCELTICKS + 1 - (i) as usize) * 500) as DWORD;
                    }
                }

                _ => {}
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as INT) as u32 {
                IDC_MOUSEKEYS_ACTIVATE_CHECK => {
                    (*pGlobalData).mouseKeys.dwFlags ^= MKF_HOTKEYACTIVE as DWORD;
                }

                IDC_MOUSEKEYS_SPEED_CHECK => {
                    (*pGlobalData).mouseKeys.dwFlags ^= MKF_MODIFIERS as DWORD;
                }

                IDC_MOUSEKEYS_ON_RADIO => {
                    (*pGlobalData).mouseKeys.dwFlags |= MKF_REPLACENUMBERS as DWORD;
                }

                IDC_MOUSEKEYS_OFF_RADIO => {
                    (*pGlobalData).mouseKeys.dwFlags &= !(MKF_REPLACENUMBERS as DWORD);
                }

                IDC_MOUSEKEYS_STATUS_CHECK => {
                    (*pGlobalData).mouseKeys.dwFlags ^= MKF_INDICATOR as DWORD;
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

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn MousePageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut lppsn: LPPSHNOTIFY = core::ptr::null_mut();

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBAL_DATA;
            if pGlobalData.is_null() {
                return FALSE as INT_PTR;
            }

            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            /* Set the checkbox */
            CheckDlgButton(
                hwndDlg,
                (IDC_MOUSE_BOX as i32),
                if ((*pGlobalData).mouseKeys.dwFlags & MKF_MOUSEKEYSON) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                },
            );

            EnableWindow(
                GetDlgItem(hwndDlg, (IDC_MOUSE_BOX as i32)),
                ((*pGlobalData).mouseKeys.dwFlags & MKF_MOUSEKEYSON) as BOOL,
            );
            EnableWindow(
                GetDlgItem(hwndDlg, (IDC_MOUSE_BUTTON as i32)),
                ((*pGlobalData).mouseKeys.dwFlags & MKF_MOUSEKEYSON) as BOOL,
            );
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_MOUSE_BOX => {
                    (*pGlobalData).mouseKeys.dwFlags ^= MKF_MOUSEKEYSON;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                IDC_MOUSE_BUTTON => {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_MOUSEKEYSOPTIONS),
                        hwndDlg,
                        Some(MouseKeysDlgProc),
                        pGlobalData as LPARAM,
                    ) != 0
                    {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == (PSN_APPLY) as u32 {
                SystemParametersInfo(
                    SPI_SETMOUSEKEYS as UINT,
                    core::mem::size_of::<MOUSEKEYS>() as UINT,
                    core::ptr::addr_of_mut!((*pGlobalData).mouseKeys) as PVOID,
                    (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT, /*0*/
                );
                return TRUE as INT_PTR;
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

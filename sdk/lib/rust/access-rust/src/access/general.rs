// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2AF6AEA959E9: [u16; 8] = [0x004Du16, 0x0069, 0x006E, 0x0075, 0x0074, 0x0065, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut nBaudArray: [UINT; 6] = [
    300 as ::core::ffi::c_int as UINT,
    1200 as ::core::ffi::c_int as UINT,
    2400 as ::core::ffi::c_int as UINT,
    4800 as ::core::ffi::c_int as UINT,
    9600 as ::core::ffi::c_int as UINT,
    19200 as ::core::ffi::c_int as UINT,
];

pub const BAUDTICKS: usize = 6;

pub const FIVE_MINS_IN_MS: u32 = 5 * 60 * 1000;
#[no_mangle]
pub unsafe extern "system" fn SerialKeysDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut i: INT;
    let mut n: INT;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            /* Add the COM-Ports 1 - 4 to the list */
            i = 0;
            while i < 4 {
                let mut szBuffer: [TCHAR; 256] = [0 as TCHAR; 256];
                let fmtComPercentD: [u16; 6] = ['C' as u16, 'O' as u16, 'M' as u16, '%' as u16, 'd' as u16, 0];

                _stprintf((szBuffer.as_mut_ptr()).cast(), fmtComPercentD.as_ptr(), i + 1);
                SendDlgItemMessage(hwndDlg, (IDC_SERIAL_PORT_COMBO as i32), CB_ADDSTRING, 0, szBuffer.as_mut_ptr() as LPARAM);
                i += 1;
            }

            /* Determine the current port */
            if !(*pGlobalData).serialKeys.lpszActivePort.is_null()
                && *(*pGlobalData).serialKeys.lpszActivePort.offset(0 as isize) != 0
            {
                i = (*(*pGlobalData).serialKeys.lpszActivePort.offset(3 as isize) as INT) - '1' as INT;
                if i < 0 || i > 3 {
                    i = 0;
                }
            } else {
                /* Make COM1 the default port */
                i = 0;
                let fmtCom1: [u16; 5] = ['C' as u16, 'O' as u16, 'M' as u16, '1' as u16, 0];
                _tcscpy(((*pGlobalData).serialKeys.lpszActivePort).cast(), fmtCom1.as_ptr());
            }

            /* Set the current port */
            SendDlgItemMessage(hwndDlg, (IDC_SERIAL_PORT_COMBO as i32), CB_SETCURSEL, i as WPARAM, 0);

            /* Determine the current baud rate */
            n = 0;
            i = 0;
            while i < (BAUDTICKS as i32) {
                let mut szBuffer_0: [TCHAR; 256] = [0 as TCHAR; 256];
                let fmtBaud: [u16; 8] = ['%' as u16, 'd' as u16, ' ' as u16, 'B' as u16, 'a' as u16, 'u' as u16, 'd' as u16, 0];

                _stprintf((szBuffer_0.as_mut_ptr()).cast(), fmtBaud.as_ptr(), nBaudArray[i as usize]);
                SendDlgItemMessage(hwndDlg, (IDC_SERIAL_BAUD_COMBO as i32), CB_ADDSTRING, 0, szBuffer_0.as_mut_ptr() as LPARAM);

                if (*pGlobalData).serialKeys.iBaudRate == nBaudArray[i as usize] {
                    n = i;
                }
                i += 1;
            }

            /* Set the current baud rate */
            SendDlgItemMessage(hwndDlg, (IDC_SERIAL_BAUD_COMBO as i32), CB_SETCURSEL, n as WPARAM, 0);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as INT) as u32 {
                IDOK => {
                    let fmtComPercentD2: [u16; 6] = ['C' as u16, 'O' as u16, 'M' as u16, '%' as u16, 'd' as u16, 0];
                    i = (SendDlgItemMessage(hwndDlg, (IDC_SERIAL_PORT_COMBO as i32), CB_GETCURSEL, 0, 0) + 1) as INT;
                    _stprintf(((*pGlobalData).serialKeys.lpszActivePort).cast(), fmtComPercentD2.as_ptr(), i);

                    i = SendDlgItemMessage(hwndDlg, (IDC_SERIAL_BAUD_COMBO as i32), CB_GETCURSEL, 0, 0) as INT;
                    // CB_GETCURSEL is CB_ERR (-1) with nothing selected; the C then reads
                    // nBaudArray[-1]. Keep the current rate instead of reading out of bounds.
                    if let Some(&rate) = usize::try_from(i).ok().and_then(|i| (*core::ptr::addr_of!(nBaudArray)).get(i)) {
                        (*pGlobalData).serialKeys.iBaudRate = rate;
                    }

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

unsafe extern "C" fn FillResetComboBox(hwnd: HWND) {
    let mut szBuffer: [TCHAR; 32] = [0; 32];
    let mut szMinutes: [TCHAR; 20] = [0; 20];
    let mut i: INT;

    if LoadString(
        hApplet,
        IDS_MINUTES,
        (szMinutes.as_mut_ptr()).cast(),
        20,
    ) == 0
    {
        lstrcpy((szMinutes.as_mut_ptr()).cast(), RUSTCONV_WSTR_2AF6AEA959E9.as_ptr());
    }

    i = 0;
    while i < 6 {
        _stprintf(
            (szBuffer.as_mut_ptr()).cast(),
            (&[0x0025, 0x0075, 0x0020, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T "%u %s" */,
            (i + 1) * 5,
            szMinutes.as_mut_ptr(),
        );
        SendMessage(
            hwnd,
            CB_ADDSTRING,
            0,
            szBuffer.as_mut_ptr() as LPARAM,
        );
        i += 1;
    }
}

unsafe extern "C" fn WriteGlobalData(pGlobalData: PGLOBAL_DATA) {
    let mut dwDisposition: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;

    SystemParametersInfo(
        SPI_SETACCESSTIMEOUT as UINT,
        core::mem::size_of::<ACCESSTIMEOUT>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).accessTimeout) as PVOID,
        (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT, /*0*/
    );

    SystemParametersInfo(
        SPI_SETSERIALKEYS as UINT,
        core::mem::size_of::<SERIALKEYS>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).serialKeys) as PVOID,
        (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT, /*0*/
    );

    lError = RegCreateKeyEx(
        HKEY_CURRENT_USER as HKEY,
        [
            'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
            ' ' as u16, 'P' as u16, 'a' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16,
            'A' as u16, 'c' as u16, 'c' as u16, 'e' as u16, 's' as u16, 's' as u16, 'i' as u16,
            'b' as u16, 'i' as u16, 'l' as u16, 'i' as u16, 't' as u16, 'y' as u16, 0u16,
        ]
        .as_ptr() as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE as DWORD,
        KEY_SET_VALUE as REGSAM,
        core::ptr::null_mut(),
        &raw mut hKey,
        &raw mut dwDisposition,
    ) as LONG;
    if lError != ERROR_SUCCESS as LONG {
        return;
    }

    RegSetValueEx(
        hKey,
        [
            'W' as u16, 'a' as u16, 'r' as u16, 'n' as u16, 'i' as u16, 'n' as u16, 'g' as u16,
            ' ' as u16, 'S' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 'd' as u16, 's' as u16,
            0u16,
        ]
        .as_ptr() as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        &raw mut (*pGlobalData).bWarningSounds as LPBYTE as *const BYTE,
        core::mem::size_of::<BOOL>() as DWORD,
    );

    RegSetValueEx(
        hKey,
        [
            'S' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'O' as u16,
            'n' as u16, ' ' as u16, 'A' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16,
            'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0u16,
        ]
        .as_ptr() as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        &raw mut (*pGlobalData).bSoundOnActivation as LPBYTE as *const BYTE,
        core::mem::size_of::<BOOL>() as DWORD,
    );

    RegCloseKey(hKey);
}

#[no_mangle]
pub unsafe extern "system" fn GeneralPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut lppsn: LPPSHNOTIFY = core::ptr::null_mut();
    let mut iCurSel: INT = 0;

    pGlobalData = GetWindowLongPtr(hwndDlg, (DWLP_USER)) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBAL_DATA;
            if pGlobalData.is_null() {
                return (FALSE) as INT_PTR;
            }

            SetWindowLongPtr(hwndDlg, (DWLP_USER), pGlobalData as LONG_PTR);

            /* Set access timeout info */
            CheckDlgButton(
                hwndDlg,
                ((IDC_RESET_BOX)) as i32,
                (if (*pGlobalData).accessTimeout.dwFlags & (ATF_TIMEOUTON) != 0 {
                    (BST_CHECKED)
                } else {
                    (BST_UNCHECKED)
                }) as UINT,
            );
            FillResetComboBox(GetDlgItem(hwndDlg, ((IDC_RESET_COMBO)) as i32));
            (*pGlobalData).accessTimeout.iTimeOutMSec = max(
                (FIVE_MINS_IN_MS) as DWORD,
                (*pGlobalData).accessTimeout.iTimeOutMSec,
            );
            iCurSel = (((*pGlobalData).accessTimeout.iTimeOutMSec
                / ((FIVE_MINS_IN_MS) as DWORD))
                .wrapping_sub((1) as DWORD)) as INT;
            SendDlgItemMessage(
                hwndDlg,
                ((IDC_RESET_COMBO)) as i32,
                (CB_SETCURSEL) as UINT,
                iCurSel as WPARAM,
                (0) as LPARAM,
            );
            EnableWindow(
                GetDlgItem(hwndDlg, ((IDC_RESET_COMBO)) as i32),
                (if (*pGlobalData).accessTimeout.dwFlags & (ATF_TIMEOUTON) != 0 {
                    (TRUE)
                } else {
                    (FALSE)
                }) as BOOL,
            );

            CheckDlgButton(
                hwndDlg,
                ((IDC_NOTIFICATION_MESSAGE)) as i32,
                (if (*pGlobalData).bWarningSounds != 0 {
                    (BST_CHECKED)
                } else {
                    (BST_UNCHECKED)
                }) as UINT,
            );

            CheckDlgButton(
                hwndDlg,
                ((IDC_NOTIFICATION_SOUND)) as i32,
                (if (*pGlobalData).bSoundOnActivation != 0 {
                    (BST_CHECKED)
                } else {
                    (BST_UNCHECKED)
                }) as UINT,
            );

            /* Set serial keys info */
            CheckDlgButton(
                hwndDlg,
                ((IDC_SERIAL_BOX)) as i32,
                (if (*pGlobalData).serialKeys.dwFlags & (SERKF_SERIALKEYSON) != 0 {
                    (BST_CHECKED)
                } else {
                    (BST_UNCHECKED)
                }) as UINT,
            );
            EnableWindow(
                GetDlgItem(hwndDlg, ((IDC_SERIAL_BOX)) as i32),
                (if (*pGlobalData).serialKeys.dwFlags & (SERKF_AVAILABLE) != 0 {
                    (TRUE)
                } else {
                    (FALSE)
                }) as BOOL,
            );
            EnableWindow(
                GetDlgItem(hwndDlg, ((IDC_SERIAL_BUTTON)) as i32),
                (if (*pGlobalData).serialKeys.dwFlags & (SERKF_AVAILABLE) != 0 {
                    (TRUE)
                } else {
                    (FALSE)
                }) as BOOL,
            );

            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_RESET_BOX => {
                    (*pGlobalData).accessTimeout.dwFlags ^= (ATF_TIMEOUTON);
                    EnableWindow(
                        GetDlgItem(hwndDlg, ((IDC_RESET_COMBO)) as i32),
                        (if (*pGlobalData).accessTimeout.dwFlags & (ATF_TIMEOUTON) != 0 {
                            (TRUE)
                        } else {
                            (FALSE)
                        }) as BOOL,
                    );
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                IDC_RESET_COMBO => {
                    if (HIWORD!(wParam) as ::core::ffi::c_int) as u32 == (CBN_CLOSEUP) {
                        let mut nSel: INT = 0;
                        nSel = SendDlgItemMessage(
                            hwndDlg,
                            ((IDC_RESET_COMBO)) as i32,
                            (CB_GETCURSEL) as UINT,
                            (0) as WPARAM,
                            (0) as LPARAM,
                        ) as INT;
                        (*pGlobalData).accessTimeout.iTimeOutMSec =
                            (((nSel + (1)) * ((FIVE_MINS_IN_MS)) as i32) as DWORD);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_NOTIFICATION_MESSAGE => {
                    (*pGlobalData).bWarningSounds =
                        ((*pGlobalData).bWarningSounds == 0) as ::core::ffi::c_int as BOOL;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                IDC_NOTIFICATION_SOUND => {
                    (*pGlobalData).bSoundOnActivation =
                        ((*pGlobalData).bSoundOnActivation == 0) as ::core::ffi::c_int as BOOL;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                IDC_SERIAL_BOX => {
                    (*pGlobalData).serialKeys.dwFlags ^= (SERKF_SERIALKEYSON);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                IDC_SERIAL_BUTTON => {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!((IDD_SERIALKEYSOPTIONS)),
                        hwndDlg,
                        Some(
                            SerialKeysDlgProc,
                        ),
                        pGlobalData as LPARAM,
                    ) != 0
                    {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_ADMIN_LOGON_BOX => {}

                IDC_ADMIN_USERS_BOX => {}

                _ => {}
            }
        }

        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == (PSN_APPLY) as UINT {
                WriteGlobalData(pGlobalData);
                return (TRUE) as INT_PTR;
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}

// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8476B7D2BE8A: [u16; 16] = [
    0x0043u16, 0x0075, 0x0072, 0x0073, 0x006F, 0x0072, 0x0042, 0x006C, 0x0069, 0x006E,
    0x006B, 0x0052, 0x0061, 0x0074, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_C47486B2BD9A: [u16; 22] = [
    0x0043u16, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061,
    0x006E, 0x0065, 0x006C, 0x005C, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F,
    0x0070, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ID_BLINK_TIMER: i32 = 346;

unsafe extern "C" fn UpdateCaretBlinkTimeReg(uCaretBlinkTime: UINT) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szBuffer: [WCHAR; 12] = [0; 12];

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_C47486B2BD9A.as_ptr(),
        0,
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(hKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return;
    }

    wsprintf(
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPWSTR,
        RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
        uCaretBlinkTime,
    );

    RegSetValueExW(
        hKey,
        RUSTCONV_WSTR_8476B7D2BE8A.as_ptr(),
        0 as DWORD,
        REG_SZ as DWORD,
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as *const BYTE,
        (((wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as *const WCHAR) as usize)
            + 1 as usize)
            * (core::mem::size_of::<WCHAR>() as usize)) as DWORD,
    );

    RegCloseKey(hKey);
}

unsafe extern "C" fn FillColorSchemeComboBox(hwnd: HWND) {
    let mut szValue: [TCHAR; 128] = [0 as TCHAR; 128];
    let mut dwDisposition: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;
    let mut i: INT = 0;
    let SubKeyName: [u16; 33] = [
        'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16,
        'P' as u16, 'a' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16,
        'A' as u16, 'p' as u16, 'p' as u16, 'e' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 'n' as u16,
        'c' as u16, 'e' as u16, '\\' as u16,
        'S' as u16, 'c' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, 's' as u16,
        0,
    ];

    lError = RegCreateKeyEx(HKEY_CURRENT_USER,
                            SubKeyName.as_ptr() as LPCWSTR,
                            0,
                            core::ptr::null_mut(),
                            REG_OPTION_NON_VOLATILE,
                            KEY_ENUMERATE_SUB_KEYS | KEY_EXECUTE | KEY_QUERY_VALUE,
                            core::ptr::null_mut(),
                            &raw mut hKey,
                            &raw mut dwDisposition);
    if (lError) as u32 != ERROR_SUCCESS {
        return;
    }

    i = 0;
    loop {
        dwLength = 128;
        lError = RegEnumValue(hKey,
                              i as DWORD,
                              &raw mut szValue as LPWSTR,
                              &raw mut dwLength,
                              core::ptr::null_mut(),
                              core::ptr::null_mut(),
                              core::ptr::null_mut(),
                              core::ptr::null_mut());
        if (lError) as u32 == ERROR_NO_MORE_ITEMS {
            break;
        }

        SendMessage(hwnd,
                    CB_ADDSTRING,
                    0,
                    &raw mut szValue as *mut TCHAR as LPARAM);
        i += 1;
    }

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "system" fn HighContrastDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER as core::ffi::c_int) as PGLOBAL_DATA;

    match (uMsg as core::ffi::c_int) as u32 {
        WM_INITDIALOG => {
            pGlobalData = lParam as PGLOBAL_DATA;
            SetWindowLongPtr(
                hwndDlg,
                DWLP_USER as core::ffi::c_int,
                pGlobalData as LONG_PTR,
            );

            CheckDlgButton(
                hwndDlg,
                IDC_CONTRAST_ACTIVATE_CHECK as core::ffi::c_int,
                (if ((*pGlobalData).highContrast.dwFlags & (HCF_HOTKEYACTIVE as DWORD)) != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            FillColorSchemeComboBox(GetDlgItem(hwndDlg, IDC_CONTRAST_COMBO as core::ffi::c_int));

            SendDlgItemMessage(
                hwndDlg,
                IDC_CONTRAST_COMBO as core::ffi::c_int,
                CB_SELECTSTRING as UINT,
                (-(1 as core::ffi::c_int)) as WPARAM,
                (*pGlobalData).highContrast.lpszDefaultScheme as LPARAM,
            );
        }
        WM_COMMAND => match ((wParam & 0xffff) as WORD as core::ffi::c_int) as u32 {
            IDC_CONTRAST_ACTIVATE_CHECK => {
                (*pGlobalData).highContrast.dwFlags ^= HCF_HOTKEYACTIVE as DWORD;
            }
            IDC_CONTRAST_COMBO => {
                if (((wParam >> 16) & 0xffff) as WORD as core::ffi::c_int) as u32 == CBN_SELCHANGE {
                    let mut nSel: INT = 0;

                    nSel = SendDlgItemMessage(
                        hwndDlg,
                        IDC_CONTRAST_COMBO as core::ffi::c_int,
                        CB_GETCURSEL as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as INT;
                    SendDlgItemMessage(
                        hwndDlg,
                        IDC_CONTRAST_COMBO as core::ffi::c_int,
                        CB_GETLBTEXT as UINT,
                        nSel as WPARAM,
                        (*pGlobalData).highContrast.lpszDefaultScheme as LPARAM,
                    );
                }
            }
            IDOK => {
                EndDialog(hwndDlg, TRUE as INT_PTR);
            }
            IDCANCEL => {
                EndDialog(hwndDlg, FALSE as INT_PTR);
            }
            _ => {}
        },
        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn DisplayPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();
    let mut lppsn: LPPSHNOTIFY = core::ptr::null_mut();
    let mut i: INT = 0;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBAL_DATA;
            if pGlobalData.is_null() {
                return FALSE as INT_PTR;
            }

            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            (*pGlobalData).fShowCaret = TRUE as BOOL;
            GetWindowRect(
                GetDlgItem(hwndDlg, (IDC_CURSOR_WIDTH_TEXT as i32)),
                &raw mut (*pGlobalData).rcCaret,
            );
            ScreenToClient(hwndDlg, &raw mut (*pGlobalData).rcCaret.left as LPPOINT);
            ScreenToClient(hwndDlg, &raw mut (*pGlobalData).rcCaret.right as LPPOINT);
            CopyRect(
                &raw mut (*pGlobalData).rcOldCaret,
                &raw mut (*pGlobalData).rcCaret as LPCRECT,
            );

            (*pGlobalData).rcCaret.right = ((*pGlobalData).rcCaret.left as UINT)
                .wrapping_add((*pGlobalData).uCaretWidth) as LONG;

            /* Set the checkbox */
            CheckDlgButton(
                hwndDlg,
                (IDC_CONTRAST_BOX as i32),
                if (((*pGlobalData).highContrast.dwFlags & HCF_HIGHCONTRASTON) != 0) {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                },
            );

            SendDlgItemMessage(
                hwndDlg,
                (IDC_CURSOR_BLINK_TRACK as i32),
                TBM_SETRANGE,
                TRUE as WPARAM,
                MAKELONG!(0, 10) as LPARAM,
            );
            SendDlgItemMessage(
                hwndDlg,
                (IDC_CURSOR_BLINK_TRACK as i32),
                TBM_SETPOS,
                TRUE as WPARAM,
                (12 - ((*pGlobalData).uCaretBlinkTime / 100))
                    as LPARAM,
            );

            SendDlgItemMessage(
                hwndDlg,
                (IDC_CURSOR_WIDTH_TRACK as i32),
                TBM_SETRANGE,
                TRUE as WPARAM,
                MAKELONG!(0, 19) as LPARAM,
            );
            SendDlgItemMessage(
                hwndDlg,
                (IDC_CURSOR_WIDTH_TRACK as i32),
                TBM_SETPOS,
                TRUE as WPARAM,
                ((*pGlobalData).uCaretWidth.wrapping_sub(1 as UINT)) as LPARAM,
            );

            /* Start the blink timer */
            (*pGlobalData).uCaretBlinkTime = if (*pGlobalData).uCaretBlinkTime >= 1200 {
                (0 as UINT).wrapping_sub(1 as UINT)
            } else {
                (*pGlobalData).uCaretBlinkTime
            };
            if ((*pGlobalData).uCaretBlinkTime as INT) > (0 as INT) {
                SetTimer(
                    hwndDlg,
                    ID_BLINK_TIMER as UINT_PTR,
                    (*pGlobalData).uCaretBlinkTime,
                    None,
                );
            } else {
                PostMessage(
                    hwndDlg,
                    WM_TIMER as UINT,
                    ID_BLINK_TIMER as WPARAM,
                    0 as LPARAM,
                );
            }

            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_CONTRAST_BOX => {
                    (*pGlobalData).highContrast.dwFlags ^= HCF_HIGHCONTRASTON as DWORD;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
                IDC_CONTRAST_BUTTON => {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_CONTRASTOPTIONS),
                        hwndDlg,
                        Some(HighContrastDlgProc),
                        pGlobalData as LPARAM,
                    ) != 0
                    {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                _ => {}
            }
        }

        WM_HSCROLL => {
            match (GetWindowLongPtr(lParam as HWND, GWL_ID)) as u32 {
                IDC_CURSOR_BLINK_TRACK => {
                    i = SendDlgItemMessage(
                        hwndDlg,
                        (IDC_CURSOR_BLINK_TRACK as i32),
                        TBM_GETPOS,
                        0,
                        0,
                    ) as INT;
                    (*pGlobalData).uCaretBlinkTime = (12 as UINT)
                        .wrapping_sub(i as UINT)
                        .wrapping_mul(100 as UINT);
                    KillTimer(hwndDlg, (ID_BLINK_TIMER) as UINT_PTR);
                    (*pGlobalData).uCaretBlinkTime = if (*pGlobalData).uCaretBlinkTime >= 1200 {
                        (0 as UINT).wrapping_sub(1 as UINT)
                    } else {
                        (*pGlobalData).uCaretBlinkTime
                    };
                    if ((*pGlobalData).uCaretBlinkTime as INT) > (0 as INT) {
                        SetTimer(
                            hwndDlg,
                            ID_BLINK_TIMER as UINT_PTR,
                            (*pGlobalData).uCaretBlinkTime,
                            None,
                        );
                    } else if (*pGlobalData).fShowCaret != 0 {
                        SendMessage(
                            hwndDlg,
                            WM_TIMER as UINT,
                            ID_BLINK_TIMER as WPARAM,
                            0 as LPARAM,
                        );
                    }
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                IDC_CURSOR_WIDTH_TRACK => {
                    i = SendDlgItemMessage(
                        hwndDlg,
                        (IDC_CURSOR_WIDTH_TRACK as i32),
                        TBM_GETPOS,
                        0,
                        0,
                    ) as INT;
                    (*pGlobalData).uCaretWidth = (i as UINT).wrapping_add(1 as UINT);
                    (*pGlobalData).rcCaret.right = ((*pGlobalData).rcCaret.left as UINT)
                        .wrapping_add((*pGlobalData).uCaretWidth) as LONG;
                    if (*pGlobalData).fShowCaret != 0 {
                        let hDC: HDC = GetDC(hwndDlg);
                        let hBrush: HBRUSH = GetSysColorBrush((COLOR_BTNTEXT) as i32);
                        FillRect(hDC, &raw mut (*pGlobalData).rcCaret as LPCRECT, hBrush);
                        ReleaseDC(hwndDlg, hDC);
                    } else {
                        InvalidateRect(
                            hwndDlg,
                            &raw mut (*pGlobalData).rcOldCaret as LPCRECT,
                            TRUE as BOOL,
                        );
                    }
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
                _ => {}
            }
        }

        WM_TIMER => {
            if wParam == ID_BLINK_TIMER as WPARAM {
                if (*pGlobalData).fShowCaret != 0 {
                    let hDC: HDC = GetDC(hwndDlg);
                    let hBrush: HBRUSH = GetSysColorBrush((COLOR_BTNTEXT) as i32);
                    FillRect(hDC, &raw mut (*pGlobalData).rcCaret as LPCRECT, hBrush);
                    ReleaseDC(hwndDlg, hDC);
                } else {
                    InvalidateRect(
                        hwndDlg,
                        &raw mut (*pGlobalData).rcOldCaret as LPCRECT,
                        TRUE as BOOL,
                    );
                }

                (*pGlobalData).fShowCaret = (((*pGlobalData).fShowCaret == 0) as i32) as BOOL;
            }
        }

        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == PSN_APPLY as _ {
                if SetCaretBlinkTime((*pGlobalData).uCaretBlinkTime) != 0 {
                    UpdateCaretBlinkTimeReg((*pGlobalData).uCaretBlinkTime);
                }

                SystemParametersInfo(
                    SPI_SETCARETWIDTH as UINT,
                    0 as UINT,
                    IntToPtr!((*pGlobalData).uCaretWidth as INT) as PVOID,
                    (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT, /*0*/
                );
                SystemParametersInfo(
                    SPI_SETHIGHCONTRAST as UINT,
                    core::mem::size_of::<HIGHCONTRAST>() as UINT,
                    &raw mut (*pGlobalData).highContrast as PVOID,
                    (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT, /*0*/
                );
                return TRUE as INT_PTR;
            }
        }

        WM_DESTROY => {
            KillTimer(hwndDlg, (ID_BLINK_TIMER) as UINT_PTR);
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

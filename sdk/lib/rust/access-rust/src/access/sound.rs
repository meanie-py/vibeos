use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn OnInitDialog(hwndDlg: HWND, pGlobalData: PGLOBAL_DATA) {
    let mut szBuffer: [TCHAR; 256] = [0 as TCHAR; 256];
    let mut i: UINT;

    /* Add strings to the combo-box */
    i = 0;
    while i < 4 {
        LoadString(hApplet, IDS_SENTRY_NONE + i, (szBuffer.as_mut_ptr()).cast(), 256);
        SendDlgItemMessage(hwndDlg, (IDC_SENTRY_COMBO as i32), CB_ADDSTRING, 0, core::ptr::addr_of_mut!(szBuffer) as *mut TCHAR as LPARAM);
        i += 1;
    }

    /* Select a combo-box item */
    SendDlgItemMessage(hwndDlg, (IDC_SENTRY_COMBO as i32), CB_SETCURSEL, (*pGlobalData).ssSoundSentry.iWindowsEffect as WPARAM, 0);

    /* Initialize SoundSentry settings */
    if ((*pGlobalData).ssSoundSentry.dwFlags & SSF_AVAILABLE) == 0 {
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SENTRY_BOX as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SENTRY_TEXT as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SENTRY_COMBO as i32)), (FALSE) as BOOL);
    } else {
        if ((*pGlobalData).ssSoundSentry.dwFlags & SSF_SOUNDSENTRYON) != 0 {
            CheckDlgButton(hwndDlg, (IDC_SENTRY_BOX as i32), BST_CHECKED);
        } else {
            EnableWindow(GetDlgItem(hwndDlg, (IDC_SENTRY_TEXT as i32)), (FALSE) as BOOL);
            EnableWindow(GetDlgItem(hwndDlg, (IDC_SENTRY_COMBO as i32)), (FALSE) as BOOL);
        }
    }

    /* Initialize ShowSounds settings */
    if (*pGlobalData).bShowSounds != 0 {
        CheckDlgButton(hwndDlg, (IDC_SSHOW_BOX as i32), BST_CHECKED);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SoundPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBAL_DATA;

            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            OnInitDialog(hwndDlg, pGlobalData);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_SENTRY_BOX => {
                    (*pGlobalData).ssSoundSentry.dwFlags ^= SSF_SOUNDSENTRYON as DWORD;
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_SENTRY_TEXT as i32)),
                        if ((*pGlobalData).ssSoundSentry.dwFlags & (SSF_SOUNDSENTRYON as DWORD)) != 0 {
                            (TRUE as UINT) as i32
                        } else {
                            (FALSE as UINT) as i32
                        },
                    );
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_SENTRY_COMBO as i32)),
                        if ((*pGlobalData).ssSoundSentry.dwFlags & (SSF_SOUNDSENTRYON as DWORD)) != 0 {
                            TRUE as BOOL
                        } else {
                            FALSE as BOOL
                        },
                    );
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                IDC_SENTRY_COMBO => {
                    if (HIWORD!(wParam)) as u32 == CBN_SELENDOK {
                        (*pGlobalData).ssSoundSentry.iWindowsEffect =
                            SendMessage(
                                lParam as HWND,
                                CB_GETCURSEL,
                                0,
                                0,
                            ) as DWORD;
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_SSHOW_BOX => {
                    (*pGlobalData).bShowSounds = ((*pGlobalData).bShowSounds == 0) as BOOL;
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_APPLY => {
                    SystemParametersInfo(
                        SPI_SETSOUNDSENTRY,
                        core::mem::size_of::<SOUNDSENTRY>() as UINT,
                        core::ptr::addr_of_mut!((*pGlobalData).ssSoundSentry) as PVOID,
                        (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT,
                    );
                    SystemParametersInfo(
                        SPI_SETSHOWSOUNDS,
                        (*pGlobalData).bShowSounds as UINT,
                        core::ptr::null_mut(),
                        (SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) as UINT,
                    );
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

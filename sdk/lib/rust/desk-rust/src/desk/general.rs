// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn InitFontSizeList(hWnd: HWND) {
    let mut hInf: *mut core::ffi::c_void;
    let mut hKey: HKEY;
    let mut hFontSize: HWND;
    let mut Context: core::mem::MaybeUninit<[u8; 512]> = core::mem::MaybeUninit::uninit();
    let mut i: i32;
    let mut ci: i32 = 0;
    let mut dwSize: u32;
    let mut dwValue: u32;
    let mut dwType: u32;

    hFontSize = GetDlgItem(hWnd, IDC_FONTSIZE_COMBO);

    hInf = SetupOpenInfFile(w!("font.inf"), core::ptr::null_mut(),
                            INF_STYLE_WIN4, core::ptr::null_mut());

    if hInf != INVALID_HANDLE_VALUE {
        if SetupFindFirstLine(hInf, w!("Font Sizes"), core::ptr::null_mut(), Context.as_mut_ptr() as *mut _) != 0 {
            if RegOpenKeyEx(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Hardware Profiles\\Current\\Software\\Fonts"),
                             0, KEY_READ, &mut hKey) == ERROR_SUCCESS {
                dwSize = MAX_PATH;
                dwType = REG_DWORD;

                if RegQueryValueEx(hKey, w!("LogPixels"), core::ptr::null_mut(), &mut dwType,
                                    (&mut dwValue as *mut u32) as LPBYTE, &mut dwSize) != ERROR_SUCCESS {
                    dwValue = 0;
                }

                RegCloseKey(hKey);
            }

            loop {
                let mut Buffer: [TCHAR; LINE_LEN] = core::mem::zeroed();
                let mut Desc: [TCHAR; LINE_LEN] = core::mem::zeroed();

                if SetupGetStringField(Context.as_mut_ptr() as *mut _, 0, Buffer.as_mut_ptr(), Buffer.len() / core::mem::size_of::<TCHAR>(), core::ptr::null_mut()) != 0 &&
                    SetupGetIntField(Context.as_mut_ptr() as *mut _, 1, &mut ci) != 0 {
                    _stprintf(Desc.as_mut_ptr(), w!("%s (%d DPI)"), Buffer.as_ptr(), ci);
                    i = SendMessage(hFontSize, CB_ADDSTRING, 0, Desc.as_ptr() as LPARAM) as i32;
                    if i != CB_ERR {
                        SendMessage(hFontSize, CB_SETITEMDATA, i as WPARAM, ci as LPARAM);
                    }

                    if dwValue as i32 == ci {
                        SendMessage(hFontSize, CB_SETCURSEL, i as WPARAM, 0);
                        SetWindowText(GetDlgItem(hWnd, IDC_FONTSIZE_CUSTOM), Desc.as_ptr());
                    } else {
                        SendMessage(hFontSize, CB_SETCURSEL, 0, 0);
                    }
                }

                if SetupFindNextLine(Context.as_mut_ptr() as *mut _, Context.as_mut_ptr() as *mut _) == 0 {
                    break;
                }
            }
        }
    }

    SetupCloseInfFile(hInf);
}
unsafe extern "C" fn InitRadioButtons(hWnd: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();

    if RegOpenKeyEx(HKEY_CURRENT_USER,
                     w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Controls Folder\\Display"),
                     0, KEY_READ, &mut hKey) == ERROR_SUCCESS
    {
        let mut szBuf: [TCHAR; 64] = [0; 64];
        let mut dwSize: u32 = 64;

        if RegQueryValueEx(hKey, w!("DynaSettingsChange"), 0, core::ptr::null_mut(),
                            szBuf.as_mut_ptr() as LPBYTE, &mut dwSize) == ERROR_SUCCESS
        {
            match _ttoi(szBuf.as_ptr())
            {
                0 => {
                    SendDlgItemMessage(hWnd, IDC_RESTART_RB, BM_SETCHECK, 1, 1);
                }
                1 => {
                    SendDlgItemMessage(hWnd, IDC_WITHOUTREBOOT_RB, BM_SETCHECK, 1, 1);
                }
                3 => {
                    SendDlgItemMessage(hWnd, IDC_ASKME_RB, BM_SETCHECK, 1, 1);
                }
                _ => {}
            }
        }
        else
        {
            SendDlgItemMessage(hWnd, IDC_WITHOUTREBOOT_RB, BM_SETCHECK, 1, 1);
        }

        RegCloseKey(hKey);
    }
    else
    {
        SendDlgItemMessage(hWnd, IDC_WITHOUTREBOOT_RB, BM_SETCHECK, 1, 1);
    }
}
#[no_mangle]
pub unsafe extern "system" fn AdvGeneralPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut DispDevice: PDISPLAY_DEVICE_ENTRY = core::ptr::null_mut();
    let mut Ret: INT_PTR = 0;

    if uMsg != WM_INITDIALOG {
        DispDevice = GetWindowLongPtr(hwndDlg, DWLP_USER) as PDISPLAY_DEVICE_ENTRY;
    }

    match uMsg {
        WM_INITDIALOG => {
            DispDevice = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PDISPLAY_DEVICE_ENTRY;
            SetWindowLongPtr(hwndDlg, DWLP_USER, DispDevice as isize);

            InitFontSizeList(hwndDlg);
            InitRadioButtons(hwndDlg);

            Ret = TRUE;
        }
        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDC_FONTSIZE_COMBO => {
                    if HIWORD!(wParam) == CBN_SELCHANGE {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                IDC_RESTART_RB | IDC_WITHOUTREBOOT_RB | IDC_ASKME_RB => {
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/general.c
//  * PURPOSE:         Advanced General settings
//  */
//
// #include "desk.h"
//
// static VOID
// InitFontSizeList(HWND hWnd)
// {
//     HINF hInf;
//     HKEY hKey;
//     HWND hFontSize;
//     INFCONTEXT Context;
//     int i, ci = 0;
//     DWORD dwSize, dwValue, dwType;
//
//     hFontSize = GetDlgItem(hWnd, IDC_FONTSIZE_COMBO);
//
//     hInf = SetupOpenInfFile(_T("font.inf"), NULL,
//                             INF_STYLE_WIN4, NULL);
//
//     if (hInf != INVALID_HANDLE_VALUE)
//     {
//         if (SetupFindFirstLine(hInf, _T("Font Sizes"), NULL, &Context))
//         {
//             if (RegOpenKeyEx(HKEY_LOCAL_MACHINE, _T("SYSTEM\\CurrentControlSet\\Hardware Profiles\\Current\\Software\\Fonts"),
//                              0, KEY_READ, &hKey) == ERROR_SUCCESS)
//             {
//                 dwSize = MAX_PATH;
//                 dwType = REG_DWORD;
//
//                 if (RegQueryValueEx(hKey, _T("LogPixels"), NULL, &dwType,
//                                     (LPBYTE)&dwValue, &dwSize) != ERROR_SUCCESS)
//                 {
//                     dwValue = 0;
//                 }
//
//                 RegCloseKey(hKey);
//             }
//
//             for (;;)
//             {
//                 TCHAR Buffer[LINE_LEN];
//                 TCHAR Desc[LINE_LEN];
//
//                 if (SetupGetStringField(&Context, 0, Buffer, sizeof(Buffer) / sizeof(TCHAR), NULL) &&
//                     SetupGetIntField(&Context, 1, &ci))
//                 {
//                     _stprintf(Desc, _T("%s (%d DPI)"), Buffer, ci);
//                     i = SendMessage(hFontSize, CB_ADDSTRING, 0, (LPARAM)Desc);
//                     if (i != CB_ERR)
//                         SendMessage(hFontSize, CB_SETITEMDATA, (WPARAM)i, (LPARAM)ci);
//
//                     if ((int)dwValue == ci)
//                     {
//                         SendMessage(hFontSize, CB_SETCURSEL, (WPARAM)i, 0);
//                         SetWindowText(GetDlgItem(hWnd, IDC_FONTSIZE_CUSTOM), Desc);
//                     }
//                     else
//                         SendMessage(hFontSize, CB_SETCURSEL, 0, 0);
//                 }
//
//                 if (!SetupFindNextLine(&Context, &Context))
//                 {
//                     break;
//                 }
//             }
//         }
//     }
//
//     SetupCloseInfFile(hInf);
// }
//
// static VOID
// InitRadioButtons(HWND hWnd)
// {
//     HKEY hKey;
//
//     if (RegOpenKeyEx(HKEY_CURRENT_USER,
//                      _T("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Controls Folder\\Display"),
//                      0, KEY_READ, &hKey) == ERROR_SUCCESS)
//     {
//         TCHAR szBuf[64];
//         DWORD dwSize = 64;
//
//         if (RegQueryValueEx(hKey, _T("DynaSettingsChange"), 0, NULL,
//                             (LPBYTE)szBuf, &dwSize) == ERROR_SUCCESS)
//         {
//             switch (_ttoi(szBuf))
//             {
//                 case 0:
//                     SendDlgItemMessage(hWnd, IDC_RESTART_RB, BM_SETCHECK, 1, 1);
//                     break;
//                 case 1:
//                     SendDlgItemMessage(hWnd, IDC_WITHOUTREBOOT_RB, BM_SETCHECK, 1, 1);
//                     break;
//                 case 3:
//                     SendDlgItemMessage(hWnd, IDC_ASKME_RB, BM_SETCHECK, 1, 1);
//                     break;
//             }
//         }
//         else
//             SendDlgItemMessage(hWnd, IDC_WITHOUTREBOOT_RB, BM_SETCHECK, 1, 1);
//
//         RegCloseKey(hKey);
//     }
//     else
//         SendDlgItemMessage(hWnd, IDC_WITHOUTREBOOT_RB, BM_SETCHECK, 1, 1);
// }
//
// INT_PTR __stdcall
// AdvGeneralPageProc(HWND hwndDlg,
//                    UINT uMsg,
//                    WPARAM wParam,
//                    LPARAM lParam)
// {
//     PDISPLAY_DEVICE_ENTRY DispDevice = NULL;
//     INT_PTR Ret = 0;
//
//     if (uMsg != WM_INITDIALOG)
//         DispDevice = (PDISPLAY_DEVICE_ENTRY)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             DispDevice = (PDISPLAY_DEVICE_ENTRY)(((LPPROPSHEETPAGE)lParam)->lParam);
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)DispDevice);
//
//             InitFontSizeList(hwndDlg);
//             InitRadioButtons(hwndDlg);
//
//             Ret = TRUE;
//             break;
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_FONTSIZE_COMBO:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//                 case IDC_RESTART_RB:
//                 case IDC_WITHOUTREBOOT_RB:
//                 case IDC_ASKME_RB:
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                 break;
//             }
//             break;
//     }
//
//     return Ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

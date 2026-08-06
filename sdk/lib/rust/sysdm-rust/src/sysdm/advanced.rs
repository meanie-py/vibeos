// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut BugLink: [TCHAR; 27] = [
    b'h' as TCHAR, b't' as TCHAR, b't' as TCHAR, b'p' as TCHAR, b':' as TCHAR,
    b'/' as TCHAR, b'/' as TCHAR, b'j' as TCHAR, b'i' as TCHAR, b'r' as TCHAR,
    b'a' as TCHAR, b'.' as TCHAR, b'r' as TCHAR, b'e' as TCHAR, b'a' as TCHAR,
    b'c' as TCHAR, b't' as TCHAR, b'o' as TCHAR, b's' as TCHAR, b'.' as TCHAR,
    b'o' as TCHAR, b'r' as TCHAR, b'g' as TCHAR, b'/' as TCHAR, b'\0' as TCHAR,
    0, 0
];

pub static mut ReportAsWorkstationKey: [TCHAR; 62] = [
    b'S' as TCHAR, b'Y' as TCHAR, b'S' as TCHAR, b'T' as TCHAR, b'E' as TCHAR,
    b'M' as TCHAR, b'\\' as TCHAR, b'C' as TCHAR, b'u' as TCHAR, b'r' as TCHAR,
    b'r' as TCHAR, b'e' as TCHAR, b'n' as TCHAR, b't' as TCHAR, b'C' as TCHAR,
    b'o' as TCHAR, b'n' as TCHAR, b't' as TCHAR, b'r' as TCHAR, b'o' as TCHAR,
    b'l' as TCHAR, b'S' as TCHAR, b'e' as TCHAR, b't' as TCHAR, b'\\' as TCHAR,
    b'C' as TCHAR, b'o' as TCHAR, b'n' as TCHAR, b't' as TCHAR, b'r' as TCHAR,
    b'o' as TCHAR, b'l' as TCHAR, b'\\' as TCHAR, b'R' as TCHAR, b'e' as TCHAR,
    b'a' as TCHAR, b'c' as TCHAR, b't' as TCHAR, b'O' as TCHAR, b'S' as TCHAR,
    b'\\' as TCHAR, b'S' as TCHAR, b'e' as TCHAR, b't' as TCHAR, b't' as TCHAR,
    b'i' as TCHAR, b'n' as TCHAR, b'g' as TCHAR, b's' as TCHAR, b'\\' as TCHAR,
    b'V' as TCHAR, b'e' as TCHAR, b'r' as TCHAR, b's' as TCHAR, b'i' as TCHAR,
    b'o' as TCHAR, b'n' as TCHAR, b'\0' as TCHAR, 0, 0, 0, 0, 0
];
unsafe extern "C" fn OnOK(hwndDlg: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut ReportAsWorkstation: u32;

    ReportAsWorkstation = if SendDlgItemMessageW(hwndDlg,
                                               (IDC_REPORTASWORKSTATION) as i32,
                                               BM_GETCHECK,
                                               0,
                                               0) == BST_CHECKED { 1 } else { 0 };

    if RegCreateKeyEx(HKEY_LOCAL_MACHINE,
                       ReportAsWorkstationKey.as_ptr(),
                       0,
                       core::ptr::null_mut(),
                       0,
                       KEY_WRITE,
                       core::ptr::null_mut(),
                       &mut hKey,
                       core::ptr::null_mut()) == (ERROR_SUCCESS) as i32
    {
        RegSetValueEx(hKey,
                      _T("ReportAsWorkstation"),
                      0,
                      REG_DWORD,
                      (&ReportAsWorkstation as *const u32) as LPBYTE,
                      core::mem::size_of::<u32>() as u32);

        RegCloseKey(hKey);
    }
}
unsafe extern "C" fn OnInitSysSettingsDialog(hwndDlg: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwVal: u32 = 0;
    let mut dwType: u32 = REG_DWORD;
    let cbData: u32 = core::mem::size_of::<u32>() as u32;
    let mut ReportAsWorkstation: u32 = if (*SharedUserData).NtProductType == NtProductWinNt { 1 } else { 0 };

    if RegOpenKeyEx(HKEY_LOCAL_MACHINE,
                     ReportAsWorkstationKey.as_ptr(),
                     0,
                     KEY_READ,
                     &mut hKey) == (ERROR_SUCCESS) as i32
    {
        let mut cbData_local = cbData;
        if RegQueryValueEx(hKey,
                            _T((b"ReportAsWorkstation\0".as_ptr() as *const i8).cast()),
                            0,
                            &mut dwType,
                            &mut dwVal as *mut u32 as LPBYTE,
                            &mut cbData_local) == (ERROR_SUCCESS) as i32
        {
            if dwType == REG_DWORD && cbData_local == core::mem::size_of::<u32>() as u32 {
                ReportAsWorkstation = if dwVal != (FALSE) as u32 { 1 } else { 0 };
            }
        }

        RegCloseKey(hKey);
    }
    SendDlgItemMessageW(hwndDlg, (IDC_REPORTASWORKSTATION) as i32, BM_SETCHECK,
                        if ReportAsWorkstation != 0 { BST_CHECKED } else { BST_UNCHECKED }, 0);
}

#[no_mangle]
pub unsafe extern "system" fn SysSettingsDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            OnInitSysSettingsDialog(hwndDlg);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    OnOK(hwndDlg);
                    EndDialog(hwndDlg, 0);
                    return TRUE;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, 0);
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn AdvancedPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {}

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_PERFOR => {
                    ShowPerformanceOptions(hwndDlg);
                }

                IDC_USERPROFILE => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_USERPROFILE),
                        hwndDlg,
                        UserProfileDlgProc,
                    );
                }

                IDC_STAREC => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_STARTUPRECOVERY),
                        hwndDlg,
                        StartRecDlgProc,
                    );
                }

                IDC_SYSSETTINGS => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_SYSSETTINGS),
                        hwndDlg,
                        SysSettingsDlgProc,
                    );
                }

                IDC_ENVVAR => {
                    DialogBox!(
                        hApplet,
                        MAKEINTRESOURCE!(IDD_ENVIRONMENT_VARIABLES),
                        hwndDlg,
                        EnvironmentDlgProc,
                    );
                }

                IDC_ERRORREPORT => {
                    ShellExecute(
                        core::ptr::null_mut(),
                        _T("open"),
                        BugLink.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        SW_SHOWNORMAL,
                    );
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/advanced.c
//  * PURPOSE:     Memory, start-up and profiles settings
//  * COPYRIGHT:   Copyright Thomas Weidenmueller <w3seek@reactos.org>
//                 Copyright 2006 - 2009 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "precomp.h"
// #define NTOS_MODE_USER
// #include <ndk/pstypes.h> /* For SharedUserData */
//
// static TCHAR BugLink[] = _T("http://jira.reactos.org/");
// static TCHAR ReportAsWorkstationKey[] = _T("SYSTEM\\CurrentControlSet\\Control\\ReactOS\\Settings\\Version");
//
//
// static VOID
// OnOK(HWND hwndDlg)
// {
//     HKEY hKey;
//     DWORD ReportAsWorkstation;
//
//     ReportAsWorkstation = (SendDlgItemMessageW(hwndDlg,
//                                                IDC_REPORTASWORKSTATION,
//                                                BM_GETCHECK,
//                                                0,
//                                                0) == BST_CHECKED);
//
//     if (RegCreateKeyEx(HKEY_LOCAL_MACHINE,
//                        ReportAsWorkstationKey,
//                        0,
//                        NULL,
//                        0,
//                        KEY_WRITE,
//                        NULL,
//                        &hKey,
//                        NULL) == ERROR_SUCCESS)
//     {
//         RegSetValueEx(hKey,
//                       _T("ReportAsWorkstation"),
//                       0,
//                       REG_DWORD,
//                       (LPBYTE)&ReportAsWorkstation,
//                       sizeof(DWORD));
//
//         RegCloseKey(hKey);
//     }
// }
//
// static VOID
// OnInitSysSettingsDialog(HWND hwndDlg)
// {
//     HKEY hKey;
//     DWORD dwVal = 0;
//     DWORD dwType = REG_DWORD;
//     DWORD cbData = sizeof(DWORD);
//     BOOL ReportAsWorkstation = SharedUserData->NtProductType == NtProductWinNt;
//
//     if (RegOpenKeyEx(HKEY_LOCAL_MACHINE,
//                      ReportAsWorkstationKey,
//                      0,
//                      KEY_READ,
//                      &hKey) == ERROR_SUCCESS)
//     {
//         if (RegQueryValueEx(hKey,
//                             _T("ReportAsWorkstation"),
//                             0,
//                             &dwType,
//                             (LPBYTE)&dwVal,
//                             &cbData) == ERROR_SUCCESS)
//         {
//             if (dwType == REG_DWORD && cbData == sizeof(DWORD))
//                 ReportAsWorkstation = dwVal != FALSE;
//         }
//
//         RegCloseKey(hKey);
//     }
//     SendDlgItemMessageW(hwndDlg, IDC_REPORTASWORKSTATION, BM_SETCHECK,
//                         ReportAsWorkstation ? BST_CHECKED : BST_UNCHECKED, 0);
// }
//
// INT_PTR __stdcall
// SysSettingsDlgProc(HWND hwndDlg,
//                    UINT uMsg,
//                    WPARAM wParam,
//                    LPARAM lParam)
// {
//     UNREFERENCED_PARAMETER(lParam);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             OnInitSysSettingsDialog(hwndDlg);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDOK:
//                     OnOK(hwndDlg);
//                     EndDialog(hwndDlg, 0);
//                     return TRUE;
//
//                 case IDCANCEL:
//                     EndDialog(hwndDlg, 0);
//                     return TRUE;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// AdvancedPageProc(HWND hwndDlg,
//                  UINT uMsg,
//                  WPARAM wParam,
//                  LPARAM lParam)
// {
//     UNREFERENCED_PARAMETER(lParam);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             break;
//
//         case WM_COMMAND:
//         {
//             switch (LOWORD(wParam))
//             {
//                 case IDC_PERFOR:
//                     ShowPerformanceOptions(hwndDlg);
//                     break;
//
//                 case IDC_USERPROFILE:
//                     DialogBox(hApplet,
//                               MAKEINTRESOURCE(IDD_USERPROFILE),
//                               hwndDlg,
//                               UserProfileDlgProc);
//                     break;
//
//                 case IDC_STAREC:
//                     DialogBox(hApplet,
//                               MAKEINTRESOURCE(IDD_STARTUPRECOVERY),
//                               hwndDlg,
//                               StartRecDlgProc);
//                     break;
//
//                 case IDC_SYSSETTINGS:
//                     DialogBox(hApplet,
//                               MAKEINTRESOURCE(IDD_SYSSETTINGS),
//                               hwndDlg,
//                               SysSettingsDlgProc);
//                     break;
//
//                 case IDC_ENVVAR:
//                     DialogBox(hApplet,
//                               MAKEINTRESOURCE(IDD_ENVIRONMENT_VARIABLES),
//                               hwndDlg,
//                               EnvironmentDlgProc);
//                     break;
//
//                 case IDC_ERRORREPORT:
//                     ShellExecute(NULL,
//                                  _T("open"),
//                                  BugLink,
//                                  NULL,
//                                  NULL,
//                                  SW_SHOWNORMAL);
//                     break;
//             }
//         }
//
//         break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

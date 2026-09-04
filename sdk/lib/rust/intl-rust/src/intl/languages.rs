// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetJapaneseUninstallCmdLine(hwnd: HWND, pszCmdLine: LPWSTR, cchCmdLine: usize) -> i32 {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: i32;
    let mut dwSize: u32;

    *pszCmdLine.offset(0) = UNICODE_NULL;

    error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                          w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{80F03D6E-0549-4202-BE81-FF583F56A7A8}_is1"),
                          0,
                          KEY_READ,
                          &mut hKey);
    if error != (ERROR_SUCCESS) as i32 {
        return (FALSE) as i32;
    }

    dwSize = (cchCmdLine * core::mem::size_of::<u16>()) as u32;
    error = RegQueryValueExW(hKey, w!("UninstallString"), core::ptr::null_mut(), core::ptr::null_mut(), pszCmdLine as LPBYTE, &mut dwSize);
    if error != (ERROR_SUCCESS) as i32 {
        RegCloseKey(hKey);
        return (FALSE) as i32;
    }

    *pszCmdLine.offset((cchCmdLine - 1) as isize) = UNICODE_NULL;
    RegCloseKey(hKey);
    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn HasJapanesePackage(hwnd: HWND) -> i32 {
    let mut szPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    GetJapaneseUninstallCmdLine(hwnd, szPath.as_mut_ptr(), (_countof!(szPath)) as usize)
}
#[no_mangle]
pub unsafe extern "system" fn LanguagesPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut shInputDll: SHELLEXECUTEINFOW = core::mem::zeroed();
    let mut pGlobalData: PGLOBALDATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBALDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            if (*pGlobalData).bIsUserAdmin == 0 {
                EnableWindow(GetDlgItem(hwndDlg, (IDC_INST_FILES_FOR_RTOL_LANG) as i32), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_INST_FILES_FOR_ASIAN) as i32), (FALSE) as BOOL);
            }

            match (PRIMARYLANGID!(GetUserDefaultLangID())) as u32 {
                LANG_JAPANESE => {
                    if HasJapanesePackage(hwndDlg) != 0 {
                        CheckDlgButton(hwndDlg, (IDC_INST_FILES_FOR_ASIAN) as i32, BST_CHECKED);
                    }
                }
                LANG_CHINESE | LANG_KOREAN | _ => {
                    EnableWindow(GetDlgItem(hwndDlg, (IDC_INST_FILES_FOR_ASIAN) as i32), (FALSE) as BOOL);
                }
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_DETAIL_BUTTON => {
                    if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                        memset(
                            core::ptr::addr_of_mut!(shInputDll) as *mut core::ffi::c_void,
                            0x0,
                            core::mem::size_of::<SHELLEXECUTEINFOW>(),
                        );
                        shInputDll.cbSize = core::mem::size_of_val(&shInputDll) as u32;
                        shInputDll.hwnd = hwndDlg;
                        shInputDll.lpVerb = w!("open");
                        shInputDll.lpFile = w!("RunDll32.exe");
                        shInputDll.lpParameters = w!("shell32.dll,Control_RunDLL input.dll");
                        if ShellExecuteExW(core::ptr::addr_of_mut!(shInputDll)) == 0 {
                            PrintErrorMsgBox(IDS_ERROR_INPUT_DLL);
                        }
                    }
                }

                IDC_INST_FILES_FOR_ASIAN => {
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).code == PSN_APPLY as u32 {
                match (PRIMARYLANGID!(GetUserDefaultLangID())) as u32 {
                    LANG_JAPANESE => {
                        if IsDlgButtonChecked(hwndDlg, (IDC_INST_FILES_FOR_ASIAN) as i32) == BST_CHECKED {
                            if HasJapanesePackage(hwndDlg) == 0 {
                                ShellExecuteW(
                                    hwndDlg,
                                    core::ptr::null_mut(),
                                    w!("rapps.exe"),
                                    w!("/INSTALL jpn-package"),
                                    core::ptr::null_mut(),
                                    (SW_SHOWNORMAL) as INT,
                                );
                            }
                        } else {
                            let mut szUninstall: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
                            if GetJapaneseUninstallCmdLine(
                                hwndDlg,
                                szUninstall.as_mut_ptr(),
                                _countof!(szUninstall) as usize,
                            ) != 0
                            {
                                let mut pchArgs: PWCHAR = PathGetArgsW(szUninstall.as_mut_ptr());
                                if !pchArgs.is_null() && *pchArgs != 0 {
                                    pchArgs = pchArgs.offset(-1);
                                    *pchArgs = UNICODE_NULL;
                                    pchArgs = pchArgs.offset(1);
                                }
                                PathUnquoteSpacesW(szUninstall.as_mut_ptr());

                                ShellExecuteW(
                                    hwndDlg,
                                    core::ptr::null_mut(),
                                    szUninstall.as_ptr(),
                                    pchArgs,
                                    core::ptr::null_mut(),
                                    (SW_SHOWNORMAL) as INT,
                                );
                            }
                        }
                    }

                    LANG_CHINESE | LANG_KOREAN | _ => {}
                }

                PropSheet_UnChanged!(GetParent(hwndDlg), hwndDlg);
            }
        }

        _ => {}
    }

    FALSE as INT_PTR
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "intl.h"
//
// #include <shellapi.h>
// #include <shlobj.h>
// #include <shlwapi.h>
// #include <strsafe.h>
//
// /* What is the uninstallation command line of "ReactOS JPN Package"? */
// BOOL GetJapaneseUninstallCmdLine(HWND hwnd, LPWSTR pszCmdLine, SIZE_T cchCmdLine)
// {
//     HKEY hKey;
//     LONG error;
//     DWORD dwSize;
//
//     pszCmdLine[0] = UNICODE_NULL;
//
//     error = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                           L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\"
//                           L"{80F03D6E-0549-4202-BE81-FF583F56A7A8}_is1",
//                           0,
//                           KEY_READ,
//                           &hKey);
//     if (error != ERROR_SUCCESS)
//         return FALSE;
//
//     dwSize = cchCmdLine * sizeof(WCHAR);
//     error = RegQueryValueExW(hKey, L"UninstallString", NULL, NULL, (LPBYTE)pszCmdLine, &dwSize);
//     if (error != ERROR_SUCCESS)
//     {
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     pszCmdLine[cchCmdLine - 1] = UNICODE_NULL;
//     RegCloseKey(hKey);
//     return TRUE;
// }
//
// /* Is there any installed "ReactOS JPN Package"? */
// BOOL HasJapanesePackage(HWND hwnd)
// {
//     WCHAR szPath[MAX_PATH];
//     return GetJapaneseUninstallCmdLine(hwnd, szPath, _countof(szPath));
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// LanguagesPageProc(HWND hwndDlg,
//                   UINT uMsg,
//                   WPARAM wParam,
//                   LPARAM lParam)
// {
//     SHELLEXECUTEINFOW shInputDll;
//     PGLOBALDATA pGlobalData;
//
//     pGlobalData = (PGLOBALDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             pGlobalData = (PGLOBALDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pGlobalData);
//
//             if (!pGlobalData->bIsUserAdmin)
//             {
//                 EnableWindow(GetDlgItem(hwndDlg, IDC_INST_FILES_FOR_RTOL_LANG), FALSE);
//                 EnableWindow(GetDlgItem(hwndDlg, IDC_INST_FILES_FOR_ASIAN), FALSE);
//             }
//
//             /* EAST ASIAN specific */
//             switch (PRIMARYLANGID(GetUserDefaultLangID()))
//             {
//                 case LANG_JAPANESE:
//                     if (HasJapanesePackage(hwndDlg))
//                     {
//                         CheckDlgButton(hwndDlg, IDC_INST_FILES_FOR_ASIAN, BST_CHECKED);
//                     }
//                     break;
//
//                 case LANG_CHINESE: /* Not supported yet */
//                 case LANG_KOREAN: /* Not supported yet */
//                 default:
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_INST_FILES_FOR_ASIAN), FALSE);
//                     break;
//             }
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 /* If "detail" button pressed */
//                 case IDC_DETAIL_BUTTON:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         memset(&shInputDll, 0x0, sizeof(SHELLEXECUTEINFOW));
//                         shInputDll.cbSize = sizeof(shInputDll);
//                         shInputDll.hwnd = hwndDlg;
//                         shInputDll.lpVerb = L"open";
//                         shInputDll.lpFile = L"RunDll32.exe";
//                         shInputDll.lpParameters = L"shell32.dll,Control_RunDLL input.dll";
//                         if (ShellExecuteExW(&shInputDll) == 0)
//                         {
//                             PrintErrorMsgBox(IDS_ERROR_INPUT_DLL);
//                         }
//                     }
//                     break;
//
//                 case IDC_INST_FILES_FOR_ASIAN:
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//             }
//             break;
//
//         case WM_NOTIFY:
//             if (((LPNMHDR)lParam)->code == (UINT)PSN_APPLY) /* Apply changes */
//             {
//                 /* EAST ASIAN specific */
//                 switch (PRIMARYLANGID(GetUserDefaultLangID()))
//                 {
//                     case LANG_JAPANESE:
//                         if (IsDlgButtonChecked(hwndDlg, IDC_INST_FILES_FOR_ASIAN) == BST_CHECKED)
//                         {
//                             if (!HasJapanesePackage(hwndDlg))
//                             {
//                                 /* Install now */
//                                 ShellExecuteW(hwndDlg, NULL, L"rapps.exe", L"/INSTALL jpn-package",
//                                               NULL, SW_SHOWNORMAL);
//                             }
//                         }
//                         else
//                         {
//                             WCHAR szUninstall[MAX_PATH];
//                             if (GetJapaneseUninstallCmdLine(hwndDlg, szUninstall, _countof(szUninstall)))
//                             {
//                                 /* Go to arguments of command line */
//                                 PWCHAR pchArgs = PathGetArgsW(szUninstall);
//                                 if (pchArgs && *pchArgs)
//                                 {
//                                     --pchArgs;
//                                     /* pchArgs pointer is inside szUninstall,
//                                      * so we have to split both strings */
//                                     *pchArgs = UNICODE_NULL;
//                                     ++pchArgs;
//                                 }
//                                 PathUnquoteSpacesW(szUninstall);
//
//                                 /* Uninstall now */
//                                 ShellExecuteW(hwndDlg, NULL, szUninstall, pchArgs, NULL, SW_SHOWNORMAL);
//                             }
//                         }
//                         break;
//
//                     case LANG_CHINESE: /* Not supported yet */
//                     case LANG_KOREAN: /* Not supported yet */
//                     default:
//                         break;
//                 }
//
//                 PropSheet_UnChanged(GetParent(hwndDlg), hwndDlg);
//             }
//             break;
//     }
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

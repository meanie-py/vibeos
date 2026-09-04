// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hwndTimeEnum: HWND = core::ptr::null_mut();
unsafe extern "system" fn TimeFormatEnumProc(lpTimeFormatString: PWSTR) -> BOOL {
    SendMessageW(
        hwndTimeEnum,
        CB_ADDSTRING,
        0,
        lpTimeFormatString as LPARAM,
    );

    TRUE as BOOL
}

unsafe extern "C" fn UpdateTimeSample(hWnd: HWND, pGlobalData: PGLOBALDATA) {
    let mut szBuffer: [u16; (MAX_SAMPLES_STR_SIZE) as usize] = [0; (MAX_SAMPLES_STR_SIZE) as usize];

    GetTimeFormatW(
        (*pGlobalData).UserLCID,
        0,
        core::ptr::null_mut(),
        (*pGlobalData).szTimeFormat.as_ptr(),
        szBuffer.as_mut_ptr(),
        (MAX_SAMPLES_STR_SIZE) as i32,
    );
    SendDlgItemMessageW(
        hWnd,
        (IDC_TIMESAMPLE) as i32,
        WM_SETTEXT,
        0,
        szBuffer.as_ptr() as LPARAM,
    );
}
unsafe extern "C" fn InitTimeFormatCB(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA) {
    SendDlgItemMessageW(hwndDlg, (IDC_TIMEFORMAT) as i32,
                        CB_LIMITTEXT, (MAX_TIMEFORMAT) as WPARAM, 0);

    hwndTimeEnum = GetDlgItem(hwndDlg, (IDC_TIMEFORMAT) as i32);
    EnumTimeFormatsW(Some(TimeFormatEnumProc), (*pGlobalData).UserLCID, 0);

    SendDlgItemMessageW(hwndDlg, (IDC_TIMEFORMAT) as i32,
                        CB_SELECTSTRING,
                        -1i32 as WPARAM,
                        (*pGlobalData).szTimeFormat.as_ptr() as LPARAM);
}
unsafe extern "C" fn InitTimeSeparatorCB(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA) {
    SendDlgItemMessageW(hwndDlg, (IDC_TIMESEPARATOR) as i32,
                        CB_LIMITTEXT, (MAX_TIMESEPARATOR) as WPARAM, 0);

    SendDlgItemMessageW(hwndDlg, (IDC_TIMESEPARATOR) as i32,
                        CB_ADDSTRING,
                        0,
                        (*pGlobalData).szTimeSep.as_ptr() as LPARAM);

    SendDlgItemMessageW(hwndDlg, (IDC_TIMESEPARATOR) as i32,
                        CB_SETCURSEL,
                        0,
                        0);
}
unsafe extern "C" fn InitAmSymbol(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA) {
    let nLen: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_TIMEAMSYMBOL) as i32,
                        CB_LIMITTEXT, (MAX_TIMEAMSYMBOL) as WPARAM, 0);

    nLen = (wcslen((*pGlobalData).szTimeAM.as_ptr())) as i32;

    SendDlgItemMessageW(hwndDlg, (IDC_TIMEAMSYMBOL) as i32,
                        CB_ADDSTRING,
                        0,
                        (*pGlobalData).szTimeAM.as_ptr() as LPARAM);
    if nLen != 0
    {
        SendDlgItemMessageW(hwndDlg, (IDC_TIMEAMSYMBOL) as i32,
                            CB_ADDSTRING,
                            0,
                            w!("") as LPARAM);
    }

    SendDlgItemMessageW(hwndDlg, (IDC_TIMEAMSYMBOL) as i32,
                        CB_SETCURSEL,
                        0,
                        0);
}
unsafe extern "C" fn InitPmSymbol(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA) {
    let mut nLen: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_TIMEPMSYMBOL) as i32,
                        CB_LIMITTEXT, (MAX_TIMEPMSYMBOL) as WPARAM, 0);

    nLen = (wcslen((*pGlobalData).szTimeAM.as_ptr())) as i32;

    SendDlgItemMessageW(hwndDlg, (IDC_TIMEPMSYMBOL) as i32,
                        CB_ADDSTRING,
                        0,
                        (*pGlobalData).szTimePM.as_ptr() as LPARAM);
    if nLen != 0
    {
        SendDlgItemMessageW(hwndDlg, (IDC_TIMEPMSYMBOL) as i32,
                            CB_ADDSTRING,
                            0,
                            w!("") as LPARAM);
    }
    SendDlgItemMessageW(hwndDlg, (IDC_TIMEPMSYMBOL) as i32,
                        CB_SETCURSEL,
                        0,
                        0);
}
unsafe extern "C" fn GetTimeSetting(
    hwndDlg: HWND,
    pGlobalData: *mut GLOBALDATA,
) -> BOOL {
    let mut szTimeFormat: [u16; (MAX_TIMEFORMAT) as usize] = [0; (MAX_TIMEFORMAT) as usize];
    let mut szTimeSep: [u16; (MAX_TIMESEPARATOR) as usize] = [0; (MAX_TIMESEPARATOR) as usize];
    let mut szTimeAM: [u16; (MAX_TIMEAMSYMBOL) as usize] = [0; (MAX_TIMEAMSYMBOL) as usize];
    let mut szTimePM: [u16; (MAX_TIMEPMSYMBOL) as usize] = [0; (MAX_TIMEPMSYMBOL) as usize];

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_TIMEFORMAT) as INT,
                            szTimeFormat.as_mut_ptr(),
                            MAX_TIMEFORMAT);

    if szTimeFormat[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_TIMESEPARATOR) as INT,
                            szTimeSep.as_mut_ptr(),
                            MAX_TIMESEPARATOR);

    if szTimeSep[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_TIMEAMSYMBOL) as INT,
                            szTimeAM.as_mut_ptr(),
                            MAX_TIMEAMSYMBOL);

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_TIMEPMSYMBOL) as INT,
                            szTimePM.as_mut_ptr(),
                            MAX_TIMEPMSYMBOL);

    wcscpy((*pGlobalData).szTimeFormat.as_mut_ptr(), szTimeFormat.as_ptr());
    wcscpy((*pGlobalData).szTimeSep.as_mut_ptr(), szTimeSep.as_ptr());
    wcscpy((*pGlobalData).szTimeAM.as_mut_ptr(), szTimeAM.as_ptr());
    wcscpy((*pGlobalData).szTimePM.as_mut_ptr(), szTimePM.as_ptr());

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn TimePageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBALDATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as *const _PROPSHEETPAGEW)).lParam as PGLOBALDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            InitTimeFormatCB(hwndDlg, pGlobalData);

            InitTimeSeparatorCB(hwndDlg, pGlobalData);

            InitAmSymbol(hwndDlg, pGlobalData);

            InitPmSymbol(hwndDlg, pGlobalData);

            UpdateTimeSample(hwndDlg, pGlobalData);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_TIMEFORMAT | IDC_TIMESEPARATOR | IDC_TIMEAMSYMBOL | IDC_TIMEPMSYMBOL => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 || HIWORD!(wParam) == (CBN_EDITCHANGE) as u16 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).code == PSN_APPLY as u32 {
                if GetTimeSetting(hwndDlg, pGlobalData) != 0 {
                    (*pGlobalData).bUserLocaleChanged = (TRUE) as i32;
                    UpdateTimeSample(hwndDlg, pGlobalData);
                }
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
// /*
//  *  ReactOS
//  *  Copyright (C) 2004 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * PROJECT:         ReactOS International Control Panel
//  * FILE:            dll/cpl/intl/time.c
//  * PURPOSE:         Time property page
//  * PROGRAMMERS:     Eric Kohl
//  *                  Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "intl.h"
//
// static HWND hwndTimeEnum = NULL;
//
// static BOOL __stdcall
// TimeFormatEnumProc(PWSTR lpTimeFormatString)
// {
//     SendMessageW(hwndTimeEnum,
//                 CB_ADDSTRING,
//                 0,
//                 (LPARAM)lpTimeFormatString);
//
//     return TRUE;
// }
//
// static VOID
// UpdateTimeSample(HWND hWnd, PGLOBALDATA pGlobalData)
// {
//     WCHAR szBuffer[MAX_SAMPLES_STR_SIZE];
//
//     GetTimeFormatW(pGlobalData->UserLCID, 0, NULL,
//                    pGlobalData->szTimeFormat, szBuffer,
//                    MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hWnd, IDC_TIMESAMPLE, WM_SETTEXT, 0, (LPARAM)szBuffer);
// }
//
//
// static
// VOID
// InitTimeFormatCB(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     /* Get the time format */
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEFORMAT,
//                         CB_LIMITTEXT, MAX_TIMEFORMAT, 0);
//
//     /* Add available time formats to the list */
//     hwndTimeEnum = GetDlgItem(hwndDlg, IDC_TIMEFORMAT);
//     EnumTimeFormatsW(TimeFormatEnumProc, pGlobalData->UserLCID, 0);
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEFORMAT,
//                         CB_SELECTSTRING,
//                         -1,
//                         (LPARAM)pGlobalData->szTimeFormat);
// }
//
// static
// VOID
// InitTimeSeparatorCB(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     SendDlgItemMessageW(hwndDlg, IDC_TIMESEPARATOR,
//                         CB_LIMITTEXT, MAX_TIMESEPARATOR, 0);
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMESEPARATOR,
//                         CB_ADDSTRING,
//                         0,
//                         (LPARAM)pGlobalData->szTimeSep);
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMESEPARATOR,
//                         CB_SETCURSEL,
//                         0, /* Index */
//                         0);
// }
//
//
// static
// VOID
// InitAmSymbol(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     int nLen;
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEAMSYMBOL,
//                         CB_LIMITTEXT, MAX_TIMEAMSYMBOL, 0);
//
//     nLen = wcslen(pGlobalData->szTimeAM);
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEAMSYMBOL,
//                         CB_ADDSTRING,
//                         0,
//                         (LPARAM)pGlobalData->szTimeAM);
//     if (nLen != 0)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_TIMEAMSYMBOL,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)L"");
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEAMSYMBOL,
//                         CB_SETCURSEL,
//                         0, /* Index */
//                         0);
// }
//
//
// static
// VOID
// InitPmSymbol(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     int nLen;
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEPMSYMBOL,
//                         CB_LIMITTEXT, MAX_TIMEPMSYMBOL, 0);
//
//     nLen = wcslen(pGlobalData->szTimeAM);
//
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEPMSYMBOL,
//                         CB_ADDSTRING,
//                         0,
//                         (LPARAM)pGlobalData->szTimePM);
//     if (nLen != 0)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_TIMEPMSYMBOL,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)L"");
//     }
//     SendDlgItemMessageW(hwndDlg, IDC_TIMEPMSYMBOL,
//                         CB_SETCURSEL,
//                         0, /* Index */
//                         0);
// }
//
//
// static
// BOOL
// GetTimeSetting(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR szTimeFormat[MAX_TIMEFORMAT];
//     WCHAR szTimeSep[MAX_TIMESEPARATOR];
//     WCHAR szTimeAM[MAX_TIMEAMSYMBOL];
//     WCHAR szTimePM[MAX_TIMEPMSYMBOL];
//
//     /* Time format */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_TIMEFORMAT,
//                             szTimeFormat,
//                             MAX_TIMEFORMAT);
//
//     /* Check the time format */
//     if (szTimeFormat[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Time separator */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_TIMESEPARATOR,
//                             szTimeSep,
//                             MAX_TIMESEPARATOR);
//
//     /* Check the time separator */
//     if (szTimeSep[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* AM symbol */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_TIMEAMSYMBOL,
//                             szTimeAM,
//                             MAX_TIMEAMSYMBOL);
//
//     /* PM symbol */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_TIMEPMSYMBOL,
//                             szTimePM,
//                             MAX_TIMEPMSYMBOL);
//
//     /* Store settings in global data */
//     wcscpy(pGlobalData->szTimeFormat, szTimeFormat);
//     wcscpy(pGlobalData->szTimeSep, szTimeSep);
//     wcscpy(pGlobalData->szTimeAM, szTimeAM);
//     wcscpy(pGlobalData->szTimePM, szTimePM);
//
//     return TRUE;
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// TimePageProc(HWND hwndDlg,
//              UINT uMsg,
//              WPARAM wParam,
//              LPARAM lParam)
// {
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
//             /* Get the time format */
//             InitTimeFormatCB(hwndDlg, pGlobalData);
//
//             /* Get the time separator */
//             InitTimeSeparatorCB(hwndDlg, pGlobalData);
//
//             /* Get the AM symbol */
//             InitAmSymbol(hwndDlg, pGlobalData);
//
//             /* Get the PM symbol */
//             InitPmSymbol(hwndDlg, pGlobalData);
//
//             /* Update the time format sample */
//             UpdateTimeSample(hwndDlg, pGlobalData);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_TIMEFORMAT:
//                 case IDC_TIMESEPARATOR:
//                 case IDC_TIMEAMSYMBOL:
//                 case IDC_TIMEPMSYMBOL:
//                     if (HIWORD(wParam) == CBN_SELCHANGE ||
//                         HIWORD(wParam) == CBN_EDITCHANGE)
//                     {
//                         /* Enable the Apply button */
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//             }
//             break;
//
//         case WM_NOTIFY:
//             if (((LPNMHDR)lParam)->code == (UINT)PSN_APPLY)
//             {
//                 if (GetTimeSetting(hwndDlg, pGlobalData))
//                 {
//                     pGlobalData->bUserLocaleChanged = TRUE;
//                     UpdateTimeSample(hwndDlg, pGlobalData);
//                 }
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

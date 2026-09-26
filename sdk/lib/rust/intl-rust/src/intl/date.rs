// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const YEAR_STR_MAX_SIZE: usize = 5;

pub const MAX_SHRT_DATE_SEPARATORS: usize = 3;

pub static STD_DATE_SEP: &[u16] = &[0x002E, 0x0000]; // L"."

pub const YEAR_DIFF: i32 = 99;

pub const MAX_YEAR: i32 = 9999;

pub static mut hwndEnum: HWND = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "C" fn isDateCompAl(alpha: u16) -> BOOL {
    if (alpha == 'y' as u16) || (alpha == 'M' as u16) || (alpha == 'd' as u16) || (alpha == ' ' as u16) {
        return TRUE as BOOL;
    } else {
        return FALSE as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FindDateSep(szSourceStr: *const u16) -> LPTSTR {
    let mut pszFoundSep: PWSTR;
    let mut nDateCompCount: u32 = 0;
    let mut nDateSepCount: u32 = 0;

    pszFoundSep = HeapAlloc(GetProcessHeap(), 0, (MAX_SAMPLES_STR_SIZE * (core::mem::size_of::<u16>()) as u32) as SIZE_T) as LPWSTR;
    if pszFoundSep.is_null() {
        return core::ptr::null_mut();
    }

    wcscpy(pszFoundSep, STD_DATE_SEP.as_ptr());

    while nDateCompCount < wcslen(szSourceStr) as u32 {
        if isDateCompAl(*szSourceStr.offset(nDateCompCount as isize)) == 0 && (*szSourceStr.offset(nDateCompCount as isize) != 0x0027u16) {
            while isDateCompAl(*szSourceStr.offset(nDateCompCount as isize)) == 0 && (*szSourceStr.offset(nDateCompCount as isize) != 0x0027u16) {
                *pszFoundSep.offset(nDateSepCount as isize) = *szSourceStr.offset(nDateCompCount as isize);
                nDateSepCount += 1;
                nDateCompCount += 1;
            }

            *pszFoundSep.offset(nDateSepCount as isize) = 0x0000u16;
            return (pszFoundSep).cast();
        }

        nDateCompCount += 1;
    }

    (pszFoundSep).cast()
}
unsafe extern "C" fn SetShortDateSep(hwndDlg: HWND, pszShortDateSep: PWSTR) -> i32 {
    let mut nSepStrSize: i32;
    let mut nSepCount: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                        WM_GETTEXT,
                        MAX_SAMPLES_STR_SIZE as WPARAM,
                        pszShortDateSep as LPARAM);

    nSepStrSize = wcslen(pszShortDateSep) as i32;

    nSepCount = 0;
    while nSepCount < nSepStrSize {
        if iswalnum(*pszShortDateSep.offset(nSepCount as isize) as _) != 0 || (*pszShortDateSep.offset(nSepCount as isize) == 0x0027u16) {
            PrintErrorMsgBox(IDS_ERROR_SYMBOL_SEPARATE);
            return (FALSE) as i32;
        }
        nSepCount += 1;
    }

    if nSepStrSize == 0 {
        PrintErrorMsgBox(IDS_ERROR_SYMBOL_SEPARATE);
        return (FALSE) as i32;
    }

    return (TRUE) as i32;
}
unsafe extern "C" fn SetShortDateFormat(hwndDlg: HWND, pszShortDateFmt: PWSTR) -> u8 {
    let mut szShortDateSep: [u16; (MAX_SAMPLES_STR_SIZE) as usize] = [0; (MAX_SAMPLES_STR_SIZE) as usize];
    let mut szFoundDateSep: [u16; (MAX_SAMPLES_STR_SIZE) as usize] = [0; (MAX_SAMPLES_STR_SIZE) as usize];
    let mut pszResultStr: PWSTR;
    let mut pszFoundSep: LPTSTR;
    let mut OpenApostFlg: u8 = (FALSE) as BOOLEAN;
    let mut nFmtStrSize: i32;
    let mut nDateCompCount: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATEFMT_COMBO) as i32,
                        WM_GETTEXT,
                        MAX_SAMPLES_STR_SIZE as WPARAM,
                        pszShortDateFmt as LPARAM);

    SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                        WM_GETTEXT,
                        MAX_SAMPLES_STR_SIZE as WPARAM,
                        szShortDateSep.as_mut_ptr() as LPARAM);

    nFmtStrSize = (wcslen(pszShortDateFmt)) as i32;

    nDateCompCount = 0;
    while nDateCompCount < nFmtStrSize {
        if *pszShortDateFmt.offset(nDateCompCount as isize) == 0x0027 {
            OpenApostFlg = if OpenApostFlg != 0 { 0 } else { 1 };
        }

        if iswalnum((*pszShortDateFmt.offset(nDateCompCount as isize)) as i32) != 0 &&
            isDateCompAl(*pszShortDateFmt.offset(nDateCompCount as isize)) == 0 &&
            OpenApostFlg == 0
        {
            PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_SHORT);
            return (FALSE as BOOL) as BOOLEAN;
        }
        nDateCompCount += 1;
    }

    if OpenApostFlg != 0 || nFmtStrSize == 0 {
        PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_SHORT);
        return (FALSE as BOOL) as BOOLEAN;
    }

    pszFoundSep = FindDateSep(pszShortDateFmt);
    if pszFoundSep != core::ptr::null_mut() {
        wcscpy(szFoundDateSep.as_mut_ptr(), pszFoundSep);
        pszResultStr = ReplaceSubStr(pszShortDateFmt, szShortDateSep.as_mut_ptr(), szFoundDateSep.as_mut_ptr());
        if pszResultStr != core::ptr::null_mut() {
            wcscpy(pszShortDateFmt, pszResultStr);
            HeapFree(GetProcessHeap(), 0, (pszResultStr).cast());
        }

        HeapFree(GetProcessHeap(), 0, (pszFoundSep).cast());
    }

    return (TRUE as BOOL) as BOOLEAN;
}
unsafe extern "C" fn SetLongDateFormat(hwndDlg: HWND, pszLongDateFmt: PWSTR) -> u8 {
    let mut OpenApostFlg: u8 = (FALSE) as BOOLEAN;
    let nFmtStrSize: i32;
    let mut nDateCompCount: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_LONGDATEFMT_COMBO) as i32,
                        WM_GETTEXT,
                        MAX_SAMPLES_STR_SIZE as WPARAM,
                        pszLongDateFmt as LPARAM);

    nFmtStrSize = wcslen(pszLongDateFmt) as i32;

    nDateCompCount = 0;
    while nDateCompCount < nFmtStrSize {
        if *pszLongDateFmt.offset(nDateCompCount as isize) == 0x0027u16 {
            OpenApostFlg = if OpenApostFlg != 0 { 0 } else { 1 };
        }

        if iswalnum((*pszLongDateFmt.offset(nDateCompCount as isize) as u32) as i32) != 0 &&
            isDateCompAl(*pszLongDateFmt.offset(nDateCompCount as isize)) == 0 &&
            OpenApostFlg == 0
        {
            PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_LONG);
            return (FALSE as BOOL) as BOOLEAN;
        }
        nDateCompCount += 1;
    }

    if OpenApostFlg != 0 || nFmtStrSize == 0 {
        PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_LONG);
        return (FALSE as BOOL) as BOOLEAN;
    }

    return (TRUE as BOOL) as BOOLEAN;
}
unsafe extern "C" fn InitShortDateSepSamples(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let ShortDateSepSamples: [PWSTR; MAX_SHRT_DATE_SEPARATORS] = [
        w!(".") as PWSTR,
        w!("/") as PWSTR,
        w!("-") as PWSTR,
    ];
    let mut nCBIndex: INT_PTR;
    let nRetCode: INT_PTR;

    SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < MAX_SHRT_DATE_SEPARATORS as INT_PTR {
        SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                            CB_ADDSTRING,
                            0 as WPARAM,
                            ShortDateSepSamples[nCBIndex as usize] as LPARAM);
        nCBIndex += 1;
    }

    nRetCode = SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                                   CB_SELECTSTRING,
                                   -1i32 as WPARAM,
                                   (*pGlobalData).szDateSep.as_ptr() as LPARAM);

    if nRetCode == CB_ERR as INT_PTR {
        SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                            CB_ADDSTRING,
                            0 as WPARAM,
                            (*pGlobalData).szDateSep.as_ptr() as LPARAM);
        SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                            CB_SELECTSTRING,
                            -1i32 as WPARAM,
                            (*pGlobalData).szDateSep.as_ptr() as LPARAM);
    }
}
unsafe extern "system" fn ShortDateFormatEnumProc(lpTimeFormatString: PWSTR) -> BOOL {
    SendMessageW(
        hwndEnum,
        CB_ADDSTRING,
        0,
        lpTimeFormatString as LPARAM,
    );

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "C" fn InitShortDateCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut nRetCode: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATEFMT_COMBO) as i32,
                        CB_LIMITTEXT,
                        (MAX_SHORTDATEFORMAT) as WPARAM,
                        0);
    SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATESEP_COMBO) as i32,
                        CB_LIMITTEXT,
                        (MAX_DATESEPARATOR) as WPARAM,
                        0);

    SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATEFMT_COMBO) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    hwndEnum = GetDlgItem(hwndDlg, (IDC_SHRTDATEFMT_COMBO) as i32);
    EnumDateFormatsW(Some(ShortDateFormatEnumProc), (*pGlobalData).UserLCID, DATE_SHORTDATE);

    nRetCode = (SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATEFMT_COMBO) as i32,
                                   CB_SELECTSTRING,
                                   -1i32 as WPARAM,
                                   (*pGlobalData).szShortDateFormat.as_ptr() as LPARAM)) as i32;

    if nRetCode == CB_ERR as c_int {
        SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATEFMT_COMBO) as i32,
                            CB_ADDSTRING,
                            0,
                            (*pGlobalData).szShortDateFormat.as_ptr() as LPARAM);
        SendDlgItemMessageW(hwndDlg, (IDC_SHRTDATEFMT_COMBO) as i32,
                            CB_SELECTSTRING,
                            -1i32 as WPARAM,
                            (*pGlobalData).szShortDateFormat.as_ptr() as LPARAM);
    }
}
unsafe extern "C" fn InitLongDateCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut nRetCode: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_LONGDATEFMT_COMBO) as i32,
                        CB_LIMITTEXT,
                        (MAX_LONGDATEFORMAT) as WPARAM,
                        0);

    SendDlgItemMessageW(hwndDlg, (IDC_LONGDATEFMT_COMBO) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    hwndEnum = GetDlgItem(hwndDlg, (IDC_LONGDATEFMT_COMBO) as i32);
    EnumDateFormatsW(Some(ShortDateFormatEnumProc), (*pGlobalData).UserLCID, DATE_LONGDATE);

    nRetCode = (SendDlgItemMessageW(hwndDlg, (IDC_LONGDATEFMT_COMBO) as i32,
                                   CB_SELECTSTRING,
                                   -1i32 as WPARAM,
                                   (*pGlobalData).szLongDateFormat.as_ptr() as LPARAM)) as i32;

    if nRetCode == CB_ERR as c_int {
        SendDlgItemMessageW(hwndDlg, (IDC_LONGDATEFMT_COMBO) as i32,
                            CB_ADDSTRING,
                            0,
                            (*pGlobalData).szLongDateFormat.as_ptr() as LPARAM);
        SendDlgItemMessageW(hwndDlg, (IDC_LONGDATEFMT_COMBO) as i32,
                            CB_SELECTSTRING,
                            -1i32 as WPARAM,
                            (*pGlobalData).szLongDateFormat.as_ptr() as LPARAM);
    }
}
unsafe extern "C" fn SetMaxDate(hwndDlg: HWND, lcid: LCID) {
    let mut szMaxDateVal: [u16; YEAR_STR_MAX_SIZE] = [0; YEAR_STR_MAX_SIZE];
    let hWndYearSpin: HWND;
    let nSpinVal: i32;

    hWndYearSpin = GetDlgItem(hwndDlg, (IDC_SCR_MAX_YEAR) as i32);

    nSpinVal = LOWORD!(SendMessageW(hWndYearSpin,
                                   UDM_GETPOS,
                                   0,
                                   0)) as i32;

    _itow(nSpinVal, szMaxDateVal.as_mut_ptr(), (DECIMAL_RADIX) as i32);

    SetCalendarInfoW(lcid,
                     CAL_GREGORIAN,
                     48,
                     szMaxDateVal.as_ptr() as PCWSTR);
}
unsafe extern "C" fn GetMaxDate(lcid: LCID) -> i32 {
    let mut nMaxDateVal: i32 = 0;

    GetCalendarInfoW(
        lcid,
        CAL_GREGORIAN,
        CAL_ITWODIGITYEARMAX | CAL_RETURN_NUMBER,
        core::ptr::null_mut(),
        0,
        &mut nMaxDateVal as *mut i32 as LPDWORD,
    );

    nMaxDateVal
}
unsafe extern "C" fn SetMinDate(hwndDlg: HWND) {
    let mut OutBuffer: [u16; YEAR_STR_MAX_SIZE] = [0; YEAR_STR_MAX_SIZE];
    let hWndYearSpin: HWND;
    let nSpinVal: i32;

    hWndYearSpin = GetDlgItem(hwndDlg, (IDC_SCR_MAX_YEAR) as i32);

    nSpinVal = LOWORD!(SendMessageW(hWndYearSpin,
                                   UDM_GETPOS,
                                   0,
                                   0)) as i32;

    wsprintf(OutBuffer.as_mut_ptr(), w!("%d"), (nSpinVal - YEAR_DIFF) as u32);
    SendDlgItemMessageW(hwndDlg, (IDC_FIRSTYEAR_EDIT) as i32,
                        WM_SETTEXT,
                        0,
                        OutBuffer.as_ptr() as LPARAM);
}
unsafe extern "C" fn InitMinMaxDateSpin(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut OutBuffer: [u16; YEAR_STR_MAX_SIZE] = [0; YEAR_STR_MAX_SIZE];
    let hWndYearSpin: HWND;

    SendDlgItemMessageW(hwndDlg, (IDC_FIRSTYEAR_EDIT) as i32,
                        EM_LIMITTEXT,
                        (MAX_YEAR_EDIT) as WPARAM,
                        0);
    SendDlgItemMessageW(hwndDlg, (IDC_SECONDYEAR_EDIT) as i32,
                        EM_LIMITTEXT,
                        (MAX_YEAR_EDIT) as WPARAM,
                        0);

    hWndYearSpin = GetDlgItem(hwndDlg, (IDC_SCR_MAX_YEAR) as i32);

    wsprintf(OutBuffer.as_mut_ptr(), w!("%04d"), GetMaxDate((*pGlobalData).UserLCID) as u32);
    SendDlgItemMessageW(hwndDlg, (IDC_SECONDYEAR_EDIT) as i32,
                        WM_SETTEXT,
                        0,
                        OutBuffer.as_ptr() as LPARAM);

    wsprintf(OutBuffer.as_mut_ptr(), w!("%04d"), (GetMaxDate((*pGlobalData).UserLCID) - YEAR_DIFF) as u32);
    SendDlgItemMessageW(hwndDlg, (IDC_FIRSTYEAR_EDIT) as i32,
                        WM_SETTEXT,
                        0,
                        OutBuffer.as_ptr() as LPARAM);

    SendMessageW(hWndYearSpin,
                 UDM_SETRANGE,
                 0,
                 (MAKELONG!(MAX_YEAR,YEAR_DIFF)) as LPARAM);

    SendMessageW(hWndYearSpin,
                 UDM_SETPOS,
                 0,
                 (MAKELONG!(GetMaxDate((*pGlobalData).UserLCID),0)) as LPARAM);
}
unsafe extern "C" fn UpdateDateLocaleSamples(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut OutBuffer: [u16; (MAX_SAMPLES_STR_SIZE) as usize] = [0; (MAX_SAMPLES_STR_SIZE) as usize];

    GetDateFormatW(
        (*pGlobalData).UserLCID,
        0,
        core::ptr::null(),
        (*pGlobalData).szShortDateFormat.as_ptr(),
        OutBuffer.as_mut_ptr(),
        (MAX_SAMPLES_STR_SIZE) as i32,
    );
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_SHRTDATESAMPLE_EDIT) as i32,
        WM_SETTEXT,
        0,
        OutBuffer.as_ptr() as LPARAM,
    );

    GetDateFormatW(
        (*pGlobalData).UserLCID,
        0,
        core::ptr::null(),
        (*pGlobalData).szLongDateFormat.as_ptr(),
        OutBuffer.as_mut_ptr(),
        (MAX_SAMPLES_STR_SIZE) as i32,
    );
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_LONGDATESAMPLE_EDIT) as i32,
        WM_SETTEXT,
        0,
        OutBuffer.as_ptr() as LPARAM,
    );
}
unsafe extern "C" fn GetDateSetting(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA) -> u8 {
    let mut szLongDateFormat: [u16; (MAX_LONGDATEFORMAT) as usize] = [0; (MAX_LONGDATEFORMAT) as usize];
    let mut szShortDateFormat: [u16; (MAX_SHORTDATEFORMAT) as usize] = [0; (MAX_SHORTDATEFORMAT) as usize];
    let mut szDateSeparator: [u16; (MAX_DATESEPARATOR) as usize] = [0; (MAX_DATESEPARATOR) as usize];

    if SetLongDateFormat(hwndDlg, szLongDateFormat.as_mut_ptr()) == 0 ||
       SetShortDateFormat(hwndDlg, szShortDateFormat.as_mut_ptr()) == 0 ||
       SetShortDateSep(hwndDlg, szDateSeparator.as_mut_ptr()) == 0
    {
        return (FALSE as BOOL) as BOOLEAN;
    }

    wcscpy((*pGlobalData).szLongDateFormat.as_mut_ptr(), szLongDateFormat.as_ptr());
    wcscpy((*pGlobalData).szShortDateFormat.as_mut_ptr(), szShortDateFormat.as_ptr());
    wcscpy((*pGlobalData).szDateSep.as_mut_ptr(), szDateSeparator.as_ptr());

    return (TRUE as BOOL) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn DatePageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBALDATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as *mut _PROPSHEETPAGEW)).lParam as PGLOBALDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            InitMinMaxDateSpin(hwndDlg, pGlobalData);
            UpdateDateLocaleSamples(hwndDlg, pGlobalData);
            InitShortDateCB(hwndDlg, pGlobalData);
            InitLongDateCB(hwndDlg, pGlobalData);
            InitShortDateSepSamples(hwndDlg, pGlobalData);
            (*pGlobalData).bEnableYearNotification = (TRUE) as i32;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_SECONDYEAR_EDIT => {
                    if HIWORD!(wParam) == (EN_CHANGE) as u16 &&
                        !pGlobalData.is_null() &&
                        (*pGlobalData).bEnableYearNotification == (TRUE) as i32 {
                        SetMinDate(hwndDlg);

                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_CALTYPE_COMBO |
                IDC_HIJCHRON_COMBO |
                IDC_SHRTDATEFMT_COMBO |
                IDC_LONGDATEFMT_COMBO |
                IDC_SHRTDATESEP_COMBO => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 ||
                        HIWORD!(wParam) == (CBN_EDITCHANGE) as u16 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).code == PSN_APPLY as u32 {
                if GetDateSetting(hwndDlg, pGlobalData) != 0 {
                    (*pGlobalData).bUserLocaleChanged = (TRUE) as i32;
                    SetMaxDate(hwndDlg, (*pGlobalData).UserLCID);
                    UpdateDateLocaleSamples(hwndDlg, pGlobalData);
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
//  * FILE:            dll/cpl/intl/date.c
//  * PURPOSE:         Date property page
//  * PROGRAMMERS:     Eric Kohl
//  *                  Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "intl.h"
//
// /* GLOBALS ******************************************************************/
//
// #define YEAR_STR_MAX_SIZE        5
// #define MAX_SHRT_DATE_SEPARATORS 3
// #define STD_DATE_SEP             L"."
// #define YEAR_DIFF                (99)
// #define MAX_YEAR                 (9999)
//
// static HWND hwndEnum = NULL;
//
// /* FUNCTIONS ****************************************************************/
//
// /* If char is 'y' or 'M' or 'd' return TRUE, else FALSE */
// BOOL
// isDateCompAl(WCHAR alpha)
// {
//     if ((alpha == L'y') || (alpha == L'M') || (alpha == L'd') || (alpha == L' '))
//         return TRUE;
//     else
//         return FALSE;
// }
//
// /* Find first date separator in string */
// LPTSTR
// FindDateSep(const WCHAR *szSourceStr)
// {
//     PWSTR pszFoundSep;
//     UINT nDateCompCount=0;
//     UINT nDateSepCount=0;
//
//     pszFoundSep = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, MAX_SAMPLES_STR_SIZE * sizeof(WCHAR));
//     if (pszFoundSep == NULL)
//         return NULL;
//
//     wcscpy(pszFoundSep,STD_DATE_SEP);
//
//     while (nDateCompCount < wcslen(szSourceStr))
//     {
//         if (!isDateCompAl(szSourceStr[nDateCompCount]) && (szSourceStr[nDateCompCount] != L'\''))
//         {
//             while (!isDateCompAl(szSourceStr[nDateCompCount]) && (szSourceStr[nDateCompCount] != L'\''))
//             {
//                 pszFoundSep[nDateSepCount++] = szSourceStr[nDateCompCount];
//                 nDateCompCount++;
//             }
//
//             pszFoundSep[nDateSepCount] = L'\0';
//             return pszFoundSep;
//         }
//
//         nDateCompCount++;
//     }
//
//     return pszFoundSep;
// }
//
// /* Replace given template in source string with string to replace and return received string */
//
//
// /* Setted up short date separator to registry */
// static BOOL
// SetShortDateSep(HWND hwndDlg, PWSTR pszShortDateSep)
// {
//     INT nSepStrSize;
//     INT nSepCount;
//
//     /* Get separator */
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                         WM_GETTEXT,
//                         (WPARAM)MAX_SAMPLES_STR_SIZE,
//                         (LPARAM)pszShortDateSep);
//
//     /* Get separator string size */
//     nSepStrSize = wcslen(pszShortDateSep);
//
//     /* Check date components */
//     for (nSepCount = 0; nSepCount < nSepStrSize; nSepCount++)
//     {
//         if (iswalnum(pszShortDateSep[nSepCount]) || (pszShortDateSep[nSepCount] == L'\''))
//         {
//             PrintErrorMsgBox(IDS_ERROR_SYMBOL_SEPARATE);
//             return FALSE;
//         }
//     }
//
//     if (nSepStrSize == 0)
//     {
//         PrintErrorMsgBox(IDS_ERROR_SYMBOL_SEPARATE);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* Setted up short date format to registry */
// static BOOL
// SetShortDateFormat(HWND hwndDlg, PWSTR pszShortDateFmt)
// {
//     WCHAR szShortDateSep[MAX_SAMPLES_STR_SIZE];
//     WCHAR szFoundDateSep[MAX_SAMPLES_STR_SIZE];
//     PWSTR pszResultStr;
//     PWSTR pszFoundSep;
//     BOOL OpenApostFlg = FALSE;
//     INT nFmtStrSize;
//     INT nDateCompCount;
//
//     /* Get format */
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATEFMT_COMBO,
//                         WM_GETTEXT,
//                         (WPARAM)MAX_SAMPLES_STR_SIZE,
//                         (LPARAM)pszShortDateFmt);
//
//     /* Get separator */
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                         WM_GETTEXT,
//                         (WPARAM)MAX_SAMPLES_STR_SIZE,
//                         (LPARAM)szShortDateSep);
//
//     /* Get format-string size */
//     nFmtStrSize = wcslen(pszShortDateFmt);
//
//     /* Check date components */
//     for (nDateCompCount = 0; nDateCompCount < nFmtStrSize; nDateCompCount++)
//     {
//         if (pszShortDateFmt[nDateCompCount] == L'\'')
//         {
//             OpenApostFlg = !OpenApostFlg;
//         }
//
//         if (iswalnum(pszShortDateFmt[nDateCompCount]) &&
//             !isDateCompAl(pszShortDateFmt[nDateCompCount]) &&
//             !OpenApostFlg)
//         {
//             PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_SHORT);
//             return FALSE;
//         }
//     }
//
//     if (OpenApostFlg || nFmtStrSize == 0)
//     {
//         PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_SHORT);
//         return FALSE;
//     }
//
//     pszFoundSep = FindDateSep(pszShortDateFmt);
//     if (pszFoundSep != NULL)
//     {
//         /* Substring replacement of separator */
//         wcscpy(szFoundDateSep, pszFoundSep);
//         pszResultStr = ReplaceSubStr(pszShortDateFmt, szShortDateSep, szFoundDateSep);
//         if (pszResultStr != NULL)
//         {
//             wcscpy(pszShortDateFmt, pszResultStr);
//             HeapFree(GetProcessHeap(), 0, pszResultStr);
//         }
//
//         HeapFree(GetProcessHeap(), 0, pszFoundSep);
//     }
//
//     return TRUE;
// }
//
// /* Setted up long date format to registry */
// static BOOL
// SetLongDateFormat(HWND hwndDlg, PWSTR pszLongDateFmt)
// {
//     BOOL OpenApostFlg = FALSE;
//     INT nFmtStrSize;
//     INT nDateCompCount;
//
//     /* Get format */
//     SendDlgItemMessageW(hwndDlg, IDC_LONGDATEFMT_COMBO,
//                         WM_GETTEXT,
//                         (WPARAM)MAX_SAMPLES_STR_SIZE,
//                         (LPARAM)pszLongDateFmt);
//
//     /* Get format string size */
//     nFmtStrSize = wcslen(pszLongDateFmt);
//
//     /* Check date components */
//     for (nDateCompCount = 0; nDateCompCount < nFmtStrSize; nDateCompCount++)
//     {
//         if (pszLongDateFmt[nDateCompCount] == L'\'')
//         {
//             OpenApostFlg = !OpenApostFlg;
//         }
//
//         if (iswalnum(pszLongDateFmt[nDateCompCount]) &&
//             !isDateCompAl(pszLongDateFmt[nDateCompCount]) &&
//             !OpenApostFlg)
//         {
//             PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_LONG);
//             return FALSE;
//         }
//     }
//
//     if (OpenApostFlg || nFmtStrSize == 0)
//     {
//         PrintErrorMsgBox(IDS_ERROR_SYMBOL_FORMAT_LONG);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* Init short date separator control box */
// static VOID
// InitShortDateSepSamples(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     PWSTR ShortDateSepSamples[MAX_SHRT_DATE_SEPARATORS] =
//     {
//         L".",
//         L"/",
//         L"-"
//     };
//     INT nCBIndex;
//     INT nRetCode;
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of separators */
//     for (nCBIndex = 0; nCBIndex < MAX_SHRT_DATE_SEPARATORS; nCBIndex++)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)ShortDateSepSamples[nCBIndex]);
//     }
//
//     /* Set current item to value from registry */
//     nRetCode = SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                                    CB_SELECTSTRING,
//                                    -1,
//                                    (LPARAM)pGlobalData->szDateSep);
//
//     /* If it is not successful, add new value to list and select them */
//     if (nRetCode == CB_ERR)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)pGlobalData->szDateSep);
//         SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                             CB_SELECTSTRING,
//                             -1,
//                             (LPARAM)pGlobalData->szDateSep);
//     }
// }
//
// static BOOL __stdcall
// ShortDateFormatEnumProc(PWSTR lpTimeFormatString)
// {
//     SendMessageW(hwndEnum,
//                  CB_ADDSTRING,
//                  0,
//                  (LPARAM)lpTimeFormatString);
//
//     return TRUE;
// }
//
// /* Init short date control box */
// VOID
// InitShortDateCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     INT nRetCode;
//
//     /* Limit text lengths */
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATEFMT_COMBO,
//                         CB_LIMITTEXT,
//                         MAX_SHORTDATEFORMAT,
//                         0);
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESEP_COMBO,
//                         CB_LIMITTEXT,
//                         MAX_DATESEPARATOR,
//                         0);
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATEFMT_COMBO,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Enumerate short date formats */
//     hwndEnum = GetDlgItem(hwndDlg, IDC_SHRTDATEFMT_COMBO);
//     EnumDateFormatsW(ShortDateFormatEnumProc, pGlobalData->UserLCID, DATE_SHORTDATE);
//
//     /* Set current item to value from registry */
//     nRetCode = SendDlgItemMessageW(hwndDlg, IDC_SHRTDATEFMT_COMBO,
//                                    CB_SELECTSTRING,
//                                    -1,
//                                    (LPARAM)pGlobalData->szShortDateFormat);
//
//     /* If it is not successful, add new value to list and select them */
//     if (nRetCode == CB_ERR)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_SHRTDATEFMT_COMBO,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)pGlobalData->szShortDateFormat);
//         SendDlgItemMessageW(hwndDlg, IDC_SHRTDATEFMT_COMBO,
//                             CB_SELECTSTRING,
//                             -1,
//                             (LPARAM)pGlobalData->szShortDateFormat);
//     }
// }
//
// /* Init long date control box */
// static VOID
// InitLongDateCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     INT nRetCode;
//
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_LONGDATEFMT_COMBO,
//                         CB_LIMITTEXT,
//                         MAX_LONGDATEFORMAT,
//                         0);
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_LONGDATEFMT_COMBO,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Enumerate short long formats */
//     hwndEnum = GetDlgItem(hwndDlg, IDC_LONGDATEFMT_COMBO);
//     EnumDateFormatsW(ShortDateFormatEnumProc, pGlobalData->UserLCID, DATE_LONGDATE);
//
//     /* Set current item to value from registry */
//     nRetCode = SendDlgItemMessageW(hwndDlg, IDC_LONGDATEFMT_COMBO,
//                                    CB_SELECTSTRING,
//                                    -1,
//                                    (LPARAM)pGlobalData->szLongDateFormat);
//
//     /* If it is not successful, add new value to list and select them */
//     if (nRetCode == CB_ERR)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_LONGDATEFMT_COMBO,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)pGlobalData->szLongDateFormat);
//         SendDlgItemMessageW(hwndDlg, IDC_LONGDATEFMT_COMBO,
//                             CB_SELECTSTRING,
//                             -1,
//                             (LPARAM)pGlobalData->szLongDateFormat);
//     }
// }
//
// /* Set up max date value to registry */
// static VOID
// SetMaxDate(HWND hwndDlg, LCID lcid)
// {
//     WCHAR szMaxDateVal[YEAR_STR_MAX_SIZE];
//     HWND hWndYearSpin;
//     INT nSpinVal;
//
//     hWndYearSpin = GetDlgItem(hwndDlg, IDC_SCR_MAX_YEAR);
//
//     /* Get spin value */
//     nSpinVal = LOWORD(SendMessageW(hWndYearSpin,
//                                    UDM_GETPOS,
//                                    0,
//                                    0));
//
//     /* convert to wide char */
//     _itow(nSpinVal, szMaxDateVal, DECIMAL_RADIX);
//
//     /* Save max date value */
//     SetCalendarInfoW(lcid,
//                      CAL_GREGORIAN,
//                      48 , /* CAL_ITWODIGITYEARMAX */
//                      (PCWSTR)szMaxDateVal);
// }
//
// /* Get max date value from registry set */
// static INT
// GetMaxDate(LCID lcid)
// {
//     INT nMaxDateVal = 0;
//
//     GetCalendarInfoW(lcid,
//                      CAL_GREGORIAN,
//                      CAL_ITWODIGITYEARMAX | CAL_RETURN_NUMBER,
//                      NULL,
//                      0, /* ret type - number */
//                      (LPDWORD)&nMaxDateVal);
//
//     return nMaxDateVal;
// }
//
// /* Set's MIN data edit control value to MAX-99 */
// static VOID
// SetMinDate(HWND hwndDlg)
// {
//     WCHAR OutBuffer[YEAR_STR_MAX_SIZE];
//     HWND hWndYearSpin;
//     INT nSpinVal;
//
//     hWndYearSpin = GetDlgItem(hwndDlg, IDC_SCR_MAX_YEAR);
//
//     /* Get spin value */
//     nSpinVal = LOWORD(SendMessageW(hWndYearSpin,
//                                    UDM_GETPOS,
//                                    0,
//                                    0));
//
//     /* Set min year value */
//     wsprintf(OutBuffer, L"%d", (DWORD)nSpinVal - YEAR_DIFF);
//     SendDlgItemMessageW(hwndDlg, IDC_FIRSTYEAR_EDIT,
//                         WM_SETTEXT,
//                         0,
//                         (LPARAM)OutBuffer);
// }
//
// /* Init spin control */
// static VOID
// InitMinMaxDateSpin(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     WCHAR OutBuffer[YEAR_STR_MAX_SIZE];
//     HWND hWndYearSpin;
//
//     /* Limit text lengths */
//     SendDlgItemMessageW(hwndDlg, IDC_FIRSTYEAR_EDIT,
//                         EM_LIMITTEXT,
//                         MAX_YEAR_EDIT,
//                         0);
//     SendDlgItemMessageW(hwndDlg, IDC_SECONDYEAR_EDIT,
//                         EM_LIMITTEXT,
//                         MAX_YEAR_EDIT,
//                         0);
//
//     hWndYearSpin = GetDlgItem(hwndDlg, IDC_SCR_MAX_YEAR);
//
//     /* Init max date value */
//     wsprintf(OutBuffer, L"%04d", (DWORD)GetMaxDate(pGlobalData->UserLCID));
//     SendDlgItemMessageW(hwndDlg, IDC_SECONDYEAR_EDIT,
//                         WM_SETTEXT,
//                         0,
//                         (LPARAM)OutBuffer);
//
//     /* Init min date value */
//     wsprintf(OutBuffer, L"%04d", (DWORD)GetMaxDate(pGlobalData->UserLCID) - YEAR_DIFF);
//     SendDlgItemMessageW(hwndDlg, IDC_FIRSTYEAR_EDIT,
//                         WM_SETTEXT,
//                         0,
//                         (LPARAM)OutBuffer);
//
//     /* Init updown control */
//     /* Set bounds */
//     SendMessageW(hWndYearSpin,
//                  UDM_SETRANGE,
//                  0,
//                  MAKELONG(MAX_YEAR,YEAR_DIFF));
//
//     /* Set current value */
//     SendMessageW(hWndYearSpin,
//                  UDM_SETPOS,
//                  0,
//                  MAKELONG(GetMaxDate(pGlobalData->UserLCID),0));
// }
//
// /* Update all date locale samples */
// static VOID
// UpdateDateLocaleSamples(HWND hwndDlg,
//                         PGLOBALDATA pGlobalData)
// {
//     WCHAR OutBuffer[MAX_SAMPLES_STR_SIZE];
//
//     /* Get short date format sample */
//     GetDateFormatW(pGlobalData->UserLCID, 0, NULL,
//                    pGlobalData->szShortDateFormat, OutBuffer,
//                    MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hwndDlg, IDC_SHRTDATESAMPLE_EDIT, WM_SETTEXT,
//                         0, (LPARAM)OutBuffer);
//
//     /* Get long date sample */
//     GetDateFormatW(pGlobalData->UserLCID, 0, NULL,
//                    pGlobalData->szLongDateFormat, OutBuffer,
//                    MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hwndDlg, IDC_LONGDATESAMPLE_EDIT,
//                         WM_SETTEXT, 0, (LPARAM)OutBuffer);
// }
//
//
// static
// BOOL
// GetDateSetting(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR szLongDateFormat[MAX_LONGDATEFORMAT];
//     WCHAR szShortDateFormat[MAX_SHORTDATEFORMAT];
//     WCHAR szDateSeparator[MAX_DATESEPARATOR];
//
//     if (!SetLongDateFormat(hwndDlg, szLongDateFormat) ||
//         !SetShortDateFormat(hwndDlg, szShortDateFormat) ||
//         !SetShortDateSep(hwndDlg, szDateSeparator))
//     {
//         return FALSE;
//     }
//
//     /* Store settings in global data */
//     wcscpy(pGlobalData->szLongDateFormat, szLongDateFormat);
//     wcscpy(pGlobalData->szShortDateFormat, szShortDateFormat);
//     wcscpy(pGlobalData->szDateSep, szDateSeparator);
//
//     return TRUE;
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// DatePageProc(HWND hwndDlg,
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
//             InitMinMaxDateSpin(hwndDlg, pGlobalData);
//             UpdateDateLocaleSamples(hwndDlg, pGlobalData);
//             InitShortDateCB(hwndDlg, pGlobalData);
//             InitLongDateCB(hwndDlg, pGlobalData);
//             InitShortDateSepSamples(hwndDlg, pGlobalData);
//             /* TODO: Add other calendar types */
//             pGlobalData->bEnableYearNotification = TRUE;
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_SECONDYEAR_EDIT:
//                     if (HIWORD(wParam) == EN_CHANGE &&
//                         pGlobalData != NULL &&
//                         pGlobalData->bEnableYearNotification == TRUE)
//                     {
//                         SetMinDate(hwndDlg);
//
//                         /* Enable the Apply button */
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//
//                 case IDC_CALTYPE_COMBO:
//                 case IDC_HIJCHRON_COMBO:
//                 case IDC_SHRTDATEFMT_COMBO:
//                 case IDC_LONGDATEFMT_COMBO:
//                 case IDC_SHRTDATESEP_COMBO:
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
//                 if (GetDateSetting(hwndDlg, pGlobalData))
//                 {
//                     pGlobalData->bUserLocaleChanged = TRUE;
//                     SetMaxDate(hwndDlg, pGlobalData->UserLCID);
//                     UpdateDateLocaleSamples(hwndDlg, pGlobalData);
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

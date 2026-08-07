// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const POSITIVE_EXAMPLE: &[u16] = &[0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039, 0x002E, 0x0030, 0x0030, 0x0000];
pub const NEGATIVE_EXAMPLE: &[u16] = &[0x002D, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039, 0x002E, 0x0030, 0x0030, 0x0000];
unsafe extern "C" fn UpdateExamples(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut szBuffer: [u16; (MAX_FMT_SIZE) as usize] = [0; (MAX_FMT_SIZE) as usize];
    let mut CurrencyFormat: CURRENCYFMTW = core::mem::zeroed();

    CurrencyFormat.NumDigits = ((*pGlobalData).nCurrDigits) as u32;
    CurrencyFormat.LeadingZero = ((*pGlobalData).nNumLeadingZero) as u32;
    CurrencyFormat.Grouping = GroupingFormats[(*pGlobalData).nCurrGrouping as usize].nInteger;
    CurrencyFormat.lpDecimalSep = (*pGlobalData).szCurrDecimalSep.as_mut_ptr();
    CurrencyFormat.lpThousandSep = (*pGlobalData).szCurrThousandSep.as_mut_ptr();
    CurrencyFormat.NegativeOrder = ((*pGlobalData).nCurrNegFormat) as u32;
    CurrencyFormat.PositiveOrder = ((*pGlobalData).nCurrPosFormat) as u32;
    CurrencyFormat.lpCurrencySymbol = (*pGlobalData).szCurrSymbol.as_mut_ptr();

    GetCurrencyFormatW((*pGlobalData).UserLCID, 0,
                       POSITIVE_EXAMPLE.as_ptr(),
                       &mut CurrencyFormat, szBuffer.as_mut_ptr(), (MAX_FMT_SIZE) as i32);

    SendDlgItemMessageW(hwndDlg, (IDC_CURRENCYPOSSAMPLE) as i32, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);

    GetCurrencyFormatW((*pGlobalData).UserLCID, 0,
                       NEGATIVE_EXAMPLE.as_ptr(),
                       &mut CurrencyFormat, szBuffer.as_mut_ptr(), (MAX_FMT_SIZE) as i32);

    SendDlgItemMessageW(hwndDlg, (IDC_CURRENCYNEGSAMPLE) as i32, WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
}
unsafe extern "C" fn InitCurrencySymbols(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    SendDlgItemMessageW(hwndDlg, (IDC_CURRENCYSYMBOL) as i32,
                        CB_LIMITTEXT,
                        (MAX_CURRSYMBOL - 1) as WPARAM,
                        0);

    SendDlgItemMessageW(hwndDlg, (IDC_CURRENCYSYMBOL) as i32,
                        CB_ADDSTRING,
                        0,
                        (*pGlobalData).szCurrSymbol.as_ptr() as LPARAM);

    SendDlgItemMessageW(hwndDlg, (IDC_CURRENCYSYMBOL) as i32,
                        CB_SETCURSEL,
                        0,
                        0);
}
unsafe extern "C" fn InitCurrencyPositiveFormats(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut szBuffer: [u16; (MAX_FMT_SIZE) as usize] = [0; (MAX_FMT_SIZE) as usize];
    let mut cyFmt: CURRENCYFMTW = core::mem::zeroed();
    let mut i: i32;

    cyFmt.NumDigits = ((*pGlobalData).nCurrDigits) as u32;
    cyFmt.LeadingZero = 0;
    cyFmt.Grouping = 3;
    cyFmt.lpDecimalSep = (*pGlobalData).szCurrDecimalSep.as_mut_ptr();
    cyFmt.lpThousandSep = (*pGlobalData).szCurrThousandSep.as_mut_ptr();
    cyFmt.lpCurrencySymbol = (*pGlobalData).szCurrSymbol.as_mut_ptr();
    cyFmt.NegativeOrder = 0;

    i = 0;
    while i < 4 {
        cyFmt.PositiveOrder = i as u32;
        GetCurrencyFormatW(
            (*pGlobalData).UserLCID,
            0,
            w!("1.1"),
            &mut cyFmt,
            szBuffer.as_mut_ptr(),
            (MAX_FMT_SIZE) as i32,
        );

        SendDlgItemMessageW(
            hwndDlg,
            (IDC_CURRENCYPOSVALUE) as i32,
            CB_INSERTSTRING,
            -1i32 as WPARAM,
            szBuffer.as_ptr() as LPARAM,
        );

        i += 1;
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYPOSVALUE) as i32,
        CB_SETCURSEL,
        (*pGlobalData).nCurrPosFormat as WPARAM,
        0,
    );
}
unsafe extern "C" fn InitCurrencyNegativeFormats(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut szBuffer: [u16; (MAX_FMT_SIZE) as usize] = [0u16; (MAX_FMT_SIZE) as usize];
    let mut cyFmt: CURRENCYFMTW = core::mem::zeroed();
    let mut i;

    // negative currency values
    cyFmt.NumDigits = ((*pGlobalData).nCurrDigits) as u32;
    cyFmt.LeadingZero = 0;
    cyFmt.Grouping = 3;
    cyFmt.lpDecimalSep = (*pGlobalData).szCurrDecimalSep.as_mut_ptr();
    cyFmt.lpThousandSep = (*pGlobalData).szCurrThousandSep.as_mut_ptr();
    cyFmt.lpCurrencySymbol = (*pGlobalData).szCurrSymbol.as_mut_ptr();
    cyFmt.PositiveOrder = 0;

    i = 0;
    while i < 16 {
        cyFmt.NegativeOrder = i;
        GetCurrencyFormatW(
            (*pGlobalData).UserLCID,
            0,
            w!("-1.1"),
            core::ptr::addr_of!(cyFmt),
            szBuffer.as_mut_ptr(),
            (MAX_FMT_SIZE) as i32,
        );

        SendDlgItemMessageW(
            hwndDlg,
            (IDC_CURRENCYNEGVALUE) as i32,
            CB_INSERTSTRING,
            -1i32 as WPARAM,
            (szBuffer.as_ptr()) as LPARAM,
        );

        i += 1;
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYNEGVALUE) as i32,
        CB_SETCURSEL,
        ((*pGlobalData).nCurrNegFormat) as WPARAM,
        0,
    );
}
unsafe extern "C" fn InitCurrencyDecimalSeparators(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYDECSEP) as i32,
        CB_LIMITTEXT,
        (MAX_CURRDECIMALSEP - 1) as WPARAM,
        0,
    );

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYDECSEP) as i32,
        CB_ADDSTRING,
        0,
        core::ptr::addr_of!((*pGlobalData).szCurrDecimalSep) as LPARAM,
    );

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYDECSEP) as i32,
        CB_SETCURSEL,
        0,
        0,
    );
}
unsafe extern "C" fn InitCurrencyNumFracDigits(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut szBuffer: [u16; MAX_FMT_SIZE as usize] = [0; MAX_FMT_SIZE as usize];

    let mut i = 0 as i32;
    while i < 10 as i32 {
        szBuffer[0 as usize] = (b'0' as u16) + i as u16;
        szBuffer[1 as usize] = 0 as u16;
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_CURRENCYDECNUM) as i32,
            CB_ADDSTRING,
            0,
            szBuffer.as_mut_ptr() as LPARAM,
        );
        i += 1;
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYDECNUM) as i32,
        CB_SETCURSEL,
        ((*pGlobalData).nCurrDigits as LPARAM) as WPARAM,
        0 as LPARAM,
    );
}
unsafe extern "C" fn InitCurrencyGroupSeparators(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYGRPSEP) as i32,
        CB_LIMITTEXT,
        (MAX_CURRTHOUSANDSEP - 1) as WPARAM,
        0,
    );
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYGRPSEP) as i32,
        CB_ADDSTRING,
        0,
        core::ptr::addr_of!((*pGlobalData).szCurrThousandSep) as LPARAM,
    );
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYGRPSEP) as i32,
        CB_SETCURSEL,
        0,
        0,
    );
}
unsafe extern "C" fn InitDigitGroupCB(hwndDlg: HWND, pGlobalData: *mut GLOBALDATA) {
    let mut szBuffer: [u16; (MAX_FMT_SIZE) as usize] = [0u16; (MAX_FMT_SIZE) as usize];
    let mut cyFmt: CURRENCYFMTW = core::mem::zeroed();

    cyFmt.NumDigits = 0;
    cyFmt.LeadingZero = 0;
    cyFmt.lpDecimalSep = w!("") as PWSTR;
    cyFmt.lpThousandSep = (*pGlobalData).szCurrThousandSep.as_mut_ptr();
    cyFmt.PositiveOrder = 0;
    cyFmt.NegativeOrder = 0;
    cyFmt.lpCurrencySymbol = w!("") as PWSTR;

    for i in 0..MAX_GROUPINGFORMATS {
        cyFmt.Grouping = GroupingFormats[i as usize].nInteger;

        GetCurrencyFormatW(
            (*pGlobalData).UserLCID,
            0,
            w!("123456789"),
            &cyFmt,
            szBuffer.as_mut_ptr(),
            (MAX_FMT_SIZE) as i32,
        );
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_CURRENCYGRPNUM) as i32,
            CB_INSERTSTRING,
            -1i32 as WPARAM,
            szBuffer.as_ptr() as LPARAM,
        );
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_CURRENCYGRPNUM) as i32,
        CB_SETCURSEL,
        ((*pGlobalData).nCurrGrouping) as WPARAM,
        0,
    );
}
unsafe extern "C" fn GetCurrencySetting(
    hwndDlg: HWND,
    pGlobalData: *mut GLOBALDATA,
) -> i32 {
    let mut szCurrSymbol: [u16; (MAX_CURRSYMBOL) as usize] = [0; (MAX_CURRSYMBOL) as usize];
    let mut szCurrDecimalSep: [u16; (MAX_CURRDECIMALSEP) as usize] = [0; (MAX_CURRDECIMALSEP) as usize];
    let mut szCurrThousandSep: [u16; (MAX_CURRTHOUSANDSEP) as usize] = [0; (MAX_CURRTHOUSANDSEP) as usize];
    let mut nCurrPosFormat: i32 = 0;
    let mut nCurrNegFormat: i32 = 0;
    let mut nCurrDigits: i32 = 0;
    let mut nCurrGrouping: i32 = 0;

    GetSelectedComboBoxText(
        hwndDlg,
        (IDC_CURRENCYSYMBOL) as INT,
        szCurrSymbol.as_mut_ptr(),
        MAX_CURRSYMBOL,
    );

    if szCurrSymbol[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxIndex(
        hwndDlg,
        (IDC_CURRENCYPOSVALUE) as INT,
        &mut nCurrPosFormat,
    );
    GetSelectedComboBoxIndex(
        hwndDlg,
        (IDC_CURRENCYNEGVALUE) as INT,
        &mut nCurrNegFormat,
    );
    GetSelectedComboBoxText(
        hwndDlg,
        (IDC_CURRENCYDECSEP) as INT,
        szCurrDecimalSep.as_mut_ptr(),
        MAX_CURRDECIMALSEP,
    );

    if szCurrDecimalSep[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxIndex(
        hwndDlg,
        (IDC_CURRENCYDECNUM) as INT,
        &mut nCurrDigits,
    );
    GetSelectedComboBoxText(
        hwndDlg,
        (IDC_CURRENCYGRPSEP) as INT,
        szCurrThousandSep.as_mut_ptr(),
        MAX_CURRTHOUSANDSEP,
    );

    if szCurrThousandSep[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxIndex(
        hwndDlg,
        (IDC_CURRENCYGRPNUM) as INT,
        &mut nCurrGrouping,
    );

    wcscpy(
        (*pGlobalData).szCurrSymbol.as_mut_ptr(),
        szCurrSymbol.as_ptr(),
    );
    (*pGlobalData).nCurrGrouping = nCurrGrouping;
    wcscpy(
        (*pGlobalData).szCurrDecimalSep.as_mut_ptr(),
        szCurrDecimalSep.as_ptr(),
    );
    wcscpy(
        (*pGlobalData).szCurrThousandSep.as_mut_ptr(),
        szCurrThousandSep.as_ptr(),
    );
    (*pGlobalData).nCurrPosFormat = nCurrPosFormat;
    (*pGlobalData).nCurrNegFormat = nCurrNegFormat;
    (*pGlobalData).nCurrDigits = nCurrDigits;

    (TRUE) as i32
}
#[no_mangle]
pub unsafe extern "system" fn CurrencyPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    match uMsg {
        WM_INITDIALOG => {
            let pGlobalData = (*(lParam as *const PROPSHEETPAGE)).lParam as PGLOBALDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);
            InitCurrencySymbols(hwndDlg, pGlobalData);
            InitCurrencyPositiveFormats(hwndDlg, pGlobalData);
            InitCurrencyNegativeFormats(hwndDlg, pGlobalData);
            InitCurrencyDecimalSeparators(hwndDlg, pGlobalData);
            InitCurrencyNumFracDigits(hwndDlg, pGlobalData);
            InitCurrencyGroupSeparators(hwndDlg, pGlobalData);
            InitDigitGroupCB(hwndDlg, pGlobalData);
            UpdateExamples(hwndDlg, pGlobalData);
        }
        WM_COMMAND => {
            let loword = LOWORD!(wParam);
            if loword == (IDC_CURRENCYSYMBOL) as u16
                || loword == (IDC_CURRENCYPOSVALUE) as u16
                || loword == (IDC_CURRENCYNEGVALUE) as u16
                || loword == (IDC_CURRENCYDECSEP) as u16
                || loword == (IDC_CURRENCYDECNUM) as u16
                || loword == (IDC_CURRENCYGRPSEP) as u16
                || loword == (IDC_CURRENCYGRPNUM) as u16
            {
                let hiword = HIWORD!(wParam);
                if hiword == (CBN_SELCHANGE) as u16 || hiword == (CBN_EDITCHANGE) as u16 {
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
            }
        }
        WM_NOTIFY => {
            if (*(lParam as *const NMHDR)).code as u32 == (PSN_APPLY) as u32 {
                if GetCurrencySetting(hwndDlg, pGlobalData) != 0 {
                    (*pGlobalData).bUserLocaleChanged = (TRUE) as i32;
                    UpdateExamples(hwndDlg, pGlobalData);
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
//  * FILE:            dll/cpl/intl/currency.c
//  * PURPOSE:         Currency property page
//  * PROGRAMMERS:     Eric Kohl
//  *                  Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "intl.h"
//
// #define POSITIVE_EXAMPLE   L"123456789.00"
// #define NEGATIVE_EXAMPLE   L"-123456789.00"
//
//
// static VOID
// UpdateExamples(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     WCHAR szBuffer[MAX_FMT_SIZE];
//     CURRENCYFMTW CurrencyFormat;
//
//     CurrencyFormat.NumDigits = pGlobalData->nCurrDigits;
//     CurrencyFormat.LeadingZero = pGlobalData->nNumLeadingZero;
//     CurrencyFormat.Grouping = GroupingFormats[pGlobalData->nCurrGrouping].nInteger;
//     CurrencyFormat.lpDecimalSep = pGlobalData->szCurrDecimalSep;
//     CurrencyFormat.lpThousandSep = pGlobalData->szCurrThousandSep;
//     CurrencyFormat.NegativeOrder = pGlobalData->nCurrNegFormat;
//     CurrencyFormat.PositiveOrder = pGlobalData->nCurrPosFormat;
//     CurrencyFormat.lpCurrencySymbol = pGlobalData->szCurrSymbol;
//
//     /* Positive example */
//     GetCurrencyFormatW(pGlobalData->UserLCID, 0,
//                        POSITIVE_EXAMPLE,
//                        &CurrencyFormat, szBuffer, MAX_FMT_SIZE);
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYPOSSAMPLE, WM_SETTEXT, 0, (LPARAM)szBuffer);
//
//     /* Negative example */
//     GetCurrencyFormatW(pGlobalData->UserLCID, 0,
//                        NEGATIVE_EXAMPLE,
//                        &CurrencyFormat, szBuffer, MAX_FMT_SIZE);
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYNEGSAMPLE, WM_SETTEXT, 0, (LPARAM)szBuffer);
// }
//
//
// static VOID
// InitCurrencySymbols(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYSYMBOL,
//                         CB_LIMITTEXT,
//                         MAX_CURRSYMBOL - 1,
//                         0);
//
//     /* Set currency symbols */
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYSYMBOL,
//                         CB_ADDSTRING,
//                         0,
//                         (LPARAM)pGlobalData->szCurrSymbol);
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYSYMBOL,
//                         CB_SETCURSEL,
//                         0, /* Index */
//                         0);
// }
//
//
// static VOID
// InitCurrencyPositiveFormats(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     WCHAR szBuffer[MAX_FMT_SIZE];
//     CURRENCYFMTW cyFmt;
//     INT i;
//
//     /* positive currency values */
//     cyFmt.NumDigits = pGlobalData->nCurrDigits;
//     cyFmt.LeadingZero = 0;
//     cyFmt.Grouping = 3;
//     cyFmt.lpDecimalSep = pGlobalData->szCurrDecimalSep;
//     cyFmt.lpThousandSep = pGlobalData->szCurrThousandSep;
//     cyFmt.lpCurrencySymbol = pGlobalData->szCurrSymbol;
//     cyFmt.NegativeOrder = 0;
//
//     for (i = 0; i < 4; i++)
//     {
//         cyFmt.PositiveOrder = i;
//         GetCurrencyFormatW(pGlobalData->UserLCID, 0,
//                            L"1.1",
//                            &cyFmt, szBuffer, MAX_FMT_SIZE);
//
//         SendDlgItemMessageW(hwndDlg, IDC_CURRENCYPOSVALUE,
//                             CB_INSERTSTRING,
//                             -1,
//                             (LPARAM)szBuffer);
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYPOSVALUE,
//                         CB_SETCURSEL,
//                         pGlobalData->nCurrPosFormat,
//                         0);
// }
//
//
// static VOID
// InitCurrencyNegativeFormats(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     WCHAR szBuffer[MAX_FMT_SIZE];
//     CURRENCYFMTW cyFmt;
//     int i;
//
//     /* negative currency values */
//     cyFmt.NumDigits = pGlobalData->nCurrDigits;
//     cyFmt.LeadingZero = 0;
//     cyFmt.Grouping = 3;
//     cyFmt.lpDecimalSep = pGlobalData->szCurrDecimalSep;
//     cyFmt.lpThousandSep = pGlobalData->szCurrThousandSep;
//     cyFmt.lpCurrencySymbol = pGlobalData->szCurrSymbol;
//     cyFmt.PositiveOrder = 0;
//
//     for (i = 0; i < 16; i++)
//     {
//         cyFmt.NegativeOrder = i;
//         GetCurrencyFormatW(pGlobalData->UserLCID, 0,
//                            L"-1.1",
//                            &cyFmt, szBuffer, MAX_FMT_SIZE);
//
//         SendDlgItemMessageW(hwndDlg, IDC_CURRENCYNEGVALUE,
//                             CB_INSERTSTRING,
//                             -1,
//                             (LPARAM)szBuffer);
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYNEGVALUE,
//                         CB_SETCURSEL,
//                         pGlobalData->nCurrNegFormat,
//                         0);
// }
//
//
// static VOID
// InitCurrencyDecimalSeparators(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYDECSEP,
//                         CB_LIMITTEXT,
//                         MAX_CURRDECIMALSEP - 1,
//                         0);
//
//     /* Decimal separator */
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYDECSEP,
//                         CB_ADDSTRING,
//                         0,
//                         (LPARAM)pGlobalData->szCurrDecimalSep);
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYDECSEP,
//                         CB_SETCURSEL,
//                         0, /* Index */
//                         0);
// }
//
//
// /* Initialize the number of fractional digits */
// static VOID
// InitCurrencyNumFracDigits(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     WCHAR szBuffer[MAX_FMT_SIZE];
//     int i;
//
//     /* Create standard list of fractional symbols */
//     for (i = 0; i < 10; i++)
//     {
//         szBuffer[0] = L'0' + i;
//         szBuffer[1] = 0;
//         SendDlgItemMessageW(hwndDlg, IDC_CURRENCYDECNUM,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)szBuffer);
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYDECNUM,
//                         CB_SETCURSEL,
//                         pGlobalData->nCurrDigits,
//                         0);
// }
//
//
// /* Initialize the list of group separators */
// static VOID
// InitCurrencyGroupSeparators(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYGRPSEP,
//                         CB_LIMITTEXT,
//                         MAX_CURRTHOUSANDSEP - 1,
//                         0);
//
//     /* Digit group separator */
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYGRPSEP,
//                         CB_ADDSTRING,
//                         0,
//                         (LPARAM)pGlobalData->szCurrThousandSep);
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYGRPSEP,
//                         CB_SETCURSEL,
//                         0, /* Index */
//                         0);
// }
//
//
// static VOID
// InitDigitGroupCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     WCHAR szBuffer[MAX_FMT_SIZE];
//     CURRENCYFMTW cyFmt;
//     INT i;
//
//     /* Digit grouping */
//     cyFmt.NumDigits = 0;
//     cyFmt.LeadingZero = 0;
//     cyFmt.lpDecimalSep = L"";
//     cyFmt.lpThousandSep = pGlobalData->szCurrThousandSep;
//     cyFmt.PositiveOrder = 0;
//     cyFmt.NegativeOrder = 0;
//     cyFmt.lpCurrencySymbol = L"";
//
//     for (i = 0 ; i < MAX_GROUPINGFORMATS ; i++)
//     {
//        cyFmt.Grouping = GroupingFormats[i].nInteger;
//
//        GetCurrencyFormatW(pGlobalData->UserLCID, 0,
//                           L"123456789",
//                           &cyFmt, szBuffer, MAX_FMT_SIZE);
//        SendDlgItemMessageW(hwndDlg, IDC_CURRENCYGRPNUM,
//                            CB_INSERTSTRING,
//                            -1,
//                            (LPARAM)szBuffer);
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_CURRENCYGRPNUM,
//                         CB_SETCURSEL,
//                         pGlobalData->nCurrGrouping,
//                         0);
// }
//
//
// static
// BOOL
// GetCurrencySetting(
//      HWND hwndDlg,
//      PGLOBALDATA pGlobalData)
// {
//     WCHAR szCurrSymbol[MAX_CURRSYMBOL];
//     WCHAR szCurrDecimalSep[MAX_CURRDECIMALSEP];
//     WCHAR szCurrThousandSep[MAX_CURRTHOUSANDSEP];
//     INT nCurrPosFormat;
//     INT nCurrNegFormat;
//     INT nCurrDigits;
//     INT nCurrGrouping;
//
//     /* Currency symbol */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_CURRENCYSYMBOL,
//                             szCurrSymbol,
//                             MAX_CURRSYMBOL);
//
//     if (szCurrSymbol[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Positive Amount */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_CURRENCYPOSVALUE,
//                              &nCurrPosFormat);
//
//     /* Negative Amount */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_CURRENCYNEGVALUE,
//                              &nCurrNegFormat);
//
//     /* Decimal separator */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_CURRENCYDECSEP,
//                             szCurrDecimalSep,
//                             MAX_CURRDECIMALSEP);
//
//     if (szCurrDecimalSep[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Number of fractional digits */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_CURRENCYDECNUM,
//                              &nCurrDigits);
//
//     /* Grouping symbol */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_CURRENCYGRPSEP,
//                             szCurrThousandSep,
//                             MAX_CURRTHOUSANDSEP);
//
//     if (szCurrThousandSep[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Digit grouping */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_CURRENCYGRPNUM,
//                              &nCurrGrouping);
//
//     /* Store settings in global data */
//     wcscpy(pGlobalData->szCurrSymbol, szCurrSymbol);
//     pGlobalData->nCurrGrouping = nCurrGrouping;
//     wcscpy(pGlobalData->szCurrDecimalSep, szCurrDecimalSep);
//     wcscpy(pGlobalData->szCurrThousandSep, szCurrThousandSep);
//     pGlobalData->nCurrPosFormat = nCurrPosFormat;
//     pGlobalData->nCurrNegFormat = nCurrNegFormat;
//     pGlobalData->nCurrDigits = nCurrDigits;
//
//     return TRUE;
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// CurrencyPageProc(HWND hwndDlg,
//                  UINT uMsg,
//                  WPARAM wParam,
//                  LPARAM lParam)
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
//             InitCurrencySymbols(hwndDlg, pGlobalData);
//             InitCurrencyPositiveFormats(hwndDlg, pGlobalData);
//             InitCurrencyNegativeFormats(hwndDlg, pGlobalData);
//             InitCurrencyDecimalSeparators(hwndDlg, pGlobalData);
//             InitCurrencyNumFracDigits(hwndDlg, pGlobalData);
//             InitCurrencyGroupSeparators(hwndDlg, pGlobalData);
//             InitDigitGroupCB(hwndDlg, pGlobalData);
//             UpdateExamples(hwndDlg, pGlobalData);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_CURRENCYSYMBOL:
//                 case IDC_CURRENCYPOSVALUE:
//                 case IDC_CURRENCYNEGVALUE:
//                 case IDC_CURRENCYDECSEP:
//                 case IDC_CURRENCYDECNUM:
//                 case IDC_CURRENCYGRPSEP:
//                 case IDC_CURRENCYGRPNUM:
//                     if (HIWORD(wParam) == CBN_SELCHANGE ||
//                         HIWORD(wParam) == CBN_EDITCHANGE)
//                     {
//                         /* Enable the Apply button */
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//             }
//             break;
//
//         case WM_NOTIFY:
//             if (((LPNMHDR)lParam)->code == (UINT)PSN_APPLY)
//             {
//                 if (GetCurrencySetting(hwndDlg, pGlobalData))
//                 {
//                     pGlobalData->bUserLocaleChanged = TRUE;
//                     UpdateExamples(hwndDlg, pGlobalData);
//                 }
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

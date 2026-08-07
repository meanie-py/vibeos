// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const SAMPLE_NUMBER: PCWSTR = unsafe {
    w!("123456789")
};

pub const SAMPLE_NEG_NUMBER: PCWSTR = unsafe {
    w!("-123456789")
};

pub const MAX_NUM_SEP_SAMPLES: usize = 2;

pub const MAX_FRAC_NUM_SAMPLES: usize = 10;

pub const MAX_FIELD_SEP_SAMPLES: usize = 1;

pub const MAX_FIELD_DIG_SAMPLES: usize = 3;

pub const MAX_NEG_SIGN_SAMPLES: usize = 1;

pub const MAX_NEG_NUMBERS_SAMPLES: usize = 5;

pub const MAX_LEAD_ZEROES_SAMPLES: usize = 2;

pub const MAX_LIST_SEP_SAMPLES: usize = 1;

pub const MAX_UNITS_SYS_SAMPLES: usize = 2;

pub static mut lpNumSepSamples: [PWSTR; 2] = unsafe {
    [
        w!(",") as PWSTR,
        w!(".") as PWSTR,
    ]
};

pub static mut lpFieldSepSamples: [PWSTR; 1] = unsafe {
    [
        w!(" ") as PWSTR,
    ]
};

pub static mut lpFieldDigNumSamples: [PWSTR; 3] = unsafe {
    [
        w!("0;0") as PWSTR,
        w!("3;0") as PWSTR,
        w!("3;2;0") as PWSTR,
    ]
};

pub static mut lpNegSignSamples: [PWSTR; 1] = unsafe {
    [
        w!("-") as PWSTR,
    ]
};

pub static mut lpNegNumFmtSamples: [PWSTR; 5] = unsafe {
    [
        w!("(1,1)") as PWSTR,
        w!("-1,1") as PWSTR,
        w!("- 1,1") as PWSTR,
        w!("1,1-") as PWSTR,
        w!("1,1 -") as PWSTR,
    ]
};

pub static mut lpLeadNumFmtSamples: [PWSTR; 2] = unsafe {
    [
        w!(",7") as PWSTR,
        w!("0,7") as PWSTR,
    ]
};

pub static mut lpListSepSamples: [PWSTR; 1] = unsafe {
    [
        w!(";") as PWSTR,
    ]
};
unsafe extern "C" fn InitNumDecimalSepCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut nCBIndex: i32;
    let nRetCode: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERDSYMBOL) as i32,
                        CB_LIMITTEXT,
                        (MAX_NUMDECIMALSEP - 1) as WPARAM,
                        0);

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERDSYMBOL) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < (MAX_NUM_SEP_SAMPLES) as i32 {
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERDSYMBOL) as i32,
                            CB_ADDSTRING,
                            0,
                            lpNumSepSamples[nCBIndex as usize] as LPARAM);
        nCBIndex += 1;
    }

    nRetCode = (SendDlgItemMessageW(hwndDlg, (IDC_NUMBERDSYMBOL) as i32,
                                   CB_SELECTSTRING,
                                   -1i32 as WPARAM,
                                   (*pGlobalData).szNumDecimalSep.as_ptr() as LPARAM)) as i32;

    if nRetCode == CB_ERR {
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERDSYMBOL) as i32,
                            CB_ADDSTRING,
                            (MAX_NUM_SEP_SAMPLES) as WPARAM,
                            (*pGlobalData).szNumDecimalSep.as_ptr() as LPARAM);
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERDSYMBOL) as i32,
                            CB_SELECTSTRING,
                            -1i32 as WPARAM,
                            (*pGlobalData).szNumDecimalSep.as_ptr() as LPARAM);
    }
}
unsafe extern "C" fn InitNumOfFracSymbCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut szFracCount: [u16; (MAX_SAMPLES_STR_SIZE) as usize] = [0; (MAX_SAMPLES_STR_SIZE) as usize];
    let mut nCBIndex: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNDIGDEC) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < (MAX_FRAC_NUM_SAMPLES) as i32 {
        _itow(nCBIndex, szFracCount.as_mut_ptr(), (DECIMAL_RADIX) as i32);

        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNDIGDEC) as i32,
                            CB_ADDSTRING,
                            0,
                            szFracCount.as_ptr() as LPARAM);

        nCBIndex += 1;
    }

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNDIGDEC) as i32,
                        CB_SETCURSEL,
                        (*pGlobalData).nNumDigits as WPARAM,
                        0 as LPARAM);
}
unsafe extern "C" fn InitNumFieldSepCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut nCBIndex: i32;
    let mut nRetCode: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDIGITGRSYM) as i32,
                        CB_LIMITTEXT,
                        (MAX_NUMTHOUSANDSEP - 1) as WPARAM,
                        0);

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDIGITGRSYM) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < (MAX_FIELD_SEP_SAMPLES) as i32 {
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDIGITGRSYM) as i32,
                            CB_ADDSTRING,
                            0,
                            lpFieldSepSamples[nCBIndex as usize] as LPARAM);
        nCBIndex += 1;
    }

    nRetCode = (SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDIGITGRSYM) as i32,
                                   CB_SELECTSTRING,
                                   -1i32 as WPARAM,
                                   (*pGlobalData).szNumThousandSep.as_ptr() as LPARAM)) as i32;

    if nRetCode == CB_ERR {
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDIGITGRSYM) as i32,
                            CB_ADDSTRING,
                            0,
                            (*pGlobalData).szNumThousandSep.as_ptr() as LPARAM);
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDIGITGRSYM) as i32,
                            CB_SELECTSTRING,
                            -1i32 as WPARAM,
                            (*pGlobalData).szNumThousandSep.as_ptr() as LPARAM);
    }
}
unsafe extern "C" fn InitFieldDigNumCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut pszFieldDigNumSmpl: PWSTR;
    let mut nCBIndex: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDGROUPING) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < (MAX_FIELD_DIG_SAMPLES) as i32 {
        pszFieldDigNumSmpl = InsSpacesFmt(SAMPLE_NUMBER, lpFieldDigNumSamples[nCBIndex as usize]);
        if !pszFieldDigNumSmpl.is_null() {
            SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDGROUPING) as i32,
                                CB_ADDSTRING,
                                0,
                                pszFieldDigNumSmpl as LPARAM);
            HeapFree(GetProcessHeap(), 0, pszFieldDigNumSmpl as *mut _);
        }
        nCBIndex += 1;
    }

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDGROUPING) as i32,
                        CB_SETCURSEL,
                        (*pGlobalData).nNumGrouping as WPARAM,
                        0 as LPARAM);
}
unsafe extern "C" fn InitNegSignCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut nCBIndex: i32;
    let nRetCode: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNSIGNSYM) as i32,
                        CB_LIMITTEXT,
                        (MAX_NUMNEGATIVESIGN - 1) as WPARAM,
                        0);

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNSIGNSYM) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < (MAX_NEG_SIGN_SAMPLES) as i32 {
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNSIGNSYM) as i32,
                            CB_ADDSTRING,
                            0,
                            lpNegSignSamples[nCBIndex as usize] as LPARAM);
        nCBIndex += 1;
    }

    nRetCode = (SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNSIGNSYM) as i32,
                                   CB_SELECTSTRING,
                                   -1i32 as WPARAM,
                                   (*pGlobalData).szNumNegativeSign.as_ptr() as LPARAM)) as i32;

    if nRetCode == CB_ERR {
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNSIGNSYM) as i32,
                            CB_ADDSTRING,
                            0,
                            (*pGlobalData).szNumNegativeSign.as_ptr() as LPARAM);
        SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNSIGNSYM) as i32,
                            CB_SELECTSTRING,
                            -1i32 as WPARAM,
                            (*pGlobalData).szNumNegativeSign.as_ptr() as LPARAM);
    }
}
unsafe extern "C" fn InitNegNumFmtCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut pszString1: PWSTR;
    let mut pszString2: PWSTR;
    let mut nCBIndex: INT;

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNNUMFORMAT) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < (MAX_NEG_NUMBERS_SAMPLES) as i32 {
        pszString1 = ReplaceSubStr(lpNegNumFmtSamples[nCBIndex as usize],
                                   (*pGlobalData).szNumDecimalSep.as_mut_ptr(),
                                   w!(","));
        if pszString1 != core::ptr::null_mut() {
            pszString2 = ReplaceSubStr(pszString1,
                                       (*pGlobalData).szNumNegativeSign.as_mut_ptr(),
                                       w!("-"));
            if pszString2 != core::ptr::null_mut() {
                SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNNUMFORMAT) as i32,
                                    CB_ADDSTRING,
                                    0,
                                    pszString2 as LPARAM);

                HeapFree(GetProcessHeap(), 0, (pszString2).cast());
            }

            HeapFree(GetProcessHeap(), 0, (pszString1).cast());
        }

        nCBIndex += 1;
    }

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSNNUMFORMAT) as i32,
                        CB_SETCURSEL,
                        (*pGlobalData).nNumNegFormat as WPARAM,
                        0 as LPARAM);
}
unsafe extern "C" fn InitLeadingZeroesCB(hwndDlg: HWND, pGlobalData: *mut GLOBALDATA) {
    let mut pszResultStr: PWSTR;
    let mut nCBIndex: i32;

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDISPLEADZER) as i32,
                        CB_RESETCONTENT,
                        0 as WPARAM,
                        0 as LPARAM);

    nCBIndex = 0;
    while nCBIndex < (MAX_LEAD_ZEROES_SAMPLES) as i32 {
        pszResultStr = ReplaceSubStr(lpLeadNumFmtSamples[nCBIndex as usize],
                                     (*pGlobalData).szNumDecimalSep.as_mut_ptr(),
                                     w!(","));
        if pszResultStr != core::ptr::null_mut() {
            SendDlgItemMessage(hwndDlg, (IDC_NUMBERSDISPLEADZER) as i32,
                               CB_ADDSTRING,
                               0 as WPARAM,
                               pszResultStr as LPARAM);
            HeapFree(GetProcessHeap(), 0, (pszResultStr).cast());
        }
        nCBIndex += 1;
    }

    SendDlgItemMessageW(hwndDlg, (IDC_NUMBERSDISPLEADZER) as i32,
                        CB_SETCURSEL,
                        (*pGlobalData).nNumLeadingZero as WPARAM,
                        0 as LPARAM);
}
unsafe extern "C" fn InitListSepCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut nCBIndex: INT;
    let mut nRetCode: INT;

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_NUMBERSLSEP) as i32,
        CB_LIMITTEXT,
        (MAX_NUMLISTSEP - 1) as WPARAM,
        0,
    );

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_NUMBERSLSEP) as i32,
        CB_RESETCONTENT,
        0 as WPARAM,
        0 as LPARAM,
    );

    nCBIndex = 0;
    while nCBIndex < (MAX_LIST_SEP_SAMPLES) as i32 {
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_NUMBERSLSEP) as i32,
            CB_ADDSTRING,
            0,
            lpListSepSamples[nCBIndex as usize] as LPARAM,
        );
        nCBIndex += 1;
    }

    nRetCode = (SendDlgItemMessageW(
        hwndDlg,
        (IDC_NUMBERSLSEP) as i32,
        CB_SELECTSTRING,
        -1i32 as WPARAM,
        (*pGlobalData).szNumListSep.as_ptr() as LPARAM,
    )) as i32;

    if nRetCode == CB_ERR {
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_NUMBERSLSEP) as i32,
            CB_ADDSTRING,
            0,
            (*pGlobalData).szNumListSep.as_ptr() as LPARAM,
        );
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_NUMBERSLSEP) as i32,
            CB_SELECTSTRING,
            -1i32 as WPARAM,
            (*pGlobalData).szNumListSep.as_ptr() as LPARAM,
        );
    }
}
unsafe extern "C" fn InitUnitsSysCB(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut szUnitName: [u16; 128] = [0; 128];
    let mut nCBIndex: i32;

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_NUMBERSMEASSYS) as i32,
        CB_RESETCONTENT,
        0 as WPARAM,
        0 as LPARAM,
    );

    nCBIndex = 0;
    while nCBIndex < (MAX_UNITS_SYS_SAMPLES) as i32 {
        LoadStringW(
            hApplet,
            IDS_METRIC + nCBIndex as u32,
            szUnitName.as_mut_ptr(),
            128,
        );

        SendDlgItemMessageW(
            hwndDlg,
            (IDC_NUMBERSMEASSYS) as i32,
            CB_ADDSTRING,
            0,
            szUnitName.as_ptr() as LPARAM,
        );

        nCBIndex += 1;
    }

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_NUMBERSMEASSYS) as i32,
        CB_SETCURSEL,
        (*pGlobalData).nNumMeasure as WPARAM,
        0 as LPARAM,
    );
}
unsafe extern "C" fn UpdateNumSamples(hwndDlg: HWND, pGlobalData: PGLOBALDATA) {
    let mut OutBuffer: [u16; (MAX_FMT_SIZE) as usize] = [0; (MAX_FMT_SIZE) as usize];
    let mut NumberFormat: NUMBERFMT = core::mem::zeroed();

    NumberFormat.NumDigits = ((*pGlobalData).nNumDigits) as u32;
    NumberFormat.LeadingZero = ((*pGlobalData).nNumLeadingZero) as u32;
    NumberFormat.Grouping = GroupingFormats[(*pGlobalData).nNumGrouping as usize].nInteger;
    NumberFormat.lpDecimalSep = (*pGlobalData).szNumDecimalSep.as_mut_ptr();
    NumberFormat.lpThousandSep = (*pGlobalData).szNumThousandSep.as_mut_ptr();
    NumberFormat.NegativeOrder = ((*pGlobalData).nNumNegFormat) as u32;

    GetNumberFormatW(
        (*pGlobalData).UserLCID,
        0,
        SAMPLE_NUMBER,
        &mut NumberFormat,
        OutBuffer.as_mut_ptr(),
        (MAX_FMT_SIZE) as i32,
    );

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_NUMBERSPOSSAMPLE) as i32,
        WM_SETTEXT,
        0,
        OutBuffer.as_ptr() as LPARAM,
    );

    GetNumberFormatW(
        (*pGlobalData).UserLCID,
        0,
        SAMPLE_NEG_NUMBER,
        &mut NumberFormat,
        OutBuffer.as_mut_ptr(),
        (MAX_FMT_SIZE) as i32,
    );

    SendDlgItemMessageW(
        hwndDlg,
        (IDC_NUMBERSNEGSAMPLE) as i32,
        WM_SETTEXT,
        0,
        OutBuffer.as_ptr() as LPARAM,
    );
}
unsafe extern "C" fn GetNumberSetting(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA) -> BOOL {
    let mut szNumDecimalSep: [u16; (MAX_NUMDECIMALSEP) as usize] = [0; (MAX_NUMDECIMALSEP) as usize];
    let mut szNumThousandSep: [u16; (MAX_NUMTHOUSANDSEP) as usize] = [0; (MAX_NUMTHOUSANDSEP) as usize];
    let mut szNumNegativeSign: [u16; (MAX_NUMNEGATIVESIGN) as usize] = [0; (MAX_NUMNEGATIVESIGN) as usize];
    let mut szNumListSep: [u16; (MAX_NUMLISTSEP) as usize] = [0; (MAX_NUMLISTSEP) as usize];
    let mut nNumDigits: i32 = 0;
    let mut nNumGrouping: i32 = 0;
    let mut nNumNegFormat: i32 = 0;
    let mut nNumLeadingZero: i32 = 0;
    let mut nNumMeasure: i32 = 0;

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_NUMBERDSYMBOL) as INT,
                            szNumDecimalSep.as_mut_ptr(),
                            MAX_NUMDECIMALSEP);

    if szNumDecimalSep[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxIndex(hwndDlg,
                             (IDC_NUMBERSNDIGDEC) as INT,
                             core::ptr::addr_of_mut!(nNumDigits));

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_NUMBERSDIGITGRSYM) as INT,
                            szNumThousandSep.as_mut_ptr(),
                            MAX_NUMTHOUSANDSEP);

    if szNumThousandSep[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxIndex(hwndDlg,
                             (IDC_NUMBERSDGROUPING) as INT,
                             core::ptr::addr_of_mut!(nNumGrouping));

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_NUMBERSNSIGNSYM) as INT,
                            szNumNegativeSign.as_mut_ptr(),
                            MAX_NUMNEGATIVESIGN);

    if szNumNegativeSign[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxIndex(hwndDlg,
                             (IDC_NUMBERSNNUMFORMAT) as INT,
                             core::ptr::addr_of_mut!(nNumNegFormat));

    GetSelectedComboBoxIndex(hwndDlg,
                             (IDC_NUMBERSDISPLEADZER) as INT,
                             core::ptr::addr_of_mut!(nNumLeadingZero));

    GetSelectedComboBoxText(hwndDlg,
                            (IDC_NUMBERSLSEP) as INT,
                            szNumListSep.as_mut_ptr(),
                            MAX_NUMLISTSEP);

    if szNumListSep[0] == 0 {
        return (FALSE) as i32;
    }

    GetSelectedComboBoxIndex(hwndDlg,
                             (IDC_NUMBERSMEASSYS) as INT,
                             core::ptr::addr_of_mut!(nNumMeasure));

    wcscpy((*pGlobalData).szNumDecimalSep.as_mut_ptr(), szNumDecimalSep.as_ptr());
    wcscpy((*pGlobalData).szNumThousandSep.as_mut_ptr(), szNumThousandSep.as_ptr());
    wcscpy((*pGlobalData).szNumNegativeSign.as_mut_ptr(), szNumNegativeSign.as_ptr());
    wcscpy((*pGlobalData).szNumListSep.as_mut_ptr(), szNumListSep.as_ptr());
    (*pGlobalData).nNumGrouping = nNumGrouping;
    (*pGlobalData).nNumDigits = nNumDigits;
    (*pGlobalData).nNumNegFormat = nNumNegFormat;
    (*pGlobalData).nNumLeadingZero = nNumLeadingZero;
    (*pGlobalData).nNumMeasure = nNumMeasure;

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn NumbersPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBALDATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBALDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            InitNumDecimalSepCB(hwndDlg, pGlobalData);
            InitNumOfFracSymbCB(hwndDlg, pGlobalData);
            InitNumFieldSepCB(hwndDlg, pGlobalData);
            InitFieldDigNumCB(hwndDlg, pGlobalData);
            InitNegSignCB(hwndDlg, pGlobalData);
            InitNegNumFmtCB(hwndDlg, pGlobalData);
            InitLeadingZeroesCB(hwndDlg, pGlobalData);
            InitListSepCB(hwndDlg, pGlobalData);
            InitUnitsSysCB(hwndDlg, pGlobalData);
            UpdateNumSamples(hwndDlg, pGlobalData);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_NUMBERDSYMBOL | IDC_NUMBERSNDIGDEC | IDC_NUMBERSDIGITGRSYM
                | IDC_NUMBERSDGROUPING | IDC_NUMBERSNSIGNSYM | IDC_NUMBERSNNUMFORMAT
                | IDC_NUMBERSDISPLEADZER | IDC_NUMBERSLSEP | IDC_NUMBERSMEASSYS => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 || HIWORD!(wParam) == (CBN_EDITCHANGE) as u16 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).code == PSN_APPLY as UINT {
                if GetNumberSetting(hwndDlg, pGlobalData) != 0 {
                    (*pGlobalData).bUserLocaleChanged = (TRUE) as i32;
                    UpdateNumSamples(hwndDlg, pGlobalData);
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
//  * FILE:            dll/cpl/intl/numbers.c
//  * PURPOSE:         Numbers property page
//  * PROGRAMMERS:     Eric Kohl
//  *                  Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "intl.h"
//
// #define SAMPLE_NUMBER               L"123456789"
// #define SAMPLE_NEG_NUMBER           L"-123456789"
// #define MAX_NUM_SEP_SAMPLES         2
// #define MAX_FRAC_NUM_SAMPLES        10
// #define MAX_FIELD_SEP_SAMPLES       1
// #define MAX_FIELD_DIG_SAMPLES       3
// #define MAX_NEG_SIGN_SAMPLES        1
// #define MAX_NEG_NUMBERS_SAMPLES     5
// #define MAX_LEAD_ZEROES_SAMPLES     2
// #define MAX_LIST_SEP_SAMPLES        1
// #define MAX_UNITS_SYS_SAMPLES       2
//
// static PWSTR lpNumSepSamples[MAX_NUM_SEP_SAMPLES] =
//     {L",", L"."};
// static PWSTR lpFieldSepSamples[MAX_FIELD_SEP_SAMPLES] =
//     {L" "};
// static PWSTR lpFieldDigNumSamples[MAX_FIELD_DIG_SAMPLES] =
//     {L"0;0", L"3;0", L"3;2;0"};
// static PWSTR lpNegSignSamples[MAX_NEG_SIGN_SAMPLES] =
//     {L"-"};
// static PWSTR lpNegNumFmtSamples[MAX_NEG_NUMBERS_SAMPLES] =
//     {L"(1,1)", L"-1,1", L"- 1,1", L"1,1-", L"1,1 -"};
// static PWSTR lpLeadNumFmtSamples[MAX_LEAD_ZEROES_SAMPLES] =
//     {L",7", L"0,7"};
// static PWSTR lpListSepSamples[MAX_LIST_SEP_SAMPLES] =
//     {L";"};
//
//
// /* Init num decimal separator control box */
// static VOID
// InitNumDecimalSepCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     INT nCBIndex;
//     INT nRetCode;
//
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERDSYMBOL,
//                         CB_LIMITTEXT,
//                         MAX_NUMDECIMALSEP - 1,
//                         0);
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERDSYMBOL,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of decimal separators */
//     for (nCBIndex = 0; nCBIndex < MAX_NUM_SEP_SAMPLES; nCBIndex++)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERDSYMBOL,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)lpNumSepSamples[nCBIndex]);
//     }
//
//     /* Set current item to value from registry */
//     nRetCode = SendDlgItemMessageW(hwndDlg, IDC_NUMBERDSYMBOL,
//                                    CB_SELECTSTRING,
//                                    -1,
//                                    (LPARAM)pGlobalData->szNumDecimalSep);
//
//     /* If it is not successful, add new values to list and select them */
//     if (nRetCode == CB_ERR)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERDSYMBOL,
//                             CB_ADDSTRING,
//                             MAX_NUM_SEP_SAMPLES,
//                             (LPARAM)pGlobalData->szNumDecimalSep);
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERDSYMBOL,
//                             CB_SELECTSTRING,
//                             -1,
//                             (LPARAM)pGlobalData->szNumDecimalSep);
//     }
// }
//
// /* Init number of fractional symbols control box */
// static VOID
// InitNumOfFracSymbCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     WCHAR szFracCount[MAX_SAMPLES_STR_SIZE];
//     INT nCBIndex;
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNDIGDEC,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of fractional symbols */
//     for (nCBIndex = 0; nCBIndex < MAX_FRAC_NUM_SAMPLES; nCBIndex++)
//     {
//         /* Convert to wide char */
//         _itow(nCBIndex, szFracCount, DECIMAL_RADIX);
//
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNDIGDEC,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)szFracCount);
//     }
//
//     /* Set current item to value from registry */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNDIGDEC,
//                         CB_SETCURSEL,
//                         (WPARAM)pGlobalData->nNumDigits,
//                         (LPARAM)0);
// }
//
// /* Init field separator control box */
// static VOID
// InitNumFieldSepCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     INT nCBIndex;
//     INT nRetCode;
//
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDIGITGRSYM,
//                         CB_LIMITTEXT,
//                         MAX_NUMTHOUSANDSEP - 1,
//                         0);
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDIGITGRSYM,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of field separators */
//     for (nCBIndex = 0; nCBIndex < MAX_FIELD_SEP_SAMPLES; nCBIndex++)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDIGITGRSYM,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)lpFieldSepSamples[nCBIndex]);
//     }
//
//     /* Set current item to value from registry */
//     nRetCode = SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDIGITGRSYM,
//                                    CB_SELECTSTRING,
//                                    -1,
//                                    (LPARAM)pGlobalData->szNumThousandSep);
//
//     /* If it is not success, add new values to list and select them */
//     if (nRetCode == CB_ERR)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDIGITGRSYM,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)pGlobalData->szNumThousandSep);
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDIGITGRSYM,
//                             CB_SELECTSTRING,
//                             -1,
//                             (LPARAM)pGlobalData->szNumThousandSep);
//     }
// }
//
// /* Init number of digits in field control box */
// static VOID
// InitFieldDigNumCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     PWSTR pszFieldDigNumSmpl;
//     INT nCBIndex;
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDGROUPING,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of field digits num */
//     for (nCBIndex = 0; nCBIndex < MAX_FIELD_DIG_SAMPLES; nCBIndex++)
//     {
//         pszFieldDigNumSmpl = InsSpacesFmt(SAMPLE_NUMBER, lpFieldDigNumSamples[nCBIndex]);
//         if (pszFieldDigNumSmpl != NULL)
//         {
//             SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDGROUPING,
//                                 CB_ADDSTRING,
//                                 0,
//                                 (LPARAM)pszFieldDigNumSmpl);
//             HeapFree(GetProcessHeap(), 0, pszFieldDigNumSmpl);
//         }
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDGROUPING,
//                         CB_SETCURSEL,
//                         (WPARAM)pGlobalData->nNumGrouping,
//                         (LPARAM)0);
// }
//
// /* Init negative sign control box */
// static VOID
// InitNegSignCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     INT nCBIndex;
//     INT nRetCode;
//
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNSIGNSYM,
//                         CB_LIMITTEXT,
//                         MAX_NUMNEGATIVESIGN - 1,
//                         0);
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNSIGNSYM,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of signs */
//     for (nCBIndex = 0; nCBIndex < MAX_NEG_SIGN_SAMPLES; nCBIndex++)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNSIGNSYM,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)lpNegSignSamples[nCBIndex]);
//     }
//
//     /* Set current item to value from registry */
//     nRetCode = SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNSIGNSYM,
//                                    CB_SELECTSTRING,
//                                    -1,
//                                    (LPARAM)pGlobalData->szNumNegativeSign);
//
//     /* If  it is not successful, add new values to list and select them */
//     if (nRetCode == CB_ERR)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNSIGNSYM,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)pGlobalData->szNumNegativeSign);
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNSIGNSYM,
//                             CB_SELECTSTRING,
//                             -1,
//                             (LPARAM)pGlobalData->szNumNegativeSign);
//     }
// }
//
// /* Init negative numbers format control box */
// static VOID
// InitNegNumFmtCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     PWSTR pszString1, pszString2;
//     INT nCBIndex;
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNNUMFORMAT,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of negative numbers formats */
//     for (nCBIndex = 0; nCBIndex < MAX_NEG_NUMBERS_SAMPLES; nCBIndex++)
//     {
//         /* Replace standard separator to setted */
//         pszString1 = ReplaceSubStr(lpNegNumFmtSamples[nCBIndex],
//                                    pGlobalData->szNumDecimalSep,
//                                    L",");
//         if (pszString1 != NULL)
//         {
//             /* Replace standard negative sign to setted */
//             pszString2 = ReplaceSubStr(pszString1,
//                                        pGlobalData->szNumNegativeSign,
//                                        L"-");
//             if (pszString2 != NULL)
//             {
//                 SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNNUMFORMAT,
//                                     CB_ADDSTRING,
//                                     0,
//                                     (LPARAM)pszString2);
//
//                 HeapFree(GetProcessHeap(), 0, pszString2);
//             }
//
//             HeapFree(GetProcessHeap(), 0, pszString1);
//         }
//     }
//
//     /* Set current item to value from registry */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNNUMFORMAT,
//                         CB_SETCURSEL,
//                         (WPARAM)pGlobalData->nNumNegFormat,
//                         (LPARAM)0);
// }
//
// /* Init leading zeroes control box */
// static VOID
// InitLeadingZeroesCB(HWND hwndDlg, PGLOBALDATA pGlobalData)
// {
//     PWSTR pszResultStr;
//     INT nCBIndex;
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDISPLEADZER,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create list of standard leading zeroes formats */
//     for (nCBIndex = 0; nCBIndex < MAX_LEAD_ZEROES_SAMPLES; nCBIndex++)
//     {
//         pszResultStr = ReplaceSubStr(lpLeadNumFmtSamples[nCBIndex],
//                                      pGlobalData->szNumDecimalSep,
//                                      L",");
//         if (pszResultStr != NULL)
//         {
//             SendDlgItemMessage(hwndDlg, IDC_NUMBERSDISPLEADZER,
//                                CB_ADDSTRING,
//                                0,
//                                (LPARAM)pszResultStr);
//             HeapFree(GetProcessHeap(), 0, pszResultStr);
//         }
//     }
//
//     /* Set current item to value from registry */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSDISPLEADZER,
//                         CB_SETCURSEL,
//                         (WPARAM)pGlobalData->nNumLeadingZero,
//                         (LPARAM)0);
// }
//
// static VOID
// InitListSepCB(HWND hwndDlg,
//               PGLOBALDATA pGlobalData)
// {
//     INT nCBIndex;
//     INT nRetCode;
//
//     /* Limit text length */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSLSEP,
//                         CB_LIMITTEXT,
//                         MAX_NUMLISTSEP - 1,
//                         0);
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSLSEP,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create standard list of signs */
//     for (nCBIndex = 0; nCBIndex < MAX_LIST_SEP_SAMPLES; nCBIndex++)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSLSEP,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)lpListSepSamples[nCBIndex]);
//     }
//
//     /* Set current item to value from registry */
//     nRetCode = SendDlgItemMessageW(hwndDlg, IDC_NUMBERSLSEP,
//                                    CB_SELECTSTRING,
//                                    -1,
//                                    (LPARAM)pGlobalData->szNumListSep);
//
//     /* If it is not successful, add new values to list and select them */
//     if (nRetCode == CB_ERR)
//     {
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSLSEP,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)pGlobalData->szNumListSep);
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSLSEP,
//                             CB_SELECTSTRING,
//                             -1,
//                             (LPARAM)pGlobalData->szNumListSep);
//     }
// }
//
// /* Init system of units control box */
// static VOID
// InitUnitsSysCB(HWND hwndDlg,
//                PGLOBALDATA pGlobalData)
// {
//     WCHAR szUnitName[128];
//     INT nCBIndex;
//
//     /* Clear all box content */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSMEASSYS,
//                         CB_RESETCONTENT,
//                         (WPARAM)0,
//                         (LPARAM)0);
//
//     /* Create list of standard system of units */
//     for (nCBIndex = 0; nCBIndex < MAX_UNITS_SYS_SAMPLES; nCBIndex++)
//     {
//         LoadStringW(hApplet, IDS_METRIC + nCBIndex, szUnitName, 128);
//
//         SendDlgItemMessageW(hwndDlg, IDC_NUMBERSMEASSYS,
//                             CB_ADDSTRING,
//                             0,
//                             (LPARAM)szUnitName);
//     }
//
//     /* Set current item to value from registry */
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSMEASSYS,
//                         CB_SETCURSEL,
//                         (WPARAM)pGlobalData->nNumMeasure,
//                         (LPARAM)0);
// }
//
// /* Update all numbers locale samples */
// static VOID
// UpdateNumSamples(HWND hwndDlg,
//                  PGLOBALDATA pGlobalData)
// {
//     WCHAR OutBuffer[MAX_FMT_SIZE];
//     NUMBERFMT NumberFormat;
//
//     NumberFormat.NumDigits = pGlobalData->nNumDigits;
//     NumberFormat.LeadingZero = pGlobalData->nNumLeadingZero;
//     NumberFormat.Grouping = GroupingFormats[pGlobalData->nNumGrouping].nInteger;
//     NumberFormat.lpDecimalSep = pGlobalData->szNumDecimalSep;
//     NumberFormat.lpThousandSep = pGlobalData->szNumThousandSep;
//     NumberFormat.NegativeOrder = pGlobalData->nNumNegFormat;
//
//     /* Get positive number format sample */
//     GetNumberFormatW(pGlobalData->UserLCID,
//                      0,
//                      SAMPLE_NUMBER,
//                      &NumberFormat,
//                      OutBuffer,
//                      MAX_FMT_SIZE);
//
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSPOSSAMPLE,
//                         WM_SETTEXT,
//                         0,
//                         (LPARAM)OutBuffer);
//
//     /* Get positive number format sample */
//     GetNumberFormatW(pGlobalData->UserLCID,
//                      0,
//                      SAMPLE_NEG_NUMBER,
//                      &NumberFormat,
//                      OutBuffer,
//                      MAX_FMT_SIZE);
//
//     SendDlgItemMessageW(hwndDlg, IDC_NUMBERSNEGSAMPLE,
//                         WM_SETTEXT,
//                         0,
//                         (LPARAM)OutBuffer);
// }
//
//
// static
// BOOL
// GetNumberSetting(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR szNumDecimalSep[MAX_NUMDECIMALSEP];
//     WCHAR szNumThousandSep[MAX_NUMTHOUSANDSEP];
//     WCHAR szNumNegativeSign[MAX_NUMNEGATIVESIGN];
//     WCHAR szNumListSep[MAX_NUMLISTSEP];
//     INT nNumDigits;
//     INT nNumGrouping;
//     INT nNumNegFormat;
//     INT nNumLeadingZero;
//     INT nNumMeasure;
//
//     /* Decimal symbol */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_NUMBERDSYMBOL,
//                             szNumDecimalSep,
//                             MAX_NUMDECIMALSEP);
//
//     if (szNumDecimalSep[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Number of digits after decimal */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_NUMBERSNDIGDEC,
//                              &nNumDigits);
//
//     /* Digit grouping symbol */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_NUMBERSDIGITGRSYM,
//                             szNumThousandSep,
//                             MAX_NUMTHOUSANDSEP);
//
//     if (szNumThousandSep[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Digit grouping */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_NUMBERSDGROUPING,
//                              &nNumGrouping);
//
//     /* Negative sign symbol */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_NUMBERSNSIGNSYM,
//                             szNumNegativeSign,
//                             MAX_NUMNEGATIVESIGN);
//
//     if (szNumNegativeSign[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Negative number format */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_NUMBERSNNUMFORMAT,
//                              &nNumNegFormat);
//
//     /* Display leading zeros */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_NUMBERSDISPLEADZER,
//                              &nNumLeadingZero);
//
//     /* List separator */
//     GetSelectedComboBoxText(hwndDlg,
//                             IDC_NUMBERSLSEP,
//                             szNumListSep,
//                             MAX_NUMLISTSEP);
//
//     if (szNumListSep[0] == L'\0')
//     {
//         /* TODO: Show error message */
//
//         return FALSE;
//     }
//
//     /* Measurement system */
//     GetSelectedComboBoxIndex(hwndDlg,
//                              IDC_NUMBERSMEASSYS,
//                              &nNumMeasure);
//
//     /* Store settings in global data */
//     wcscpy(pGlobalData->szNumDecimalSep, szNumDecimalSep);
//     wcscpy(pGlobalData->szNumThousandSep, szNumThousandSep);
//     wcscpy(pGlobalData->szNumNegativeSign, szNumNegativeSign);
//     wcscpy(pGlobalData->szNumListSep, szNumListSep);
//     pGlobalData->nNumGrouping = nNumGrouping;
//     pGlobalData->nNumDigits = nNumDigits;
//     pGlobalData->nNumNegFormat = nNumNegFormat;
//     pGlobalData->nNumLeadingZero = nNumLeadingZero;
//     pGlobalData->nNumMeasure = nNumMeasure;
//
//     return TRUE;
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// NumbersPageProc(HWND hwndDlg,
//                 UINT uMsg,
//                 WPARAM wParam,
//                 LPARAM lParam)
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
//             InitNumDecimalSepCB(hwndDlg, pGlobalData);
//             InitNumOfFracSymbCB(hwndDlg, pGlobalData);
//             InitNumFieldSepCB(hwndDlg, pGlobalData);
//             InitFieldDigNumCB(hwndDlg, pGlobalData);
//             InitNegSignCB(hwndDlg, pGlobalData);
//             InitNegNumFmtCB(hwndDlg, pGlobalData);
//             InitLeadingZeroesCB(hwndDlg, pGlobalData);
//             InitListSepCB(hwndDlg, pGlobalData);
//             InitUnitsSysCB(hwndDlg, pGlobalData);
//             UpdateNumSamples(hwndDlg, pGlobalData);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_NUMBERDSYMBOL:
//                 case IDC_NUMBERSNDIGDEC:
//                 case IDC_NUMBERSDIGITGRSYM:
//                 case IDC_NUMBERSDGROUPING:
//                 case IDC_NUMBERSNSIGNSYM:
//                 case IDC_NUMBERSNNUMFORMAT:
//                 case IDC_NUMBERSDISPLEADZER:
//                 case IDC_NUMBERSLSEP:
//                 case IDC_NUMBERSMEASSYS:
//                     if (HIWORD(wParam) == CBN_SELCHANGE || HIWORD(wParam) == CBN_EDITCHANGE)
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
//                 if (GetNumberSetting(hwndDlg, pGlobalData))
//                 {
//                     pGlobalData->bUserLocaleChanged = TRUE;
//                     UpdateNumSamples(hwndDlg, pGlobalData);
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

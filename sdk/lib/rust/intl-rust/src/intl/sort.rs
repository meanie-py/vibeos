// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut bSortPage: BOOL = 0;
pub static mut userLcid: LCID = 0;
pub static mut hWndSortList: HWND = core::ptr::null_mut();
unsafe extern "system" fn SortTestEnumProc(lpLocale: PWSTR) -> BOOL {
    let lcid: LCID;

    lcid = wcstoul(lpLocale, core::ptr::null_mut(), 16);

    if (LANGIDFROMLCID!(lcid) == LANGIDFROMLCID!(userLcid)) &&
        (SORTIDFROMLCID!(lcid) != SORTIDFROMLCID!(userLcid))
    {
        bSortPage = TRUE as BOOL;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn IsSortPageNeeded(lcid: LCID) -> BOOL {
    if lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
        lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT) {
        return TRUE as BOOL;
    }

    userLcid = lcid;
    bSortPage = (FALSE) as i32;

    EnumSystemLocalesW(Some(SortTestEnumProc), LCID_ALTERNATE_SORTS);

    return bSortPage;
}
unsafe extern "system" fn SortEnumProc(lpLocale: LPTSTR) -> BOOL {
    let mut lcid: LCID;
    let mut lang: [TCHAR; 255] = [0; 255];
    let index: INT_PTR;

    lcid = wcstoul(lpLocale, core::ptr::null_mut(), 16);

    if (LANGIDFROMLCID!(lcid) == LANGIDFROMLCID!(userLcid)) &&
       (SORTIDFROMLCID!(lcid) != SORTIDFROMLCID!(userLcid))
    {
        GetLocaleInfoW(lcid, LOCALE_SSORTNAME, (lang.as_mut_ptr()).cast(), (core::mem::size_of_val(&lang) / core::mem::size_of::<TCHAR>()) as i32);

        index = SendMessageW(hWndSortList,
                             CB_ADDSTRING,
                             0,
                             lang.as_ptr() as LPARAM);

        SendMessageW(hWndSortList,
                     CB_SETITEMDATA,
                     index as WPARAM,
                     lcid as LPARAM);
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn CreateSortList(hwnd: HWND, lcid: LCID) {
    let mut lang: [u16; 255] = [0; 255];
    let mut index: i32;

    hWndSortList = hwnd;

    /* Handle special case for Spanish (Spain) */
    if lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
        lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT)
    {
        /* Add traditional sorting */
        GetLocaleInfoW((MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH)) as LCID,
                       LOCALE_SSORTNAME, lang.as_mut_ptr(), (core::mem::size_of_val(&lang)/core::mem::size_of::<TCHAR>()) as i32);

        index = (SendMessageW(hwnd,
                             CB_ADDSTRING,
                             0,
                             lang.as_ptr() as LPARAM)) as i32;

        SendMessageW(hwnd,
                     CB_SETITEMDATA,
                     index as WPARAM,
                     MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH) as LPARAM);

        /* Add modern sorting */
        GetLocaleInfoW((MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH_MODERN)) as LCID,
                       LOCALE_SSORTNAME, lang.as_mut_ptr(), (core::mem::size_of_val(&lang)/core::mem::size_of::<TCHAR>()) as i32);

        index = (SendMessageW(hwnd,
                             CB_ADDSTRING,
                             0,
                             lang.as_ptr() as LPARAM)) as i32;

        SendMessageW(hwnd,
                     CB_SETITEMDATA,
                     index as WPARAM,
                     MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH_MODERN) as LPARAM);
    }
    else
    {
        userLcid = lcid;

        GetLocaleInfoW(lcid & 0xFFFF, LOCALE_SSORTNAME, lang.as_mut_ptr(), (core::mem::size_of_val(&lang)/core::mem::size_of::<u16>()) as i32);

        index = (SendMessageW(hWndSortList,
                             CB_ADDSTRING,
                             0,
                             lang.as_ptr() as LPARAM)) as i32;

        SendMessageW(hWndSortList,
                     CB_SETITEMDATA,
                     index as WPARAM,
                     (lcid & 0xFFFF) as LPARAM);

        EnumSystemLocalesW(Some(SortEnumProc), LCID_ALTERNATE_SORTS);
    }

    /* Select current locale */
    /* or should it be System and not user? */
    GetLocaleInfoW(lcid, LOCALE_SSORTNAME, lang.as_mut_ptr(), (core::mem::size_of_val(&lang)/core::mem::size_of::<u16>()) as i32);

    SendMessageW(hwnd,
                 CB_SELECTSTRING,
                 -1i32 as WPARAM,
                 lang.as_ptr() as LPARAM);
}

#[no_mangle]
pub unsafe extern "system" fn SortPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBALDATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = (*(lParam as LPPROPSHEETPAGE)).lParam as PGLOBALDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            CreateSortList(
                GetDlgItem(hwndDlg, (IDC_SORTLIST_COMBO) as i32),
                (*pGlobalData).UserLCID,
            );
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_SORTLIST_COMBO => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).code == PSN_APPLY as u32 {
                let NewLcid: LCID;
                let iCurSel: i32;

                iCurSel = (SendDlgItemMessage(
                    hwndDlg,
                    (IDC_SORTLIST_COMBO) as i32,
                    CB_GETCURSEL,
                    0,
                    0,
                )) as i32;
                if iCurSel == CB_ERR {
                    return FALSE as INT_PTR;
                }

                NewLcid = (SendDlgItemMessage(
                    hwndDlg,
                    IDC_SORTLIST_COMBO as c_int,
                    CB_GETITEMDATA,
                    iCurSel as WPARAM,
                    0,
                )) as u32;
                if NewLcid == CB_ERR as LCID {
                    return FALSE as INT_PTR;
                }

                (*pGlobalData).UserLCID = NewLcid;
                (*pGlobalData).bUserLocaleChanged = (TRUE) as i32;
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
//  * PROJECT:         ReactOS International Control Panel
//  * FILE:            dll/cpl/intl/sort.c
//  * PURPOSE:         Sorting property page
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "intl.h"
//
// static BOOL bSortPage = FALSE;
// static LCID userLcid;
//
// static HWND hWndSortList = NULL;
//
//
// static BOOL __stdcall
// SortTestEnumProc(PWSTR lpLocale)
// {
//     LCID lcid;
//
//     lcid = wcstoul(lpLocale, NULL, 16);
//
//     if ((LANGIDFROMLCID(lcid) == LANGIDFROMLCID(userLcid)) &&
//         (SORTIDFROMLCID(lcid) != SORTIDFROMLCID(userLcid)))
//         bSortPage = TRUE;
//
//     return TRUE;
// }
//
//
// BOOL
// IsSortPageNeeded(LCID lcid)
// {
//     /* Handle special case for Spanish (Spain) */
//     if (lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
//         lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT))
//         return TRUE;
//
//     userLcid = lcid;
//     bSortPage = FALSE;
//
//     EnumSystemLocalesW(SortTestEnumProc, LCID_ALTERNATE_SORTS);
//
//     return bSortPage;
// }
//
//
// static BOOL __stdcall
// SortEnumProc(LPTSTR lpLocale)
// {
//     LCID lcid;
//     WCHAR lang[255];
//     INT index;
//
//     lcid = wcstoul(lpLocale, NULL, 16);
//
//     if ((LANGIDFROMLCID(lcid) == LANGIDFROMLCID(userLcid)) &&
//         (SORTIDFROMLCID(lcid) != SORTIDFROMLCID(userLcid)))
//     {
//         GetLocaleInfoW(lcid, LOCALE_SSORTNAME, lang, sizeof(lang)/sizeof(WCHAR));
//
//         index = SendMessageW(hWndSortList,
//                              CB_ADDSTRING,
//                              0,
//                              (LPARAM)lang);
//
//         SendMessageW(hWndSortList,
//                      CB_SETITEMDATA,
//                      index,
//                      (LPARAM)lcid);
//     }
//
//     return TRUE;
// }
//
// static VOID
// CreateSortList(HWND hwnd, LCID lcid)
// {
//     WCHAR lang[255];
//     INT index;
//
//     hWndSortList = hwnd;
//
//     /* Handle special case for Spanish (Spain) */
//     if (lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
//         lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT))
//     {
//         /* Add traditional sorting */
//         GetLocaleInfoW(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH),
//                        LOCALE_SSORTNAME, lang, sizeof(lang)/sizeof(TCHAR));
//
//         index = SendMessageW(hwnd,
//                              CB_ADDSTRING,
//                              0,
//                              (LPARAM)lang);
//
//         SendMessageW(hwnd,
//                      CB_SETITEMDATA,
//                      index,
//                      (LPARAM)MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH));
//
//         /* Add modern sorting */
//         GetLocaleInfoW(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN),
//                        LOCALE_SSORTNAME, lang, sizeof(lang)/sizeof(TCHAR));
//
//         index = SendMessageW(hwnd,
//                              CB_ADDSTRING,
//                              0,
//                              (LPARAM)lang);
//
//         SendMessageW(hwnd,
//                      CB_SETITEMDATA,
//                      index,
//                      (LPARAM)MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN));
//     }
//     else
//     {
//         userLcid = lcid;
//
//         GetLocaleInfoW(lcid & 0xFFFF, LOCALE_SSORTNAME, lang, sizeof(lang)/sizeof(WCHAR));
//
//         index = SendMessageW(hWndSortList,
//                              CB_ADDSTRING,
//                              0,
//                              (LPARAM)lang);
//
//         SendMessageW(hWndSortList,
//                      CB_SETITEMDATA,
//                      index,
//                      (LPARAM)lcid & 0xFFFF);
//
//         EnumSystemLocalesW(SortEnumProc, LCID_ALTERNATE_SORTS);
//     }
//
//     /* Select current locale */
//     /* or should it be System and not user? */
//     GetLocaleInfoW(lcid, LOCALE_SSORTNAME, lang, sizeof(lang)/sizeof(WCHAR));
//
//     SendMessageW(hwnd,
//                  CB_SELECTSTRING,
//                  -1,
//                  (LPARAM)lang);
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// SortPageProc(HWND hwndDlg,
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
//             CreateSortList(GetDlgItem(hwndDlg, IDC_SORTLIST_COMBO), pGlobalData->UserLCID);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_SORTLIST_COMBO:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//             }
//             break;
//
//         case WM_NOTIFY:
//             if (((LPNMHDR)lParam)->code == (UINT)PSN_APPLY)
//             {
//                 LCID NewLcid;
//                 INT iCurSel;
//
//                 iCurSel = SendDlgItemMessage(hwndDlg,
//                                              IDC_SORTLIST_COMBO,
//                                              CB_GETCURSEL,
//                                              0,
//                                              0);
//                 if (iCurSel == CB_ERR)
//                     break;
//
//                 NewLcid = SendDlgItemMessage(hwndDlg,
//                                              IDC_SORTLIST_COMBO,
//                                              CB_GETITEMDATA,
//                                              iCurSel,
//                                              0);
//                 if (NewLcid == (LCID)CB_ERR)
//                     break;
//
//                 /* Save the new LCID */
//                 pGlobalData->UserLCID = NewLcid;
//                 pGlobalData->bUserLocaleChanged = TRUE;
//             }
//             break;
//     }
//
//   return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

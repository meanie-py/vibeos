// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// File-scope preamble for effappdlg.c
// No declarations to translate - the provided C code contains only macro definitions and macro invocations,
// which are handled inline by the function translator.
unsafe extern "C" fn EffAppearanceDlgUpdateControls(hwndDlg: HWND, g: *mut GLOBALS) {
    let mut state: WPARAM;

    state = (SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_ANIMATION) as i32, BM_GETCHECK, 0, 0)) as u32;
    (*g).SchemeAdv.Effects.bMenuAnimation = (((state == BST_CHECKED) != (IDC_EFFAPPEARANCE_ANIMATION == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as u8) as i32;

    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_ANIMATIONTYPE) as i32, CB_SETCURSEL, (*g).SchemeAdv.Effects.bMenuFade as WPARAM, 0);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_EFFAPPEARANCE_ANIMATIONTYPE) as i32), (*g).SchemeAdv.Effects.bMenuAnimation as i32);

    state = (SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_SMOOTHING) as i32, BM_GETCHECK, 0, 0)) as u32;
    (*g).SchemeAdv.Effects.bFontSmoothing = (((state == BST_CHECKED) != (IDC_EFFAPPEARANCE_SMOOTHING == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as u8) as i32;

    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_SMOOTHINGTYPE) as i32, CB_SETCURSEL, ((*g).SchemeAdv.Effects.uiFontSmoothingType - 1) as WPARAM, 0);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_EFFAPPEARANCE_SMOOTHINGTYPE) as i32), (*g).SchemeAdv.Effects.bFontSmoothing as i32);

    state = (SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_SETDROPSHADOW) as i32, BM_GETCHECK, 0, 0)) as u32;
    (*g).SchemeAdv.Effects.bDropShadow = (((state == BST_CHECKED) != (IDC_EFFAPPEARANCE_SETDROPSHADOW == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as u8) as i32;

    state = (SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_DRAGFULLWINDOWS) as i32, BM_GETCHECK, 0, 0)) as u32;
    (*g).SchemeAdv.Effects.bDragFullWindows = (((state == BST_CHECKED) != (IDC_EFFAPPEARANCE_DRAGFULLWINDOWS == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as u8) as i32;

    state = (SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_KEYBOARDCUES) as i32, BM_GETCHECK, 0, 0)) as u32;
    (*g).SchemeAdv.Effects.bKeyboardCues = (((state == BST_CHECKED) != (IDC_EFFAPPEARANCE_KEYBOARDCUES == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as u8) as i32;

    state = (SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_FLATMENUS) as i32, BM_GETCHECK, 0, 0)) as u32;
    (*g).SchemeAdv.bFlatMenus = ((state == BST_CHECKED) as u8) as i32;

    (*g).bSchemeChanged = (1u8) as i32;
}

unsafe extern "C" fn EffAppearanceDlgSaveCurrentValues(hwndDlg: HWND, g: *mut GLOBALS) {}
unsafe extern "C" fn AddToCombobox(Combo: INT, hwndDlg: HWND, From: INT, To: INT) {
    let mut iElement: INT;
    let mut tstrText: [TCHAR; 80] = [0; 80];

    iElement = From;
    while iElement <= To {
        LoadString(
            hApplet,
            iElement as u32,
            tstrText.as_mut_ptr() as LPTSTR,
            (_countof!(tstrText)) as i32,
        );
        SendDlgItemMessage(
            hwndDlg,
            Combo,
            CB_ADDSTRING,
            0,
            tstrText.as_ptr() as LPARAM,
        );
        iElement += 1;
    }
}
unsafe extern "C" fn EffAppearanceDlg_Init(hwndDlg: HWND, g: *mut GLOBALS) {
    let mut state: WPARAM;

    (*g).SchemeAdv = (*g).Scheme;

    state = if (*g).SchemeAdv.Effects.bMenuAnimation != (((IDC_EFFAPPEARANCE_ANIMATION == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as i32) {
        BST_CHECKED as WPARAM
    } else {
        BST_UNCHECKED as WPARAM
    };
    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_ANIMATION) as i32, BM_SETCHECK, state, 0);

    AddToCombobox((IDC_EFFAPPEARANCE_ANIMATIONTYPE) as INT, hwndDlg, (IDS_SLIDEEFFECT) as INT, (IDS_FADEEFFECT) as INT);

    state = if (*g).SchemeAdv.Effects.bFontSmoothing != (((IDC_EFFAPPEARANCE_SMOOTHING == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as i32) {
        BST_CHECKED as WPARAM
    } else {
        BST_UNCHECKED as WPARAM
    };
    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_SMOOTHING) as i32, BM_SETCHECK, state, 0);

    AddToCombobox((IDC_EFFAPPEARANCE_SMOOTHINGTYPE) as INT, hwndDlg, (IDS_STANDARDEFFECT) as INT, (IDS_CLEARTYPEEFFECT) as INT);

    state = if (*g).SchemeAdv.Effects.bDropShadow != (((IDC_EFFAPPEARANCE_SETDROPSHADOW == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as i32) {
        BST_CHECKED as WPARAM
    } else {
        BST_UNCHECKED as WPARAM
    };
    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_SETDROPSHADOW) as i32, BM_SETCHECK, state, 0);

    state = if (*g).SchemeAdv.Effects.bDragFullWindows != (((IDC_EFFAPPEARANCE_DRAGFULLWINDOWS == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as i32) {
        BST_CHECKED as WPARAM
    } else {
        BST_UNCHECKED as WPARAM
    };
    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_DRAGFULLWINDOWS) as i32, BM_SETCHECK, state, 0);

    state = if (*g).SchemeAdv.Effects.bKeyboardCues != (((IDC_EFFAPPEARANCE_KEYBOARDCUES == IDC_EFFAPPEARANCE_KEYBOARDCUES)) as i32) {
        BST_CHECKED as WPARAM
    } else {
        BST_UNCHECKED as WPARAM
    };
    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_KEYBOARDCUES) as i32, BM_SETCHECK, state, 0);

    state = if ((*g).SchemeAdv.bFlatMenus == (1u8) as i32) == (IDC_EFFAPPEARANCE_FLATMENUS == IDC_EFFAPPEARANCE_KEYBOARDCUES) {
        BST_UNCHECKED as WPARAM
    } else {
        BST_CHECKED as WPARAM
    };
    SendDlgItemMessageW(hwndDlg, (IDC_EFFAPPEARANCE_FLATMENUS) as i32, BM_SETCHECK, state, 0);

    EffAppearanceDlgUpdateControls(hwndDlg, g);
}
#[no_mangle]
pub unsafe extern "system" fn EffAppearanceDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let g: *mut GLOBALS;

    g = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut GLOBALS;

    match uMsg {
        WM_INITDIALOG => {
            let g = lParam as *mut GLOBALS;
            SetWindowLongPtr(hwndDlg, DWLP_USER, (g as isize) as i32);
            EffAppearanceDlg_Init(hwndDlg, g);
        }

        WM_DESTROY => {}

        WM_COMMAND => {
            match LOWORD!(wParam) as u32 {
                IDOK => {
                    EffAppearanceDlgSaveCurrentValues(hwndDlg, g);
                    EndDialog(hwndDlg, (IDOK) as INT_PTR);
                }

                IDCANCEL => {
                    (*g).SchemeAdv = (*g).Scheme;
                    EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
                }

                IDC_EFFAPPEARANCE_ANIMATION
                | IDC_EFFAPPEARANCE_SMOOTHING
                | IDC_EFFAPPEARANCE_SETDROPSHADOW
                | IDC_EFFAPPEARANCE_DRAGFULLWINDOWS
                | IDC_EFFAPPEARANCE_KEYBOARDCUES
                | IDC_EFFAPPEARANCE_FLATMENUS => {
                    if HIWORD!(wParam) as u32 == BN_CLICKED {
                        EffAppearanceDlgUpdateControls(hwndDlg, g);
                    }
                }

                IDC_EFFAPPEARANCE_ANIMATIONTYPE | IDC_EFFAPPEARANCE_SMOOTHINGTYPE => {
                    if HIWORD!(wParam) as u32 == CBN_SELCHANGE {
                        let Index = SendDlgItemMessageW(
                            hwndDlg,
                            (IDC_EFFAPPEARANCE_SMOOTHINGTYPE) as i32,
                            CB_GETCURSEL,
                            0,
                            0,
                        );

                        (*g).SchemeAdv.Effects.bMenuFade = (SendDlgItemMessageW(
                            hwndDlg,
                            (IDC_EFFAPPEARANCE_ANIMATIONTYPE) as i32,
                            CB_GETCURSEL,
                            0,
                            0,
                        )) as i32;
                        (*g).SchemeAdv.Effects.uiFontSmoothingType = if Index == CB_ERR {
                            0
                        } else {
                            (Index + 1) as u32
                        };

                        EffAppearanceDlgUpdateControls(hwndDlg, g);
                    }
                }

                _ => {
                    return FALSE;
                }
            }
        }

        _ => {
            return ((FALSE) as i64) as i32;
        }
    }

    TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * PURPOSE:         Effects appearance dialog
//  * PROGRAMMERS:     Jan Roeloffzen <jroeloffzen@hotmail.com>
//  *                  Ismael Ferreras Morezuelas <swyterzone+reactos@gmail.com>
//  */
//
// #include "desk.h"
//
// /* Update all the controls with the current values for the selected screen element */
// static VOID
// EffAppearanceDlgUpdateControls(HWND hwndDlg, GLOBALS *g)
// {
//     WPARAM state;
//
// #define SAVE_CHECKBOX(__CONTROL_ID, __MEMBER)                               \
// do { \
//     state = SendDlgItemMessageW(hwndDlg, __CONTROL_ID, BM_GETCHECK, 0, 0);  \
//     g->SchemeAdv.Effects.__MEMBER = /* Do a XOR of both the conditions */   \
//         ((state == BST_CHECKED) != (__CONTROL_ID == IDC_EFFAPPEARANCE_KEYBOARDCUES)); \
// } while(0)
//
// #define SAVE_CHECKBOX_SCH(__CONTROL_ID, __MEMBER)                           \
// do { \
//     state = SendDlgItemMessageW(hwndDlg, __CONTROL_ID, BM_GETCHECK, 0, 0);  \
//     g->SchemeAdv.__MEMBER = (state == BST_CHECKED);                         \
// } while(0)
//
// #define RSET_COMBOBOX(__CONTROL_ID, __PARENT_MEMBER, __MEMBER)                                          \
// do { \
//     SendDlgItemMessageW(hwndDlg, __CONTROL_ID, CB_SETCURSEL, (WPARAM)g->SchemeAdv.Effects.__MEMBER, 0); \
//     EnableWindow(GetDlgItem(hwndDlg, __CONTROL_ID), g->SchemeAdv.Effects.__PARENT_MEMBER);              \
// } while(0)
//
//     /* Animated menu transitions section (checkbox + combo) */
//     SAVE_CHECKBOX(IDC_EFFAPPEARANCE_ANIMATION,       bMenuAnimation);
//     RSET_COMBOBOX(IDC_EFFAPPEARANCE_ANIMATIONTYPE,   bMenuAnimation, bMenuFade);
//
//     /* Font antialiasing section (checkbox + combo) */
//     SAVE_CHECKBOX(IDC_EFFAPPEARANCE_SMOOTHING,       bFontSmoothing);
//     RSET_COMBOBOX(IDC_EFFAPPEARANCE_SMOOTHINGTYPE,   bFontSmoothing, uiFontSmoothingType - 1);
//
//     /* Other checkboxes */
//     SAVE_CHECKBOX(IDC_EFFAPPEARANCE_SETDROPSHADOW,   bDropShadow);
//     SAVE_CHECKBOX(IDC_EFFAPPEARANCE_DRAGFULLWINDOWS, bDragFullWindows);
//     SAVE_CHECKBOX(IDC_EFFAPPEARANCE_KEYBOARDCUES,    bKeyboardCues);
//     SAVE_CHECKBOX_SCH(IDC_EFFAPPEARANCE_FLATMENUS,   bFlatMenus);
//
// #undef SAVE_CHECKBOX
// #undef RSET_COMBOBOX
//
//     g->bSchemeChanged = TRUE;
// }
//
// static VOID
// EffAppearanceDlgSaveCurrentValues(HWND hwndDlg, GLOBALS *g)
// {
//     /* The settings get saved at the end of ApplyScheme() in theme.c,
//      * when clicking Apply in the main dialog. */
// }
//
// static VOID
// AddToCombobox(INT Combo, HWND hwndDlg, INT From, INT To)
// {
//     INT iElement;
//     TCHAR tstrText[80];
//
//     for (iElement = From; iElement <= To; iElement++)
//     {
//         LoadString(hApplet, iElement, (LPTSTR)tstrText, _countof(tstrText));
//         SendDlgItemMessage(hwndDlg, Combo, CB_ADDSTRING, 0, (LPARAM)tstrText);
//     }
// }
//
// /* Initialize the effects appearance dialog from the scheme populated in LoadCurrentScheme(), in theme.c */
// static VOID
// EffAppearanceDlg_Init(HWND hwndDlg, GLOBALS *g)
// {
//     WPARAM state;
//
//     /* Copy the current theme values */
//     g->SchemeAdv = g->Scheme;
//
// #define INIT_CHECKBOX(__CONTROL_ID, __MEMBER)                           \
// do { \
//     state = /* Do a XOR of both the conditions */                       \
//         ((g->SchemeAdv.Effects.__MEMBER) != (__CONTROL_ID == IDC_EFFAPPEARANCE_KEYBOARDCUES)) \
//             ? BST_CHECKED : BST_UNCHECKED;                              \
//     SendDlgItemMessageW(hwndDlg, __CONTROL_ID, BM_SETCHECK, state, 0);  \
// } while(0)
//
// #define INIT_CHECKBOX_SCH(__CONTROL_ID, __MEMBER)                       \
// do { \
//     state = /* Do a XOR of both the conditions */                       \
//         ((g->SchemeAdv.__MEMBER) == TRUE)                               \
//             ? BST_CHECKED : BST_UNCHECKED;                              \
//     SendDlgItemMessageW(hwndDlg, __CONTROL_ID, BM_SETCHECK, state, 0);  \
// } while(0)
//
// #define FILL_COMBOBOX(__CONTROL_ID, __FIRST_STR, __LAST_STR) \
//     AddToCombobox(__CONTROL_ID, hwndDlg, __FIRST_STR, __LAST_STR)
//
//     /* Animated menu transitions section (checkbox + combo) */
//     INIT_CHECKBOX(IDC_EFFAPPEARANCE_ANIMATION,       bMenuAnimation);
//     FILL_COMBOBOX(IDC_EFFAPPEARANCE_ANIMATIONTYPE,   IDS_SLIDEEFFECT,
//                                                      IDS_FADEEFFECT);
//
//     /* Font antialiasing section (checkbox + combo) */
//     INIT_CHECKBOX(IDC_EFFAPPEARANCE_SMOOTHING,       bFontSmoothing);
//     FILL_COMBOBOX(IDC_EFFAPPEARANCE_SMOOTHINGTYPE,   IDS_STANDARDEFFECT,
//                                                      IDS_CLEARTYPEEFFECT);
//
//     /* Other checkboxes */
//     INIT_CHECKBOX(IDC_EFFAPPEARANCE_SETDROPSHADOW,   bDropShadow);
//     INIT_CHECKBOX(IDC_EFFAPPEARANCE_DRAGFULLWINDOWS, bDragFullWindows);
//     INIT_CHECKBOX(IDC_EFFAPPEARANCE_KEYBOARDCUES,    bKeyboardCues);
//     INIT_CHECKBOX_SCH(IDC_EFFAPPEARANCE_FLATMENUS,   bFlatMenus);
//
// #undef INIT_CHECKBOX
// #undef FILL_COMBOBOX
//
//     /* Update the controls */
//     EffAppearanceDlgUpdateControls(hwndDlg, g);
// }
//
// INT_PTR __stdcall
// EffAppearanceDlgProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     GLOBALS* g;
//
//     g = (GLOBALS*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             g = (GLOBALS*)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)g);
//             EffAppearanceDlg_Init(hwndDlg, g);
//             break;
//
//         case WM_DESTROY:
//             break;
//
//         case WM_COMMAND:
//             switch(LOWORD(wParam))
//             {
//                 case IDOK:
//                     EffAppearanceDlgSaveCurrentValues(hwndDlg, g);
//                     EndDialog(hwndDlg, IDOK);
//                     break;
//
//                 case IDCANCEL:
//                     g->SchemeAdv = g->Scheme;
//                     EndDialog(hwndDlg, IDCANCEL);
//                     break;
//
//                 case IDC_EFFAPPEARANCE_ANIMATION:
//                 case IDC_EFFAPPEARANCE_SMOOTHING:
//                 case IDC_EFFAPPEARANCE_SETDROPSHADOW:
//                 case IDC_EFFAPPEARANCE_DRAGFULLWINDOWS:
//                 case IDC_EFFAPPEARANCE_KEYBOARDCUES:
//                 case IDC_EFFAPPEARANCE_FLATMENUS:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                         EffAppearanceDlgUpdateControls(hwndDlg, g);
//                     break;
//
//                 case IDC_EFFAPPEARANCE_ANIMATIONTYPE:
//                 case IDC_EFFAPPEARANCE_SMOOTHINGTYPE:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         INT Index =
//                             SendDlgItemMessageW(hwndDlg, IDC_EFFAPPEARANCE_SMOOTHINGTYPE,
//                                                 CB_GETCURSEL, 0, 0);
//
//                         g->SchemeAdv.Effects.bMenuFade =
//                             SendDlgItemMessageW(hwndDlg, IDC_EFFAPPEARANCE_ANIMATIONTYPE,
//                                                 CB_GETCURSEL, 0, 0);
//                         g->SchemeAdv.Effects.uiFontSmoothingType = (Index == CB_ERR) ? 0 : (Index + 1);
//
//                         EffAppearanceDlgUpdateControls(hwndDlg, g);
//                     }
//                     break;
//
//                 default:
//                     return FALSE;
//             }
//             break;
//
//         default:
//             return FALSE;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

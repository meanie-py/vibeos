// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn AppearancePage_UpdateThemePreview(hwndDlg: HWND, g: *mut GLOBALS) {
    if (*g).ActiveTheme.ThemeActive != 0 {
        let mut rcWindow: RECT = core::mem::zeroed();

        GetClientRect(GetDlgItem(hwndDlg, IDC_APPEARANCE_PREVIEW), &mut rcWindow);
        if DrawThemePreview((*g).hdcThemePreview, &mut (*g).Scheme, &mut (*g).ActiveTheme, &mut rcWindow) != 0 {
            SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_SET_HDC_PREVIEW, 0, (*g).hdcThemePreview as LPARAM);
            return;
        }
    }

    SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_UPDATETHEME, 0, &mut (*g).Scheme as *mut _ as LPARAM);
    SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_SET_HDC_PREVIEW, 0, 0);
}
unsafe extern "C" fn AppearancePage_LoadSelectedScheme(hwndDlg: HWND, g: *mut GLOBALS) {
    if (*g).ActiveTheme.ThemeActive == FALSE {
        LoadSchemeFromReg(core::ptr::addr_of_mut!((*g).Scheme), core::ptr::addr_of_mut!((*g).ActiveTheme));
    } else {
        LoadSchemeFromTheme(core::ptr::addr_of_mut!((*g).Scheme), core::ptr::addr_of_mut!((*g).ActiveTheme));
    }

    g_GlobalData.desktop_color = (*g).Scheme.crColor[COLOR_DESKTOP];
}
unsafe extern "C" fn AppearancePage_ShowStyles(
    hwndDlg: HWND,
    nIDDlgItem: i32,
    pStyles: PTHEME_STYLE,
    pActiveStyle: PTHEME_STYLE,
) {
    let mut iListIndex: i32;
    let hwndList: HWND = GetDlgItem(hwndDlg, nIDDlgItem);
    let mut pCurrentStyle: PTHEME_STYLE;

    SendMessage(hwndList, CB_RESETCONTENT, 0, 0);

    pCurrentStyle = pStyles;
    while !pCurrentStyle.is_null() {
        iListIndex = SendMessage(hwndList, CB_ADDSTRING, 0, (*pCurrentStyle).DisplayName as LPARAM);
        SendMessage(hwndList, CB_SETITEMDATA, iListIndex as WPARAM, pCurrentStyle as LPARAM);
        if pCurrentStyle == pActiveStyle {
            SendMessage(hwndList, CB_SETCURSEL, iListIndex as WPARAM, 0);
        }
        pCurrentStyle = (*pCurrentStyle).NextStyle;
    }
}
unsafe extern "C" fn AppearancePage_ShowColorSchemes(hwndDlg: HWND, g: *mut GLOBALS) {
    AppearancePage_ShowStyles(
        hwndDlg,
        IDC_APPEARANCE_COLORSCHEME,
        (*(*g).ActiveTheme.Theme).ColoursList,
        (*g).ActiveTheme.Color,
    );
}
unsafe extern "C" fn AppearancePage_ShowSizes(hwndDlg: HWND, g: *mut GLOBALS) {
    let pSizes: PTHEME_STYLE;

    if !(*(*g).ActiveTheme.Theme).SizesList.is_null() {
        pSizes = (*(*g).ActiveTheme.Theme).SizesList;
    } else {
        pSizes = (*(*g).ActiveTheme.Color).ChildStyle;
    }

    AppearancePage_ShowStyles(hwndDlg,
                          IDC_APPEARANCE_SIZE,
                          pSizes,
                          (*g).ActiveTheme.Size);
}
unsafe extern "C" fn AppearancePage_OnInit(hwndDlg: HWND) -> INT_PTR {
    let mut iListIndex: i32;
    let mut hwndTheme: HWND;
    let mut g: *mut GLOBALS;
    let mut rcPreview: RECT = core::mem::zeroed();
    let mut hdcScreen: HDC;
    let mut pTheme: *mut THEME;

    g = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<GLOBALS>()) as *mut GLOBALS;
    if g.is_null() {
        return FALSE;
    }

    SetWindowLongPtr(hwndDlg, DWLP_USER, g as isize);

    (*g).bInitializing = TRUE;

    if LoadCurrentScheme(core::ptr::addr_of_mut!((*g).Scheme)) == 0 {
        return FALSE;
    }

    (*g).pThemes = LoadThemes();
    if !(*g).pThemes.is_null() {
        let mut bLoadedTheme: u32 = FALSE;

        if !g_GlobalData.pwszAction.is_null() &&
           !g_GlobalData.pwszFile.is_null() &&
           wcscmp(g_GlobalData.pwszAction, w!("OpenMSTheme") as *const u16) == 0 {
            bLoadedTheme = FindOrAppendTheme((*g).pThemes,
                                                  g_GlobalData.pwszFile,
                                                  core::ptr::null_mut(),
                                                  core::ptr::null_mut(),
                                                  core::ptr::addr_of_mut!((*g).ActiveTheme));
        }

        if bLoadedTheme != 0 {
            (*g).bThemeChanged = TRUE;
            (*g).bSchemeChanged = TRUE;

            PostMessageW(GetParent(hwndDlg), PSM_CHANGED, hwndDlg as WPARAM, 0);

            AppearancePage_LoadSelectedScheme(hwndDlg, g);
        } else {
            if GetActiveTheme((*g).pThemes, core::ptr::addr_of_mut!((*g).ActiveTheme)) == 0 {
                (*g).ActiveTheme.ThemeActive = FALSE;
            }
        }

        if GetActiveClassicTheme((*g).pThemes, core::ptr::addr_of_mut!((*g).ClassicTheme)) == 0 {
            (*g).ClassicTheme.Theme = (*g).pThemes;
            (*g).ClassicTheme.Color = (*(*g).pThemes).ColoursList;
            (*g).ClassicTheme.Size = (*(*g).ClassicTheme.Color).ChildStyle;
        }

        if (*g).ActiveTheme.ThemeActive == FALSE {
            (*g).ActiveTheme = (*g).ClassicTheme;
        }

        GetClientRect(GetDlgItem(hwndDlg, IDC_APPEARANCE_PREVIEW), core::ptr::addr_of_mut!(rcPreview));

        hdcScreen = GetDC(core::ptr::null_mut());
        (*g).hbmpThemePreview = CreateCompatibleBitmap(hdcScreen, rcPreview.right, rcPreview.bottom);
        (*g).hdcThemePreview = CreateCompatibleDC(hdcScreen);
        SelectObject((*g).hdcThemePreview, (*g).hbmpThemePreview);
        ReleaseDC(core::ptr::null_mut(), hdcScreen);

        hwndTheme = GetDlgItem(hwndDlg, IDC_APPEARANCE_VISUAL_STYLE);

        pTheme = (*g).pThemes;
        while !pTheme.is_null() {
            iListIndex = SendMessage(hwndTheme, CB_ADDSTRING, 0, (*pTheme).DisplayName as LPARAM);
            SendMessage(hwndTheme, CB_SETITEMDATA, iListIndex as WPARAM, pTheme as LPARAM);
            if pTheme == (*g).ActiveTheme.Theme {
                SendMessage(hwndTheme, CB_SETCURSEL, iListIndex as WPARAM, 0);
            }
            pTheme = (*pTheme).NextTheme;
        }

        if !(*g).ActiveTheme.Theme.is_null() {
            AppearancePage_ShowColorSchemes(hwndDlg, g);
            AppearancePage_ShowSizes(hwndDlg, g);
            AppearancePage_UpdateThemePreview(hwndDlg, g);
        }
    }
    (*g).bInitializing = FALSE;

    return FALSE;
}
unsafe extern "C" fn AppearancePage_OnDestroy(hwndDlg: HWND, g: *mut GLOBALS) {
    HeapFree(GetProcessHeap(), 0, g as *mut _);
}
unsafe extern "C" fn GetSelectedData(hwndDlg: HWND, nIDDlgItem: i32) -> PVOID {
    let hwndCombo: HWND;
    let sel: INT;

    hwndCombo = GetDlgItem(hwndDlg, nIDDlgItem);
    sel = SendMessage(hwndCombo, CB_GETCURSEL, 0, 0) as INT;
    if sel == CB_ERR {
        return core::ptr::null_mut();
    }
    return SendMessage(hwndCombo, CB_GETITEMDATA, sel as WPARAM, 0) as PVOID;
}
#[no_mangle]
pub unsafe extern "system" fn AppearancePageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut g: *mut GLOBALS;
    let mut lpnm: LPNMHDR;

    g = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut GLOBALS;

    match uMsg {
        WM_INITDIALOG => {
            return AppearancePage_OnInit(hwndDlg);
        }

        WM_DESTROY => {
            AppearancePage_OnDestroy(hwndDlg, g);
        }

        WM_COMMAND => {
            if g.is_null() || (*g).bInitializing != 0 {
                return FALSE;
            }

            match LOWORD(wParam) {
                IDC_APPEARANCE_EFFECTS => {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE(IDD_EFFAPPEARANCE),
                        hwndDlg,
                        EffAppearanceDlgProc,
                        g as LPARAM,
                    ) == IDOK
                    {
                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                        (*g).Scheme = (*g).SchemeAdv;
                        (*g).bSchemeChanged = TRUE;
                    }
                }

                IDC_APPEARANCE_ADVANCED => {
                    if DialogBoxParam(
                        hApplet,
                        MAKEINTRESOURCE(IDD_ADVAPPEARANCE),
                        hwndDlg,
                        AdvAppearanceDlgProc,
                        g as LPARAM,
                    ) == IDOK
                    {
                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                        (*g).bSchemeChanged = TRUE;
                        (*g).Scheme = (*g).SchemeAdv;
                        g_GlobalData.desktop_color = (*g).Scheme.crColor[COLOR_DESKTOP];

                        AppearancePage_UpdateThemePreview(hwndDlg, g);
                    }
                }

                IDC_APPEARANCE_COLORSCHEME => {
                    if HIWORD(wParam) == CBN_SELCHANGE {
                        (*g).ActiveTheme.Color =
                            GetSelectedData(hwndDlg, IDC_APPEARANCE_COLORSCHEME) as PTHEME_STYLE;
                        if !(*(*g).ActiveTheme.Color).ChildStyle.is_null() {
                            (*g).ActiveTheme.Size = (*(*g).ActiveTheme.Color).ChildStyle;
                        }

                        (*g).bSchemeChanged = TRUE;
                        if (*g).ActiveTheme.ThemeActive != 0 {
                            (*g).bThemeChanged = TRUE;
                        }

                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);

                        AppearancePage_LoadSelectedScheme(hwndDlg, g);
                        AppearancePage_ShowSizes(hwndDlg, g);
                        AppearancePage_UpdateThemePreview(hwndDlg, g);
                    }
                }

                IDC_APPEARANCE_VISUAL_STYLE => {
                    if HIWORD(wParam) == CBN_SELCHANGE {
                        let pTheme: PTHEME =
                            GetSelectedData(hwndDlg, IDC_APPEARANCE_VISUAL_STYLE) as PTHEME;

                        if (*g).ClassicTheme.Theme == pTheme {
                            (*g).ActiveTheme = (*g).ClassicTheme;
                        } else {
                            (*g).ActiveTheme.Theme = pTheme;
                            (*g).ActiveTheme.Size = (*pTheme).SizesList;
                            (*g).ActiveTheme.Color = (*pTheme).ColoursList;
                            (*g).ActiveTheme.ThemeActive = TRUE;
                        }

                        (*g).bThemeChanged = TRUE;
                        (*g).bSchemeChanged = TRUE;

                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);

                        AppearancePage_ShowColorSchemes(hwndDlg, g);
                        AppearancePage_ShowSizes(hwndDlg, g);
                        AppearancePage_LoadSelectedScheme(hwndDlg, g);
                        AppearancePage_UpdateThemePreview(hwndDlg, g);
                    }
                }

                IDC_APPEARANCE_SIZE => {
                    if HIWORD(wParam) == CBN_SELCHANGE {
                        (*g).ActiveTheme.Size =
                            GetSelectedData(hwndDlg, IDC_APPEARANCE_SIZE) as PTHEME_STYLE;
                        (*g).bSchemeChanged = TRUE;
                        if (*g).ActiveTheme.ThemeActive != 0 {
                            (*g).bThemeChanged = TRUE;
                        }

                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);

                        AppearancePage_LoadSelectedScheme(hwndDlg, g);
                        AppearancePage_UpdateThemePreview(hwndDlg, g);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            lpnm = lParam as LPNMHDR;
            match (*lpnm).code {
                PSN_APPLY => {
                    if (*g).bThemeChanged != 0 {
                        ActivateTheme(core::ptr::addr_of_mut!((*g).ActiveTheme));
                    }

                    if (*g).bSchemeChanged != 0 {
                        ApplyScheme(
                            core::ptr::addr_of_mut!((*g).Scheme),
                            core::ptr::addr_of_mut!((*g).ActiveTheme),
                        );
                        if (*g).ActiveTheme.ThemeActive == FALSE {
                            (*g).ClassicTheme = (*g).ActiveTheme;
                        }
                    }

                    AppearancePage_UpdateThemePreview(hwndDlg, g);
                    (*g).bThemeChanged = FALSE;
                    (*g).bSchemeChanged = FALSE;
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR as LONG_PTR);
                    return TRUE;
                }

                PSN_KILLACTIVE => {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, FALSE as LONG_PTR);
                    return TRUE;
                }

                PSN_SETACTIVE => {
                    if (*g).Scheme.crColor[COLOR_DESKTOP] != g_GlobalData.desktop_color {
                        (*g).Scheme.crColor[COLOR_DESKTOP] = g_GlobalData.desktop_color;
                        AppearancePage_UpdateThemePreview(hwndDlg, g);
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/appearance.c
//  * PURPOSE:         Appearance property page
//  *
//  * PROGRAMMERS:     Trevor McCort (lycan359@gmail.com)
//  *                  Timo Kreuzer (timo[dot]kreuzer[at]web[dot]de)
//  */
//
// #include "desk.h"
//
// /******************************************************************************/
//
// static void
// AppearancePage_UpdateThemePreview(HWND hwndDlg, GLOBALS *g)
// {
//     if (g->ActiveTheme.ThemeActive)
//     {
//         RECT rcWindow;
//
//         GetClientRect(GetDlgItem(hwndDlg, IDC_APPEARANCE_PREVIEW), &rcWindow);
//         if (DrawThemePreview(g->hdcThemePreview, &g->Scheme, &g->ActiveTheme, &rcWindow))
//         {
//             SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_SET_HDC_PREVIEW, 0, (LPARAM)g->hdcThemePreview);
//             return;
//         }
//     }
//
//     SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_UPDATETHEME, 0, (LPARAM)&g->Scheme);
//     SendDlgItemMessage(hwndDlg, IDC_APPEARANCE_PREVIEW, PVM_SET_HDC_PREVIEW, 0, 0);
// }
//
// static void
// AppearancePage_LoadSelectedScheme(HWND hwndDlg, GLOBALS *g)
// {
//     if (g->ActiveTheme.ThemeActive == FALSE )
//     {
//         LoadSchemeFromReg(&g->Scheme, &g->ActiveTheme);
//     }
//     else
//     {
//         LoadSchemeFromTheme(&g->Scheme, &g->ActiveTheme);
//     }
//
//     g_GlobalData.desktop_color = g->Scheme.crColor[COLOR_DESKTOP];
// }
//
// static void
// AppearancePage_ShowStyles(HWND hwndDlg, int nIDDlgItem, PTHEME_STYLE pStyles, PTHEME_STYLE pActiveStyle)
// {
//     int iListIndex;
//     HWND hwndList = GetDlgItem(hwndDlg, nIDDlgItem);
//     PTHEME_STYLE pCurrentStyle;
//
//     SendMessage(hwndList, CB_RESETCONTENT , 0, 0);
//
//     for (pCurrentStyle = pStyles;
//          pCurrentStyle;
//          pCurrentStyle = pCurrentStyle->NextStyle)
//     {
//         iListIndex = SendMessage(hwndList, CB_ADDSTRING, 0, (LPARAM)pCurrentStyle->DisplayName);
//         SendMessage(hwndList, CB_SETITEMDATA, iListIndex, (LPARAM)pCurrentStyle);
//         if (pCurrentStyle == pActiveStyle)
//         {
//             SendMessage(hwndList, CB_SETCURSEL, (WPARAM)iListIndex, 0);
//         }
//     }
// }
//
// static void
// AppearancePage_ShowColorSchemes(HWND hwndDlg, GLOBALS *g)
// {
//     AppearancePage_ShowStyles(hwndDlg,
//                               IDC_APPEARANCE_COLORSCHEME,
//                               g->ActiveTheme.Theme->ColoursList,
//                               g->ActiveTheme.Color);
// }
//
// static void
// AppearancePage_ShowSizes(HWND hwndDlg, GLOBALS *g)
// {
//     PTHEME_STYLE pSizes;
//
//     if (g->ActiveTheme.Theme->SizesList)
//         pSizes = g->ActiveTheme.Theme->SizesList;
//     else
//         pSizes = g->ActiveTheme.Color->ChildStyle;
//
//     AppearancePage_ShowStyles(hwndDlg,
//                               IDC_APPEARANCE_SIZE,
//                               pSizes,
//                               g->ActiveTheme.Size);
// }
//
// static INT_PTR
// AppearancePage_OnInit(HWND hwndDlg)
// {
//     INT iListIndex;
//     HWND hwndTheme;
//     GLOBALS *g;
//     RECT rcPreview;
//     HDC hdcScreen;
//     PTHEME pTheme;
//
//     g = (GLOBALS*)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(GLOBALS));
//     if (g == NULL)
//         return FALSE;
//
//     SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)g);
//
//     g->bInitializing = TRUE;
//
//     if (!LoadCurrentScheme(&g->Scheme))
//         return FALSE;
//
//     g->pThemes = LoadThemes();
//     if (g->pThemes)
//     {
//         BOOL bLoadedTheme = FALSE;
//
//         if (g_GlobalData.pwszAction &&
//             g_GlobalData.pwszFile &&
//             wcscmp(g_GlobalData.pwszAction, L"OpenMSTheme") == 0)
//         {
//             bLoadedTheme = FindOrAppendTheme(g->pThemes,
//                                              g_GlobalData.pwszFile,
//                                              NULL,
//                                              NULL,
//                                              &g->ActiveTheme);
//         }
//
//         if (bLoadedTheme)
//         {
//             g->bThemeChanged = TRUE;
//             g->bSchemeChanged = TRUE;
//
//             PostMessageW(GetParent(hwndDlg), PSM_CHANGED, (WPARAM)hwndDlg, 0);
//
//             AppearancePage_LoadSelectedScheme(hwndDlg, g);
//         }
//         else
//         {
//             if (!GetActiveTheme(g->pThemes, &g->ActiveTheme))
//             {
//                 g->ActiveTheme.ThemeActive = FALSE;
//             }
//         }
//
//         /*
//          * Keep a copy of the selected classic theme in order to select this
//          * when user selects the classic theme (and not a horrible random theme )
//          */
//         if (!GetActiveClassicTheme(g->pThemes, &g->ClassicTheme))
//         {
//             g->ClassicTheme.Theme = g->pThemes;
//             g->ClassicTheme.Color = g->pThemes->ColoursList;
//             g->ClassicTheme.Size = g->ClassicTheme.Color->ChildStyle;
//         }
//
//         if (g->ActiveTheme.ThemeActive == FALSE)
//             g->ActiveTheme = g->ClassicTheme;
//
//         GetClientRect(GetDlgItem(hwndDlg, IDC_APPEARANCE_PREVIEW), &rcPreview);
//
//         hdcScreen = GetDC(NULL);
//         g->hbmpThemePreview = CreateCompatibleBitmap(hdcScreen, rcPreview.right, rcPreview.bottom);
//         g->hdcThemePreview = CreateCompatibleDC(hdcScreen);
//         SelectObject(g->hdcThemePreview, g->hbmpThemePreview);
//         ReleaseDC(NULL, hdcScreen);
//
//         hwndTheme = GetDlgItem(hwndDlg, IDC_APPEARANCE_VISUAL_STYLE);
//
//         for (pTheme = g->pThemes; pTheme; pTheme = pTheme->NextTheme)
//         {
//             iListIndex = SendMessage(hwndTheme, CB_ADDSTRING, 0, (LPARAM)pTheme->DisplayName);
//             SendMessage(hwndTheme, CB_SETITEMDATA, iListIndex, (LPARAM)pTheme);
//             if (pTheme == g->ActiveTheme.Theme)
//             {
//                 SendMessage(hwndTheme, CB_SETCURSEL, (WPARAM)iListIndex, 0);
//             }
//         }
//
//         if (g->ActiveTheme.Theme)
//         {
//             AppearancePage_ShowColorSchemes(hwndDlg, g);
//             AppearancePage_ShowSizes(hwndDlg, g);
//             AppearancePage_UpdateThemePreview(hwndDlg, g);
//         }
//     }
//     g->bInitializing = FALSE;
//
//     return FALSE;
// }
//
// static VOID
// AppearancePage_OnDestroy(HWND hwndDlg, GLOBALS *g)
// {
//     HeapFree(GetProcessHeap(), 0, g);
// }
//
// static PVOID
// GetSelectedData(HWND hwndDlg, int nIDDlgItem)
// {
//     HWND hwndCombo;
//     INT sel;
//
//     hwndCombo = GetDlgItem(hwndDlg, nIDDlgItem);
//     sel = SendMessage(hwndCombo, CB_GETCURSEL, 0, 0);
//     if (sel == CB_ERR)
//         return NULL;
//     return (PVOID)SendMessage(hwndCombo, CB_GETITEMDATA, (WPARAM)sel, 0);
// }
//
// INT_PTR __stdcall
// AppearancePageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     GLOBALS *g;
//     LPNMHDR lpnm;
//
//     g = (GLOBALS*)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             return AppearancePage_OnInit(hwndDlg);
//
//         case WM_DESTROY:
//             AppearancePage_OnDestroy(hwndDlg, g);
//             break;
//
//         case WM_COMMAND:
//             if (g == NULL || g->bInitializing)
//                 return FALSE;
//
//             switch (LOWORD(wParam))
//             {
//                 case IDC_APPEARANCE_EFFECTS:
//                     if (DialogBoxParam(hApplet, MAKEINTRESOURCE(IDD_EFFAPPEARANCE),
//                                        hwndDlg, EffAppearanceDlgProc, (LPARAM)g) == IDOK)
//                     {
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                         g->Scheme = g->SchemeAdv;
//                         g->bSchemeChanged = TRUE;
//                     }
//                     break;
//
//                 case IDC_APPEARANCE_ADVANCED:
//                     if (DialogBoxParam(hApplet, MAKEINTRESOURCE(IDD_ADVAPPEARANCE),
//                                        hwndDlg, AdvAppearanceDlgProc, (LPARAM)g) == IDOK)
//                     {
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                         g->bSchemeChanged = TRUE;
//                         g->Scheme = g->SchemeAdv;
//                         g_GlobalData.desktop_color = g->Scheme.crColor[COLOR_DESKTOP];
//
//                         AppearancePage_UpdateThemePreview(hwndDlg, g);
//                     }
//                     break;
//
//                 case IDC_APPEARANCE_COLORSCHEME:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         g->ActiveTheme.Color = (PTHEME_STYLE)GetSelectedData(hwndDlg, IDC_APPEARANCE_COLORSCHEME);
//                         if (g->ActiveTheme.Color->ChildStyle != NULL)
//                             g->ActiveTheme.Size = g->ActiveTheme.Color->ChildStyle;
//
//                         g->bSchemeChanged = TRUE;
//                         if (g->ActiveTheme.ThemeActive)
//                             g->bThemeChanged = TRUE;
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//
//                         AppearancePage_LoadSelectedScheme(hwndDlg, g);
//                         AppearancePage_ShowSizes(hwndDlg, g);
//                         AppearancePage_UpdateThemePreview(hwndDlg, g);
//                     }
//                     break;
//
//                 case IDC_APPEARANCE_VISUAL_STYLE:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         PTHEME pTheme  = (PTHEME)GetSelectedData(hwndDlg, IDC_APPEARANCE_VISUAL_STYLE);
//
//                         if (g->ClassicTheme.Theme == pTheme)
//                             g->ActiveTheme = g->ClassicTheme;
//                         else
//                         {
//                             g->ActiveTheme.Theme = pTheme;
//                             g->ActiveTheme.Size = pTheme->SizesList;
//                             g->ActiveTheme.Color = pTheme->ColoursList;
//                             g->ActiveTheme.ThemeActive = TRUE;
//                         }
//
//                         g->bThemeChanged = TRUE;
//                         g->bSchemeChanged = TRUE;
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//
//                         AppearancePage_ShowColorSchemes(hwndDlg, g);
//                         AppearancePage_ShowSizes(hwndDlg, g);
//                         AppearancePage_LoadSelectedScheme(hwndDlg, g);
//                         AppearancePage_UpdateThemePreview(hwndDlg, g);
//                     }
//                     break;
//
//                 case IDC_APPEARANCE_SIZE:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         g->ActiveTheme.Size = (PTHEME_STYLE)GetSelectedData(hwndDlg, IDC_APPEARANCE_SIZE);
//                         g->bSchemeChanged = TRUE;
//                         if (g->ActiveTheme.ThemeActive)
//                             g->bThemeChanged = TRUE;
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//
//                         AppearancePage_LoadSelectedScheme(hwndDlg, g);
//                         AppearancePage_UpdateThemePreview(hwndDlg, g);
//                     }
//             }
//             break;
//
//         case WM_NOTIFY:
//             lpnm = (LPNMHDR)lParam;
//             switch (lpnm->code)
//             {
//                 case PSN_APPLY:
//
//                     if (g->bThemeChanged)
//                     {
//                         ActivateTheme(&g->ActiveTheme);
//                     }
//
//                     if (g->bSchemeChanged)
//                     {
//                         ApplyScheme(&g->Scheme, &g->ActiveTheme);
//                         if (g->ActiveTheme.ThemeActive == FALSE)
//                             g->ClassicTheme = g->ActiveTheme;
//                     }
//
//                     AppearancePage_UpdateThemePreview(hwndDlg, g);
//                     g->bThemeChanged = FALSE;
//                     g->bSchemeChanged = FALSE;
//                     SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (LONG_PTR)PSNRET_NOERROR);
//                     return TRUE;
//
//                 case PSN_KILLACTIVE:
//                     SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (LONG_PTR)FALSE);
//                     return TRUE;
//
//                 case PSN_SETACTIVE:
//                     if (g->Scheme.crColor[COLOR_DESKTOP] != g_GlobalData.desktop_color)
//                     {
//                         g->Scheme.crColor[COLOR_DESKTOP] = g_GlobalData.desktop_color;
//                         AppearancePage_UpdateThemePreview(hwndDlg, g);
//                     }
//                     break;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LIC_CONTEXT {
    pub hIcon: HICON,
    pub hIconSm: HICON,
}

pub type LIC_CONTEXT = _LIC_CONTEXT;
pub type PLIC_CONTEXT = *mut _LIC_CONTEXT;
#[no_mangle]
pub unsafe extern "system" fn LicenceDlgProc(
    hDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pLicInfo: *mut LIC_CONTEXT;

    UNREFERENCED_PARAMETER(lParam);

    pLicInfo = GetWindowLongPtr(hDlg, DWLP_USER) as *mut LIC_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pLicInfo = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<LIC_CONTEXT>(),
            ) as *mut LIC_CONTEXT;
            if pLicInfo.is_null() {
                EndDialog(hDlg, 0);
                return FALSE;
            }
            SetWindowLongPtr(hDlg, DWLP_USER, pLicInfo as isize);
            return OnInitDialog(hDlg, pLicInfo);
        }

        WM_DESTROY => {
            if !pLicInfo.is_null() {
                if !(*pLicInfo).hIconSm.is_null() {
                    DestroyIcon((*pLicInfo).hIconSm);
                }

                if !(*pLicInfo).hIcon.is_null() {
                    DestroyIcon((*pLicInfo).hIcon);
                }

                HeapFree(GetProcessHeap(), 0, pLicInfo as *mut _);
            }
        }

        WM_COMMAND => {
            if (LOWORD(wParam) == IDOK) || (LOWORD(wParam) == IDCANCEL) {
                EndDialog(hDlg, LOWORD(wParam));
                return TRUE;
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
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/licence.c
//  * PURPOSE:     Licence dialog box message handler
//  * COPYRIGHT:   Copyright 2006 Ged Murphy <gedmurphy@gmail.com>
//  *
//  */
//
// #include "precomp.h"
//
// typedef struct _LIC_CONTEXT
// {
//     HICON hIcon;
//     HICON hIconSm;
// } LIC_CONTEXT, *PLIC_CONTEXT;
//
//
// static BOOL
// OnInitDialog(HWND hDlg, PLIC_CONTEXT pLicInfo)
// {
//     HRSRC hResInfo;
//     HGLOBAL hResMem;
//     PCSTR LicenseText;
//
//     pLicInfo->hIcon = LoadImage(hApplet,
//                                 MAKEINTRESOURCE(IDI_LICENSE),
//                                 IMAGE_ICON,
//                                 GetSystemMetrics(SM_CXICON),
//                                 GetSystemMetrics(SM_CYICON),
//                                 LR_DEFAULTCOLOR);
//     pLicInfo->hIconSm = LoadImage(hApplet,
//                                   MAKEINTRESOURCE(IDI_LICENSE),
//                                   IMAGE_ICON,
//                                   GetSystemMetrics(SM_CXSMICON),
//                                   GetSystemMetrics(SM_CYSMICON),
//                                   LR_DEFAULTCOLOR);
//
//     SendMessage(hDlg, WM_SETICON, ICON_BIG, (LPARAM)pLicInfo->hIcon);
//     SendMessage(hDlg, WM_SETICON, ICON_SMALL, (LPARAM)pLicInfo->hIconSm);
//
//     /* Load license from resource */
//     if (!(hResInfo = FindResource(hApplet,
//                                   MAKEINTRESOURCE(RC_LICENSE),
//                                   MAKEINTRESOURCE(RTDATA))) ||
//         !(hResMem = LoadResource(hApplet, hResInfo)) ||
//         !(LicenseText = LockResource(hResMem)))
//     {
//         ShowLastWin32Error(hDlg);
//         return FALSE;
//     }
//
//     /* Insert the license into the edit control */
//     SetDlgItemTextA(hDlg,
//                     IDC_LICENCEEDIT,
//                     LicenseText);
//
//     PostMessage(GetDlgItem(hDlg, IDC_LICENCEEDIT),
//                 EM_SETSEL,
//                 -1,
//                 0);
//     PostMessage(GetDlgItem(hDlg, IDC_LICENCEEDIT), WM_VSCROLL, SB_TOP, 0);
//     return TRUE;
// }
//
//
// INT_PTR __stdcall
// LicenceDlgProc(HWND hDlg,
//                UINT uMsg,
//                WPARAM wParam,
//                LPARAM lParam)
// {
//     PLIC_CONTEXT pLicInfo;
//
//     UNREFERENCED_PARAMETER(lParam);
//
//     pLicInfo = (PLIC_CONTEXT)GetWindowLongPtr(hDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             pLicInfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(LIC_CONTEXT));
//             if (pLicInfo == NULL)
//             {
//                 EndDialog(hDlg, 0);
//                 return FALSE;
//             }
//             SetWindowLongPtr(hDlg, DWLP_USER, (LONG_PTR)pLicInfo);
//             return OnInitDialog(hDlg, pLicInfo);
//
//         case WM_DESTROY:
//             if (pLicInfo)
//             {
//                 if (pLicInfo->hIconSm)
//                     DestroyIcon(pLicInfo->hIconSm);
//
//                 if (pLicInfo->hIcon)
//                     DestroyIcon(pLicInfo->hIcon);
//
//                 HeapFree(GetProcessHeap(), 0, pLicInfo);
//             }
//             break;
//
//         case WM_COMMAND:
//             if ((LOWORD(wParam) == IDOK) || (LOWORD(wParam) == IDCANCEL))
//             {
//                 EndDialog(hDlg,
//                           LOWORD(wParam));
//                 return TRUE;
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

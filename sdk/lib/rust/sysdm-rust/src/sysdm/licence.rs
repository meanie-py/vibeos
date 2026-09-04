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

/// licence.c:20 `static BOOL OnInitDialog(HWND, PLIC_CONTEXT)`.
///
/// Hand-written: the translate run could not get this body past the envelope and it was
/// dropped with `--skip-fn`, which the run's AUDIT.md records rather than passing over in
/// silence. Everything below is a direct transcription of the C.
///
/// `RC_LICENSE`/`RTDATA` are resource ids, so both reach FindResource through
/// MAKEINTRESOURCE. `SetDlgItemTextA` and `PCSTR` are the A spellings in the C too: the
/// licence blob is a raw ANSI resource, not a wide string, and stays narrow under -DUNICODE.
unsafe extern "C" fn OnInitDialog(hDlg: HWND, pLicInfo: PLIC_CONTEXT) -> BOOL {
    let hResInfo: HRSRC;
    let hResMem: HGLOBAL;
    let LicenseText: PCSTR;

    (*pLicInfo).hIcon = LoadImage(
        hApplet,
        MAKEINTRESOURCE!(IDI_LICENSE),
        IMAGE_ICON,
        GetSystemMetrics((SM_CXICON) as c_int),
        GetSystemMetrics((SM_CYICON) as c_int),
        LR_DEFAULTCOLOR,
    ) as HICON;
    (*pLicInfo).hIconSm = LoadImage(
        hApplet,
        MAKEINTRESOURCE!(IDI_LICENSE),
        IMAGE_ICON,
        GetSystemMetrics((SM_CXSMICON) as c_int),
        GetSystemMetrics((SM_CYSMICON) as c_int),
        LR_DEFAULTCOLOR,
    ) as HICON;

    SendMessage(hDlg, WM_SETICON, (ICON_BIG) as WPARAM, (*pLicInfo).hIcon as LPARAM);
    SendMessage(hDlg, WM_SETICON, (ICON_SMALL) as WPARAM, (*pLicInfo).hIconSm as LPARAM);

    /* Load license from resource */
    hResInfo = FindResource(hApplet, MAKEINTRESOURCE!(RC_LICENSE), MAKEINTRESOURCE!(RTDATA));
    if hResInfo.is_null() {
        ShowLastWin32Error(hDlg);
        return FALSE;
    }
    hResMem = LoadResource(hApplet, hResInfo);
    if hResMem.is_null() {
        ShowLastWin32Error(hDlg);
        return FALSE;
    }
    LicenseText = LockResource(hResMem) as PCSTR;
    if LicenseText.is_null() {
        ShowLastWin32Error(hDlg);
        return FALSE;
    }

    /* Insert the license into the edit control */
    SetDlgItemTextA(hDlg, (IDC_LICENCEEDIT) as c_int, LicenseText);

    PostMessage(
        GetDlgItem(hDlg, (IDC_LICENCEEDIT) as c_int),
        EM_SETSEL,
        (-1i32) as WPARAM,
        0,
    );
    PostMessage(
        GetDlgItem(hDlg, (IDC_LICENCEEDIT) as c_int),
        WM_VSCROLL,
        (SB_TOP) as WPARAM,
        0,
    );
    TRUE
}
#[no_mangle]
pub unsafe extern "system" fn LicenceDlgProc(
    hDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pLicInfo: *mut LIC_CONTEXT;

    UNREFERENCED_PARAMETER!(lParam);

    pLicInfo = GetWindowLongPtr(hDlg, DWLP_USER) as *mut LIC_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pLicInfo = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<LIC_CONTEXT>()) as SIZE_T,
            ) as *mut LIC_CONTEXT;
            if pLicInfo.is_null() {
                EndDialog(hDlg, 0);
                return (FALSE) as INT_PTR;
            }
            SetWindowLongPtr(hDlg, DWLP_USER, pLicInfo as LONG_PTR);
            return OnInitDialog(hDlg, pLicInfo) as INT_PTR;
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
            if (LOWORD!(wParam) == (IDOK) as u16) || (LOWORD!(wParam) == (IDCANCEL) as u16) {
                EndDialog(hDlg, (LOWORD!(wParam)) as INT_PTR);
                return (TRUE) as INT_PTR;
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
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

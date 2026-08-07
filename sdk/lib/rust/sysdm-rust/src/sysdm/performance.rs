// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn RegTreeOpt_OnDestroy(pRTO: *mut IRegTreeOptions) {
    if !pRTO.is_null() {
        IRegTreeOptions_WalkTree(pRTO, WALK_TREE_DESTROY);
        IRegTreeOptions_Release(pRTO);
    }
}
unsafe extern "C" fn RegTreeOpt_ToggleCheckItem(pRTO: *mut IRegTreeOptions, hItem: HTREEITEM) -> BOOL {
    if !pRTO.is_null() && IRegTreeOptions_ToggleItem(pRTO, hItem) >= 0 {
        1
    } else {
        0
    }
}
unsafe extern "C" fn RegTreeOpt_OnTreeViewClick(pRTO: *mut IRegTreeOptions, hWndTree: HWND) -> BOOL {
    let mut HitTest: TV_HITTESTINFO = core::mem::zeroed();
    let dwPos: DWORD = GetMessagePos();
    HitTest.pt.x = GET_X_LPARAM!(dwPos);
    HitTest.pt.y = GET_Y_LPARAM!(dwPos);
    ScreenToClient(hWndTree, &mut HitTest.pt);
    let hItem: HTREEITEM = TreeView_HitTest!(hWndTree, &mut HitTest);
    return RegTreeOpt_ToggleCheckItem(pRTO, hItem);
}
unsafe extern "C" fn RegTreeOpt_OnTreeViewKeyDown(pRTO: *mut IRegTreeOptions, hWndTree: HWND, pKey: *mut TV_KEYDOWN) -> BOOL {
    if (*pKey).wVKey == (VK_SPACE) as u16 {
        return RegTreeOpt_ToggleCheckItem(pRTO, TreeView_GetSelection!(hWndTree));
    } else if (*pKey).wVKey == (VK_F5) as u16 && !pRTO.is_null() {
        IRegTreeOptions_WalkTree(pRTO, WALK_TREE_REFRESH);
    }
    return FALSE;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn VisualEffectsDlgProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pRTO = GetWindowLongPtrW(hDlg, DWLP_USER) as *mut IRegTreeOptions;
    let hWndTree = GetDlgItem(hDlg, IDC_TREE);

    match uMsg {
        WM_INITDIALOG => {
            let mut pRTO: *mut IRegTreeOptions = core::ptr::null_mut();
            if CoCreateInstance(
                &CLSID_CRegTreeOptions,
                core::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IRegTreeOptions,
                &mut pRTO as *mut *mut IRegTreeOptions as *mut *mut core::ffi::c_void,
            ) >= 0
            {
                let pszPath: LPCSTR = b"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects\0".as_ptr() as LPCSTR;
                SetWindowLongPtrW(hDlg, DWLP_USER, pRTO as LPARAM);
                IRegTreeOptions_InitTree(pRTO, hWndTree, HKEY_LOCAL_MACHINE, pszPath, core::ptr::null_mut());
            } else {
                let hWndPS = GetParent(hDlg);
                let iPage = PropSheet_HwndToIndex!(hWndPS, hDlg);
                PostMessage(
                    hWndPS,
                    PSM_REMOVEPAGE,
                    0,
                    SendMessageW(hWndPS, PSM_INDEXTOPAGE, (iPage) as WPARAM, 0),
                );
            }
            return TRUE;
        }

        WM_DESTROY => {
            RegTreeOpt_OnDestroy(pRTO);
            SetWindowLongPtrW(hDlg, DWLP_USER, core::ptr::null_mut() as LPARAM);
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code {
                NM_CLICK => {
                    if RegTreeOpt_OnTreeViewClick(pRTO, hWndTree) != 0 {
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                }

                TVN_KEYDOWN => {
                    if RegTreeOpt_OnTreeViewKeyDown(pRTO, hWndTree, lParam as *mut TV_KEYDOWN) != 0 {
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                }

                PSN_APPLY => {
                    if !pRTO.is_null() {
                        IRegTreeOptions_WalkTree(pRTO, WALK_TREE_SAVE);
                    }

                    SHSendMessageBroadcastW(WM_SETTINGCHANGE, 0, 0);
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn VisualEffectsDlgProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pRTO = GetWindowLongPtrW(hDlg, DWLP_USER) as *mut IRegTreeOptions;
    let hWndTree = GetDlgItem(hDlg, (IDC_TREE) as i32);

    match uMsg {
        WM_INITDIALOG => {
            let mut pRTO: *mut IRegTreeOptions = core::ptr::null_mut();
            if CoCreateInstance(
                &CLSID_CRegTreeOptions,
                core::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IRegTreeOptions,
                &mut pRTO as *mut *mut IRegTreeOptions as *mut *mut core::ffi::c_void,
            ) >= 0
            {
                let pszPath: LPCSTR = b"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects\0".as_ptr() as LPCSTR;
                SetWindowLongPtrW(hDlg, DWLP_USER, pRTO as LPARAM);
                IRegTreeOptions_InitTree(pRTO, hWndTree, HKEY_LOCAL_MACHINE, pszPath, core::ptr::null_mut());
            } else {
                let hWndPS = GetParent(hDlg);
                let iPage = PropSheet_HwndToIndex!(hWndPS, hDlg);
                PostMessage(
                    hWndPS,
                    PSM_REMOVEPAGE,
                    0,
                    SendMessageW(hWndPS, PSM_INDEXTOPAGE, (iPage) as WPARAM, 0),
                );
            }
            return TRUE;
        }

        WM_DESTROY => {
            RegTreeOpt_OnDestroy(pRTO);
            SetWindowLongPtrW(hDlg, DWLP_USER, core::ptr::null_mut() as LPARAM);
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code {
                NM_CLICK => {
                    if RegTreeOpt_OnTreeViewClick(pRTO, hWndTree) != 0 {
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                }

                TVN_KEYDOWN => {
                    if RegTreeOpt_OnTreeViewKeyDown(pRTO, hWndTree, lParam as *mut TV_KEYDOWN) != 0 {
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                }

                PSN_APPLY => {
                    if !pRTO.is_null() {
                        IRegTreeOptions_WalkTree(pRTO, WALK_TREE_SAVE);
                    }

                    SHSendMessageBroadcastW(WM_SETTINGCHANGE, 0, 0);
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn AdvancedDlgProc(
    hDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            EnableWindow(GetDlgItem(hDlg, IDC_CPUCLIENT), FALSE);
            EnableWindow(GetDlgItem(hDlg, IDC_CPUSERVER), FALSE);
            return TRUE;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_CHANGESWAP => {
                    DialogBoxW!(hApplet, MAKEINTRESOURCEW!(IDD_VIRTMEM), hDlg, Some(VirtMemDlgProc));
                }
                _ => {}
            }
        }
        _ => {}
    }
    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn AdvancedDlgProc(
    hDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            EnableWindow(GetDlgItem(hDlg, (IDC_CPUCLIENT) as i32), FALSE);
            EnableWindow(GetDlgItem(hDlg, (IDC_CPUSERVER) as i32), FALSE);
            return TRUE;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_CHANGESWAP => {
                    DialogBoxW!(hApplet, MAKEINTRESOURCEW!(IDD_VIRTMEM), hDlg, VirtMemDlgProc);
                }
                _ => {}
            }
        }
        _ => {}
    }
    FALSE
}
#[no_mangle]
pub unsafe extern "C" fn ShowPerformanceOptions(hDlg: HWND) {
    let hrInit = CoInitialize(core::ptr::null_mut());
    let mut psh = PROPSHEETHEADERW {
        dwSize: core::mem::size_of::<PROPSHEETHEADERW>() as u32,
        dwFlags: PSH_PROPSHEETPAGE | PSH_NOCONTEXTHELP,
        hwndParent: core::ptr::null_mut(),
        hInstance: core::ptr::null_mut(),
        Anonymous1: Default::default(),
        pszCaption: core::ptr::null(),
        nPages: 0,
        Anonymous2: Default::default(),
        ppsp: core::ptr::null(),
        pfnCallback: None,
        Anonymous3: Default::default(),
        hplWatermark: core::ptr::null_mut(),
    };
    let mut pages: [PROPSHEETPAGEW; 2] = core::mem::zeroed();

    pages[0].dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
    pages[0].hInstance = hApplet;
    pages[0].__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(IDD_VISUALEFFECTS);
    pages[0].pfnDlgProc = Some(VisualEffectsDlgProc);

    pages[1].dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
    pages[1].hInstance = hApplet;
    pages[1].__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(IDD_ADVANCEDPERF);
    pages[1].pfnDlgProc = Some(AdvancedDlgProc);

    psh.hwndParent = hDlg;
    psh.hInstance = hApplet;
    psh.pszCaption = MAKEINTRESOURCEW!(IDS_PERFORMANCEOPTIONS);
    psh.nPages = _countof!(pages);
    psh.__bindgen_anon_3.ppsp = pages.as_ptr();

    PropertySheetW(&mut psh);

    if hrInit >= 0 {
        CoUninitialize();
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     LGPL-2.1+ (https://spdx.org/licenses/LGPL-2.1+)
//  * FILE:        dll/cpl/sysdm/performance.c
//  * PURPOSE:     Performance settings property sheets
//  * COPYRIGHT:   Copyright 2024 Whindmar Saksit <whindsaks@proton.me>
//  *
//  */
//
// #include "precomp.h"
// #include <shlguid_undoc.h> // CLSID_CRegTreeOptions
// #include <shlobj_undoc.h> // IRegTreeOptions
//
// static VOID
// RegTreeOpt_OnDestroy(IRegTreeOptions *pRTO)
// {
//     if (pRTO)
//     {
//         IRegTreeOptions_WalkTree(pRTO, WALK_TREE_DESTROY);
//         IRegTreeOptions_Release(pRTO);
//     }
// }
//
// static BOOL
// RegTreeOpt_ToggleCheckItem(IRegTreeOptions *pRTO, HTREEITEM hItem)
// {
//     return pRTO && SUCCEEDED(IRegTreeOptions_ToggleItem(pRTO, hItem));
// }
//
// static BOOL
// RegTreeOpt_OnTreeViewClick(IRegTreeOptions *pRTO, HWND hWndTree)
// {
//     TV_HITTESTINFO HitTest;
//     DWORD dwPos = GetMessagePos();
//     HitTest.pt.x = GET_X_LPARAM(dwPos);
//     HitTest.pt.y = GET_Y_LPARAM(dwPos);
//     ScreenToClient(hWndTree, &HitTest.pt);
//     HTREEITEM hItem = TreeView_HitTest(hWndTree, &HitTest);
//     return RegTreeOpt_ToggleCheckItem(pRTO, hItem);
// }
//
// static BOOL
// RegTreeOpt_OnTreeViewKeyDown(IRegTreeOptions *pRTO, HWND hWndTree, TV_KEYDOWN *pKey)
// {
//     if (pKey->wVKey == VK_SPACE)
//         return RegTreeOpt_ToggleCheckItem(pRTO, TreeView_GetSelection(hWndTree));
//     else if (pKey->wVKey == VK_F5 && pRTO)
//         IRegTreeOptions_WalkTree(pRTO, WALK_TREE_REFRESH);
//     return FALSE;
// }
//
// static INT_PTR __stdcall
// VisualEffectsDlgProc(HWND hDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     IRegTreeOptions *pRTO = (IRegTreeOptions*)GetWindowLongPtrW(hDlg, DWLP_USER);
//     HWND hWndTree = GetDlgItem(hDlg, IDC_TREE);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             IRegTreeOptions *pRTO = NULL;
//             if (SUCCEEDED(CoCreateInstance(&CLSID_CRegTreeOptions, NULL, CLSCTX_INPROC_SERVER,
//                                            &IID_IRegTreeOptions, (void**)&pRTO)))
//             {
//                 LPCSTR pszPath = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects";
//                 SetWindowLongPtrW(hDlg, DWLP_USER, (LPARAM)pRTO);
//                 IRegTreeOptions_InitTree(pRTO, hWndTree, HKEY_LOCAL_MACHINE, pszPath, NULL);
//             }
//             else
//             {
//                 // Without IRegTreeOptions, our page is pointless so we remove it.
//                 // MSDN says we have to post the remove message from inside WM_INITDIALOG.
//                 HWND hWndPS = GetParent(hDlg);
//                 int iPage = PropSheet_HwndToIndex(hWndPS, hDlg);
//                 PostMessage(hWndPS, PSM_REMOVEPAGE, 0, SendMessageW(hWndPS, PSM_INDEXTOPAGE, iPage, 0));
//             }
//             return TRUE;
//         }
//
//         case WM_DESTROY:
//             RegTreeOpt_OnDestroy(pRTO);
//             SetWindowLongPtrW(hDlg, DWLP_USER, (LPARAM)NULL);
//             break;
//
//         case WM_NOTIFY:
//             switch (((LPNMHDR)lParam)->code)
//             {
//                 case NM_CLICK:
//                     if (RegTreeOpt_OnTreeViewClick(pRTO, hWndTree))
//                         PropSheet_Changed(GetParent(hDlg), hDlg);
//                     break;
//
//                 case TVN_KEYDOWN:
//                     if (RegTreeOpt_OnTreeViewKeyDown(pRTO, hWndTree, (TV_KEYDOWN*)lParam))
//                         PropSheet_Changed(GetParent(hDlg), hDlg);
//                     break;
//
//                 case PSN_APPLY:
//                 {
//                     if (pRTO)
//                         IRegTreeOptions_WalkTree(pRTO, WALK_TREE_SAVE);
//
//                     SHSendMessageBroadcastW(WM_SETTINGCHANGE, 0, 0); // For the ListviewShadow setting
//                     return TRUE;
//                 }
//             }
//             break;
//     }
//     return FALSE;
// }
//
// static INT_PTR __stdcall
// AdvancedDlgProc(HWND hDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             // TODO: Not implemented yet
//             EnableWindow(GetDlgItem(hDlg, IDC_CPUCLIENT), FALSE);
//             EnableWindow(GetDlgItem(hDlg, IDC_CPUSERVER), FALSE);
//             return TRUE;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_CHANGESWAP:
//                     DialogBoxW(hApplet, MAKEINTRESOURCEW(IDD_VIRTMEM), hDlg, VirtMemDlgProc);
//                     break;
//             }
//             break;
//     }
//     return FALSE;
// }
//
// VOID
// ShowPerformanceOptions(HWND hDlg)
// {
//     HRESULT hrInit = CoInitialize(NULL); // For IRegTreeOptions
//     PROPSHEETHEADERW psh = { sizeof(psh), PSH_PROPSHEETPAGE | PSH_NOCONTEXTHELP };
//     PROPSHEETPAGEW pages[2] = { 0 };
//
//     pages[0].dwSize = sizeof(*pages);
//     pages[0].hInstance = hApplet;
//     pages[0].pszTemplate = MAKEINTRESOURCEW(IDD_VISUALEFFECTS);
//     pages[0].pfnDlgProc = VisualEffectsDlgProc;
//
//     pages[1].dwSize = sizeof(*pages);
//     pages[1].hInstance = hApplet;
//     pages[1].pszTemplate = MAKEINTRESOURCEW(IDD_ADVANCEDPERF);
//     pages[1].pfnDlgProc = AdvancedDlgProc;
//
//     psh.hwndParent = hDlg;
//     psh.hInstance = hApplet;
//     psh.pszCaption = MAKEINTRESOURCEW(IDS_PERFORMANCEOPTIONS);
//     psh.nPages = _countof(pages);
//     psh.ppsp = pages;
//
//     PropertySheetW(&psh);
//
//     if (SUCCEEDED(hrInit))
//         CoUninitialize();
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

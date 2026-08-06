// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hApplet: HINSTANCE = 0 as HINSTANCE;

pub static mut Applets: [APPLET; NUM_APPLETS] = [
    APPLET {
        idIcon: IDI_CPLSYSTEM,
        idName: IDS_CPLSYSTEMNAME,
        idDescription: IDS_CPLSYSTEMDESCRIPTION,
        AppletProc: Some(SystemApplet),
    }
];

pub const MAX_SYSTEM_PAGES: u32 = 32;

pub type PCreateNetIDPropertyPage = Option<unsafe extern "system" fn() -> HPROPSHEETPAGE>;
#[no_mangle]
pub unsafe extern "C" fn ResourceMessageBox(
    hInstance: HINSTANCE,
    hwnd: HWND,
    uType: u32,
    uCaption: u32,
    uText: u32,
    mut args: ...
) -> i32 {
    let mut args: va_list;
    let mut szCaption: [u16; MAX_STR_LENGTH] = [0; MAX_STR_LENGTH];
    let mut szText: [u16; MAX_STR_LENGTH] = [0; MAX_STR_LENGTH];
    let mut szCookedText: [u16; 2 * MAX_STR_LENGTH] = [0; 2 * MAX_STR_LENGTH];

    LoadStringW(hInstance, uCaption, szCaption.as_mut_ptr(), _countof(szCaption));
    LoadStringW(hInstance, uText, szText.as_mut_ptr(), _countof(szText));

    va_start(args, uText);
    StringCchVPrintfW(szCookedText.as_mut_ptr(), _countof(szCookedText),
                      szText.as_ptr(), args);
    va_end(args);

    return MessageBoxW(hwnd,
                       szCookedText.as_ptr(),
                       szCaption.as_ptr(),
                       uType);
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn PropSheetAddPage(hpage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let ppsh = lParam as *mut PROPSHEETHEADER;
    if !ppsh.is_null() && (*ppsh).nPages < MAX_SYSTEM_PAGES {
        let index = (*ppsh).nPages as usize;
        (*ppsh).phpage[index] = hpage;
        (*ppsh).nPages += 1;
        return TRUE;
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PropSheetAddPage(hpage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let ppsh = lParam as *mut PROPSHEETHEADER;
    if !ppsh.is_null() && (*ppsh).nPages < MAX_SYSTEM_PAGES {
        let index = (*ppsh).nPages as usize;
        (*ppsh).phpage[index] = hpage;
        (*ppsh).nPages += 1;
        return TRUE;
    }

    FALSE
}
unsafe extern "C" fn InitPropSheetPage(ppsh: *mut PROPSHEETHEADER, idDlg: WORD, DlgProc: DLGPROC) -> BOOL {
    let mut hPage: HPROPSHEETPAGE;
    let mut psp: PROPSHEETPAGE;

    if (*ppsh).nPages < MAX_SYSTEM_PAGES {
        ZeroMemory(core::ptr::addr_of_mut!(psp) as LPVOID, core::mem::size_of::<PROPSHEETPAGE>());
        psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
        psp.dwFlags = PSP_DEFAULT;
        psp.hInstance = hApplet;
        psp.pszTemplate = MAKEINTRESOURCE(idDlg);
        psp.pfnDlgProc = DlgProc;

        hPage = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));
        if !hPage.is_null() {
            return PropSheetAddPage(hPage, ppsh as LPARAM);
        }
    }

    return FALSE;
}
unsafe extern "C" fn AddNetIdPage(ppsh: *mut PROPSHEETHEADER) -> HMODULE {
    let mut hPage: HPROPSHEETPAGE;
    let mut hMod: HMODULE;
    let mut pCreateNetIdPage: PCreateNetIDPropertyPage;

    hMod = LoadLibrary(TEXT(b"netid.dll\0".as_ptr()));
    if hMod != core::ptr::null_mut() {
        pCreateNetIdPage = GetProcAddress(hMod, b"CreateNetIDPropertyPage\0".as_ptr() as *const i8) as PCreateNetIDPropertyPage;
        if pCreateNetIdPage != core::ptr::null_mut() {
            hPage = pCreateNetIdPage();
            if hPage == core::ptr::null_mut() {
                FreeLibrary(hMod);
                hMod = core::ptr::null_mut();
            } else {
                if PropSheetAddPage(hPage, ppsh as LPARAM) == 0 {
                    DestroyPropertySheetPage(hPage);
                    FreeLibrary(hMod);
                    hMod = core::ptr::null_mut();
                }
            }
        } else {
            FreeLibrary(hMod);
            hMod = core::ptr::null_mut();
        }
    }

    hMod
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn PropSheetProc(
    hwndDlg: HWND,
    uMsg: u32,
    lParam: LPARAM,
) -> i32 {
    let hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(
                hApplet,
                MAKEINTRESOURCEW(IDI_CPLSYSTEM),
            );
            SendMessageW(
                hwndDlg,
                WM_SETICON,
                ICON_BIG,
                hIcon as LPARAM,
            );
        }
        _ => {}
    }
    0
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PropSheetProc(
    hwndDlg: HWND,
    uMsg: u32,
    lParam: LPARAM,
) -> i32 {
    let hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(
                hApplet,
                MAKEINTRESOURCEW(IDI_CPLSYSTEM),
            );
            SendMessageW(
                hwndDlg,
                WM_SETICON,
                ICON_BIG,
                hIcon as LPARAM,
            );
        }
        _ => {}
    }
    0
}
#[no_mangle]
pub unsafe extern "system" fn SystemApplet(hwnd: HWND, uMsg: u32, wParam: LPARAM, lParam: LPARAM) -> i32 {
    let mut hpsp: [HPROPSHEETPAGE; MAX_SYSTEM_PAGES] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut hNetIdDll: HMODULE;
    let mut hpsxa: HPSXA = core::ptr::null_mut();
    let mut nPage: i32 = 0;
    let Ret: i32;
    static mut icc: INITCOMMONCONTROLSEX = INITCOMMONCONTROLSEX {
        dwSize: core::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_LINK_CLASS,
    };

    if InitCommonControlsEx(core::ptr::addr_of_mut!(icc)) == 0 {
        return 0;
    }

    if uMsg == CPL_STARTWPARMSW && lParam != 0 {
        nPage = _wtoi(lParam as PWSTR);
    }

    if nPage == -1 {
        ShowPerformanceOptions(hwnd);
        return TRUE;
    }

    ZeroMemory(core::ptr::addr_of_mut!(psh) as LPVOID, core::mem::size_of::<PROPSHEETHEADER>());
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
    psh.hwndParent = hwnd;
    psh.hInstance = hApplet;
    psh.pszIcon = MAKEINTRESOURCEW(IDI_CPLSYSTEM);
    psh.pszCaption = MAKEINTRESOURCE(IDS_CPLSYSTEMNAME);
    psh.nPages = 0;
    psh.nStartPage = 0;
    psh.phpage = hpsp.as_mut_ptr();
    psh.pfnCallback = Some(PropSheetProc);

    InitPropSheetPage(core::ptr::addr_of_mut!(psh), IDD_PROPPAGEGENERAL, Some(GeneralPageProc));
    hNetIdDll = AddNetIdPage(core::ptr::addr_of_mut!(psh));
    InitPropSheetPage(core::ptr::addr_of_mut!(psh), IDD_PROPPAGEHARDWARE, Some(HardwarePageProc));
    InitPropSheetPage(core::ptr::addr_of_mut!(psh), IDD_PROPPAGEADVANCED, Some(AdvancedPageProc));

    hpsxa = SHCreatePropSheetExtArray(HKEY_LOCAL_MACHINE, [REGSTR_PATH_CONTROLSFOLDER, TEXT("\\System")].concat().as_ptr(), MAX_SYSTEM_PAGES - psh.nPages);
    if hpsxa != core::ptr::null_mut() {
        SHAddFromPropSheetExtArray(hpsxa, Some(PropSheetAddPage), core::ptr::addr_of_mut!(psh) as LPARAM);
    }

    if nPage != 0 && nPage <= psh.nPages as i32 {
        psh.nStartPage = nPage as u32;
    }

    Ret = PropertySheet(core::ptr::addr_of_mut!(psh)) as i32;
    let Ret = if Ret != -1 { 1 } else { 0 };

    if hpsxa != core::ptr::null_mut() {
        SHDestroyPropSheetExtArray(hpsxa);
    }

    if hNetIdDll != core::ptr::null_mut() {
        FreeLibrary(hNetIdDll);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCPl: HWND,
    uMsg: u32,
    lParam1: LPARAM,
    lParam2: LPARAM,
) -> i32 {
    let i = lParam1 as u32;

    UNREFERENCED_PARAMETER(hwndCPl);

    match uMsg {
        CPL_INIT => {
            return TRUE;
        }

        CPL_GETCOUNT => {
            return NUM_APPLETS;
        }

        CPL_INQUIRE => {
            if i < NUM_APPLETS {
                let CPlInfo = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon;
                (*CPlInfo).idName = Applets[i as usize].idName;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription;
            } else {
                return TRUE;
            }
        }

        CPL_DBLCLK => {
            if i < NUM_APPLETS {
                (Applets[i as usize].AppletProc)(hwndCPl, uMsg, lParam1, lParam2);
            } else {
                return TRUE;
            }
        }

        CPL_STARTWPARMSW => {
            if i < NUM_APPLETS {
                return (Applets[i as usize].AppletProc)(hwndCPl, uMsg, lParam1, lParam2);
            }
        }

        _ => {}
    }

    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: u32,
    lpvReserved: LPVOID,
) -> i32 {
    UNREFERENCED_PARAMETER(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH => {
            hApplet = hinstDLL;
        }
        _ => {}
    }

    TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/sysdm.c
//  * PURPOSE:     dll entry file
//  * COPYRIGHT:   Copyright Thomas Weidenmueller <w3seek@reactos.org>
//  *
//  */
//
// #include "precomp.h"
//
// #include <regstr.h>
//
// static LONG __stdcall SystemApplet(HWND hwnd, UINT uMsg, LPARAM wParam, LPARAM lParam);
// HINSTANCE hApplet = 0;
//
// /* Applets */
// APPLET Applets[NUM_APPLETS] =
// {
//     {IDI_CPLSYSTEM, IDS_CPLSYSTEMNAME, IDS_CPLSYSTEMDESCRIPTION, SystemApplet}
// };
//
// #define MAX_SYSTEM_PAGES    32
//
//
// INT __cdecl
// ResourceMessageBox(
//      HINSTANCE hInstance,
//      HWND hwnd,
//      UINT uType,
//      UINT uCaption,
//      UINT uText,
//     ...)
// {
//     va_list args;
//     WCHAR szCaption[MAX_STR_LENGTH];
//     WCHAR szText[MAX_STR_LENGTH];
//     WCHAR szCookedText[2*MAX_STR_LENGTH];
//
//     LoadStringW(hInstance, uCaption, szCaption, _countof(szCaption));
//     LoadStringW(hInstance, uText, szText, _countof(szText));
//
//     va_start(args, uText);
//     StringCchVPrintfW(szCookedText, _countof(szCookedText),
//                       szText, args);
//     va_end(args);
//
//     return MessageBoxW(hwnd,
//                        szCookedText,
//                        szCaption,
//                        uType);
// }
//
//
// static BOOL __stdcall
// PropSheetAddPage(HPROPSHEETPAGE hpage, LPARAM lParam)
// {
//     PROPSHEETHEADER *ppsh = (PROPSHEETHEADER *)lParam;
//     if (ppsh != NULL && ppsh->nPages < MAX_SYSTEM_PAGES)
//     {
//         ppsh->phpage[ppsh->nPages++] = hpage;
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static BOOL
// InitPropSheetPage(PROPSHEETHEADER *ppsh, WORD idDlg, DLGPROC DlgProc)
// {
//     HPROPSHEETPAGE hPage;
//     PROPSHEETPAGE psp;
//
//     if (ppsh->nPages < MAX_SYSTEM_PAGES)
//     {
//         ZeroMemory(&psp, sizeof(psp));
//         psp.dwSize = sizeof(psp);
//         psp.dwFlags = PSP_DEFAULT;
//         psp.hInstance = hApplet;
//         psp.pszTemplate = MAKEINTRESOURCE(idDlg);
//         psp.pfnDlgProc = DlgProc;
//
//         hPage = CreatePropertySheetPage(&psp);
//         if (hPage != NULL)
//         {
//             return PropSheetAddPage(hPage, (LPARAM)ppsh);
//         }
//     }
//
//     return FALSE;
// }
//
// typedef HPROPSHEETPAGE (WINAPI *PCreateNetIDPropertyPage)(VOID);
//
// static HMODULE
// AddNetIdPage(PROPSHEETHEADER *ppsh)
// {
//     HPROPSHEETPAGE hPage;
//     HMODULE hMod;
//     PCreateNetIDPropertyPage pCreateNetIdPage;
//
//     hMod = LoadLibrary(TEXT("netid.dll"));
//     if (hMod != NULL)
//     {
//         pCreateNetIdPage = (PCreateNetIDPropertyPage)GetProcAddress(hMod,
//                                                                     "CreateNetIDPropertyPage");
//         if (pCreateNetIdPage != NULL)
//         {
//             hPage = pCreateNetIdPage();
//             if (hPage == NULL)
//                 goto Fail;
//
//             if (!PropSheetAddPage(hPage, (LPARAM)ppsh))
//             {
//                 DestroyPropertySheetPage(hPage);
//                 goto Fail;
//             }
//         }
//         else
//         {
// Fail:
//             FreeLibrary(hMod);
//             hMod = NULL;
//         }
//     }
//
//     return hMod;
// }
//
// static int __stdcall
// PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam)
// {
//     // NOTE: This callback is needed to set large icon correctly.
//     HICON hIcon;
//     switch (uMsg)
//     {
//         case PSCB_INITIALIZED:
//         {
//             hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW(IDI_CPLSYSTEM));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// /* First Applet */
// LONG __stdcall
// SystemApplet(HWND hwnd, UINT uMsg, LPARAM wParam, LPARAM lParam)
// {
//     HPROPSHEETPAGE hpsp[MAX_SYSTEM_PAGES];
//     PROPSHEETHEADER psh;
//     HMODULE hNetIdDll;
//     HPSXA hpsxa = NULL;
//     INT nPage = 0;
//     LONG Ret;
//     static INITCOMMONCONTROLSEX icc = {sizeof(INITCOMMONCONTROLSEX), ICC_LINK_CLASS};
//
//     if (!InitCommonControlsEx(&icc))
//         return 0;
//
//     if (uMsg == CPL_STARTWPARMSW && lParam != 0)
//     {
//         nPage = _wtoi((PWSTR)lParam);
//     }
//
//     if (nPage == -1)
//     {
//         ShowPerformanceOptions(hwnd);
//         return TRUE;
//     }
//
//     ZeroMemory(&psh, sizeof(PROPSHEETHEADER));
//     psh.dwSize = sizeof(PROPSHEETHEADER);
//     psh.dwFlags =  PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
//     psh.hwndParent = hwnd;
//     psh.hInstance = hApplet;
//     psh.pszIcon = MAKEINTRESOURCEW(IDI_CPLSYSTEM);
//     psh.pszCaption = MAKEINTRESOURCE(IDS_CPLSYSTEMNAME);
//     psh.nPages = 0;
//     psh.nStartPage = 0;
//     psh.phpage = hpsp;
//     psh.pfnCallback = PropSheetProc;
//
//     InitPropSheetPage(&psh, IDD_PROPPAGEGENERAL, GeneralPageProc);
//     hNetIdDll = AddNetIdPage(&psh);
//     InitPropSheetPage(&psh, IDD_PROPPAGEHARDWARE, HardwarePageProc);
//     InitPropSheetPage(&psh, IDD_PROPPAGEADVANCED, AdvancedPageProc);
//
//     /* Load additional pages provided by shell extensions */
//     hpsxa = SHCreatePropSheetExtArray(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER TEXT("\\System"), MAX_SYSTEM_PAGES - psh.nPages);
//     if (hpsxa != NULL)
//     {
//         SHAddFromPropSheetExtArray(hpsxa, PropSheetAddPage, (LPARAM)&psh);
//     }
//
//     if (nPage != 0 && nPage <= psh.nPages)
//         psh.nStartPage = nPage;
//
//     Ret = (LONG)(PropertySheet(&psh) != -1);
//
//     if (hpsxa != NULL)
//     {
//         SHDestroyPropSheetExtArray(hpsxa);
//     }
//
//     if (hNetIdDll != NULL)
//         FreeLibrary(hNetIdDll);
//
//     return Ret;
// }
//
// /* Control Panel Callback */
// LONG __stdcall
// CPlApplet(HWND hwndCPl,
//           UINT uMsg,
//           LPARAM lParam1,
//           LPARAM lParam2)
// {
//     UINT i = (UINT)lParam1;
//
//     UNREFERENCED_PARAMETER(hwndCPl);
//
//     switch (uMsg)
//     {
//         case CPL_INIT:
//             return TRUE;
//
//         case CPL_GETCOUNT:
//             return NUM_APPLETS;
//
//         case CPL_INQUIRE:
//             if (i < NUM_APPLETS)
//             {
//                  CPLINFO *CPlInfo = (CPLINFO*)lParam2;
//                  CPlInfo->lData = 0;
//                  CPlInfo->idIcon = Applets[i].idIcon;
//                  CPlInfo->idName = Applets[i].idName;
//                  CPlInfo->idInfo = Applets[i].idDescription;
//             }
//             else
//             {
//                 return TRUE;
//             }
//             break;
//
//         case CPL_DBLCLK:
//             if (i < NUM_APPLETS)
//                 Applets[i].AppletProc(hwndCPl, uMsg, lParam1, lParam2);
//             else
//                 return TRUE;
//             break;
//
//         case CPL_STARTWPARMSW:
//             if (i < NUM_APPLETS)
//                 return Applets[i].AppletProc(hwndCPl, uMsg, lParam1, lParam2);
//             break;
//     }
//
//     return FALSE;
// }
//
//
// BOOL WINAPI
// DllMain(HINSTANCE hinstDLL,
//         DWORD dwReason,
//         LPVOID lpvReserved)
// {
//     UNREFERENCED_PARAMETER(lpvReserved);
//
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//         case DLL_THREAD_ATTACH:
//             hApplet = hinstDLL;
//             break;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `#[no_mangle]`: precomp.h declares `extern HINSTANCE hApplet`, so the island carries it as an
// extern and every sibling module resolves THAT, not this definition. Without the attribute the
// symbol is mangled, nothing defines `_hApplet`, and the link fails — which `cargo check` cannot
// see, because nothing links there.
#[no_mangle]
pub static mut hApplet: HINSTANCE = 0 as HINSTANCE;

pub static mut Applets: [APPLET; (NUM_APPLETS) as usize] = [
    APPLET {
        idIcon: (IDI_CPLSYSTEM) as i32,
        idName: (IDS_CPLSYSTEMNAME) as i32,
        idDescription: (IDS_CPLSYSTEMDESCRIPTION) as i32,
        AppletProc: Some(SystemApplet),
    }
];

pub const MAX_SYSTEM_PAGES: u32 = 32;

pub type PCreateNetIDPropertyPage = Option<unsafe extern "system" fn() -> HPROPSHEETPAGE>;
/// sysdm.c:27 `INT __cdecl ResourceMessageBox(…, ...)`, the variadic entry point.
///
/// Stable Rust cannot DEFINE a C-variadic (`c_variadic`, rust-lang#44930), so this RECEIVES
/// the varargs the va-slots way (`va_slots_rust`, as shell32's ShellMessageBoxA and the kernel's
/// printf family do): on all three targets a variadic call puts its arguments in the same slots
/// as a fixed-arity call with that many parameters (i386 cdecl stack; win64 RCX-R9 then stack;
/// arm64 x0-x7 then stack, where Windows passes varargs in integer registers), and a `va_list` is
/// a plain pointer. The trailing slots are ordinary parameters, staged into an array a `va_list`
/// points at. This replaced the `va_start` trampoline in rust_varargs_shim.c (2026-09-25).
#[no_mangle]
pub unsafe extern "C" fn ResourceMessageBox(
    hInstance: HINSTANCE,
    hwnd: HWND,
    uType: u32,
    uCaption: u32,
    uText: u32,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) -> i32 {
    let mut stage = va_slots_rust::stage(&[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
    let mut ap: *mut core::ffi::c_void = stage.as_mut_ptr() as *mut core::ffi::c_void;
    ResourceMessageBox_v(hInstance, hwnd, uType, uCaption, uText,
                         &mut ap as *mut *mut core::ffi::c_void as *mut core::ffi::c_void)
}

/// sysdm.c:27 `INT __cdecl ResourceMessageBox(…, ...)`, minus the `...`.
///
/// The variadic entry point above hands us `app`, a POINTER to the `va_list`. strsafe-rust's
/// `StringCchPrintfW_v` takes that same by-pointer form, so the C's
/// `StringCchVPrintfW(…, args)` lands as a `_v` call here rather than a by-value one.
#[no_mangle]
pub unsafe extern "C" fn ResourceMessageBox_v(
    hInstance: HINSTANCE,
    hwnd: HWND,
    uType: u32,
    uCaption: u32,
    uText: u32,
    app: *mut core::ffi::c_void,
) -> i32 {
    let mut szCaption: [u16; (MAX_STR_LENGTH) as usize] = [0; (MAX_STR_LENGTH) as usize];
    let mut szText: [u16; (MAX_STR_LENGTH) as usize] = [0; (MAX_STR_LENGTH) as usize];
    let mut szCookedText: [u16; (2 * MAX_STR_LENGTH) as usize] = [0; (2 * MAX_STR_LENGTH) as usize];

    LoadStringW(hInstance, uCaption, szCaption.as_mut_ptr(), (_countof!(szCaption)) as i32);
    LoadStringW(hInstance, uText, szText.as_mut_ptr(), (_countof!(szText)) as i32);

    strsafe_rust::StringCchPrintfW_v(
        szCookedText.as_mut_ptr(),
        (_countof!(szCookedText)) as usize,
        szText.as_ptr(),
        app,
    );

    return MessageBoxW(hwnd,
                       szCookedText.as_ptr(),
                       szCaption.as_ptr(),
                       uType);
}
// ONE definition, not a `#[cfg]` ABI twin pair: sysdm.c says `CALLBACK`, and `extern "system"`
// IS that -- stdcall on i386, the C convention everywhere else. `phpage` is a POINTER in the
// header's union, so the C's `ppsh->phpage[ppsh->nPages++]` is pointer arithmetic, not indexing.
unsafe extern "system" fn PropSheetAddPage(hpage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let ppsh = lParam as *mut PROPSHEETHEADER;
    if !ppsh.is_null() && (*ppsh).nPages < MAX_SYSTEM_PAGES {
        let index = (*ppsh).nPages as usize;
        *(*ppsh).__bindgen_anon_3.phpage.add(index) = hpage;
        (*ppsh).nPages += 1;
        return TRUE;
    }

    FALSE
}
unsafe extern "C" fn InitPropSheetPage(ppsh: *mut PROPSHEETHEADER, idDlg: WORD, DlgProc: DLGPROC) -> BOOL {
    let mut hPage: HPROPSHEETPAGE;
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();

    if (*ppsh).nPages < MAX_SYSTEM_PAGES {
        ZeroMemory!(core::ptr::addr_of_mut!(psp) as LPVOID, core::mem::size_of::<PROPSHEETPAGE>());
        psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
        psp.dwFlags = PSP_DEFAULT;
        psp.hInstance = hApplet;
        psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg);
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

    hMod = LoadLibrary(w!("netid.dll"));
    if hMod != core::ptr::null_mut() {
        pCreateNetIdPage = core::mem::transmute::<_, PCreateNetIDPropertyPage>(
            GetProcAddress(hMod, b"CreateNetIDPropertyPage\0".as_ptr() as *const i8));
        if pCreateNetIdPage.is_some() {
            hPage = (pCreateNetIdPage.unwrap_unchecked())();
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
unsafe extern "system" fn PropSheetProc(
    hwndDlg: HWND,
    uMsg: u32,
    lParam: LPARAM,
) -> i32 {
    let hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(
                hApplet,
                MAKEINTRESOURCEW!(IDI_CPLSYSTEM),
            );
            SendMessageW(
                hwndDlg,
                WM_SETICON,
                (ICON_BIG) as WPARAM,
                hIcon as LPARAM,
            );
        }
        _ => {}
    }
    0
}
#[no_mangle]
pub unsafe extern "system" fn SystemApplet(hwnd: HWND, uMsg: u32, wParam: LPARAM, lParam: LPARAM) -> i32 {
    let mut hpsp: [HPROPSHEETPAGE; (MAX_SYSTEM_PAGES) as usize] = core::mem::zeroed();
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

    ZeroMemory!(core::ptr::addr_of_mut!(psh) as LPVOID, core::mem::size_of::<PROPSHEETHEADER>());
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
    psh.hwndParent = hwnd;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_CPLSYSTEM);
    psh.pszCaption = MAKEINTRESOURCE!(IDS_CPLSYSTEMNAME);
    psh.nPages = 0;
    psh.__bindgen_anon_2.nStartPage = 0;
    psh.__bindgen_anon_3.phpage = hpsp.as_mut_ptr();
    psh.pfnCallback = Some(PropSheetProc);

    InitPropSheetPage(core::ptr::addr_of_mut!(psh), (IDD_PROPPAGEGENERAL) as WORD, Some(GeneralPageProc));
    hNetIdDll = AddNetIdPage(core::ptr::addr_of_mut!(psh));
    InitPropSheetPage(core::ptr::addr_of_mut!(psh), (IDD_PROPPAGEHARDWARE) as WORD, Some(HardwarePageProc));
    InitPropSheetPage(core::ptr::addr_of_mut!(psh), (IDD_PROPPAGEADVANCED) as WORD, Some(AdvancedPageProc));

    hpsxa = SHCreatePropSheetExtArray(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER!("\\System"), MAX_SYSTEM_PAGES - psh.nPages);
    if hpsxa != core::ptr::null_mut() {
        SHAddFromPropSheetExtArray(hpsxa, Some(PropSheetAddPage), core::ptr::addr_of_mut!(psh) as LPARAM);
    }

    if nPage != 0 && nPage <= psh.nPages as i32 {
        psh.__bindgen_anon_2.nStartPage = nPage as u32;
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

    UNREFERENCED_PARAMETER!(hwndCPl);

    match uMsg {
        CPL_INIT => {
            return TRUE;
        }

        CPL_GETCOUNT => {
            return (NUM_APPLETS) as i32;
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
                (Applets[i as usize].AppletProc.unwrap_unchecked())(hwndCPl, uMsg, lParam1, lParam2);
            } else {
                return TRUE;
            }
        }

        CPL_STARTWPARMSW => {
            if i < NUM_APPLETS {
                return (Applets[i as usize].AppletProc.unwrap_unchecked())(hwndCPl, uMsg, lParam1, lParam2);
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
    UNREFERENCED_PARAMETER!(lpvReserved);

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

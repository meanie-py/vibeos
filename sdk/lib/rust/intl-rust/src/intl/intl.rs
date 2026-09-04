// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NUM_APPLETS: usize = 1;

pub const BUFFERSIZE: usize = 512;

#[no_mangle]
pub static mut hApplet: HINSTANCE = core::ptr::null_mut();
pub static mut hSetupInf: HINF = INVALID_HANDLE_VALUE as HINF;
#[no_mangle]
pub static mut IsUnattendedSetupEnabled: u32 = 0;
#[no_mangle]
pub static mut UnattendLCID: u32 = 0;
pub static mut Applets: [APPLET; NUM_APPLETS] = [
    APPLET {
        idIcon: IDC_CPLICON,
        idName: IDS_CPLNAME,
        idDescription: IDS_CPLDESCRIPTION,
        AppletProc: Some(Applet),
    }
];
#[no_mangle]
pub unsafe extern "C" fn PrintErrorMsgBox(msg: u32) {
    let mut szErrorText: [u16; BUFFERSIZE] = core::mem::zeroed();
    let mut szErrorCaption: [u16; BUFFERSIZE] = core::mem::zeroed();

    LoadStringW(hApplet, msg, szErrorText.as_mut_ptr(), (core::mem::size_of_val(&szErrorText) / core::mem::size_of::<u16>()) as i32);
    LoadStringW(hApplet, IDS_ERROR, szErrorCaption.as_mut_ptr(), (core::mem::size_of_val(&szErrorCaption) / core::mem::size_of::<u16>()) as i32);

    MessageBoxW(core::ptr::null_mut(), szErrorText.as_ptr(), szErrorCaption.as_ptr(), MB_OK | MB_ICONERROR);
}
#[no_mangle]
pub unsafe extern "C" fn ResourceMessageBox(
    hwnd: HWND,
    uType: u32,
    uCaptionId: u32,
    uMessageId: u32,
) -> i32 {
    let mut szErrorText: [u16; BUFFERSIZE] = [0; BUFFERSIZE];
    let mut szErrorCaption: [u16; BUFFERSIZE] = [0; BUFFERSIZE];

    LoadStringW(
        hApplet,
        uMessageId,
        szErrorText.as_mut_ptr(),
        (core::mem::size_of_val(&szErrorText) / core::mem::size_of::<u16>()) as i32,
    );
    LoadStringW(
        hApplet,
        uCaptionId,
        szErrorCaption.as_mut_ptr(),
        (core::mem::size_of_val(&szErrorCaption) / core::mem::size_of::<u16>()) as i32,
    );

    MessageBoxW(hwnd, szErrorText.as_ptr(), szErrorCaption.as_ptr(), uType)
}
unsafe extern "C" fn InitIntlPropSheetPage(psp: *mut PROPSHEETPAGEW, idDlg: WORD, DlgProc: DLGPROC, lParam: LPARAM) {
    ZeroMemory!(psp as PVOID, core::mem::size_of::<PROPSHEETPAGE>());
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    (*psp).dwFlags = PSP_DEFAULT;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg as UINT_PTR);
    (*psp).pfnDlgProc = DlgProc;
    (*psp).lParam = lParam;
}
#[no_mangle]
pub unsafe extern "C" fn OpenSetupInf() -> BOOL {
    let lpCmdLine: PWSTR;
    let lpSwitch: PWSTR;
    let len: usize;

    lpCmdLine = GetCommandLineW();

    lpSwitch = wcsstr(lpCmdLine, w!("/f:\""));
    if lpSwitch.is_null() {
        return FALSE as BOOL;
    }

    len = wcslen(lpSwitch);
    if len < 5 || *lpSwitch.offset((len - 1) as isize) != 0x0022u16 {
        DPRINT1!("Invalid switch: %ls\n", lpSwitch);
        return FALSE as BOOL;
    }

    *lpSwitch.offset((len - 1) as isize) = 0u16;

    hSetupInf = SetupOpenInfFileW(lpSwitch.offset(4 as isize), core::ptr::null_mut(), INF_STYLE_OLDNT, core::ptr::null_mut());
    if hSetupInf == INVALID_HANDLE_VALUE {
        DPRINT1!("Failed to open INF file: %ls\n", lpSwitch.offset(4));
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ParseSetupInf() {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut szBuffer: [u16; 30] = [0; 30];

    if SetupFindFirstLineW(hSetupInf,
                             w!("Unattend"),
                             w!("LocaleID"),
                             &mut InfContext) == 0
    {
        SetupCloseInfFile(hSetupInf);
        DPRINT1!("SetupFindFirstLine failed\n");
        return;
    }

    if SetupGetStringFieldW(&mut InfContext, 1, szBuffer.as_mut_ptr(),
                              (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as u32, core::ptr::null_mut()) == 0
    {
        SetupCloseInfFile(hSetupInf);
        DPRINT1!("SetupGetStringField failed\n");
        return;
    }

    UnattendLCID = wcstoul(szBuffer.as_ptr(), core::ptr::null_mut(), 16);
    IsUnattendedSetupEnabled = 1;
    SetupCloseInfFile(hSetupInf);
}
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: u32, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDC_CPLICON));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    0
}

unsafe extern "system" fn Applet(hwnd: HWND, uMsg: u32, wParam: LPARAM, lParam: LPARAM) -> i32 {
    let mut psp: [PROPSHEETPAGEW; 3] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let pGlobalData: PGLOBALDATA;
    let mut nPage: i32 = 0;
    let mut ret: i32;

    if OpenSetupInf() != 0 {
        ParseSetupInf();
    }

    if uMsg == CPL_STARTWPARMSW && lParam != 0 {
        nPage = _wtoi(lParam as PWSTR);
    }

    pGlobalData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<GLOBALDATA>()) as SIZE_T) as PGLOBALDATA;
    if pGlobalData.is_null() {
        return FALSE as BOOL;
    }

    (*pGlobalData).SystemLCID = GetSystemDefaultLCID();
    (*pGlobalData).bIsUserAdmin = IsUserAdmin();

    ZeroMemory!(core::ptr::addr_of_mut!(psh) as LPVOID, core::mem::size_of::<PROPSHEETHEADER>());
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK;
    psh.hwndParent = hwnd;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDC_CPLICON);
    psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLNAME);
    psh.nPages = 0;
    psh.__bindgen_anon_2.nStartPage = 0;
    psh.__bindgen_anon_3.ppsp = psp.as_mut_ptr();
    psh.pfnCallback = Some(PropSheetProc);

    InitIntlPropSheetPage(core::ptr::addr_of_mut!(psp[0]), (IDD_GENERALPAGE) as WORD, Some(GeneralPageProc), pGlobalData as LPARAM);
    psh.nPages += 1;
    InitIntlPropSheetPage(core::ptr::addr_of_mut!(psp[1]), (IDD_LANGUAGESPAGE) as WORD, Some(LanguagesPageProc), pGlobalData as LPARAM);
    psh.nPages += 1;

    if (*pGlobalData).bIsUserAdmin != 0 {
        InitIntlPropSheetPage(core::ptr::addr_of_mut!(psp[2]), (IDD_ADVANCEDPAGE) as WORD, Some(AdvancedPageProc), pGlobalData as LPARAM);
        psh.nPages += 1;
    }

    if nPage != 0 && nPage <= psh.nPages as i32 {
        psh.__bindgen_anon_2.nStartPage = nPage as u32;
    }

    ret = PropertySheet(core::ptr::addr_of_mut!(psh)) as i32;
    if ret != -1 {
        ret = 1;
    } else {
        ret = 0;
    }
    if ret > 0 {
        SendMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, w!("intl") as LPARAM);
    }

    HeapFree(GetProcessHeap(), 0, pGlobalData as LPVOID);

    ret
}


#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCpl: HWND,
    uMsg: u32,
    lParam1: LPARAM,
    lParam2: LPARAM,
) -> i32 {
    let i = lParam1 as u32;

    match uMsg {
        CPL_INIT => {
            return (TRUE) as i32;
        }

        CPL_GETCOUNT => {
            return (NUM_APPLETS) as i32;
        }

        CPL_INQUIRE => {
            if i < (NUM_APPLETS) as u32 {
                let CPlInfo = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0;
                (*CPlInfo).idIcon = (Applets[i as usize].idIcon) as i32;
                (*CPlInfo).idName = (Applets[i as usize].idName) as i32;
                (*CPlInfo).idInfo = (Applets[i as usize].idDescription) as i32;
            } else {
                return (TRUE) as i32;
            }
        }

        CPL_DBLCLK => {
            if i < (NUM_APPLETS) as u32 {
                (Applets[i as usize].AppletProc.unwrap_unchecked())(hwndCpl, uMsg, lParam1, lParam2);
            } else {
                return (TRUE) as i32;
            }
        }

        CPL_STARTWPARMSW => {
            if i < (NUM_APPLETS) as u32 {
                return (Applets[i as usize].AppletProc.unwrap_unchecked())(hwndCpl, uMsg, lParam1, lParam2);
            }
        }

        _ => {}
    }

    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: u32,
    lpReserved: LPVOID,
) -> i32 {
    match dwReason {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH => {
            hApplet = hinstDLL;
        }
        _ => {}
    }

    (TRUE) as i32
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
//  * FILE:            dll/cpl/intl/intl.c
//  * PURPOSE:         Property sheet code
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "intl.h"
//
// #include <debug.h>
//
// #define NUM_APPLETS    (1)
//
// #define BUFFERSIZE 512
//
// static LONG __stdcall
// Applet(HWND hwnd, UINT uMsg, LPARAM wParam, LPARAM lParam);
//
//
// HINSTANCE hApplet = 0;
// HINF hSetupInf = INVALID_HANDLE_VALUE;
// DWORD IsUnattendedSetupEnabled = 0;
// DWORD UnattendLCID = 0;
//
//
// /* Applets */
// APPLET Applets[NUM_APPLETS] =
// {
//     {IDC_CPLICON, IDS_CPLNAME, IDS_CPLDESCRIPTION, Applet}
// };
//
// VOID
// PrintErrorMsgBox(UINT msg)
// {
//     WCHAR szErrorText[BUFFERSIZE];
//     WCHAR szErrorCaption[BUFFERSIZE];
//
//     LoadStringW(hApplet, msg, szErrorText, sizeof(szErrorText) / sizeof(WCHAR));
//     LoadStringW(hApplet, IDS_ERROR, szErrorCaption, sizeof(szErrorCaption) / sizeof(WCHAR));
//
//     MessageBoxW(NULL, szErrorText, szErrorCaption, MB_OK | MB_ICONERROR);
// }
//
// INT
// ResourceMessageBox(
//     HWND hwnd,
//     UINT uType,
//     UINT uCaptionId,
//     UINT uMessageId)
// {
//     WCHAR szErrorText[BUFFERSIZE];
//     WCHAR szErrorCaption[BUFFERSIZE];
//
//     LoadStringW(hApplet, uMessageId, szErrorText, sizeof(szErrorText) / sizeof(WCHAR));
//     LoadStringW(hApplet, uCaptionId, szErrorCaption, sizeof(szErrorCaption) / sizeof(WCHAR));
//
//     return MessageBoxW(hwnd, szErrorText, szErrorCaption, uType);
// }
//
// static VOID
// InitIntlPropSheetPage(PROPSHEETPAGE *psp, WORD idDlg, DLGPROC DlgProc, LPARAM lParam)
// {
//     ZeroMemory(psp, sizeof(PROPSHEETPAGE));
//     psp->dwSize = sizeof(PROPSHEETPAGE);
//     psp->dwFlags = PSP_DEFAULT;
//     psp->hInstance = hApplet;
//     psp->pszTemplate = MAKEINTRESOURCE(idDlg);
//     psp->pfnDlgProc = DlgProc;
//     psp->lParam = lParam;
// }
//
// BOOL
// OpenSetupInf(VOID)
// {
//     PWSTR lpCmdLine;
//     PWSTR lpSwitch;
//     size_t len;
//
//     lpCmdLine = GetCommandLineW();
//
//     lpSwitch = wcsstr(lpCmdLine, L"/f:\"");
//     if (!lpSwitch)
//         return FALSE;
//
//     len = wcslen(lpSwitch);
//     if (len < 5 || lpSwitch[len-1] != L'\"')
//     {
//         DPRINT1("Invalid switch: %ls\n", lpSwitch);
//         return FALSE;
//     }
//
//     lpSwitch[len-1] = L'\0';
//
//     hSetupInf = SetupOpenInfFileW(&lpSwitch[4], NULL, INF_STYLE_OLDNT, NULL);
//     if (hSetupInf == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("Failed to open INF file: %ls\n", &lpSwitch[4]);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// VOID
// ParseSetupInf(VOID)
// {
//     INFCONTEXT InfContext;
//     WCHAR szBuffer[30];
//
//     if (!SetupFindFirstLineW(hSetupInf,
//                              L"Unattend",
//                              L"LocaleID",
//                              &InfContext))
//     {
//         SetupCloseInfFile(hSetupInf);
//         DPRINT1("SetupFindFirstLine failed\n");
//         return;
//     }
//
//     if (!SetupGetStringFieldW(&InfContext, 1, szBuffer,
//                               sizeof(szBuffer) / sizeof(WCHAR), NULL))
//     {
//         SetupCloseInfFile(hSetupInf);
//         DPRINT1("SetupGetStringField failed\n");
//         return;
//     }
//
//     UnattendLCID = wcstoul(szBuffer, NULL, 16);
//     IsUnattendedSetupEnabled = 1;
//     SetupCloseInfFile(hSetupInf);
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
//             hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW(IDC_CPLICON));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// static LONG __stdcall
// Applet(HWND hwnd, UINT uMsg, LPARAM wParam, LPARAM lParam)
// {
//     PROPSHEETPAGE psp[3];
//     PROPSHEETHEADER psh;
//     PGLOBALDATA pGlobalData;
//     INT nPage = 0;
//     LONG ret;
//
//     if (OpenSetupInf())
//     {
//         ParseSetupInf();
//     }
//
//     if (uMsg == CPL_STARTWPARMSW && lParam != 0)
//         nPage = _wtoi((PWSTR)lParam);
//
//     pGlobalData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(GLOBALDATA));
//     if (pGlobalData == NULL)
//         return FALSE;
//
//     pGlobalData->SystemLCID = GetSystemDefaultLCID();
//     pGlobalData->bIsUserAdmin = IsUserAdmin();
//
//     ZeroMemory(&psh, sizeof(PROPSHEETHEADER));
//     psh.dwSize = sizeof(PROPSHEETHEADER);
//     psh.dwFlags =  PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK;
//     psh.hwndParent = hwnd;
//     psh.hInstance = hApplet;
//     psh.pszIcon = MAKEINTRESOURCEW(IDC_CPLICON);
//     psh.pszCaption = MAKEINTRESOURCEW(IDS_CPLNAME);
//     psh.nPages = 0;
//     psh.nStartPage = 0;
//     psh.ppsp = psp;
//     psh.pfnCallback = PropSheetProc;
//
//     InitIntlPropSheetPage(&psp[0], IDD_GENERALPAGE, GeneralPageProc, (LPARAM)pGlobalData);
//     psh.nPages++;
//     InitIntlPropSheetPage(&psp[1], IDD_LANGUAGESPAGE, LanguagesPageProc, (LPARAM)pGlobalData);
//     psh.nPages++;
//
//     if (pGlobalData->bIsUserAdmin)
//     {
//         InitIntlPropSheetPage(&psp[2], IDD_ADVANCEDPAGE, AdvancedPageProc, (LPARAM)pGlobalData);
//         psh.nPages++;
//     }
//
//     if (nPage != 0 && nPage <= psh.nPages)
//         psh.nStartPage = nPage;
//
//     ret = (LONG)(PropertySheet(&psh) != -1);
//     if (ret > 0)
//         SendMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)L"intl");
//
//     HeapFree(GetProcessHeap(), 0, pGlobalData);
//
//     return ret;
// }
//
//
// /* Control Panel Callback */
// LONG __stdcall
// CPlApplet(HWND hwndCpl,
//           UINT uMsg,
//           LPARAM lParam1,
//           LPARAM lParam2)
// {
//     UINT i = (UINT)lParam1;
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
//                 CPLINFO *CPlInfo = (CPLINFO*)lParam2;
//                 CPlInfo->lData = 0;
//                 CPlInfo->idIcon = Applets[i].idIcon;
//                 CPlInfo->idName = Applets[i].idName;
//                 CPlInfo->idInfo = Applets[i].idDescription;
//             }
//             else
//             {
//                 return TRUE;
//             }
//             break;
//
//         case CPL_DBLCLK:
//             if (i < NUM_APPLETS)
//                 Applets[i].AppletProc(hwndCpl, uMsg, lParam1, lParam2);
//             else
//                 return TRUE;
//             break;
//
//         case CPL_STARTWPARMSW:
//             if (i < NUM_APPLETS)
//                 return Applets[i].AppletProc(hwndCpl, uMsg, lParam1, lParam2);
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
//         LPVOID lpReserved)
// {
//     switch(dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//         case DLL_THREAD_ATTACH:
//             hApplet = hinstDLL;
//             break;
//     }
//
//   return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

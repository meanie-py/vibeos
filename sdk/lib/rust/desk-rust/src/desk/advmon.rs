// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_ADVANCED_PAGES: i32 = 32;
unsafe extern "system" fn PropSheetAddPage(hpage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let ppsh = lParam as *mut PROPSHEETHEADER;
    if !ppsh.is_null() && (*ppsh).nPages < (MAX_ADVANCED_PAGES) as u32 {
        let idx = (*ppsh).nPages as usize;
        *(*ppsh).__bindgen_anon_3.phpage.add(idx) = hpage;
        (*ppsh).nPages += 1;
        return (TRUE) as i32;
    }

    (FALSE) as i32
}
unsafe extern "C" fn DisplayAdvancedSettingsInitPropSheetPage(ppsh: *mut PROPSHEETHEADER, idDlg: WORD, DlgProc: DLGPROC, lParam: LPARAM) -> BOOL {
    let mut hPage: HPROPSHEETPAGE;
    let mut psp: PROPSHEETPAGE;

    if (*ppsh).nPages < (MAX_ADVANCED_PAGES) as u32 {
        ZeroMemory!(core::ptr::addr_of_mut!(psp) as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETPAGE>());
        psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
        psp.dwFlags = PSP_DEFAULT;
        psp.hInstance = hApplet;
        psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg);
        psp.pfnDlgProc = DlgProc;
        psp.lParam = lParam;

        hPage = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));
        if !hPage.is_null() {
            return PropSheetAddPage(hPage, ppsh as LPARAM);
        }
    }

    return (FALSE) as i32;
}
unsafe extern "C" fn BuildAdvPropTitle(pdo: *mut IDataObject, lpBuffer: LPTSTR, dwBufferLen: u32) {
    let mut uiMonitorName: u32;
    let mut uiDisplayName: u32;
    let mut lpMonitorName: LPTSTR;
    let mut lpDisplayName: LPTSTR;
    let mut szFormatBuff: [TCHAR; 32] = [0; 32];

    if LoadString(hApplet, IDS_ADVANCEDTITLEFMT, szFormatBuff.as_mut_ptr(), (_countof!(szFormatBuff)) as i32) == 0 {
        szFormatBuff[0] = ('\0' as WCHAR);
    }

    uiMonitorName = RegisterClipboardFormat(DESK_EXT_MONITORNAME!());
    uiDisplayName = RegisterClipboardFormat(DESK_EXT_DISPLAYNAME!());

    lpMonitorName = QueryDeskCplString(pdo, uiMonitorName);
    lpDisplayName = QueryDeskCplString(pdo, uiDisplayName);

    _sntprintf(lpBuffer, (dwBufferLen) as usize, szFormatBuff.as_ptr(), lpMonitorName, lpDisplayName);

    if lpMonitorName != core::ptr::null_mut() {
        LocalFree(lpMonitorName as HLOCAL);
    }
    if lpDisplayName != core::ptr::null_mut() {
        LocalFree(lpDisplayName as HLOCAL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DisplayAdvancedSettings(hWndParent: HWND, DisplayDevice: PDISPLAY_DEVICE_ENTRY) -> BOOL {
    let mut szCaption: [TCHAR; 128] = [0; 128];
    let mut hpsp: [HPROPSHEETPAGE; (MAX_ADVANCED_PAGES) as usize] = [core::ptr::null_mut(); (MAX_ADVANCED_PAGES) as usize];
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut hpsxaDev: HPSXA;
    let mut hpsxaDisp: HPSXA;
    let Ret: BOOL;
    let pdo: *mut IDataObject;

    szCaption[0] = ('\0' as WCHAR);

    ZeroMemory!(&mut psh as *mut _ as *mut core::ffi::c_void, sizeof::<PROPSHEETHEADER>());
    psh.dwSize = sizeof::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = PSH_PROPTITLE;
    psh.hwndParent = hWndParent;
    psh.hInstance = hApplet;
    psh.pszCaption = szCaption.as_mut_ptr();
    psh.__bindgen_anon_3.phpage = hpsp.as_mut_ptr();

    DisplayAdvancedSettingsInitPropSheetPage(&mut psh, (IDD_ADVANCED_GENERAL) as WORD, AdvGeneralPageProc, DisplayDevice as LPARAM);

    pdo = CreateDevSettings(DisplayDevice);

    if pdo != core::ptr::null_mut() {
        BuildAdvPropTitle(pdo, szCaption.as_mut_ptr(), _countof!(szCaption));
    }

    hpsxaDev = SHCreatePropSheetExtArrayEx(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER!("\\Device"), (MAX_ADVANCED_PAGES - (psh.nPages) as i32) as UINT, pdo);
    if hpsxaDev != core::ptr::null_mut() {
        SHAddFromPropSheetExtArray(hpsxaDev, PropSheetAddPage, &mut psh as *mut _ as LPARAM);
    }

    hpsxaDisp = SHCreatePropSheetExtArrayEx(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER!("\\Display"), (MAX_ADVANCED_PAGES - (psh.nPages) as i32) as UINT, pdo);
    if hpsxaDisp != core::ptr::null_mut() {
        SHAddFromPropSheetExtArray(hpsxaDisp, PropSheetAddPage, &mut psh as *mut _ as LPARAM);
    }

    Ret = (PropertySheet(&mut psh) != -(1 as i32)) as LONG as BOOL;

    if hpsxaDisp != core::ptr::null_mut() {
        SHDestroyPropSheetExtArray(hpsxaDisp);
    }

    if hpsxaDev != core::ptr::null_mut() {
        SHDestroyPropSheetExtArray(hpsxaDev);
    }

    IDataObject_Release(pdo);

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * PURPOSE:         Advanced monitor/display settings
//  */
//
// #include "desk.h"
//
// #define MAX_ADVANCED_PAGES  32
//
// static BOOL __stdcall
// PropSheetAddPage(HPROPSHEETPAGE hpage, LPARAM lParam)
// {
//     PROPSHEETHEADER *ppsh = (PROPSHEETHEADER *)lParam;
//     if (ppsh != NULL && ppsh->nPages < MAX_ADVANCED_PAGES)
//     {
//         ppsh->phpage[ppsh->nPages++] = hpage;
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static BOOL
// DisplayAdvancedSettingsInitPropSheetPage(PROPSHEETHEADER *ppsh, WORD idDlg, DLGPROC DlgProc, LPARAM lParam)
// {
//     HPROPSHEETPAGE hPage;
//     PROPSHEETPAGE psp;
//
//     if (ppsh->nPages < MAX_ADVANCED_PAGES)
//     {
//         ZeroMemory(&psp, sizeof(psp));
//         psp.dwSize = sizeof(psp);
//         psp.dwFlags = PSP_DEFAULT;
//         psp.hInstance = hApplet;
//         psp.pszTemplate = MAKEINTRESOURCE(idDlg);
//         psp.pfnDlgProc = DlgProc;
//         psp.lParam = lParam;
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
// static VOID
// BuildAdvPropTitle(IDataObject *pdo, LPTSTR lpBuffer, DWORD dwBufferLen)
// {
//     UINT uiMonitorName, uiDisplayName;
//     LPTSTR lpMonitorName, lpDisplayName;
//     TCHAR szFormatBuff[32];
//
//     if (!LoadString(hApplet, IDS_ADVANCEDTITLEFMT, szFormatBuff, _countof(szFormatBuff)))
//     {
//         szFormatBuff[0] = _T('\0');
//     }
//
//     uiMonitorName = RegisterClipboardFormat(DESK_EXT_MONITORNAME);
//     uiDisplayName = RegisterClipboardFormat(DESK_EXT_DISPLAYNAME);
//
//     lpMonitorName = QueryDeskCplString(pdo, uiMonitorName);
//     lpDisplayName = QueryDeskCplString(pdo, uiDisplayName);
//
//     _sntprintf(lpBuffer, dwBufferLen, szFormatBuff, lpMonitorName, lpDisplayName);
//
//     if (lpMonitorName != NULL)
//         LocalFree((HLOCAL)lpMonitorName);
//     if (lpDisplayName != NULL)
//         LocalFree((HLOCAL)lpDisplayName);
// }
//
// BOOL
// DisplayAdvancedSettings(HWND hWndParent, PDISPLAY_DEVICE_ENTRY DisplayDevice)
// {
//     TCHAR szCaption[128];
//     HPROPSHEETPAGE hpsp[MAX_ADVANCED_PAGES];
//     PROPSHEETHEADER psh;
//     HPSXA hpsxaDev, hpsxaDisp;
//     BOOL Ret;
//     IDataObject *pdo;
//
//     /* FIXME: Build the "%s and %s" caption string for the monitor and adapter name */
//     szCaption[0] = _T('\0');
//
//     ZeroMemory(&psh, sizeof(PROPSHEETHEADER));
//     psh.dwSize = sizeof(PROPSHEETHEADER);
//     psh.dwFlags = PSH_PROPTITLE;
//     psh.hwndParent = hWndParent;
//     psh.hInstance = hApplet;
//     psh.pszCaption = szCaption;
//     psh.phpage = hpsp;
//
//     DisplayAdvancedSettingsInitPropSheetPage(&psh, IDD_ADVANCED_GENERAL, AdvGeneralPageProc, (LPARAM)DisplayDevice);
//
//     pdo = CreateDevSettings(DisplayDevice);
//
//     if (pdo != NULL)
//         BuildAdvPropTitle(pdo, szCaption, _countof(szCaption));
//
//     hpsxaDev = SHCreatePropSheetExtArrayEx(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER TEXT("\\Device"), MAX_ADVANCED_PAGES - psh.nPages, pdo);
//     if (hpsxaDev != NULL)
//         SHAddFromPropSheetExtArray(hpsxaDev, PropSheetAddPage, (LPARAM)&psh);
//
//     hpsxaDisp = SHCreatePropSheetExtArrayEx(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER TEXT("\\Display"), MAX_ADVANCED_PAGES - psh.nPages, pdo);
//     if (hpsxaDisp != NULL)
//         SHAddFromPropSheetExtArray(hpsxaDisp, PropSheetAddPage, (LPARAM)&psh);
//
//     Ret = (LONG)(PropertySheet(&psh) != -1);
//
//     if (hpsxaDisp != NULL)
//         SHDestroyPropSheetExtArray(hpsxaDisp);
//
//     if (hpsxaDev != NULL)
//         SHDestroyPropSheetExtArray(hpsxaDev);
//
//     IDataObject_Release(pdo);
//
//     return Ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

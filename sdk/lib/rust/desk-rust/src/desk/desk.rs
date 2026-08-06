// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hApplet: HINSTANCE = core::ptr::null_mut();
pub static mut hCPLWindow: HWND = core::ptr::null_mut();

pub static mut Applets: [APPLET; 1] = [
    APPLET {
        idIcon: (IDC_DESK_ICON) as i32,
        idName: (IDS_CPLNAME) as i32,
        idDescription: (IDS_CPLDESCRIPTION) as i32,
        AppletProc: Some(DisplayApplet),
    }
];

#[repr(C)]
pub struct PropPageEntry {
    pub idDlg: u16,
    pub DlgProc: DLGPROC,
    pub Callback: LPFNPSPCALLBACK,
    pub Name: LPWSTR,
}

pub static PropPages: SyncTable<[PropPageEntry; 4]> = SyncTable([
    PropPageEntry {
        idDlg: (IDD_BACKGROUND) as u16,
        DlgProc: Some(BackgroundPageProc),
        Callback: None,
        Name: w!("Desktop") as LPWSTR,
    },
    PropPageEntry {
        idDlg: (IDD_SCREENSAVER) as u16,
        DlgProc: Some(ScreenSaverPageProc),
        Callback: None,
        Name: w!("Screen Saver") as LPWSTR,
    },
    PropPageEntry {
        idDlg: (IDD_APPEARANCE) as u16,
        DlgProc: Some(AppearancePageProc),
        Callback: None,
        Name: w!("Appearance") as LPWSTR,
    },
    PropPageEntry {
        idDlg: (IDD_SETTINGS) as u16,
        DlgProc: Some(SettingsPageProc),
        Callback: Some(SettingsPageCallbackProc),
        Name: w!("Settings") as LPWSTR,
    },
]);
#[no_mangle]
pub unsafe extern "C" fn LoadPopupMenu(
    hInstance: HINSTANCE,
    lpMenuName: LPCTSTR,
) -> HMENU {
    let mut hSubMenu: HMENU = core::ptr::null_mut();

    let hMenu = LoadMenu(hInstance, lpMenuName);

    if !hMenu.is_null() {
        hSubMenu = GetSubMenu(hMenu, 0);
        if !hSubMenu.is_null()
            && RemoveMenu(
                hMenu,
                0,
                MF_BYPOSITION,
            ) == 0
        {
            hSubMenu = core::ptr::null_mut();
        }

        DestroyMenu(hMenu);
    }

    hSubMenu
}
unsafe extern "system" fn DisplayAppletPropSheetAddPage(
    hpage: HPROPSHEETPAGE,
    lParam: LPARAM,
) -> i32 {
    let ppsh = lParam as *mut PROPSHEETHEADER;
    if !ppsh.is_null() && (*ppsh).nPages < MAX_DESK_PAGES {
        *(*ppsh).__bindgen_anon_3.phpage.offset((*ppsh).nPages as isize) = hpage;
        (*ppsh).nPages = (*ppsh).nPages + 1;
        return (TRUE) as i32;
    }

    (FALSE) as i32
}
unsafe extern "C" fn InitPropSheetPage(
    ppsh: *mut PROPSHEETHEADER,
    idDlg: u16,
    DlgProc: DLGPROC,
    pfnCallback: LPFNPSPCALLBACK,
) -> i32 {
    let mut hPage: HPROPSHEETPAGE;
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();

    if (*ppsh).nPages < MAX_DESK_PAGES {
        ZeroMemory!(
            &mut psp as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<PROPSHEETPAGE>(),
        );
        psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
        psp.dwFlags = PSP_DEFAULT;
        if pfnCallback.is_some() {
            psp.dwFlags |= PSP_USECALLBACK;
        }
        psp.hInstance = hApplet;
        psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg);
        psp.pfnDlgProc = DlgProc;
        psp.pfnCallback = pfnCallback;

        hPage = CreatePropertySheetPage(&mut psp);
        if !hPage.is_null() {
            return DisplayAppletPropSheetAddPage(hPage, ppsh as LPARAM);
        }
    }

    return (FALSE) as i32;
}
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: u32, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDC_DESK_ICON));
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}
unsafe extern "system" fn DisplayApplet(hwnd: HWND, uMsg: u32, wParam: LPARAM, lParam: LPARAM) -> i32 {
    let mut hpsp: [*mut core::ffi::c_void; (MAX_DESK_PAGES) as usize] = [core::ptr::null_mut(); (MAX_DESK_PAGES) as usize];
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut hpsxa: HPSXA = core::ptr::null_mut();
    let mut i: u32;
    let mut argv: *mut LPWSTR = core::ptr::null_mut();
    let mut pwszSelectedTab: LPCWSTR = core::ptr::null();
    let mut pwszFile: LPCWSTR = core::ptr::null();
    let mut pwszAction: LPCWSTR = core::ptr::null();
    let mut nPage: i32 = 0;
    let mut bitmap: BITMAP = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(wParam);

    hCPLWindow = hwnd;

    if uMsg == CPL_STARTWPARMSW && lParam != 0 {
        let mut argc: i32 = 0;
        let mut i: i32;

        nPage = _wtoi(lParam as PWSTR);

        argv = CommandLineToArgvW(lParam as LPCWSTR, &mut argc);

        if !argv.is_null() && argc != 0 {
            i = 0;
            while i < argc {
                if *(*argv.offset(i as isize)).offset(0 as isize) == '@' as u16 {
                    pwszSelectedTab = (*argv.offset(i as isize)).offset(1 as isize);
                } else if wcsncmp(*argv.offset(i as isize), w!("/Action:") as LPCWSTR, 8) == 0 {
                    pwszAction = (*argv.offset(i as isize)).offset(8 as isize);
                } else if wcsncmp(*argv.offset(i as isize), w!("/file:") as LPCWSTR, 6) == 0 {
                    pwszFile = (*argv.offset(i as isize)).offset(6 as isize);
                }
                i += 1;
            }
        }
    }

    'cleanup: loop {
        if !pwszAction.is_null() && wcsncmp(pwszAction, w!("ActivateMSTheme") as LPCWSTR, 15) == 0 {
            ActivateThemeFile(pwszFile);
            break 'cleanup;
        }

        g_GlobalData.pwszFile = pwszFile;
        g_GlobalData.pwszAction = pwszAction;
        g_GlobalData.desktop_color = GetSysColor((COLOR_DESKTOP) as i32);

        g_GlobalData.hMonitorBitmap = LoadBitmapW(hApplet, MAKEINTRESOURCEW!(IDC_MONITOR));
        if !g_GlobalData.hMonitorBitmap.is_null() {
            GetObjectW((g_GlobalData.hMonitorBitmap).cast(), core::mem::size_of::<BITMAP>() as i32, &mut bitmap as *mut _ as LPVOID);

            g_GlobalData.bmMonWidth = bitmap.bmWidth;
            g_GlobalData.bmMonHeight = bitmap.bmHeight;
        }

        ZeroMemory!(&mut psh as *mut _ as LPVOID, core::mem::size_of::<PROPSHEETHEADER>());
        psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
        psh.dwFlags = PSH_USECALLBACK | PSH_PROPTITLE | PSH_USEICONID;
        psh.hwndParent = hCPLWindow;
        psh.hInstance = hApplet;
        psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDC_DESK_ICON);
        psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLNAME);
        psh.nPages = 0;
        psh.__bindgen_anon_2.nStartPage = 0;
        psh.__bindgen_anon_3.phpage = (hpsp.as_mut_ptr()).cast();
        psh.pfnCallback = Some(PropSheetProc);

        hpsxa = SHCreatePropSheetExtArray(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER!("\\Desk"), MAX_DESK_PAGES - psh.nPages);

        i = 0;
        while i < _countof!(PropPages) {
            if !pwszSelectedTab.is_null() && _wcsicmp(pwszSelectedTab, PropPages[i as usize].Name) == 0 {
                psh.__bindgen_anon_2.nStartPage = i;
            }

            if PropPages[i as usize].idDlg == (IDD_BACKGROUND) as u16 && !hpsxa.is_null() &&
                SHReplaceFromPropSheetExtArray(hpsxa, CPLPAGE_DISPLAY_BACKGROUND, Some(DisplayAppletPropSheetAddPage), &mut psh as *mut _ as LPARAM) != 0 {
                i += 1;
                continue;
            }

            InitPropSheetPage(&mut psh, PropPages[i as usize].idDlg, PropPages[i as usize].DlgProc, PropPages[i as usize].Callback);
            i += 1;
        }

        if nPage != 0 && psh.__bindgen_anon_2.nStartPage == 0 {
            psh.__bindgen_anon_2.nStartPage = nPage as u32;
        }

        PropertySheet(&mut psh);

        break;
    }

    DeleteObject((g_GlobalData.hMonitorBitmap).cast());

    if !hpsxa.is_null() {
        SHDestroyPropSheetExtArray(hpsxa);
    }

    if !argv.is_null() {
        LocalFree(argv as LPVOID);
    }

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCPl: HWND,
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
            return _countof!(Applets) as i32;
        }

        CPL_INQUIRE => {
            if (i as usize) < _countof!(Applets) as usize {
                let CPlInfo = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon;
                (*CPlInfo).idName = Applets[i as usize].idName;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription;
            } else {
                return (TRUE) as i32;
            }
        }

        CPL_DBLCLK => {
            if (i as usize) < _countof!(Applets) as usize {
                (Applets[i as usize].AppletProc.unwrap())(hwndCPl, uMsg, lParam1, lParam2);
            } else {
                return (TRUE) as i32;
            }
        }

        CPL_STARTWPARMSW => {
            if (i as usize) < _countof!(Applets) as usize {
                return (Applets[i as usize].AppletProc.unwrap())(hwndCPl, uMsg, lParam1, lParam2);
            }
        }

        _ => {}
    }

    return (FALSE) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn InstallScreenSaverW(
    hWindow: HWND,
    hInstance: HINSTANCE,
    pszFile: LPCWSTR,
    nCmdShow: u32,
) {
    let mut rc: LRESULT;
    let mut regKey: HKEY = core::ptr::null_mut();
    let mut Timeout: i32;
    let mut dwLen: u32;
    let mut szFullPath: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    if pszFile.is_null() {
        // DPRINT removed
        SetLastError(ERROR_INVALID_PARAMETER);
        return;
    }
    // DPRINT removed

    rc = (RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Control Panel\\Desktop") as LPCWSTR,
        0,
        KEY_SET_VALUE,
        &mut regKey,
    )) as LPARAM;
    if rc == ((ERROR_SUCCESS) as i32) as LPARAM {
        let Length: usize = (wcslen(pszFile) + 1) * core::mem::size_of::<u16>();
        rc = (RegSetValueExW(
            regKey,
            w!("SCRNSAVE.EXE") as LPCWSTR,
            0,
            REG_SZ,
            pszFile as PBYTE,
            Length as u32,
        )) as LPARAM;
        RegCloseKey(regKey);
    }
    if rc != ((ERROR_SUCCESS) as i32) as LPARAM {
        // DPRINT removed
        return;
    }

    SystemParametersInfoW(
        SPI_SETSCREENSAVEACTIVE,
        (TRUE) as UINT,
        (0) as *mut c_void,
        SPIF_UPDATEINIFILE,
    );

    Timeout = 0;
    if SystemParametersInfoW(
        SPI_GETSCREENSAVETIMEOUT,
        0,
        &mut Timeout as *mut _ as PVOID,
        0,
    ) == 0 || Timeout <= 0
    {
        SystemParametersInfoW(
            SPI_SETSCREENSAVETIMEOUT,
            600,
            (0) as *mut c_void,
            SPIF_UPDATEINIFILE,
        );
    }

    dwLen = GetModuleFileNameW(hApplet, szFullPath.as_mut_ptr(), _countof!(szFullPath));
    if dwLen == 0 || GetLastError() == ERROR_INSUFFICIENT_BUFFER {
        StringCchCopyW(
            szFullPath.as_mut_ptr(),
            (_countof!(szFullPath)) as usize,
            w!("desk.cpl") as LPCWSTR,
        );
    }

    rc = (StringCchCatW(
        szFullPath.as_mut_ptr(),
        (_countof!(szFullPath)) as usize,
        w!(",,1") as LPCWSTR,
    )) as LPARAM;
    if rc < 0 {
        return;
    }

    // DPRINT removed
    Control_RunDLLW(hWindow, hInstance, szFullPath.as_ptr(), (nCmdShow) as i32);
}
#[no_mangle]
pub unsafe extern "system" fn InstallScreenSaverA(
    hWindow: HWND,
    hInstance: HINSTANCE,
    pszFile: LPCSTR,
    nCmdShow: u32,
) {
    let mut lpwString: LPWSTR;
    let mut nLength: i32;

    if pszFile.is_null() {
        // DPRINT removed
        SetLastError(ERROR_INVALID_PARAMETER);
        return;
    }

    lpwString = core::ptr::null_mut();
    nLength = MultiByteToWideChar(
        CP_ACP,
        0,
        pszFile,
        -1,
        core::ptr::null_mut(),
        0,
    );
    if nLength != 0 {
        lpwString = LocalAlloc(
            LMEM_FIXED,
            ((nLength as usize) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPWSTR;
        if !lpwString.is_null() {
            if MultiByteToWideChar(
                CP_ACP,
                0,
                pszFile,
                -1,
                lpwString,
                nLength,
            ) == 0
            {
                LocalFree(lpwString as LPVOID);
                lpwString = core::ptr::null_mut();
            }
        }
    }
    if lpwString.is_null() {
        // DPRINT removed
        return;
    }

    InstallScreenSaverW(hWindow, hInstance, lpwString, nCmdShow);

    LocalFree(lpwString as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hInstDLL: HINSTANCE,
    dwReason: u32,
    lpvReserved: LPVOID,
) -> i32 {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH => {
            CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED);
            RegisterPreviewControl(hInstDLL);
            hApplet = hInstDLL;
        }

        DLL_PROCESS_DETACH => {
            UnregisterPreviewControl(hInstDLL);
            CoUninitialize();
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
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/desk.c
//  * PURPOSE:         ReactOS Display Control Panel
//  *
//  * PROGRAMMERS:     Trevor McCort (lycan359@gmail.com)
//  */
//
// #include "desk.h"
//
// #include <shellapi.h>
// #include <cplext.h>
// #include <winnls.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* Enable this for InstallScreenSaverW() to determine a possible full path
//  * to the specified screensaver file, verify its existence and use it.
//  * (NOTE: This is not Windows desk.cpl-compatible.) */
// // #define CHECK_SCR_FULL_PATH
//
//
// VOID WINAPI Control_RunDLLW(HWND hWnd, HINSTANCE hInst, LPCWSTR cmd, DWORD nCmdShow);
//
// static LONG __stdcall DisplayApplet(HWND hwnd, UINT uMsg, LPARAM wParam, LPARAM lParam);
//
// INT_PTR __stdcall ThemesPageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam);
// INT_PTR __stdcall BackgroundPageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam);
// INT_PTR __stdcall ScreenSaverPageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam);
// INT_PTR __stdcall AppearancePageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam);
// INT_PTR __stdcall SettingsPageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam);
// UINT __stdcall SettingsPageCallbackProc(HWND hwnd, UINT uMsg, LPPROPSHEETPAGE ppsp);
//
// HINSTANCE hApplet = NULL;
// HWND hCPLWindow = NULL;
//
// /* Applets */
// APPLET Applets[] =
// {
//     {
//         IDC_DESK_ICON,
//         IDS_CPLNAME,
//         IDS_CPLDESCRIPTION,
//         DisplayApplet
//     }
// };
//
// HMENU
// LoadPopupMenu( HINSTANCE hInstance,
//                LPCTSTR lpMenuName)
// {
//     HMENU hMenu, hSubMenu = NULL;
//
//     hMenu = LoadMenu(hInstance,
//                      lpMenuName);
//
//     if (hMenu != NULL)
//     {
//         hSubMenu = GetSubMenu(hMenu,
//                               0);
//         if (hSubMenu != NULL &&
//             !RemoveMenu(hMenu,
//                         0,
//                         MF_BYPOSITION))
//         {
//             hSubMenu = NULL;
//         }
//
//         DestroyMenu(hMenu);
//     }
//
//     return hSubMenu;
// }
//
// static BOOL __stdcall
// DisplayAppletPropSheetAddPage(HPROPSHEETPAGE hpage, LPARAM lParam)
// {
//     PROPSHEETHEADER *ppsh = (PROPSHEETHEADER *)lParam;
//     if (ppsh != NULL && ppsh->nPages < MAX_DESK_PAGES)
//     {
//         ppsh->phpage[ppsh->nPages++] = hpage;
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static BOOL
// InitPropSheetPage(PROPSHEETHEADER *ppsh, WORD idDlg, DLGPROC DlgProc, LPFNPSPCALLBACK pfnCallback)
// {
//     HPROPSHEETPAGE hPage;
//     PROPSHEETPAGE psp;
//
//     if (ppsh->nPages < MAX_DESK_PAGES)
//     {
//         ZeroMemory(&psp, sizeof(psp));
//         psp.dwSize = sizeof(psp);
//         psp.dwFlags = PSP_DEFAULT;
//         if (pfnCallback != NULL)
//             psp.dwFlags |= PSP_USECALLBACK;
//         psp.hInstance = hApplet;
//         psp.pszTemplate = MAKEINTRESOURCE(idDlg);
//         psp.pfnDlgProc = DlgProc;
//         psp.pfnCallback = pfnCallback;
//
//         hPage = CreatePropertySheetPage(&psp);
//         if (hPage != NULL)
//         {
//             return DisplayAppletPropSheetAddPage(hPage, (LPARAM)ppsh);
//         }
//     }
//
//     return FALSE;
// }
//
// static const struct
// {
//     WORD idDlg;
//     DLGPROC DlgProc;
//     LPFNPSPCALLBACK Callback;
//     LPWSTR Name;
// } PropPages[] =
// {
//     /* { IDD_THEMES, ThemesPageProc, NULL, L"Themes" }, */ /* TODO: */
//     { IDD_BACKGROUND, BackgroundPageProc, NULL, L"Desktop" },
//     { IDD_SCREENSAVER, ScreenSaverPageProc, NULL, L"Screen Saver" },
//     { IDD_APPEARANCE, AppearancePageProc, NULL, L"Appearance" },
//     { IDD_SETTINGS, SettingsPageProc, SettingsPageCallbackProc, L"Settings" },
// };
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
//             hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW(IDC_DESK_ICON));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// /* Display Applet */
// static LONG __stdcall
// DisplayApplet(HWND hwnd, UINT uMsg, LPARAM wParam, LPARAM lParam)
// {
//     HPROPSHEETPAGE hpsp[MAX_DESK_PAGES];
//     PROPSHEETHEADER psh;
//     HPSXA hpsxa = NULL;
//     UINT i;
//     LPWSTR *argv = NULL;
//     LPCWSTR pwszSelectedTab = NULL;
//     LPCWSTR pwszFile = NULL;
//     LPCWSTR pwszAction = NULL;
//     INT nPage = 0;
//     BITMAP bitmap;
//
//     UNREFERENCED_PARAMETER(wParam);
//
//     hCPLWindow = hwnd;
//
//     if (uMsg == CPL_STARTWPARMSW && lParam)
//     {
//         int argc;
//         int i;
//
//         nPage = _wtoi((PWSTR)lParam);
//
//         argv = CommandLineToArgvW((LPCWSTR)lParam, &argc);
//
//         if (argv && argc)
//         {
//             for (i = 0; i<argc; i++)
//             {
//                 if (argv[i][0] == L'@')
//                     pwszSelectedTab = &argv[i][1];
//                 else if (wcsncmp(argv[i], L"/Action:", 8) == 0)
//                     pwszAction = &argv[i][8];
//                 else if (wcsncmp(argv[i], L"/file:", 6) == 0)
//                     pwszFile = &argv[i][6];
//             }
//         }
//     }
//
//     if(pwszAction && wcsncmp(pwszAction, L"ActivateMSTheme", 15) == 0)
//     {
//         ActivateThemeFile(pwszFile);
//         goto cleanup;
//     }
//
//     g_GlobalData.pwszFile = pwszFile;
//     g_GlobalData.pwszAction = pwszAction;
//     g_GlobalData.desktop_color = GetSysColor(COLOR_DESKTOP);
//
//     /* Initialize the monitor preview bitmap, used on multiple pages */
//     g_GlobalData.hMonitorBitmap = LoadBitmapW(hApplet, MAKEINTRESOURCEW(IDC_MONITOR));
//     if (g_GlobalData.hMonitorBitmap != NULL)
//     {
//         GetObjectW(g_GlobalData.hMonitorBitmap, sizeof(bitmap), &bitmap);
//
//         g_GlobalData.bmMonWidth = bitmap.bmWidth;
//         g_GlobalData.bmMonHeight = bitmap.bmHeight;
//     }
//
//     ZeroMemory(&psh, sizeof(psh));
//     psh.dwSize = sizeof(psh);
//     psh.dwFlags = PSH_USECALLBACK | PSH_PROPTITLE | PSH_USEICONID;
//     psh.hwndParent = hCPLWindow;
//     psh.hInstance = hApplet;
//     psh.pszIcon = MAKEINTRESOURCEW(IDC_DESK_ICON);
//     psh.pszCaption = MAKEINTRESOURCEW(IDS_CPLNAME);
//     psh.nPages = 0;
//     psh.nStartPage = 0;
//     psh.phpage = hpsp;
//     psh.pfnCallback = PropSheetProc;
//
//     /* Allow shell extensions to replace the background page */
//     hpsxa = SHCreatePropSheetExtArray(HKEY_LOCAL_MACHINE, REGSTR_PATH_CONTROLSFOLDER TEXT("\\Desk"), MAX_DESK_PAGES - psh.nPages);
//
//     for (i = 0; i < _countof(PropPages); i++)
//     {
//         if (pwszSelectedTab && _wcsicmp(pwszSelectedTab, PropPages[i].Name) == 0)
//             psh.nStartPage = i;
//
//         /* Override the background page if requested by a shell extension */
//         if (PropPages[i].idDlg == IDD_BACKGROUND && hpsxa != NULL &&
//             SHReplaceFromPropSheetExtArray(hpsxa, CPLPAGE_DISPLAY_BACKGROUND, DisplayAppletPropSheetAddPage, (LPARAM)&psh) != 0)
//         {
//             /* The shell extension added one or more pages to replace the background page.
//                Don't create the built-in page anymore! */
//             continue;
//         }
//
//         InitPropSheetPage(&psh, PropPages[i].idDlg, PropPages[i].DlgProc, PropPages[i].Callback);
//     }
//
//     /* NOTE: Don't call SHAddFromPropSheetExtArray here because this applet only allows
//              replacing the background page but not extending the applet by more pages */
//
//     if (nPage != 0 && psh.nStartPage == 0)
//         psh.nStartPage = nPage;
//
//     PropertySheet(&psh);
//
// cleanup:
//     DeleteObject(g_GlobalData.hMonitorBitmap);
//
//     if (hpsxa != NULL)
//         SHDestroyPropSheetExtArray(hpsxa);
//
//     if (argv)
//         LocalFree(argv);
//
//     return TRUE;
// }
//
//
// /* Control Panel Callback */
// LONG __stdcall
// CPlApplet(HWND hwndCPl, UINT uMsg, LPARAM lParam1, LPARAM lParam2)
// {
//     UINT i = (UINT)lParam1;
//
//     switch (uMsg)
//     {
//         case CPL_INIT:
//             return TRUE;
//
//         case CPL_GETCOUNT:
//             return _countof(Applets);
//
//         case CPL_INQUIRE:
//             if (i < _countof(Applets))
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
//             if (i < _countof(Applets))
//                 Applets[i].AppletProc(hwndCPl, uMsg, lParam1, lParam2);
//             else
//                 return TRUE;
//             break;
//
//         case CPL_STARTWPARMSW:
//             if (i < _countof(Applets))
//                 return Applets[i].AppletProc(hwndCPl, uMsg, lParam1, lParam2);
//             break;
//     }
//
//     return FALSE;
// }
//
// void
// WINAPI
// InstallScreenSaverW(
//      HWND hWindow,
//      HANDLE hInstance,
//      LPCWSTR pszFile,
//      UINT nCmdShow)
// {
//     LRESULT rc;
//     HKEY regKey;
//     INT Timeout;
// #ifdef CHECK_SCR_FULL_PATH
//     HANDLE hFile;
//     WIN32_FIND_DATAW fdFile;
// #endif
//     DWORD dwLen;
//     WCHAR szFullPath[MAX_PATH];
//
//     if (!pszFile)
//     {
//         DPRINT1("InstallScreenSaver() null file\n");
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return;
//     }
//     DPRINT("InstallScreenSaver() Installing screensaver %ls\n", pszFile);
//
// #ifdef CHECK_SCR_FULL_PATH
//     /* Retrieve the actual path to the file and verify whether it exists */
//     dwLen = GetFullPathNameW(pszFile, _countof(szFullPath), szFullPath, NULL);
//     if (dwLen == 0 || dwLen > _countof(szFullPath))
//     {
//         DPRINT1("InstallScreenSaver() File %ls not accessible\n", pszFile);
//         return;
//     }
//     hFile = FindFirstFile(szFullPath, &fdFile);
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("InstallScreenSaver() File %ls not found\n", pszFile);
//         return;
//     }
//     FindClose(hFile);
//     /* Use the full file path from now on */
//     pszFile = szFullPath;
// #endif
//
//     rc = RegOpenKeyExW(HKEY_CURRENT_USER,
//                        L"Control Panel\\Desktop",
//                        0,
//                        KEY_SET_VALUE,
//                        &regKey);
//     if (rc == ERROR_SUCCESS)
//     {
//         /* Set the screensaver */
//         SIZE_T Length = (wcslen(pszFile) + 1) * sizeof(WCHAR);
//         rc = RegSetValueExW(regKey,
//                             L"SCRNSAVE.EXE",
//                             0,
//                             REG_SZ,
//                             (PBYTE)pszFile,
//                             (DWORD)Length);
//         RegCloseKey(regKey);
//     }
//     if (rc != ERROR_SUCCESS)
//     {
//         DPRINT1("InstallScreenSaver() Could not change the current screensaver\n");
//         return;
//     }
//
//     SystemParametersInfoW(SPI_SETSCREENSAVEACTIVE, TRUE, 0, SPIF_UPDATEINIFILE);
//
//     /* If no screensaver timeout is present, default to 10 minutes (600 seconds) */
//     Timeout = 0;
//     if (!SystemParametersInfoW(SPI_GETSCREENSAVETIMEOUT, 0, &Timeout, 0) || (Timeout <= 0))
//         SystemParametersInfoW(SPI_SETSCREENSAVETIMEOUT, 600, 0, SPIF_UPDATEINIFILE);
//
//     /* Retrieve the name of this current instance of desk.cpl */
//     dwLen = GetModuleFileNameW(hApplet, szFullPath, _countof(szFullPath));
//     if ((dwLen == 0) || (GetLastError() == ERROR_INSUFFICIENT_BUFFER))
//     {
//         /* We failed, copy the default value */
//         StringCchCopyW(szFullPath, _countof(szFullPath), L"desk.cpl");
//     }
//
//     /* Build the desk.cpl command-line to start the ScreenSaver page.
//      * Equivalent to: "desk.cpl,ScreenSaver,@ScreenSaver" */
//     rc = StringCchCatW(szFullPath, _countof(szFullPath), L",,1");
//     if (FAILED(rc))
//         return;
//
//     /* Open the ScreenSaver page in this desk.cpl instance */
//     DPRINT("InstallScreenSaver() Starting '%ls'\n", szFullPath);
//     Control_RunDLLW(hWindow, hInstance, szFullPath, nCmdShow);
// }
//
// void
// WINAPI
// InstallScreenSaverA(
//      HWND hWindow,
//      HANDLE hInstance,
//      LPCSTR pszFile,
//      UINT nCmdShow)
// {
//     LPWSTR lpwString;
//     int nLength;
//
//     if (!pszFile)
//     {
//         DPRINT1("InstallScreenSaver() null file\n");
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return;
//     }
//
//     /* Convert the string to unicode */
//     lpwString = NULL;
//     nLength = MultiByteToWideChar(CP_ACP, 0, pszFile, -1, NULL, 0);
//     if (nLength != 0)
//     {
//         lpwString = LocalAlloc(LMEM_FIXED, nLength * sizeof(WCHAR));
//         if (lpwString)
//         {
//             if (!MultiByteToWideChar(CP_ACP, 0, pszFile, -1, lpwString, nLength))
//             {
//                 LocalFree(lpwString);
//                 lpwString = NULL;
//             }
//         }
//     }
//     if (!lpwString)
//     {
//         DPRINT1("InstallScreenSaver() not enough memory to convert string to unicode\n");
//         return;
//     }
//
//     /* Call the unicode function */
//     InstallScreenSaverW(hWindow, hInstance, lpwString, nCmdShow);
//
//     LocalFree(lpwString);
// }
//
// BOOL WINAPI
// DllMain(HINSTANCE hInstDLL, DWORD dwReason, LPVOID lpvReserved)
// {
//     UNREFERENCED_PARAMETER(lpvReserved);
//
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);
//             RegisterPreviewControl(hInstDLL);
// //        case DLL_THREAD_ATTACH:
//             hApplet = hInstDLL;
//             break;
//
//         case DLL_PROCESS_DETACH:
//             UnregisterPreviewControl(hInstDLL);
//             CoUninitialize();
//             break;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

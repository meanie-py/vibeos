use crate::msconfig::freeldrpage::FreeLdrPageWndProc;
use crate::msconfig::generalpage::GeneralPageWndProc;
use crate::msconfig::srvpage::ServicesPageWndProc;
use crate::msconfig::startuppage::StartupPageWndProc;
use crate::msconfig::systempage::SystemPageWndProc;
use crate::msconfig::toolspage::ToolsPageWndProc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hMainWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hTabWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut uXIcon: UINT = 0 as UINT;

#[no_mangle]
pub static mut uYIcon: UINT = 0 as UINT;

#[no_mangle]
pub static mut hDialogIconBig: HICON = ::core::ptr::null::<HICON__>() as *mut HICON__;

#[no_mangle]
pub static mut hDialogIconSmall: HICON = ::core::ptr::null::<HICON__>() as *mut HICON__;

pub const ETDT_DISABLE: u32 = 0x00000001;
pub const ETDT_ENABLE: u32 = 0x00000002;
pub const ETDT_USETABTEXTURE: u32 = 0x00000004;
pub const ETDT_ENABLETAB: u32 = ETDT_ENABLE | ETDT_USETABTEXTURE;

pub type ETDTProc = unsafe extern "system" fn(HWND, DWORD) -> HRESULT;
#[no_mangle]
pub unsafe extern "C" fn EnableDialogTheme(hwnd: HWND) -> BOOL {
    let mut hUXTheme: HMODULE = core::ptr::null_mut();
    let mut fnEnableThemeDialogTexture: Option<ETDTProc>;
    let uxtheme_dll: [u16; 12] = ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];

    hUXTheme = LoadLibrary(uxtheme_dll.as_ptr() as LPCTSTR) as HMODULE;

    if !hUXTheme.is_null() {
        fnEnableThemeDialogTexture =
            core::mem::transmute::<FARPROC, Option<ETDTProc>>(GetProcAddress(
                hUXTheme as HINSTANCE,
                b"EnableThemeDialogTexture\0".as_ptr() as LPCSTR,
            ));

        if fnEnableThemeDialogTexture.is_some() {
            (fnEnableThemeDialogTexture.unwrap_unchecked())(hwnd, ETDT_ENABLETAB);

            FreeLibrary(hUXTheme);
            return (TRUE) as BOOL;
        } else {
            // Failed to locate API!
            FreeLibrary(hUXTheme);
            return (FALSE) as BOOL;
        }
    } else {
        // Not running under XP? Just fail gracefully
        return (FALSE) as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DisableAllExcept(hTabDlg: HWND, idExcept: UINT) {
    let hSkip: HWND = GetDlgItem(hTabDlg, idExcept as core::ffi::c_int);
    let mut hWnd: HWND = core::ptr::null_mut();
    loop {
        hWnd = FindWindowExW(
            hTabDlg,
            hWnd,
            core::ptr::null(),
            core::ptr::null(),
        );
        if hWnd.is_null() {
            break;
        }
        EnableWindow(hWnd, (hWnd == hSkip) as i32);
    }
    if idExcept == IDC_SYSTEM_TREE as UINT && !hSkip.is_null() {
        let mut style: LONG_PTR = GetWindowLongPtrW(hSkip, GWL_STYLE);
        style &= !TVS_EDITLABELS as LONG_PTR;
        SetWindowLongPtrW(hSkip, GWL_STYLE, style);
    }
}
#[no_mangle]
pub unsafe extern "C" fn OnCreate(hWnd: HWND) -> BOOL {
    let mut szTemp: [TCHAR; 256] = [0 as TCHAR; 256];
    let mut item: TCITEM = core::mem::zeroed();

    hTabWnd = GetDlgItem(hWnd, (IDC_TAB as i32));
    hGeneralPage = CreateDialog!(hInst, MAKEINTRESOURCE!(IDD_GENERAL_PAGE), hWnd, Some(GeneralPageWndProc));
    EnableDialogTheme(hGeneralPage);
    hSystemPage = CreateDialog!(hInst, MAKEINTRESOURCE!(IDD_SYSTEM_PAGE), hWnd, Some(SystemPageWndProc));
    EnableDialogTheme(hSystemPage);
    hFreeLdrPage = CreateDialog!(hInst, MAKEINTRESOURCE!(IDD_FREELDR_PAGE), hWnd, Some(FreeLdrPageWndProc));
    EnableDialogTheme(hFreeLdrPage);
    hServicesPage = CreateDialog!(hInst, MAKEINTRESOURCE!(IDD_SERVICES_PAGE), hWnd, Some(ServicesPageWndProc));
    EnableDialogTheme(hServicesPage);
    hStartupPage = CreateDialog!(hInst, MAKEINTRESOURCE!(IDD_STARTUP_PAGE), hWnd, Some(StartupPageWndProc));
    EnableDialogTheme(hStartupPage);
    hToolsPage = CreateDialog!(hInst, MAKEINTRESOURCE!(IDD_TOOLS_PAGE), hWnd, Some(ToolsPageWndProc));
    EnableDialogTheme(hToolsPage);

    LoadString(hInst, IDS_MSCONFIG, &raw mut szTemp as LPTSTR, 256);
    SetWindowText(hWnd, &raw mut szTemp as *mut TCHAR as LPCTSTR);

    // Insert Tab Pages
    LoadString(hInst, IDS_TAB_GENERAL, &raw mut szTemp as LPTSTR, 256);
    memset(&raw mut item as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<TCITEM>() as usize);
    item.mask = TCIF_TEXT;
    item.pszText = &raw mut szTemp as *mut TCHAR;
    TabCtrl_InsertItem!(hTabWnd, 0, &raw mut item);

    LoadString(hInst, IDS_TAB_SYSTEM, &raw mut szTemp as LPTSTR, 256);
    memset(&raw mut item as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<TCITEM>() as usize);
    item.mask = TCIF_TEXT;
    item.pszText = &raw mut szTemp as *mut TCHAR;
    TabCtrl_InsertItem!(hTabWnd, 1, &raw mut item);

    LoadString(hInst, IDS_TAB_FREELDR, &raw mut szTemp as LPTSTR, 256);
    memset(&raw mut item as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<TCITEM>() as usize);
    item.mask = TCIF_TEXT;
    item.pszText = &raw mut szTemp as *mut TCHAR;
    TabCtrl_InsertItem!(hTabWnd, 2, &raw mut item);

    LoadString(hInst, IDS_TAB_SERVICES, &raw mut szTemp as LPTSTR, 256);
    memset(&raw mut item as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<TCITEM>() as usize);
    item.mask = TCIF_TEXT;
    item.pszText = &raw mut szTemp as *mut TCHAR;
    TabCtrl_InsertItem!(hTabWnd, 3, &raw mut item);

    LoadString(hInst, IDS_TAB_STARTUP, &raw mut szTemp as LPTSTR, 256);
    memset(&raw mut item as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<TCITEM>() as usize);
    item.mask = TCIF_TEXT;
    item.pszText = &raw mut szTemp as *mut TCHAR;
    TabCtrl_InsertItem!(hTabWnd, 4, &raw mut item);

    LoadString(hInst, IDS_TAB_TOOLS, &raw mut szTemp as LPTSTR, 256);
    memset(&raw mut item as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<TCITEM>() as usize);
    item.mask = TCIF_TEXT;
    item.pszText = &raw mut szTemp as *mut TCHAR;
    TabCtrl_InsertItem!(hTabWnd, 5, &raw mut item);

    MsConfig_OnTabWndSelChange();

    // FIXME: We don't support applying anything
    EnableWindow(GetDlgItem(hWnd, (IDOK) as i32), (FALSE) as BOOL);
    EnableWindow(GetDlgItem(hWnd, (IDC_BTN_APPLY as i32)), (FALSE) as BOOL);
    EnableWindow(GetDlgItem(hWnd, (IDC_BTN_HELP as i32)), (FALSE) as BOOL);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn MsConfig_OnTabWndSelChange() {
    match TabCtrl_GetCurSel!(hTabWnd) {
        0 => { //General
            ShowWindow(hGeneralPage, (SW_SHOW) as i32);
            ShowWindow(hSystemPage, (SW_HIDE) as i32);
            ShowWindow(hFreeLdrPage, (SW_HIDE) as i32);
            ShowWindow(hServicesPage, (SW_HIDE) as i32);
            ShowWindow(hStartupPage, (SW_HIDE) as i32);
            ShowWindow(hToolsPage, (SW_HIDE) as i32);
            BringWindowToTop(hGeneralPage);
        }
        1 => { //SYSTEM.INI
            ShowWindow(hGeneralPage, (SW_HIDE) as i32);
            ShowWindow(hSystemPage, (SW_SHOW) as i32);
            ShowWindow(hToolsPage, (SW_HIDE) as i32);
            ShowWindow(hStartupPage, (SW_HIDE) as i32);
            ShowWindow(hFreeLdrPage, (SW_HIDE) as i32);
            ShowWindow(hServicesPage, (SW_HIDE) as i32);
            BringWindowToTop(hSystemPage);
        }
        2 => { //Freeldr
            ShowWindow(hGeneralPage, (SW_HIDE) as i32);
            ShowWindow(hSystemPage, (SW_HIDE) as i32);
            ShowWindow(hFreeLdrPage, (SW_SHOW) as i32);
            ShowWindow(hServicesPage, (SW_HIDE) as i32);
            ShowWindow(hStartupPage, (SW_HIDE) as i32);
            ShowWindow(hToolsPage, (SW_HIDE) as i32);
            BringWindowToTop(hFreeLdrPage);
        }
        3 => { //Services
            ShowWindow(hGeneralPage, (SW_HIDE) as i32);
            ShowWindow(hSystemPage, (SW_HIDE) as i32);
            ShowWindow(hFreeLdrPage, (SW_HIDE) as i32);
            ShowWindow(hServicesPage, (SW_SHOW) as i32);
            ShowWindow(hStartupPage, (SW_HIDE) as i32);
            ShowWindow(hToolsPage, (SW_HIDE) as i32);
            BringWindowToTop(hServicesPage);
        }
        4 => { //startup
            ShowWindow(hGeneralPage, (SW_HIDE) as i32);
            ShowWindow(hSystemPage, (SW_HIDE) as i32);
            ShowWindow(hFreeLdrPage, (SW_HIDE) as i32);
            ShowWindow(hServicesPage, (SW_HIDE) as i32);
            ShowWindow(hStartupPage, (SW_SHOW) as i32);
            ShowWindow(hToolsPage, (SW_HIDE) as i32);
            BringWindowToTop(hStartupPage);
        }
        5 => { //Tools
            ShowWindow(hGeneralPage, (SW_HIDE) as i32);
            ShowWindow(hSystemPage, (SW_HIDE) as i32);
            ShowWindow(hFreeLdrPage, (SW_HIDE) as i32);
            ShowWindow(hServicesPage, (SW_HIDE) as i32);
            ShowWindow(hStartupPage, (SW_HIDE) as i32);
            ShowWindow(hToolsPage, (SW_SHOW) as i32);
            BringWindowToTop(hToolsPage);
        }
        _ => {}
    }
}

unsafe extern "C" fn SetDialogIcon(hDlg: HWND) {
    if !hDialogIconBig.is_null() {
        DestroyIcon(hDialogIconBig);
    }
    if !hDialogIconSmall.is_null() {
        DestroyIcon(hDialogIconSmall);
    }
    hDialogIconBig = LoadIconW(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        MAKEINTRESOURCE!(IDI_APPICON) as LPCTSTR,
    );
    hDialogIconSmall = LoadImage(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        MAKEINTRESOURCE!(IDI_APPICON) as LPCTSTR,
        IMAGE_ICON,
        uXIcon as core::ffi::c_int,
        uYIcon as core::ffi::c_int,
        0,
    ) as HICON;
    SendMessage(
        hDlg,
        WM_SETICON,
        (ICON_BIG) as _,
        hDialogIconBig as LPARAM,
    );
    SendMessage(
        hDlg,
        WM_SETICON,
        (ICON_SMALL) as _,
        hDialogIconSmall as LPARAM,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MsConfigWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pnmh: LPNMHDR;
    let mut uXIconNew: UINT;
    let mut uYIconNew: UINT;

    match message {
        WM_INITDIALOG => {
            hMainWnd = hDlg;

            uXIcon = GetSystemMetrics((SM_CXSMICON) as i32) as UINT;
            uYIcon = GetSystemMetrics((SM_CYSMICON) as i32) as UINT;

            SetDialogIcon(hDlg);

            return OnCreate(hDlg) as INT_PTR;
        }
        WM_SETTINGCHANGE => {
            uXIconNew = GetSystemMetrics((SM_CXSMICON) as i32) as UINT;
            uYIconNew = GetSystemMetrics((SM_CYSMICON) as i32) as UINT;

            if (uXIcon != uXIconNew) || (uYIcon != uYIconNew) {
                uXIcon = uXIconNew;
                uYIcon = uYIconNew;
                SetDialogIcon(hDlg);
            }
        }
        WM_COMMAND => {
            if (LOWORD!(wParam)) as u32 == IDOK {
                //MsConfig_OnSaveChanges();
            }

            if (LOWORD!(wParam)) as u32 == IDOK || (LOWORD!(wParam)) as u32 == IDCANCEL {
                EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                return TRUE as INT_PTR;
            }
        }
        WM_NOTIFY => {
            pnmh = lParam as LPNMHDR;
            if (*pnmh).hwndFrom == hTabWnd
                && (*pnmh).idFrom == IDC_TAB as UINT_PTR
                && (*pnmh).code == TCN_SELCHANGE as UINT
            {
                MsConfig_OnTabWndSelChange();
            }
        }
        WM_SYSCOLORCHANGE => {
            /* Forward WM_SYSCOLORCHANGE to common controls */
            SendMessage(hServicesListCtrl, WM_SYSCOLORCHANGE as UINT, 0 as WPARAM, 0 as LPARAM);
            SendMessage(hStartupListCtrl, WM_SYSCOLORCHANGE as UINT, 0 as WPARAM, 0 as LPARAM);
            SendMessage(hToolsListCtrl, WM_SYSCOLORCHANGE as UINT, 0 as WPARAM, 0 as LPARAM);
        }
        WM_DESTROY => {
            if !hToolsPage.is_null() {
                DestroyWindow(hToolsPage);
            }
            if !hGeneralPage.is_null() {
                DestroyWindow(hGeneralPage);
            }
            if !hServicesPage.is_null() {
                DestroyWindow(hServicesPage);
            }
            if !hStartupPage.is_null() {
                DestroyWindow(hStartupPage);
            }
            if !hFreeLdrPage.is_null() {
                DestroyWindow(hFreeLdrPage);
            }
            if !hSystemPage.is_null() {
                DestroyWindow(hSystemPage);
            }
            if !hDialogIconBig.is_null() {
                DestroyIcon(hDialogIconBig);
            }
            if !hDialogIconSmall.is_null() {
                DestroyIcon(hDialogIconSmall);
            }
            return DefWindowProc(hDlg, message, wParam, lParam) as INT_PTR;
        }
        _ => {}
    }

    return 0 as INT_PTR;
}

#[export_name = "wWinMain"]
pub unsafe extern "system" fn _tWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPTSTR,
    nCmdShow: i32,
) -> i32 {
    let mut InitControls: INITCOMMONCONTROLSEX = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(lpCmdLine);
    UNREFERENCED_PARAMETER!(nCmdShow);

    InitControls.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    InitControls.dwICC = (ICC_TAB_CLASSES | ICC_LISTVIEW_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(InitControls));

    hInst = hInstance;

    DialogBox!(
        hInst,
        IDD_MSCONFIG_DIALOG as LPCTSTR,
        core::ptr::null_mut(),
        Some(MsConfigWndProc),
    );

    return 0;
}

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_bRebootNeeded: BOOL = (FALSE) as i32;

static mut Applets: [APPLET; 1] = unsafe {
    [APPLET {
        idIcon: (IDI_CPLSYSTEM as i32),
        idName: (IDS_CPLSYSTEMNAME as i32),
        idDescription: (IDS_CPLSYSTEMDESCRIPTION as i32),
        AppletProc: Some(SystemApplet),
    }]
};

pub const NUM_APPLETS: usize = 1;

unsafe extern "C" fn InitPropSheetPage(page: *mut PROPSHEETPAGEW, idDlg: WORD, DlgProc: DLGPROC) {
    ZeroMemory!(
        page as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETPAGEW>(),
    );

    (*page).dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    (*page).dwFlags = PSP_DEFAULT as DWORD;
    (*page).hInstance = hApplet;
    (*page).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(idDlg);
    (*page).pfnDlgProc = DlgProc;
}

unsafe extern "C" fn AskForReboot(hwndDlg: HWND) -> BOOL {
    let mut szText: [WCHAR; 128] = [0; 128];
    let mut szCaption: [WCHAR; 64] = [0; 64];
    LoadStringW(
        hApplet,
        IDS_REBOOT_NOW as UINT,
        szText.as_mut_ptr() as LPWSTR,
        _countof!(szText) as ::core::ffi::c_int,
    );
    LoadStringW(
        hApplet,
        IDS_LANGUAGE as UINT,
        szCaption.as_mut_ptr() as LPWSTR,
        _countof!(szCaption) as ::core::ffi::c_int,
    );
    return ((MessageBoxW(
        hwndDlg,
        szText.as_mut_ptr() as LPCWSTR,
        szCaption.as_mut_ptr() as LPCWSTR,
        (MB_ICONINFORMATION | MB_YESNO) as UINT,
    )) as u32 == IDYES) as BOOL;
}
unsafe extern "system" fn PropSheetProc(mut hwndDlg: HWND, mut uMsg: UINT, mut lParam: LPARAM) -> ::core::ffi::c_int {
    match uMsg {
        PSCB_INITIALIZED => {
            let mut hIcon: HICON = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_CPLSYSTEM) as LPCWSTR);
            SendMessageW(hwndDlg, WM_SETICON as UINT, ICON_BIG as WPARAM, hIcon as LPARAM);
        }
        PSCB_BUTTONPRESSED => {
            match (lParam) as u32 {
                PSBTN_OK | PSBTN_APPLYNOW => {
                    if g_bRebootNeeded != 0 && AskForReboot(hwndDlg) != 0 {
                        EnableProcessPrivileges(SE_SHUTDOWN_NAME as LPCWSTR, (TRUE) as BOOL);
                        ExitWindowsEx((EWX_REBOOT | EWX_FORCE) as UINT, 0 as DWORD);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "system" fn SystemApplet(hwnd: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    let mut page: [PROPSHEETPAGEW; 2] = core::mem::zeroed();
    let mut header: PROPSHEETHEADERW = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(header) as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETHEADERW>());

    header.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    header.dwFlags = (PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    header.hwndParent = hwnd;
    header.hInstance = hApplet;
    header.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_CPLSYSTEM);
    header.pszCaption = MAKEINTRESOURCEW!(IDS_CPLSYSTEMNAME);
    header.nPages = ARRAYSIZE!((*core::ptr::addr_of_mut!(page))) as UINT;
    header.__bindgen_anon_2.nStartPage = 0 as UINT;
    header.__bindgen_anon_3.ppsp = core::ptr::addr_of_mut!(page) as *mut PROPSHEETPAGEW as LPCPROPSHEETPAGEW;
    header.pfnCallback = Some(PropSheetProc);

    /* Settings */
    InitPropSheetPage(
        core::ptr::addr_of_mut!(page[0]),
        IDD_PROPPAGESETTINGS as WORD,
        Some(SettingsPageProc),
    );

    /* Advanced Settings */
    InitPropSheetPage(
        core::ptr::addr_of_mut!(page[1]),
        IDD_PROPPAGEADVANCEDSETTINGS as WORD,
        Some(AdvancedSettingsPageProc),
    );

    return (PropertySheetW(core::ptr::addr_of_mut!(header) as LPCPROPSHEETHEADERW) != -(1 as INT_PTR)) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(hwndCPl: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    let mut CPlInfo: *mut CPLINFO = core::ptr::null_mut();
    let mut i: UINT = lParam1 as UINT;

    match uMsg {
        CPL_INIT => {
            return (TRUE) as LONG;
        }
        CPL_GETCOUNT => {
            return (NUM_APPLETS) as LONG;
        }
        CPL_INQUIRE => {
            if i < NUM_APPLETS as UINT {
                CPlInfo = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon;
                (*CPlInfo).idName = Applets[i as usize].idName;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription;
            } else {
                return (TRUE) as LONG;
            }
        }
        CPL_DBLCLK => {
            if i < NUM_APPLETS as UINT {
                ((Applets[i as usize].AppletProc).unwrap_unchecked())(hwndCPl, uMsg, lParam1, lParam2);
            } else {
                return (TRUE) as LONG;
            }
        }
        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH => {
            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON = core::ptr::null_mut();
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_CPLICON));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn AppletInit(hWnd: HWND) -> LONG {
    let mut psp: PROPSHEETPAGEW = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(psp) as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETPAGE>(),
    );
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = PSP_DEFAULT as DWORD;
    psp.hInstance = hApplet;
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PROPPAGEGENERAL) as LPCWSTR;
    psp.pfnDlgProc = Some(GeneralPageProc);

    ZeroMemory!(
        core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETHEADER>(),
    );
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    psh.hwndParent = hWnd;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_CPLICON) as LPCWSTR;
    psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLNAME) as LPCWSTR;
    psh.nPages = (core::mem::size_of::<PROPSHEETPAGEW>()
        / core::mem::size_of::<PROPSHEETPAGE>()) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = core::ptr::addr_of_mut!(psp) as LPCPROPSHEETPAGEW;
    psh.pfnCallback = Some(PropSheetProc);

    return (PropertySheet(core::ptr::addr_of_mut!(psh))
        != -(1 as INT_PTR)) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    mut hWnd: HWND,
    mut uMsg: UINT,
    mut lParam1: LPARAM,
    mut lParam2: LPARAM,
) -> LONG {
    match uMsg {
        CPL_INIT => {
            return TRUE as LONG;
        }
        CPL_GETCOUNT => {
            return 1 as LONG;
        }
        CPL_INQUIRE => {
            let mut CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
            (*CPlInfo).lData = 0 as LONG_PTR;
            (*CPlInfo).idIcon = IDI_CPLICON as LONG;
            (*CPlInfo).idInfo = IDS_CPLDESCRIPTION as LONG;
            (*CPlInfo).idName = IDS_CPLNAME as LONG;
        }
        CPL_DBLCLK => {
            AppletInit(hWnd);
        }
        _ => {}
    }
    return FALSE as LONG;
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

    return TRUE as BOOL;
}

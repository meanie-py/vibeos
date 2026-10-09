use crate::inetcpl::connections::connections_dlgproc;
use crate::inetcpl::content::content_dlgproc;
use crate::inetcpl::general::general_dlgproc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hcpl: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hdll: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hdll as HMODULE);
            hcpl = hdll as HMODULE;
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllInstall(bInstall: BOOL, cmdline: LPCWSTR) -> HRESULT {
    // FIXME("(%s, %s): stub\n", bInstall ? "TRUE" : "FALSE", debugstr_w(cmdline));
    S_OK
}

unsafe extern "system" fn propsheet_callback(
    mut hwnd: HWND,
    mut msg: UINT,
    mut lparam: LPARAM,
) -> ::core::ffi::c_int {
    let mut hIcon: HICON = ::core::ptr::null_mut::<HICON__>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/cpl/inetcpl/inetcpl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"propsheet_callback\0" as *const u8 as *const ::core::ffi::c_char,
            84 as ::core::ffi::c_int,
            b"(%p, 0x%08x/%d, 0x%lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwnd,
            msg,
            msg,
            lparam,
        );
    }
    match msg {
        PSCB_INITIALIZED => {
            if true {
                hIcon = LoadIconW(
                    hcpl as HINSTANCE,
                    (ICO_MAIN as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR
                        as LPWSTR as LPCWSTR,
                );
                SendMessageW(
                    hwnd,
                    WM_SETICON as UINT,
                    ICON_BIG as WPARAM,
                    hIcon as LPARAM,
                );
            } else {
                SendMessageW(
                    hwnd,
                    WM_SETICON as UINT,
                    ICON_BIG as WPARAM,
                    LoadIconW(
                        hcpl as HINSTANCE,
                        (ICO_MAIN as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                            as ULONG_PTR as LPWSTR as LPCWSTR,
                    ) as LPARAM,
                );
            }
        }
        _ => {}
    }
    return 0;
}

pub unsafe fn MAKEINTRESOURCEW(i: usize) -> LPCWSTR {
    (i & 0xffff) as LPCWSTR
}

unsafe extern "C" fn display_cpl_sheets(parent: HWND) {
    let mut icex: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut psp: [PROPSHEETPAGEW; (NUM_PROPERTY_PAGES as usize)] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut id: DWORD = 0;

    OleInitialize(core::ptr::null_mut());
    /* Initialize common controls */
    icex.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    icex.dwICC = (ICC_LISTVIEW_CLASSES | ICC_BAR_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(icex));

    ZeroMemory!(
        core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETHEADERW>(),
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW as *mut core::ffi::c_void,
        core::mem::size_of::<[PROPSHEETPAGEW; (NUM_PROPERTY_PAGES as usize)]>(),
    );

    /* Fill out all PROPSHEETPAGE */
    psp[id as usize].dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    psp[id as usize].hInstance = hcpl as HINSTANCE;
    psp[id as usize].u.pszTemplate = MAKEINTRESOURCEW((IDD_GENERAL) as _);
    psp[id as usize].pfnDlgProc = Some(general_dlgproc);
    id = id.wrapping_add(1);

    psp[id as usize].dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    psp[id as usize].hInstance = hcpl as HINSTANCE;
    psp[id as usize].u.pszTemplate = MAKEINTRESOURCEW((IDD_SECURITY) as _);
    psp[id as usize].pfnDlgProc = Some(security_dlgproc);
    id = id.wrapping_add(1);

    psp[id as usize].dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    psp[id as usize].hInstance = hcpl as HINSTANCE;
    psp[id as usize].u.pszTemplate = MAKEINTRESOURCEW((IDD_CONTENT) as _);
    psp[id as usize].pfnDlgProc = Some(content_dlgproc);
    id = id.wrapping_add(1);

    psp[id as usize].dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    psp[id as usize].hInstance = hcpl as HINSTANCE;
    psp[id as usize].u.pszTemplate = MAKEINTRESOURCEW((IDD_CONNECTIONS) as _);
    psp[id as usize].pfnDlgProc = Some(connections_dlgproc);
    id = id.wrapping_add(1);

    /* Fill out the PROPSHEETHEADER */
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    psh.dwFlags = (PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    psh.hwndParent = parent;
    psh.hInstance = hcpl as HINSTANCE;
    psh.u.pszIcon = MAKEINTRESOURCEW((ICO_MAIN as usize));
    psh.pszCaption = MAKEINTRESOURCEW((IDS_CPL_NAME as usize));
    psh.nPages = id as UINT;
    psh.u3.ppsp = core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW as LPCPROPSHEETPAGEW;
    psh.pfnCallback = Some(propsheet_callback);

    /* display the dialog */
    PropertySheetW(core::ptr::addr_of_mut!(psh));

    OleUninitialize();
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(hWnd: HWND, command: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    // WINE_TRACE debug-channel expansion elided (no behavioral effect)
    match command {
        CPL_INIT => {
            return (TRUE) as LONG;
        }
        CPL_GETCOUNT => {
            return 1;
        }
        CPL_INQUIRE => {
            let mut appletInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;

            (*appletInfo).idIcon = (ICO_MAIN as i32);
            (*appletInfo).idName = (IDS_CPL_NAME as i32);
            (*appletInfo).idInfo = (IDS_CPL_INFO as i32);
            (*appletInfo).lData = 0;
            return (TRUE) as LONG;
        }
        CPL_DBLCLK => {
            display_cpl_sheets(hWnd);
        }
        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn LaunchInternetControlPanel(parent: HWND) -> BOOL {
    display_cpl_sheets(parent);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn LaunchConnectionDialog(hParent: HWND) -> BOOL {
    // FIXME("(%p): stub\n", hParent);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn LaunchPrivacyDialog(hParent: HWND) -> BOOL {
    // FIXME("(%p): stub\n", hParent);
    (FALSE) as i32
}

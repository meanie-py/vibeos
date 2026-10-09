use crate::prelude::*;
use core::ffi::c_void;

static mut DISPLAY_CERT_MANAGER_DUMMY: u8 = 0;

unsafe extern "C" fn display_cert_manager(parent: HWND, flags: DWORD) -> BOOL {
    let mut dlg: CRYPTUI_CERT_MGR_STRUCT = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(dlg) as *mut core::ffi::c_void,
        core::mem::size_of::<CRYPTUI_CERT_MGR_STRUCT>() as usize,
    );
    dlg.dwSize = core::mem::size_of::<CRYPTUI_CERT_MGR_STRUCT>() as DWORD;
    dlg.hwndParent = parent;
    dlg.dwFlags = flags;

    return CryptUIDlgCertMgr(core::ptr::addr_of_mut!(dlg) as PCCRYPTUI_CERT_MGR_STRUCT);
}
#[no_mangle] #[no_mangle]
#[no_mangle] pub unsafe extern "system" fn LaunchSiteCertDialog(parent: HWND) -> BOOL {
    return display_cert_manager(parent, 0);
}
#[no_mangle]
pub unsafe extern "system" fn content_dlgproc(mut hwnd: HWND, mut msg: UINT, mut wparam: WPARAM, mut lparam: LPARAM) -> INT_PTR {
    if (msg != WM_SETCURSOR as UINT) && (msg != WM_NCHITTEST as UINT) && (msg != WM_MOUSEMOVE as UINT) {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/inetcpl/content.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"content_dlgproc\0" as *const u8 as *const ::core::ffi::c_char,
                81 as ::core::ffi::c_int,
                b"(%p, 0x%08x/%d, 0x%lx, 0x%lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
                hwnd,
                msg,
                msg,
                wparam,
                lparam,
            );
        }
    }
    if msg == WM_COMMAND as UINT {
        let sel: u32 = LOWORD!(wparam) as u32;
        if sel == IDC_CERT as u32 {
            display_cert_manager(hwnd, 0 as DWORD);
        } else if sel == IDC_CERT_PUBLISHER as u32 {
            display_cert_manager(hwnd, CRYPTUI_CERT_MGR_PUBLISHER_TAB as DWORD);
        }
    }
    return FALSE as INT_PTR;
}

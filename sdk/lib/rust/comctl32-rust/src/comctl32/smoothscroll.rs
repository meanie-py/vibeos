use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_commctrl);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut smoothscroll: DWORD = 2 as DWORD;

pub type SCROLLWINDOWEXPROC = Option<unsafe extern "system" fn(HWND, INT, INT, LPRECT, LPRECT, HRGN, LPRECT, DWORD) -> BOOL>;

#[repr(C)]
pub struct tagSMOOTHSCROLLSTRUCT {
    pub dwSize: DWORD,
    pub x2: DWORD,
    pub hwnd: HWND,
    pub dx: DWORD,
    pub dy: DWORD,
    pub lpscrollrect: LPRECT,
    pub lpcliprect: LPRECT,
    pub hrgnupdate: HRGN,
    pub lpupdaterect: LPRECT,
    pub flags: DWORD,
    pub stepinterval: DWORD,
    pub dx_step: DWORD,
    pub dy_step: DWORD,
    pub scrollfun: SCROLLWINDOWEXPROC,
}

pub type SMOOTHSCROLLSTRUCT = tagSMOOTHSCROLLSTRUCT;
pub type PSMOOTHSCROLLSTRUCT = *mut SMOOTHSCROLLSTRUCT;

#[no_mangle]
pub unsafe extern "system" fn SmoothScrollWindow(smooth: *const SMOOTHSCROLLSTRUCT) -> BOOL {
    let mut lpupdaterect: LPRECT = (*smooth).lpupdaterect;
    let hrgnupdate: HRGN = (*smooth).hrgnupdate;
    let mut tmprect: RECT = core::mem::zeroed();
    let mut flags: DWORD = (*smooth).flags;

    if (*smooth).dwSize as usize != (core::mem::size_of::<SMOOTHSCROLLSTRUCT>()) as usize {
        return FALSE as BOOL;
    }

    if lpupdaterect.is_null() {
        lpupdaterect = core::ptr::addr_of_mut!(tmprect);
    }
    SetRectEmpty(lpupdaterect);

    if !(flags & (0x40000 as DWORD) != 0) {
        if smoothscroll == (2 as DWORD) {
            let mut hkey: HKEY = core::ptr::null_mut();

            smoothscroll = 0 as DWORD;
            if RegOpenKeyA(
                HKEY_CURRENT_USER as HKEY,
                b"Control Panel\\Desktop\0".as_ptr() as LPCSTR,
                core::ptr::addr_of_mut!(hkey),
            ) == 0
            {
                let mut len: DWORD = 4 as DWORD;

                RegQueryValueExA(
                    hkey,
                    b"SmoothScroll\0".as_ptr() as LPCSTR,
                    0 as *mut DWORD,
                    0 as *mut DWORD,
                    core::ptr::addr_of_mut!(smoothscroll) as LPBYTE,
                    core::ptr::addr_of_mut!(len),
                );
                RegCloseKey(hkey);
            }
        }
        if smoothscroll == 0 {
            flags |= 0x20000 as DWORD;
        }
    }

    if (flags & (0x20000 as DWORD)) != 0 {
        if ((*smooth).x2 & (1 as DWORD)) != 0 && (*smooth).scrollfun.is_some() {
            return (*smooth).scrollfun.unwrap_unchecked()(
                (*smooth).hwnd,
                (*smooth).dx as INT,
                (*smooth).dy as INT,
                (*smooth).lpscrollrect,
                (*smooth).lpcliprect,
                hrgnupdate,
                lpupdaterect,
                flags & (0xffff as DWORD),
            );
        } else {
            return ScrollWindowEx(
                (*smooth).hwnd,
                (*smooth).dx as INT,
                (*smooth).dy as INT,
                (*smooth).lpscrollrect as LPCRECT,
                (*smooth).lpcliprect as LPCRECT,
                hrgnupdate,
                lpupdaterect,
                (flags & (0xffff as DWORD)) as UINT,
            ) as BOOL;
        }
    }

    /* FIXME: do timer based smooth scrolling */
    if ((*smooth).x2 & (1 as DWORD)) != 0 && (*smooth).scrollfun.is_some() {
        return (*smooth).scrollfun.unwrap_unchecked()(
            (*smooth).hwnd,
            (*smooth).dx as INT,
            (*smooth).dy as INT,
            (*smooth).lpscrollrect,
            (*smooth).lpcliprect,
            hrgnupdate,
            lpupdaterect,
            flags & (0xffff as DWORD),
        );
    } else {
        return ScrollWindowEx(
            (*smooth).hwnd,
            (*smooth).dx as INT,
            (*smooth).dy as INT,
            (*smooth).lpscrollrect as LPCRECT,
            (*smooth).lpcliprect as LPCRECT,
            hrgnupdate,
            lpupdaterect,
            (flags & (0xffff as DWORD)) as UINT,
        ) as BOOL;
    }
}

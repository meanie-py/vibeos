use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_8B73AA59F63C: [WCHAR; 10] = [0x004D, 0x006F, 0x0075, 0x0073, 0x0065, 0x004B, 0x0065, 0x0079, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
static mut g_Mk: MOUSEKEYS = unsafe { core::mem::zeroed() };
static mut g_MkState: u32 = 0;
static mut g_MkStateIcon: HICON = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn MouseKeys_Init(pSysTray: *mut CSysTray) -> HRESULT {
    // TRACE("MouseKeys_Init!\n");
    MouseKeys_Update(pSysTray)
}

#[no_mangle]
pub unsafe extern "system" fn MouseKeys_Shutdown(pSysTray: *mut CSysTray) -> HRESULT {
    if !g_MkStateIcon.is_null() {
        DestroyIcon(g_MkStateIcon);
        g_MkStateIcon = core::ptr::null_mut();
    }

    if g_MkState != 0 {
        g_MkState = 0;
        (*pSysTray).NotifyIcon((NIM_DELETE) as INT, ID_ICON_MOUSE, g_MkStateIcon, CPP_COM_WSTR_8B73AA59F63C.as_ptr(), 0);
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn MouseKeys_Update(pSysTray: *mut CSysTray) -> HRESULT {
    g_Mk.cbSize = core::mem::size_of::<MOUSEKEYS>() as _;
    SystemParametersInfoW(SPI_GETMOUSEKEYS, core::mem::size_of::<MOUSEKEYS>() as _, (core::ptr::addr_of_mut!(g_Mk)).cast(), 0);

    let mut state: UINT = 0;
    if (g_Mk.dwFlags & (MKF_INDICATOR | MKF_MOUSEKEYSON)) == (MKF_INDICATOR | MKF_MOUSEKEYSON)
    {
        if g_Mk.dwFlags & MKF_MOUSEMODE != 0
        {
            let sw = g_Mk.dwFlags & (MKF_LEFTBUTTONDOWN | MKF_LEFTBUTTONSEL | MKF_RIGHTBUTTONDOWN | MKF_RIGHTBUTTONSEL);
            match sw {
                _ if sw == 0 => {
                    state = IDI_MOUSE_NOBTN;
                }
                _ if sw == MKF_LEFTBUTTONSEL => {
                    state = IDI_MOUSE_L_ACTIVE;
                }
                _ if sw == MKF_LEFTBUTTONDOWN || sw == (MKF_LEFTBUTTONDOWN | MKF_LEFTBUTTONSEL) => {
                    state = IDI_MOUSE_L_DOWN;
                }
                _ if sw == MKF_RIGHTBUTTONSEL => {
                    state = IDI_MOUSE_R_ACTIVE;
                }
                _ if sw == MKF_RIGHTBUTTONDOWN || sw == (MKF_RIGHTBUTTONDOWN | MKF_RIGHTBUTTONSEL) => {
                    state = IDI_MOUSE_R_DOWN;
                }
                _ if sw == (MKF_LEFTBUTTONSEL | MKF_RIGHTBUTTONSEL) => {
                    state = IDI_MOUSE_LR_ACTIVE;
                }
                _ if sw == (MKF_RIGHTBUTTONDOWN | MKF_LEFTBUTTONDOWN)
                    || sw == (MKF_RIGHTBUTTONDOWN | MKF_LEFTBUTTONDOWN | MKF_LEFTBUTTONSEL)
                    || sw == (MKF_RIGHTBUTTONDOWN | MKF_LEFTBUTTONDOWN | MKF_LEFTBUTTONSEL | MKF_RIGHTBUTTONSEL)
                    || sw == (MKF_RIGHTBUTTONDOWN | MKF_LEFTBUTTONDOWN | MKF_RIGHTBUTTONSEL) => {
                    state = IDI_MOUSE_LR_DOWN;
                }
                _ if sw == (MKF_LEFTBUTTONSEL | MKF_RIGHTBUTTONDOWN)
                    || sw == (MKF_LEFTBUTTONSEL | MKF_RIGHTBUTTONDOWN | MKF_RIGHTBUTTONSEL) => {
                    state = IDI_MOUSE_L_ACTIVE_R_DOWN;
                }
                _ if sw == (MKF_LEFTBUTTONDOWN | MKF_RIGHTBUTTONSEL)
                    || sw == (MKF_LEFTBUTTONDOWN | MKF_RIGHTBUTTONSEL | MKF_LEFTBUTTONSEL) => {
                    state = IDI_MOUSE_R_ACTIVE_L_DOWN;
                }
                _ => {
                    state = IDI_MOUSE_NOBTN;
                }
            }
        }
        else
        {
            state = IDI_MOUSE_DISABLED;
        }
    }

    let mut uId: UINT = NIM_MODIFY;
    if state != g_MkState
    {
        if !g_MkStateIcon.is_null()
        {
            DestroyIcon(g_MkStateIcon);
            g_MkStateIcon = core::ptr::null_mut();
        }

        if g_MkState == 0
            { uId = NIM_ADD; }

        g_MkState = state;
        if g_MkState != 0
        {
            g_MkStateIcon = LoadIcon(g_hInstance, MAKEINTRESOURCE!(g_MkState));
        }
    }

    if g_MkState == 0
    {
        uId = NIM_DELETE;
    }

    return (*pSysTray).NotifyIcon((uId) as INT, ID_ICON_MOUSE, g_MkStateIcon, CPP_COM_WSTR_8B73AA59F63C.as_ptr(), 0);
}
#[no_mangle]
pub unsafe extern "system" fn MouseKeys_Message(pSysTray: *mut CSysTray, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: &mut LRESULT) -> HRESULT {
    match uMsg {
        ID_ICON_MOUSE => {
            match lParam {
                __c if __c == WM_LBUTTONDBLCLK as _ || __c == WM_RBUTTONDBLCLK as _ => {
                    CSysTray::RunAccessCpl(b",,4\0".as_ptr() as *const i8);
                    return S_OK;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return S_FALSE;
}

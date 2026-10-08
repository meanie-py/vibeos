use crate::appwiz::addons::install_addon;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_104786D3AB8F: [u16; 14] = [
    0x0069u16, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x005F, 0x0067, 0x0065,
    0x0063, 0x006B, 0x006F, 0x0000,
];
pub static RUSTCONV_WSTR_1697F655B61E: [u16; 10] = [0x0072u16, 0x0061, 0x0070, 0x0070, 0x0073, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_A906296EB33B: [u16; 13] = [
    0x0069u16, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x005F, 0x006D, 0x006F,
    0x006E, 0x006F, 0x0000,
];
pub static RUSTCONV_WSTR_F7A506268886: [u16; 8] = [0x002Fu16, 0x0061, 0x0070, 0x0070, 0x0077, 0x0069, 0x007A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

unsafe extern "C" fn start_params(params: *const WCHAR, hwnd_parent: HWND) -> LONG {
    if params.is_null() {
        return (FALSE) as LONG;
    }

    if strcmpW(params, RUSTCONV_WSTR_104786D3AB8F.as_ptr()) == 0 {
        install_addon(ADDON_GECKO, hwnd_parent);
        return (TRUE) as LONG;
    }

    if strcmpW(params, RUSTCONV_WSTR_A906296EB33B.as_ptr()) == 0 {
        install_addon(ADDON_MONO, hwnd_parent);
        return (TRUE) as LONG;
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(hwndCPl: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    let mut CPlInfo: *mut CPLINFO = core::ptr::null_mut();

    match uMsg {
        CPL_INIT => {
            return (TRUE) as LONG;
        }
        CPL_GETCOUNT => {
            return 1;
        }
        CPL_STARTWPARMSW => {
            return start_params(lParam2 as *const WCHAR, hwndCPl);
        }
        CPL_INQUIRE => {
            CPlInfo = lParam2 as *mut CPLINFO;
            (*CPlInfo).lData = 0;
            (*CPlInfo).idIcon = (IDI_CPLSYSTEM) as i32;
            (*CPlInfo).idName = (IDS_CPLSYSTEMNAME) as i32;
            (*CPlInfo).idInfo = (IDS_CPLSYSTEMDESCRIPTION) as i32;
        }
        CPL_DBLCLK => {
            ShellExecuteW(
                core::ptr::null_mut(),
                core::ptr::null(),
                RUSTCONV_WSTR_1697F655B61E.as_ptr(),
                RUSTCONV_WSTR_F7A506268886.as_ptr(),
                core::ptr::null(),
                (SW_SHOWNORMAL) as INT,
            );
        }
        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        v if v == DLL_PROCESS_ATTACH || v == DLL_THREAD_ATTACH => {
            CoInitialize(core::ptr::null_mut());
            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

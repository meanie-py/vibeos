// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_C7DFBF431ADC: [u16; 11] = [
    0x0074u16, 0x0061, 0x0070, 0x0069, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type LPINTERNALCONFIG = Option<unsafe extern "system" fn(HWND, u32, LPARAM, LPARAM) -> i32>;

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(hwndCPl: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    let mut lpInternalConfig: LPINTERNALCONFIG = None;
    let mut hTapi32: HINSTANCE = core::ptr::null_mut();
    let mut CPlInfo: *mut CPLINFO = core::ptr::null_mut();

    match uMsg {
        CPL_INIT => {
            return (TRUE) as LONG;
        }

        CPL_GETCOUNT => {
            return 1;
        }

        CPL_INQUIRE => {
            CPlInfo = lParam2 as *mut CPLINFO;
            (*CPlInfo).lData = 0;
            (*CPlInfo).idIcon = (IDI_CPLSYSTEM as i32);
            (*CPlInfo).idName = (IDS_CPLSYSTEMNAME as i32);
            (*CPlInfo).idInfo = (IDS_CPLSYSTEMDESCRIPTION as i32);
        }

        CPL_DBLCLK => {
            hTapi32 = LoadLibraryW(RUSTCONV_WSTR_C7DFBF431ADC.as_ptr());
            if hTapi32.is_null() {
                return (FALSE) as LONG;
            }

            lpInternalConfig = core::mem::transmute::<FARPROC, LPINTERNALCONFIG>(
                GetProcAddress(hTapi32, b"internalConfig\0".as_ptr() as *const i8),
            );
            // C: `if (!lpInternalConfig) { FreeLibrary(..); return FALSE; }` then the call.
            let Some(lpInternalConfig) = lpInternalConfig else {
                FreeLibrary(hTapi32);
                return (FALSE) as LONG;
            };
            lpInternalConfig(hwndCPl, 0, 0, 0);
            FreeLibrary(hTapi32);
            return (TRUE) as LONG;
        }

        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    (TRUE) as i32
}

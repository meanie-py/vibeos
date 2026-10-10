// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0639853DD20F: [u16; 8] = [0x004Du16, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_87498FF89732: [u16; 32] = [
    0x0054u16, 0x0068, 0x0069, 0x0073, 0x0020, 0x0066, 0x0075, 0x006E, 0x0063, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020, 0x0075, 0x006E, 0x0069,
    0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064,
    0x002E, 0x0000,
];
pub static RUSTCONV_WSTR_F5FCD819687C: [u16; 14] = [
    0x0055u16, 0x006E, 0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E,
    0x0074, 0x0065, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_F9F70C9DEDFE: [u16; 14] = [
    0x0044u16, 0x0069, 0x0061, 0x006C, 0x006F, 0x0067, 0x0020, 0x0045, 0x0072, 0x0072,
    0x006F, 0x0072, 0x0021, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn ShutdownGuiProc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> INT_PTR {
    match msg {
        WM_INITDIALOG => {}
        WM_COMMAND => {
            match LOWORD!(wparam) as UINT {
                IDC_OK => {
                    EndDialog(hwnd, IDC_OK as INT_PTR);
                }
                IDC_CANCEL => {
                    EndDialog(hwnd, IDC_CANCEL as INT_PTR);
                }
                _ => {}
            }
        }
        WM_CLOSE => {
            EndDialog(hwnd, 0 as INT_PTR);
        }
        _ => return FALSE as INT_PTR,
    }
    return TRUE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn ShutdownGuiMain(opts: CommandLineOptions) -> BOOL {
    let result: INT_PTR = DialogBoxW!(
        GetModuleHandle(core::ptr::null_mut()),
        MAKEINTRESOURCEW!(IDD_GUI),
        core::ptr::null_mut(),
        Some(ShutdownGuiProc),
    );

    match (result) as u32 {
        IDC_OK => {
            MessageBoxW(core::ptr::null_mut(), RUSTCONV_WSTR_87498FF89732.as_ptr(), RUSTCONV_WSTR_F5FCD819687C.as_ptr(), MB_OK);
        }
        IDC_CANCEL => {
            /* Exits the program */
        }
        _ => {
            MessageBoxW(core::ptr::null_mut(), RUSTCONV_WSTR_F9F70C9DEDFE.as_ptr(), RUSTCONV_WSTR_0639853DD20F.as_ptr(), MB_OK);
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}

use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn GetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut calc_HtmlHelpA: type_HtmlHelpA =
    unsafe { Some(dummy_HtmlHelpA) };

#[no_mangle]
pub static mut calc_HtmlHelpW: type_HtmlHelpW =
    unsafe { Some(dummy_HtmlHelpW) };

static mut hHtmlHelp: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// GET_CB(name) is a function-like macro: skipped (the functions inline it).
// Forward prototypes of dummy_HtmlHelpA / dummy_HtmlHelpW are not declarations
// Rust needs; the definitions are emitted with their function bodies.
unsafe extern "system" fn dummy_HtmlHelpA(
    hWnd: HWND,
    pszFile: LPCSTR,
    uCommand: UINT,
    dwData: DWORD,
) -> HWND {
    (NULL).cast()
}
unsafe extern "system" fn dummy_HtmlHelpW(
    hWnd: HWND,
    pszFile: LPCWSTR,
    uCommand: UINT,
    dwData: DWORD,
) -> HWND {
    (NULL).cast()
}
#[no_mangle]
pub unsafe extern "C" fn HtmlHelp_Start(mut hInstance: HINSTANCE) {
    let HHCTRL_OCX: [u16; 11] = ['H' as u16, 'H' as u16, 'C' as u16, 'T' as u16, 'R' as u16, 'L' as u16, '.' as u16, 'O' as u16, 'C' as u16, 'X' as u16, 0];
    hHtmlHelp = LoadLibrary(HHCTRL_OCX.as_ptr() as LPCWSTR) as HMODULE;
    if hHtmlHelp.is_null() {
        return;
    }
    calc_HtmlHelpW = ::core::mem::transmute::<FARPROC, type_HtmlHelpW>(GetProcAddress(
        hHtmlHelp as HINSTANCE,
        b"HtmlHelpW\0" as *const u8 as LPCSTR,
    ));
    if calc_HtmlHelpW.is_none() {
        calc_HtmlHelpW =
            Some(dummy_HtmlHelpW);
    }
    calc_HtmlHelpA = ::core::mem::transmute::<FARPROC, type_HtmlHelpA>(GetProcAddress(
        hHtmlHelp as HINSTANCE,
        b"HtmlHelpA\0" as *const u8 as LPCSTR,
    ));
    if calc_HtmlHelpA.is_none() {
        calc_HtmlHelpA =
            Some(dummy_HtmlHelpA);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HtmlHelp_Stop() {
    if hHtmlHelp.is_null() {
        return;
    }
    FreeLibrary(hHtmlHelp);
    hHtmlHelp = core::ptr::null_mut();
}

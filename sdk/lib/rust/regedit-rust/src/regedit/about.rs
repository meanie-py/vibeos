use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ShowAboutBox(hWnd: HWND) {
    let mut AppStr: [WCHAR; 255] = [0; 255];
    LoadStringW(hInst, IDS_APP_TITLE as UINT, core::ptr::addr_of_mut!(AppStr) as LPWSTR, ARRAY_SIZE!(AppStr) as ::core::ffi::c_int);
    ShellAboutW(hWnd, core::ptr::addr_of_mut!(AppStr) as *mut WCHAR as LPCWSTR, core::ptr::null::<WCHAR>(), LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_REGEDIT) as LPWSTR as LPCWSTR));
}

use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn OnAbout() {
    let mut szTaskmgr: [WCHAR; 128] = [0; 128];

    LoadStringW(
        hInst,
        IDS_APP_TITLE as UINT,
        szTaskmgr.as_mut_ptr() as LPWSTR,
        _countof!(szTaskmgr) as ::core::ffi::c_int,
    );
    ShellAboutW(
        hMainWnd,
        szTaskmgr.as_mut_ptr() as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        LoadIconW(
            hInst,
            MAKEINTRESOURCEW!(IDI_TASKMANAGER) as LPCWSTR,
        ),
    );
}

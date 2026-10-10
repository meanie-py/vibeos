use crate::winver::osinfo::Winver_GetOSInfo;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_33FB62474A78: [u16; 8] = [0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut Winver_hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32 {
    let mut iccx: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut OSInfo: WINVER_OS_INFO = core::mem::zeroed();

    let _ = hPrevInstance;
    let _ = lpCmdLine;
    let _ = nCmdShow;

    Winver_hInstance = hInstance;

    /* Initialize common controls */
    iccx.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccx.dwICC = (ICC_STANDARD_CLASSES | ICC_WIN95_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccx));

    if Winver_GetOSInfo(core::ptr::addr_of_mut!(OSInfo)) == 0
    {
        /* OS info is not available, display the default contents */
        StringCchCopyW(
            core::ptr::addr_of_mut!(OSInfo.szName) as *mut WCHAR,
            core::mem::size_of::<[WCHAR; 64]>() / core::mem::size_of::<WCHAR>(),
            RUSTCONV_WSTR_33FB62474A78.as_ptr(),
        );
        OSInfo.szCompatInfo[0 as usize] = UNICODE_NULL;
    }

    return ShellAboutW(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(OSInfo.szName) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(OSInfo.szCompatInfo) as *mut WCHAR as LPCWSTR,
        core::ptr::null_mut(),
    );
}

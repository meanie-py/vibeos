use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #define WIN32_NO_STATUS (preprocessor-only, no Rust equivalent)
// #define WIN32_LEAN_AND_MEAN (preprocessor-only, no Rust equivalent)

// #define REGEDIT _T("regedit.exe") -- wide (UNICODE) TCHAR string, NUL-terminated
pub static REGEDIT: [TCHAR; 12] = [
    0x72, // 'r'
    0x65, // 'e'
    0x67, // 'g'
    0x65, // 'e'
    0x64, // 'd'
    0x69, // 'i'
    0x74, // 't'
    0x2E, // '.'
    0x65, // 'e'
    0x78, // 'x'
    0x65, // 'e'
    0x00, // NUL
];

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hCurInst: HINSTANCE,
    hPrevInst: HINSTANCE,
    lpsCmdLine: LPTSTR,
    nCmdShow: i32,
) -> i32 {
    let mut szPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if GetWindowsDirectory(core::ptr::addr_of_mut!(szPath) as LPTSTR, MAX_PATH) != 0
    {
        PathAppend(core::ptr::addr_of_mut!(szPath) as LPTSTR, (REGEDIT).as_ptr());
        ShellExecute(
            core::ptr::null_mut(),
            core::ptr::null(),
            core::ptr::addr_of_mut!(szPath) as LPTSTR as *const TCHAR,
            lpsCmdLine as *const TCHAR,
            core::ptr::null(),
            nCmdShow,
        );
    } else {
        ShellExecute(
            core::ptr::null_mut(),
            core::ptr::null(),
            (REGEDIT).as_ptr(),
            lpsCmdLine as *const TCHAR,
            core::ptr::null(),
            nCmdShow,
        );
    }

    return 0;
}

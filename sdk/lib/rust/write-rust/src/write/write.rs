use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut SZ_WORDPAD: [WCHAR; 13] = [
    '\\' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];
#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    mut hInstance: HINSTANCE,
    mut hOldInstance: HINSTANCE,
    mut szCmdParagraph: LPWSTR,
    mut res: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut stinf: STARTUPINFOW = core::mem::zeroed();
    let mut info: PROCESS_INFORMATION = core::mem::zeroed();

    'failed: {
        if GetSystemDirectoryW(
            path.as_mut_ptr(),
            (MAX_PATH - 1 - (lstrlenW((SZ_WORDPAD).as_ptr()) as UINT)) as UINT,
        ) == 0
        {
            break 'failed;
        }
        lstrcatW(path.as_mut_ptr(), (SZ_WORDPAD).as_ptr());

        stinf.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
        GetStartupInfoW(core::ptr::addr_of_mut!(stinf));

        if CreateProcessW(
            path.as_mut_ptr(),
            GetCommandLineW(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (FALSE) as BOOL,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(stinf),
            core::ptr::addr_of_mut!(info),
        ) == 0
        {
            break 'failed;
        }
        CloseHandle(info.hProcess);
        CloseHandle(info.hThread);
        return 0;
    }

    // failed:
    LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()),
        IDS_FAILED,
        path.as_mut_ptr(),
        (MAX_PATH) as i32,
    );
    MessageBoxW(
        core::ptr::null_mut(),
        path.as_mut_ptr(),
        core::ptr::null_mut(),
        MB_OK | MB_ICONERROR,
    );
    return 1;
}

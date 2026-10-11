use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `IEWinMain` is a function prototype, not a constant, type, or table. It is declared in the extern import list, so no declaration is emitted here.

unsafe extern "C" fn check_native_ie() -> BOOL {
    let mut handle: DWORD = 0;
    let mut size: DWORD = 0;
    let mut file_desc: LPWSTR = core::ptr::null_mut();
    let mut bytes: UINT = 0;
    let mut buf: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ret: BOOL = (TRUE as i32);
    let mut translation: LPWORD = core::ptr::null_mut();

    static browseui_dllW: [WCHAR; 13] = [
    0x0062u16, 0x0072, 0x006F, 0x0077, 0x0073, 0x0065, 0x0075, 0x0069, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
    static wineW: [WCHAR; 5] = [0x0057u16, 0x0069, 0x006E, 0x0065, 0x0000];
    static translationW: [WCHAR; 25] = [
    0x005Cu16, 0x0056, 0x0061, 0x0072, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E,
    0x0066, 0x006F, 0x005C, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006C, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
    static file_desc_fmtW: [WCHAR; 41] = [
    0x005Cu16, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0078,
    0x0025, 0x0030, 0x0034, 0x0078, 0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0044,
    0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0000,
];
    let mut file_desc_strW: [WCHAR; 48] = [0; 48];

    size = GetFileVersionInfoSizeW(
        core::ptr::addr_of!(browseui_dllW) as LPCWSTR,
        core::ptr::addr_of_mut!(handle),
    );
    if size == 0 {
        return (TRUE) as BOOL;
    }

    buf = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        size as SIZE_T,
    ) as *mut core::ffi::c_void;
    GetFileVersionInfoW(
        core::ptr::addr_of!(browseui_dllW) as LPCWSTR,
        0 as DWORD,
        size,
        buf as PVOID,
    );
    if VerQueryValueW(
        buf as LPCVOID,
        core::ptr::addr_of!(translationW) as LPCWSTR,
        core::ptr::addr_of_mut!(translation) as *mut LPVOID,
        core::ptr::addr_of_mut!(bytes),
    ) != 0
    {
        wsprintfW(
            core::ptr::addr_of_mut!(file_desc_strW) as LPWSTR,
            core::ptr::addr_of!(file_desc_fmtW) as LPCWSTR,
            *translation.offset(0 as isize) as core::ffi::c_int,
            *translation.offset(1 as isize) as core::ffi::c_int,
        );
        ret = (VerQueryValueW(
            buf as LPCVOID,
            core::ptr::addr_of!(file_desc_strW) as LPCWSTR,
            core::ptr::addr_of_mut!(file_desc) as *mut LPVOID,
            core::ptr::addr_of_mut!(bytes),
        ) == 0
            || strstrW!(file_desc, core::ptr::addr_of!(wineW) as LPCWSTR).is_null()) as BOOL;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        buf as PVOID,
    );
    return ret;
}

unsafe extern "C" fn register_iexplore(doregister: BOOL) -> DWORD {
    let mut hres: HRESULT;

    if check_native_ie() != 0 {
        WINE_MESSAGE(b"Native IE detected, not doing registration\n\0".as_ptr() as *const i8);
        return 0 as DWORD;
    }

    hres = RegInstallA(
        core::ptr::null_mut(),
        if doregister != 0 {
            b"RegisterIE\0".as_ptr() as *const i8
        } else {
            b"UnregisterIE\0".as_ptr() as *const i8
        },
        core::ptr::null_mut(),
    );
    return (hres < 0 as HRESULT) as i32 as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    prev: HINSTANCE,
    cmdline: *mut WCHAR,
    show: i32,
) -> i32 {
    static regserverW: [u16; 10] = [0x0072u16, 0x0065, 0x0067, 0x0073, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000];
    static unregserverW: [u16; 12] = [
    0x0075u16, 0x006E, 0x0072, 0x0065, 0x0067, 0x0073, 0x0065, 0x0072, 0x0076, 0x0065,
    0x0072, 0x0000,
];

    if *cmdline == '-' as WCHAR || *cmdline == '/' as WCHAR {
        if strcmpiW!(
            cmdline.offset(1 as isize),
            core::ptr::addr_of!(regserverW) as *const WCHAR,
        ) == 0
        {
            return register_iexplore((TRUE) as BOOL) as i32;
        }
        if strcmpiW!(
            cmdline.offset(1 as isize),
            core::ptr::addr_of!(unregserverW) as *const WCHAR,
        ) == 0
        {
            return register_iexplore((FALSE) as BOOL) as i32;
        }
    }

    return IEWinMain(cmdline, show) as i32;
}

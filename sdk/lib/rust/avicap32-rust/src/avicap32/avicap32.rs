// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0FE281D2A052: [u16; 20] = [
    0x0055u16, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0020, 0x0044, 0x0072,
    0x0069, 0x0076, 0x0065, 0x0072, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_B5B0A80B585D: [u16; 13] = [
    0x0046u16, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_E82F21F7F22E: [u16; 56] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x004D, 0x0065, 0x0064, 0x0069, 0x0061, 0x0052, 0x0065,
    0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0073, 0x005C, 0x006D, 0x0073,
    0x0076, 0x0069, 0x0064, 0x0065, 0x006F, 0x0000,
];
pub static RUSTCONV_WSTR_F3D8D400CF21: [u16; 7] = [0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_F56CCA838643: [u16; 21] = [
    0x0056u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x003A, 0x0020, 0x0025,
    0x0064, 0x002E, 0x0025, 0x0064, 0x002E, 0x0025, 0x0064, 0x002E, 0x0025, 0x0064,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub const CAP_DESC_MAX: i32 = 32;
#[no_mangle]
pub unsafe extern "system" fn CaptureWindowProc(hwnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match Msg {
        WM_CREATE => {}
        WM_PAINT => {}
        WM_DESTROY => {}
        _ => {}
    }

    return DefWindowProc(hwnd, Msg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn capCreateCaptureWindowW(
    lpszWindowName: LPCWSTR,
    dwStyle: DWORD,
    x: INT,
    y: INT,
    nWidth: INT,
    nHeight: INT,
    hWnd: HWND,
    nID: INT,
) -> HWND {
    let mut szWindowClass: [u16; 10] = [0x0043u16, 0x006C, 0x0073, 0x0043, 0x0061, 0x0070, 0x0057, 0x0069, 0x006E, 0x0000];
    let mut WndClass: WNDCLASSEXW = core::mem::zeroed();
    let mut dwExStyle: DWORD = 0;

    WndClass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    WndClass.lpszClassName = szWindowClass.as_mut_ptr() as LPCWSTR;
    WndClass.lpfnWndProc = Some(CaptureWindowProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT) as WNDPROC;
    WndClass.hInstance = hInstance;
    WndClass.style = (CS_HREDRAW | CS_VREDRAW) as UINT;
    WndClass.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW);
    WndClass.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;

    if RegisterClassExW(core::ptr::addr_of_mut!(WndClass)) == 0 as ATOM
    {
        if GetLastError() != ERROR_ALREADY_EXISTS {
            return core::ptr::null_mut();
        }
    }

    return CreateWindowExW(
        dwExStyle,
        szWindowClass.as_mut_ptr() as LPCWSTR,
        lpszWindowName,
        dwStyle,
        x,
        y,
        nWidth,
        nHeight,
        hWnd,
        (ULongToHandle!(nID as ULONG)).cast(),
        hInstance,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn capCreateCaptureWindowA(
    lpszWindowName: LPCSTR,
    dwStyle: DWORD,
    x: INT,
    y: INT,
    nWidth: INT,
    nHeight: INT,
    hWnd: HWND,
    nID: INT,
) -> HWND {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Wnd: HWND = core::ptr::null_mut();

    if !lpszWindowName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(Name), lpszWindowName);
    } else {
        (*core::ptr::addr_of_mut!(Name)).Buffer = core::ptr::null_mut();
    }

    Wnd = capCreateCaptureWindowW(
        (*core::ptr::addr_of_mut!(Name)).Buffer as LPCWSTR,
        dwStyle,
        x,
        y,
        nWidth,
        nHeight,
        hWnd,
        nID,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(Name));
    return Wnd;
}

#[no_mangle]
pub unsafe extern "system" fn capGetDriverDescriptionW(
    wDriverIndex: WORD,
    lpszName: LPWSTR,
    cbName: INT,
    lpszVer: LPWSTR,
    cbVer: INT,
) -> BOOL {
    let mut dwSize: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut szDriver: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szDriverName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szVersion: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();

    if !lpszName.is_null() && cbName != 0 {
        *lpszName.add(0 as usize) = 0x0000u16;
    }

    if !lpszVer.is_null() && cbVer != 0 {
        *lpszVer.add(0 as usize) = 0x0000u16;
    }

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_E82F21F7F22E.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) != (ERROR_SUCCESS as i32)
    {
        return (FALSE) as BOOL;
    }

    dwSize = (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
        / core::mem::size_of::<WCHAR>()) as DWORD;

    while RegEnumKeyExW(
        hKey,
        dwIndex,
        core::ptr::addr_of_mut!(szDriver) as *mut WCHAR as LPWSTR,
        core::ptr::addr_of_mut!(dwSize),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == (ERROR_SUCCESS as i32)
    {
        if RegOpenKeyW(
            hKey,
            core::ptr::addr_of_mut!(szDriver) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(hSubKey),
        ) == (ERROR_SUCCESS as i32)
        {
            dwSize = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;

            if RegQueryValueExW(
                hSubKey,
                RUSTCONV_WSTR_F3D8D400CF21.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as LPBYTE,
                core::ptr::addr_of_mut!(dwSize),
            ) == (ERROR_SUCCESS as i32)
            {
                dwSize = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;

                if RegQueryValueExW(
                    hSubKey,
                    RUSTCONV_WSTR_B5B0A80B585D.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(szDriverName) as *mut WCHAR as LPBYTE,
                    core::ptr::addr_of_mut!(dwSize),
                ) != (ERROR_SUCCESS as i32)
                {
                    wcscpy(
                        core::ptr::addr_of_mut!(szDriverName) as *mut WCHAR,
                        RUSTCONV_WSTR_0FE281D2A052.as_ptr(),
                    );
                }

                if dwIndex == (wDriverIndex as DWORD) {
                    if !lpszName.is_null() && cbName != 0 {
                        lstrcpynW(
                            lpszName,
                            core::ptr::addr_of_mut!(szDriverName) as *mut WCHAR as LPCWSTR,
                            cbName,
                        );
                    }

                    if !lpszVer.is_null() && cbVer != 0 {
                        let mut Version: LPVOID = core::ptr::null_mut();
                        let mut Ms: LPVOID = core::ptr::null_mut();
                        let mut dwInfoSize: DWORD = 0;
                        let mut FileInfo: VS_FIXEDFILEINFO = core::mem::zeroed();
                        let mut Ls: UINT = 0;

                        dwInfoSize = GetFileVersionInfoSize(
                            core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as LPCWSTR,
                            core::ptr::null_mut(),
                        );
                        if dwInfoSize != 0 {
                            Version = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                dwInfoSize as SIZE_T,
                            ) as LPVOID;

                            if !Version.is_null() {
                                GetFileVersionInfo(
                                    core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as LPCWSTR,
                                    0,
                                    dwInfoSize,
                                    Version,
                                );

                                if VerQueryValueW(
                                    Version as LPCVOID,
                                    RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
                                    core::ptr::addr_of_mut!(Ms),
                                    core::ptr::addr_of_mut!(Ls),
                                ) != 0
                                {
                                    memmove(
                                        core::ptr::addr_of_mut!(FileInfo) as *mut core::ffi::c_void,
                                        Ms as *const core::ffi::c_void,
                                        Ls as usize,
                                    );
                                    _swprintf(
                                        core::ptr::addr_of_mut!(szVersion) as *mut WCHAR,
                                        RUSTCONV_WSTR_F56CCA838643.as_ptr(),
                                        HIWORD!(FileInfo.dwFileVersionMS) as i32,
                                        LOWORD!(FileInfo.dwFileVersionMS) as i32,
                                        HIWORD!(FileInfo.dwFileVersionLS) as i32,
                                        LOWORD!(FileInfo.dwFileVersionLS) as i32,
                                    );

                                    lstrcpynW(
                                        lpszVer,
                                        core::ptr::addr_of_mut!(szVersion) as *mut WCHAR as LPCWSTR,
                                        cbVer,
                                    );
                                }
                                HeapFree(
                                    GetProcessHeap(),
                                    0,
                                    Version as PVOID,
                                );
                            }
                        }
                    }

                    RegCloseKey(hSubKey);
                    RegCloseKey(hKey);
                    return (TRUE) as BOOL;
                }
            }

            RegCloseKey(hSubKey);
        }

        dwSize = (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
            / core::mem::size_of::<WCHAR>()) as DWORD;
        dwIndex = dwIndex.wrapping_add(1);
    }

    RegCloseKey(hKey);

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn capGetDriverDescriptionA(
    mut wDriverIndex: WORD,
    mut lpszName: LPSTR,
    mut cbName: INT,
    mut lpszVer: LPSTR,
    mut cbVer: INT,
) -> BOOL {
    let mut DevName: [WCHAR; (CAP_DESC_MAX as _)] = [0; (CAP_DESC_MAX as _)];
    let mut DevVer: [WCHAR; (CAP_DESC_MAX as _)] = [0; (CAP_DESC_MAX as _)];
    let mut Result: BOOL = 0;
    Result = capGetDriverDescriptionW(
        wDriverIndex,
        core::ptr::addr_of_mut!(DevName) as LPWSTR,
        CAP_DESC_MAX as INT,
        core::ptr::addr_of_mut!(DevVer) as LPWSTR,
        CAP_DESC_MAX as INT,
    );
    if Result != 0 {
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(DevName) as *mut WCHAR as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            lpszName,
            cbName as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(DevVer) as *mut WCHAR as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            lpszVer,
            cbVer as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn AppCleanup(hInst: HINSTANCE) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn videoThunk32(
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
    dwUnknown3: DWORD,
    dwUnknown4: DWORD,
    dwUnknown5: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            TRACE!("avicap32 attached!\n");
            hInstance = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

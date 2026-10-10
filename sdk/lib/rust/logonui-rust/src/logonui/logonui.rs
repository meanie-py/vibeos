// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_A014E4404844: [u16; 24] = [
    0x0053u16, 0x0061, 0x0076, 0x0069, 0x006E, 0x0067, 0x0020, 0x0079, 0x006F, 0x0075,
    0x0072, 0x0020, 0x0073, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073,
    0x002E, 0x002E, 0x002E, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_pInfo: PINFO = ::core::ptr::null::<_INFO>() as *mut _INFO;

unsafe extern "C" fn DrawBaseBackground(hdcDesktop: HDC) -> HDC {
    let mut hdcMem: HDC;

    hdcMem = NT5_DrawBaseBackground(hdcDesktop);

    return hdcMem;
}

unsafe extern "C" fn DrawLogoffScreen(hdcMem: HDC) {
    NT5_CreateLogoffScreen(RUSTCONV_WSTR_A014E4404844.as_ptr() as LPWSTR, hdcMem);
}

unsafe extern "C" fn ParseCmdline(mut argc: ::core::ffi::c_int, mut argv: *mut *mut WCHAR) -> BOOL {
    let _ = (&mut argc, &mut argv);
    return TRUE as BOOL;
}

unsafe extern "C" fn Run() {
    let mut hDesktopWnd: HWND = ::core::ptr::null_mut();
    let mut hdcDesktop: HDC = ::core::ptr::null_mut();
    let mut hdcMem: HDC = ::core::ptr::null_mut();

    /* Get the screen size */
    (*g_pInfo).cx = GetSystemMetrics(SM_CXSCREEN as ::core::ffi::c_int) as INT;
    (*g_pInfo).cy = GetSystemMetrics(SM_CYSCREEN as ::core::ffi::c_int) as INT;

    hDesktopWnd = GetDesktopWindow();

    /* Get the DC for the desktop */
    hdcDesktop = GetDCEx(hDesktopWnd, ::core::ptr::null_mut(), DCX_CACHE as DWORD);
    if !hdcDesktop.is_null() {
        /* Initialize the base background onto a DC */
        hdcMem = DrawBaseBackground(hdcDesktop);
        if !hdcMem.is_null() {
            /* TEST : Draw logoff screen */
            DrawLogoffScreen(hdcMem);

            /* Blit the off-screen DC to the desktop */
            BitBlt(
                hdcDesktop,
                0 as INT,
                0 as INT,
                (*g_pInfo).cx,
                (*g_pInfo).cy,
                hdcMem,
                0 as INT,
                0 as INT,
                SRCCOPY as DWORD,
            );

            /* Delete the memory DC */
            DeleteDC(hdcMem);
        }

        /* Release the desktop DC */
        ReleaseDC(hDesktopWnd, hdcDesktop);
    }
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInst: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpszCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32 {
    let mut lpArgs: *mut LPWSTR = core::ptr::null_mut();
    let mut NumArgs: INT = 0;

    /* Allocate memory for the data */
    g_pInfo = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<INFO>() as SIZE_T,
    ) as PINFO;
    if g_pInfo.is_null() {
        return -(1 as i32);
    }

    (*g_pInfo).hInstance = hInst;

    /* Get the command line args */
    lpArgs = CommandLineToArgvW(lpszCmdLine as LPCWSTR, core::ptr::addr_of_mut!(NumArgs));
    if !lpArgs.is_null() {
        /* Parse the command line */
        if ParseCmdline(NumArgs as i32, lpArgs as *mut *mut WCHAR) != 0 {
            /* Start the main routine */
            Run();
        }
    }

    /* Free the data */
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        g_pInfo as PVOID,
    );

    return 0 as i32;
}

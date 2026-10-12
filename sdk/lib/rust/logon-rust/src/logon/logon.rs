// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_6C49BEACD7CB: [u16; 6] = [0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_C3B9D763EC47: [u16; 35] = [
    0x0046u16, 0x0061, 0x0074, 0x0061, 0x006C, 0x0020, 0x0045, 0x0072, 0x0072, 0x006F,
    0x0072, 0x003A, 0x0020, 0x0043, 0x006F, 0x0075, 0x006C, 0x0064, 0x0020, 0x006E,
    0x006F, 0x0074, 0x0020, 0x006C, 0x006F, 0x0061, 0x0064, 0x0020, 0x0062, 0x0069,
    0x0074, 0x006D, 0x0061, 0x0070, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const APP_TIMER: i32 = 1;

pub const APP_TIMER_INTERVAL: i32 = 10000;

unsafe extern "C" fn GetScreenSaverBitmap() -> HBITMAP {
    let mut osvi: OSVERSIONINFOEX = core::mem::zeroed();
    osvi.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEX>() as DWORD;
    GetVersionEx((core::ptr::addr_of_mut!(osvi) as POSVERSIONINFO).cast());

    return LoadImageW(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        (if osvi.wProductType == (VER_NT_WORKSTATION as u8) {
            MAKEINTRESOURCEW!(IDB_WORKSTATION)
        } else {
            MAKEINTRESOURCEW!(IDB_SERVER)
        }) as LPCWSTR,
        IMAGE_BITMAP as UINT,
        0 as core::ffi::c_int,
        0 as core::ffi::c_int,
        LR_CREATEDIBSECTION as UINT,
    ) as HBITMAP;
}

#[no_mangle]
pub unsafe extern "system" fn ScreenSaverProc(
    mut hWnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    static mut bitmap: HBITMAP = core::ptr::null_mut();

    match uMsg {
        WM_CREATE => {
            bitmap = GetScreenSaverBitmap();
            if bitmap.is_null() {
                /* Extremely unlikely, message not localized. */
                MessageBoxW(
                    hWnd,
                    RUSTCONV_WSTR_C3B9D763EC47.as_ptr(),
                    RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(),
                    (MB_OK | MB_ICONEXCLAMATION) as UINT,
                );
            }

            SetTimer(
                hWnd,
                APP_TIMER as UINT_PTR,
                APP_TIMER_INTERVAL as UINT,
                None,
            );
        }
        WM_PAINT => {
            let mut bm: BITMAP = core::mem::zeroed();
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut hdc: HDC = core::ptr::null_mut();
            let mut hdcMem: HDC = core::ptr::null_mut();
            let mut hbmOld: HBITMAP = core::ptr::null_mut();
            let mut rect: RECT = core::mem::zeroed();

            hdc = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
            hdcMem = CreateCompatibleDC(hdc);
            hbmOld = SelectObject(hdcMem, bitmap as HGDIOBJ) as HBITMAP;
            GetObjectW(
                bitmap as HANDLE,
                core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(bm) as LPVOID,
            );

            GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
            if rect.right < bm.bmWidth || rect.bottom < bm.bmHeight {
                StretchBlt(
                    hdc,
                    RANDOM!(
                        0,
                        rect.right - (bm.bmWidth / 5),
                    ),
                    RANDOM!(
                        0,
                        rect.bottom - (bm.bmHeight / 5),
                    ),
                    bm.bmWidth / 5,
                    bm.bmHeight / 5,
                    hdcMem,
                    0,
                    0,
                    bm.bmWidth,
                    bm.bmHeight,
                    SRCCOPY,
                );
            } else {
                BitBlt(
                    hdc,
                    RANDOM!(0, rect.right - bm.bmWidth),
                    RANDOM!(0, rect.bottom - bm.bmHeight),
                    bm.bmWidth,
                    bm.bmHeight,
                    hdcMem,
                    0,
                    0,
                    SRCCOPY,
                );
            }

            SelectObject(hdcMem, hbmOld as HGDIOBJ);
            DeleteDC(hdcMem);
            EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
        }
        WM_TIMER => {
            InvalidateRect(hWnd, core::ptr::null(), (TRUE) as BOOL);
        }
        WM_DESTROY => {
            KillTimer(hWnd, APP_TIMER as UINT_PTR);
            DeleteObject(bitmap as HGDIOBJ);
            PostQuitMessage(0 as ::core::ffi::c_int);
        }
        _ => {
            /* Pass window messages to the default screensaver window procedure */
            return DefScreenSaverProc(hWnd, uMsg, wParam, lParam);
        }
    }

    return 0 as LRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn ScreenSaverConfigureDialog(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn RegisterDialogClasses(hInst: HANDLE) -> BOOL {
    let mut szMessage: [WCHAR; 256] = [0; 256];
    let mut szTitle: [WCHAR; 25] = [0; 25];

    LoadStringW(
        hInst as HINSTANCE,
        IDS_TEXT as UINT,
        szMessage.as_mut_ptr() as LPWSTR,
        _countof!(szMessage) as ::core::ffi::c_int,
    );
    LoadStringW(
        hInst as HINSTANCE,
        IDS_DESCRIPTION as UINT,
        szTitle.as_mut_ptr() as LPWSTR,
        _countof!(szTitle) as ::core::ffi::c_int,
    );

    MessageBoxW(
        core::ptr::null_mut(),
        szMessage.as_mut_ptr() as LPCWSTR,
        szTitle.as_mut_ptr() as LPCWSTR,
        (MB_OK | MB_ICONEXCLAMATION) as UINT,
    );

    return (FALSE) as BOOL;
}

// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_CD8397DE2CFB: [u16; 6] = [0x0041u16, 0x0072, 0x0069, 0x0061, 0x006C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NT5_TOP_BORDER_HEIGHT: i32 = 80;

pub const NT5_BOTTOM_BORDER_HEIGHT: i32 = 96;

unsafe extern "C" fn NT5_DrawLogoffCaptionText(mut lpText: LPWSTR, mut hdcMem: HDC) {
    let mut hFont: HFONT = core::ptr::null_mut::<HFONT__>();
    let mut LogFont: LOGFONTW = core::mem::zeroed::<LOGFONTW>();
    let mut TextRect: RECT = core::mem::zeroed::<RECT>();
    let mut PrevBkMode: INT = 0;

    /* Setup the font we'll use */
    ZeroMemory!(&raw mut LogFont as *mut core::ffi::c_void, core::mem::size_of::<LOGFONTW>() as usize);
    LogFont.lfCharSet = DEFAULT_CHARSET as BYTE;
    LogFont.lfHeight = 22 as LONG;
    LogFont.lfWeight = 109 as LONG; // From WinXP disassembly
    StringCchCopyW(
        &raw mut LogFont.lfFaceName as *mut u16,
        _countof!(LogFont.lfFaceName) as usize,
        RUSTCONV_WSTR_CD8397DE2CFB.as_ptr(),
    );

    /* Create it */
    hFont = CreateFontIndirectW(&raw mut LogFont);
    if !hFont.is_null() {
        /* Set the font and font colour */
        SelectObject(hdcMem, hFont as HGDIOBJ);
        SetTextColor(
            hdcMem,
            RGB!(255, 255, 255),
        );

        /* Create the text rect */
        TextRect.top = (((*g_pInfo).cy as core::ffi::c_int / 2) + 34) as LONG;
        TextRect.bottom = (((*g_pInfo).cy as core::ffi::c_int / 2) + 34
            + (GetDeviceCaps(hdcMem, LOGPIXELSY as core::ffi::c_int) as core::ffi::c_int)) as LONG;
        TextRect.left = (((*g_pInfo).cx as core::ffi::c_int) / 3) as LONG;
        TextRect.right = ((((*g_pInfo).cx as core::ffi::c_int) / 2) + 35 + 137) as LONG;

        /* Set the background mode to transparent */
        PrevBkMode = SetBkMode(hdcMem, TRANSPARENT as core::ffi::c_int) as INT;

        /* Draw the text to the mem DC */
        DrawTextW(
            hdcMem,
            lpText as LPCWSTR,
            -1,
            &raw mut TextRect,
            (DT_NOPREFIX | DT_WORDBREAK | DT_RIGHT) as UINT,
        ); // WinXP disassembly uses 0x812

        /* Set the previous background mode */
        SetBkMode(hdcMem, PrevBkMode);

        /* Delete the font */
        DeleteObject(hFont as HGDIOBJ);
    }
}

unsafe extern "C" fn NT5_DrawLogoffIcon(hdcMem: HDC) {
    let mut hBitmap: HBITMAP = core::ptr::null_mut();
    let mut bitmap: BITMAP = core::mem::zeroed();
    let mut hTempDC: HDC = core::ptr::null_mut();

    /* Load the XP logo */
    hBitmap = LoadImageW(
        (*g_pInfo).hInstance,
        MAKEINTRESOURCEW!(IDB_MAIN_ROS_LOGO),
        IMAGE_BITMAP,
        0,
        0,
        LR_DEFAULTCOLOR,
    ) as HBITMAP;
    if !hBitmap.is_null() {
        /* Get the bitmap dimensions */
        GetObjectW(
            hBitmap as HANDLE,
            core::mem::size_of::<BITMAP>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(bitmap) as LPVOID,
        );

        /* Create a temp DC for the bitmap */
        hTempDC = CreateCompatibleDC(hdcMem);
        if !hTempDC.is_null() {
            /* Select the bitmap onto the temp DC */
            SelectObject(hTempDC, hBitmap as HGDIOBJ);

            /* Paint it onto the centre block */
            BitBlt(
                hdcMem,
                ((*g_pInfo).cx / 2) + 35,
                ((*g_pInfo).cy / 2) - 72,
                bitmap.bmWidth,
                bitmap.bmHeight,
                hTempDC,
                0,
                0,
                SRCCOPY,
            );

            /* Delete the DC */
            DeleteDC(hTempDC);
        }

        /* Delete the bitmap */
        DeleteObject(hBitmap as HGDIOBJ);
    }
}

#[no_mangle]
pub unsafe extern "C" fn NT5_RefreshLogoffScreenText(lpText: LPWSTR, hdcMem: HDC) {
    NT5_DrawLogoffCaptionText(lpText, hdcMem);
}
#[no_mangle]
pub unsafe extern "C" fn NT5_CreateLogoffScreen(lpText: LPWSTR, hdcMem: HDC) {
    /* Draw the reactos logo */
    NT5_DrawLogoffIcon(hdcMem);

    /* Draw the first text string */
    NT5_DrawLogoffCaptionText(lpText, hdcMem);
}
#[no_mangle]
pub unsafe extern "C" fn NT5_DrawBaseBackground(hdcDesktop: HDC) -> HDC {
    let mut hBitmap: HBITMAP = core::ptr::null_mut();
    let mut hdcMem: HDC = core::ptr::null_mut();

    /* Create an an off screen DC to match the desktop DC */
    hdcMem = CreateCompatibleDC(hdcDesktop);
    if !hdcMem.is_null() {
        /* Create a bitmap to draw the logoff screen onto */
        hBitmap = CreateCompatibleBitmap(hdcDesktop, (*g_pInfo).cx, (*g_pInfo).cy);
        if !hBitmap.is_null() {
            /* Select it onto our off screen DC*/
            SelectObject(hdcMem, hBitmap as HGDIOBJ);

            /* Draw the centre block */
            {
                let mut hTempBitmap: HBITMAP = core::ptr::null_mut();
                let mut hBrush: HBRUSH = core::ptr::null_mut();
                let mut bitmap: BITMAP = core::mem::zeroed();
                let mut hTempDC: HDC = core::ptr::null_mut();

                /* Paint the blue centre block */
                hBrush = CreateSolidBrush(RGB!(
                    90,
                    126,
                    220
                ));
                SelectObject(hdcMem, hBrush as HGDIOBJ);
                PatBlt(
                    hdcMem,
                    0,
                    NT5_TOP_BORDER_HEIGHT,
                    (*g_pInfo).cx,
                    (*g_pInfo).cy - NT5_TOP_BORDER_HEIGHT - NT5_BOTTOM_BORDER_HEIGHT,
                    PATCOPY
                );
                DeleteObject(hBrush as HGDIOBJ);

                /* Load the shine effect */
                hTempBitmap = LoadImageW(
                    (*g_pInfo).hInstance,
                    MAKEINTRESOURCEW!(IDB_MAIN_PANEL_SHINE),
                    IMAGE_BITMAP,
                    0,
                    0,
                    LR_DEFAULTCOLOR
                ) as HBITMAP;
                if !hTempBitmap.is_null() {
                    /* Get the bitmap dimensions */
                    GetObjectW(
                        hTempBitmap as HANDLE,
                        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
                        core::ptr::addr_of_mut!(bitmap) as LPVOID
                    );

                    /* Create a temp DC for the bitmap */
                    hTempDC = CreateCompatibleDC(hdcDesktop);
                    if !hTempDC.is_null() {
                        /* Select the bitmap onto the temp DC */
                        SelectObject(hTempDC, hTempBitmap as HGDIOBJ);

                        /* Paint it onto the top left of the centre block */
                        BitBlt(
                            hdcMem,
                            0,
                            NT5_TOP_BORDER_HEIGHT,
                            bitmap.bmWidth,
                            bitmap.bmHeight,
                            hTempDC,
                            0,
                            0,
                            SRCCOPY
                        );

                        /* Delete the DC */
                        DeleteDC(hTempDC);
                    }

                    /* Delete the bitmap */
                    DeleteObject(hTempBitmap as HGDIOBJ);
                }
            }

            /* Draw the top border */
            {
                let mut hTempBitmap: HBITMAP = core::ptr::null_mut();
                let mut hBrush: HBRUSH = core::ptr::null_mut();
                let mut bitmap: BITMAP = core::mem::zeroed();
                let mut hTempDC: HDC = core::ptr::null_mut();

                /* Create the blue brush and paint the top bar */
                hBrush = CreateSolidBrush(RGB!(
                    0,
                    48,
                    156
                ));
                SelectObject(hdcMem, hBrush as HGDIOBJ);
                PatBlt(
                    hdcMem,
                    0,
                    0,
                    (*g_pInfo).cx,
                    NT5_TOP_BORDER_HEIGHT,
                    PATCOPY
                );
                DeleteObject(hBrush as HGDIOBJ);

                /* Load the top divider strip */
                hTempBitmap = LoadImageW(
                    (*g_pInfo).hInstance,
                    MAKEINTRESOURCEW!(IDB_TOP_DIVIDER_STRIP),
                    IMAGE_BITMAP,
                    0,
                    0,
                    LR_DEFAULTCOLOR
                ) as HBITMAP;
                if !hTempBitmap.is_null() {
                    /* Get the bitmap dimensions */
                    GetObjectW(
                        hTempBitmap as HANDLE,
                        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
                        core::ptr::addr_of_mut!(bitmap) as LPVOID
                    );

                    /* Create a temp DC for the bitmap */
                    hTempDC = CreateCompatibleDC(hdcDesktop);
                    if !hTempDC.is_null() {
                        /* Select the bitmap onto the temp DC */
                        SelectObject(hTempDC, hTempBitmap as HGDIOBJ);

                        /* Paint the bitmap */
                        StretchBlt(
                            hdcMem,
                            0,
                            NT5_TOP_BORDER_HEIGHT - bitmap.bmHeight,
                            (*g_pInfo).cx,
                            NT5_TOP_BORDER_HEIGHT,
                            hTempDC,
                            0,
                            0,
                            bitmap.bmWidth,
                            NT5_TOP_BORDER_HEIGHT,
                            SRCCOPY
                        );

                        /* Delete the DC */
                        DeleteDC(hTempDC);
                    }

                    /* Delete the bitmap */
                    DeleteObject(hTempBitmap as HGDIOBJ);
                }
            }

            /* Draw the bottom border */
            {
                let mut hTempBitmap: HBITMAP = core::ptr::null_mut();
                let mut vertex: [TRIVERTEX; 2] = core::mem::zeroed();
                let mut gRect: GRADIENT_RECT = core::mem::zeroed();
                let mut bitmap: BITMAP = core::mem::zeroed();
                let mut hTempDC: HDC = core::ptr::null_mut();

                /*
                 * We paint the divider strip first as it's 3
                 * pixels high but MS only show 2 of them.
                 */

                /* Load the bottom divider strip */
                hTempBitmap = LoadImage(
                    (*g_pInfo).hInstance,
                    MAKEINTRESOURCE!(IDB_BOTTOM_DIVIDER_STRIP),
                    IMAGE_BITMAP,
                    0,
                    0,
                    LR_DEFAULTCOLOR
                ) as HBITMAP;
                if !hTempBitmap.is_null() {
                    /* Get the bitmap dimensions */
                    GetObjectW(
                        hTempBitmap as HANDLE,
                        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
                        core::ptr::addr_of_mut!(bitmap) as LPVOID
                    );

                    /* Create a temp DC for the bitmap */
                    hTempDC = CreateCompatibleDC(hdcDesktop);
                    if !hTempDC.is_null() {
                        /* Select the bitmap onto the temp DC */
                        SelectObject(hTempDC, hTempBitmap as HGDIOBJ);

                        /* Paint the bitmap */
                        StretchBlt(
                            hdcMem,
                            0,
                            (*g_pInfo).cy - NT5_BOTTOM_BORDER_HEIGHT,
                            (*g_pInfo).cx,
                            (*g_pInfo).cy - NT5_BOTTOM_BORDER_HEIGHT + bitmap.bmHeight,
                            hTempDC,
                            0,
                            0,
                            bitmap.bmWidth,
                            (*g_pInfo).cy - NT5_BOTTOM_BORDER_HEIGHT + bitmap.bmHeight,
                            SRCCOPY
                        );

                        /* Delete the DC */
                        DeleteDC(hTempDC);
                    }

                    /* Delete the bitmap */
                    DeleteObject(hTempBitmap as HGDIOBJ);
                }

                /* Setup the left hand vertex */
                vertex[0 as usize].x = 0 as LONG;
                vertex[0 as usize].y = (*g_pInfo).cy - NT5_BOTTOM_BORDER_HEIGHT + 2; // paint over 1 pixel of the bitmap
                vertex[0 as usize].Red = 0x3900 as COLOR16;
                vertex[0 as usize].Green = 0x3400 as COLOR16;
                vertex[0 as usize].Blue = 0xAE00 as COLOR16;
                vertex[0 as usize].Alpha = 0x0000 as COLOR16;

                /* Setup the right hand vertex */
                vertex[1 as usize].x = (*g_pInfo).cx as LONG;
                vertex[1 as usize].y = (*g_pInfo).cy as LONG;
                vertex[1 as usize].Red = 0x0000 as COLOR16;
                vertex[1 as usize].Green = 0x3000 as COLOR16;
                vertex[1 as usize].Blue = 0x9600 as COLOR16;
                vertex[1 as usize].Alpha = 0x0000 as COLOR16;

                /* Set the vertex structs */
                gRect.UpperLeft = 0 as ULONG;
                gRect.LowerRight = 1 as ULONG;

                /* Paint the gradient across the bottom */
                GradientFill(
                    hdcMem,
                    core::ptr::addr_of_mut!(vertex) as PTRIVERTEX,
                    2 as ULONG,
                    core::ptr::addr_of_mut!(gRect) as PVOID,
                    1 as ULONG,
                    GRADIENT_FILL_RECT_H as ULONG
                );
            }

            /* Delete the bitmap */
            DeleteObject(hBitmap as HGDIOBJ);
        }
    }

    return hdcMem;
}

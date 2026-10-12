use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_CardBitmaps: [HBITMAP; 68] =
    [::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__; 68];

#[no_mangle]
pub static mut g_hModule: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
// The C names the parameter `Unknown`, which the island also defines as a `_MEDIA_TYPE`
// constant: as a Rust parameter it is a refutable pattern.
pub unsafe extern "system" fn WEP(Unknown_: DWORD) -> BOOL {
    UNREFERENCED_PARAMETER!(Unknown_);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn cdtInit(Width: *mut INT, Height: *mut INT) -> BOOL {
    let mut dwIndex: DWORD;

    /* Report card width and height to user */
    *Width = (CARD_WIDTH as i32);
    *Height = (CARD_HEIGHT as i32);

    /* Load images */
    dwIndex = 0;
    while dwIndex < MAX_CARD_BITMAPS {
        g_CardBitmaps[dwIndex as usize] =
            LoadBitmapA(g_hModule, MAKEINTRESOURCEA!(dwIndex + 1)) as HBITMAP;
        dwIndex += 1;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn cdtTerm() {
    let mut dwIndex: DWORD;
    dwIndex = 0 as DWORD;
    while dwIndex < MAX_CARD_BITMAPS as DWORD {
        DeleteObject(g_CardBitmaps[dwIndex as usize] as HGDIOBJ);
        dwIndex = dwIndex.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn cdtDraw(hdc: HDC, x: INT, y: INT, card: INT, type_: INT, color: COLORREF) -> BOOL {
    return cdtDrawExt(hdc, x, y, (CARD_WIDTH) as INT, (CARD_HEIGHT) as INT, card, type_, color);
}

unsafe extern "C" fn BltCard(hdc: HDC, x: INT, y: INT, dx: INT, dy: INT, hdcCard: HDC, dwRasterOp: DWORD, bStretch: BOOL) {
    if bStretch != 0 {
        StretchBlt(hdc, x, y, dx, dy, hdcCard, 0, 0, (CARD_WIDTH as i32), (CARD_HEIGHT as i32), dwRasterOp);
    } else {
        BitBlt(hdc, x, y, dx, dy, hdcCard, 0, 0, dwRasterOp);
        /*
         * This is need when using Microsoft images, because they use two-color red/white images for
         * red cards and thus needs fix-up of the edge to black color.
         */
    }
}

#[no_mangle]
pub unsafe extern "system" fn cdtDrawExt(
    mut hdc: HDC,
    mut x: INT,
    mut y: INT,
    mut dx: INT,
    mut dy: INT,
    mut card: INT,
    mut type_0: INT,
    mut color: COLORREF,
) -> BOOL {
    let mut dwRasterOp: DWORD = SRCCOPY;
    let mut bSaveEdges: BOOL = (TRUE as i32);
    let mut bStretch: BOOL = (FALSE as i32);

    if type_0 & (ectSAVEEDGESMASK as i32) != 0 {
        type_0 &= (!ectSAVEEDGESMASK) as i32;
        bSaveEdges = (FALSE as i32);
    }

    if dx != (CARD_WIDTH as i32) || dy != (CARD_HEIGHT as i32) {
        bStretch = (TRUE as i32);
        bSaveEdges = (FALSE as i32);
    }

    let mut sel: u8 = 2;
    match (type_0) as u32 {
        ectINVERTED => {
            dwRasterOp = NOTSRCCOPY;
            sel = 1;
        }
        ectFACES => {
            sel = 1;
        }
        ectBACKS => {
            card -= 1;
            sel = 2;
        }
        ectEMPTYNOBG => {
            dwRasterOp = SRCAND;
            sel = 3;
        }
        ectEMPTY => {
            sel = 3;
        }
        ectERASE => {
            sel = 2;
        }
        ectREDX => {
            card = 66;
            sel = 2;
        }
        ectGREENO => {
            card = 67;
            sel = 2;
        }
        _ => {
            return (FALSE) as BOOL;
        }
    }
    match sel {
        1 => {
            card = (card % 4) * 13 + (card / 4);
        }
        3 => {
            card = 52;
        }
        _ => {}
    }

    if type_0 == (ectEMPTY as i32) || type_0 == (ectERASE as i32) {
        let mut pPoint: POINT = POINT { x: 0, y: 0 };
        let mut hBrush: HBRUSH = core::ptr::null_mut();
        let mut hOldBrush: HBRUSH = core::ptr::null_mut();

        hBrush = CreateSolidBrush(color);
        GetDCOrgEx(hdc, core::ptr::addr_of_mut!(pPoint));
        SetBrushOrgEx(hdc, pPoint.x, pPoint.y, core::ptr::null_mut());
        hOldBrush = SelectObject(hdc, hBrush as HGDIOBJ) as HBRUSH;
        PatBlt(hdc, x, y, dx, dy, PATCOPY);
        SelectObject(hdc, hOldBrush as HGDIOBJ);
        DeleteObject(hBrush as HGDIOBJ);
    }
    if type_0 != (ectERASE as i32) {
        let mut hdcCard: HDC = core::ptr::null_mut();
        let mut OldBkColor: COLORREF = 0;

        hdcCard = CreateCompatibleDC(hdc);
        SelectObject(hdcCard, g_CardBitmaps[card as usize] as HGDIOBJ);
        OldBkColor = SetBkColor(
            hdc,
            if type_0 == (ectFACES as i32) {
                0xFFFFFF
            } else {
                color
            },
        );
        if bSaveEdges != 0 {
            let mut SavedPixels: [COLORREF; 12] = [0; 12];
            SavedPixels[0] = GetPixel(hdc, x, y);
            SavedPixels[1] = GetPixel(hdc, x + 1, y);
            SavedPixels[2] = GetPixel(hdc, x, y + 1);
            SavedPixels[3] = GetPixel(hdc, x + dx - 1, y);
            SavedPixels[4] = GetPixel(hdc, x + dx - 2, y);
            SavedPixels[5] = GetPixel(hdc, x + dx - 1, y + 1);
            SavedPixels[6] = GetPixel(hdc, x, y + dy - 1);
            SavedPixels[7] = GetPixel(hdc, x + 1, y + dy - 1);
            SavedPixels[8] = GetPixel(hdc, x, y + dy - 2);
            SavedPixels[9] = GetPixel(hdc, x + dx - 1, y + dy - 1);
            SavedPixels[10] = GetPixel(hdc, x + dx - 2, y + dy - 1);
            SavedPixels[11] = GetPixel(hdc, x + dx - 1, y + dy - 2);

            BltCard(hdc, x, y, dx, dy, hdcCard, dwRasterOp, bStretch);

            SetPixel(hdc, x, y, SavedPixels[0]);
            SetPixel(hdc, x + 1, y, SavedPixels[1]);
            SetPixel(hdc, x, y + 1, SavedPixels[2]);
            SetPixel(hdc, x + dx - 1, y, SavedPixels[3]);
            SetPixel(hdc, x + dx - 2, y, SavedPixels[4]);
            SetPixel(hdc, x + dx - 1, y + 1, SavedPixels[5]);
            SetPixel(hdc, x, y + dy - 1, SavedPixels[6]);
            SetPixel(hdc, x + 1, y + dy - 1, SavedPixels[7]);
            SetPixel(hdc, x, y + dy - 2, SavedPixels[8]);
            SetPixel(hdc, x + dx - 1, y + dy - 1, SavedPixels[9]);
            SetPixel(hdc, x + dx - 2, y + dy - 1, SavedPixels[10]);
            SetPixel(hdc, x + dx - 1, y + dy - 2, SavedPixels[11]);
        } else {
            BltCard(hdc, x, y, dx, dy, hdcCard, dwRasterOp, bStretch);
        }
        SetBkColor(hdc, OldBkColor);
        DeleteDC(hdcCard);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn cdtAnimate(hdc: HDC, cardback: ::core::ffi::c_int, x: ::core::ffi::c_int, y: ::core::ffi::c_int, frame: ::core::ffi::c_int) -> BOOL {
    UNREFERENCED_PARAMETER!(frame);
    UNREFERENCED_PARAMETER!(y);
    UNREFERENCED_PARAMETER!(x);
    UNREFERENCED_PARAMETER!(cardback);
    UNREFERENCED_PARAMETER!(hdc);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    UNREFERENCED_PARAMETER!(lpvReserved);

    if fdwReason == DLL_PROCESS_ATTACH {
        g_hModule = hinstDLL;
    }

    return (TRUE) as BOOL;
}

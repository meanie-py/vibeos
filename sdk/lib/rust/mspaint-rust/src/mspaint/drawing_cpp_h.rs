use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn CreateGeometricPen(rgbColor: COLORREF, thickness: INT) -> HPEN {
    let mut logbrush: LOGBRUSH = core::mem::zeroed();
    logbrush.lbStyle = BS_SOLID;
    logbrush.lbColor = rgbColor;
    logbrush.lbHatch = 0;
    return ExtCreatePen(PS_GEOMETRIC | PS_SOLID | PS_ENDCAP_ROUND | PS_JOIN_ROUND, thickness as u32, core::ptr::addr_of_mut!(logbrush), 0, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn Line(hdc: HDC, x1: i32, y1: i32, x2: i32, y2: i32, color: COLORREF, thickness: i32) {
    let oldPen: HPEN = SelectObject(hdc, (CreatePen((PS_SOLID as i32), thickness, color)).cast()) as HPEN;
    MoveToEx(hdc, x1, y1, core::ptr::null_mut());
    LineTo(hdc, x2, y2);
    SetPixelV(hdc, x2, y2, color);
    DeleteObject(SelectObject(hdc, oldPen as HGDIOBJ));
}
#[no_mangle]
pub unsafe extern "C" fn Rect(hdc: HDC, x1: LONG, y1: LONG, x2: LONG, y2: LONG, fg: COLORREF, bg: COLORREF, thickness: i32, style: i32) {
    let mut oldBrush: HBRUSH;
    let mut logbrush: LOGBRUSH = core::mem::zeroed();
    let oldPen: HPEN = SelectObject(hdc, CreatePen((PS_SOLID as i32), thickness, fg) as HGDIOBJ) as HPEN;
    logbrush.lbStyle = if style == 0 { BS_HOLLOW } else { BS_SOLID };
    logbrush.lbColor = if style == 2 { fg } else { bg };
    logbrush.lbHatch = 0;
    oldBrush = SelectObject(hdc, CreateBrushIndirect(core::ptr::addr_of_mut!(logbrush)) as HGDIOBJ) as HBRUSH;
    Rectangle(hdc, x1, y1, x2, y2);
    DeleteObject(SelectObject(hdc, oldBrush as HGDIOBJ));
    DeleteObject(SelectObject(hdc, oldPen as HGDIOBJ));
}
#[no_mangle]
pub unsafe extern "C" fn Ellp(hdc: HDC, x1: LONG, y1: LONG, x2: LONG, y2: LONG, fg: COLORREF, bg: COLORREF, thickness: i32, style: i32) {
    let oldPen: HPEN = SelectObject(hdc, (CreatePen((PS_SOLID as i32), thickness, fg)).cast()) as HPEN;
    let mut logbrush: LOGBRUSH = core::mem::zeroed();
    logbrush.lbStyle = if style == 0 { BS_HOLLOW } else { BS_SOLID };
    logbrush.lbColor = if style == 2 { fg } else { bg };
    logbrush.lbHatch = 0;
    let oldBrush: HBRUSH = SelectObject(hdc, (CreateBrushIndirect(core::ptr::addr_of_mut!(logbrush))).cast()) as HBRUSH;
    Ellipse(hdc, x1, y1, x2, y2);
    DeleteObject(SelectObject(hdc, (oldBrush).cast()));
    DeleteObject(SelectObject(hdc, (oldPen).cast()));
}
#[no_mangle]
pub unsafe extern "C" fn RRect(hdc: HDC, x1: LONG, y1: LONG, x2: LONG, y2: LONG, fg: COLORREF, bg: COLORREF, thickness: i32, style: i32) {
    let mut logbrush: LOGBRUSH = core::mem::zeroed();
    let mut oldBrush: HBRUSH;
    let oldPen: HPEN = SelectObject(hdc, (CreatePen((PS_SOLID as i32), thickness, fg)).cast()) as HPEN;
    logbrush.lbStyle = if style == 0 { BS_HOLLOW } else { BS_SOLID };
    logbrush.lbColor = if style == 2 { fg } else { bg };
    logbrush.lbHatch = 0;
    oldBrush = SelectObject(hdc, (CreateBrushIndirect(core::ptr::addr_of_mut!(logbrush))).cast()) as HBRUSH;
    RoundRect(hdc, x1, y1, x2, y2, 16, 16);
    DeleteObject(SelectObject(hdc, oldBrush as _) as HGDIOBJ);
    DeleteObject(SelectObject(hdc, oldPen as _) as HGDIOBJ);
}
#[no_mangle]
pub unsafe extern "C" fn Poly(hdc: HDC, lpPoints: *mut POINT, nCount: i32, fg: COLORREF, bg: COLORREF, thickness: i32, style: i32, closed: BOOL, inverted: BOOL) {
    let mut logbrush: LOGBRUSH = core::mem::zeroed();
    let mut oldBrush: HBRUSH = core::ptr::null_mut();
    let oldPen: HPEN = SelectObject(hdc, (CreatePen((PS_SOLID as i32), thickness, fg)).cast()) as HPEN;
    let oldRop: UINT = (GetROP2(hdc)) as u32;

    if inverted != 0 {
        SetROP2(hdc, (R2_NOTXORPEN as i32));
    }

    logbrush.lbStyle = if style == 0 { BS_HOLLOW } else { BS_SOLID };
    logbrush.lbColor = if style == 2 { fg } else { bg };
    logbrush.lbHatch = 0;
    oldBrush = SelectObject(hdc, (CreateBrushIndirect(core::ptr::addr_of_mut!(logbrush))).cast()) as HBRUSH;
    if closed != 0 {
        Polygon(hdc, lpPoints, nCount);
    } else {
        Polyline(hdc, lpPoints, nCount);
    }
    DeleteObject(SelectObject(hdc, (oldBrush).cast()));
    DeleteObject(SelectObject(hdc, (oldPen).cast()));

    SetROP2(hdc, (oldRop) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Bezier(hdc: HDC, p1: POINT, p2: POINT, p3: POINT, p4: POINT, color: COLORREF, thickness: i32) {
    let mut oldPen: HPEN;
    let mut fourPoints: [POINT; 4] = core::mem::zeroed();
    fourPoints[0] = p1;
    fourPoints[1] = p2;
    fourPoints[2] = p3;
    fourPoints[3] = p4;
    oldPen = SelectObject(hdc, CreatePen((PS_SOLID as i32), thickness, color) as HGDIOBJ) as HPEN;
    PolyBezier(hdc, fourPoints.as_ptr(), 4);
    DeleteObject(SelectObject(hdc, oldPen as HGDIOBJ) as HGDIOBJ);
}
#[no_mangle]
pub unsafe extern "C" fn Fill(hdc: HDC, x: i32, y: i32, color: COLORREF) {
    let hBrush: HBRUSH = CreateSolidBrush(color);
    Fill(hdc, x, y, hBrush);
    DeleteObject((hBrush).cast());
}
#[no_mangle]
pub unsafe extern "C" fn Erase(hdc: HDC, x1: LONG, y1: LONG, x2: LONG, y2: LONG, color: COLORREF, radius: LONG) {
    let hBrush: HBRUSH = CreateSolidBrush(color);
    Erase(hdc, x1, y1, x2, y2, hBrush, radius);
    DeleteObject((hBrush).cast());
}
#[no_mangle]
pub unsafe extern "C" fn Replace(hdc: HDC, x1: LONG, y1: LONG, x2: LONG, y2: LONG, fg: COLORREF, bg: COLORREF, radius: LONG) {
    let mut b: LONG = max(1, max(labs(x2 - x1), labs(y2 - y1)));

    let mut a: LONG = 0;
    while a <= b {
        let cx: LONG = (x1 * (b - a) + x2 * a) / b;
        let cy: LONG = (y1 * (b - a) + y2 * a) / b;
        let rc: RECT = RECT { left: cx - radius, top: cy - radius, right: cx + radius, bottom: cy + radius };
        let mut y: LONG = rc.top;
        while y < rc.bottom {
            let mut x: LONG = rc.left;
            while x < rc.right {
                if GetPixel(hdc, x, y) == fg {
                    SetPixelV(hdc, x, y, bg);
                }
                x += 1;
            }
            y += 1;
        }
        a += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Airbrush(hdc: HDC, x: LONG, y: LONG, color: COLORREF, r: LONG) {
    let mut dy: LONG = -r;
    while dy <= r {
        let mut dx: LONG = -r;
        while dx <= r {
            if (dx * dx + dy * dy <= r * r) && (rand() % r == 0) {
                SetPixelV(hdc, x + dx, y + dy, color);
            }
            dx += 1;
        }
        dy += 1;
    }
}

unsafe extern "C" fn BrushInternal(hdc: HDC, x1: LONG, y1: LONG, mut x2: LONG, y2: LONG, style: LONG, thickness: INT) {
    let mut a: LONG;
    let mut b: LONG = max(1, max(labs(x2 - x1), labs(y2 - y1)));
    match style {
        BrushStyleRound => {
            a = 0;
            while a <= b {
                Ellipse(hdc,
                        (x1 * (b - a) + x2 * a) / b - (thickness / 2),
                        (y1 * (b - a) + y2 * a) / b - (thickness / 2),
                        (x1 * (b - a) + x2 * a) / b + (thickness / 2) + 1,
                        (y1 * (b - a) + y2 * a) / b + (thickness / 2) + 1);
                a += 1;
            }
        }

        BrushStyleSquare => {
            a = 0;
            while a <= b {
                Rectangle(hdc,
                          (x1 * (b - a) + x2 * a) / b - (thickness / 2),
                          (y1 * (b - a) + y2 * a) / b - (thickness / 2),
                          (x1 * (b - a) + x2 * a) / b + (thickness / 2) + 1,
                          (y1 * (b - a) + y2 * a) / b + (thickness / 2) + 1);
                a += 1;
            }
        }

        BrushStyleForeSlash | BrushStyleBackSlash => {
            let mut offsetTop: POINT;
            let mut offsetBottom: POINT;
            if style == BrushStyleForeSlash {
                offsetTop = POINT { x: (thickness - 1) / 2 + 1, y: -(thickness - 1) / 2 };
                offsetBottom = POINT { x: -thickness / 2, y: thickness / 2 + 1 };
            } else {
                offsetTop = POINT { x: -thickness / 2, y: -thickness / 2 };
                offsetBottom = POINT { x: (thickness - 1) / 2 + 1, y: (thickness - 1) / 2 + 1 };
            }
            if x1 == x2 && y1 == y2 {
                x2 += 1;
            }
            let points: [POINT; 4] = [
                POINT { x: x1 + offsetTop.x, y: y1 + offsetTop.y },
                POINT { x: x1 + offsetBottom.x, y: y1 + offsetBottom.y },
                POINT { x: x2 + offsetBottom.x, y: y2 + offsetBottom.y },
                POINT { x: x2 + offsetTop.x, y: y2 + offsetTop.y },
            ];
            Polygon(hdc, points.as_ptr(), _countof!(points) as INT);
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn Brush(hdc: HDC, x1: i32, y1: i32, x2: i32, y2: i32, color: COLORREF, style: i32, thickness: i32) {
    let hBrush: HBRUSH = CreateSolidBrush(color);
    Brush(hdc, x1, y1, x2, y2, hBrush, style, thickness);
    DeleteObject((hBrush).cast());
}
#[no_mangle]
pub unsafe extern "C" fn RectSel(hdc: HDC, x1: LONG, y1: LONG, x2: LONG, y2: LONG) {
    let mut oldBrush: HBRUSH;
    let mut logbrush: LOGBRUSH = core::mem::zeroed();
    let oldPen: HPEN = SelectObject(hdc, (CreatePen((PS_DOT as i32), 1, GetSysColor((COLOR_HIGHLIGHT) as i32))).cast()) as HPEN;
    let oldRop: UINT = (GetROP2(hdc)) as u32;

    SetROP2(hdc, (R2_NOTXORPEN as i32));

    logbrush.lbStyle = BS_HOLLOW;
    logbrush.lbColor = 0;
    logbrush.lbHatch = 0;
    oldBrush = SelectObject(hdc, (CreateBrushIndirect(core::ptr::addr_of_mut!(logbrush))).cast()) as HBRUSH;
    Rectangle(hdc, x1, y1, x2, y2);
    DeleteObject(SelectObject(hdc, oldBrush as HGDIOBJ));
    DeleteObject(SelectObject(hdc, oldPen as HGDIOBJ));

    SetROP2(hdc, (oldRop) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Text(hdc: HDC, x1: LONG, y1: LONG, x2: LONG, y2: LONG, fg: COLORREF, bg: COLORREF, lpchText: LPCWSTR, font: HFONT, style: LONG) {
    let iSaveDC: INT = SaveDC(hdc); // We will modify the clipping region. Save now.

    let mut rc: CRect = CRect__make_1(x1, y1, x2, y2);

    if style == 0 // Transparent
    {
        SetBkMode(hdc, (TRANSPARENT as i32));
    }
    else // Opaque
    {
        SetBkMode(hdc, (OPAQUE as i32));
        SetBkColor(hdc, bg);

        let hbr: HBRUSH = CreateSolidBrush(bg);
        FillRect(hdc, core::ptr::addr_of!(rc.__base0), hbr); // Fill the background
        DeleteObject((hbr).cast());
    }

    IntersectClipRect(hdc, rc.__base0.left, rc.__base0.top, rc.__base0.right, rc.__base0.bottom);

    let hFontOld: HGDIOBJ = SelectObject(hdc, (font).cast());
    SetTextColor(hdc, fg);
    let uFormat: UINT = DT_LEFT | DT_TOP | DT_EDITCONTROL | DT_NOPREFIX | DT_NOCLIP |
                         DT_EXPANDTABS | DT_WORDBREAK;
    DrawTextW(hdc, lpchText, -1, core::ptr::addr_of_mut!(rc.__base0), uFormat);
    SelectObject(hdc, hFontOld);

    RestoreDC(hdc, iSaveDC); // Restore
}
#[no_mangle]
pub unsafe extern "C" fn ColorKeyedMaskBlt(hdcDest: HDC, nXDest: i32, nYDest: i32, nWidth: i32, nHeight: i32, hdcSrc: HDC, nXSrc: i32, nYSrc: i32, nSrcWidth: i32, nSrcHeight: i32, hbmMask: HBITMAP, keyColor: COLORREF) -> BOOL {
    let mut hTempDC1: HDC = core::mem::zeroed();
    let mut hTempDC2: HDC = core::mem::zeroed();
    let mut hbmTempColor: HBITMAP = core::mem::zeroed();
    let mut hbmTempMask: HBITMAP = core::mem::zeroed();
    let mut hbmOld1: HGDIOBJ = core::mem::zeroed();
    let mut hbmOld2: HGDIOBJ = core::mem::zeroed();

    if hbmMask == core::mem::zeroed() {
        if keyColor == CLR_INVALID {
            StretchBlt(hdcDest, nXDest, nYDest, nWidth, nHeight,
                         hdcSrc, nXSrc, nYSrc, nSrcWidth, nSrcHeight, SRCCOPY);
        } else {
            GdiTransparentBlt(hdcDest, nXDest, nYDest, nWidth, nHeight,
                                hdcSrc, nXSrc, nYSrc, nSrcWidth, nSrcHeight, keyColor);
        }
        return (TRUE) as BOOL;
    } else if nWidth == nSrcWidth && nHeight == nSrcHeight && keyColor == CLR_INVALID {
        MaskBlt(hdcDest, nXDest, nYDest, nWidth, nHeight,
                  hdcSrc, nXSrc, nYSrc, hbmMask, 0, 0, MAKEROP4!(SRCCOPY, 0xAA0029));
        return (TRUE) as BOOL;
    }

    hTempDC1 = CreateCompatibleDC(hdcDest);
    hTempDC2 = CreateCompatibleDC(hdcDest);
    hbmTempMask = CreateBitmap(nWidth, nHeight, 1, 1, core::ptr::null_mut());
    hbmTempColor = CreateColorDIB(nWidth, nHeight, RGB!(255, 255, 255));

    // hbmTempMask <-- hbmMask (stretched)
    hbmOld1 = SelectObject(hTempDC1, (hbmMask).cast());
    hbmOld2 = SelectObject(hTempDC2, (hbmTempMask).cast());
    StretchBlt(hTempDC2, 0, 0, nWidth, nHeight, hTempDC1, 0, 0, nSrcWidth, nSrcHeight, SRCCOPY);
    SelectObject(hTempDC2, hbmOld2);
    SelectObject(hTempDC1, hbmOld1);

    hbmOld1 = SelectObject(hTempDC1, (hbmTempColor).cast());
    if keyColor == CLR_INVALID {
        // hbmTempColor <-- hdcSrc (stretched)
        StretchBlt(hTempDC1, 0, 0, nWidth, nHeight,
                     hdcSrc, nXSrc, nYSrc, nSrcWidth, nSrcHeight, SRCCOPY);

        // hdcDest <-- hbmTempColor (masked)
        MaskBlt(hdcDest, nXDest, nYDest, nWidth, nHeight, hTempDC1, 0, 0,
                  hbmTempMask, 0, 0, MAKEROP4!(SRCCOPY, 0xAA0029));
    } else {
        // hbmTempColor <-- hdcDest
        BitBlt(hTempDC1, 0, 0, nWidth, nHeight, hdcDest, nXDest, nYDest, SRCCOPY);

        // hbmTempColor <-- hdcSrc (color key)
        GdiTransparentBlt(hTempDC1, 0, 0, nWidth, nHeight,
                            hdcSrc, nXSrc, nYSrc, nSrcWidth, nSrcHeight, keyColor);

        // hdcDest <-- hbmTempColor (masked)
        MaskBlt(hdcDest, nXDest, nYDest, nWidth, nHeight, hTempDC1, 0, 0,
                  hbmTempMask, 0, 0, MAKEROP4!(SRCCOPY, 0xAA0029));
    }
    SelectObject(hTempDC1, hbmOld1);

    DeleteObject((hbmTempColor).cast());
    DeleteObject((hbmTempMask).cast());
    DeleteDC(hTempDC2);
    DeleteDC(hTempDC1);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DrawXorRect(hdc: HDC, prc: *const RECT) {
    let oldPen: HGDIOBJ = SelectObject(hdc, (CreatePen((PS_SOLID as i32), 0, RGB!(0, 0, 0))).cast());
    let oldBrush: HGDIOBJ = SelectObject(hdc, GetStockObject((NULL_BRUSH as i32)));
    let oldRop2: i32 = SetROP2(hdc, (R2_NOTXORPEN as i32));
    Rectangle(hdc, (*prc).left, (*prc).top, (*prc).right, (*prc).bottom);
    SetROP2(hdc, oldRop2);
    SelectObject(hdc, oldBrush);
    DeleteObject(SelectObject(hdc, oldPen));
}
#[no_mangle]
pub unsafe extern "C" fn CreateDitherBrush(color: COLORREF, monoColor0: COLORREF, monoColor1: COLORREF) -> HBRUSH {
    // 8x8 Bayer ordered dithering matrix (0 to 63)
    static S_BAYER_MATRIX: [[u8; 8]; 8] =
    [
        [ 0, 32, 8, 40, 2, 34, 10, 42 ],
        [ 48, 16, 56, 24, 50, 18, 58, 26 ],
        [ 12, 44, 4, 36, 14, 46, 6, 38 ],
        [ 60, 28, 52, 20, 62, 30, 54, 22 ],
        [ 3, 35, 11, 43, 1, 33, 9, 41 ],
        [ 51, 19, 59, 27, 49, 17, 57, 25 ],
        [ 15, 47, 7, 39, 13, 45, 5, 37 ],
        [ 63, 31, 55, 23, 61, 29, 53, 21 ],
    ];
    let sum: i32 = (GetRValue!(color) as i32) + (GetGValue!(color) as i32) + (GetBValue!(color) as i32);
    let mut brightness: i32 = sum / 3;
    if brightness < 0 {
        brightness = 0;
    }
    if brightness >= 255 {
        brightness = 256; // White out
    }

    let mut bmi: BITMAPINFO = core::mem::zeroed();
    bmi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = 8;
    bmi.bmiHeader.biHeight = -8; // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 24;

    let b0: u8 = GetBValue!(monoColor0) as u8;
    let g0: u8 = GetGValue!(monoColor0) as u8;
    let r0: u8 = GetRValue!(monoColor0) as u8;
    let b1: u8 = GetBValue!(monoColor1) as u8;
    let g1: u8 = GetGValue!(monoColor1) as u8;
    let r1: u8 = GetRValue!(monoColor1) as u8;

    let mut pixels: [u8; 8 * 8 * 3] =
        [0u8; 8 * 8 * 3];
    let mut y: i32 = 0;
    while y < 8 {
        let mut index: i32 = y * ((3 * CHAR_BIT)) as i32;
        let mut x: i32 = 0;
        while x < 8 {
            let threshold: i32 = (S_BAYER_MATRIX[y as usize][x as usize] as i32) * 255 / 63;
            if brightness > threshold {
                pixels[index as usize] = b1; // Blue
                index += 1;
                pixels[index as usize] = g1; // Green
                index += 1;
                pixels[index as usize] = r1; // Red
                index += 1;
            } else {
                pixels[index as usize] = b0; // Blue
                index += 1;
                pixels[index as usize] = g0; // Green
                index += 1;
                pixels[index as usize] = r0; // Red
                index += 1;
            }
            x += 1;
        }
        y += 1;
    }

    let hdc: HDC = GetDC(core::ptr::null_mut());
    let hBitmap: HBITMAP = CreateDIBitmap(hdc, core::ptr::addr_of!(bmi.bmiHeader), CBM_INIT, pixels.as_mut_ptr() as *const core::ffi::c_void, core::ptr::addr_of!(bmi), DIB_RGB_COLORS);
    ReleaseDC(core::ptr::null_mut(), hdc);

    if hBitmap.is_null() {
        return core::ptr::null_mut();
    }

    let hBrush: HBRUSH = CreatePatternBrush(hBitmap);
    DeleteObject((hBitmap).cast());

    hBrush
}

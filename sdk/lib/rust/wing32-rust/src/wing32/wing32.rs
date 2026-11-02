use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn WinGCreateDC() -> HDC {
    return CreateCompatibleDC(core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn WinGRecommendDIBFormat(bmi: *mut BITMAPINFO) -> BOOL {
    if bmi.is_null() {
        return (FALSE) as BOOL;
    }

    (*bmi).bmiHeader.biSize = ::core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    (*bmi).bmiHeader.biWidth = 320 as LONG;
    (*bmi).bmiHeader.biHeight = 1 as LONG;
    (*bmi).bmiHeader.biPlanes = 1 as WORD;
    (*bmi).bmiHeader.biBitCount = 8 as WORD;
    (*bmi).bmiHeader.biCompression = BI_RGB as DWORD;
    (*bmi).bmiHeader.biSizeImage = 0 as DWORD;
    (*bmi).bmiHeader.biXPelsPerMeter = 0 as LONG;
    (*bmi).bmiHeader.biYPelsPerMeter = 0 as LONG;
    (*bmi).bmiHeader.biClrUsed = 0 as DWORD;
    (*bmi).bmiHeader.biClrImportant = 0 as DWORD;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WinGCreateBitmap(hdc: HDC, bmi: *mut BITMAPINFO, bits: *mut *mut core::ffi::c_void) -> HBITMAP {
    return CreateDIBSection(hdc, bmi, DIB_RGB_COLORS, bits, core::ptr::null_mut(), 0);
}

#[no_mangle]
pub unsafe extern "system" fn WinGGetDIBPointer(hbmp: HBITMAP, bmi: *mut BITMAPINFO) -> *mut core::ffi::c_void {
    let mut ds: DIBSECTION = core::mem::zeroed();
    if GetObjectW(
        hbmp as HANDLE,
        core::mem::size_of::<DIBSECTION>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(ds) as LPVOID,
    ) as usize == core::mem::size_of::<DIBSECTION>()
    {
        if !bmi.is_null() {
            (*bmi).bmiHeader = ds.dsBmih;
        }
        return ds.dsBm.bmBits as *mut core::ffi::c_void;
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn WinGSetDIBColorTable(
    hdc: HDC,
    start: UINT,
    end: UINT,
    colors: *mut RGBQUAD,
) -> UINT {
    SetDIBColorTable(hdc, start, end, colors)
}

#[no_mangle]
pub unsafe extern "system" fn WinGGetDIBColorTable(
    hdc: HDC,
    start: UINT,
    end: UINT,
    colors: *mut RGBQUAD,
) -> UINT {
    GetDIBColorTable(hdc, start, end, colors)
}

#[no_mangle]
pub unsafe extern "system" fn WinGCreateHalftonePalette() -> HPALETTE {
    let mut hdc: HDC;
    let mut hpal: HPALETTE;

    hdc = GetDC(core::ptr::null_mut());
    hpal = CreateHalftonePalette(hdc);
    ReleaseDC(core::ptr::null_mut(), hdc);

    return hpal;
}

#[no_mangle]
pub unsafe extern "system" fn WinGCreateHalftoneBrush(hdc: HDC, color: COLORREF, type_0: i32) -> HBRUSH {
    return CreateSolidBrush(color);
}

#[no_mangle]
pub unsafe extern "system" fn WinGStretchBlt(
    hdcDst: HDC, xDst: INT, yDst: INT, widthDst: INT, heightDst: INT,
    hdcSrc: HDC, xSrc: INT, ySrc: INT, widthSrc: INT, heightSrc: INT,
) -> BOOL {
    let old_blt_mode: INT;
    let ret: BOOL;
    old_blt_mode = SetStretchBltMode(hdcDst, (COLORONCOLOR as i32));
    ret = StretchBlt(
        hdcDst, xDst, yDst, widthDst, heightDst,
        hdcSrc, xSrc, ySrc, widthSrc, heightSrc, SRCCOPY,
    );
    SetStretchBltMode(hdcDst, old_blt_mode);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn WinGBitBlt(hdcDst: HDC, xDst: INT, yDst: INT, width: INT, height: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT) -> BOOL {
    return BitBlt(hdcDst, xDst, yDst, width, height, hdcSrc, xSrc, ySrc, SRCCOPY as DWORD);
}

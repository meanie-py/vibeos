use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_09C0D8FDD014: [WCHAR; 3] = [0x004F, 0x004B, 0x0000];
pub static CPP_COM_WSTR_1F367FB724B3: [WCHAR; 8] = [0x0026, 0x0049, 0x0067, 0x006E, 0x006F, 0x0072, 0x0065, 0x0000];
pub static CPP_COM_WSTR_229BA167C275: [WCHAR; 2] = [0x0075, 0x0000];
pub static CPP_COM_WSTR_4CA0338A5BD8: [WCHAR; 7] = [0x0043, 0x0061, 0x006E, 0x0063, 0x0065, 0x006C, 0x0000];
pub static CPP_COM_WSTR_61F3B37003C0: [WCHAR; 7] = [0x0026, 0x0052, 0x0065, 0x0074, 0x0072, 0x0079, 0x0000];
pub static CPP_COM_WSTR_672463267A33: [WCHAR; 6] = [0x0052, 0x0045, 0x0042, 0x0041, 0x0052, 0x0000];
pub static CPP_COM_WSTR_77F906B94309: [WCHAR; 2] = [0x0034, 0x0000];
pub static CPP_COM_WSTR_893EDE57DDD9: [WCHAR; 5] = [0x0026, 0x0059, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_91CDBB74B095: [WCHAR; 8] = [0x0054, 0x004F, 0x004F, 0x004C, 0x0042, 0x0041, 0x0052, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A944EB43577D: [WCHAR; 4] = [0x0026, 0x004E, 0x006F, 0x0000];
pub static CPP_COM_WSTR_B728528EE8A8: [WCHAR; 8] = [0x004D, 0x0061, 0x0072, 0x006C, 0x0065, 0x0074, 0x0074, 0x0000];
pub static CPP_COM_WSTR_DDBF4DF411E1: [WCHAR; 7] = [0x0057, 0x0049, 0x004E, 0x0044, 0x004F, 0x0057, 0x0000];
pub static CPP_COM_WSTR_FD9E5FF53298: [WCHAR; 7] = [0x0026, 0x0041, 0x0062, 0x006F, 0x0072, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut CUIFTheme__s_hUXTHEME: HINSTANCE = core::ptr::null_mut();
pub static mut CUIFTheme__s_fnOpenThemeData: FN_OpenThemeData = None;
pub static mut CUIFTheme__s_fnCloseThemeData: FN_CloseThemeData = None;
pub static mut CUIFTheme__s_fnDrawThemeBackground: FN_DrawThemeBackground = None;
pub static mut CUIFTheme__s_fnDrawThemeParentBackground: FN_DrawThemeParentBackground = None;
pub static mut CUIFTheme__s_fnDrawThemeText: FN_DrawThemeText = None;
pub static mut CUIFTheme__s_fnDrawThemeIcon: FN_DrawThemeIcon = None;
pub static mut CUIFTheme__s_fnGetThemeBackgroundExtent: FN_GetThemeBackgroundExtent = None;
pub static mut CUIFTheme__s_fnGetThemeBackgroundContentRect: FN_GetThemeBackgroundContentRect = None;
pub static mut CUIFTheme__s_fnGetThemeTextExtent: FN_GetThemeTextExtent = None;
pub static mut CUIFTheme__s_fnGetThemePartSize: FN_GetThemePartSize = None;
pub static mut CUIFTheme__s_fnDrawThemeEdge: FN_DrawThemeEdge = None;
pub static mut CUIFTheme__s_fnGetThemeColor: FN_GetThemeColor = None;
pub static mut CUIFTheme__s_fnGetThemeMargins: FN_GetThemeMargins = None;
pub static mut CUIFTheme__s_fnGetThemeFont: FN_GetThemeFont = None;
pub static mut CUIFTheme__s_fnGetThemeSysColor: FN_GetThemeSysColor = None;
pub static mut CUIFTheme__s_fnGetThemeSysSize: FN_GetThemeSysSize = None;
pub static mut CUIFSystemInfo__s_pSystemInfo: *mut CUIFSystemInfo = core::ptr::null_mut();
pub static mut CUIFScheme__s_pColorTableSys: *mut CUIFColorTableSys = core::ptr::null_mut();
pub static mut CUIFScheme__s_pColorTableOff10: *mut CUIFColorTableOff10 = core::ptr::null_mut();






























































































































































































































































































#[no_mangle]
pub unsafe extern "C" fn cicCreateUIFScheme(type_: DWORD) -> *mut CUIFScheme {
    return &mut (*CUIFSchemeDef__new_in_0(cpp_alloc_CicNoThrow::<CUIFSchemeDef>(), type_)).__base0 as *mut CUIFScheme;
}
#[no_mangle]
pub unsafe extern "C" fn cicInitUIFUtil() {
    if CUIFBitmapDC__s_phdcSrc.is_null() {
        CUIFBitmapDC__s_phdcSrc = CUIFBitmapDC__new_in_0(cpp_alloc_CicNoThrow::<CUIFBitmapDC>(), (TRUE) as BOOL);
    }

    if CUIFBitmapDC__s_phdcMask.is_null() {
        CUIFBitmapDC__s_phdcMask = CUIFBitmapDC__new_in_0(cpp_alloc_CicNoThrow::<CUIFBitmapDC>(), (TRUE) as BOOL);
    }

    if CUIFBitmapDC__s_phdcDst.is_null() {
        CUIFBitmapDC__s_phdcDst = CUIFBitmapDC__new_in_0(cpp_alloc_CicNoThrow::<CUIFBitmapDC>(), (TRUE) as BOOL);
    }

    if !CUIFBitmapDC__s_phdcSrc.is_null() && !CUIFBitmapDC__s_phdcMask.is_null() && !CUIFBitmapDC__s_phdcDst.is_null() {
        CUIFBitmapDC__s_fInitBitmapDCs = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicDoneUIFUtil() {
    if !CUIFBitmapDC__s_phdcSrc.is_null() {
        CUIFBitmapDC__delete(CUIFBitmapDC__s_phdcSrc);
        CUIFBitmapDC__s_phdcSrc = core::ptr::null_mut();
    }

    if !CUIFBitmapDC__s_phdcMask.is_null() {
        CUIFBitmapDC__delete(CUIFBitmapDC__s_phdcMask);
        CUIFBitmapDC__s_phdcMask = core::ptr::null_mut();
    }

    if !CUIFBitmapDC__s_phdcDst.is_null() {
        CUIFBitmapDC__delete(CUIFBitmapDC__s_phdcDst);
        CUIFBitmapDC__s_phdcDst = core::ptr::null_mut();
    }

    CUIFBitmapDC__s_fInitBitmapDCs = (FALSE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn cicMirrorBitmap(hBitmap: HBITMAP, hbrBack: HBRUSH) -> HBITMAP {
    let mut bm: BITMAP = core::mem::zeroed();
    if (CUIFBitmapDC__s_fInitBitmapDCs == 0)
        || (GetObject(hBitmap as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID) == 0)
    {
        return core::ptr::null_mut();
    }

    (*CUIFBitmapDC__s_phdcSrc).SetBitmap__0(hBitmap);
    (*CUIFBitmapDC__s_phdcDst).SetDIB(bm.bmWidth, bm.bmHeight, 1, 32);
    (*CUIFBitmapDC__s_phdcMask).SetDIB(bm.bmWidth, bm.bmHeight, 1, 32);

    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: bm.bmWidth,
        bottom: bm.bmHeight,
    };
    FillRect((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), core::ptr::addr_of_mut!(rc), hbrBack);

    SetLayout((*CUIFBitmapDC__s_phdcMask).op_to_HDC(), LAYOUT_RTL);

    BitBlt((*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, bm.bmWidth, bm.bmHeight, (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCCOPY);

    SetLayout((*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0);

    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, bm.bmWidth, bm.bmHeight, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 1, 0, SRCCOPY);

    (*CUIFBitmapDC__s_phdcSrc).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcDst).Uninit((TRUE) as i32);
    return (*CUIFBitmapDC__s_phdcDst).DetachBitmap();
}
#[no_mangle]
pub unsafe extern "C" fn cicCreateDitherBrush() -> HBRUSH {
    let mut bytes: [u8; 16] = [0; 16];
    ZeroMemory!(core::ptr::addr_of_mut!(bytes) as *mut core::ffi::c_void, core::mem::size_of::<[u8; 16]>());
    bytes[0] = 0x55;
    bytes[4] = 0x55;
    bytes[8] = 0x55;
    bytes[12] = 0x55;
    bytes[2] = 0xAA;
    bytes[6] = 0xAA;
    bytes[10] = 0xAA;
    bytes[14] = 0xAA;
    let hBitmap: HBITMAP = CreateBitmap(8, 8, 1, 1, bytes.as_mut_ptr() as *mut core::ffi::c_void);
    if hBitmap.is_null() {
        return core::ptr::null_mut();
    }

    let mut lb: LOGBRUSH = core::mem::zeroed();
    lb.lbHatch = (hBitmap as usize) as _;
    lb.lbStyle = BS_PATTERN;
    let hbr: HBRUSH = CreateBrushIndirect(&mut lb);
    DeleteObject((hBitmap).cast());
    return hbr;
}
#[no_mangle]
pub unsafe extern "C" fn cicCreateDisabledBitmap(prc: LPCRECT, hbmMask: HBITMAP, hbr1: HBRUSH, hbr2: HBRUSH, bPressed: BOOL) -> HBITMAP {
    if CUIFBitmapDC__s_fInitBitmapDCs == 0 {
        return core::ptr::null_mut();
    }

    let width: LONG = (*prc).right - (*prc).left;
    let height: LONG = (*prc).bottom - (*prc).top;

    (*CUIFBitmapDC__s_phdcDst).SetDIB(width, height, 1, 32);
    (*CUIFBitmapDC__s_phdcMask).SetBitmap__0(hbmMask);
    (*CUIFBitmapDC__s_phdcSrc).SetDIB(width, height, 1, 32);

    let mut rc: RECT = RECT { left: 0, top: 0, right: width, bottom: height };
    FillRect((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), &mut rc as *mut RECT as *const RECT, hbr1);

    let hbrWhite: HBRUSH = GetStockObject((WHITE_BRUSH as i32)) as HBRUSH;
    FillRect((*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), &mut rc as *mut RECT as *const RECT, hbrWhite);

    BitBlt((*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, SRCINVERT);
    if bPressed != 0 {
        BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 1, 1, width, height,
                 (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCPAINT);
    } else {
        BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height,
                 (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCPAINT);
    }

    FillRect((*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), &mut rc as *mut RECT as *const RECT, hbr2);

    BitBlt((*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, SRCPAINT);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCAND);

    (*CUIFBitmapDC__s_phdcSrc).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcDst).Uninit((TRUE) as i32);
    return (*CUIFBitmapDC__s_phdcDst).DetachBitmap();
}
#[no_mangle]
pub unsafe extern "C" fn cicCreateShadowMaskBmp(prc: LPRECT, hbm1: HBITMAP, hbm2: HBITMAP, hbr1: HBRUSH, hbr2: HBRUSH) -> HBITMAP {
    if CUIFBitmapDC__s_fInitBitmapDCs == 0 {
        return core::ptr::null_mut();
    }

    (*prc).left -= 1;
    (*prc).top -= 1;

    let width: LONG = (*prc).right - (*prc).left;
    let height: LONG = (*prc).bottom - (*prc).top;

    let mut bitmapDC: CUIFBitmapDC = CUIFBitmapDC__make_0((TRUE) as BOOL);

    (*CUIFBitmapDC__s_phdcDst).SetDIB(width, height, 1, 32);
    (*CUIFBitmapDC__s_phdcSrc).SetBitmap__0(hbm1);
    (*CUIFBitmapDC__s_phdcMask).SetBitmap__0(hbm2);
    bitmapDC.SetDIB(width, height, 1, 32);

    let mut rc: RECT = RECT { left: 0, top: 0, right: width, bottom: height };

    FillRect((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), core::ptr::addr_of_mut!(rc), hbr1);
    FillRect(bitmapDC.op_to_HDC(), core::ptr::addr_of_mut!(rc), hbr2);

    BitBlt(bitmapDC.op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, SRCPAINT);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 2, 2, width, height, bitmapDC.op_to_HDC(), 0, 0, SRCAND);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, SRCAND);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCINVERT);

    (*CUIFBitmapDC__s_phdcSrc).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcDst).Uninit((TRUE) as i32);
    return (*CUIFBitmapDC__s_phdcDst).DetachBitmap();
}
#[no_mangle]
pub unsafe extern "C" fn cicChangeBitmapColor(prc: LPCRECT, hbm: HBITMAP, rgbBack: COLORREF, rgbFore: COLORREF) -> HBITMAP {
    if CUIFBitmapDC__s_fInitBitmapDCs == 0 {
        return core::ptr::null_mut();
    }

    let width: INT = (*prc).right - (*prc).left;
    let height: INT = (*prc).bottom - (*prc).top;

    let mut brush: CUIFSolidBrush = CUIFSolidBrush__make_0(rgbFore);

    (*CUIFBitmapDC__s_phdcDst).SetDIB(width, height, 1, 32);
    (*CUIFBitmapDC__s_phdcSrc).SetBitmap__0(hbm);
    (*CUIFBitmapDC__s_phdcMask).SetBitmap__1(width, height, 1, 1);

    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCCOPY);
    SelectObject((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), (brush).op_to_HBRUSH() as HGDIOBJ);
    SetBkColor((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), rgbBack);

    BitBlt((*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, MERGECOPY);
    SetBkColor((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), RGB!(255, 255, 255));
    SetTextColor((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), RGB!(0, 0, 0));
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, 0xE20746);

    (*CUIFBitmapDC__s_phdcSrc).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcDst).Uninit((TRUE) as i32);
    return (*CUIFBitmapDC__s_phdcDst).DetachBitmap();
}
#[no_mangle]
pub unsafe extern "C" fn cicConvertBlackBKGBitmap(prc: LPCRECT, hbm1: HBITMAP, hbm2: HBITMAP, mut hBrush: HBRUSH) -> HBITMAP {
    if CUIFBitmapDC__s_fInitBitmapDCs == 0 {
        return core::ptr::null_mut();
    }

    if IS_INTRESOURCE!(hBrush) != false {
        hBrush = GetSysColorBrush(hBrush as isize as LONG - 1); // HandleToLong
    }

    let mut lb: LOGBRUSH = core::mem::zeroed::<LOGBRUSH>();
    GetObject(hBrush as HGDIOBJ, core::mem::size_of::<LOGBRUSH>() as i32, core::ptr::addr_of_mut!(lb) as LPVOID);
    if lb.lbStyle != 0 || lb.lbColor != 0 {
        return core::ptr::null_mut();
    }

    let width: INT = (*prc).right - (*prc).left;
    let height: INT = (*prc).bottom - (*prc).top;

    let hBitmap: HBITMAP = cicChangeBitmapColor(prc, hbm1, 0, RGB!(255, 255, 255));
    if hBitmap.is_null() {
        return core::ptr::null_mut();
    }

    (*CUIFBitmapDC__s_phdcDst).SetDIB(width, height, 1, 32);
    (*CUIFBitmapDC__s_phdcSrc).SetBitmap__0(hBitmap);
    (*CUIFBitmapDC__s_phdcMask).SetBitmap__0(hbm2);

    let mut rc: RECT = RECT { left: 0, top: 0, right: width, bottom: height };

    let hbrWhite: HBRUSH = GetStockObject((WHITE_BRUSH as i32)) as HBRUSH;
    FillRect((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), core::ptr::addr_of_mut!(rc), hbrWhite);

    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, 0x660046);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, 0x8800C6);

    (*CUIFBitmapDC__s_phdcSrc).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcDst).Uninit((TRUE) as i32);
    DeleteObject(hBitmap as HGDIOBJ);
    return (*CUIFBitmapDC__s_phdcDst).DetachBitmap();
}
#[no_mangle]
pub unsafe extern "C" fn cicCreateMaskBmp(prc: LPCRECT, hbm1: HBITMAP, hbm2: HBITMAP, hbr: HBRUSH, rgbColor: COLORREF, rgbBack: COLORREF) -> HBITMAP {
    if CUIFBitmapDC__s_fInitBitmapDCs == 0 {
        return core::ptr::null_mut();
    }

    let width: INT = (*prc).right - (*prc).left;
    let height: INT = (*prc).bottom - (*prc).top;
    let hBitmap: HBITMAP = cicConvertBlackBKGBitmap(prc, hbm1, hbm2, hbr);
    if !hBitmap.is_null() {
        return hBitmap;
    }

    (*CUIFBitmapDC__s_phdcDst).SetDIB(width, height, 1, 32);
    (*CUIFBitmapDC__s_phdcSrc).SetBitmap__0(hbm1);
    (*CUIFBitmapDC__s_phdcMask).SetBitmap__0(hbm2);

    let mut rc: RECT = RECT { left: 0, top: 0, right: width, bottom: height };

    let OldTextColor: COLORREF = SetTextColor((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), rgbColor);
    let OldBkColor: COLORREF = SetBkColor((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), rgbBack);
    FillRect((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), core::ptr::addr_of!(rc), hbr);
    SetTextColor((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), OldTextColor);
    SetBkColor((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), OldBkColor);

    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, SRCAND);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, width, height, (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCINVERT);
    (*CUIFBitmapDC__s_phdcSrc).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcDst).Uninit((TRUE) as i32);

    return (*CUIFBitmapDC__s_phdcDst).DetachBitmap();
}
#[no_mangle]
pub unsafe extern "C" fn cicGetIconBitmaps(hIcon: HICON, hbm1: *mut HBITMAP, hbm2: *mut HBITMAP, pSize: *const SIZE) -> BOOL {
    if CUIFBitmapDC__s_fInitBitmapDCs == 0 {
        return (FALSE) as BOOL;
    }

    let mut size: SIZE = core::mem::zeroed();
    if !pSize.is_null() {
        size = *pSize;
    } else {
        if cicGetIconSize(hIcon, core::ptr::addr_of_mut!(size)) == 0 {
            return (FALSE) as BOOL;
        }
    }

    (*CUIFBitmapDC__s_phdcSrc).SetDIB(size.cx, size.cy, 1, 32);
    (*CUIFBitmapDC__s_phdcMask).SetBitmap__1(size.cx, size.cy, 1, 1);

    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: size.cx,
        bottom: size.cy,
    };
    FillRect(
        (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(),
        core::ptr::addr_of_mut!(rc),
        GetStockObject((BLACK_BRUSH as i32)) as HBRUSH,
    );

    DrawIconEx(
        (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(),
        0,
        0,
        hIcon,
        size.cx,
        size.cy,
        0,
        0 as HBRUSH,
        DI_NORMAL,
    );
    DrawIconEx(
        (*CUIFBitmapDC__s_phdcMask).op_to_HDC(),
        0,
        0,
        hIcon,
        size.cx,
        size.cy,
        0,
        0 as HBRUSH,
        DI_MASK,
    );

    (*CUIFBitmapDC__s_phdcSrc).Uninit((TRUE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((TRUE) as i32);
    *hbm1 = (*CUIFBitmapDC__s_phdcSrc).DetachBitmap();
    *hbm2 = (*CUIFBitmapDC__s_phdcMask).DetachBitmap();
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn cicDrawMaskBmpOnDC(hDC: HDC, prc: LPCRECT, hbmp: HBITMAP, hbmpMask: HBITMAP) {
    if CUIFBitmapDC__s_fInitBitmapDCs == 0 {
        return;
    }

    let cx: LONG = (*prc).right - (*prc).left;
    let cy: LONG = (*prc).bottom - (*prc).top;
    (*CUIFBitmapDC__s_phdcDst).SetDIB(cx, cy, 1, 32);
    (*CUIFBitmapDC__s_phdcSrc).SetBitmap__0(hbmp);
    (*CUIFBitmapDC__s_phdcMask).SetBitmap__0(hbmpMask);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, cx, cy, hDC, (*prc).left, (*prc).top, SRCCOPY);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, cx, cy, (*CUIFBitmapDC__s_phdcMask).op_to_HDC(), 0, 0, SRCAND);
    BitBlt((*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, cx, cy, (*CUIFBitmapDC__s_phdcSrc).op_to_HDC(), 0, 0, SRCINVERT);
    BitBlt(hDC, (*prc).left, (*prc).top, cx, cy, (*CUIFBitmapDC__s_phdcDst).op_to_HDC(), 0, 0, SRCCOPY);
    (*CUIFBitmapDC__s_phdcSrc).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcMask).Uninit((FALSE) as i32);
    (*CUIFBitmapDC__s_phdcDst).Uninit((FALSE) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn cicGetWorkAreaRect(pt: POINT, prc: LPRECT) {
    SystemParametersInfo(SPI_GETWORKAREA, 0, (prc).cast(), 0);

    let hMon: HMONITOR = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
    if !hMon.is_null() {
        let mut mi: MONITORINFO = core::mem::zeroed();
        mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
        if GetMonitorInfo(hMon, core::ptr::addr_of_mut!(mi)) != 0 {
            *prc = mi.rcWork;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicGetScreenRect(pt: POINT, prc: LPRECT) {
    *prc = RECT { left: 0, top: 0, right: GetSystemMetrics((SM_CXSCREEN as i32)), bottom: GetSystemMetrics((SM_CYSCREEN as i32)) };
    let hMon: HMONITOR = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
    if !hMon.is_null() {
        let mut mi: MONITORINFO = core::mem::zeroed();
        mi.cbSize = core::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfo(hMon, core::ptr::addr_of_mut!(mi));
        *prc = mi.rcMonitor;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicIsFullScreenSize(hWnd: HWND) -> BOOL {
    let mut rc: RECT = core::mem::zeroed();

    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rc));
    return ((rc.left <= 0)
        && (rc.top <= 0)
        && (rc.right >= GetSystemMetrics((SM_CXFULLSCREEN as i32)))
        && (rc.bottom >= GetSystemMetrics((SM_CYFULLSCREEN as i32)))) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn cicGetIconSize(hIcon: HICON, pSize: LPSIZE) -> BOOL {
    let mut IconInfo: ICONINFO = core::mem::zeroed();
    if !(GetIconInfo(hIcon, core::ptr::addr_of_mut!(IconInfo)) != 0) {
        return (FALSE) as BOOL;
    }

    let mut bm: BITMAP = core::mem::zeroed();
    GetObject(
        IconInfo.hbmColor as HGDIOBJ,
        core::mem::size_of::<BITMAP>() as i32,
        core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void,
    );
    DeleteObject(IconInfo.hbmColor as HGDIOBJ);
    DeleteObject(IconInfo.hbmMask as HGDIOBJ);
    (*pSize).cx = bm.bmWidth;
    (*pSize).cy = bm.bmHeight;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn cicInitUIFSys() {
    CUIFSystemInfo__s_pSystemInfo = CUIFSystemInfo__new_in_0(cpp_alloc_CicNoThrow::<CUIFSystemInfo>());
    if !CUIFSystemInfo__s_pSystemInfo.is_null() {
        (*CUIFSystemInfo__s_pSystemInfo).Initialize();
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicDoneUIFSys() {
    if !CUIFSystemInfo__s_pSystemInfo.is_null() {
        CUIFSystemInfo__delete(CUIFSystemInfo__s_pSystemInfo);
        CUIFSystemInfo__s_pSystemInfo = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicUpdateUIFSys() {
    if !CUIFSystemInfo__s_pSystemInfo.is_null() {
        (*CUIFSystemInfo__s_pSystemInfo).GetSystemMetrics();
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicInitUIFScheme() {
    let mut pColorTable: *mut CUIFColorTable;

    CUIFScheme__s_pColorTableSys = CUIFColorTableSys__new_in_0(cpp_alloc_CicNoThrow::<CUIFColorTableSys>());
    pColorTable = core::ptr::addr_of_mut!((*CUIFScheme__s_pColorTableSys).__base0);
    if !pColorTable.is_null() {
        (*pColorTable).InitColor();
        (*pColorTable).InitBrush();
    }

    CUIFScheme__s_pColorTableOff10 = CUIFColorTableOff10__new_in_0(cpp_alloc_CicNoThrow::<CUIFColorTableOff10>());
    pColorTable = core::ptr::addr_of_mut!((*CUIFScheme__s_pColorTableOff10).__base0);
    if !pColorTable.is_null() {
        (*pColorTable).InitColor();
        (*pColorTable).InitBrush();
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicUpdateUIFScheme() {
    if !CUIFScheme__s_pColorTableSys.is_null() {
        (*CUIFScheme__s_pColorTableSys).__base0.Update();
    }
    if !CUIFScheme__s_pColorTableOff10.is_null() {
        (*CUIFScheme__s_pColorTableOff10).__base0.Update();
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicDoneUIFScheme() {
    if !CUIFScheme__s_pColorTableSys.is_null() {
        CUIFColorTableSys__delete(CUIFScheme__s_pColorTableSys);
        CUIFScheme__s_pColorTableSys = core::ptr::null_mut();
    }
    if !CUIFScheme__s_pColorTableOff10.is_null() {
        CUIFColorTableOff10__delete(CUIFScheme__s_pColorTableOff10);
        CUIFScheme__s_pColorTableOff10 = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicInitUIFLib() {
    cicInitUIFSys();
    cicInitUIFScheme();
    cicInitUIFUtil();
}
#[no_mangle]
pub unsafe extern "C" fn cicDoneUIFLib() {
    cicDoneUIFScheme();
    cicDoneUIFSys();
    cicDoneUIFUtil();
}

impl CUIFSystemInfo {
    pub unsafe fn GetSystemMetrics(&mut self) {
        let this_: *mut Self = self;
        let hDC: HDC = GetDC(core::ptr::null_mut());
        (*this_).m_cBitsPixels = (GetDeviceCaps(hDC, (BITSPIXEL as i32))) as u32;
        ReleaseDC(core::ptr::null_mut(), hDC);
    
        let mut HighContrast: HIGHCONTRAST = core::mem::zeroed();
        HighContrast.cbSize = core::mem::size_of::<HIGHCONTRAST>() as u32;
        SystemParametersInfo(
            SPI_GETHIGHCONTRAST,
            core::mem::size_of::<HIGHCONTRAST>() as u32,
            core::ptr::addr_of_mut!(HighContrast) as *mut core::ffi::c_void,
            0,
        );
        (*this_).m_bHighContrast1 = ((HighContrast.dwFlags & HCF_HIGHCONTRASTON) != 0) as BOOL;
        let rgbBtnText: COLORREF = GetSysColor((COLOR_BTNTEXT as i32));
        let rgbBtnFace: COLORREF = GetSysColor((COLOR_BTNFACE as i32));
        let black: COLORREF = RGB!(0, 0, 0);
        let white: COLORREF = RGB!(255, 255, 255);
        (*this_).m_bHighContrast2 = ((*this_).m_bHighContrast1 != 0
            || (rgbBtnText == black && rgbBtnFace == white)
            || (rgbBtnText == white && rgbBtnFace == black)) as BOOL;
    }
    pub unsafe fn Initialize(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.dwOSVersionInfoSize = (core::mem::size_of::<OSVERSIONINFO>()) as DWORD;
        GetVersionEx(this_ as *mut OSVERSIONINFO);
        (*this_).GetSystemMetrics();
    }
}

impl CUIFTheme {
    pub unsafe fn InternalOpenThemeData(&mut self, hWnd: HWND) -> HRESULT {
        let this_: *mut Self = self;
        if hWnd.is_null() || (*this_).m_pszClassList.is_null() {
            return E_FAIL;
        }
    
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnOpenThemeData), (&[0x0075, 0x0078, 0x0074, 0x0068, 0x0065, 0x006d, 0x0065, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "uxtheme.dll" */, b"OpenThemeData\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
        (*this_).m_hTheme = (CUIFTheme__s_fnOpenThemeData.unwrap_unchecked())(hWnd, (*this_).m_pszClassList);
        return (if !(*this_).m_hTheme.is_null() { S_OK } else { E_FAIL });
    }
    pub unsafe fn EnsureThemeData(&mut self, hWnd: HWND) -> HRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_hTheme.is_null() {
            return S_OK;
        }
        return (*this_).InternalOpenThemeData(hWnd);
    }
    pub unsafe fn CloseThemeData(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_hTheme.is_null() {
            return S_OK;
        }
    
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnCloseThemeData), (&[0x0075, 0x0078, 0x0074, 0x0068, 0x0065, 0x006d, 0x0065, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "uxtheme.dll" */, b"CloseThemeData\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
    
        let hr: HRESULT = (CUIFTheme__s_fnCloseThemeData.unwrap_unchecked())((*this_).m_hTheme);
        (*this_).m_hTheme = core::ptr::null_mut();
        return hr;
    }
    pub unsafe fn DrawThemeBackground__body(&mut self, hDC: HDC, iStateId: i32, pRect: LPCRECT, pClipRect: LPCRECT) -> HRESULT {
        let this_: *mut Self = self;
        let uxtheme_dll = ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
        if ((cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnDrawThemeBackground), uxtheme_dll.as_ptr(), b"DrawThemeBackground\0".as_ptr() as *const i8)) == 0) {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnDrawThemeBackground.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, pRect, pClipRect);
    }
    pub unsafe fn DrawThemeParentBackground__body(&mut self, hwnd: HWND, hDC: HDC, prc: LPRECT) -> HRESULT {
        let this_: *mut Self = self;
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnDrawThemeParentBackground), (&[0x0075, 0x0078, 0x0074, 0x0068, 0x0065, 0x006d, 0x0065, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "uxtheme.dll" */, b"DrawThemeParentBackground\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnDrawThemeParentBackground.unwrap_unchecked())(hwnd, hDC, prc);
    }
    pub unsafe fn DrawThemeText__body(&mut self, hDC: HDC, iStateId: i32, pszText: LPCWSTR, cchText: i32, dwTextFlags: DWORD, dwTextFlags2: DWORD, pRect: LPCRECT) -> HRESULT {
        let this_: *mut Self = self;
        static UXTHEME_DLL: [u16; 12] = ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
        if (cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnDrawThemeText), (UXTHEME_DLL.as_ptr()), b"DrawThemeText\0".as_ptr() as *const i8)) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnDrawThemeText.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, pszText, cchText, dwTextFlags, dwTextFlags2, pRect);
    }
    pub unsafe fn DrawThemeIcon__body(&mut self, hDC: HDC, iStateId: i32, pRect: LPCRECT, himl: HIMAGELIST, iImageIndex: i32) -> HRESULT {
        let this_: *mut Self = self;
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnDrawThemeIcon), [('u' as u16), ('x' as u16), ('t' as u16), ('h' as u16), ('e' as u16), ('m' as u16), ('e' as u16), ('.' as u16), ('d' as u16), ('l' as u16), ('l' as u16), 0u16].as_ptr(), b"DrawThemeIcon\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnDrawThemeIcon.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, pRect, himl, iImageIndex);
    }
    pub unsafe fn GetThemeBackgroundExtent__body(&mut self, hDC: HDC, iStateId: i32, pContentRect: LPCRECT, pExtentRect: LPRECT) -> HRESULT {
        let this_: *mut Self = self;
        if (cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeBackgroundExtent), (&[0x0075, 0x0078, 0x0074, 0x0068, 0x0065, 0x006d, 0x0065, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "uxtheme.dll" */, b"GetThemeBackgroundExtent\0".as_ptr() as *const i8)) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnGetThemeBackgroundExtent.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, pContentRect, pExtentRect);
    }
    pub unsafe fn GetThemeBackgroundContentRect__body(&mut self, hDC: HDC, iStateId: i32, pBoundingRect: LPCRECT, pContentRect: LPRECT) -> HRESULT {
        let this_: *mut Self = self;
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeBackgroundContentRect), (['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0].as_ptr()), b"GetThemeBackgroundContentRect\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnGetThemeBackgroundContentRect.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, pBoundingRect, pContentRect);
    }
    pub unsafe fn GetThemeTextExtent__body(&mut self, hDC: HDC, iStateId: i32, pszText: LPCWSTR, cchCharCount: i32, dwTextFlags: DWORD, pBoundingRect: LPCRECT, pExtentRect: LPRECT) -> HRESULT {
        let this_: *mut Self = self;
        if !(cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeTextExtent), ([ 'u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0u16 ].as_ptr()), b"GetThemeTextExtent\0".as_ptr() as *const i8) != 0) {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnGetThemeTextExtent.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, pszText, cchCharCount, dwTextFlags, pBoundingRect, pExtentRect);
    }
    pub unsafe fn GetThemePartSize__body(&mut self, hDC: HDC, iStateId: INT, prc: LPRECT, eSize: THEMESIZE, psz: *mut SIZE) -> HRESULT {
        let this_: *mut Self = self;
        let uxtheme: [u16; 12] = ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemePartSize), (uxtheme.as_ptr()), b"GetThemePartSize\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnGetThemePartSize.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, prc, eSize, psz);
    }
    pub unsafe fn DrawThemeEdge__body(&mut self, hDC: HDC, iStateId: i32, pDestRect: LPCRECT, uEdge: UINT, uFlags: UINT, pContentRect: LPRECT) -> HRESULT {
        let this_: *mut Self = self;
        static UXTHEME_DLL: [u16; 12] = ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnDrawThemeEdge), (UXTHEME_DLL.as_ptr()), b"DrawThemeEdge\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnDrawThemeEdge.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, pDestRect, uEdge, uFlags, pContentRect);
    }
    pub unsafe fn GetThemeColor__body(&mut self, iStateId: i32, iPropId: i32, pColor: *mut COLORREF) -> HRESULT {
        let this_: *mut Self = self;
        if (cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeColor), (&[0x0075, 0x0078, 0x0074, 0x0068, 0x0065, 0x006d, 0x0065, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "uxtheme.dll" */, b"GetThemeColor\0".as_ptr() as *const i8) as usize) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnGetThemeColor.unwrap_unchecked())((*this_).m_hTheme, (*this_).m_iPartId, iStateId, iPropId, pColor);
    }
    pub unsafe fn GetThemeMargins__body(&mut self, hDC: HDC, iStateId: i32, iPropId: i32, prc: LPRECT, pMargins: *mut MARGINS) -> HRESULT {
        let this_: *mut Self = self;
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeMargins), ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0u16].as_ptr(), b"GetThemeMargins\0".as_ptr() as *const i8) == 0 {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnGetThemeMargins.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, iPropId, prc, pMargins);
    }
    pub unsafe fn GetThemeFont__body(&mut self, hDC: HDC, iStateId: i32, iPropId: i32, pFont: *mut LOGFONTW) -> HRESULT {
        let this_: *mut Self = self;
        let uxtheme_dll: [u16; 12] = ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
        if ((cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeFont), uxtheme_dll.as_ptr(), b"GetThemeFont\0".as_ptr() as *const i8)) == 0) {
            return E_FAIL;
        }
        return (CUIFTheme__s_fnGetThemeFont.unwrap_unchecked())((*this_).m_hTheme, hDC, (*this_).m_iPartId, iStateId, iPropId, pFont);
    }
    pub unsafe fn GetThemeSysColor__body(&mut self, iColorId: INT) -> COLORREF {
        let this_: *mut Self = self;
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeSysColor), ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0u16].as_ptr(), b"GetThemeSysColor\0".as_ptr() as *const i8) == 0 {
            return RGB!(0, 0, 0);
        }
        return (CUIFTheme__s_fnGetThemeSysColor.unwrap_unchecked())((*this_).m_hTheme, iColorId);
    }
    pub unsafe fn GetThemeSysSize__body(&mut self, iSizeId: i32) -> i32 {
        let this_: *mut Self = self;
        if cicGetFN(core::ptr::addr_of_mut!(CUIFTheme__s_hUXTHEME), core::ptr::addr_of_mut!(CUIFTheme__s_fnGetThemeSysSize), ['u' as u16, 'x' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'm' as u16, 'e' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0].as_ptr(), b"GetThemeSysSize\0".as_ptr() as *const i8) == 0 {
            return 0;
        }
        return (CUIFTheme__s_fnGetThemeSysSize.unwrap_unchecked())((*this_).m_hTheme, iSizeId);
    }
    pub unsafe fn SetActiveTheme__body(&mut self, pszClassList: LPCWSTR, iPartId: INT, iStateId: INT) {
        let this_: *mut Self = self;
        (*this_).m_iPartId = iPartId;
        (*this_).m_iStateId = iStateId;
        (*this_).m_pszClassList = pszClassList;
    }
}

impl CUIFObject {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFObject, nObjectID: DWORD, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
        (*this_).__base0.m_pszClassList = core::ptr::null_mut();
        (*this_).__base0.m_hTheme = core::ptr::null_mut();
        (*this_).m_pParent = pParent;
        (*this_).m_nObjectID = nObjectID;
        (*this_).m_style = style;
    
        if !prc.is_null() {
            (*this_).m_rc = *prc;
        } else {
            (*this_).m_rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        }
    
        if !(*this_).m_pParent.is_null() {
            (*this_).m_pWindow = (*(*this_).m_pParent).m_pWindow;
            (*this_).m_pScheme = (*(*this_).m_pParent).m_pScheme;
        } else {
            (*this_).m_pWindow = core::ptr::null_mut();
            (*this_).m_pScheme = core::ptr::null_mut();
        }
    
        (*this_).m_bVisible = (TRUE) as i32;
        (*this_).m_bEnable = (*this_).m_bVisible;
    
        (*this_).m_hFont = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;
        (*this_).m_bHasCustomFont = (FALSE) as i32;
    
        (*this_).m_pszToolTip = core::ptr::null_mut();
    
        (*this_).m_dwUnknown4[0 as usize] = (1 as DWORD).wrapping_neg(); //FIXME: name
        (*this_).m_dwUnknown4[1 as usize] = (1 as DWORD).wrapping_neg(); //FIXME: name
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pWindow.is_null() {
            let mut pToolTip: *mut CUIFToolTip = (*(*this_).m_pWindow).m_pToolTip;
            if !pToolTip.is_null() && (*pToolTip).m_pToolTipTarget == this_ {
                (*pToolTip).m_pToolTipTarget = core::ptr::null_mut();
            }
        }
    
        if !(*this_).m_pszToolTip.is_null() {
            cpp_delete_array((*this_).m_pszToolTip);
            (*this_).m_pszToolTip = core::ptr::null_mut();
        }
    
        let mut iObj: usize = (*this_).m_ObjectArray.size();
        while iObj > 0 {
            iObj -= 1;
            CUIFObject__delete((*this_).m_ObjectArray[iObj]);
        }
        (*this_).m_ObjectArray.clear();
    
        if !(*this_).m_pWindow.is_null() {
            (*(*this_).m_pWindow).RemoveUIObj(this_);
        }
    
        (*this_).__base0.CloseThemeData();
    }
    pub unsafe fn StartCapture(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pWindow.is_null() {
            (*(*this_).m_pWindow).SetCaptureObject(this_);
        }
    }
    pub unsafe fn EndCapture(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pWindow.is_null() {
            (*(*this_).m_pWindow).SetCaptureObject(core::ptr::null_mut());
        }
    }
    pub unsafe fn IsCapture(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).m_pWindow != core::ptr::null_mut() && ((*(*this_).m_pWindow).m_pCaptured == this_)) as BOOL;
    }
    pub unsafe fn IsRTL(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pWindow.is_null() {
            return (FALSE) as BOOL;
        }
        return (((*(*this_).m_pWindow).__base0.m_style & UIF_WINDOW_LAYOUTRTL) != 0) as BOOL;
    }
    pub unsafe fn NotifyCommand(&mut self, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pParent.is_null() {
            return (*(*this_).m_pParent).OnObjectNotify(this_, wParam, lParam);
        }
        return 0;
    }
    pub unsafe fn ObjectFromPoint(&mut self, pt: POINT) -> *mut CUIFObject {
        let this_: *mut Self = self;
        if (*this_).PtInObject(pt) == 0 {
            return core::ptr::null_mut();
        }
    
        let mut pFound: *mut CUIFObject = this_;
        let mut i: usize = 0;
        while i < (*this_).m_ObjectArray.size() {
            let base: *mut *mut CUIFObject =
                core::ptr::addr_of_mut!((*this_).m_ObjectArray.__base0) as *mut *mut CUIFObject;
            let elem: *mut CUIFObject = *base.add(i);
            let pObject: *mut CUIFObject = (*elem).ObjectFromPoint(pt);
            if !pObject.is_null() {
                pFound = pObject;
            }
            i += 1;
        }
        return pFound;
    }
    pub unsafe fn SetScheme(&mut self, scheme: *mut CUIFScheme) {
        let this_: *mut Self = self;
        (*this_).m_pScheme = scheme;
        let mut i: usize = 0;
        while i < (*this_).m_ObjectArray.size() as usize {
            (*(*(*this_).m_ObjectArray.__base0.add(i))).SetScheme(scheme);
            i += 1;
        }
    }
    pub unsafe fn StartTimer(&mut self, wParam: WPARAM) {
        let this_: *mut Self = self;
        if !(*this_).m_pWindow.is_null() {
            (*(*this_).m_pWindow).SetTimerObject(this_, (wParam) as UINT);
        }
    }
    pub unsafe fn EndTimer(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pWindow.is_null() {
            (*(*this_).m_pWindow).SetTimerObject(core::ptr::null_mut(), 0);
        }
    }
    pub unsafe fn OnPaint__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if !(((*(*this_).m_pWindow).__base0.m_style & UIF_WINDOW_ENABLETHEMED) != 0) || (*this_).OnPaintTheme(hDC) == 0 {
            (*this_).OnPaintNoTheme(hDC);
        }
    }
    pub unsafe fn OnSetCursor__body(&mut self, uMsg: UINT,
        x: LONG,
        y: LONG,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
    pub unsafe fn GetRect__body(&mut self, prc: LPRECT) {
        let this_: *mut Self = self;
        *prc = (*this_).m_rc;
    }
    pub unsafe fn SetRect__body(&mut self, prc: LPCRECT) {
        let this_: *mut Self = self;
        (*this_).m_rc = *prc;
        if !(*this_).m_pWindow.is_null() {
            (*(*this_).m_pWindow).OnObjectMoved(this_);
        }
        (*this_).CallOnPaint();
    }
    pub unsafe fn PtInObject__body(&mut self, pt: POINT) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).m_bVisible != 0 && PtInRect(core::ptr::addr_of_mut!((*this_).m_rc), pt) != 0) as BOOL;
    }
    pub unsafe fn PaintObject__body(&mut self, hDC: HDC, mut prc: LPCRECT) {
        let this_: *mut Self = self;
        if (*this_).m_bVisible == 0 {
            return;
        }
    
        if prc.is_null() {
            prc = core::ptr::addr_of!((*this_).m_rc);
        }
    
        (*this_).OnPaint(hDC);
    
        let mut iItem: usize = 0;
        while iItem < (*this_).m_ObjectArray.size() {
            let pObject: *mut CUIFObject =
                *(core::ptr::addr_of_mut!((*this_).m_ObjectArray.__base0) as *mut *mut CUIFObject).add(iItem);
            let mut rc: RECT = core::mem::zeroed();
            if IntersectRect(
                core::ptr::addr_of_mut!(rc),
                prc,
                core::ptr::addr_of!((*pObject).m_rc),
            ) != 0
            {
                (*pObject).PaintObject(hDC, core::ptr::addr_of!(rc));
            }
            iItem += 1;
        }
    }
    pub unsafe fn CallOnPaint__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pWindow.is_null() {
            (*(*this_).m_pWindow).UpdateUI(core::ptr::addr_of!((*this_).m_rc));
        }
    }
    pub unsafe fn Enable__body(&mut self, bEnable: BOOL) {
        let this_: *mut Self = self;
        if (*this_).m_bEnable == bEnable {
            return;
        }
    
        (*this_).m_bEnable = bEnable;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_ObjectArray.size() {
            let p: *mut *mut CUIFObject =
                core::ptr::addr_of_mut!((*this_).m_ObjectArray.__base0) as *mut *mut CUIFObject;
            let pItem: *mut CUIFObject = *p.add(iItem);
            (*pItem).Enable(bEnable);
            iItem += 1;
        }
    
        (*this_).CallOnPaint();
    }
    pub unsafe fn Show__body(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        if (*this_).m_bVisible == bVisible {
            return;
        }
    
        (*this_).m_bVisible = bVisible;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_ObjectArray.size() {
            let pItem: *mut *mut CUIFObject =
                core::ptr::addr_of_mut!((*this_).m_ObjectArray.__base0) as *mut *mut CUIFObject;
            (**pItem.add(iItem)).Show(bVisible);
            iItem += 1;
        }
    
        if (*this_).m_bVisible != 0 || !(*this_).m_pParent.is_null() {
            (*(*this_).m_pParent).CallOnPaint();
        }
    }
    pub unsafe fn SetFontToThis__body(&mut self, mut hFont: HFONT) {
        let this_: *mut Self = self;
        (*this_).m_bHasCustomFont = if hFont.is_null() { 0 } else { 1 };
        if hFont.is_null() {
            hFont = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;
        }
        (*this_).m_hFont = hFont;
    }
    pub unsafe fn SetFont__body(&mut self, hFont: HFONT) {
        let this_: *mut Self = self;
        (*this_).SetFontToThis(hFont);
    
        let mut iItem: usize = 0;
        while iItem < (*this_).m_ObjectArray.size() {
            let base: *mut *mut CUIFObject =
                core::ptr::addr_of_mut!((*this_).m_ObjectArray.__base0) as *mut *mut CUIFObject;
            (*(*base.add(iItem))).SetFont(hFont);
            iItem += 1;
        }
    
        (*this_).CallOnPaint();
    }
    pub unsafe fn SetStyle__body(&mut self, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).m_style = style;
    }
    pub unsafe fn AddUIObj__body(&mut self, pObject: *mut CUIFObject) {
        let this_: *mut Self = self;
        (*this_).m_ObjectArray.Add(pObject);
        (*this_).CallOnPaint();
    }
    pub unsafe fn RemoveUIObj__body(&mut self, pObject: *mut CUIFObject) {
        let this_: *mut Self = self;
        if (*this_).m_ObjectArray.Remove(pObject) != 0 {
            (*this_).CallOnPaint();
        }
    }
    pub unsafe fn OnObjectNotify__body(&mut self, pObject: *mut CUIFObject, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pParent.is_null() {
            return (*(*this_).m_pParent).OnObjectNotify(pObject, wParam, lParam);
        }
        return 0;
    }
    pub unsafe fn SetToolTip__body(&mut self, pszToolTip: LPCWSTR) {
        let this_: *mut Self = self;
        if !(*this_).m_pszToolTip.is_null() {
            cpp_delete_array((*this_).m_pszToolTip);
            (*this_).m_pszToolTip = core::ptr::null_mut();
        }
    
        if !pszToolTip.is_null() {
            let cch: usize = wcslen(pszToolTip);
            (*this_).m_pszToolTip = cpp_new_array_CicNoThrow::<WCHAR>(cch + 1);
            if !(*this_).m_pszToolTip.is_null() {
                lstrcpynW((*this_).m_pszToolTip, pszToolTip, (cch + 1) as i32);
            }
        }
    }
    pub unsafe fn DetachWndObj__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pWindow.is_null() {
            let mut pToolTip: *mut CUIFToolTip = (*(*this_).m_pWindow).m_pToolTip;
            if !pToolTip.is_null() && (*pToolTip).m_pToolTipTarget == this_ {
                (*pToolTip).m_pToolTipTarget = core::ptr::null_mut();
            }
    
            (*(*this_).m_pWindow).RemoveUIObj(this_);
            (*this_).m_pWindow = core::ptr::null_mut();
        }
    }
    pub unsafe fn ClearWndObj__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_pWindow = core::ptr::null_mut();
        let mut iItem: usize = 0;
        while iItem < (*this_).m_ObjectArray.size() {
            (*(*(*this_).m_ObjectArray.__base0.add(iItem))).ClearWndObj();
            iItem += 1;
        }
    }
    pub unsafe fn ClearTheme__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.CloseThemeData();
        let mut iItem: usize = 0;
        while iItem < (*this_).m_ObjectArray.size() {
            let pItems: *mut *mut CUIFObject =
                core::ptr::addr_of_mut!((*this_).m_ObjectArray.__base0) as *mut *mut CUIFObject;
            (*(*pItems.add(iItem))).ClearTheme();
            iItem += 1;
        }
    }
}

impl CUIFColorTableSys {
    pub unsafe fn GetBrush(&mut self, iColor: INT) -> HBRUSH {
        let this_: *mut Self = self;
        if (*this_).m_hBrushes[iColor as usize].is_null() {
            (*this_).m_hBrushes[iColor as usize] = CreateSolidBrush((*this_).m_rgbColors[iColor as usize]);
        }
        return (*this_).m_hBrushes[iColor as usize];
    }
    pub unsafe fn InitColor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_rgbColors[0 as usize] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[1 as usize] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[2 as usize] = GetSysColor((COLOR_ACTIVEBORDER as i32));
        (*this_).m_rgbColors[3 as usize] = GetSysColor((COLOR_ACTIVECAPTION as i32));
        (*this_).m_rgbColors[4 as usize] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[5 as usize] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[6 as usize] = GetSysColor((COLOR_BTNTEXT as i32));
        (*this_).m_rgbColors[7 as usize] = GetSysColor((COLOR_CAPTIONTEXT as i32));
        (*this_).m_rgbColors[8 as usize] = GetSysColor((COLOR_GRAYTEXT as i32));
        (*this_).m_rgbColors[9 as usize] = GetSysColor((COLOR_HIGHLIGHT as i32));
        (*this_).m_rgbColors[10 as usize] = GetSysColor((COLOR_HIGHLIGHTTEXT as i32));
        (*this_).m_rgbColors[11 as usize] = GetSysColor((COLOR_INACTIVECAPTION as i32));
        (*this_).m_rgbColors[12 as usize] = GetSysColor((COLOR_INACTIVECAPTIONTEXT as i32));
        (*this_).m_rgbColors[13 as usize] = GetSysColor((COLOR_MENUTEXT as i32));
        (*this_).m_rgbColors[14 as usize] = GetSysColor((COLOR_WINDOW as i32));
        (*this_).m_rgbColors[15 as usize] = GetSysColor((COLOR_WINDOWTEXT as i32));
    }
    pub unsafe fn InitBrush__body(&mut self) {
        let this_: *mut Self = self;
        ZeroMemory!((*this_).m_hBrushes.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[HBRUSH; 16 as usize]>());
    }
    pub unsafe fn DoneBrush__body(&mut self) {
        let this_: *mut Self = self;
        let mut i: usize = 0;
        while i < _countof!((*this_).m_hBrushes) as usize {
            if !(*this_).m_hBrushes[i].is_null() {
                DeleteObject(((*this_).m_hBrushes[i]).cast());
                (*this_).m_hBrushes[i] = core::ptr::null_mut();
            }
            i += 1;
        }
    }
}

impl CUIFColorTableOff10 {
    pub unsafe fn GetBrush(&mut self, iColor: INT) -> HBRUSH {
        let this_: *mut Self = self;
        if (*this_).m_hBrushes[iColor as usize].is_null() {
            (*this_).m_hBrushes[iColor as usize] = CreateSolidBrush((*this_).m_rgbColors[iColor as usize]);
        }
        return (*this_).m_hBrushes[iColor as usize];
    }
    pub unsafe fn InitColor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_rgbColors[0] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[1] = GetSysColor((COLOR_WINDOW as i32));
        (*this_).m_rgbColors[2] = GetSysColor((COLOR_WINDOW as i32));
        (*this_).m_rgbColors[3] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[4] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[5] = GetSysColor((COLOR_WINDOW as i32));
        (*this_).m_rgbColors[6] = GetSysColor((COLOR_HIGHLIGHT as i32));
        (*this_).m_rgbColors[7] = GetSysColor((COLOR_WINDOWTEXT as i32));
        (*this_).m_rgbColors[8] = GetSysColor((COLOR_HIGHLIGHT as i32));
        (*this_).m_rgbColors[9] = GetSysColor((COLOR_HIGHLIGHT as i32));
        (*this_).m_rgbColors[10] = GetSysColor((COLOR_HIGHLIGHTTEXT as i32));
        (*this_).m_rgbColors[11] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[12] = GetSysColor((COLOR_BTNTEXT as i32));
        (*this_).m_rgbColors[13] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[14] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[15] = GetSysColor((COLOR_WINDOW as i32));
        (*this_).m_rgbColors[16] = GetSysColor((COLOR_HIGHLIGHT as i32));
        (*this_).m_rgbColors[17] = GetSysColor((COLOR_BTNTEXT as i32));
        (*this_).m_rgbColors[18] = GetSysColor((COLOR_WINDOW as i32));
        (*this_).m_rgbColors[19] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[20] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[21] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[22] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[23] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[24] = GetSysColor((COLOR_CAPTIONTEXT as i32));
        (*this_).m_rgbColors[25] = GetSysColor((COLOR_HIGHLIGHT as i32));
        (*this_).m_rgbColors[26] = GetSysColor((COLOR_HIGHLIGHTTEXT as i32));
        (*this_).m_rgbColors[27] = GetSysColor((COLOR_BTNFACE as i32));
        (*this_).m_rgbColors[28] = GetSysColor((COLOR_BTNTEXT as i32));
        (*this_).m_rgbColors[29] = GetSysColor((COLOR_BTNSHADOW as i32));
        (*this_).m_rgbColors[30] = GetSysColor((COLOR_BTNTEXT as i32));
        (*this_).m_rgbColors[31] = GetSysColor((COLOR_WINDOWTEXT as i32));
    }
    pub unsafe fn InitBrush__body(&mut self) {
        let this_: *mut Self = self;
        ZeroMemory!((*this_).m_hBrushes.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[HBRUSH; 32]>());
    }
    pub unsafe fn DoneBrush__body(&mut self) {
        let this_: *mut Self = self;
        let mut i: usize = 0;
        while i < _countof!((*this_).m_hBrushes) as usize {
            if !(*this_).m_hBrushes[i].is_null() {
                DeleteObject(((*this_).m_hBrushes[i]).cast());
                (*this_).m_hBrushes[i] = core::ptr::null_mut();
            }
            i += 1;
        }
    }
}

impl CUIFIcon {
    pub unsafe fn GetImageList(&mut self, bMirror: BOOL) -> HIMAGELIST {
        let this_: *mut Self = self;
        if (*this_).m_hImageList.is_null() {
            return core::ptr::null_mut();
        }
    
        if !(*this_).m_hIcon.is_null() {
            let mut iconSize: SIZE = core::mem::zeroed();
            cicGetIconSize((*this_).m_hIcon, core::ptr::addr_of_mut!(iconSize));
    
            let mut uFlags: UINT = ILC_COLOR32 | ILC_MASK;
            if bMirror != 0 {
                uFlags |= ILC_MIRROR;
            }
    
            (*this_).m_hImageList = ImageList_Create(iconSize.cx, iconSize.cy, uFlags, 1, 0);
            if !(*this_).m_hImageList.is_null() {
                ImageList_ReplaceIcon((*this_).m_hImageList, -1, (*this_).m_hIcon);
            }
    
            return (*this_).m_hImageList;
        }
    
        core::ptr::null_mut()
    }
}

impl CUIFBitmapDC {
    pub unsafe fn __ctor_0(&mut self, bMemory: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_hBitmap = core::ptr::null_mut();
        (*this_).m_hOldBitmap = core::ptr::null_mut();
        (*this_).m_hOldObject = core::ptr::null_mut();
        if bMemory != 0 {
            (*this_).m_hDC = CreateCompatibleDC(core::ptr::null_mut());
        } else {
            let display: [u16; 8] = ['D' as u16, 'I' as u16, 'S' as u16, 'P' as u16, 'L' as u16, 'A' as u16, 'Y' as u16, 0];
            (*this_).m_hDC = CreateDC(display.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Uninit((FALSE) as i32);
        DeleteDC((*this_).m_hDC);
    }
    pub unsafe fn Uninit(&mut self, bKeep: BOOL) {
        let this_: *mut Self = self;
        if !(*this_).m_hOldBitmap.is_null() {
            SelectObject((*this_).m_hDC, (*this_).m_hOldBitmap);
            (*this_).m_hOldBitmap = core::ptr::null_mut();
        }
    
        if !(*this_).m_hOldObject.is_null() {
            SelectObject((*this_).m_hDC, (*this_).m_hOldObject);
            (*this_).m_hOldObject = core::ptr::null_mut();
        }
    
        if bKeep == 0 {
            if !(*this_).m_hBitmap.is_null() {
                DeleteObject(((*this_).m_hBitmap).cast());
                (*this_).m_hBitmap = core::ptr::null_mut();
            }
        }
    }
    pub unsafe fn SetBitmap__0(&mut self, hBitmap: HBITMAP) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_hDC as usize != 0 {
            (*this_).m_hOldBitmap = SelectObject((*this_).m_hDC, (hBitmap).cast());
        }
        return (TRUE) as BOOL;
    }
    pub unsafe fn SetBitmap__1(&mut self, cx: LONG, cy: LONG, cPlanes: WORD, cBitCount: WORD) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_hBitmap = CreateBitmap(cx, cy, cPlanes as u32, cBitCount as u32, core::ptr::null());
        (*this_).m_hOldBitmap = SelectObject((*this_).m_hDC, ((*this_).m_hBitmap).cast());
        return (TRUE) as BOOL;
    }
    pub unsafe fn SetDIB(&mut self, cx: LONG, cy: LONG, cPlanes: WORD, cBitCount: WORD) -> BOOL {
        let this_: *mut Self = self;
        let mut bmi: BITMAPINFO = core::mem::zeroed();
        ZeroMemory!(core::ptr::addr_of_mut!(bmi) as *mut core::ffi::c_void, core::mem::size_of::<BITMAPINFO>());
        bmi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
        bmi.bmiHeader.biWidth = cx;
        bmi.bmiHeader.biHeight = cy;
        bmi.bmiHeader.biPlanes = cPlanes;
        bmi.bmiHeader.biBitCount = cBitCount;
        bmi.bmiHeader.biCompression = BI_RGB;
        (*this_).m_hBitmap = CreateDIBSection((*this_).m_hDC, core::ptr::addr_of_mut!(bmi) as *const BITMAPINFO, DIB_RGB_COLORS, core::ptr::null_mut(), core::ptr::null_mut(), 0);
        (*this_).m_hOldBitmap = SelectObject((*this_).m_hDC, ((*this_).m_hBitmap).cast());
        return (TRUE) as BOOL;
    }
}

impl CUIFScheme {
    pub unsafe fn FillRect__body(&mut self, hDC: HDC, prc: LPCRECT, iColor: INT) {
        let this_: *mut Self = self;
        FillRect(hDC, prc, (*this_).GetBrush(iColor));
    }
    pub unsafe fn FrameRect__body(&mut self, hDC: HDC, prc: LPCRECT, iColor: INT) {
        let this_: *mut Self = self;
        FrameRect(hDC, prc, (*this_).GetBrush(iColor));
    }
}

impl CUIFSchemeDef {
    pub unsafe fn GetType__body(&mut self) -> DWORD {
        let this_: *mut Self = self;
        (*this_).m_dwType
    }
    pub unsafe fn GetColor__body(&mut self, iColor: INT) -> COLORREF {
        let this_: *mut Self = self;
        return (*CUIFScheme__s_pColorTableSys).GetColor(iColor);
    }
    pub unsafe fn GetBrush__body(&mut self, iColor: INT) -> HBRUSH {
        let this_: *mut Self = self;
        return (*CUIFScheme__s_pColorTableSys).GetBrush(iColor);
    }
    pub unsafe fn CyMenuItem__body(&mut self, cyText: INT) -> INT {
        let this_: *mut Self = self;
        return cyText + 2;
    }
    pub unsafe fn CxSizeFrame__body(&mut self) -> INT {
        let this_: *mut Self = self;
        return GetSystemMetrics((SM_CXSIZEFRAME as i32));
    }
    pub unsafe fn CySizeFrame__body(&mut self) -> INT {
        let this_: *mut Self = self;
        return GetSystemMetrics((SM_CYSIZEFRAME as i32));
    }
    pub unsafe fn DrawSelectionRect__body(&mut self, hDC: HDC, prc: LPCRECT, arg2: i32) {
        let this_: *mut Self = self;
        FillRect(hDC, prc, (*this_).GetBrush(6));
    }
    pub unsafe fn GetCtrlFaceOffset__body(&mut self, dwUnknownFlags: DWORD, dwDrawFlags: DWORD, pSize: LPSIZE) {
        let this_: *mut Self = self;
        if (dwDrawFlags & UIF_DRAW_PRESSED) == 0 {
            if (dwDrawFlags & 0x2) != 0 {
                if (dwUnknownFlags & 0x10) != 0 {
                    (*pSize).cx = -1;
                    (*pSize).cy = (*pSize).cx;
                    return;
                }
                (*pSize).cx = (((dwUnknownFlags & 0x20) != 0) as i32);
                (*pSize).cy = (*pSize).cx;
            } else {
                if (dwDrawFlags & 0x1) == 0 {
                    (*pSize).cx = -(((dwUnknownFlags & 1) != 0) as i32);
                    (*pSize).cy = (*pSize).cx;
                    return;
                }
                if (dwUnknownFlags & 0x4) != 0 {
                    (*pSize).cx = -1;
                    (*pSize).cy = (*pSize).cx;
                    return;
                }
                (*pSize).cx = (((dwUnknownFlags & 0x8) != 0) as i32);
                (*pSize).cy = (*pSize).cx;
            }
            return;
        }
    
        if (dwUnknownFlags & 0x40) == 0 {
            (*pSize).cx = (((dwUnknownFlags & 0x80) != 0) as i32);
            (*pSize).cy = (*pSize).cx;
            return;
        }
    
        (*pSize).cx = -1;
        (*pSize).cy = (*pSize).cx;
    }
    pub unsafe fn DrawCtrlBkgd__body(&mut self, hDC: HDC, prc: LPCRECT, dwUnknownFlags: DWORD, dwDrawFlags: DWORD) {
        let this_: *mut Self = self;
        FillRect(hDC, prc, (*this_).GetBrush(9));
    
        if (dwDrawFlags & UIF_DRAW_PRESSED) == 0 && (dwDrawFlags & 0x2) != 0 {
            return;
        }
    
        let hbrDither: HBRUSH = cicCreateDitherBrush();
        if hbrDither.is_null() {
            return;
        }
    
        let rgbOldText: COLORREF = SetTextColor(hDC, GetSysColor((COLOR_BTNFACE as i32)));
        let rgbOldBack: COLORREF = SetBkColor(hDC, GetSysColor((COLOR_BTNHIGHLIGHT as i32)));
    
        let mut rc: RECT = *prc;
        InflateRect(core::ptr::addr_of_mut!(rc), -2, -2);
        FillRect(hDC, core::ptr::addr_of_mut!(rc), hbrDither);
        SetTextColor(hDC, rgbOldText);
        SetBkColor(hDC, rgbOldBack);
        DeleteObject((hbrDither).cast());
    }
    pub unsafe fn DrawCtrlEdge__body(&mut self, hDC: HDC, prc: LPCRECT, dwUnknownFlags: DWORD, dwDrawFlags: DWORD) {
        let this_: *mut Self = self;
        let mut uEdge: UINT = BDR_RAISEDINNER;
    
        if dwDrawFlags & 0x10 != 0 {
            if !(dwUnknownFlags & 0x40 != 0) {
                if dwUnknownFlags & 0x80 != 0 {
                    uEdge = BDR_SUNKENOUTER;
                } else {
                    return;
                }
            }
        } else if dwDrawFlags & 0x2 != 0 {
            if !(dwUnknownFlags & 0x10 != 0) {
                if dwUnknownFlags & 0x20 != 0 {
                    uEdge = BDR_SUNKENOUTER;
                } else {
                    return;
                }
            }
        } else if dwDrawFlags & 0x1 != 0 {
            if !(dwUnknownFlags & 0x4 != 0) {
                if dwUnknownFlags & 0x8 != 0 {
                    uEdge = BDR_SUNKENOUTER;
                } else {
                    return;
                }
            }
        } else if !(dwUnknownFlags & 0x1 != 0) {
            return;
        }
    
        let mut rc: RECT = *prc;
        DrawEdge(hDC, core::ptr::addr_of_mut!(rc), uEdge, BF_RECT);
    }
    pub unsafe fn DrawCtrlText__body(&mut self, hDC: HDC, prc: LPCRECT, pszText: LPCWSTR, mut cchText: INT, dwDrawFlags: DWORD, bRight: BOOL) {
        let this_: *mut Self = self;
        let rgbOldText: COLORREF = GetTextColor(hDC);
        let OldBkMode: INT = SetBkMode(hDC, (TRANSPARENT as i32));
    
        if cchText == -1 {
            cchText = lstrlenW(pszText);
        }
    
        let mut rc: RECT = *prc;
        if dwDrawFlags & UIF_DRAW_DISABLED != 0 {
            OffsetRect(core::ptr::addr_of_mut!(rc), 1, 1);
            SetTextColor(hDC, GetSysColor((COLOR_BTNHIGHLIGHT as i32)));
            ExtTextOutW(
                hDC,
                (if bRight != 0 { rc.right } else { rc.left }),
                rc.top,
                ETO_CLIPPED,
                core::ptr::addr_of!(rc),
                pszText,
                (cchText) as UINT,
                core::ptr::null_mut(),
            );
            OffsetRect(core::ptr::addr_of_mut!(rc), -1, -1);
        }
    
        SetTextColor(hDC, GetSysColor((COLOR_BTNTEXT as i32)));
        ExtTextOutW(
            hDC,
            (if bRight != 0 { rc.right } else { rc.left }),
            rc.top,
            ETO_CLIPPED,
            core::ptr::addr_of!(rc),
            pszText,
            (cchText) as UINT,
            core::ptr::null_mut(),
        );
    
        SetTextColor(hDC, rgbOldText);
        SetBkMode(hDC, OldBkMode);
    }
    pub unsafe fn DrawCtrlIcon__body(&mut self, hDC: HDC, prc: LPCRECT, hIcon: HICON, dwDrawFlags: DWORD, pSize: LPSIZE) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_bMirroring != 0 {
            let mut hbm1: HBITMAP = core::mem::zeroed();
            let mut hbm2: HBITMAP = core::mem::zeroed();
            if cicGetIconBitmaps(hIcon, core::ptr::addr_of_mut!(hbm1), core::ptr::addr_of_mut!(hbm2), pSize) != 0 {
                (*this_).DrawCtrlBitmap(hDC, prc, hbm1, hbm2, dwDrawFlags);
                DeleteObject((hbm1).cast());
                DeleteObject((hbm2).cast());
            }
        } else {
            let mut uFlags: UINT = DST_PREFIXTEXT | DST_TEXT;
            if dwDrawFlags & UIF_DRAW_DISABLED != 0 {
                uFlags |= DSS_MONO | DSS_DISABLED;
            }
            DrawState(hDC, core::ptr::null_mut(), None, hIcon as LPARAM, 0, (*prc).left, (*prc).top, 0, 0, uFlags);
        }
    }
    pub unsafe fn DrawCtrlBitmap__body(&mut self, hDC: HDC, prc: LPCRECT, mut hbm1: HBITMAP, mut hbm2: HBITMAP, dwDrawFlags: DWORD) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_bMirroring != 0 {
            hbm1 = cicMirrorBitmap(hbm1, (*this_).GetBrush(9));
            hbm2 = cicMirrorBitmap(hbm2, GetStockObject((BLACK_BRUSH as i32)) as HBRUSH);
        }
    
        let mut hBrush: HBRUSH = UlongToHandle!(COLOR_BTNFACE + 1) as HBRUSH;
        let mut bBrushCreated: BOOL = (FALSE) as i32;
        if !hbm2.is_null() {
            let mut hBitmap: HBITMAP = core::ptr::null_mut();
            if (dwDrawFlags & UIF_DRAW_DISABLED) != 0 {
                hBitmap = cicCreateDisabledBitmap(prc, hbm2, (*this_).GetBrush(9), (*this_).GetBrush(11), (TRUE) as BOOL);
            } else {
                if (dwDrawFlags & UIF_DRAW_PRESSED) != 0 && (dwDrawFlags & 0x2) == 0 {
                    hBrush = cicCreateDitherBrush();
                    bBrushCreated = (TRUE) as i32;
                }
    
                let rgbFace: COLORREF = GetSysColor((COLOR_BTNFACE as i32));
                let rgbHighlight: COLORREF = GetSysColor((COLOR_BTNHIGHLIGHT as i32));
                hBitmap = cicCreateMaskBmp(prc, hbm1, hbm2, hBrush, rgbFace, rgbHighlight);
            }
    
            if !hBitmap.is_null() {
                DrawState(hDC, core::ptr::null_mut(), None, hBitmap as LPARAM, 0,
                            (*prc).left, (*prc).top,
                            (*prc).right - (*prc).left, (*prc).bottom - (*prc).top,
                            DST_BITMAP);
                DeleteObject((hBitmap).cast());
            }
        } else {
            let mut uFlags: UINT = DST_BITMAP;
            if (dwDrawFlags & UIF_DRAW_DISABLED) != 0 {
                uFlags |= DSS_MONO | DSS_DISABLED;
            }
    
            DrawState(hDC, core::ptr::null_mut(), None, hbm1 as LPARAM, 0,
                        (*prc).left, (*prc).top,
                        (*prc).right - (*prc).left, (*prc).bottom - (*prc).top,
                        uFlags);
        }
    
        if bBrushCreated != 0 {
            DeleteObject((hBrush).cast());
        }
    
        if (*this_).__base0.m_bMirroring != 0 {
            DeleteObject((hbm1).cast());
            DeleteObject((hbm2).cast());
        }
    }
    pub unsafe fn DrawMenuBitmap__body(&mut self, hDC: HDC, prc: LPCRECT, hbm1: HBITMAP, hbm2: HBITMAP, dwDrawFlags: DWORD) {
        let this_: *mut Self = self;
        (*this_).DrawCtrlBitmap(hDC, prc, hbm1, hbm2, dwDrawFlags);
    }
    pub unsafe fn DrawMenuSeparator__body(&mut self, hDC: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut rc: RECT = *prc;
        rc.bottom = rc.top + (rc.bottom - rc.top) / 2;
        FillRect(hDC, core::ptr::addr_of_mut!(rc), UlongToHandle!(COLOR_BTNSHADOW + 1) as HBRUSH);
    
        rc = *prc;
        rc.top += (rc.bottom - rc.top) / 2;
        FillRect(hDC, core::ptr::addr_of_mut!(rc), UlongToHandle!(COLOR_BTNHIGHLIGHT + 1) as HBRUSH);
    }
    pub unsafe fn DrawFrameCtrlBkgd__body(&mut self, hDC: HDC, prc: LPCRECT, dwUnknownFlags: DWORD, dwDrawFlags: DWORD) {
        let this_: *mut Self = self;
        (*this_).DrawCtrlBkgd(hDC, prc, dwUnknownFlags, dwDrawFlags);
    }
    pub unsafe fn DrawFrameCtrlEdge__body(&mut self, hDC: HDC, prc: LPCRECT, dwUnknownFlags: DWORD, dwDrawFlags: DWORD) {
        let this_: *mut Self = self;
        (*this_).DrawCtrlEdge(hDC, prc, dwUnknownFlags, dwDrawFlags);
    }
    pub unsafe fn DrawFrameCtrlIcon__body(&mut self, hDC: HDC, prc: LPCRECT, hIcon: HICON, dwDrawFlags: DWORD, pSize: LPSIZE) {
        let this_: *mut Self = self;
        (*this_).DrawCtrlIcon(hDC, prc, hIcon, dwDrawFlags, pSize);
    }
    pub unsafe fn DrawFrameCtrlBitmap__body(&mut self, hDC: HDC, prc: LPCRECT, hbm1: HBITMAP, hbm2: HBITMAP, dwDrawFlags: DWORD) {
        let this_: *mut Self = self;
        (*this_).DrawCtrlBitmap(hDC, prc, hbm1, hbm2, dwDrawFlags);
    }
    pub unsafe fn DrawWndFrame__body(&mut self, hDC: HDC, prc: LPCRECT, r#type: DWORD, unused1: DWORD, unused2: DWORD) {
        let this_: *mut Self = self;
        let mut rc: RECT = *prc;
        if r#type != 0 && r#type <= 2 {
            DrawEdge(hDC, core::ptr::addr_of_mut!(rc), BDR_RAISED, BF_RECT);
        } else {
            (*this_).__base0.FrameRect(hDC, core::ptr::addr_of_mut!(rc), 14);
        }
    }
    pub unsafe fn DrawDragHandle__body(&mut self, hDC: HDC, prc: LPCRECT, bVertical: BOOL) {
        let this_: *mut Self = self;
        let mut rc: RECT;
        if bVertical != 0 {
            rc = RECT { left: (*prc).left, top: (*prc).top + 1, right: (*prc).right, bottom: (*prc).top + 4 };
        } else {
            rc = RECT { left: (*prc).left + 1, top: (*prc).top, right: (*prc).left + 4, bottom: (*prc).bottom };
        }
        DrawEdge(hDC, core::ptr::addr_of_mut!(rc), BDR_RAISEDINNER, BF_RECT);
    }
    pub unsafe fn DrawSeparator__body(&mut self, hDC: HDC, prc: LPCRECT, bVertical: BOOL) {
        let this_: *mut Self = self;
        let hLightPen: HPEN = CreatePen((PS_SOLID as i32), 0, GetSysColor((COLOR_BTNHIGHLIGHT as i32)));
        if hLightPen.is_null() {
            return;
        }
    
        let hShadowPen: HPEN = CreatePen((PS_SOLID as i32), 0, GetSysColor((COLOR_BTNSHADOW as i32)));
        if hShadowPen.is_null() {
            DeleteObject((hLightPen).cast());
            return;
        }
    
        let hPenOld: HGDIOBJ = SelectObject(hDC, (hShadowPen).cast());
        if bVertical != 0 {
            MoveToEx(hDC, (*prc).left, (*prc).top + 1, core::ptr::null_mut());
            LineTo(hDC, (*prc).right, (*prc).top + 1);
            SelectObject(hDC, (hLightPen).cast());
            MoveToEx(hDC, (*prc).left, (*prc).top + 2, core::ptr::null_mut());
            LineTo(hDC, (*prc).right, (*prc).top + 2);
        } else {
            MoveToEx(hDC, (*prc).left + 1, (*prc).top, core::ptr::null_mut());
            LineTo(hDC, (*prc).left + 1, (*prc).bottom);
            SelectObject(hDC, (hLightPen).cast());
            MoveToEx(hDC, (*prc).left + 2, (*prc).top, core::ptr::null_mut());
            LineTo(hDC, (*prc).left + 2, (*prc).bottom);
        }
        SelectObject(hDC, hPenOld);
    
        DeleteObject((hShadowPen).cast());
        DeleteObject((hLightPen).cast());
    }
}

impl CUIFWindow {
    pub unsafe fn __ctor_0(&mut self, hInst: HINSTANCE, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut(), 0, core::ptr::null_mut(), style);
        (*this_).__init_vtables();
        {
            (*this_).m_hInst = hInst;
            (*this_).m_nLeft = 200;
            (*this_).m_nTop = 200;
            (*this_).m_nWidth = 200;
            (*this_).m_nHeight = 200;
            (*this_).m_hWnd = core::ptr::null_mut();
            (*this_).__base0.m_pWindow = this_;
            (*this_).m_pCaptured = core::ptr::null_mut();
            (*this_).m_pTimerObject = core::ptr::null_mut();
            (*this_).m_pPointed = core::ptr::null_mut();
            (*this_).m_bPointing = (FALSE) as i32;
            (*this_).m_pToolTip = core::ptr::null_mut();
            (*this_).m_pShadow = core::ptr::null_mut();
            (*this_).m_bShowShadow = (TRUE) as i32;
            (*this_).m_pBehindModal = core::ptr::null_mut();
            (*this_).CreateScheme();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pToolTip.is_null() {
            CUIFToolTip__delete((*this_).m_pToolTip);
            (*this_).m_pToolTip = core::ptr::null_mut();
        }
        if !(*this_).m_pShadow.is_null() {
            CUIFShadow__delete((*this_).m_pShadow);
            (*this_).m_pShadow = core::ptr::null_mut();
        }
        let mut i: usize = (*this_).__base0.m_ObjectArray.size();
        while i > 0 {
            i -= 1;
            let pObject: *mut CUIFObject = *(core::ptr::addr_of_mut!((*this_).__base0.m_ObjectArray) as *mut *mut CUIFObject).add(i);
            *(core::ptr::addr_of_mut!((*this_).__base0.m_ObjectArray) as *mut *mut CUIFObject).add(i) = core::ptr::null_mut();
            (*this_).__base0.m_ObjectArray.Remove(pObject);
            CUIFObject__delete(pObject);
        }
        if !(*this_).__base0.m_pScheme.is_null() {
            CUIFScheme__delete((*this_).__base0.m_pScheme);
            (*this_).__base0.m_pScheme = core::ptr::null_mut();
        }
    }
    pub unsafe fn GetThis(hWnd: HWND) -> *mut CUIFWindow {
        return GetWindowLongPtr(hWnd, GWLP_USERDATA) as *mut CUIFWindow;
    }
    pub unsafe fn SetThis(hWnd: HWND, dwNewLong: LONG_PTR) {
        SetWindowLongPtr(hWnd, GWLP_USERDATA, dwNewLong);
    }
    pub unsafe fn Initialize__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let pszClass: LPCTSTR = (*this_).GetClassNameW();
    
        let mut wcx: WNDCLASSEX = core::mem::zeroed();
        ZeroMemory!(core::ptr::addr_of_mut!(wcx) as *mut _, core::mem::size_of::<WNDCLASSEX>());
        wcx.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
        if GetClassInfoEx((*this_).m_hInst, pszClass, core::ptr::addr_of_mut!(wcx)) == 0 {
            ZeroMemory!(core::ptr::addr_of_mut!(wcx) as *mut _, core::mem::size_of::<WNDCLASSEX>());
            wcx.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
            wcx.style = (CS_HREDRAW as UINT) | (CS_VREDRAW as UINT) | (CS_DBLCLKS as UINT);
            wcx.lpfnWndProc = Some(CUIFWindow::WindowProcedure);
            wcx.cbClsExtra = 0 as INT;
            wcx.cbWndExtra = (core::mem::size_of::<LONG_PTR>() as INT) * (2 as INT);
            wcx.hInstance = (*this_).m_hInst;
            wcx.hIcon = core::ptr::null_mut();
            wcx.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW as usize as LPCWSTR);
            wcx.lpszClassName = pszClass;
            wcx.hbrBackground = core::ptr::null_mut();
            wcx.lpszMenuName = core::ptr::null();
            wcx.hIconSm = core::ptr::null_mut();
            RegisterClassEx(core::ptr::addr_of_mut!(wcx));
        }
    
        cicUpdateUIFSys();
        cicUpdateUIFScheme();
    
        if (*this_).__base0.m_style & (UIF_WINDOW_TOOLTIP as DWORD) != 0 {
            let style: DWORD = ((*this_).__base0.m_style & (UIF_WINDOW_LAYOUTRTL as DWORD)) | (UIF_WINDOW_TOPMOST as DWORD) | (0x10 as DWORD);
            (*this_).m_pToolTip = CUIFToolTip__new_in_0(
                cpp_alloc_CicNoThrow::<CUIFToolTip>(),
                (*this_).m_hInst,
                style,
                this_,
            );
            if !(*this_).m_pToolTip.is_null() {
                (*(*this_).m_pToolTip).__base0.Initialize();
            }
        }
    
        if (*this_).__base0.m_style & (UIF_WINDOW_SHADOW as DWORD) != 0 {
            (*this_).m_pShadow = CUIFShadow__new_in_0(
                cpp_alloc_CicNoThrow::<CUIFShadow>(),
                (*this_).m_hInst,
                UIF_WINDOW_TOPMOST as DWORD,
                this_,
            );
            if !(*this_).m_pShadow.is_null() {
                (*(*this_).m_pShadow).Initialize();
            }
        }
    
        return (*this_).__base0.Initialize();
    }
    pub unsafe fn Show__body(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        if IsWindow((*this_).m_hWnd) == 0 {
            return;
        }
    
        if bVisible != 0 && (((*this_).__base0.m_style & UIF_WINDOW_TOPMOST) != 0) {
            SetWindowPos((*this_).m_hWnd, core::ptr::null_mut(), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);
        }
    
        (*this_).__base0.m_bVisible = bVisible;
        ShowWindow((*this_).m_hWnd, if bVisible != 0 { (SW_SHOWNOACTIVATE as i32) } else { 0 });
    }
    pub unsafe fn SetRect__body(&mut self, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut Rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
    
        if IsWindow((*this_).m_hWnd) != 0 {
            GetClientRect((*this_).m_hWnd, core::ptr::addr_of_mut!(Rect));
        }
    
        (*this_).__base0.SetRect(core::ptr::addr_of!(Rect));
    }
    pub unsafe fn PaintObject__body(&mut self, mut hDC: HDC, mut prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut bGotDC: BOOL = (FALSE) as i32;
        if hDC.is_null() {
            hDC = GetDC((*this_).m_hWnd);
            bGotDC = (TRUE) as i32;
        }
    
        if prc.is_null() {
            prc = core::ptr::addr_of!((*this_).__base0.m_rc);
        }
    
        let hMemDC: HDC = CreateCompatibleDC(hDC);
        let hbmMem: HBITMAP = CreateCompatibleBitmap(hDC, (*prc).right - (*prc).left, (*prc).bottom - (*prc).top);
    
        let hbmOld: HGDIOBJ = SelectObject(hMemDC, hbmMem as HGDIOBJ);
        SetViewportOrgEx(hMemDC, -(*prc).left, -(*prc).top, core::ptr::null_mut());
        if (((*this_).__base0.__base0.EnsureThemeData((*this_).m_hWnd)) < 0) ||
            (((((*this_).__base0.m_style & UIF_WINDOW_CHILD) == 0) || (((*this_).__base0.__base0.DrawThemeParentBackground((*this_).m_hWnd, hMemDC, core::ptr::addr_of_mut!((*this_).__base0.m_rc))) < 0)) &&
             (((*this_).__base0.__base0.DrawThemeBackground(hMemDC, (*this_).__base0.__base0.m_iStateId, core::ptr::addr_of!((*this_).__base0.m_rc), core::ptr::null_mut())) < 0))
        {
            if !(*this_).__base0.m_pScheme.is_null() {
                (*(*this_).__base0.m_pScheme).FillRect(hMemDC, prc, 22);
            }
        }
    
        (*this_).__base0.PaintObject(hMemDC, prc);
        BitBlt(hDC, (*prc).left, (*prc).top,
                      (*prc).right - (*prc).left, (*prc).bottom - (*prc).top,
                 hMemDC, (*prc).left, (*prc).top, SRCCOPY);
        SelectObject(hMemDC, hbmOld);
        DeleteObject(hbmMem as HGDIOBJ);
        DeleteDC(hMemDC);
    
        if bGotDC != 0 {
            ReleaseDC((*this_).m_hWnd, hDC);
        }
    }
    pub unsafe fn RemoveUIObj__body(&mut self, pRemove: *mut CUIFObject) {
        let this_: *mut Self = self;
        if pRemove == (*this_).m_pCaptured {
            (*this_).SetCaptureObject(core::ptr::null_mut());
        }
    
        if pRemove == (*this_).m_pTimerObject {
            (*this_).m_pTimerObject = core::ptr::null_mut();
            KillTimer((*this_).m_hWnd, (USER_TIMER_ID) as UINT_PTR);
        }
    
        if pRemove == (*this_).m_pPointed {
            (*this_).m_pPointed = core::ptr::null_mut();
        }
    
        (*this_).__base0.RemoveUIObj(pRemove);
    }
    pub unsafe fn SetCaptureObject(&mut self, pCaptured: *mut CUIFObject) {
        let this_: *mut Self = self;
        if !pCaptured.is_null() {
            (*this_).m_pCaptured = pCaptured;
            (*this_).SetCapture((TRUE) as i32);
        } else {
            (*this_).m_pCaptured = core::ptr::null_mut();
            (*this_).SetCapture((FALSE) as i32);
        }
    }
    pub unsafe fn SetObjectPointed(&mut self, pPointed: *mut CUIFObject, pt: POINT) {
        let this_: *mut Self = self;
        if pPointed == (*this_).m_pPointed {
            return;
        }
    
        if !(*this_).m_pCaptured.is_null() {
            if (*this_).m_pCaptured == (*this_).m_pPointed && (*(*this_).m_pPointed).m_bEnable != 0 {
                (*(*this_).m_pPointed).OnMouseOut(pt.x, pt.y);
            }
        } else if !(*this_).m_pPointed.is_null() && (*(*this_).m_pPointed).m_bEnable != 0 {
            (*(*this_).m_pPointed).OnMouseOut(pt.x, pt.y);
        }
    
        (*this_).m_pPointed = pPointed;
    
        if !(*this_).m_pCaptured.is_null() {
            if (*this_).m_pCaptured == (*this_).m_pPointed && (*(*this_).m_pPointed).m_bEnable != 0 {
                (*(*this_).m_pPointed).OnMouseIn(pt.x, pt.y);
            }
        } else if !(*this_).m_pPointed.is_null() && (*(*this_).m_pPointed).m_bEnable != 0 {
            (*(*this_).m_pPointed).OnMouseIn(pt.x, pt.y);
        }
    }
    pub unsafe fn CreateScheme(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).__base0.m_pScheme.is_null() {
            CUIFScheme__delete((*this_).__base0.m_pScheme);
            (*this_).__base0.m_pScheme = core::ptr::null_mut();
        }
    
        let mut iScheme: INT = 0;
        if ((*this_).__base0.m_style & UIF_WINDOW_USESCHEME1) != 0 {
            iScheme = 1;
        } else if ((*this_).__base0.m_style & UIF_WINDOW_USESCHEME2) != 0 {
            iScheme = 2;
        } else if ((*this_).__base0.m_style & UIF_WINDOW_USESCHEME3) != 0 {
            iScheme = 3;
        }
    
        (*this_).__base0.m_pScheme = cicCreateUIFScheme(iScheme as DWORD);
        (*this_).__base0.SetScheme((*this_).__base0.m_pScheme);
    }
    pub unsafe fn GetWorkArea(&mut self, prcWnd: LPCRECT, prcWorkArea: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        if ((*this_).__base0.m_style & (UIF_WINDOW_WORKAREA | UIF_WINDOW_MONITOR)) == 0 {
            return (FALSE) as BOOL;
        }
    
        let hMon: HMONITOR = MonitorFromRect(prcWnd, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = core::mem::zeroed();
        mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
        if hMon.is_null() || GetMonitorInfo(hMon, core::ptr::addr_of_mut!(mi)) == 0 {
            if ((*this_).__base0.m_style & UIF_WINDOW_WORKAREA) != 0 {
                return SystemParametersInfo(SPI_GETWORKAREA, 0, prcWorkArea as *mut core::ffi::c_void, 0);
            }
    
            (*prcWorkArea).top = 0;
            (*prcWorkArea).left = (*prcWorkArea).top;
            (*prcWorkArea).right = GetSystemMetrics((SM_CXSCREEN as i32));
            (*prcWorkArea).bottom = GetSystemMetrics((SM_CYSCREEN as i32));
            return (TRUE) as BOOL;
        }
    
        if ((*this_).__base0.m_style & UIF_WINDOW_WORKAREA) != 0 {
            *prcWorkArea = mi.rcWork;
            return (TRUE) as BOOL;
        }
    
        *prcWorkArea = mi.rcMonitor;
        return (TRUE) as BOOL;
    }
    pub unsafe fn AdjustWindowPosition(&mut self) {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        rc.left = (*this_).m_nLeft;
        rc.right = (*this_).m_nLeft + (*this_).m_nWidth;
        rc.top = (*this_).m_nTop;
        rc.bottom = (*this_).m_nTop + (*this_).m_nHeight;
    
        let mut rcWorkArea: RECT = core::mem::zeroed();
        if (*this_).GetWorkArea(&mut rc, &mut rcWorkArea) == 0 {
            return;
        }
    
        if (*this_).m_nLeft < rcWorkArea.left {
            (*this_).m_nLeft = rcWorkArea.left;
        }
        if (*this_).m_nTop < rcWorkArea.top {
            (*this_).m_nTop = rcWorkArea.top;
        }
        if (*this_).m_nLeft + (*this_).m_nWidth >= rcWorkArea.right {
            (*this_).m_nLeft = rcWorkArea.right - (*this_).m_nWidth;
        }
        if (*this_).m_nTop + (*this_).m_nHeight >= rcWorkArea.bottom {
            (*this_).m_nTop = rcWorkArea.bottom - (*this_).m_nHeight;
        }
    }
    pub unsafe fn SetBehindModal(&mut self, pBehindModal: *mut CUIFWindow,
    ) {
        let this_: *mut Self = self;
        (*this_).m_pBehindModal = pBehindModal;
    }
    pub unsafe fn SetTimerObject(&mut self, pTimerObject: *mut CUIFObject, uElapse: UINT) {
        let this_: *mut Self = self;
        if !pTimerObject.is_null() {
            (*this_).m_pTimerObject = pTimerObject;
            SetTimer((*this_).m_hWnd, (USER_TIMER_ID) as UINT_PTR, uElapse, None);
        } else {
            (*this_).m_pTimerObject = core::ptr::null_mut();
            KillTimer((*this_).m_hWnd, (USER_TIMER_ID) as UINT_PTR);
        }
    }
    pub unsafe extern "system" fn WindowProcedure(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let mut This: *mut CUIFWindow;
    
        if uMsg == WM_NCCREATE {
            This = (*(lParam as *mut CREATESTRUCT)).lpCreateParams as *mut CUIFWindow;
            CUIFWindow::SetThis(hWnd, (This) as LONG_PTR);
            (*This).m_hWnd = hWnd;
        } else {
            This = CUIFWindow::GetThis(hWnd);
        }
    
        if uMsg == WM_GETMINMAXINFO {
            if !This.is_null() {
                return (*This).WindowProc(hWnd, uMsg, wParam, lParam);
            } else {
                return DefWindowProc(hWnd, uMsg, wParam, lParam);
            }
        }
    
        if This.is_null() {
            return 0;
        }
    
        if uMsg == WM_NCDESTROY {
            (*This).m_hWnd = core::ptr::null_mut();
            SetWindowLongPtr(hWnd, GWLP_USERDATA, 0);
        }
    
        return (*This).WindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn GetWndStyle__body(&mut self) -> DWORD {
        let this_: *mut Self = self;
        let mut ret: DWORD;
    
        if (*this_).__base0.m_style & UIF_WINDOW_CHILD != 0 {
            ret = WS_CHILD | WS_CLIPSIBLINGS;
        } else {
            ret = WS_POPUP | WS_DISABLED;
        }
    
        if (*this_).__base0.m_style & UIF_WINDOW_USESCHEME1 != 0 {
            ret |= WS_BORDER;
        } else if (*this_).__base0.m_style & UIF_WINDOW_DLGFRAME != 0 {
            ret |= WS_DLGFRAME;
        } else if ((*this_).__base0.m_style & UIF_WINDOW_USESCHEME2 != 0) || ((*this_).__base0.m_style & 0x10 != 0) {
            ret |= WS_BORDER;
        }
    
        return ret;
    }
    pub unsafe fn GetWndStyleEx__body(&mut self) -> DWORD {
        let this_: *mut Self = self;
        let mut ret: DWORD = 0;
        if ((*this_).__base0.m_style & UIF_WINDOW_TOPMOST) != 0 {
            ret |= WS_EX_TOPMOST;
        }
        if ((*this_).__base0.m_style & UIF_WINDOW_TOOLWINDOW) != 0 {
            ret |= WS_EX_TOOLWINDOW;
        }
        if ((*this_).__base0.m_style & UIF_WINDOW_LAYOUTRTL) != 0 {
            ret |= WS_EX_LAYOUTRTL;
        }
        return ret;
    }
    pub unsafe fn CreateWnd__body(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let hWnd: HWND = CreateWindowEx((*this_).GetWndStyleEx(), (*this_).GetClassNameW(), (*this_).GetWndTitle(), (*this_).GetWndStyle(),
                                   (*this_).m_nLeft, (*this_).m_nTop, (*this_).m_nWidth, (*this_).m_nHeight,
                                   hwndParent, core::ptr::null_mut(), (*this_).m_hInst, this_ as *mut core::ffi::c_void);
        if !(*this_).m_pToolTip.is_null() {
            (*(*this_).m_pToolTip).__base0.CreateWnd(hWnd);
        }
        if !(*this_).m_pShadow.is_null() {
            (*(*this_).m_pShadow).__base0.CreateWnd(hWnd);
        }
        return hWnd;
    }
    pub unsafe fn Move__body(&mut self, x: INT, y: INT, nWidth: INT, nHeight: INT) {
        let this_: *mut Self = self;
        (*this_).m_nLeft = x;
        (*this_).m_nTop = y;
        if nWidth >= 0 {
            (*this_).m_nWidth = nWidth;
        }
        if nHeight >= 0 {
            (*this_).m_nHeight = nHeight;
        }
        if IsWindow((*this_).m_hWnd) != 0 {
            (*this_).AdjustWindowPosition();
            MoveWindow((*this_).m_hWnd, (*this_).m_nLeft, (*this_).m_nTop, (*this_).m_nWidth, (*this_).m_nHeight, (TRUE) as BOOL);
        }
    }
    pub unsafe fn AnimateWnd__body(&mut self, dwTime: DWORD, dwFlags: DWORD) -> BOOL {
        let this_: *mut Self = self;
        if IsWindow((*this_).m_hWnd) == 0 {
            return (FALSE) as BOOL;
        }
    
        let bVisible: BOOL = ((dwFlags & 0x10000) == 0) as BOOL;
        (*this_).OnAnimationStart();
        let ret: BOOL = AnimateWindow((*this_).m_hWnd, dwTime, dwFlags);
        if ret == 0 {
            (*this_).__base0.m_bVisible = bVisible;
        }
        (*this_).OnAnimationEnd();
        return ret;
    }
    pub unsafe fn OnObjectMoved__body(&mut self, pObject: *mut CUIFObject) {
        let this_: *mut Self = self;
        if IsWindow((*this_).m_hWnd) == 0 {
            return;
        }
    
        let mut pt: POINT = core::mem::zeroed();
        GetCursorPos(core::ptr::addr_of_mut!(pt));
        ScreenToClient((*this_).m_hWnd, core::ptr::addr_of_mut!(pt));
        let mut pt2: POINT = pt;
        let pFound: *mut CUIFObject = (*this_).__base0.ObjectFromPoint(pt);
        (*this_).SetObjectPointed(pFound, pt2);
    }
    pub unsafe fn OnSettingChange__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        let _ = this_;
        DefWindowProc(hWnd, uMsg, wParam, lParam)
    }
    pub unsafe fn WindowProc__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        'sw: {
            match uMsg {
                WM_GETOBJECT => {
                    return (*this_).OnGetObject(hWnd, WM_GETOBJECT, wParam, lParam) as LRESULT;
                }
    
                WM_SYSCOLORCHANGE => {
                    cicUpdateUIFScheme();
                    (*this_).OnSysColorChange();
                    return 0;
                }
    
                WM_ENDSESSION => {
                    (*this_).OnEndSession(hWnd, wParam, lParam);
                    return 0;
                }
    
                WM_SHOWWINDOW => {
                    if !(*this_).m_pShadow.is_null() && (*this_).m_bShowShadow != 0 {
                        (*(*this_).m_pShadow).Show(wParam as BOOL);
                    }
                    return (*this_).OnShowWindow(hWnd, WM_SHOWWINDOW, wParam, lParam);
                }
    
                WM_SETTINGCHANGE => {
                    cicUpdateUIFSys();
                    cicUpdateUIFScheme();
                    return (*this_).OnSettingChange(hWnd, WM_SETTINGCHANGE, wParam, lParam);
                }
                WM_SETCURSOR => {
                    let mut Point: POINT = core::mem::zeroed();
                    GetCursorPos(core::ptr::addr_of_mut!(Point));
                    ScreenToClient((*this_).m_hWnd, core::ptr::addr_of_mut!(Point));
    
                    if !(*this_).m_pBehindModal.is_null() {
                        (*(*this_).m_pBehindModal).ModalMouseNotify(HIWORD!(lParam) as UINT, Point.x, Point.y);
                        return (TRUE) as LRESULT;
                    }
    
                    if (*this_).m_bPointing == 0 {
                        SetTimer((*this_).m_hWnd, (POINTING_TIMER_ID) as UINT_PTR, 1000, None);
                        (*this_).m_bPointing = (TRUE) as i32;
                    }
    
                    if !(*this_).m_pToolTip.is_null() {
                        let mut msg: MSG = MSG {
                            hwnd: (*this_).m_hWnd,
                            message: HIWORD!(lParam) as UINT,
                            wParam: 0,
                            lParam: MAKELPARAM!(Point.x as WORD, Point.y as WORD),
                            ..core::mem::zeroed()
                        };
                        (*(*this_).m_pToolTip).RelayEvent(core::ptr::addr_of_mut!(msg));
                    }
    
                    if ((*this_).__base0.m_style & UIF_WINDOW_NOMOUSEMSG) == 0 {
                        (*this_).HandleMouseMsg(HIWORD!(lParam) as UINT, Point.x, Point.y);
                    }
    
                    return (TRUE) as LRESULT;
                }
                WM_MOUSEACTIVATE => {
                    return (MA_NOACTIVATE) as LRESULT;
                }
                WM_ERASEBKGND => {
                    return (*this_).OnEraseBkGnd(hWnd, WM_ERASEBKGND, wParam, lParam) as LRESULT;
                }
                WM_CREATE => {
                    (*this_).SetRect(core::ptr::null_mut());
                    (*this_).OnCreate(hWnd);
                    return 0;
                }
                WM_DESTROY => {
                    if !(*this_).m_pToolTip.is_null() && IsWindow((*(*this_).m_pToolTip).__base0.op_to_HWND()) != 0 {
                        DestroyWindow((*(*this_).m_pToolTip).__base0.op_to_HWND());
                    }
                    if !(*this_).m_pShadow.is_null() && IsWindow((*(*this_).m_pShadow).__base0.op_to_HWND()) != 0 {
                        DestroyWindow((*(*this_).m_pShadow).__base0.op_to_HWND());
                    }
                    (*this_).OnDestroy(hWnd);
                    return 0;
                }
                WM_SIZE => {
                    (*this_).SetRect(core::ptr::null_mut());
                    return 0;
                }
                WM_ACTIVATE => {
                    return (*this_).OnActivate(hWnd, WM_ACTIVATE, wParam, lParam);
                }
                WM_SETFOCUS => {
                    (*this_).OnSetFocus(hWnd);
                    return 0;
                }
                WM_KILLFOCUS => {
                    (*this_).OnKillFocus(hWnd);
                    return 0;
                }
                WM_PAINT => {
                    let mut Paint: PAINTSTRUCT = core::mem::zeroed();
                    let hDC: HDC = BeginPaint(hWnd, core::ptr::addr_of_mut!(Paint));
                    (*this_).PaintObject(hDC, core::ptr::addr_of_mut!(Paint.rcPaint));
                    EndPaint(hWnd, core::ptr::addr_of_mut!(Paint));
                    return 0;
                }
                WM_PRINTCLIENT => {
                    (*this_).PaintObject(wParam as HDC, core::ptr::null_mut());
                    return 0;
                }
                WM_THEMECHANGED => {
                    (*this_).OnThemeChanged(hWnd, wParam, lParam);
                    return 0;
                }
                WM_COMMAND => {
                    return 0;
                }
                WM_TIMER => {
                    'sw2: {
                        match wParam {
                            _ if wParam == USER_TIMER_ID as WPARAM => {
                                if !(*this_).m_pTimerObject.is_null() {
                                    (*(*this_).m_pTimerObject).OnTimer();
                                }
                                break 'sw2;
                            }
                            _ if wParam == POINTING_TIMER_ID as WPARAM => {
                                let mut pt: POINT = core::mem::zeroed();
                                GetCursorPos(core::ptr::addr_of_mut!(pt));
    
                                let mut pt2: POINT = pt;
                                ScreenToClient((*this_).m_hWnd, core::ptr::addr_of_mut!(pt2));
    
                                let mut rc: RECT = core::mem::zeroed();
                                GetWindowRect((*this_).m_hWnd, core::ptr::addr_of_mut!(rc));
    
                                if PtInRect(core::ptr::addr_of_mut!(rc), pt) != 0
                                    && WindowFromPoint(pt) == (*this_).m_hWnd
                                {
                                    (*(*this_).m_pBehindModal).ModalMouseNotify(WM_MOUSEMOVE, pt2.x, pt2.y);
                                } else {
                                    KillTimer((*this_).m_hWnd, (POINTING_TIMER_ID) as UINT_PTR);
                                    (*this_).m_bPointing = (FALSE) as i32;
                                    (*this_).SetObjectPointed(core::ptr::null_mut(), pt2);
                                    (*this_).OnMouseOutFromWindow(pt2.x, pt2.y);
                                }
    
                                if !(*this_).m_pToolTip.is_null() {
                                    let mut msg: MSG = MSG {
                                        hwnd: (*this_).m_hWnd,
                                        message: WM_MOUSEMOVE,
                                        wParam: 0,
                                        lParam: MAKELPARAM!(pt2.x as WORD, pt2.y as WORD),
                                        ..core::mem::zeroed()
                                    };
                                    (*(*this_).m_pToolTip).RelayEvent(core::ptr::addr_of_mut!(msg));
                                }
                                break 'sw2;
                            }
                            _ => {
                                (*this_).OnTimer(wParam);
                                break 'sw2;
                            }
                        }
                    }
                    break 'sw;
                }
                WM_MOUSEMOVE
                | WM_LBUTTONDOWN
                | WM_LBUTTONDBLCLK
                | WM_LBUTTONUP
                | WM_MBUTTONDOWN
                | WM_MBUTTONDBLCLK
                | WM_MBUTTONUP
                | WM_RBUTTONDOWN
                | WM_RBUTTONDBLCLK
                | WM_RBUTTONUP => {
                    if !(*this_).m_pBehindModal.is_null() {
                        (*(*this_).m_pBehindModal).ModalMouseNotify(uMsg, LOWORD!(lParam) as SHORT as LONG, HIWORD!(lParam) as SHORT as LONG);
                    } else {
                        (*this_).HandleMouseMsg(uMsg, LOWORD!(lParam) as SHORT as LONG, HIWORD!(lParam) as SHORT as LONG);
                    }
                    break 'sw;
                }
                WM_KEYUP => {
                    (*this_).OnKeyUp(hWnd, wParam, lParam);
                    break 'sw;
                }
                WM_WINDOWPOSCHANGING => {
                    let pwp: *mut WINDOWPOS = lParam as *mut WINDOWPOS;
                    if !(*this_).m_pShadow.is_null() && ((*pwp).flags & SWP_HIDEWINDOW) != 0 {
                        (*(*this_).m_pShadow).Show((FALSE) as BOOL);
                    }
                    if ((*pwp).flags & SWP_NOZORDER) == 0
                        && (*pwp).hwndInsertAfter == (*(*this_).m_pShadow).__base0.m_hWnd
                    {
                        (*pwp).flags |= SWP_NOZORDER;
                    }
                    (*(*this_).m_pShadow).OnOwnerWndMoved((((*pwp).flags & SWP_NOSIZE) == 0) as BOOL);
                    return (*this_).OnWindowPosChanging(hWnd, WM_WINDOWPOSCHANGING, wParam, lParam);
                }
                WM_WINDOWPOSCHANGED => {
                    let pwp: *mut WINDOWPOS = lParam as *mut WINDOWPOS;
                    if !(*this_).m_pShadow.is_null() {
                        (*(*this_).m_pShadow).OnOwnerWndMoved((((*pwp).flags & SWP_NOSIZE) == 0) as BOOL);
                    }
                    return (*this_).OnWindowPosChanged(hWnd, WM_WINDOWPOSCHANGED, wParam, lParam);
                }
                WM_NOTIFY => {
                    (*this_).OnNotify(hWnd, wParam, lParam);
                    return 0;
                }
                WM_NOTIFYFORMAT => {
                    return (*this_).OnNotifyFormat(hWnd, wParam, lParam);
                }
                WM_DISPLAYCHANGE => {
                    cicUpdateUIFSys();
                    cicUpdateUIFScheme();
                    return (*this_).OnDisplayChange(hWnd, WM_DISPLAYCHANGE, wParam, lParam);
                }
                WM_NCDESTROY => {
                    (*this_).OnNCDestroy(hWnd);
                    return 0;
                }
                WM_KEYDOWN => {
                    (*this_).OnKeyDown(hWnd, wParam, lParam);
                    return 0;
                }
                _ => {
                    if uMsg >= WM_USER {
                        let pThis: *mut CUIFWindow = CUIFWindow::GetThis(hWnd);
                        (*pThis).OnUser(hWnd, uMsg, wParam, lParam);
                        break 'sw;
                    }
                    return DefWindowProc(hWnd, uMsg, wParam, lParam);
                }
            }
        }
    
        return 0;
    }
    pub unsafe fn OnThemeChanged__body(&mut self, hWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        (*this_).__base0.ClearTheme();
    }
    pub unsafe fn UpdateUI__body(&mut self, prc: LPCRECT) {
        let this_: *mut Self = self;
        if IsWindow((*this_).m_hWnd) != 0 {
            InvalidateRect((*this_).m_hWnd, prc, (FALSE) as BOOL);
        }
    }
    pub unsafe fn SetCapture__body(&mut self, bSet: i32) {
        let this_: *mut Self = self;
        if bSet != 0 {
            SetCapture((*this_).m_hWnd);
        } else {
            ReleaseCapture();
        }
    }
    pub unsafe fn OnAnimationEnd__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pShadow.is_null() && (*this_).m_bShowShadow != 0 {
            (*(*this_).m_pShadow).Show((*this_).__base0.m_bVisible);
        }
    }
    pub unsafe fn HandleMouseMsg__body(&mut self, uMsg: UINT, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: x, y: y };
    
        let mut pFound: *mut CUIFObject = (*this_).__base0.ObjectFromPoint(pt);
        let _ = &mut pFound;
    
        (*this_).SetObjectPointed(pFound, pt);
    
        if !(*this_).m_pCaptured.is_null() {
            pFound = (*this_).m_pCaptured;
        }
    
        if pFound.is_null() || (*this_).__base0.OnSetCursor(uMsg, pt.x, pt.y) != 0 {
            let hCursor: HCURSOR = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
            SetCursor(hCursor);
        }
    
        if !pFound.is_null() && (*pFound).m_bEnable != 0 {
            match uMsg {
                WM_MOUSEMOVE => {
                    (*pFound).OnMouseMove(pt.x, pt.y);
                }
                WM_LBUTTONDOWN => {
                    (*pFound).OnLButtonDown(pt.x, pt.y);
                }
                WM_LBUTTONUP => {
                    (*pFound).OnLButtonUp(pt.x, pt.y);
                }
                WM_RBUTTONDOWN => {
                    (*pFound).OnRButtonDown(pt.x, pt.y);
                }
                WM_RBUTTONUP => {
                    (*pFound).OnRButtonUp(pt.x, pt.y);
                }
                WM_MBUTTONDOWN => {
                    (*pFound).OnMButtonDown(pt.x, pt.y);
                }
                WM_MBUTTONUP => {
                    (*pFound).OnMButtonUp(pt.x, pt.y);
                }
                _ => {}
            }
        }
    }
    pub unsafe fn ClientRectToWindowRect__body(&mut self, lpRect: LPRECT) {
        let this_: *mut Self = self;
        let mut style: DWORD;
        let mut exstyle: DWORD;
        if IsWindow((*this_).m_hWnd) != 0 {
            style = GetWindowLongPtr((*this_).m_hWnd, GWL_STYLE) as DWORD;
            exstyle = GetWindowLongPtr((*this_).m_hWnd, GWL_EXSTYLE) as DWORD;
        } else {
            style = (*this_).GetWndStyle();
            exstyle = (*this_).GetWndStyleEx();
        }
        AdjustWindowRectEx(lpRect, style, (FALSE) as BOOL, exstyle);
    }
    pub unsafe fn GetWindowFrameSize__body(&mut self, pSize: LPSIZE) {
        let this_: *mut Self = self;
        let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    
        (*this_).ClientRectToWindowRect(core::ptr::addr_of_mut!(rc));
        (*pSize).cx = (rc.right - rc.left) / 2;
        (*pSize).cy = (rc.bottom - rc.top) / 2;
    }
}

impl CUIFToolTip {
    pub unsafe fn __ctor_0(&mut self, hInst: HINSTANCE, style: DWORD, pToolTipOwner: *mut CUIFWindow) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(hInst, style);
        (*this_).__init_vtables();
        {
            (*this_).m_pToolTipOwner = pToolTipOwner;
            (*this_).m_rcToolTipMargin.left = 2;
            (*this_).m_rcToolTipMargin.top = 2;
            (*this_).m_rcToolTipMargin.right = 2;
            (*this_).m_rcToolTipMargin.bottom = 2;
            (*this_).m_pToolTipTarget = core::ptr::null_mut();
            (*this_).m_pszToolTipText = core::ptr::null_mut();
            (*this_).m_dwUnknown10 = 0; //FIXME: name and type
            (*this_).m_nDelayTimeType2 = -1;
            (*this_).m_nDelayTimeType3 = -1;
            (*this_).m_nDelayTimeType1 = -1;
            (*this_).m_cxToolTipWidth = -1;
            (*this_).m_bToolTipHasBkColor = 0;
            (*this_).m_bToolTipHasTextColor = 0;
            (*this_).m_rgbToolTipBkColor = 0;
            (*this_).m_rgbToolTipTextColor = 0;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pToolTipOwner.is_null() {
            (*(*this_).m_pToolTipOwner).m_pToolTip = core::ptr::null_mut();
        }
        if !(*this_).m_pszToolTipText.is_null() {
            cpp_delete_array((*this_).m_pszToolTipText);
        }
    }
    pub unsafe fn GetDelayTime(&mut self, uType: UINT) -> LONG {
        let this_: *mut Self = self;
        let mut nDelayTime: LONG;
        match uType {
            1 => {
                nDelayTime = (*this_).m_nDelayTimeType1;
                if nDelayTime == -1 {
                    return (GetDoubleClickTime() / 5) as LONG;
                }
                return nDelayTime;
            }
            2 => {
                nDelayTime = (*this_).m_nDelayTimeType2;
                if nDelayTime == -1 {
                    return (10 * GetDoubleClickTime()) as LONG;
                }
                return nDelayTime;
            }
            3 => {
                nDelayTime = (*this_).m_nDelayTimeType3;
                if nDelayTime == -1 {
                    return (GetDoubleClickTime()) as LONG;
                }
                return nDelayTime;
            }
            _ => {
                return 0;
            }
        }
    }
    pub unsafe fn GetMargin(&mut self, prc: LPRECT) {
        let this_: *mut Self = self;
        if !prc.is_null() {
            *prc = (*this_).m_rcToolTipMargin;
        }
    }
    pub unsafe fn GetTipBkColor(&mut self) -> COLORREF {
        let this_: *mut Self = self;
        if (*this_).m_bToolTipHasBkColor != 0 {
            return (*this_).m_rgbToolTipBkColor;
        }
        return GetSysColor((COLOR_INFOBK as i32));
    }
    pub unsafe fn GetTipTextColor(&mut self) -> COLORREF {
        let this_: *mut Self = self;
        if (*this_).m_bToolTipHasTextColor != 0 {
            return (*this_).m_rgbToolTipTextColor;
        }
        return GetSysColor((COLOR_INFOTEXT as i32));
    }
    pub unsafe fn FindObject(&mut self, hWnd: HWND, pt: POINT) -> *mut CUIFObject {
        let this_: *mut Self = self;
        if hWnd == (*(*this_).m_pToolTipOwner).op_to_HWND() {
            return (*(*this_).m_pToolTipOwner).__base0.ObjectFromPoint(pt);
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn ShowTip(&mut self) {
        let this_: *mut Self = self;
        KillTimer((*this_).__base0.m_hWnd, (TOOLTIP_TIMER_ID) as UINT_PTR);
    
        if (*this_).m_pToolTipTarget.is_null() {
            return;
        }
    
        let pszText: LPCWSTR = (*(*this_).m_pToolTipTarget).GetToolTip();
        if pszText.is_null() {
            return;
        }
    
        if (*this_).m_pToolTipTarget.is_null() || (*(*this_).m_pToolTipTarget).OnShowToolTip() != 0 {
            return;
        }
    
        let mut Point: POINT = core::mem::zeroed();
        GetCursorPos(core::ptr::addr_of_mut!(Point));
        ScreenToClient(
            (*(*(*this_).m_pToolTipTarget).m_pWindow).op_to_HWND(),
            core::ptr::addr_of_mut!(Point),
        );
    
        let mut rc: RECT = core::mem::zeroed();
        (*(*this_).m_pToolTipTarget).GetRect(core::ptr::addr_of_mut!(rc));
        if PtInRect(core::ptr::addr_of_mut!(rc), Point) == 0 {
            return;
        }
    
        let cchText: usize = wcslen(pszText);
        (*this_).m_pszToolTipText = cpp_new_array_CicNoThrow::<WCHAR>(cchText + 1);
        if (*this_).m_pszToolTipText.is_null() {
            return;
        }
    
        lstrcpynW(
            (*this_).m_pszToolTipText,
            pszText,
            (cchText + 1) as i32,
        );
    
        let mut size: SIZE = core::mem::zeroed();
        (*this_).GetTipWindowSize(core::ptr::addr_of_mut!(size));
    
        let mut rc2: RECT = rc;
        ClientToScreen(
            (*(*(*this_).m_pToolTipTarget).m_pWindow).op_to_HWND(),
            core::ptr::addr_of_mut!(rc) as LPPOINT,
        );
        ClientToScreen(
            (*(*(*this_).m_pToolTipTarget).m_pWindow).op_to_HWND(),
            core::ptr::addr_of_mut!(rc.right) as LPPOINT,
        );
        (*this_).GetTipWindowRect(core::ptr::addr_of_mut!(rc2), size, core::ptr::addr_of_mut!(rc));
    
        (*this_).m_bShowToolTip = (TRUE) as i32;
        (*this_).__base0.Move(rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top);
        (*this_).__base0.Show((TRUE) as BOOL);
    }
    pub unsafe fn HideTip(&mut self) {
        let this_: *mut Self = self;
        KillTimer((*this_).__base0.m_hWnd, (TOOLTIP_TIMER_ID) as UINT_PTR);
        (*this_).m_bShowToolTip = (FALSE) as i32;
    
        if !(*this_).m_pToolTipTarget.is_null() {
            (*(*this_).m_pToolTipTarget).OnHideToolTip();
        }
    
        if (*this_).__base0.__base0.m_bVisible != 0 {
            if !(*this_).m_pszToolTipText.is_null() {
                cpp_delete_array((*this_).m_pszToolTipText);
                (*this_).m_pszToolTipText = core::ptr::null_mut();
            }
            (*this_).__base0.Show((FALSE) as BOOL);
        }
    }
    pub unsafe fn GetTipWindowSize(&mut self, pSize: LPSIZE) {
        let this_: *mut Self = self;
        if (*this_).m_pszToolTipText.is_null() {
            return;
        }
    
        let hDC: HDC = GetDC((*this_).__base0.m_hWnd);
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.__base0.m_hFont).cast());
    
        let mut rcText: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut cyText: INT;
        if (*this_).m_cxToolTipWidth <= 0 {
            cyText = DrawTextW(
                hDC,
                (*this_).m_pszToolTipText,
                -(1),
                core::ptr::addr_of_mut!(rcText),
                (DT_CALCRECT | DT_SINGLELINE),
            );
        } else {
            rcText.right = (*this_).m_cxToolTipWidth;
            cyText = DrawTextW(
                hDC,
                (*this_).m_pszToolTipText,
                -(1),
                core::ptr::addr_of_mut!(rcText),
                (DT_CALCRECT | DT_WORDBREAK),
            );
        }
    
        let mut rcMargin: RECT = core::mem::zeroed();
        (*this_).GetMargin(core::ptr::addr_of_mut!(rcMargin));
    
        let mut rc: RECT = core::mem::zeroed();
        rc.left = rcText.left - rcMargin.left;
        rc.top = rcText.top - rcMargin.top;
        rc.right = rcText.right + rcMargin.right;
        rc.bottom = rcText.top + cyText + rcMargin.bottom;
        (*this_).__base0.ClientRectToWindowRect(core::ptr::addr_of_mut!(rc));
    
        (*pSize).cx = rc.right - rc.left;
        (*pSize).cy = rc.bottom - rc.top;
    
        SelectObject(hDC, hFontOld);
        ReleaseDC((*this_).__base0.m_hWnd, hDC);
    }
    pub unsafe fn GetTipWindowRect(&mut self, pRect: LPRECT, toolTipSize: SIZE, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut Point: POINT = core::mem::zeroed();
        GetCursorPos(core::ptr::addr_of_mut!(Point));
    
        let hCursor: HCURSOR = GetCursor();
        let mut IconInfo: ICONINFO = core::mem::zeroed();
        let mut yHotspot: INT = 0;
        let mut cyCursor: INT = GetSystemMetrics((SM_CYCURSOR as i32));
        if !hCursor.is_null() && GetIconInfo(hCursor, core::ptr::addr_of_mut!(IconInfo)) != 0 {
            let mut bm: BITMAP = core::mem::zeroed();
            GetObject((IconInfo.hbmMask).cast(), core::mem::size_of::<BITMAP>() as INT, core::ptr::addr_of_mut!(bm) as LPVOID);
            if IconInfo.fIcon == 0 {
                cyCursor = bm.bmHeight;
                yHotspot = (IconInfo.yHotspot) as i32;
                if IconInfo.hbmColor.is_null() {
                    cyCursor = bm.bmHeight / 2;
                }
            }
            if !IconInfo.hbmColor.is_null() {
                DeleteObject((IconInfo.hbmColor).cast());
            }
            if !IconInfo.hbmMask.is_null() {
                DeleteObject((IconInfo.hbmMask).cast());
            }
        }
    
        let mut rcMonitor: RECT = core::mem::zeroed();
        rcMonitor.left = 0;
        rcMonitor.top = 0;
        rcMonitor.right = GetSystemMetrics((SM_CXSCREEN as i32));
        rcMonitor.bottom = GetSystemMetrics((SM_CYSCREEN as i32));
    
        let hMon: HMONITOR = MonitorFromPoint(Point, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = core::mem::zeroed();
        if !hMon.is_null() {
            mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
            if GetMonitorInfo(hMon, core::ptr::addr_of_mut!(mi)) != 0 {
                rcMonitor = mi.rcMonitor;
            }
        }
    
        (*pRect).left = Point.x;
        (*pRect).right = (*pRect).left + toolTipSize.cx;
        (*pRect).top = Point.y + cyCursor - yHotspot;
        (*pRect).bottom = (*pRect).top + toolTipSize.cy;
    
        if rcMonitor.right < (*pRect).right {
            (*pRect).left = rcMonitor.right - toolTipSize.cx;
            (*pRect).right = rcMonitor.right;
        }
        if (*pRect).left < rcMonitor.left {
            (*pRect).left = rcMonitor.left;
            (*pRect).right = rcMonitor.left + toolTipSize.cx;
        }
        if rcMonitor.bottom < (*pRect).bottom {
            (*pRect).top = rcMonitor.bottom - toolTipSize.cy;
            (*pRect).bottom = rcMonitor.bottom;
        }
        if (*pRect).top < rcMonitor.top {
            (*pRect).top = rcMonitor.top;
            (*pRect).bottom = rcMonitor.top + toolTipSize.cy;
        }
    }
    pub unsafe fn RelayEvent(&mut self, pMsg: LPMSG) {
        let this_: *mut Self = self;
        if pMsg.is_null() {
            return;
        }
    
        match (*pMsg).message {
            WM_MOUSEMOVE => {
                if (*this_).__base0.__base0.m_bEnable != 0
                    && GetKeyState((VK_LBUTTON as i32)) >= 0
                    && GetKeyState((VK_MBUTTON as i32)) >= 0
                    && GetKeyState((VK_RBUTTON as i32)) >= 0
                {
                    let pt: POINT = POINT {
                        x: LOWORD!((*pMsg).lParam) as SHORT as LONG,
                        y: HIWORD!((*pMsg).lParam) as SHORT as LONG,
                    };
                    let pFound: *mut CUIFObject = (*this_).FindObject((*pMsg).hwnd, pt);
                    if !pFound.is_null() {
                        if (*this_).m_pToolTipTarget != pFound {
                            (*this_).HideTip();
    
                            let DelayTime: LONG;
                            if (*this_).__base0.__base0.m_bVisible == 0 {
                                DelayTime = (*this_).GetDelayTime(3);
                            } else {
                                DelayTime = (*this_).GetDelayTime(1);
                            }
                            SetTimer(
                                (*this_).__base0.m_hWnd,
                                (TOOLTIP_TIMER_ID) as UINT_PTR,
                                DelayTime as UINT,
                                None,
                            );
                        }
                    } else {
                        (*this_).HideTip();
                    }
                    (*this_).m_pToolTipTarget = pFound;
                }
            }
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                (*this_).HideTip();
            }
            _ => {}
        }
    }
    pub unsafe fn OnPaint__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.__base0.m_hFont).cast());
        let iBkModeOld: INT = SetBkMode(hDC, (TRANSPARENT as i32));
    
        let rgbTextColor: COLORREF = (*this_).GetTipTextColor();
        let rgbOldTextColor: COLORREF = SetTextColor(hDC, rgbTextColor);
    
        let rgbBkColor: COLORREF = (*this_).GetTipBkColor();
        let hbrBack: HBRUSH = CreateSolidBrush(rgbBkColor);
        let mut rc: RECT = (*this_).__base0.__base0.m_rc;
        if !hbrBack.is_null() {
            FillRect(hDC, core::ptr::addr_of_mut!(rc), hbrBack);
            DeleteObject((hbrBack).cast());
        }
    
        let mut rcMargin: RECT = core::mem::zeroed();
        (*this_).GetMargin(core::ptr::addr_of_mut!(rcMargin));
    
        rc.left += rcMargin.left;
        rc.top += rcMargin.top;
        rc.right -= rcMargin.right;
        rc.bottom -= rcMargin.bottom;
    
        if (*this_).m_cxToolTipWidth <= 0 {
            DrawTextW(hDC, (*this_).m_pszToolTipText, -1, core::ptr::addr_of_mut!(rc), DT_SINGLELINE);
        } else {
            DrawTextW(hDC, (*this_).m_pszToolTipText, -1, core::ptr::addr_of_mut!(rc), DT_WORDBREAK);
        }
    
        SetTextColor(hDC, rgbOldTextColor);
        SetBkMode(hDC, iBkModeOld);
        SelectObject(hDC, hFontOld);
    }
    pub unsafe fn Enable__body(&mut self, bEnable: BOOL) {
        let this_: *mut Self = self;
        if !(bEnable != 0) {
            (*this_).HideTip();
        }
        (*this_).__base0.__base0.Enable(bEnable);
    }
    pub unsafe fn OnTimer__body(&mut self, wParam: WPARAM) {
        let this_: *mut Self = self;
        if wParam == (TOOLTIP_TIMER_ID) as _ {
            (*this_).ShowTip();
        }
    }
}

impl CUIFShadow {
    pub unsafe fn __ctor_0(&mut self, hInst: HINSTANCE, style: DWORD, pShadowOwner: *mut CUIFWindow) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(hInst, (style | UIF_WINDOW_TOOLWINDOW));
        (*this_).__init_vtables();
        {
        (*this_).m_pShadowOwner = pShadowOwner;
        (*this_).m_rgbShadowColor = RGB!(0, 0, 0);
        (*this_).m_dwUnknown11[0 as usize] = 0;
        (*this_).m_dwUnknown11[1 as usize] = 0;
        (*this_).m_yShadowDelta = 0;
        (*this_).m_xShadowDelta = 0;
        (*this_).m_bLayerAvailable = (FALSE) as i32;
    }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pShadowOwner.is_null() {
            (*(*this_).m_pShadowOwner).m_pShadow = core::ptr::null_mut();
        }
    }
    pub unsafe fn InitSettings(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_bLayerAvailable = (FALSE) as i32;
        (*this_).m_rgbShadowColor = RGB!(128, 128, 128);
        (*this_).m_yShadowDelta = 2;
        (*this_).m_xShadowDelta = (*this_).m_yShadowDelta;
    }
    pub unsafe fn InitShadow(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bLayerAvailable != 0 {
            // FIXME
        }
    }
    pub unsafe fn AdjustWindowPos(&mut self) {
        let this_: *mut Self = self;
        let hwndOwner: HWND = (*(*this_).m_pShadowOwner).op_to_HWND();
        if IsWindow((*this_).__base0.m_hWnd) == 0 {
            return;
        }
    
        let mut rc: RECT = core::mem::zeroed();
        GetWindowRect(hwndOwner, core::ptr::addr_of_mut!(rc));
        SetWindowPos(
            (*this_).__base0.m_hWnd,
            hwndOwner,
            rc.left + (*this_).m_xShadowDelta,
            rc.top + (*this_).m_yShadowDelta,
            rc.right - rc.left,
            rc.bottom - rc.top,
            SWP_NOOWNERZORDER | SWP_NOACTIVATE,
        );
    }
    pub unsafe fn OnOwnerWndMoved(&mut self, bDoSize: BOOL) {
        let this_: *mut Self = self;
        if IsWindow((*this_).__base0.m_hWnd) != 0 && IsWindowVisible((*this_).__base0.m_hWnd) != 0 {
            (*this_).AdjustWindowPos();
            if bDoSize != 0 {
                (*this_).InitShadow();
            }
        }
    }
    pub unsafe fn Initialize__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).InitSettings();
        return (*this_).__base0.Initialize();
    }
    pub unsafe fn GetWndStyleEx__body(&mut self) -> DWORD {
        let this_: *mut Self = self;
        let mut exstyle: DWORD = (*this_).__base0.GetWndStyleEx();
        if (*this_).m_bLayerAvailable != 0 {
            exstyle |= WS_EX_LAYERED;
        }
        return exstyle;
    }
    pub unsafe fn OnPaint__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        let mut rc: RECT = (*this_).__base0.__base0.m_rc;
        let mut hBrush: HBRUSH = CreateSolidBrush((*this_).m_rgbShadowColor);
        FillRect(hDC, core::ptr::addr_of_mut!(rc), hBrush);
        DeleteObject((hBrush).cast());
    }
    pub unsafe fn OnWindowPosChanging__body(&mut self, hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        let wp: *mut WINDOWPOS = lParam as *mut WINDOWPOS;
        (*wp).hwndInsertAfter = (*(*this_).m_pShadowOwner).op_to_HWND();
        return DefWindowProc(hWnd, Msg, wParam, lParam);
    }
    pub unsafe fn OnSettingChange__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).InitSettings();
    
        let mut exstyle: DWORD;
        if (*this_).m_bLayerAvailable != 0 {
            exstyle = GetWindowLongPtr((*this_).__base0.m_hWnd, GWL_EXSTYLE) as DWORD | WS_EX_LAYERED;
        } else {
            exstyle = GetWindowLongPtr((*this_).__base0.m_hWnd, GWL_EXSTYLE) as DWORD & !WS_EX_LAYERED;
        }
    
        SetWindowLongPtr((*this_).__base0.m_hWnd, GWL_EXSTYLE, (exstyle) as _);
    
        (*this_).AdjustWindowPos();
        (*this_).InitShadow();
    
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn Show__body(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        if bVisible != 0 && IsWindow((*this_).__base0.m_hWnd) != 0 && !(IsWindowVisible((*this_).__base0.m_hWnd) != 0) {
            (*this_).AdjustWindowPos();
            (*this_).InitShadow();
        }
    
        if IsWindow((*this_).__base0.m_hWnd) != 0 {
            (*this_).__base0.__base0.m_bVisible = bVisible;
            ShowWindow((*this_).__base0.m_hWnd, if bVisible != 0 { (SW_SHOWNOACTIVATE as i32) } else { (SW_HIDE as i32) });
        }
    }
}

impl CUIFMenu {
    pub unsafe fn __ctor_0(&mut self, hInst: HINSTANCE, style: DWORD, dwUnknown14: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(hInst, style);
        (*this_).__init_vtables();
        {
            (*this_).m_nSelectedID = (-(1 as i32)) as UINT;
            (*this_).m_dwUnknown14 = dwUnknown14;
            (*this_).SetMenuFont();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_MenuItems.size() {
            CUIFMenuItem__delete((*this_).m_MenuItems[iItem]);
            iItem += 1;
        }
    
        DeleteObject(((*this_).m_hMenuFont).cast());
        (*this_).ClearMenuFont();
    }
    pub unsafe fn CancelMenu(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pVisibleSubMenu.is_null() {
            (*this_).UninitShow();
        } else if (*this_).m_bModal != 0 {
            (*this_).SetSelectedId(0xFFFFFFFF);
            PostMessage((*this_).__base0.m_hWnd, WM_NULL, 0, 0);
        }
    }
    pub unsafe fn ClearMenuFont(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__base0.SetFont(core::ptr::null_mut());
        DeleteObject(((*this_).__base0.__base0.m_hFont).cast());
    }
    pub unsafe fn GetNextItem(&mut self, pItem: *mut CUIFMenuItem) -> *mut CUIFMenuItem {
        let this_: *mut Self = self;
        let mut iItem: usize;
        let cItems: usize = (*this_).m_MenuItems.size();
    
        if cItems == 0 {
            return core::ptr::null_mut();
        }
    
        if (*this_).m_pSelectedItem.is_null() {
            return (*this_).m_MenuItems[0];
        }
    
        iItem = 0;
        while iItem < cItems {
            let cur: usize = iItem;
            iItem += 1;
            if (*this_).m_MenuItems[cur] == pItem {
                break;
            }
        }
    
        loop {
            if iItem == cItems {
                iItem = 0;
            }
            if (*this_).m_MenuItems[iItem].is_null() || (*(*this_).m_MenuItems[iItem]).m_bMenuItemDisabled == 0 {
                break;
            }
            iItem += 1;
        }
    
        return (*this_).m_MenuItems[iItem];
    }
    pub unsafe fn GetPrevItem(&mut self, pItem: *mut CUIFMenuItem) -> *mut CUIFMenuItem {
        let this_: *mut Self = self;
        let mut iItem: isize;
        let cItems: isize = (*this_).m_MenuItems.size() as isize;
    
        if cItems == 0 {
            return core::ptr::null_mut();
        }
    
        if (*this_).m_pSelectedItem.is_null() {
            return (*this_).m_MenuItems[(cItems - 1) as usize];
        }
    
        iItem = cItems - 1;
        while iItem >= 0 {
            let cur: isize = iItem;
            iItem -= 1;
            if (*this_).m_MenuItems[cur as usize] == pItem {
                break;
            }
        }
    
        loop {
            if iItem < 0 {
                iItem = cItems - 1;
            }
            let it: *mut CUIFMenuItem = (*this_).m_MenuItems[iItem as usize];
            if it.is_null() || (*it).m_bMenuItemDisabled == 0 {
                break;
            }
            iItem -= 1;
        }
    
        (*this_).m_MenuItems[iItem as usize]
    }
    pub unsafe fn GetTopSubMenu(&mut self) -> *mut CUIFMenu {
        let this_: *mut Self = self;
        let mut pMenu: *mut CUIFMenu;
        pMenu = this_;
        while !(*pMenu).m_pParentMenu.is_null() {
            pMenu = (*pMenu).m_pParentMenu;
        }
        return pMenu;
    }
    pub unsafe fn InsertItem(&mut self, pItem: *mut CUIFMenuItem) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_MenuItems.Add(pItem) == 0 {
            return (FALSE) as BOOL;
        }
    
        (*pItem).__base0.SetFont((*this_).__base0.__base0.m_hFont);
        return (TRUE) as BOOL;
    }
    pub unsafe fn InsertSeparator(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let pSep: *mut CUIFMenuItemSeparator = CUIFMenuItemSeparator__new_in_0(cpp_alloc_CicNoThrow::<CUIFMenuItemSeparator>(), this_);
        if pSep.is_null() {
            return (FALSE) as BOOL;
        }
    
        if (*this_).m_MenuItems.Add(core::ptr::addr_of_mut!((*pSep).__base0)) == 0 {
            CUIFMenuItemSeparator__delete(pSep);
            return (FALSE) as BOOL;
        }
    
        (*pSep).__base0.__base0.Initialize();
        return (TRUE) as BOOL;
    }
    pub unsafe fn PostKey(&mut self, bUp: BOOL, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        if (*this_).m_bModal != 0 {
            // NOTE: hWnd parameter will be populated in CUIFMenu::ModalMessageLoop.
            if bUp != 0 {
                PostMessage(core::ptr::null_mut(), WM_KEYUP, wParam, lParam);
            } else {
                PostMessage(core::ptr::null_mut(), WM_KEYDOWN, wParam, lParam);
            }
        }
    }
    pub unsafe fn SetMenuFont(&mut self) {
        let this_: *mut Self = self;
        let mut height: LONG = 14;
    
        let mut ncm: NONCLIENTMETRICS = core::mem::zeroed();
        ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICS>() as UINT;
        if SystemParametersInfo(SPI_GETNONCLIENTMETRICS, 0, core::ptr::addr_of_mut!(ncm) as LPVOID, 0) != 0
        {
            let hFont: HFONT = CreateFontIndirect(core::ptr::addr_of!(ncm.lfMenuFont));
            (*this_).__base0.__base0.SetFont(hFont);
    
            let mut lfHeight: LONG = ncm.lfMenuFont.lfHeight;
            if lfHeight < 0 {
                lfHeight = -lfHeight;
            }
            height = (ncm.iMenuHeight + lfHeight) / 2;
        }
    
        (*this_).m_hMenuFont = CreateFontW(height, 0, 0, 0, (FW_NORMAL as i32), 0, 0, 0, SYMBOL_CHARSET,
                                    0, 0, 0, 0, CPP_COM_WSTR_B728528EE8A8.as_ptr());
        let cxSmallIcon: INT = GetSystemMetrics((SM_CXSMICON as i32));
        (*this_).m_cxyMargin = max!(height, cxSmallIcon) + 2;
    }
    pub unsafe fn SetSelectedId(&mut self, nSelectID: UINT) {
        let this_: *mut Self = self;
        let mut pMenu: *mut CUIFMenu = this_;
    
        while !(*pMenu).m_pVisibleSubMenu.is_null() {
            pMenu = (*pMenu).m_pVisibleSubMenu;
        }
    
        (*pMenu).m_nSelectedID = nSelectID;
    }
    pub unsafe fn SetSelectedItem(&mut self, pItem: *mut CUIFMenuItem) {
        let this_: *mut Self = self;
        let pOldItem: *mut CUIFMenuItem = (*this_).m_pSelectedItem;
        if pOldItem == pItem {
            return;
        }
    
        (*this_).m_pSelectedItem = pItem;
        if !pOldItem.is_null() {
            (*pOldItem).__base0.CallOnPaint();
        }
        if !(*this_).m_pSelectedItem.is_null() {
            (*(*this_).m_pSelectedItem).__base0.CallOnPaint();
        }
    }
    pub unsafe fn ShowModalPopup(&mut self, pWindow: *mut CUIFWindow, prc: LPCRECT, bFlag: BOOL) -> UINT {
        let this_: *mut Self = self;
        let mut pCaptured: *mut CUIFObject = core::ptr::null_mut();
        if !pWindow.is_null() {
            pCaptured = (*pWindow).m_pCaptured;
            (*this_).__base0.SetCaptureObject(core::ptr::null_mut());
        }
    
        let mut nSelectedID: UINT = (1 as UINT).wrapping_neg();
        if (*this_).InitShow(pWindow, prc, bFlag, (TRUE) as BOOL) != 0 {
            (*this_).m_bModal = (TRUE) as i32;
            (*pWindow).SetBehindModal(core::ptr::addr_of_mut!((*this_).__base0));
            (*this_).ModalMessageLoop();
            nSelectedID = (*this_).m_nSelectedID;
            (*pWindow).SetBehindModal(core::ptr::null_mut());
            (*this_).m_bModal = (FALSE) as i32;
        }
    
        (*this_).UninitShow();
    
        if !pWindow.is_null() {
            (*pWindow).SetCaptureObject(pCaptured);
        }
    
        return nSelectedID;
    }
    pub unsafe fn ShowSubPopup(&mut self, pSubMenu: *mut CUIFMenu, prc: LPCRECT, bFlag: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_pVisibleSubMenu = pSubMenu;
        (*this_).InitShow(core::ptr::addr_of_mut!((*pSubMenu).__base0), prc, bFlag, (TRUE) as BOOL);
    }
    pub unsafe fn OnKeyDown__body(&mut self, hWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        let mut pTargetItem: *mut CUIFMenuItem = core::ptr::null_mut();
    
        let vKey: BYTE = wParam as BYTE;
    
        match (vKey) as u32 {
            VK_ESCAPE => {
                (*this_).CancelMenu();
                return;
            }
    
            VK_LEFT => {
                if (*this_).m_pVisibleSubMenu.is_null() {
                    return;
                }
    
                (*this_).CancelMenu();
                return;
            }
    
            VK_RIGHT => {
                if !(*this_).m_pSelectedItem.is_null() && !(*(*this_).m_pSelectedItem).m_pSubMenu.is_null() {
                    (*(*this_).m_pSelectedItem).ShowSubPopup();
                    let pSubMenu: *mut CUIFMenu = (*(*this_).m_pSelectedItem).m_pSubMenu;
                    pTargetItem = (*pSubMenu).GetNextItem(core::ptr::null_mut());
                    (*pSubMenu).SetSelectedItem(pTargetItem);
                }
                return;
            }
    
            VK_UP => {
                pTargetItem = (*this_).GetPrevItem((*this_).m_pSelectedItem);
                (*this_).SetSelectedItem(pTargetItem);
                return;
            }
    
            VK_DOWN => {
                pTargetItem = (*this_).GetNextItem((*this_).m_pSelectedItem);
                (*this_).SetSelectedItem(pTargetItem);
                return;
            }
    
            VK_RETURN => {
                // break
            }
    
            _ => {
                if !((b'A' <= vKey && vKey <= b'Z') || (b'0' <= vKey && vKey <= b'9')) {
                    return;
                }
    
                let mut iItem: usize = 0;
                while iItem < (*this_).m_MenuItems.size() {
                    let pItem: *mut CUIFMenuItem = (*this_).m_MenuItems[iItem];
                    if (*pItem).m_nMenuItemVKey == vKey as UINT {
                        (*this_).SetSelectedItem(pItem);
                        break;
                    }
                    iItem += 1;
                }
    
                if iItem == (*this_).m_MenuItems.size() {
                    return;
                }
            }
        }
    
        if !(*this_).m_pSelectedItem.is_null() && (*(*this_).m_pSelectedItem).m_bMenuItemGrayed == 0 {
            let pSubMenu: *mut CUIFMenu = (*(*this_).m_pSelectedItem).m_pSubMenu;
            if !pSubMenu.is_null() {
                (*(*this_).m_pSelectedItem).ShowSubPopup();
                pTargetItem = (*pSubMenu).GetNextItem(core::ptr::null_mut());
                (*pSubMenu).SetSelectedItem(pTargetItem);
            } else {
                (*this_).SetSelectedId((*(*this_).m_pSelectedItem).m_nMenuItemID);
                PostMessage((*this_).__base0.m_hWnd, WM_NULL, 0, 0);
            }
        }
    }
    pub unsafe fn HandleMouseMsg__body(&mut self, uMsg: UINT, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        let mut pt = POINT { x: x, y: y };
        if PtInRect(core::ptr::addr_of_mut!((*this_).__base0.__base0.m_rc), pt) == 0
            && (uMsg == WM_LBUTTONDOWN
                || uMsg == WM_RBUTTONDOWN
                || uMsg == WM_LBUTTONUP
                || uMsg == WM_RBUTTONUP)
        {
            (*this_).SetSelectedId((1 as u32).wrapping_neg());
            PostMessage((*this_).__base0.m_hWnd, WM_NULL, 0, 0);
        }
        (*this_).__base0.HandleMouseMsg(uMsg, x, y);
    }
    pub unsafe fn ModalMouseNotify__body(&mut self, uMsg: UINT, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if uMsg == WM_LBUTTONDOWN || uMsg == WM_RBUTTONDOWN {
            (*this_).CancelMenu();
        }
    }
    pub unsafe fn ModalMessageLoop__body(&mut self) {
        let this_: *mut Self = self;
        let mut msg: MSG = core::mem::zeroed();
        while GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 && msg.message != WM_NULL &&
              (msg.hwnd == (*this_).__base0.m_hWnd || msg.message <= WM_MOUSEFIRST || WM_MOUSELAST <= msg.message)
        {
            if msg.hwnd.is_null() && WM_KEYFIRST <= msg.message && msg.message <= WM_KEYLAST {
                msg.hwnd = (*(*this_).GetTopSubMenu()).__base0.m_hWnd;
            }
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }
    pub unsafe fn InitShow__body(&mut self, pWindow: *mut CUIFWindow, prc: LPCRECT, bFlag: BOOL, bDoAnimation: BOOL) -> BOOL {
        let this_: *mut Self = self;
        let mut hWnd: HWND = core::ptr::null_mut();
        if !pWindow.is_null() {
            hWnd = (*pWindow).op_to_HWND();
        }
    
        (*this_).__base0.CreateWnd(hWnd);
    
        (*this_).m_bHasMargin = (FALSE) as i32;
    
        let mut iItem: usize = 0;
        while iItem < (*this_).m_MenuItems.size() {
            if !(*this_).m_MenuItems[iItem].is_null() {
                (*(*this_).m_MenuItems[iItem]).InitMenuExtent();
            }
            iItem += 1;
        }
    
        let mut cxMax: INT = 0;
    
        (*this_).m_cxMenuExtent = 0;
        iItem = 0;
        while iItem < (*this_).m_MenuItems.size() {
            'body: {
                let mut pItem: *mut CUIFMenuItem = (*this_).m_MenuItems[iItem];
                if pItem.is_null() {
                    break 'body;
                }
    
                let cxItem: INT = (*this_).m_cxyMargin + (*pItem).m_MenuLeftExtent.cx;
                if cxMax < cxItem {
                    cxMax = cxItem;
                }
                (*this_).m_cxMenuExtent = max!((*this_).m_cxMenuExtent, (*pItem).m_MenuRightExtent.cx);
                if (*this_).m_bHasMargin == 0 && !(*pItem).m_hbmColor.is_null() && (*pItem).IsCheck() != 0 {
                    (*this_).m_bHasMargin = (TRUE) as i32;
                }
            }
            iItem += 1;
        }
    
        let mut rcItem: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        iItem = 0;
        while iItem < (*this_).m_MenuItems.size() {
            'body: {
                let mut pItem: *mut CUIFMenuItem = (*this_).m_MenuItems[iItem];
                if pItem.is_null() {
                    break 'body;
                }
    
                let cyItem: INT = (*pItem).m_MenuLeftExtent.cy;
                rcItem.right = rcItem.left + cxMax + (*this_).m_cxMenuExtent;
                rcItem.bottom = rcItem.top + cyItem;
                (*pItem).__base0.SetRect(core::ptr::addr_of_mut!(rcItem));
                rcItem.top += cyItem;
                (*this_).__base0.__base0.AddUIObj(core::ptr::addr_of_mut!((*pItem).__base0));
            }
            iItem += 1;
        }
    
        rcItem.top = 0;
        let style: DWORD = GetWindowLongPtr(hWnd, GWL_STYLE) as DWORD;
        cxMax = rcItem.right;
        let mut cyMax: INT = rcItem.bottom;
        if (style & WS_DLGFRAME) != 0 {
            cxMax = rcItem.right + 2 * GetSystemMetrics((SM_CXDLGFRAME as i32));
            cyMax += 2 * GetSystemMetrics((SM_CYDLGFRAME as i32));
        } else if (style & WS_BORDER) != 0 {
            cxMax = rcItem.right + 2 * GetSystemMetrics((SM_CXBORDER as i32));
            cyMax += 2 * GetSystemMetrics((SM_CYBORDER as i32));
        }
    
        let mut rc: RECT = RECT {
            left: 0,
            top: 0,
            right: GetSystemMetrics((SM_CXSCREEN as i32)),
            bottom: GetSystemMetrics((SM_CYSCREEN as i32)),
        };
    
        let mut rc3: RECT = *prc;
        let hMon: HMONITOR = MonitorFromRect(core::ptr::addr_of_mut!(rc3), MONITOR_DEFAULTTONEAREST);
        if !hMon.is_null() {
            let mut mi: MONITORINFO = core::mem::zeroed();
            mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
            if GetMonitorInfo(hMon, core::ptr::addr_of_mut!(mi)) != 0 {
                rc = mi.rcMonitor;
            }
        }
    
        if ((*this_).__base0.__base0.m_style & 0x200) != 0 {
            rcItem.left -= cxMax;
        }
    
        let mut x: INT = 0;
        let mut y: INT = 0;
        let mut dwFlags2: DWORD = 0;
    
        if bFlag != 0 {
            let mut bottom: INT = (*prc).bottom;
            x = rcItem.left + (*prc).left;
            if rcItem.top + bottom + cyMax > rc.bottom {
                bottom = (*prc).top - cyMax;
                dwFlags2 = 8;
            } else {
                dwFlags2 = 4;
            }
    
            y = rcItem.top + bottom;
    
            if rcItem.left + cxMax + (*prc).right > rc.right {
                x = rc.right - cxMax;
            }
        } else {
            y = rcItem.top + (*prc).top;
            if rcItem.left + (*prc).right + cxMax > rc.right {
                x = rcItem.left + (*prc).left - cxMax;
                dwFlags2 = 2;
            } else {
                x = rcItem.left + (*prc).right;
                dwFlags2 = 1;
            }
            if rcItem.top + cyMax + (*prc).bottom > rc.bottom {
                y = rc.bottom - cyMax;
            }
        }
    
        if x > rc.right - cxMax {
            x = rc.right - cxMax;
        }
        if x < rc.left {
            x = rc.left;
        }
        if y > rc.bottom - cyMax {
            y = rc.bottom - cyMax;
        }
        if y < rc.top {
            y = rc.top;
        }
    
        (*this_).__base0.Move(x, y, cxMax, -1);
    
        (*this_).__base0.SetRect(core::ptr::null());
    
        let mut bAnimation: BOOL = (FALSE) as i32;
        if bDoAnimation != 0
            && SystemParametersInfo(SPI_GETMENUANIMATION, 0, (core::ptr::addr_of_mut!(bAnimation)).cast(), 0) != 0
            && bAnimation != 0
        {
            let mut bMenuFade: BOOL = (FALSE) as i32;
            if SystemParametersInfoA(SPI_GETMENUFADE, 0, (core::ptr::addr_of_mut!(bMenuFade)).cast(), 0) == 0 {
                bMenuFade = (FALSE) as i32;
            }
    
            let dwFlags: DWORD = (if bMenuFade != 0 { 0x80000 } else { dwFlags2 }) | 0x40000;
            if (*this_).__base0.AnimateWnd(200, dwFlags) == 0 {
                (*this_).__base0.Show((TRUE) as BOOL);
            }
        } else {
            (*this_).__base0.Show((TRUE) as BOOL);
        }
    
        if !(*this_).m_pVisibleSubMenu.is_null() {
            (*(*this_).m_pVisibleSubMenu).m_pParentMenu = this_;
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn UninitShow__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if !(*this_).m_pParentMenu.is_null() {
            (*(*this_).m_pParentMenu).UninitShow();
        }
    
        (*this_).__base0.Show((FALSE) as BOOL);
    
        if !(*this_).m_pVisibleSubMenu.is_null() {
            (*(*this_).m_pVisibleSubMenu).m_pParentMenu = core::ptr::null_mut();
        }
    
        let mut iItem: usize = 0;
        while iItem < (*this_).m_MenuItems.size() {
            (*this_).__base0.RemoveUIObj(core::ptr::addr_of_mut!((*(*this_).m_MenuItems[iItem]).__base0));
            iItem += 1;
        }
    
        DestroyWindow((*this_).__base0.m_hWnd);
        return (TRUE) as BOOL;
    }
}

impl CUIFMenuItem {
    pub unsafe fn DrawArrow(&mut self, hDC: HDC, xLeft: INT, yTop: INT) {
        let this_: *mut Self = self;
        if (*this_).m_pSubMenu.is_null() {
            return;
        }
    
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*(*this_).m_pMenu).m_hMenuFont).cast());
        TextOutW(hDC, xLeft, yTop, CPP_COM_WSTR_77F906B94309.as_ptr(), 1); // rightward triangle
        SelectObject(hDC, hFontOld);
    }
    pub unsafe fn DrawBitmapProc(&mut self, hDC: HDC, xLeft: INT, yTop: INT) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_pScheme.is_null() || (*this_).m_hbmColor.is_null() {
            return;
        }
    
        let mut bm: BITMAP = core::mem::zeroed();
        GetObject(((*this_).m_hbmColor).cast(), core::mem::size_of::<BITMAP>() as INT, core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void);
    
        let mut xLeft: INT = xLeft;
        let mut yTop: INT = yTop;
        let mut width: INT = (*(*this_).m_pMenu).m_cxyMargin as INT;
        let mut height: INT = (*this_).__base0.m_rc.bottom - (*this_).__base0.m_rc.top;
        if width > bm.bmWidth {
            width = bm.bmWidth;
            xLeft += (width - bm.bmWidth) / 2;
        }
        if height > bm.bmHeight {
            height = bm.bmHeight;
            yTop += (height - bm.bmHeight) / 2;
        }
    
        let mut rc: RECT = RECT {
            left: xLeft,
            top: yTop,
            right: xLeft + width,
            bottom: yTop + height,
        };
    
        if (*this_).__base0.IsRTL() != 0 {
            (*(*this_).__base0.m_pScheme).m_bMirroring = (TRUE) as i32;
        }
    
        if (*(*this_).m_pMenu).m_pSelectedItem != this_ || (*this_).m_bMenuItemDisabled != 0 {
            (*(*this_).__base0.m_pScheme).DrawFrameCtrlBitmap(hDC, core::ptr::addr_of_mut!(rc), (*this_).m_hbmColor, (*this_).m_hbmMask, 0);
        } else {
            (*(*this_).__base0.m_pScheme).DrawFrameCtrlBitmap(hDC, core::ptr::addr_of_mut!(rc), (*this_).m_hbmColor, (*this_).m_hbmMask, UIF_DRAW_PRESSED);
        }
    
        if (*this_).__base0.IsRTL() != 0 {
            (*(*this_).__base0.m_pScheme).m_bMirroring = (FALSE) as i32;
        }
    }
    pub unsafe fn DrawCheck(&mut self, hDC: HDC, xLeft: INT, yTop: INT) {
        let this_: *mut Self = self;
        if (*this_).IsCheck() == 0 {
            return;
        }
    
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*(*this_).m_pMenu).m_hMenuFont).cast());
        let mut wch: WCHAR = if (*this_).m_bMenuItemChecked != 0 { 0x0061u16 } else { 0x0068u16 }; // checkmark or bullet
        TextOutW(hDC, xLeft, yTop, core::ptr::addr_of!(wch), 1);
        SelectObject(hDC, hFontOld);
    }
    pub unsafe fn DrawUnderline(&mut self, hDC: HDC, xText: INT, yText: INT, hbr: HBRUSH) {
        let this_: *mut Self = self;
        if (*this_).m_ichMenuItemPrefix > (*this_).m_cchMenuItemLeft {
            return;
        }
    
        let mut PrePrefixSize: SIZE = core::mem::zeroed();
        let mut PostPrefixSize: SIZE = core::mem::zeroed();
        GetTextExtentPoint32W(hDC, (*this_).m_pszMenuItemLeft, (*this_).m_ichMenuItemPrefix as INT, core::ptr::addr_of_mut!(PrePrefixSize));
        GetTextExtentPoint32W(hDC, (*this_).m_pszMenuItemLeft, ((*this_).m_ichMenuItemPrefix as INT) + 1, core::ptr::addr_of_mut!(PostPrefixSize));
    
        let bHeadPrefix: BOOL = ((*this_).m_ichMenuItemPrefix == 0 as UINT) as BOOL;
    
        let mut rc: RECT = core::mem::zeroed();
        rc.left = xText + PrePrefixSize.cx + ((bHeadPrefix == 0) as INT);
        rc.right = xText + PostPrefixSize.cx;
        rc.top = (yText + PostPrefixSize.cy) - 1;
        rc.bottom = yText + PostPrefixSize.cy;
        FillRect(hDC, core::ptr::addr_of!(rc), hbr);
    }
    pub unsafe fn __ctor_0(&mut self, pMenu: *mut CUIFMenu, bDisabled: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::addr_of_mut!((*pMenu).__base0.__base0), 0, core::ptr::null_mut(), 0);
        (*this_).__init_vtables();
        {
            (*this_).m_ichMenuItemPrefix = (1 as UINT).wrapping_neg();
            (*this_).m_nMenuItemID = 0;
            (*this_).m_pszMenuItemLeft = core::ptr::null_mut();
            (*this_).m_pszMenuItemRight = core::ptr::null_mut();
            (*this_).m_cchMenuItemLeft = 0;
            (*this_).m_cchMenuItemRight = 0;
            (*this_).m_nMenuItemVKey = 0;
            (*this_).m_hbmColor = core::ptr::null_mut();
            (*this_).m_hbmMask = core::ptr::null_mut();
            (*this_).m_bMenuItemChecked = (FALSE) as i32;
            (*this_).m_bMenuItemGrayed = (FALSE) as i32;
            (*this_).m_bMenuItemDisabled = bDisabled;
            (*this_).m_pMenu = pMenu;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pszMenuItemLeft.is_null() {
            cpp_delete_array((*this_).m_pszMenuItemLeft);
            (*this_).m_pszMenuItemLeft = core::ptr::null_mut();
        }
    
        if !(*this_).m_pszMenuItemRight.is_null() {
            cpp_delete_array((*this_).m_pszMenuItemRight);
            (*this_).m_pszMenuItemRight = core::ptr::null_mut();
        }
    
        if !(*this_).m_pSubMenu.is_null() {
            CUIFMenu__delete((*this_).m_pSubMenu);
            (*this_).m_pSubMenu = core::ptr::null_mut();
        }
    }
    pub unsafe fn Init(&mut self, nMenuItemID: UINT, pszText: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_nMenuItemID = nMenuItemID;
    
        if pszText.is_null() {
            (*this_).m_pszMenuItemLeft = core::ptr::null_mut();
            (*this_).m_cchMenuItemLeft = 0;
            return (TRUE) as BOOL;
        }
    
        let cch: INT = lstrlenW(pszText);
        (*this_).m_pszMenuItemLeft = cpp_new_array_CicNoThrow::<WCHAR>(cch + 1);
        if (*this_).m_pszMenuItemLeft.is_null() {
            return (FALSE) as BOOL;
        }
    
        let mut pch0: *const WCHAR = pszText;
        let mut ich1: INT;
        let mut ich2: INT;
        ich1 = 0;
        while *pch0 != 0 && *pch0 != 0x0009 {
            if *pch0 == 0x0026 && {
                pch0 = pch0.add(1);
                *pch0 != 0x0026
            } {
                (*this_).m_nMenuItemVKey = VkKeyScanW(*pch0) as UINT;
                if (*this_).m_nMenuItemVKey == 0 {
                    (*this_).m_nMenuItemVKey =
                        (VkKeyScanA(*(pch0 as *const BYTE) as CHAR) as BYTE) as UINT;
                }
                (*this_).m_ichMenuItemPrefix = ich1 as UINT;
            }
            *(*this_).m_pszMenuItemLeft.offset(ich1 as isize) = *pch0;
            ich1 += 1;
            pch0 = pch0.add(1);
        }
        *(*this_).m_pszMenuItemLeft.offset(ich1 as isize) = 0;
        (*this_).m_cchMenuItemLeft = lstrlenW((*this_).m_pszMenuItemLeft) as UINT;
    
        if *pch0 == 0x0009 {
            (*this_).m_cchMenuItemRight = 0;
            (*this_).m_pszMenuItemRight = cpp_new_array_CicNoThrow::<WCHAR>(cch + 1);
            if !(*this_).m_pszMenuItemRight.is_null() {
                pch0 = pch0.add(1);
                let mut wch: WCHAR = *pch0;
                ich2 = 0;
                while *pch0 != 0 {
                    *(*this_).m_pszMenuItemRight.offset(ich2 as isize) = wch;
                    pch0 = pch0.add(1);
                    wch = *pch0;
                    ich2 += 1;
                }
                *(*this_).m_pszMenuItemRight.offset(ich2 as isize) = 0;
                (*this_).m_cchMenuItemRight = lstrlenW((*this_).m_pszMenuItemRight) as UINT;
            }
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn IsCheck(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).m_bMenuItemChecked != 0 || (*this_).m_bMenuItemForceChecked != 0) as BOOL;
    }
    pub unsafe fn SetBitmapMask(&mut self, hbmMask: HBITMAP) {
        let this_: *mut Self = self;
        (*this_).m_hbmMask = hbmMask;
        (*this_).__base0.CallOnPaint();
    }
    pub unsafe fn ShowSubPopup(&mut self) {
        let this_: *mut Self = self;
        let mut rc: RECT = (*this_).__base0.m_rc;
        ClientToScreen((*(*this_).__base0.m_pWindow).op_to_HWND(), core::ptr::addr_of_mut!(rc) as LPPOINT);
        ClientToScreen((*(*this_).__base0.m_pWindow).op_to_HWND(), core::ptr::addr_of_mut!(rc.right) as LPPOINT);
        (*(*this_).m_pSubMenu).ShowSubPopup((*this_).m_pMenu, core::ptr::addr_of_mut!(rc), (FALSE) as i32);
    }
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if (*this_).m_bMenuItemGrayed == 0 && (*this_).m_bMenuItemDisabled == 0 && (*this_).m_pSubMenu.is_null() {
            (*(*this_).m_pMenu).SetSelectedId((*this_).m_nMenuItemID);
            PostMessage((*(*this_).__base0.m_pWindow).op_to_HWND(), WM_NULL, 0, 0);
        }
    }
    pub unsafe fn OnMouseIn__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if !(*(*this_).m_pMenu).m_pParentMenu.is_null() {
            (*(*(*this_).m_pMenu).m_pParentMenu).CancelMenu();
        }
    
        if !(*this_).m_pSubMenu.is_null() {
            let mut Delay: DWORD = 0;
            if SystemParametersInfo(SPI_GETMENUSHOWDELAY, 0, core::ptr::addr_of_mut!(Delay) as *mut core::ffi::c_void, 0) == 0 {
                Delay = 300;
            }
    
            (*this_).__base0.StartTimer((Delay) as WPARAM);
        }
    
        (*(*this_).m_pMenu).SetSelectedItem(this_);
    }
    pub unsafe fn OnPaint__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if (*(*this_).m_pMenu).__base0.__base0.m_style & UIF_MENU_USE_OFF10 != 0 {
            (*this_).OnPaintO10(hDC);
        } else {
            (*this_).OnPaintDef(hDC);
        }
    }
    pub unsafe fn OnTimer__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.EndTimer();
        if (*(*this_).m_pMenu).__base0.m_pPointed == this_ as *mut CUIFObject {
            (*this_).ShowSubPopup();
        }
    }
    pub unsafe fn InitMenuExtent__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_pszMenuItemLeft.is_null() {
            if !(*this_).m_hbmColor.is_null() {
                let mut bm: BITMAP = core::mem::zeroed();
                GetObject((*this_).m_hbmColor as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID);
                (*this_).m_MenuLeftExtent.cx = bm.bmWidth + 2;
                (*this_).m_MenuLeftExtent.cy = bm.bmHeight + 4;
            } else {
                (*this_).m_MenuLeftExtent.cx = 0;
                (*this_).m_MenuLeftExtent.cy = 0;
            }
            return;
        }
    
        let hDC: HDC = GetDC((*(*this_).__base0.m_pWindow).op_to_HWND());
    
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.m_hFont).cast());
        GetTextExtentPoint32W(hDC, (*this_).m_pszMenuItemLeft, ((*this_).m_cchMenuItemLeft) as i32, core::ptr::addr_of_mut!((*this_).m_MenuLeftExtent));
        (*this_).m_MenuLeftExtent.cx += 16;
        (*this_).m_MenuLeftExtent.cy += 8;
        if !(*this_).m_pszMenuItemRight.is_null() {
            GetTextExtentPoint32W(hDC, (*this_).m_pszMenuItemRight, ((*this_).m_cchMenuItemRight) as i32, core::ptr::addr_of_mut!((*this_).m_MenuRightExtent));
            (*this_).m_MenuRightExtent.cy += 8;
        }
        SelectObject(hDC, hFontOld);
    
        if !(*this_).m_pSubMenu.is_null() {
            (*this_).m_MenuLeftExtent.cx += (*this_).m_MenuLeftExtent.cy + 2;
        }
        if (*(*this_).m_pMenu).__base0.__base0.m_style & UIF_MENU_USE_OFF10 != 0 {
            (*this_).m_MenuLeftExtent.cx += 24;
        }
    
        ReleaseDC((*(*this_).__base0.m_pWindow).op_to_HWND(), hDC);
    }
    pub unsafe fn OnPaintDef__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.m_hFont).cast());
    
        let mut textSize: SIZE = core::mem::zeroed();
        GetTextExtentPoint32W(hDC, (*this_).m_pszMenuItemLeft, ((*this_).m_cchMenuItemLeft) as i32, core::ptr::addr_of_mut!(textSize));
    
        let cxyMargin: LONG = if (*(*this_).m_pMenu).m_bHasMargin != 0 { (*(*this_).m_pMenu).m_cxyMargin } else { 0 };
    
        let cySpace: LONG = (*this_).__base0.m_rc.bottom - (*this_).__base0.m_rc.top - textSize.cy;
        let xCheck: LONG = (*this_).__base0.m_rc.left;
        let yCheck: LONG = (*this_).__base0.m_rc.top + cySpace / 2;
        let xBitmap: LONG = (*this_).__base0.m_rc.left + cxyMargin;
        let yBitmap: LONG = (*this_).__base0.m_rc.top;
        let xText: LONG = (*this_).__base0.m_rc.left + cxyMargin + (*(*this_).m_pMenu).m_cxyMargin + 2;
        let yText: LONG = (*this_).__base0.m_rc.top + cySpace / 2;
    
        let xArrow: LONG = (*this_).__base0.m_rc.right + (*this_).__base0.m_rc.left - 10;
    
        SetBkMode(hDC, (TRANSPARENT as i32));
    
        if (*this_).m_bMenuItemGrayed != 0
        {
            let mut uOptions: UINT = ETO_CLIPPED;
            if (*this_).m_bMenuItemDisabled != 0 || (*(*this_).m_pMenu).m_pSelectedItem != this_
            {
                SetTextColor(hDC, GetSysColor((COLOR_BTNHIGHLIGHT as i32)));
                ExtTextOutW(hDC, xText + 1, yText + 1, ETO_CLIPPED, core::ptr::addr_of_mut!((*this_).__base0.m_rc), (*this_).m_pszMenuItemLeft,
                              (*this_).m_cchMenuItemLeft, core::ptr::null_mut());
                (*this_).DrawCheck(hDC, xCheck + 1, yCheck + 1);
                (*this_).DrawBitmapProc(hDC, xBitmap + 1, yBitmap + 1);
                (*this_).DrawArrow(hDC, xArrow + 1, yText + 1);
            }
            else
            {
                SetBkColor(hDC, GetSysColor((COLOR_HIGHLIGHT as i32)));
                uOptions = ETO_CLIPPED | ETO_OPAQUE;
            }
            SetTextColor(hDC, GetSysColor((COLOR_BTNSHADOW as i32)));
            ExtTextOutW(hDC, xText, yText, uOptions, core::ptr::addr_of_mut!((*this_).__base0.m_rc), (*this_).m_pszMenuItemLeft,
                          (*this_).m_cchMenuItemLeft, core::ptr::null_mut());
            (*this_).DrawUnderline(hDC, xText, yText, UlongToHandle!(COLOR_BTNSHADOW + 1) as HBRUSH);
        }
        else if (*this_).m_bMenuItemDisabled != 0 || (*(*this_).m_pMenu).m_pSelectedItem != this_
        {
            SetTextColor(hDC, GetSysColor((COLOR_MENUTEXT as i32)));
            ExtTextOutW(hDC, xText, yText, ETO_CLIPPED, core::ptr::addr_of_mut!((*this_).__base0.m_rc), (*this_).m_pszMenuItemLeft,
                          (*this_).m_cchMenuItemLeft, core::ptr::null_mut());
            (*this_).DrawUnderline(hDC, xText, yText, UlongToHandle!(COLOR_MENUTEXT + 1) as HBRUSH);
        }
        else
        {
            SetTextColor(hDC, GetSysColor((COLOR_HIGHLIGHTTEXT as i32)));
            SetBkColor(hDC, GetSysColor((COLOR_HIGHLIGHT as i32)));
            ExtTextOutW(hDC, xText, yText, ETO_CLIPPED | ETO_OPAQUE, core::ptr::addr_of_mut!((*this_).__base0.m_rc),
                          (*this_).m_pszMenuItemLeft, (*this_).m_cchMenuItemLeft, core::ptr::null_mut());
            (*this_).DrawUnderline(hDC, xText, yText, UlongToHandle!(COLOR_HIGHLIGHTTEXT + 1) as HBRUSH);
        }
    
        (*this_).DrawCheck(hDC, xCheck, yCheck);
        (*this_).DrawBitmapProc(hDC, xBitmap, yBitmap);
        (*this_).DrawArrow(hDC, xArrow, yText);
    
        SelectObject(hDC, hFontOld);
    }
    pub unsafe fn OnPaintO10__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_pScheme.is_null() {
            return;
        }
    
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.m_hFont).cast());
    
        let mut textSize: SIZE = core::mem::zeroed();
        GetTextExtentPoint32W(hDC, (*this_).m_pszMenuItemLeft, ((*this_).m_cchMenuItemLeft) as i32, core::ptr::addr_of_mut!(textSize));
    
        let cySpace: LONG = (*this_).__base0.m_rc.bottom - (*this_).__base0.m_rc.top - textSize.cy;
        let xCheck: LONG = (*this_).__base0.m_rc.left;
        let yCheck: LONG = (*this_).__base0.m_rc.top + cySpace / 2;
        let cxyMargin: LONG = if (*(*this_).m_pMenu).m_bHasMargin != 0 {
            (*(*this_).m_pMenu).m_cxyMargin
        } else {
            0
        };
    
        let xBitmap: LONG = (*this_).__base0.m_rc.left + cxyMargin;
        let yBitmap: LONG = (*this_).__base0.m_rc.top;
        let xText: LONG = (*this_).__base0.m_rc.left + (*(*this_).m_pMenu).m_cxyMargin + cxyMargin + 8;
        let yText: LONG = (*this_).__base0.m_rc.top + cySpace / 2;
        let xArrow: LONG = (*this_).__base0.m_rc.left - textSize.cy + (*this_).__base0.m_rc.right - 2;
        let xRightText: LONG = (*this_).__base0.m_rc.right - (*(*this_).m_pMenu).m_cxMenuExtent - 8;
    
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetRect(core::ptr::addr_of_mut!(rc));
    
        if (*this_).m_bMenuItemDisabled != 0 || (*(*this_).m_pMenu).m_pSelectedItem != this_ {
            rc.right = (*(*this_).m_pMenu).m_cxyMargin + rc.left + 2;
            if (*(*this_).m_pMenu).m_bHasMargin != 0 {
                rc.right += (*(*this_).m_pMenu).m_cxyMargin;
            }
    
            FillRect(
                hDC,
                core::ptr::addr_of_mut!(rc),
                (*(*this_).__base0.m_pScheme).GetBrush(9),
            );
        } else {
            (*(*this_).__base0.m_pScheme).DrawCtrlBkgd(hDC, core::ptr::addr_of_mut!(rc), 0, UIF_DRAW_PRESSED);
            (*(*this_).__base0.m_pScheme).DrawCtrlEdge(hDC, core::ptr::addr_of_mut!(rc), 0, UIF_DRAW_PRESSED);
        }
    
        SetBkMode(hDC, (TRANSPARENT as i32));
    
        if (*this_).m_bMenuItemGrayed != 0 {
            SetTextColor(hDC, (*(*this_).__base0.m_pScheme).GetColor(11));
            ExtTextOutW(
                hDC,
                xText,
                yText,
                ETO_CLIPPED,
                core::ptr::addr_of_mut!((*this_).__base0.m_rc),
                (*this_).m_pszMenuItemLeft,
                (*this_).m_cchMenuItemLeft,
                core::ptr::null_mut(),
            );
        } else if (*this_).m_bMenuItemDisabled != 0 || (*(*this_).m_pMenu).m_pSelectedItem != this_ {
            SetTextColor(hDC, (*(*this_).__base0.m_pScheme).GetColor(10));
            ExtTextOutW(
                hDC,
                xText,
                yText,
                ETO_CLIPPED,
                core::ptr::addr_of_mut!((*this_).__base0.m_rc),
                (*this_).m_pszMenuItemLeft,
                (*this_).m_cchMenuItemLeft,
                core::ptr::null_mut(),
            );
        } else {
            SetTextColor(hDC, (*(*this_).__base0.m_pScheme).GetColor(5));
            ExtTextOutW(
                hDC,
                xText,
                yText,
                ETO_CLIPPED,
                core::ptr::addr_of_mut!((*this_).__base0.m_rc),
                (*this_).m_pszMenuItemLeft,
                (*this_).m_cchMenuItemLeft,
                core::ptr::null_mut(),
            );
        }
    
        (*this_).DrawUnderline(hDC, xText, yText, (*(*this_).__base0.m_pScheme).GetBrush(5));
    
        if !(*this_).m_pszMenuItemRight.is_null() {
            ExtTextOutW(
                hDC,
                xRightText,
                yText,
                ETO_CLIPPED,
                core::ptr::addr_of_mut!((*this_).__base0.m_rc),
                (*this_).m_pszMenuItemRight,
                (*this_).m_cchMenuItemRight,
                core::ptr::null_mut(),
            );
        }
    
        (*this_).DrawCheck(hDC, xCheck, yCheck);
        (*this_).DrawBitmapProc(hDC, xBitmap, yBitmap);
        (*this_).DrawArrow(hDC, xArrow, yText);
    
        SelectObject(hDC, hFontOld);
    }
}

impl CUIFMenuItemSeparator {
    pub unsafe fn __ctor_0(&mut self, pMenu: *mut CUIFMenu) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pMenu, (TRUE) as i32);
        (*this_).__init_vtables();
        {
            (*this_).__base0.m_nMenuItemID = (1 as u32).wrapping_neg();
        }
    }
    pub unsafe fn InitMenuExtent__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.m_MenuLeftExtent.cx = 0;
        (*this_).__base0.m_MenuLeftExtent.cy = 6;
    }
    pub unsafe fn OnPaintDef__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.__base0.m_pScheme.is_null() {
            return;
        }
    
        let mut rc: RECT = core::mem::zeroed();
        rc.left = (*this_).__base0.__base0.m_rc.left + 2;
        rc.top = (*this_).__base0.__base0.m_rc.top + ((*this_).__base0.__base0.m_rc.bottom - (*this_).__base0.__base0.m_rc.top - 2) / 2;
        rc.right = (*this_).__base0.__base0.m_rc.right - 2;
        rc.bottom = rc.top + 2;
        (*(*this_).__base0.__base0.m_pScheme).DrawMenuSeparator(hDC, core::ptr::addr_of_mut!(rc));
    }
    pub unsafe fn OnPaintO10__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.__base0.m_pScheme.is_null() {
            return;
        }
    
        let cx: LONG = (*this_).__base0.__base0.m_rc.right - (*this_).__base0.__base0.m_rc.left - 4;
        let cy: LONG = ((*this_).__base0.__base0.m_rc.bottom - (*this_).__base0.__base0.m_rc.top - 2) / 2;
    
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.__base0.GetRect(&mut rc);
    
        rc.right = rc.left + (*(*this_).__base0.m_pMenu).m_cxyMargin + 2;
        if (*(*this_).__base0.m_pMenu).m_bHasMargin != 0 {
            rc.right += (*(*this_).__base0.m_pMenu).m_cxyMargin;
        }
    
        let hBrush: HBRUSH = (*(*this_).__base0.__base0.m_pScheme).GetBrush(9);
        FillRect(hDC, &mut rc, hBrush);
        rc = RECT {
            left: (*this_).__base0.__base0.m_rc.left + (*(*this_).__base0.m_pMenu).m_cxyMargin + 4,
            top: (*this_).__base0.__base0.m_rc.top + cy,
            right: (*this_).__base0.__base0.m_rc.left + cx + 2,
            bottom: (*this_).__base0.__base0.m_rc.top + cy + 1,
        };
        (*(*this_).__base0.__base0.m_pScheme).DrawMenuSeparator(hDC, &mut rc);
    }
}

impl CUIFButton {
    pub unsafe fn DrawBitmapProc(&mut self, hDC: HDC, prc: LPCRECT, bPressed: BOOL) {
        let this_: *mut Self = self;
        let width: INT = (*this_).__base0.m_rc.right - (*this_).__base0.m_rc.left;
        let height: INT = (*this_).__base0.m_rc.bottom - (*this_).__base0.m_rc.top;
        if !(*this_).m_hbmButton2.is_null() {
            let hbmMask: HBITMAP = cicCreateMaskBmp(
                core::ptr::addr_of_mut!((*this_).__base0.m_rc),
                (*this_).m_hbmButton1,
                (*this_).m_hbmButton2,
                UlongToHandle!(COLOR_BTNFACE + 1) as HBRUSH,
                0,
                0,
            );
            DrawState(
                hDC,
                core::ptr::null_mut(),
                None,
                hbmMask as LPARAM,
                0,
                (*prc).left + bPressed,
                (*prc).top + bPressed,
                width - bPressed,
                height - bPressed,
                DST_BITMAP
                    | (if (*this_).__base0.m_bEnable != 0 {
                        0
                    } else {
                        DSS_MONO | DSS_DISABLED
                    }),
            );
            DeleteObject((hbmMask).cast());
        } else {
            DrawState(
                hDC,
                core::ptr::null_mut(),
                None,
                (*this_).m_hbmButton1 as LPARAM,
                0,
                (*prc).left + bPressed,
                (*prc).top + bPressed,
                width - bPressed,
                height - bPressed,
                DST_BITMAP
                    | (if (*this_).__base0.m_bEnable != 0 {
                        0
                    } else {
                        DSS_MONO | DSS_DISABLED
                    }),
            );
        }
    }
    pub unsafe fn DrawEdgeProc(&mut self, hDC: HDC, prc: LPCRECT, bPressed: BOOL) {
        let this_: *mut Self = self;
        let mut rc: RECT = *prc;
        if bPressed != 0 {
            DrawEdge(hDC, &mut rc, BDR_SUNKENOUTER, BF_RECT);
        } else {
            DrawEdge(hDC, &mut rc, BDR_RAISEDINNER, BF_RECT);
        }
    }
    pub unsafe fn DrawIconProc(&mut self, hDC: HDC, prc: LPRECT, bPressed: BOOL) {
        let this_: *mut Self = self;
        let width: INT = (*prc).right - (*prc).left;
        let height: INT = (*prc).bottom - (*prc).top;
        let mut rc: RECT = RECT { left: 0, top: 0, right: width, bottom: height };
    
        let hMemDC: HDC = CreateCompatibleDC(hDC);
        if hMemDC.is_null() {
            return;
        }
    
        let hbmMem: HBITMAP = CreateCompatibleBitmap(hDC, width, height);
        if hbmMem.is_null() {
            DeleteDC(hMemDC);
            return;
        }
    
        let hbmOld: HGDIOBJ = SelectObject(hMemDC, hbmMem as HGDIOBJ);
        if (*this_).__base0.m_bEnable != 0 {
            BitBlt(hMemDC, rc.left, rc.top, width, height, hDC, (*prc).left, (*prc).top, SRCCOPY);
        } else {
            let hbrWhite: HBRUSH = GetStockObject((WHITE_BRUSH as i32)) as HBRUSH;
            FillRect(hMemDC, core::ptr::addr_of_mut!(rc), hbrWhite);
        }
    
        if (*this_).__base0.m_style & UIF_BUTTON_LARGE_ICON != 0 {
            DrawIconEx(
                hMemDC,
                2 + bPressed,
                2 + bPressed,
                (*this_).m_ButtonIcon.m_hIcon,
                width - 4,
                height - 4,
                0,
                core::ptr::null_mut(),
                DI_NORMAL,
            );
        } else {
            DrawIconEx(
                hMemDC,
                (width - 16) / 2 + bPressed,
                (height - 16) / 2 + bPressed,
                (*this_).m_ButtonIcon.m_hIcon,
                16,
                16,
                0,
                core::ptr::null_mut(),
                DI_NORMAL,
            );
        }
    
        SelectObject(hMemDC, hbmOld);
        DrawState(
            hDC,
            core::ptr::null_mut(),
            None,
            hbmMem as LPARAM,
            0,
            (*prc).left,
            (*prc).top,
            width,
            height,
            DST_BITMAP
                | (if (*this_).__base0.m_bEnable != 0 {
                    0
                } else {
                    DSS_MONO | DSS_DISABLED
                }),
        );
        DeleteObject(hbmMem as HGDIOBJ);
        DeleteDC(hMemDC);
    }
    pub unsafe fn DrawTextProc(&mut self, hDC: HDC, prc: LPCRECT, bPressed: BOOL) {
        let this_: *mut Self = self;
        if (*this_).m_pszButtonText.is_null() {
            return;
        }
    
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.m_hFont).cast());
        let cchText: INT = lstrlenW((*this_).m_pszButtonText);
        let mut textSize: SIZE = core::mem::zeroed();
        GetTextExtentPoint32W(hDC, (*this_).m_pszButtonText, cchText, core::ptr::addr_of_mut!(textSize));
    
        let mut xText: INT;
        let mut yText: INT;
        if ((*this_).__base0.m_style & UIF_BUTTON_H_ALIGN_MASK) == UIF_BUTTON_H_ALIGN_CENTER {
            xText = ((*this_).__base0.m_rc.right - (*this_).__base0.m_rc.left - textSize.cx) / 2;
        } else if ((*this_).__base0.m_style & UIF_BUTTON_H_ALIGN_MASK) == UIF_BUTTON_H_ALIGN_RIGHT {
            xText = (*this_).__base0.m_rc.right - (*this_).__base0.m_rc.left - textSize.cx;
        } else {
            xText = 0;
        }
    
        if ((*this_).__base0.m_style & UIF_BUTTON_V_ALIGN_MASK) == UIF_BUTTON_V_ALIGN_MIDDLE {
            yText = ((*this_).__base0.m_rc.bottom - (*this_).__base0.m_rc.top - textSize.cy) / 2;
        } else if ((*this_).__base0.m_style & UIF_BUTTON_V_ALIGN_MASK) == UIF_BUTTON_V_ALIGN_BOTTOM {
            yText = (*this_).__base0.m_rc.bottom - (*this_).__base0.m_rc.top - textSize.cy;
        } else {
            yText = 0;
        }
    
        SetBkMode(hDC, (TRANSPARENT as i32));
    
        if (*this_).__base0.m_bEnable != 0 {
            SetTextColor(hDC, GetSysColor((COLOR_BTNTEXT as i32)));
            ExtTextOutW(
                hDC,
                xText + (*prc).left + bPressed,
                yText + (*prc).top + bPressed,
                ETO_CLIPPED,
                prc,
                (*this_).m_pszButtonText,
                (cchText) as UINT,
                core::ptr::null_mut(),
            );
        } else {
            SetTextColor(hDC, GetSysColor((COLOR_BTNHILIGHT as i32)));
            ExtTextOutW(
                hDC,
                xText + (*prc).left + bPressed + 1,
                yText + (*prc).top + bPressed + 1,
                ETO_CLIPPED,
                prc,
                (*this_).m_pszButtonText,
                (cchText) as UINT,
                core::ptr::null_mut(),
            );
    
            SetTextColor(hDC, GetSysColor((COLOR_BTNSHADOW as i32)));
            ExtTextOutW(
                hDC,
                xText + (*prc).left + bPressed,
                yText + (*prc).top + bPressed,
                ETO_CLIPPED,
                prc,
                (*this_).m_pszButtonText,
                (cchText) as UINT,
                core::ptr::null_mut(),
            );
        }
    
        SelectObject(hDC, hFontOld);
    }
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFObject, nObjectID: DWORD, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pParent, nObjectID, prc, style);
        (*this_).__init_vtables();
        {
            (*this_).m_ButtonIcon.m_hIcon = core::ptr::null_mut();
            (*this_).m_ButtonIcon.m_hImageList = core::ptr::null_mut();
            (*this_).m_dwUnknown9 = 0;
            (*this_).m_uButtonStatus = 0;
            (*this_).m_bPressed = (FALSE) as i32;
            (*this_).m_hbmButton1 = core::ptr::null_mut();
            (*this_).m_hbmButton2 = core::ptr::null_mut();
            (*this_).m_pszButtonText = core::ptr::null_mut();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pszButtonText.is_null() {
            cpp_delete_array((*this_).m_pszButtonText);
            (*this_).m_pszButtonText = core::ptr::null_mut();
        }
    
        if !(*this_).m_ButtonIcon.m_hImageList.is_null() {
            ImageList_Destroy((*this_).m_ButtonIcon.m_hImageList);
        }
    }
    pub unsafe fn SetIcon(&mut self, hIcon: HICON) {
        let this_: *mut Self = self;
        (*this_).m_ButtonIcon.op_assign(hIcon);
    
        if !(*this_).m_ButtonIcon.m_hIcon.is_null() {
            (*this_).GetIconSize((*this_).m_ButtonIcon.m_hIcon, core::ptr::addr_of_mut!((*this_).m_IconSize));
        } else {
            (*this_).m_IconSize.cy = 0;
            (*this_).m_IconSize.cx = 0;
        }
    
        (*this_).__base0.CallOnPaint();
    }
    pub unsafe fn SetText(&mut self, pszText: LPCWSTR) {
        let this_: *mut Self = self;
        if !(*this_).m_pszButtonText.is_null() {
            cpp_delete_array((*this_).m_pszButtonText);
            (*this_).m_pszButtonText = core::ptr::null_mut();
        }
    
        (*this_).m_TextSize.cx = 0;
        (*this_).m_TextSize.cy = 0;
    
        if !pszText.is_null() {
            let cch: INT = lstrlenW(pszText);
            (*this_).m_pszButtonText = cpp_new_array_CicNoThrow::<WCHAR>(cch + 1);
            if (*this_).m_pszButtonText.is_null() {
                return;
            }
    
            lstrcpynW((*this_).m_pszButtonText, pszText, cch + 1);
            (*this_).GetTextSize((*this_).m_pszButtonText, core::ptr::addr_of_mut!((*this_).m_TextSize));
        }
    
        (*this_).__base0.CallOnPaint();
    }
    pub unsafe fn GetIconSize(&mut self, hIcon: HICON, pSize: LPSIZE) {
        let this_: *mut Self = self;
        let mut IconInfo: ICONINFO = core::mem::zeroed();
        if GetIconInfo(hIcon, core::ptr::addr_of_mut!(IconInfo)) != 0 {
            let mut bm: BITMAP = core::mem::zeroed();
            GetObject(
                (IconInfo.hbmColor).cast(),
                core::mem::size_of::<BITMAP>() as i32,
                core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void,
            );
            DeleteObject((IconInfo.hbmColor).cast());
            DeleteObject((IconInfo.hbmMask).cast());
            (*pSize).cx = bm.bmWidth;
            (*pSize).cy = bm.bmHeight;
        } else {
            (*pSize).cx = GetSystemMetrics((SM_CXSMICON as i32));
            (*pSize).cy = GetSystemMetrics((SM_CYSMICON as i32));
        }
    }
    pub unsafe fn GetTextSize(&mut self, pszText: LPCWSTR, pSize: LPSIZE) {
        let this_: *mut Self = self;
        let hDC: HDC = GetDC(core::ptr::null_mut());
        let cchText: INT = lstrlenW(pszText);
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.m_hFont).cast());
    
        if (*this_).__base0.m_bHasCustomFont == 0
            && ((*this_).__base0.__base0.EnsureThemeData((*(*this_).__base0.m_pWindow).op_to_HWND()) >= 0)
        {
            let mut rc: RECT = core::mem::zeroed();
            (*this_).__base0.__base0.GetThemeTextExtent(
                hDC,
                0,
                pszText,
                cchText,
                0,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(rc),
            );
            (*pSize).cx = rc.right;
            (*pSize).cy = rc.bottom;
        } else {
            GetTextExtentPoint32W(hDC, pszText, cchText, pSize);
        }
    
        if ((*this_).__base0.m_style & UIF_BUTTON_VERTICAL) != 0 {
            let tmp: INT = (*pSize).cx;
            (*pSize).cx = (*pSize).cy;
            (*pSize).cy = tmp;
        }
    
        SelectObject(hDC, hFontOld);
        ReleaseDC(core::ptr::null_mut(), hDC);
    }
    pub unsafe fn Enable__body(&mut self, bEnable: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.Enable(bEnable);
        if !((*this_).__base0.m_bEnable != 0) {
            (*this_).SetStatus(0);
            if (*this_).__base0.IsCapture() != 0 {
                (*this_).__base0.EndCapture();
            }
        }
    }
    pub unsafe fn OnMouseIn__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        let _ = (x, y);
        if ((*this_).__base0.m_style & 0x30) == 0x20 {
            if (((*this_).__base0.IsCapture()) != 0) {
                (*this_).SetStatus(0);
            } else {
                (*this_).SetStatus(2);
            }
        } else {
            if (((*this_).__base0.IsCapture()) != 0) {
                (*this_).SetStatus(1);
            } else {
                (*this_).SetStatus(2);
            }
        }
    }
    pub unsafe fn OnMouseOut__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if ((*this_).__base0.m_style & 0x30) == 0x20 {
            (*this_).SetStatus(0);
        } else if (*this_).__base0.IsCapture() != 0 {
            (*this_).SetStatus(3);
        } else {
            (*this_).SetStatus(0);
        }
    }
    pub unsafe fn OnLButtonDown__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        (*this_).SetStatus(1);
        (*this_).__base0.StartCapture();
        if ((*this_).__base0.m_style & 0x30) == 0x20 {
            (*this_).__base0.NotifyCommand(1, 0);
        }
    }
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: x, y: y };
        let mut bCapture: BOOL = (*this_).__base0.IsCapture();
        if bCapture != 0 {
            (*this_).__base0.EndCapture();
        }
    
        let mut bNotInObject: BOOL = (((*this_).__base0.m_style & 0x30) == 0x20) as BOOL;
        if ((*this_).__base0.m_style & 0x30) != 0x10 {
            bNotInObject = ((*this_).__base0.PtInObject(pt) == 0) as BOOL;
            if bNotInObject != 0 {
                (*this_).SetStatus(0);
                return;
            }
        } else {
            if bNotInObject == 0 {
                bNotInObject = ((*this_).__base0.PtInObject(pt) == 0) as BOOL;
                if bNotInObject == 0 {
                    (*this_).SetStatus(2);
                    (*this_).__base0.NotifyCommand(1, 0);
                    return;
                }
            }
            (*this_).SetStatus(0);
            return;
        }
    
        (*this_).SetStatus(2);
    
        if bCapture != 0 {
            (*this_).m_bPressed = ((*this_).m_bPressed == 0) as BOOL;
            (*this_).__base0.NotifyCommand(1, 0);
        }
    }
    pub unsafe fn OnPaintNoTheme__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        FillRect(hDC, core::ptr::addr_of_mut!((*this_).__base0.m_rc), UlongToHandle!(COLOR_BTNFACE + 1) as HBRUSH);
    
        if ((*this_).m_bPressed != 0) && (((*this_).m_uButtonStatus == 0) || ((*this_).m_uButtonStatus == 3)) {
            let hbr: HBRUSH = cicCreateDitherBrush();
            if !hbr.is_null() {
                let OldTextColor: COLORREF = SetTextColor(hDC, GetSysColor((COLOR_BTNFACE as i32)));
                let OldBkColor: COLORREF = SetBkColor(hDC, GetSysColor((COLOR_BTNHIGHLIGHT as i32)));
                let mut rc: RECT = (*this_).__base0.m_rc;
                InflateRect(core::ptr::addr_of_mut!(rc), -2, -2);
                FillRect(hDC, core::ptr::addr_of_mut!(rc), hbr);
                SetTextColor(hDC, OldTextColor);
                SetBkColor(hDC, OldBkColor);
                DeleteObject((hbr).cast());
            }
        }
    
        let bPressed: BOOL = (((*this_).m_bPressed != 0) || ((*this_).m_uButtonStatus == 1)) as BOOL;
        if !(*this_).m_hbmButton1.is_null() {
            (*this_).DrawBitmapProc(hDC, core::ptr::addr_of_mut!((*this_).__base0.m_rc), bPressed);
        } else if !(*this_).m_ButtonIcon.m_hIcon.is_null() {
            (*this_).DrawIconProc(hDC, core::ptr::addr_of_mut!((*this_).__base0.m_rc), bPressed);
        } else {
            (*this_).DrawTextProc(hDC, core::ptr::addr_of_mut!((*this_).__base0.m_rc), bPressed);
        }
    
        if ((*this_).m_bPressed != 0) || ((*this_).m_uButtonStatus == 1) {
            (*this_).DrawEdgeProc(hDC, core::ptr::addr_of_mut!((*this_).__base0.m_rc), (TRUE) as BOOL);
        } else if 2 <= (*this_).m_uButtonStatus && (*this_).m_uButtonStatus <= 3 {
            (*this_).DrawEdgeProc(hDC, core::ptr::addr_of_mut!((*this_).__base0.m_rc), (FALSE) as BOOL);
        }
    }
    pub unsafe fn SetStatus__body(&mut self, uStatus: UINT) {
        let this_: *mut Self = self;
        if uStatus != (*this_).m_uButtonStatus {
            (*this_).m_uButtonStatus = uStatus;
            (*this_).__base0.CallOnPaint();
        }
    }
}

impl CUIFButton2 {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFObject, nObjectID: DWORD, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pParent, nObjectID, prc, style);
        (*this_).__init_vtables();
        {
            (*this_).__base0.__base0.__base0.m_iStateId = 0;
            (*this_).__base0.__base0.__base0.m_iPartId = (BP_PUSHBUTTON) as i32;
            (*this_).__base0.__base0.__base0.m_pszClassList = CPP_COM_WSTR_91CDBB74B095.as_ptr();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__base0.__base0.CloseThemeData();
    }
    pub unsafe fn MakeDrawFlag(&mut self) -> DWORD {
        let this_: *mut Self = self;
        let mut dwDrawFlags: DWORD = 0;
        if (*this_).__base0.m_bPressed != 0 {
            dwDrawFlags |= UIF_DRAW_PRESSED;
        }
        if (*this_).__base0.m_uButtonStatus == 1 {
            dwDrawFlags |= 0x2;
        } else if 2 <= (*this_).__base0.m_uButtonStatus && (*this_).__base0.m_uButtonStatus <= 3 {
            dwDrawFlags |= 0x1;
        }
        if (*this_).__base0.__base0.m_bEnable == 0 {
            dwDrawFlags |= UIF_DRAW_DISABLED;
        }
        return dwDrawFlags;
    }
    pub unsafe fn OnPaintTheme__body(&mut self, hDC: HDC,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
    pub unsafe fn OnPaintNoTheme__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.__base0.m_pScheme.is_null() {
            return;
        }
    
        let width: INT = (*this_).__base0.__base0.m_rc.right - (*this_).__base0.__base0.m_rc.left;
        let height: INT = (*this_).__base0.__base0.m_rc.bottom - (*this_).__base0.__base0.m_rc.top;
        let hdcMem: HDC = CreateCompatibleDC(hDC);
        if hdcMem.is_null() {
            return;
        }
    
        let hbmMem: HBITMAP = CreateCompatibleBitmap(hDC, width, height);
        if hbmMem.is_null() {
            DeleteDC(hdcMem);
            return;
        }
    
        let hbmOld: HGDIOBJ = SelectObject(hdcMem, (hbmMem).cast());
        let hFontOld: HGDIOBJ = SelectObject(hdcMem, ((*this_).__base0.__base0.m_hFont).cast());
        let mut rcBack: RECT = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
    
        let mut cxText: INT = 0;
        let mut cyText: INT = 0;
        let mut cxContent: INT = 0;
        let mut cyContent: INT = 0;
        let mut cxyBorders: INT;
        let mut cxButton: INT;
        let mut cyButton: INT;
        if !(*this_).__base0.m_pszButtonText.is_null() {
            cxText = (*this_).__base0.m_TextSize.cx;
            cyText = (*this_).__base0.m_TextSize.cy;
        }
    
        if !(*this_).__base0.m_ButtonIcon.m_hIcon.is_null() {
            cxContent = (*this_).__base0.m_IconSize.cx;
            cyContent = (*this_).__base0.m_IconSize.cy;
        } else if !(*this_).__base0.m_hbmButton1.is_null() {
            cxContent = (*this_).m_BitmapSize.cx;
            cyContent = (*this_).m_BitmapSize.cy;
        }
    
        if ((*this_).__base0.__base0.m_style & UIF_BUTTON_VERTICAL) != 0 {
            cxyBorders = if cyText != 0 && cyContent != 0 {
                2
            } else {
                0
            };
    
            cxButton = cxContent;
            cyButton = cyText + cyContent + cxyBorders;
            if cxText > cxContent {
                cxButton = cxText;
            }
        } else {
            cxyBorders = if cxText != 0 && cxContent != 0 {
                2
            } else {
                0
            };
    
            cxButton = cxText + cxContent + cxyBorders;
            cyButton = cyContent;
            if cyText > cyButton {
                cyButton = cyText;
            }
        }
    
        let mut xOffset: INT;
        let mut yOffset: INT;
        if ((*this_).__base0.__base0.m_style & UIF_BUTTON_H_ALIGN_MASK) == UIF_BUTTON_H_ALIGN_CENTER {
            xOffset = (rcBack.left + rcBack.right - cxButton) / 2;
        } else if ((*this_).__base0.__base0.m_style & UIF_BUTTON_H_ALIGN_MASK) == UIF_BUTTON_H_ALIGN_RIGHT {
            xOffset = rcBack.right - cxText - 2;
        } else {
            xOffset = rcBack.left + 2;
        }
    
        if ((*this_).__base0.__base0.m_style & UIF_BUTTON_V_ALIGN_MASK) == UIF_BUTTON_V_ALIGN_MIDDLE {
            yOffset = (rcBack.top + rcBack.bottom - cyButton) / 2;
        } else if ((*this_).__base0.__base0.m_style & UIF_BUTTON_V_ALIGN_MASK) == UIF_BUTTON_V_ALIGN_BOTTOM {
            yOffset = rcBack.bottom - cyButton - 2;
        } else {
            yOffset = rcBack.top + 2;
        }
    
        let mut rc: RECT = RECT {
            left: xOffset,
            top: yOffset,
            right: xOffset + cxButton,
            bottom: cyButton + yOffset,
        };
        let mut offsetSize: SIZE = SIZE {
            cx: 0,
            cy: 0,
        };
        let dwDrawFlags: DWORD = (*this_).MakeDrawFlag();
        (*(*this_).__base0.__base0.m_pScheme).GetCtrlFaceOffset(
            if ((*this_).__base0.__base0.m_style & 0x200) != 0 {
                0xA5
            } else {
                0x54
            },
            dwDrawFlags,
            core::ptr::addr_of_mut!(offsetSize),
        );
        OffsetRect(core::ptr::addr_of_mut!(rc), offsetSize.cx, offsetSize.cy);
    
        let mut rcImage: RECT = core::mem::zeroed();
        let mut rcText: RECT = core::mem::zeroed();
        if ((*this_).__base0.__base0.m_style & UIF_BUTTON_VERTICAL) != 0 {
            rcImage.left = (rc.left + rc.right - cxContent) / 2;
            rcImage.top = rc.top;
            rcImage.right = rcImage.left + cxContent;
            rcImage.bottom = rc.top + cyContent;
            rcText.left = (rc.left + rc.right - cxText) / 2;
            rcText.top = rc.bottom - cyText;
            rcText.right = rcText.left + cxText;
            rcText.bottom = rc.bottom;
        } else {
            rcImage.left = rc.left;
            rcImage.top = (rc.top + rc.bottom - cyContent) / 2;
            rcImage.bottom = rcImage.top + cyContent;
            rcImage.right = rc.left + cxContent;
            rcText.left = rc.right - cxText;
            rcText.top = (rc.top + rc.bottom - cyText) / 2;
            rcText.right = rc.right;
            rcText.bottom = rcText.top + cyText;
        }
    
        if (((*this_).__base0.__base0.IsRTL()) != 0) {
            (*(*this_).__base0.__base0.m_pScheme).m_bMirroring = (TRUE) as i32;
        }
    
        (*(*this_).__base0.__base0.m_pScheme).DrawCtrlBkgd(
            hdcMem,
            core::ptr::addr_of_mut!(rcBack),
            if ((*this_).__base0.__base0.m_style & 0x200) != 0 {
                0xA5
            } else {
                0x54
            },
            dwDrawFlags,
        );
        if !(*this_).__base0.m_pszButtonText.is_null() {
            (*(*this_).__base0.__base0.m_pScheme).DrawCtrlText(
                hdcMem,
                core::ptr::addr_of_mut!(rcText),
                (*this_).__base0.m_pszButtonText,
                -1,
                dwDrawFlags,
                (((*this_).__base0.__base0.m_style & UIF_BUTTON_VERTICAL) != 0) as BOOL,
            );
        }
    
        if !(*this_).__base0.m_ButtonIcon.m_hIcon.is_null() {
            (*(*this_).__base0.__base0.m_pScheme).DrawCtrlIcon(
                hdcMem,
                core::ptr::addr_of_mut!(rcImage),
                (*this_).__base0.m_ButtonIcon.m_hIcon,
                dwDrawFlags,
                core::ptr::addr_of_mut!((*this_).__base0.m_IconSize),
            );
        } else if !(*this_).__base0.m_hbmButton1.is_null() {
            (*(*this_).__base0.__base0.m_pScheme).DrawCtrlBitmap(
                hdcMem,
                core::ptr::addr_of_mut!(rcImage),
                (*this_).__base0.m_hbmButton1,
                (*this_).__base0.m_hbmButton2,
                dwDrawFlags,
            );
        }
    
        if (((*this_).__base0.__base0.IsRTL()) != 0) {
            (*(*this_).__base0.__base0.m_pScheme).m_bMirroring = (FALSE) as i32;
        }
    
        (*(*this_).__base0.__base0.m_pScheme).DrawCtrlEdge(
            hdcMem,
            core::ptr::addr_of_mut!(rcBack),
            if ((*this_).__base0.__base0.m_style & 0x200) != 0 {
                0xA5
            } else {
                0x54
            },
            dwDrawFlags,
        );
    
        BitBlt(
            hDC,
            (*this_).__base0.__base0.m_rc.left,
            (*this_).__base0.__base0.m_rc.top,
            width,
            height,
            hdcMem,
            0,
            0,
            SRCCOPY,
        );
        SelectObject(hdcMem, hFontOld);
        SelectObject(hdcMem, hbmOld);
        DeleteObject((hbmMem).cast());
    }
}

impl CUIFToolbarMenuButton {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFToolbarButton, nObjectID: DWORD, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::addr_of_mut!((*pParent).__base0), nObjectID, prc, style);
        (*this_).__init_vtables();
        {
            (*this_).m_pToolbarButton = pParent;
    
            let hFont: HFONT = CreateFontW(8, 8, 0, 0, (FW_NORMAL as i32), 0, 0, 0, SYMBOL_CHARSET,
                0, 0, 0, 0, CPP_COM_WSTR_B728528EE8A8.as_ptr());
            (*this_).__base0.__base0.__base0.SetFont(hFont);
            (*this_).__base0.__base0.SetText(CPP_COM_WSTR_229BA167C275.as_ptr()); // downward triangle
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        DeleteObject(((*this_).__base0.__base0.__base0.m_hFont).cast());
        (*this_).__base0.__base0.__base0.SetFont(core::ptr::null_mut());
    }
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        (*this_).__base0.__base0.OnLButtonUp__body(x, y); // CUIFButton::OnLButtonUp, non-virtual
        (*(*this_).m_pToolbarButton).OnUnknownMouse0();
    }
    pub unsafe fn OnSetCursor__body(&mut self, uMsg: UINT, x: LONG, y: LONG) -> BOOL {
        let this_: *mut Self = self;
        (*(*this_).m_pToolbarButton).__base0.OnSetCursor(uMsg, x, y);
        return (FALSE) as BOOL;
    }
}

impl CUIFToolbarButtonElement {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFToolbarButton, nObjectID: DWORD, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::addr_of_mut!((*pParent).__base0), nObjectID, prc, style);
        (*this_).__init_vtables();
        {
            (*this_).m_pToolbarButton = pParent;
        }
    }
    pub unsafe fn GetToolTip__body(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        if !(*this_).m_pToolbarButton.is_null() {
            return (*(*this_).m_pToolbarButton).GetToolTip();
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        (*this_).__base0.__base0.OnLButtonUp(x, y);
        if ((*(*this_).m_pToolbarButton).m_dwToolbarButtonFlags & 0x30000) == 0x20000 {
            (*(*this_).m_pToolbarButton).OnUnknownMouse0();
        } else {
            (*(*this_).m_pToolbarButton).OnLeftClick();
        }
    }
    pub unsafe fn OnRButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if ((*(*this_).m_pToolbarButton).m_dwToolbarButtonFlags & 0x30000) != 0x20000 {
            (*(*this_).m_pToolbarButton).OnRightClick();
        }
    }
}

impl CUIFToolbarButton {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFObject, nObjectID: DWORD, prc: LPCRECT, style: DWORD, dwToolbarButtonFlags: DWORD, pszUnknownText: LPCWSTR) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pParent, nObjectID, prc, style);
        (*this_).__init_vtables();
        {
            (*this_).m_dwToolbarButtonFlags = dwToolbarButtonFlags;
            (*this_).m_pszUnknownText = pszUnknownText;
        }
    }
    pub unsafe fn Init(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let mut rc1: RECT;
        let mut rc2: RECT;
        rc2 = (*this_).__base0.m_rc;
        rc1 = rc2;
    
        if ((*this_).m_dwToolbarButtonFlags & 0x30000) == 0x30000
        {
            rc1.right -= 12;
            rc2.left = rc1.right + 1;
        }
    
        let mut style: DWORD = UIF_BUTTON_LARGE_ICON | UIF_BUTTON_V_ALIGN_MIDDLE | UIF_BUTTON_H_ALIGN_CENTER;
        if (*this_).m_dwToolbarButtonFlags & 0x2000 != 0 {
            style |= 0x10;
        }
        if (*this_).m_dwToolbarButtonFlags & 0x80000 != 0 {
            style |= UIF_BUTTON_VERTICAL;
        }
        (*this_).m_pToolbarButtonElement = CUIFToolbarButtonElement__new_in_0(
            cpp_alloc_CicNoThrow::<CUIFToolbarButtonElement>(),
            this_,
            (*this_).__base0.m_nObjectID, // ID: CUIFObject::m_nObjectID
            core::ptr::addr_of_mut!(rc1),
            style,
        );
        if (*this_).m_pToolbarButtonElement.is_null() {
            return (FALSE) as BOOL;
        }
    
        (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0.Initialize();
        (*this_).__base0.AddUIObj(core::ptr::addr_of_mut!(
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0
        ));
    
        if (*this_).__base0.m_bEnable == 0 {
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.Enable((FALSE) as BOOL);
        }
    
        if ((*this_).m_dwToolbarButtonFlags & 0x30000) == 0x30000
        {
            style = UIF_BUTTON_LARGE_ICON | UIF_BUTTON_H_ALIGN_CENTER;
            if (*this_).m_dwToolbarButtonFlags & 0x80000 != 0 {
                style |= UIF_BUTTON_VERTICAL;
            }
    
            (*this_).m_pToolbarMenuButton = CUIFToolbarMenuButton__new_in_0(
                cpp_alloc_CicNoThrow::<CUIFToolbarMenuButton>(),
                this_,
                0,
                core::ptr::addr_of!(rc2),
                style,
            );
            if (*this_).m_pToolbarMenuButton.is_null() {
                return (FALSE) as BOOL;
            }
    
            (*(*this_).m_pToolbarMenuButton).__base0.__base0.__base0.Initialize();
            (*this_).__base0.AddUIObj(core::ptr::addr_of_mut!(
                (*(*this_).m_pToolbarMenuButton).__base0.__base0.__base0
            ));
    
            if (*this_).__base0.m_bEnable == 0 {
                (*(*this_).m_pToolbarMenuButton).__base0.__base0.Enable((FALSE) as BOOL);
            }
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn GetIcon(&mut self) -> HICON {
        let this_: *mut Self = self;
        return (*(*this_).m_pToolbarButtonElement).__base0.__base0.m_ButtonIcon.m_hIcon;
    }
    pub unsafe fn SetIcon(&mut self, hIcon: HICON) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolbarButtonElement).__base0.__base0.SetIcon(hIcon);
    }
    pub unsafe fn ClearWndObj__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pToolbarButtonElement.is_null() {
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0.ClearWndObj();
        }
        if !(*this_).m_pToolbarMenuButton.is_null() {
            (*(*this_).m_pToolbarMenuButton).__base0.__base0.__base0.ClearWndObj();
        }
    
        (*this_).__base0.ClearWndObj();
    }
    pub unsafe fn DetachWndObj__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pToolbarButtonElement.is_null() {
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0.DetachWndObj();
        }
        if !(*this_).m_pToolbarMenuButton.is_null() {
            (*(*this_).m_pToolbarMenuButton).__base0.__base0.__base0.DetachWndObj();
        }
    
        (*this_).__base0.DetachWndObj();
    }
    pub unsafe fn Enable__body(&mut self, bEnable: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.Enable(bEnable);
        if !(*this_).m_pToolbarButtonElement.is_null() {
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.Enable(bEnable);
        }
        if !(*this_).m_pToolbarMenuButton.is_null() {
            (*(*this_).m_pToolbarMenuButton).__base0.__base0.Enable(bEnable);
        }
    }
    pub unsafe fn GetToolTip__body(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        return (*this_).__base0.GetToolTip();
    }
    pub unsafe fn SetActiveTheme__body(&mut self, pszClassList: LPCWSTR, iPartId: INT, iStateId: INT) {
        let this_: *mut Self = self;
        if !(*this_).m_pToolbarButtonElement.is_null() {
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0.__base0.SetActiveTheme(pszClassList, iPartId, iStateId);
        }
        if !(*this_).m_pToolbarMenuButton.is_null() {
            (*(*this_).m_pToolbarMenuButton).__base0.__base0.__base0.__base0.SetActiveTheme(pszClassList, iPartId, iStateId);
        }
    
        (*this_).__base0.__base0.m_pszClassList = pszClassList;
        (*this_).__base0.__base0.m_iPartId = iPartId;
        (*this_).__base0.__base0.m_iStateId = iStateId;
    }
    pub unsafe fn SetFont__body(&mut self, hFont: HFONT) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0.SetFont(hFont);
    }
    pub unsafe fn SetRect__body(&mut self, prc: LPCRECT) {
        let this_: *mut Self = self;
        (*this_).__base0.SetRect(prc);
    
        let mut rc1: RECT;
        let mut rc2: RECT;
        rc2 = (*this_).__base0.m_rc;
        rc1 = rc2;
    
        if ((*this_).m_dwToolbarButtonFlags & 0x30000) == 0x30000
        {
            rc1.right -= 12;
            rc2.left = rc1.right + 1;
        }
    
        if !(*this_).m_pToolbarButtonElement.is_null() {
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0.SetRect(core::ptr::addr_of!(rc1));
        }
        if !(*this_).m_pToolbarMenuButton.is_null() {
            (*(*this_).m_pToolbarMenuButton).__base0.__base0.__base0.SetRect(core::ptr::addr_of!(rc2));
        }
    }
    pub unsafe fn SetToolTip__body(&mut self, pszToolTip: LPCWSTR) {
        let this_: *mut Self = self;
        (*this_).__base0.SetToolTip(pszToolTip);
        if !(*this_).m_pToolbarButtonElement.is_null() {
            (*(*this_).m_pToolbarButtonElement).__base0.__base0.__base0.SetToolTip(pszToolTip);
        }
        if !(*this_).m_pToolbarMenuButton.is_null() {
            (*(*this_).m_pToolbarMenuButton).__base0.__base0.__base0.SetToolTip(pszToolTip);
        }
    }
}

impl CUIFGripper {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFObject, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pParent, 0, prc, style);
        (*this_).__init_vtables();
        {
            (*this_).__base0.__base0.m_iStateId = 0;
            (*this_).__base0.__base0.m_pszClassList = CPP_COM_WSTR_672463267A33.as_ptr();
            if ((*this_).__base0.m_style & UIF_GRIPPER_VERTICAL) != 0 {
                (*this_).__base0.__base0.m_iPartId = (RP_GRIPPERVERT) as i32;
            } else {
                (*this_).__base0.__base0.m_iPartId = (RP_GRIPPER) as i32;
            }
        }
    }
    pub unsafe fn OnMouseMove__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if (*this_).__base0.IsCapture() != 0 {
            let mut pt: POINT = core::mem::zeroed();
            GetCursorPos(core::ptr::addr_of_mut!(pt));
            (*(*this_).__base0.m_pWindow).Move(pt.x - (*this_).m_ptGripper.x, pt.y - (*this_).m_ptGripper.y, -1, -1);
        }
    }
    pub unsafe fn OnLButtonDown__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        (*this_).__base0.StartCapture();
        (*this_).m_ptGripper.x = x;
        (*this_).m_ptGripper.y = y;
        ScreenToClient((*(*this_).__base0.m_pWindow).op_to_HWND(), core::ptr::addr_of_mut!((*this_).m_ptGripper));
        let mut rc: RECT = core::mem::zeroed();
        GetWindowRect((*(*this_).__base0.m_pWindow).op_to_HWND(), core::ptr::addr_of_mut!(rc));
        (*this_).m_ptGripper.x -= rc.left;
        (*this_).m_ptGripper.y -= rc.top;
    }
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if (*this_).__base0.IsCapture() != 0 {
            (*this_).__base0.EndCapture();
        }
    }
    pub unsafe fn OnPaintTheme__body(&mut self, hDC: HDC) -> BOOL {
        let this_: *mut Self = self;
        if (((*this_).__base0.__base0.EnsureThemeData((*(*this_).__base0.m_pWindow).op_to_HWND())) < 0) {
            return (FALSE) as BOOL;
        }
    
        if ((*this_).__base0.m_style & UIF_GRIPPER_VERTICAL) != 0 {
            (*this_).__base0.m_rc.top += 2;
            (*this_).__base0.m_rc.bottom -= 2;
        } else {
            (*this_).__base0.m_rc.left += 2;
            (*this_).__base0.m_rc.right -= 2;
        }
    
        if (((*this_).__base0.__base0.DrawThemeBackground(hDC, 1, core::ptr::addr_of_mut!((*this_).__base0.m_rc), core::ptr::null())) < 0) {
            return (FALSE) as BOOL;
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnPaintNoTheme__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if !(*this_).__base0.m_pScheme.is_null() {
            (*(*this_).__base0.m_pScheme).DrawDragHandle(
                hDC,
                core::ptr::addr_of_mut!((*this_).__base0.m_rc),
                (((*this_).__base0.m_style & UIF_GRIPPER_VERTICAL) != 0) as BOOL,
            );
            return;
        }
    
        let mut rc: RECT;
        if ((*this_).__base0.m_style & UIF_GRIPPER_VERTICAL) != 0 {
            rc = RECT {
                left: (*this_).__base0.m_rc.left,
                top: (*this_).__base0.m_rc.top + 1,
                right: (*this_).__base0.m_rc.right,
                bottom: (*this_).__base0.m_rc.top + 4,
            };
        } else {
            rc = RECT {
                left: (*this_).__base0.m_rc.left + 1,
                top: (*this_).__base0.m_rc.top,
                right: (*this_).__base0.m_rc.left + 4,
                bottom: (*this_).__base0.m_rc.bottom,
            };
        }
    
        DrawEdge(
            hDC,
            core::ptr::addr_of_mut!(rc),
            BDR_RAISEDINNER,
            BF_RECT,
        );
    }
    pub unsafe fn OnSetCursor__body(&mut self, uMsg: UINT, x: LONG, y: LONG) -> BOOL {
        let this_: *mut Self = self;
        let hCursor: HCURSOR = LoadCursor(core::ptr::null_mut(), IDC_SIZEALL);
        SetCursor(hCursor);
        return (TRUE) as BOOL;
    }
    pub unsafe fn SetStyle__body(&mut self, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.m_style = style;
        if ((*this_).__base0.m_style & UIF_GRIPPER_VERTICAL) != 0 {
            (*this_).__base0.__base0.SetActiveTheme(CPP_COM_WSTR_672463267A33.as_ptr(), (RP_GRIPPERVERT) as INT, 0);
        } else {
            (*this_).__base0.__base0.SetActiveTheme(CPP_COM_WSTR_672463267A33.as_ptr(), (RP_GRIPPER) as INT, 0);
        }
    }
}

impl CUIFWndFrame {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFObject, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pParent, 0, prc, style);
        (*this_).__init_vtables();
        {
            (*this_).__base0.__base0.m_iPartId = 7;
            (*this_).__base0.__base0.m_iStateId = 0;
            (*this_).__base0.__base0.m_pszClassList = CPP_COM_WSTR_DDBF4DF411E1.as_ptr();
            (*this_).m_dwHitTest = 0;
            (*this_).m_cyFrame = 0;
            (*this_).m_cxFrame = 0;
    
            if !(*this_).__base0.m_pScheme.is_null() {
                if ((*this_).__base0.m_style & 0xF) != 0
                    && ((*this_).__base0.m_style & 0xF) <= 2
                {
                    (*this_).m_cxFrame = (*(*this_).__base0.m_pScheme).CxSizeFrame();
                    (*this_).m_cyFrame = (*(*this_).__base0.m_pScheme).CySizeFrame();
                } else {
                    (*this_).m_cxFrame = (*(*this_).__base0.m_pScheme).CxWndBorder();
                    (*this_).m_cyFrame = (*(*this_).__base0.m_pScheme).CyWndBorder();
                }
            }
    
            (*this_).m_cxMin = GetSystemMetrics((SM_CXMIN as i32));
            (*this_).m_cyMin = GetSystemMetrics((SM_CYMIN as i32));
        }
    }
    pub unsafe fn GetFrameSize(&mut self, pSize: LPSIZE) {
        let this_: *mut Self = self;
        (*pSize).cx = (*this_).m_cxFrame;
        (*pSize).cy = (*this_).m_cyFrame;
    }
    pub unsafe fn HitTest(&mut self, x: LONG, y: LONG) -> DWORD {
        let this_: *mut Self = self;
        let mut dwFlags: DWORD = 0;
        if (*this_).__base0.m_rc.left <= x && x < (*this_).__base0.m_rc.left + (*this_).m_cxFrame {
            dwFlags |= 0x10;
        }
        if (*this_).__base0.m_rc.top <= y && y < (*this_).__base0.m_rc.top + (*this_).m_cyFrame {
            dwFlags |= 0x20;
        }
        if (*this_).__base0.m_rc.right - (*this_).m_cxFrame <= x && x < (*this_).__base0.m_rc.right {
            dwFlags |= 0x40;
        }
        if (*this_).__base0.m_rc.bottom - (*this_).m_cyFrame <= y && y < (*this_).__base0.m_rc.bottom {
            dwFlags |= 0x80;
        }
        return dwFlags;
    }
    pub unsafe fn OnMouseMove__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if (*this_).__base0.IsCapture() == 0 {
            return;
        }
    
        let mut Point: POINT = core::mem::zeroed();
        ClientToScreen((*(*this_).__base0.m_pWindow).op_to_HWND(), core::ptr::addr_of_mut!(Point));
    
        let mut rc: RECT = (*this_).m_rcWnd;
    
        if ((*this_).m_dwHitTest & 0x10) != 0 {
            rc.left = Point.x + (*this_).m_rcWnd.left - (*this_).m_ptHit.x;
        }
    
        if ((*this_).m_dwHitTest & 0x20) != 0 {
            rc.top = Point.y + (*this_).m_rcWnd.top - (*this_).m_ptHit.y;
        }
    
        if ((*this_).m_dwHitTest & 0x40) != 0 {
            rc.right = Point.x + (*this_).m_rcWnd.right - (*this_).m_ptHit.x;
            if rc.right <= rc.left + (*this_).m_cxMin {
                rc.right = rc.left + (*this_).m_cxMin;
            }
        }
    
        if ((*this_).m_dwHitTest & 0x80) != 0 {
            rc.bottom = Point.y + (*this_).m_rcWnd.bottom - (*this_).m_ptHit.y;
            if rc.bottom <= rc.top + (*this_).m_cyMin {
                rc.bottom = rc.top + (*this_).m_cyMin;
            }
        }
    
        (*(*this_).__base0.m_pWindow).Move(rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top);
    }
    pub unsafe fn OnLButtonDown__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        let mut Point: POINT = POINT { x: x, y: y };
        let mut hitTest: DWORD = (*this_).__base0.m_style & (*this_).HitTest(x, y);
        if !(hitTest != 0) {
            return;
        }
    
        ClientToScreen((*(*this_).__base0.m_pWindow).op_to_HWND(), core::ptr::addr_of_mut!(Point));
        (*this_).m_ptHit = Point;
        (*this_).__base0.m_pWindow = (*this_).__base0.m_pWindow;
        (*this_).m_dwHitTest = hitTest;
        GetWindowRect((*(*this_).__base0.m_pWindow).op_to_HWND(), core::ptr::addr_of_mut!((*this_).m_rcWnd));
        (*this_).__base0.StartCapture();
    }
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if (((*this_).__base0.IsCapture()) != 0) {
            (*this_).__base0.EndCapture();
        }
    }
    pub unsafe fn OnPaintTheme__body(&mut self, hDC: HDC) -> BOOL {
        let this_: *mut Self = self;
        if ((*this_).__base0.__base0.EnsureThemeData((*(*this_).__base0.m_pWindow).op_to_HWND()) < 0) {
            return (FALSE) as BOOL;
        }
    
        let mut rc: RECT = (*this_).__base0.m_rc;
        rc.right = (*this_).m_cxFrame;
        if ((*this_).__base0.__base0.DrawThemeEdge(hDC, 0, core::ptr::addr_of_mut!(rc), 5, 1, core::ptr::null_mut()) < 0) {
            return (FALSE) as BOOL;
        }
    
        rc = (*this_).__base0.m_rc;
        rc.left = rc.right - (*this_).m_cxFrame;
        if ((*this_).__base0.__base0.DrawThemeEdge(hDC, 0, core::ptr::addr_of_mut!(rc), 10, 4, core::ptr::null_mut()) < 0) {
            return (FALSE) as BOOL;
        }
    
        rc = (*this_).__base0.m_rc;
        rc.bottom = (*this_).m_cyFrame;
        if ((*this_).__base0.__base0.DrawThemeEdge(hDC, 0, core::ptr::addr_of_mut!(rc), 5, 2, core::ptr::null_mut()) < 0) {
            return (FALSE) as BOOL;
        }
    
        rc = (*this_).__base0.m_rc;
        rc.top = rc.bottom - (*this_).m_cyFrame;
        if ((*this_).__base0.__base0.DrawThemeEdge(hDC, 0, core::ptr::addr_of_mut!(rc), 10, 8, core::ptr::null_mut()) < 0) {
            return (FALSE) as BOOL;
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnPaintNoTheme__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_pScheme.is_null() {
            return;
        }
    
        let mut type_: DWORD = 0;
        if ((*this_).__base0.m_style & 0xF) == 1 {
            type_ = 1;
        } else if ((*this_).__base0.m_style & 0xF) == 2 {
            type_ = 2;
        }
    
        (*(*this_).__base0.m_pScheme).DrawWndFrame(
            hDC,
            core::ptr::addr_of_mut!((*this_).__base0.m_rc),
            type_,
            ((*this_).m_cxFrame) as DWORD,
            ((*this_).m_cyFrame) as DWORD,
        );
    }
    pub unsafe fn OnSetCursor__body(&mut self, uMsg: UINT, x: LONG, y: LONG) -> BOOL {
        let this_: *mut Self = self;
        let mut dwHitTest: DWORD = (*this_).m_dwHitTest;
        if (*this_).__base0.IsCapture() == 0 {
            dwHitTest = (*this_).__base0.m_style & (*this_).HitTest(x, y);
        }
    
        let mut pszCursor: LPTSTR = core::ptr::null_mut();
        match dwHitTest {
            0x30 | 0xC0 => {
                pszCursor = IDC_SIZENWSE;
            }
            0x90 | 0x60 => {
                pszCursor = IDC_SIZENESW;
            }
            0x10 | 0x40 => {
                pszCursor = IDC_SIZEWE as LPTSTR;
            }
            0x20 | 0x80 => {
                pszCursor = IDC_SIZENS as LPTSTR;
            }
            _ => {
                return (FALSE) as BOOL;
            }
        }
    
        let hCursor: HCURSOR = LoadCursor(core::ptr::null_mut(), pszCursor);
        SetCursor(hCursor);
        return (TRUE) as BOOL;
    }
}

impl CUIFBalloonButton {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CUIFObject, nObjectID: DWORD, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pParent, nObjectID, prc, style);
        (*this_).__init_vtables();
        {
        (*this_).m_nCommandID = 0;
    }
    }
    pub unsafe fn OnPaint__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        let mut rc: RECT = (*this_).__base0.__base0.m_rc;
        OffsetRect(core::ptr::addr_of_mut!(rc), -rc.left, -rc.top);
    
        let hMemDC: HDC = CreateCompatibleDC(hDC);
        let hbmMem: HBITMAP = CreateCompatibleBitmap(hDC, rc.right, rc.bottom);
        let hbmOld: HGDIOBJ = SelectObject(hMemDC, (hbmMem).cast());
    
        let bPressed: BOOL;
        let rgbShadow: COLORREF;
        let rgbBorder: COLORREF;
        if (*this_).__base0.m_uButtonStatus == 1
        {
            bPressed = (TRUE) as i32;
            rgbShadow = GetSysColor((COLOR_BTNSHADOW as i32));
            rgbBorder = GetSysColor((COLOR_BTNHIGHLIGHT as i32));
        }
        else
        {
            bPressed = (FALSE) as i32;
            if (*this_).__base0.m_uButtonStatus < 4
            {
                rgbShadow = GetSysColor((COLOR_BTNHIGHLIGHT as i32));
                rgbBorder = GetSysColor((COLOR_BTNSHADOW as i32));
            }
            else
            {
                rgbShadow = GetSysColor((COLOR_INFOBK as i32));
                rgbBorder = GetSysColor((COLOR_INFOBK as i32));
            }
        }
    
        let rgbInfoBk: COLORREF = GetSysColor((COLOR_INFOBK as i32));
        let hbrBack: HBRUSH = CreateSolidBrush(rgbInfoBk);
        FillRect(hMemDC, core::ptr::addr_of_mut!(rc), hbrBack);
        DeleteObject((hbrBack).cast());
    
        (*this_).DrawTextProc(hMemDC, core::ptr::addr_of_mut!(rc), bPressed);
    
        let hNullBrush: HBRUSH = GetStockObject((NULL_BRUSH as i32)) as HBRUSH;
        let hbrOld: HGDIOBJ = SelectObject(hMemDC, (hNullBrush).cast());
    
        let mut hPen: HPEN = CreatePen((PS_SOLID as i32), 0, rgbShadow);
        let mut hPenOld: HGDIOBJ = SelectObject(hMemDC, (hPen).cast());
        RoundRect(hMemDC, rc.left, rc.top, rc.right - 1, rc.bottom - 1, 6, 6);
        SelectObject(hMemDC, hPenOld);
        DeleteObject((hPen).cast());
    
        hPen = CreatePen((PS_SOLID as i32), 0, rgbBorder);
        hPenOld = SelectObject(hMemDC, (hPen).cast());
        RoundRect(hMemDC, rc.left + 1, rc.top + 1, rc.right, rc.bottom, 6, 6);
        SelectObject(hMemDC, hPenOld);
        DeleteObject((hPen).cast());
    
        hPen = CreatePen((PS_SOLID as i32), 0, GetSysColor((COLOR_BTNFACE as i32)));
        hPenOld = SelectObject(hMemDC, (hPen).cast());
        RoundRect(hMemDC, rc.left + 1, rc.top + 1, rc.right - 1, rc.bottom - 1, 6, 6);
        SelectObject(hMemDC, hPenOld);
        DeleteObject((hPen).cast());
    
        SelectObject(hMemDC, hbrOld);
        BitBlt(hDC, (*this_).__base0.__base0.m_rc.left, (*this_).__base0.__base0.m_rc.top, (*this_).__base0.__base0.m_rc.right - (*this_).__base0.__base0.m_rc.left, (*this_).__base0.__base0.m_rc.bottom - (*this_).__base0.__base0.m_rc.top,
                 hMemDC, rc.left, rc.top, SRCCOPY);
        SelectObject(hMemDC, hbmOld);
        DeleteObject((hbmMem).cast());
        DeleteDC(hMemDC);
    }
    pub unsafe fn DrawTextProc(&mut self, hDC: HDC, prc: LPCRECT, bPressed: BOOL) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_pszButtonText.is_null() {
            return;
        }
    
        let mut uFlags: UINT = DT_SINGLELINE;
    
        if ((*this_).__base0.__base0.m_style & UIF_BUTTON_H_ALIGN_MASK) == UIF_BUTTON_H_ALIGN_CENTER {
            uFlags |= DT_CENTER;
        } else if ((*this_).__base0.__base0.m_style & UIF_BUTTON_H_ALIGN_MASK) == UIF_BUTTON_H_ALIGN_RIGHT {
            uFlags |= DT_RIGHT;
        }
    
        if ((*this_).__base0.__base0.m_style & UIF_BUTTON_V_ALIGN_MASK) == UIF_BUTTON_V_ALIGN_MIDDLE {
            uFlags |= DT_VCENTER;
        } else if ((*this_).__base0.__base0.m_style & UIF_BUTTON_V_ALIGN_MASK) == UIF_BUTTON_V_ALIGN_BOTTOM {
            uFlags |= DT_BOTTOM;
        }
    
        let rgbOldColor: COLORREF = SetTextColor(hDC, GetSysColor((COLOR_BTNTEXT as i32)));
        let nOldBkMode: INT = SetBkMode(hDC, (TRANSPARENT as i32));
    
        let mut rc: RECT = *prc;
        if bPressed != 0 {
            OffsetRect(core::ptr::addr_of_mut!(rc), 1, 1);
        }
    
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.__base0.m_hFont).cast());
        DrawTextW(hDC, (*this_).__base0.m_pszButtonText, -1, core::ptr::addr_of_mut!(rc), uFlags);
        SelectObject(hDC, hFontOld);
    
        SetBkMode(hDC, nOldBkMode);
        SetTextColor(hDC, rgbOldColor);
    }
}

impl CUIFBalloonWindow {
    pub unsafe fn __ctor_0(&mut self, hInst: HINSTANCE, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(hInst, style);
        (*this_).__init_vtables();
        {
            (*this_).m_dwUnknown6 = (1 as u32).wrapping_neg();
            (*this_).m_nActionID = (-1isize) as WPARAM;
            (*this_).m_hRgn = core::ptr::null_mut();
            (*this_).m_pszBalloonText = core::ptr::null_mut();
            (*this_).m_bHasTextColor = (FALSE) as i32;
            (*this_).m_bHasBkColor = (*this_).m_bHasTextColor;
            (*this_).m_rgbBkColor = 0;
            (*this_).m_rgbTextColor = 0;
            (*this_).m_ptTarget.y = 0;
            (*this_).m_ptTarget.x = (*this_).m_ptTarget.y;
            ZeroMemory!(
                core::ptr::addr_of_mut!((*this_).m_rcExclude) as *mut core::ffi::c_void,
                core::mem::size_of::<RECT>(),
            );
            (*this_).m_dwUnknown7 = 0;
            (*this_).m_nBalloonType = 0;
            (*this_).m_dwUnknown8[0 as usize] = 0;
            (*this_).m_dwUnknown8[1 as usize] = 0;
            (*this_).m_ptBalloon.y = 0;
            (*this_).m_ptBalloon.x = (*this_).m_ptBalloon.y;
            (*this_).m_cButtons = 0;
            (*this_).m_hwndNotif = core::ptr::null_mut();
            (*this_).m_uNotifMsg = 0;
            (*this_).m_rcMargin.left = 8;
            (*this_).m_rcMargin.top = 8;
            (*this_).m_rcMargin.right = 8;
            (*this_).m_rcMargin.bottom = 8;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pszBalloonText.is_null() {
            cpp_delete_array((*this_).m_pszBalloonText);
            (*this_).m_pszBalloonText = core::ptr::null_mut();
        }
    }
    pub unsafe fn Initialize__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).__base0.Initialize();
    
        if ((*this_).__base0.__base0.m_style & UIF_BALLOON_WINDOW_TYPE_MASK) == UIF_BALLOON_WINDOW_OK {
            (*this_).AddButton(IDOK);
        } else if ((*this_).__base0.__base0.m_style & UIF_BALLOON_WINDOW_TYPE_MASK) == UIF_BALLOON_WINDOW_YESNO {
            (*this_).AddButton(IDYES);
            (*this_).AddButton(IDNO);
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnCreate__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        let _ = hWnd;
        (*this_).m_nActionID = (0usize).wrapping_sub(1 as usize) as WPARAM;
        (*this_).AdjustPos();
    }
    pub unsafe fn OnDestroy__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        (*this_).SendNotification((*this_).m_nActionID);
        (*this_).DoneWindowRegion();
    }
    pub unsafe fn OnKeyDown__body(&mut self, hWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        let mut pButton: *mut CUIFBalloonButton = core::ptr::null_mut();
    
        match wParam {
            _ if wParam == VK_RETURN as WPARAM => {
                pButton = (*this_).FindUIObject(0) as *mut CUIFBalloonButton;
            }
            _ if wParam == VK_ESCAPE as WPARAM => {
                (*this_).m_nActionID = (-(1 as isize)) as WPARAM;
                DestroyWindow((*this_).__base0.m_hWnd);
                return;
            }
            _ if wParam == ('Y' as u16) as WPARAM => {
                pButton = (*this_).FindButton(IDYES);
            }
            _ if wParam == ('N' as u16) as WPARAM => {
                pButton = (*this_).FindButton(IDNO);
            }
            _ => {}
        }
    
        if pButton.is_null() {
            return;
        }
    
        (*this_).m_nActionID = (*pButton).m_nCommandID as WPARAM;
        DestroyWindow((*this_).__base0.m_hWnd);
    }
    pub unsafe fn OnObjectNotify__body(&mut self, pObject: *mut CUIFObject, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        let mut pButton: *mut CUIFBalloonButton = pObject as *mut CUIFBalloonButton;
        (*this_).m_nActionID = (*pButton).m_nCommandID as WPARAM;
        DestroyWindow((*this_).__base0.m_hWnd);
        return 0;
    }
    pub unsafe fn OnPaint__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.__base0.GetRect(core::ptr::addr_of_mut!(rc));
        (*this_).PaintFrameProc(hDC, core::ptr::addr_of!(rc));
    
        match (*this_).m_nBalloonType {
            1 => {
                rc.top += 16;
            }
            2 => {
                rc.right -= 16;
            }
            3 => {
                rc.left += 16;
            }
            _ => {
                rc.bottom -= 16;
            }
        }
    
        let mut rcMargin: RECT = core::mem::zeroed();
        (*this_).GetMargin(core::ptr::addr_of_mut!(rcMargin));
    
        rc.left += rcMargin.left;
        rc.top += rcMargin.top;
        rc.right -= rcMargin.right;
        rc.bottom -= rcMargin.bottom;
    
        (*this_).PaintMessageProc(hDC, core::ptr::addr_of_mut!(rc), (*this_).m_pszBalloonText);
    }
    pub unsafe fn AddButton(&mut self, nCommandId: UINT) {
        let this_: *mut Self = self;
        let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if !((IDOK <= nCommandId) && (nCommandId <= IDNO)) {
            return;
        }
    
        let pButton: *mut CUIFBalloonButton = CUIFBalloonButton__new_in_0(cpp_alloc_CicNoThrow::<CUIFBalloonButton>(), (this_).cast(), (*this_).m_cButtons, core::ptr::addr_of_mut!(rc), 5);
        let _ = (core::ptr::addr_of_mut!(rc), 5);
        if pButton.is_null() {
            return;
        }
    
        (*pButton).__base0.__base0.Initialize();
        (*pButton).m_nCommandID = nCommandId;
    
        let mut pszText: LPCWSTR;
        match nCommandId {
            IDOK => {
                pszText = CPP_COM_WSTR_09C0D8FDD014.as_ptr();
            }
            IDCANCEL => {
                pszText = CPP_COM_WSTR_4CA0338A5BD8.as_ptr();
            }
            IDABORT => {
                pszText = CPP_COM_WSTR_FD9E5FF53298.as_ptr();
            }
            IDRETRY => {
                pszText = CPP_COM_WSTR_61F3B37003C0.as_ptr();
            }
            IDIGNORE => {
                pszText = CPP_COM_WSTR_1F367FB724B3.as_ptr();
            }
            IDYES => {
                pszText = CPP_COM_WSTR_893EDE57DDD9.as_ptr();
            }
            _ => {
                pszText = CPP_COM_WSTR_A944EB43577D.as_ptr();
            }
        }
    
        (*pButton).__base0.SetText(pszText);
    
        (*this_).__base0.__base0.AddUIObj(core::ptr::addr_of_mut!((*pButton).__base0.__base0));
        (*this_).m_cButtons += 1;
    }
    pub unsafe fn AdjustPos(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn CreateRegion(&mut self, prc: LPCRECT) -> HRGN {
        let this_: *mut Self = self;
        let mut Points: [POINT; 4] = core::mem::zeroed();
        let mut hRgn: HRGN = core::mem::zeroed();
        let mut bFlag: BOOL = (FALSE) as i32;
        let mut x: LONG = 0;
        let mut y: LONG = 0;
    
        match (*this_).m_nBalloonType {
            1 => {
                hRgn = CreateRoundRectRgn((*prc).left, (*prc).top + 16, (*prc).right, (*prc).bottom, 16, 16);
                y = (*prc).top + 16;
                bFlag = (TRUE) as i32;
            }
    
            2 => {
                hRgn = CreateRoundRectRgn((*prc).left, (*prc).top, (*prc).right - 16, (*prc).bottom, 16, 16);
                x = (*prc).right - 17;
            }
    
            3 => {
                hRgn = CreateRoundRectRgn((*prc).left + 16, (*prc).top, (*prc).right, (*prc).bottom, 16, 16);
                x = (*prc).left + 16;
            }
    
            _ => {
                hRgn = CreateRoundRectRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom - 16, 16, 16);
                y = (*prc).bottom - 17;
                bFlag = (TRUE) as i32;
            }
        }
    
        if bFlag != 0 {
            x = (*this_).m_ptBalloon.x;
            Points[1].x = x;
            Points[1].y = (*this_).m_ptBalloon.y;
            {
                let v: LONG = y;
                Points[3].y = v;
                Points[2].y = v;
                Points[0].y = v;
            }
            Points[2].x = x + 10 * (2 * (if (*this_).m_dwUnknown8[0] == 0 { 1 } else { 0 }) - 1);
        } else {
            Points[2].x = x;
            {
                let v: LONG = (*this_).m_ptBalloon.y;
                Points[3].y = v;
                Points[1].y = v;
                Points[0].y = v;
                y = v;
            }
            Points[1].x = (*this_).m_ptBalloon.x;
            Points[2].y = y + 10 * (2 * (if (*this_).m_dwUnknown8[0] == 2 { 1 } else { 0 }) - 1);
        }
    
        Points[3].x = x;
        Points[0].x = x;
    
        let hPolygonRgn: HRGN = CreatePolygonRgn(Points.as_ptr(), (_countof!(Points)) as i32, (WINDING as i32));
        CombineRgn(hRgn, hRgn, hPolygonRgn, (RGN_OR as i32));
        DeleteObject((hPolygonRgn).cast());
        hRgn
    }
    pub unsafe fn DoneWindowRegion(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hRgn.is_null() {
            SetWindowRgn((*this_).__base0.m_hWnd, core::ptr::null_mut(), (TRUE) as BOOL);
            DeleteObject(((*this_).m_hRgn).cast());
            (*this_).m_hRgn = core::ptr::null_mut();
        }
    }
    pub unsafe fn FindButton(&mut self, nCommandID: UINT) -> *mut CUIFBalloonButton {
        let this_: *mut Self = self;
        let mut iButton: UINT = 0;
        while iButton < (*this_).m_cButtons {
            let pButton: *mut CUIFBalloonButton = (*this_).FindUIObject(iButton) as *mut CUIFBalloonButton;
            if !pButton.is_null() && ((*pButton).m_nCommandID == nCommandID) {
                return pButton;
            }
            iButton += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn FindUIObject(&mut self, nObjectID: UINT) -> *mut CUIFObject {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).__base0.__base0.m_ObjectArray.size() {
            let pObject: *mut CUIFObject = *(core::ptr::addr_of_mut!((*this_).__base0.__base0.m_ObjectArray.__base0) as *mut *mut CUIFObject).add(iItem);
            if (*pObject).m_nObjectID == nObjectID {
                return pObject;
            }
            iItem += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn GetBalloonBkColor(&mut self) -> COLORREF {
        let this_: *mut Self = self;
        if (*this_).m_bHasBkColor != 0 {
            return (*this_).m_rgbBkColor;
        } else {
            return GetSysColor((COLOR_INFOBK as i32));
        }
    }
    pub unsafe fn GetBalloonTextColor(&mut self) -> COLORREF {
        let this_: *mut Self = self;
        if (*this_).m_bHasTextColor != 0 {
            return (*this_).m_rgbTextColor;
        } else {
            return GetSysColor((COLOR_INFOTEXT as i32));
        }
    }
    pub unsafe fn GetButtonSize(&mut self, pSize: LPSIZE) {
        let this_: *mut Self = self;
        static DISPLAY_TEXT: [u16; 8] = ['D' as u16, 'I' as u16, 'S' as u16, 'P' as u16, 'L' as u16, 'A' as u16, 'Y' as u16, 0];
        let hDisplayDC: HDC = CreateDC((DISPLAY_TEXT.as_ptr()), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    
        let mut tm: TEXTMETRIC = core::mem::zeroed();
        let hFontOld: HGDIOBJ = SelectObject(hDisplayDC, ((*this_).__base0.__base0.m_hFont).cast());
        GetTextMetrics(hDisplayDC, core::ptr::addr_of_mut!(tm));
        SelectObject(hDisplayDC, hFontOld);
        DeleteDC(hDisplayDC);
    
        (*pSize).cx = 16 * tm.tmAveCharWidth;
        (*pSize).cy = tm.tmHeight + 10;
    }
    pub unsafe fn GetMargin(&mut self, prcMargin: LPRECT) {
        let this_: *mut Self = self;
        if !prcMargin.is_null() {
            *prcMargin = (*this_).m_rcMargin;
        }
    }
    pub unsafe fn SetExcludeRect(&mut self, prcExclude: LPCRECT) {
        let this_: *mut Self = self;
        (*this_).m_rcExclude = *prcExclude;
        (*this_).AdjustPos();
    }
    pub unsafe fn SetTargetPos(&mut self, ptTarget: POINT) {
        let this_: *mut Self = self;
        (*this_).m_ptTarget = ptTarget;
        (*this_).AdjustPos();
    }
    pub unsafe fn SetText(&mut self, mut pszText: LPCWSTR) {
        let this_: *mut Self = self;
        if !(*this_).m_pszBalloonText.is_null() {
            cpp_delete_array((*this_).m_pszBalloonText);
            (*this_).m_pszBalloonText = core::ptr::null_mut();
        }
    
        if pszText.is_null() {
            pszText = CPP_COM_WSTR_96A296D224F2.as_ptr();
        }
    
        let cch: usize = wcslen(pszText);
        (*this_).m_pszBalloonText = cpp_new_array_CicNoThrow::<WCHAR>(cch + 1);
        if !(*this_).m_pszBalloonText.is_null() {
            lstrcpynW((*this_).m_pszBalloonText, pszText, (cch + 1) as i32);
        }
    
        (*this_).AdjustPos();
    }
    pub unsafe fn InitWindowRegion(&mut self) {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.__base0.GetRect(core::ptr::addr_of_mut!(rc));
        (*this_).m_hRgn = (*this_).CreateRegion(core::ptr::addr_of!(rc));
        if !(*this_).m_hRgn.is_null() {
            SetWindowRgn((*this_).__base0.m_hWnd, (*this_).m_hRgn, (TRUE) as BOOL);
        }
    }
    pub unsafe fn LayoutObject(&mut self) {
        let this_: *mut Self = self;
        let mut size: SIZE = core::mem::zeroed();
        (*this_).GetButtonSize(core::ptr::addr_of_mut!(size));
    
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.__base0.GetRect(core::ptr::addr_of_mut!(rc));
    
        match (*this_).m_nBalloonType {
            1 => {
                rc.top += 16;
            }
            2 => {
                rc.right -= 16;
            }
            3 => {
                rc.left += 16;
            }
            _ => {
                rc.bottom -= 16;
            }
        }
    
        let mut rcMargin: RECT = core::mem::zeroed();
        (*this_).GetMargin(core::ptr::addr_of_mut!(rcMargin));
        rc.left += rcMargin.left;
        rc.top += rcMargin.top;
        rc.right -= rcMargin.right;
        rc.bottom -= rcMargin.bottom;
    
        let xLeft: LONG = ((rc.left.wrapping_add(rc.right) as UINT).wrapping_sub(
            (size.cx as UINT).wrapping_mul(
                (((*this_).m_cButtons.wrapping_sub(1) / 2)
                    .wrapping_sub((*this_).m_cButtons)),
            ),
        ) / 2) as LONG;
    
        let mut iButton: UINT = 0;
        while iButton < (*this_).m_cButtons {
            'body: {
                let UIObject: *mut CUIFObject = (*this_).FindUIObject(iButton);
                if UIObject.is_null() {
                    break 'body;
                }
    
                rcMargin.left = (xLeft as UINT).wrapping_add(
                    iButton.wrapping_mul(((size.cx * 3 / 2) as UINT)),
                ) as LONG;
                rcMargin.top = rc.bottom - size.cy;
                rcMargin.right = rcMargin.left + size.cx;
                rcMargin.bottom = rc.bottom;
    
                (*UIObject).SetRect(core::ptr::addr_of!(rcMargin));
                (*UIObject).Show((TRUE) as BOOL);
            }
            iButton += 1;
        }
    }
    pub unsafe fn PaintFrameProc(&mut self, hDC: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let hRgn: HRGN = (*this_).CreateRegion(prc);
        let hbrBack: HBRUSH = CreateSolidBrush((*this_).GetBalloonBkColor());
        let hbrFrame: HBRUSH = CreateSolidBrush(GetSysColor((COLOR_WINDOWFRAME as i32)));
        FillRgn(hDC, hRgn, hbrBack);
        FrameRgn(hDC, hRgn, hbrFrame, 1, 1);
        DeleteObject((hbrBack).cast());
        DeleteObject((hbrFrame).cast());
        DeleteObject((hRgn).cast());
    }
    pub unsafe fn PaintMessageProc(&mut self, hDC: HDC, prc: LPRECT, pszText: LPCWSTR) {
        let this_: *mut Self = self;
        let hFontOld: HGDIOBJ = SelectObject(hDC, ((*this_).__base0.__base0.m_hFont).cast());
        let rgbOldColor: COLORREF = SetTextColor(hDC, (*this_).GetBalloonTextColor());
        let nOldBkMode: INT = SetBkMode(hDC, (TRANSPARENT as i32));
        DrawTextW(hDC, pszText, -1, prc, DT_WORDBREAK);
        SelectObject(hDC, hFontOld);
        SetTextColor(hDC, rgbOldColor);
        SetBkMode(hDC, nOldBkMode);
    }
    pub unsafe fn SendNotification(&mut self, wParam: WPARAM) {
        let this_: *mut Self = self;
        if !(*this_).m_hwndNotif.is_null() {
            PostMessage((*this_).m_hwndNotif, (*this_).m_uNotifMsg, wParam, 0);
        }
    }
}

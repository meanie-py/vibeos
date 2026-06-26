//! win32ss/gdi/gdi32/objects/pen.c → Rust. Pen-creation client wrappers:
//! CreatePenIndirect / CreatePen.
//
// NEEDS-IMPORT: GetStockObject — WINAPI export defined in objects/gdiobj.c → gdiobj.rs
//   (extern "system" / i386 `_GetStockObject@4`). Declared extern below.

use win32k_sys::win32k::*;

// ── Cross-file gdi32 client function this file calls (defined in gdiobj.rs). ──
//    objects/gdiobj.c   HGDIOBJ WINAPI GetStockObject(INT) → extern "system"
extern "system" {
    fn GetStockObject(fnObject: INT) -> HGDIOBJ;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreatePenIndirect(lplgpn: *const LOGPEN) -> HPEN {
    /* Note same behoir as Windows 2000/XP/VISTA, they do not care if  plgpn is NULL, it will crash */
    CreatePen(
        (*lplgpn).lopnStyle as INT,
        (*lplgpn).lopnWidth.x as INT,
        (*lplgpn).lopnColor,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreatePen(
    mut nPenStyle: INT,
    nWidth: INT,
    crColor: COLORREF,
) -> HPEN {
    /*    HPEN hPen;
        PBRUSH_ATTR Pen_Attr;
    */
    if nPenStyle < PS_SOLID as INT {
        nPenStyle = PS_SOLID as INT;
    }
    if nPenStyle > PS_DASHDOTDOT as INT {
        if nPenStyle == PS_NULL as INT {
            return GetStockObject(NULL_PEN as INT) as HPEN;
        }
        if nPenStyle != PS_INSIDEFRAME as INT {
            nPenStyle = PS_SOLID as INT;
        }
    }
    // #if 0 block (hGetPEBHandle / GdiGetHandleUserData fast path) is disabled in C.
    NtGdiCreatePen(nPenStyle, nWidth, crColor, core::ptr::null_mut())
}

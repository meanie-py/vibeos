#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};

// Debug-channel macros — no kernel symbol, no-op.
#[allow(unused_macros)]
#[allow(unused_macros)]
#[allow(unused_macros)]

// PtrToUlong (basetsd): (ULONG)(ULONG_PTR)(p)
#[inline(always)]
fn PtrToUlong<T>(p: *const T) -> u32 {
    p as usize as u32
}

// RGB(r,g,b) macro (wingdi.h): (COLORREF)((BYTE)r | ((WORD)(BYTE)g << 8) | ((DWORD)(BYTE)b << 16))
#[inline(always)]
unsafe fn RGB(r: u32, g: u32, b: u32) -> COLORREF {
    ((r & 0xff) | ((g & 0xff) << 8) | ((b & 0xff) << 16)) as COLORREF
}

// RECTL_vOffsetRect (rect.h FORCEINLINE): offset all four edges.
#[inline(always)]
unsafe fn RECTL_vOffsetRect(prcl: LPRECT, cx: i32, cy: i32) {
    (*prcl).left += cx;
    (*prcl).right += cx;
    (*prcl).top += cy;
    (*prcl).bottom += cy;
}

// gpsi tail (anonymous PERUSERSERVERINFO member): tagSERVERINFO ends with
// __bindgen_padding_0 representing it; cast that tail to access its fields
// (hbrGray etc.). Matches focus.rs / nonclient.rs.
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// ---- file-scope preamble (translated) ----
pub static LTInnerNormal: [core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, -1,
    -1, COLOR_3DDKSHADOW as core::ffi::c_schar, COLOR_3DDKSHADOW as core::ffi::c_schar, -1,
    -1, -1, -1, -1,
];

pub static LTOuterNormal: [core::ffi::c_schar; 16] = [
    -1, COLOR_3DLIGHT as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, -1,
    COLOR_BTNHIGHLIGHT as core::ffi::c_schar, COLOR_3DLIGHT as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, -1,
    COLOR_3DDKSHADOW as core::ffi::c_schar, COLOR_3DLIGHT as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, -1,
    -1, COLOR_3DLIGHT as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, -1,
];

pub static RBInnerNormal: [core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, -1,
    -1, COLOR_3DLIGHT as core::ffi::c_schar, COLOR_3DLIGHT as core::ffi::c_schar, -1,
    -1, -1, -1, -1,
];

pub static RBOuterNormal: [core::ffi::c_schar; 16] = [
    -1, COLOR_3DDKSHADOW as core::ffi::c_schar, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, -1,
    COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_3DDKSHADOW as core::ffi::c_schar, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, -1,
    COLOR_3DLIGHT as core::ffi::c_schar, COLOR_3DDKSHADOW as core::ffi::c_schar, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, -1,
    -1, COLOR_3DDKSHADOW as core::ffi::c_schar, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, -1,
];

// #define RBInnerSoft RBInnerNormal  /* These are the same */
// #define RBOuterSoft RBOuterNormal
pub static RBInnerSoft: [core::ffi::c_schar; 16] = RBInnerNormal;
pub static RBOuterSoft: [core::ffi::c_schar; 16] = RBOuterNormal;

pub static LTInnerSoft: [core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_3DLIGHT as core::ffi::c_schar, COLOR_3DLIGHT as core::ffi::c_schar, -1,
    -1, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, -1,
    -1, -1, -1, -1,
];

pub static LTOuterSoft: [core::ffi::c_schar; 16] = [
    -1, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, COLOR_3DDKSHADOW as core::ffi::c_schar, -1,
    COLOR_3DLIGHT as core::ffi::c_schar, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, COLOR_3DDKSHADOW as core::ffi::c_schar, -1,
    COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, COLOR_3DDKSHADOW as core::ffi::c_schar, -1,
    -1, COLOR_BTNHIGHLIGHT as core::ffi::c_schar, COLOR_3DDKSHADOW as core::ffi::c_schar, -1,
];

pub static LTRBOuterMono: [core::ffi::c_schar; 16] = [
    -1, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar,
    COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar,
    COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar,
    COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar, COLOR_WINDOWFRAME as core::ffi::c_schar,
];

pub static LTRBInnerMono: [core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOW as core::ffi::c_schar,
    -1, COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOW as core::ffi::c_schar,
    -1, COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOW as core::ffi::c_schar, COLOR_WINDOW as core::ffi::c_schar,
];

pub static LTRBOuterFlat: [core::ffi::c_schar; 16] = [
    -1, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar,
    COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar,
    COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar,
    COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar, COLOR_BTNSHADOW as core::ffi::c_schar,
];

pub static LTRBInnerFlat: [core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNFACE as core::ffi::c_schar,
    -1, COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNFACE as core::ffi::c_schar,
    -1, COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNFACE as core::ffi::c_schar, COLOR_BTNFACE as core::ffi::c_schar,
];

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDrawDiagEdge(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT) -> BOOL {
    IntDrawDiagEdge_impl(hdc, rc, uType, uFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDrawDiagEdge(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT) -> BOOL {
    IntDrawDiagEdge_impl(hdc, rc, uType, uFlags)
}

unsafe fn IntDrawDiagEdge_impl(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT) -> BOOL {
    let mut Points: [POINT; 4] = core::mem::zeroed();
    let InnerI: i8;
    let OuterI: i8;
    let mut InnerPen: HPEN;
    let mut OuterPen: HPEN;
    let mut SavePoint: POINT = core::mem::zeroed();
    let SavePen: HPEN;
    let mut spx: i32;
    let mut spy: i32;
    let mut epx: i32;
    let mut epy: i32;
    let Width = (*rc).right - (*rc).left;
    let Height = (*rc).bottom - (*rc).top;
    let SmallDiam = if Width > Height { Height } else { Width };
    let retval: BOOL = (!(((uType & BDR_INNER) == BDR_INNER
        || (uType & BDR_OUTER) == BDR_OUTER)
        && (uFlags & (BF_FLAT | BF_MONO)) == 0)) as BOOL;
    let add: i32 = (if LTRBInnerMono[(uType & (BDR_INNER | BDR_OUTER)) as usize] != -1 { 1 } else { 0 })
        + (if LTRBOuterMono[(uType & (BDR_INNER | BDR_OUTER)) as usize] != -1 { 1 } else { 0 });

    OuterPen = NtGdiGetStockObject(NULL_PEN as i32) as HPEN;
    InnerPen = OuterPen;
    SavePen = NtGdiSelectPen(hdc, InnerPen) as HPEN;
    spx = 0;
    spy = 0;
    epx = 0;
    epy = 0;

    if (uFlags & BF_MONO) != 0 {
        InnerI = LTRBInnerMono[(uType & (BDR_INNER | BDR_OUTER)) as usize];
        OuterI = LTRBOuterMono[(uType & (BDR_INNER | BDR_OUTER)) as usize];
    } else if (uFlags & BF_FLAT) != 0 {
        InnerI = LTRBInnerFlat[(uType & (BDR_INNER | BDR_OUTER)) as usize];
        OuterI = LTRBOuterFlat[(uType & (BDR_INNER | BDR_OUTER)) as usize];
    } else if (uFlags & BF_SOFT) != 0 {
        if (uFlags & BF_BOTTOM) != 0 {
            InnerI = RBInnerNormal[(uType & (BDR_INNER | BDR_OUTER)) as usize];
            OuterI = RBOuterNormal[(uType & (BDR_INNER | BDR_OUTER)) as usize];
        } else {
            InnerI = LTInnerSoft[(uType & (BDR_INNER | BDR_OUTER)) as usize];
            OuterI = LTOuterSoft[(uType & (BDR_INNER | BDR_OUTER)) as usize];
        }
    } else {
        if (uFlags & BF_BOTTOM) != 0 {
            InnerI = RBInnerNormal[(uType & (BDR_INNER | BDR_OUTER)) as usize];
            OuterI = RBOuterNormal[(uType & (BDR_INNER | BDR_OUTER)) as usize];
        } else {
            InnerI = LTInnerNormal[(uType & (BDR_INNER | BDR_OUTER)) as usize];
            OuterI = LTOuterNormal[(uType & (BDR_INNER | BDR_OUTER)) as usize];
        }
    }

    if InnerI != -1 {
        InnerPen = NtGdiGetStockObject(DC_PEN as i32) as HPEN;
    }
    if OuterI != -1 {
        OuterPen = NtGdiGetStockObject(DC_PEN as i32) as HPEN;
    }

    GreMoveTo(hdc, 0, 0, &mut SavePoint as *mut _ as *mut _POINTL);

    match uFlags & BF_RECT {
        0 | BF_LEFT | BF_BOTTOM | BF_BOTTOMLEFT => {
            epx = (*rc).left - 1;
            spx = epx + SmallDiam;
            epy = (*rc).bottom;
            spy = epy - SmallDiam;
        }
        BF_TOPLEFT | BF_BOTTOMRIGHT => {
            epx = (*rc).left - 1;
            spx = epx + SmallDiam;
            epy = (*rc).top - 1;
            spy = epy + SmallDiam;
        }
        _ => {
            spx = (*rc).left;
            epx = spx + SmallDiam;
            spy = (*rc).bottom - 1;
            epy = spy - SmallDiam;
        }
    }

    GreMoveTo(hdc, spx, spy, core::ptr::null_mut());
    NtGdiSelectPen(hdc, OuterPen);
    IntSetDCPenColor(hdc, IntGetSysColor(OuterI as i32));
    NtGdiLineTo(hdc, epx, epy);

    NtGdiSelectPen(hdc, InnerPen);
    IntSetDCPenColor(hdc, IntGetSysColor(InnerI as i32));

    match uFlags & (BF_RECT | BF_DIAGONAL) {
        x if x == BF_DIAGONAL_ENDBOTTOMLEFT
            || x == (BF_DIAGONAL | BF_BOTTOM)
            || x == BF_DIAGONAL
            || x == (BF_DIAGONAL | BF_LEFT) =>
        {
            GreMoveTo(hdc, spx - 1, spy, core::ptr::null_mut());
            NtGdiLineTo(hdc, epx, epy - 1);
            Points[0].x = spx - add;
            Points[0].y = spy;
            Points[1].x = (*rc).left;
            Points[1].y = (*rc).top;
            Points[2].x = epx + 1;
            Points[2].y = epy - 1 - add;
            Points[3] = Points[2];
        }
        x if x == BF_DIAGONAL_ENDBOTTOMRIGHT => {
            GreMoveTo(hdc, spx - 1, spy, core::ptr::null_mut());
            NtGdiLineTo(hdc, epx, epy + 1);
            Points[0].x = spx - add;
            Points[0].y = spy;
            Points[1].x = (*rc).left;
            Points[1].y = (*rc).bottom - 1;
            Points[2].x = epx + 1;
            Points[2].y = epy + 1 + add;
            Points[3] = Points[2];
        }
        x if x == (BF_DIAGONAL | BF_BOTTOM | BF_RIGHT | BF_TOP)
            || x == (BF_DIAGONAL | BF_BOTTOM | BF_RIGHT | BF_TOP | BF_LEFT)
            || x == BF_DIAGONAL_ENDTOPRIGHT
            || x == (BF_DIAGONAL | BF_RIGHT | BF_TOP | BF_LEFT) =>
        {
            GreMoveTo(hdc, spx + 1, spy, core::ptr::null_mut());
            NtGdiLineTo(hdc, epx, epy + 1);
            Points[0].x = epx - 1;
            Points[0].y = epy + 1 + add;
            Points[1].x = (*rc).right - 1;
            Points[1].y = (*rc).top + add;
            Points[2].x = (*rc).right - 1;
            Points[2].y = (*rc).bottom - 1;
            Points[3].x = spx + add;
            Points[3].y = spy;
        }
        x if x == BF_DIAGONAL_ENDTOPLEFT => {
            GreMoveTo(hdc, spx, spy - 1, core::ptr::null_mut());
            NtGdiLineTo(hdc, epx + 1, epy);
            Points[0].x = epx + 1 + add;
            Points[0].y = epy + 1;
            Points[1].x = (*rc).right - 1;
            Points[1].y = (*rc).top;
            Points[2].x = (*rc).right - 1;
            Points[2].y = (*rc).bottom - 1 - add;
            Points[3].x = spx;
            Points[3].y = spy - add;
        }
        x if x == (BF_DIAGONAL | BF_TOP)
            || x == (BF_DIAGONAL | BF_BOTTOM | BF_TOP)
            || x == (BF_DIAGONAL | BF_BOTTOM | BF_TOP | BF_LEFT) =>
        {
            GreMoveTo(hdc, spx + 1, spy - 1, core::ptr::null_mut());
            NtGdiLineTo(hdc, epx, epy);
            Points[0].x = epx - 1;
            Points[0].y = epy + 1;
            Points[1].x = (*rc).right - 1;
            Points[1].y = (*rc).top;
            Points[2].x = (*rc).right - 1;
            Points[2].y = (*rc).bottom - 1 - add;
            Points[3].x = spx + add;
            Points[3].y = spy - add;
        }
        x if x == (BF_DIAGONAL | BF_RIGHT)
            || x == (BF_DIAGONAL | BF_RIGHT | BF_LEFT)
            || x == (BF_DIAGONAL | BF_RIGHT | BF_LEFT | BF_BOTTOM) =>
        {
            GreMoveTo(hdc, spx, spy, core::ptr::null_mut());
            NtGdiLineTo(hdc, epx - 1, epy + 1);
            Points[0].x = spx;
            Points[0].y = spy;
            Points[1].x = (*rc).left;
            Points[1].y = (*rc).top + add;
            Points[2].x = epx - 1 - add;
            Points[2].y = epy + 1 + add;
            Points[3] = Points[2];
        }
        _ => {}
    }

    if (uFlags & BF_MIDDLE) != 0 && retval != 0 {
        let hbsave: HBRUSH;
        let hpsave: HPEN;
        hbsave = NtGdiSelectBrush(hdc, NtGdiGetStockObject(DC_BRUSH as i32) as HBRUSH) as HBRUSH;
        hpsave = NtGdiSelectPen(hdc, NtGdiGetStockObject(DC_PEN as i32) as HPEN) as HPEN;
        IntSetDCBrushColor(hdc, IntGetSysColor(if (uFlags & BF_MONO) != 0 { COLOR_WINDOW as i32 } else { COLOR_BTNFACE as i32 }));
        IntSetDCPenColor(hdc, IntGetSysColor(if (uFlags & BF_MONO) != 0 { COLOR_WINDOW as i32 } else { COLOR_BTNFACE as i32 }));
        IntPolygon(hdc, Points.as_mut_ptr() as *mut _POINTL, 4);
        NtGdiSelectBrush(hdc, hbsave);
        NtGdiSelectPen(hdc, hpsave);
    }

    if (uFlags & BF_ADJUST) != 0 {
        if (uFlags & BF_LEFT) != 0 {
            (*rc).left += add;
        }
        if (uFlags & BF_RIGHT) != 0 {
            (*rc).right -= add;
        }
        if (uFlags & BF_TOP) != 0 {
            (*rc).top += add;
        }
        if (uFlags & BF_BOTTOM) != 0 {
            (*rc).bottom -= add;
        }
    }

    NtGdiSelectPen(hdc, SavePen);
    GreMoveTo(hdc, SavePoint.x, SavePoint.y, core::ptr::null_mut());

    retval
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDrawRectEdge(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT) -> BOOL {
    IntDrawRectEdge_impl(hdc, rc, uType, uFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDrawRectEdge(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT) -> BOOL {
    IntDrawRectEdge_impl(hdc, rc, uType, uFlags)
}

unsafe fn IntDrawRectEdge_impl(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT) -> BOOL {
    let LTInnerI: i8;
    let LTOuterI: i8;
    let RBInnerI: i8;
    let RBOuterI: i8;
    let mut LTInnerPen: HPEN;
    let mut LTOuterPen: HPEN;
    let mut RBInnerPen: HPEN;
    let mut RBOuterPen: HPEN;
    let mut InnerRect: RECT = *rc;
    let mut SavePoint: POINT = core::mem::zeroed();
    let SavePen: HPEN;
    let mut LBpenplus: i32 = 0;
    let mut LTpenplus: i32 = 0;
    let mut RTpenplus: i32 = 0;
    let mut RBpenplus: i32 = 0;

    let retval: BOOL = (!(((uType & BDR_INNER) == BDR_INNER
        || (uType & BDR_OUTER) == BDR_OUTER)
        && (uFlags & (BF_FLAT | BF_MONO)) == 0)) as BOOL;

    LTInnerPen = NtGdiGetStockObject(NULL_PEN as i32) as HPEN;
    LTOuterPen = LTInnerPen;
    RBInnerPen = LTInnerPen;
    RBOuterPen = LTInnerPen;
    SavePen = NtGdiSelectPen(hdc, LTInnerPen) as HPEN;

    let idx = (uType & (BDR_INNER | BDR_OUTER)) as usize;

    if uFlags & BF_MONO != 0 {
        LTInnerI = LTRBInnerMono[idx];
        RBInnerI = LTInnerI;
        LTOuterI = LTRBOuterMono[idx];
        RBOuterI = LTOuterI;
    } else if uFlags & BF_FLAT != 0 {
        let mut li = LTRBInnerFlat[idx];
        LTOuterI = LTRBOuterFlat[idx];
        RBOuterI = LTOuterI;
        if li != -1 {
            li = COLOR_BTNFACE as i8;
        }
        LTInnerI = li;
        RBInnerI = li;
    } else if uFlags & BF_SOFT != 0 {
        LTInnerI = LTInnerSoft[idx];
        LTOuterI = LTOuterSoft[idx];
        RBInnerI = RBInnerSoft[idx];
        RBOuterI = RBOuterSoft[idx];
    } else {
        LTInnerI = LTInnerNormal[idx];
        LTOuterI = LTOuterNormal[idx];
        RBInnerI = RBInnerNormal[idx];
        RBOuterI = RBOuterNormal[idx];
    }

    if (uFlags & BF_BOTTOMLEFT) == BF_BOTTOMLEFT {
        LBpenplus = 1;
    }
    if (uFlags & BF_TOPRIGHT) == BF_TOPRIGHT {
        RTpenplus = 1;
    }
    if (uFlags & BF_BOTTOMRIGHT) == BF_BOTTOMRIGHT {
        RBpenplus = 1;
    }
    if (uFlags & BF_TOPLEFT) == BF_TOPLEFT {
        LTpenplus = 1;
    }

    if LTInnerI != -1 {
        LTInnerPen = NtGdiGetStockObject(DC_PEN as i32) as HPEN;
    }
    if LTOuterI != -1 {
        LTOuterPen = NtGdiGetStockObject(DC_PEN as i32) as HPEN;
    }
    if RBInnerI != -1 {
        RBInnerPen = NtGdiGetStockObject(DC_PEN as i32) as HPEN;
    }
    if RBOuterI != -1 {
        RBOuterPen = NtGdiGetStockObject(DC_PEN as i32) as HPEN;
    }

    if (uFlags & BF_MIDDLE) != 0 && retval != 0 {
        FillRect(
            hdc,
            &InnerRect,
            IntGetSysColorBrush(if uFlags & BF_MONO != 0 {
                COLOR_WINDOW as i32
            } else {
                COLOR_BTNFACE as i32
            }) as HBRUSH,
        );
    }
    GreMoveTo(hdc, 0, 0, &mut SavePoint as *mut _ as *mut _POINTL);

    NtGdiSelectPen(hdc, LTOuterPen);
    IntSetDCPenColor(hdc, IntGetSysColor(LTOuterI as i32));
    if uFlags & BF_TOP != 0 {
        GreMoveTo(hdc, InnerRect.left, InnerRect.top, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.right, InnerRect.top);
    }
    if uFlags & BF_LEFT != 0 {
        GreMoveTo(hdc, InnerRect.left, InnerRect.top, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.left, InnerRect.bottom);
    }
    NtGdiSelectPen(hdc, RBOuterPen);
    IntSetDCPenColor(hdc, IntGetSysColor(RBOuterI as i32));
    if uFlags & BF_BOTTOM != 0 {
        GreMoveTo(hdc, InnerRect.left, InnerRect.bottom - 1, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.right, InnerRect.bottom - 1);
    }
    if uFlags & BF_RIGHT != 0 {
        GreMoveTo(hdc, InnerRect.right - 1, InnerRect.top, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.right - 1, InnerRect.bottom);
    }

    NtGdiSelectPen(hdc, LTInnerPen);
    IntSetDCPenColor(hdc, IntGetSysColor(LTInnerI as i32));
    if uFlags & BF_TOP != 0 {
        GreMoveTo(hdc, InnerRect.left + LTpenplus, InnerRect.top + 1, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.right - RTpenplus, InnerRect.top + 1);
    }
    if uFlags & BF_LEFT != 0 {
        GreMoveTo(hdc, InnerRect.left + 1, InnerRect.top + LTpenplus, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.left + 1, InnerRect.bottom - LBpenplus);
    }
    NtGdiSelectPen(hdc, RBInnerPen);
    IntSetDCPenColor(hdc, IntGetSysColor(RBInnerI as i32));
    if uFlags & BF_BOTTOM != 0 {
        GreMoveTo(hdc, InnerRect.left + LBpenplus, InnerRect.bottom - 2, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.right - RBpenplus, InnerRect.bottom - 2);
    }
    if uFlags & BF_RIGHT != 0 {
        GreMoveTo(hdc, InnerRect.right - 2, InnerRect.top + RTpenplus, core::ptr::null_mut());
        NtGdiLineTo(hdc, InnerRect.right - 2, InnerRect.bottom - RBpenplus);
    }

    if ((uFlags & BF_MIDDLE) != 0 && retval != 0) || (uFlags & BF_ADJUST) != 0 {
        let add: i32 = (if LTRBInnerMono[idx] != -1 { 1 } else { 0 })
            + (if LTRBOuterMono[idx] != -1 { 1 } else { 0 });

        if uFlags & BF_LEFT != 0 {
            InnerRect.left += add;
        }
        if uFlags & BF_RIGHT != 0 {
            InnerRect.right -= add;
        }
        if uFlags & BF_TOP != 0 {
            InnerRect.top += add;
        }
        if uFlags & BF_BOTTOM != 0 {
            InnerRect.bottom -= add;
        }

        if uFlags & BF_ADJUST != 0 {
            *rc = InnerRect;
        }
    }

    NtGdiSelectPen(hdc, SavePen);
    GreMoveTo(hdc, SavePoint.x, SavePoint.y, core::ptr::null_mut());
    retval
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS_MakeSquareRect(src: LPRECT, dst: LPRECT) -> i32 {
    UITOOLS_MakeSquareRect_impl(src, dst)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS_MakeSquareRect(src: LPRECT, dst: LPRECT) -> i32 {
    UITOOLS_MakeSquareRect_impl(src, dst)
}

unsafe fn UITOOLS_MakeSquareRect_impl(src: LPRECT, dst: LPRECT) -> i32 {
    let Width: i32 = (*src).right - (*src).left;
    let Height: i32 = (*src).bottom - (*src).top;
    let SmallDiam: i32 = if Width > Height { Height } else { Width };

    // out-param write (volatile to defeat captures(none) elimination)
    let mut tmp: RECT = *src;
    if Width < Height {
        tmp.top += (Height - Width) / 2;
        tmp.bottom = tmp.top + SmallDiam;
    } else if Width > Height {
        tmp.left += (Width - Height) / 2;
        tmp.right = tmp.left + SmallDiam;
    }
    core::ptr::write_volatile(dst, tmp);

    SmallDiam
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS_DrawCheckedRect(dc: HDC, rect: LPRECT) {
    UITOOLS_DrawCheckedRect_impl(dc, rect)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS_DrawCheckedRect(dc: HDC, rect: LPRECT) {
    UITOOLS_DrawCheckedRect_impl(dc, rect)
}

unsafe fn UITOOLS_DrawCheckedRect_impl(dc: HDC, rect: LPRECT) {
    if IntGetSysColor(COLOR_BTNHIGHLIGHT as i32) == RGB(255, 255, 255) {
        FillRect(dc, rect, IntGetSysColorBrush(COLOR_BTNFACE as i32) as HBRUSH);
        let bg: COLORREF = IntGdiSetBkColor(dc, RGB(255, 255, 255));
        let hbsave: HBRUSH = NtGdiSelectBrush(dc, (*gpsi_per_user()).hbrGray);
        NtGdiPatBlt(
            dc,
            (*rect).left,
            (*rect).top,
            (*rect).right - (*rect).left,
            (*rect).bottom - (*rect).top,
            0x00FA0089,
        );
        NtGdiSelectBrush(dc, hbsave);
        IntGdiSetBkColor(dc, bg);
    } else {
        FillRect(dc, rect, IntGetSysColorBrush(COLOR_BTNHIGHLIGHT as i32) as HBRUSH);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS95_DFC_ButtonPush(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DFC_ButtonPush_impl(dc, r, uFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS95_DFC_ButtonPush(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DFC_ButtonPush_impl(dc, r, uFlags)
}

unsafe fn UITOOLS95_DFC_ButtonPush_impl(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let edge: UINT;
    let mut myr: RECT = *r;

    if uFlags & (DFCS_PUSHED | DFCS_CHECKED | DFCS_FLAT) != 0 {
        edge = EDGE_SUNKEN;
    } else {
        edge = EDGE_RAISED;
    }

    if uFlags & DFCS_CHECKED != 0 {
        if uFlags & DFCS_MONO != 0 {
            IntDrawRectEdge(dc, &mut myr, edge, BF_MONO | BF_RECT | BF_ADJUST);
        } else {
            IntDrawRectEdge(dc, &mut myr, edge, (uFlags & DFCS_FLAT) | BF_RECT | BF_SOFT | BF_ADJUST);
        }

        UITOOLS_DrawCheckedRect(dc, &mut myr);
    } else {
        if uFlags & DFCS_MONO != 0 {
            IntDrawRectEdge(dc, &mut myr, edge, BF_MONO | BF_RECT | BF_ADJUST);
            FillRect(dc, &myr, IntGetSysColorBrush(COLOR_BTNFACE as i32) as HBRUSH);
        } else {
            IntDrawRectEdge(dc, r, edge, (uFlags & DFCS_FLAT) | BF_MIDDLE | BF_RECT | BF_SOFT);
        }
    }

    if uFlags & DFCS_ADJUSTRECT != 0 {
        (*r).left += 2;
        (*r).right -= 2;
        (*r).top += 2;
        (*r).bottom -= 2;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS95_DFC_ButtonCheckRadio(dc: HDC, r: LPRECT, uFlags: u32, Radio: i32) -> i32 {
    UITOOLS95_DFC_ButtonCheckRadio_impl(dc, r, uFlags, Radio)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS95_DFC_ButtonCheckRadio(dc: HDC, r: LPRECT, uFlags: u32, Radio: i32) -> i32 {
    UITOOLS95_DFC_ButtonCheckRadio_impl(dc, r, uFlags, Radio)
}

unsafe fn UITOOLS95_DFC_ButtonCheckRadio_impl(dc: HDC, r: LPRECT, uFlags: u32, Radio: i32) -> i32 {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFont: HFONT;
    let hOldFont: HFONT;
    let mut i: i32;
    let OutRight: u16;
    let OutLeft: u16;
    let InRight: u16;
    let InLeft: u16;
    let Center: u16;
    let mut myr: RECT = core::mem::zeroed();
    let cxy: i32;
    let nBkMode: i32;

    cxy = UITOOLS_MakeSquareRect(r, &mut myr);
    if Radio != 0 {
        OutRight = b'j' as u16;
        OutLeft = b'k' as u16;
        InRight = b'l' as u16;
        InLeft = b'm' as u16;
        Center = b'n' as u16;
    } else {
        OutRight = b'c' as u16;
        OutLeft = b'd' as u16;
        InRight = b'e' as u16;
        InLeft = b'f' as u16;
        Center = b'g' as u16;
    }

    core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
    lf.lfHeight = cxy;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as i32;
    lf.lfCharSet = DEFAULT_CHARSET as u8;
    let marlett: [u16; 8] = [b'M' as u16, b'a' as u16, b'r' as u16, b'l' as u16, b'e' as u16, b't' as u16, b't' as u16, 0];
    core::ptr::copy_nonoverlapping(marlett.as_ptr(), lf.lfFaceName.as_mut_ptr(), 8);
    hFont = GreCreateFontIndirectW(&mut lf);
    hOldFont = NtGdiSelectFont(dc, hFont);

    nBkMode = GreGetBkMode(dc);

    if Radio != 0 && (uFlags & 0xff) == DFCS_BUTTONRADIOMASK {
        IntGdiSetBkMode(dc, OPAQUE as i32);
        IntGdiSetTextColor(dc, IntGetSysColor(COLOR_WINDOWFRAME as i32));
        GreTextOutW(dc, myr.left, myr.top, &Center, 1);
        IntGdiSetBkMode(dc, TRANSPARENT as i32);
        IntGdiSetTextColor(dc, IntGetSysColor(COLOR_WINDOWFRAME as i32));
        GreTextOutW(dc, myr.left, myr.top, &OutRight, 1);
        IntGdiSetTextColor(dc, IntGetSysColor(COLOR_WINDOWFRAME as i32));
        GreTextOutW(dc, myr.left, myr.top, &OutLeft, 1);
    } else {
        IntGdiSetBkMode(dc, TRANSPARENT as i32);

        i = if uFlags & (DFCS_INACTIVE | DFCS_PUSHED) == 0 { COLOR_WINDOW as i32 } else { COLOR_BTNFACE as i32 };
        IntGdiSetTextColor(dc, IntGetSysColor(i));
        GreTextOutW(dc, myr.left, myr.top, &Center, 1);

        if uFlags & (DFCS_FLAT | DFCS_MONO) != 0 {
            IntGdiSetTextColor(dc, IntGetSysColor(COLOR_WINDOWFRAME as i32));
            GreTextOutW(dc, myr.left, myr.top, &OutRight, 1);
            GreTextOutW(dc, myr.left, myr.top, &OutLeft, 1);
            GreTextOutW(dc, myr.left, myr.top, &InRight, 1);
            GreTextOutW(dc, myr.left, myr.top, &InLeft, 1);
        } else {
            IntGdiSetTextColor(dc, IntGetSysColor(COLOR_BTNSHADOW as i32));
            GreTextOutW(dc, myr.left, myr.top, &OutRight, 1);
            IntGdiSetTextColor(dc, IntGetSysColor(COLOR_BTNHIGHLIGHT as i32));
            GreTextOutW(dc, myr.left, myr.top, &OutLeft, 1);
            IntGdiSetTextColor(dc, IntGetSysColor(COLOR_3DDKSHADOW as i32));
            GreTextOutW(dc, myr.left, myr.top, &InRight, 1);
            IntGdiSetTextColor(dc, IntGetSysColor(COLOR_3DLIGHT as i32));
            GreTextOutW(dc, myr.left, myr.top, &InLeft, 1);
        }
    }

    if uFlags & DFCS_CHECKED != 0 {
        let Check: u16 = if Radio != 0 { b'i' as u16 } else { b'b' as u16 };

        IntGdiSetTextColor(dc, IntGetSysColor(if uFlags & DFCS_INACTIVE != 0 { COLOR_BTNSHADOW as i32 } else { COLOR_WINDOWTEXT as i32 }));
        GreTextOutW(dc, myr.left, myr.top, &Check, 1);
    }

    IntGdiSetTextColor(dc, IntGetSysColor(COLOR_WINDOWTEXT as i32));
    NtGdiSelectFont(dc, hOldFont);
    GreDeleteObject(hFont as _);
    IntGdiSetBkMode(dc, nBkMode);

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS95_DrawFrameButton(hdc: HDC, rc: LPRECT, uState: UINT) -> BOOL {
    UITOOLS95_DrawFrameButton_impl(hdc, rc, uState)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS95_DrawFrameButton(hdc: HDC, rc: LPRECT, uState: UINT) -> BOOL {
    UITOOLS95_DrawFrameButton_impl(hdc, rc, uState)
}

unsafe fn UITOOLS95_DrawFrameButton_impl(hdc: HDC, rc: LPRECT, uState: UINT) -> BOOL {
    match uState & 0x1f {
        x if x == DFCS_BUTTONPUSH as u32 => {
            return UITOOLS95_DFC_ButtonPush(hdc, rc, uState);
        }
        x if x == DFCS_BUTTONCHECK as u32 || x == DFCS_BUTTON3STATE as u32 => {
            return UITOOLS95_DFC_ButtonCheckRadio(hdc, rc, uState, 0);
        }
        x if x == DFCS_BUTTONRADIOIMAGE as u32 || x == DFCS_BUTTONRADIOMASK as u32 => {
            if uState & DFCS_BUTTONRADIOIMAGE as u32 != 0 {
                FillRect(hdc, rc, NtGdiGetStockObject(BLACK_BRUSH as i32) as HBRUSH);
            } else {
                FillRect(hdc, rc, NtGdiGetStockObject(WHITE_BRUSH as i32) as HBRUSH);
            }

            return UITOOLS95_DFC_ButtonCheckRadio(hdc, rc, uState, 1);
        }
        x if x == DFCS_BUTTONRADIO as u32 => {
            return UITOOLS95_DFC_ButtonCheckRadio(hdc, rc, uState, 1);
        }
        _ => {
            ERR!("Invalid button state=0x%04x\n", uState);
        }
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS95_DrawFrameCaption(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DrawFrameCaption_impl(dc, r, uFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS95_DrawFrameCaption(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DrawFrameCaption_impl(dc, r, uFlags)
}

unsafe fn UITOOLS95_DrawFrameCaption_impl(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFont: HFONT;
    let hOldFont: HFONT;
    let clrsave: COLORREF;
    let mut myr: RECT = core::mem::zeroed();
    let bkmode: INT;
    let Symbol: WCHAR;

    match uFlags & 0xf {
        x if x == DFCS_CAPTIONCLOSE => {
            Symbol = b'r' as WCHAR;
        }
        x if x == DFCS_CAPTIONHELP => {
            Symbol = b's' as WCHAR;
        }
        x if x == DFCS_CAPTIONMIN => {
            Symbol = b'0' as WCHAR;
        }
        x if x == DFCS_CAPTIONMAX => {
            Symbol = b'1' as WCHAR;
        }
        x if x == DFCS_CAPTIONRESTORE => {
            Symbol = b'2' as WCHAR;
        }
        _ => {
            return 0;
        }
    }

    IntDrawRectEdge(
        dc,
        r,
        if uFlags & DFCS_PUSHED != 0 { EDGE_SUNKEN } else { EDGE_RAISED },
        BF_RECT | BF_MIDDLE | BF_SOFT,
    );

    core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
    UITOOLS_MakeSquareRect(r, &mut myr);
    myr.left += 1;
    myr.top += 1;
    myr.right -= 1;
    myr.bottom -= 1;
    if uFlags & DFCS_PUSHED != 0 {
        RECTL_vOffsetRect(&mut myr, 1, 1);
    }
    lf.lfHeight = myr.bottom - myr.top;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as LONG;
    lf.lfCharSet = DEFAULT_CHARSET as u8;
    lf.lfQuality = NONANTIALIASED_QUALITY as u8;

    let marlett: [u16; 8] = [
        b'M' as u16, b'a' as u16, b'r' as u16, b'l' as u16, b'e' as u16, b't' as u16, b't' as u16, 0,
    ];
    core::ptr::copy_nonoverlapping(
        marlett.as_ptr(),
        lf.lfFaceName.as_mut_ptr(),
        marlett.len(),
    );

    hFont = GreCreateFontIndirectW(&mut lf);
    hOldFont = NtGdiSelectFont(dc, hFont);
    clrsave = GreGetTextColor(dc);
    bkmode = GreGetBkMode(dc);

    IntGdiSetBkMode(dc, TRANSPARENT as INT);

    let mut SymbolLocal: WCHAR = Symbol;

    if uFlags & DFCS_INACTIVE != 0 {
        IntGdiSetTextColor(dc, IntGetSysColor(COLOR_BTNHIGHLIGHT as i32));
        GreTextOutW(dc, myr.left + 1, myr.top + 1, &mut SymbolLocal, 1);
    }

    IntGdiSetTextColor(
        dc,
        IntGetSysColor(if uFlags & DFCS_INACTIVE != 0 {
            COLOR_BTNSHADOW as i32
        } else {
            COLOR_BTNTEXT as i32
        }),
    );

    GreTextOutW(dc, myr.left, myr.top, &mut SymbolLocal, 1);

    IntGdiSetTextColor(dc, clrsave);
    NtGdiSelectFont(dc, hOldFont);
    IntGdiSetBkMode(dc, bkmode);
    GreDeleteObject(hFont as _);
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS95_DrawFrameScroll(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DrawFrameScroll_impl(dc, r, uFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS95_DrawFrameScroll(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DrawFrameScroll_impl(dc, r, uFlags)
}

unsafe fn UITOOLS95_DrawFrameScroll_impl(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFont: HFONT;
    let hOldFont: HFONT;
    let clrsave: COLORREF;
    let mut myr: RECT = core::mem::zeroed();
    let bkmode: INT;
    let mut Symbol: WCHAR;

    let marlett: [u16; 8] = [
        'M' as u16, 'a' as u16, 'r' as u16, 'l' as u16, 'e' as u16, 't' as u16, 't' as u16, 0,
    ];

    match uFlags & 0x1f {
        x if x == DFCS_SCROLLCOMBOBOX || x == DFCS_SCROLLDOWN => {
            Symbol = '6' as u16;
        }
        x if x == DFCS_SCROLLUP => {
            Symbol = '5' as u16;
        }
        x if x == DFCS_SCROLLLEFT => {
            Symbol = '3' as u16;
        }
        x if x == DFCS_SCROLLRIGHT => {
            Symbol = '4' as u16;
        }
        x if x == DFCS_SCROLLSIZEGRIP || x == DFCS_SCROLLSIZEGRIPRIGHT => {
            core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
            UITOOLS_MakeSquareRect(r, &mut myr);
            lf.lfHeight = myr.bottom - myr.top;
            lf.lfWidth = 0;
            lf.lfWeight = FW_NORMAL as i32;
            lf.lfCharSet = DEFAULT_CHARSET as u8;
            core::ptr::copy_nonoverlapping(
                marlett.as_ptr(),
                lf.lfFaceName.as_mut_ptr(),
                marlett.len(),
            );
            let hFont = GreCreateFontIndirectW(&mut lf);
            let hOldFont = NtGdiSelectFont(dc, hFont);
            let clrsave = GreGetTextColor(dc);
            let bkmode = GreGetBkMode(dc);
            IntGdiSetBkMode(dc, TRANSPARENT as i32);
            if (uFlags & (DFCS_MONO | DFCS_FLAT)) == 0 {
                IntGdiSetTextColor(dc, IntGetSysColor(COLOR_BTNHIGHLIGHT as i32));
                Symbol = if (uFlags & 0xff) == DFCS_SCROLLSIZEGRIP {
                    'o' as u16
                } else {
                    'x' as u16
                };
                GreTextOutW(dc, myr.left, myr.top, &mut Symbol, 1);
                IntGdiSetTextColor(dc, IntGetSysColor(COLOR_BTNSHADOW as i32));
            } else {
                IntGdiSetTextColor(dc, IntGetSysColor(COLOR_WINDOWFRAME as i32));
            }
            Symbol = if (uFlags & 0xff) == DFCS_SCROLLSIZEGRIP {
                'p' as u16
            } else {
                'y' as u16
            };
            GreTextOutW(dc, myr.left, myr.top, &mut Symbol, 1);
            IntGdiSetTextColor(dc, clrsave);
            NtGdiSelectFont(dc, hOldFont);
            IntGdiSetBkMode(dc, bkmode);
            GreDeleteObject(hFont as _);
            return 1;
        }
        _ => {
            return 0;
        }
    }

    IntDrawRectEdge(
        dc,
        r,
        if (uFlags & DFCS_PUSHED) != 0 {
            EDGE_SUNKEN
        } else {
            EDGE_RAISED
        },
        (uFlags & DFCS_FLAT) | BF_MIDDLE | BF_RECT,
    );
    core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
    UITOOLS_MakeSquareRect(r, &mut myr);
    myr.left += 1;
    myr.top += 1;
    myr.right -= 1;
    myr.bottom -= 1;
    if (uFlags & DFCS_PUSHED) != 0 {
        RECTL_vOffsetRect(&mut myr, 1, 1);
    }
    lf.lfHeight = myr.bottom - myr.top;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as i32;
    lf.lfCharSet = DEFAULT_CHARSET as u8;
    core::ptr::copy_nonoverlapping(
        marlett.as_ptr(),
        lf.lfFaceName.as_mut_ptr(),
        marlett.len(),
    );
    hFont = GreCreateFontIndirectW(&mut lf);
    hOldFont = NtGdiSelectFont(dc, hFont);
    clrsave = GreGetTextColor(dc);
    bkmode = GreGetBkMode(dc);
    IntGdiSetBkMode(dc, TRANSPARENT as i32);
    if (uFlags & DFCS_INACTIVE) != 0 {
        IntGdiSetTextColor(dc, IntGetSysColor(COLOR_BTNHIGHLIGHT as i32));
        GreTextOutW(dc, myr.left + 1, myr.top + 1, &mut Symbol, 1);
    }
    IntGdiSetTextColor(
        dc,
        IntGetSysColor(if (uFlags & DFCS_INACTIVE) != 0 {
            COLOR_BTNSHADOW as i32
        } else {
            COLOR_BTNTEXT as i32
        }),
    );
    GreTextOutW(dc, myr.left, myr.top, &mut Symbol, 1);
    IntGdiSetTextColor(dc, clrsave);
    NtGdiSelectFont(dc, hOldFont);
    IntGdiSetBkMode(dc, bkmode);
    GreDeleteObject(hFont as _);
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UITOOLS95_DrawFrameMenu(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DrawFrameMenu_impl(dc, r, uFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UITOOLS95_DrawFrameMenu(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    UITOOLS95_DrawFrameMenu_impl(dc, r, uFlags)
}

unsafe fn UITOOLS95_DrawFrameMenu_impl(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFont: HFONT;
    let hOldFont: HFONT;
    let Symbol: WCHAR;
    let mut myr: RECT = core::mem::zeroed();
    let cxy: INT;

    cxy = UITOOLS_MakeSquareRect(r, core::ptr::addr_of_mut!(myr));

    match uFlags & 0x1f {
        x if x == DFCS_MENUARROWUP => {
            Symbol = '5' as WCHAR;
        }
        x if x == DFCS_MENUARROWDOWN => {
            Symbol = '6' as WCHAR;
        }
        x if x == DFCS_MENUARROW => {
            Symbol = '8' as WCHAR;
        }
        x if x == DFCS_MENUARROWRIGHT => {
            Symbol = 'w' as WCHAR;
        }
        x if x == DFCS_MENUBULLET => {
            Symbol = 'h' as WCHAR;
        }
        x if x == DFCS_MENUCHECK || x == (DFCS_MENUCHECK | DFCS_MENUBULLET) => {
            Symbol = 'a' as WCHAR;
        }
        _ => {
            return 0;
        }
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(lf) as *mut _,
        core::mem::size_of::<LOGFONTW>() as _,
    );
    lf.lfHeight = cxy;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as _;
    lf.lfCharSet = DEFAULT_CHARSET as _;
    let marlett: [WCHAR; 8] = [
        'M' as WCHAR,
        'a' as WCHAR,
        'r' as WCHAR,
        'l' as WCHAR,
        'e' as WCHAR,
        't' as WCHAR,
        't' as WCHAR,
        0,
    ];
    RtlCopyMemory(
        lf.lfFaceName.as_mut_ptr() as *mut _,
        marlett.as_ptr() as *const _,
        core::mem::size_of_val(&marlett) as _,
    );

    hFont = GreCreateFontIndirectW(core::ptr::addr_of_mut!(lf));
    hOldFont = NtGdiSelectFont(dc, hFont);

    if (uFlags & 0x1f) == DFCS_MENUARROWUP || (uFlags & 0x1f) == DFCS_MENUARROWDOWN {
        IntGdiSetTextColor(
            dc,
            IntGetSysColor(if (uFlags & DFCS_INACTIVE) != 0 {
                COLOR_BTNSHADOW as _
            } else {
                COLOR_BTNTEXT as _
            }),
        );
    }

    GreTextOutW(dc, myr.left, myr.top, core::ptr::addr_of!(Symbol), 1);

    NtGdiSelectFont(dc, hOldFont);
    GreDeleteObject(hFont as _);
    1
}

#[no_mangle]
pub unsafe extern "system" fn FrameRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32 {
    let r: RECT = *lprc;

    if (r.right <= r.left) || (r.bottom <= r.top) {
        return 0;
    }
    let oldbrush: HBRUSH = NtGdiSelectBrush(hDC, hbr);
    if oldbrush.is_null() {
        return 0;
    }

    NtGdiPatBlt(hDC, r.left, r.top, 1, r.bottom - r.top, PATCOPY as _);
    NtGdiPatBlt(hDC, r.right - 1, r.top, 1, r.bottom - r.top, PATCOPY as _);
    NtGdiPatBlt(hDC, r.left, r.top, r.right - r.left, 1, PATCOPY as _);
    NtGdiPatBlt(hDC, r.left, r.bottom - 1, r.right - r.left, 1, PATCOPY as _);

    NtGdiSelectBrush(hDC, oldbrush);
    1
}

#[no_mangle]
pub unsafe extern "system" fn FillRect(hDC: HDC, lprc: *const RECT, mut hbr: HBRUSH) -> i32 {
    let mut prevhbr: HBRUSH = core::ptr::null_mut();

    if !hbr.is_null() {
        if hbr <= (COLOR_MENUBAR + 1) as HBRUSH {
            hbr = IntGetSysColorBrush(PtrToUlong(hbr as *const core::ffi::c_void).wrapping_sub(1) as i32) as HBRUSH;
        }

        prevhbr = NtGdiSelectBrush(hDC, hbr);
        if prevhbr.is_null() {
            return 0;
        }
    }

    let Ret = NtGdiPatBlt(
        hDC,
        (*lprc).left,
        (*lprc).top,
        (*lprc).right - (*lprc).left,
        (*lprc).bottom - (*lprc).top,
        PATCOPY as _,
    );

    if !prevhbr.is_null() {
        NtGdiSelectBrush(hDC, prevhbr);
    }

    Ret as i32
}

#[no_mangle]
pub unsafe extern "system" fn DrawEdge(hDC: HDC, rc: LPRECT, edge: UINT, flags: UINT) -> BOOL {
    if flags & BF_DIAGONAL != 0 {
        IntDrawDiagEdge(hDC, rc, edge, flags)
    } else {
        IntDrawRectEdge(hDC, rc, edge, flags)
    }
}

#[no_mangle]
pub unsafe extern "system" fn DrawFrameControl(hDC: HDC, rc: LPRECT, uType: UINT, uState: UINT) -> BOOL {
    if GreGetMapMode(hDC) != MM_TEXT as i32 {
        return 0;
    }

    match uType {
        x if x == DFC_BUTTON as u32 => UITOOLS95_DrawFrameButton(hDC, rc, uState),
        x if x == DFC_CAPTION as u32 => UITOOLS95_DrawFrameCaption(hDC, rc, uState),
        x if x == DFC_MENU as u32 => {
            let ret: BOOL;
            let rgbOldText: COLORREF;
            let iOldBackMode: INT;

            if uState & (DFCS_MENUARROWUP | DFCS_MENUARROWDOWN) != 0 {
                if uState & DFCS_TRANSPARENT == 0 {
                    FillRect(hDC, rc, IntGetSysColorBrush(COLOR_MENU as i32) as HBRUSH);
                }
            } else {
                FillRect(hDC, rc, NtGdiGetStockObject(WHITE_BRUSH as INT) as HBRUSH);
            }

            rgbOldText = IntGdiSetTextColor(hDC, RGB(0, 0, 0));
            iOldBackMode = IntGdiSetBkMode(hDC, TRANSPARENT as INT);
            ret = UITOOLS95_DrawFrameMenu(hDC, rc, uState);
            IntGdiSetBkMode(hDC, iOldBackMode);
            IntGdiSetTextColor(hDC, rgbOldText);
            ret
        }
        x if x == DFC_SCROLL as u32 => UITOOLS95_DrawFrameScroll(hDC, rc, uState),
        _ => 0,
    }
}

use win32k_sys::win32k::*;

use core::ffi::c_void;
use crate::floatobj::*;
// are all static inline / macros (no symbols). Inline via the FLOATOBJ shim.
#[inline(always)]
unsafe fn INTERNAL_APPLY_MATRIX(matrix: PMATRIX, points: LPPOINT, mut count: u32) -> u8 {
    while count != 0 {
        count -= 1;
        let p = points.offset(count as isize);
        let mut x: FLOATOBJ = core::mem::zeroed();
        let mut y: FLOATOBJ = core::mem::zeroed();
        let mut tmp: FLOATOBJ;
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(x), (*p).x);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efM11));
        tmp = (*matrix).efM21;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).y);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efDx));
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(y), (*p).y);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efM22));
        tmp = (*matrix).efM12;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).x);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efDy));
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(x), core::ptr::addr_of_mut!((*p).x)) == 0 { return 0; }
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(y), core::ptr::addr_of_mut!((*p).y)) == 0 { return 0; }
    }
    1
}
#[inline(always)]
unsafe fn IntLPtoDP(pdc: *mut DC, ppt: LPPOINT, count: i32) -> BOOL {
    DC_vUpdateWorldToDevice(pdc);
    INTERNAL_APPLY_MATRIX(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice) as PMATRIX, ppt, count as u32) as BOOL
}
#[inline(always)]
unsafe fn IntDPtoLP(pdc: *mut DC, ppt: LPPOINT, count: i32) -> BOOL {
    DC_vUpdateDeviceToWorld(pdc);
    INTERNAL_APPLY_MATRIX(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxDeviceToWorld) as PMATRIX, ppt, count as u32) as BOOL
}
#[inline(always)] unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> u8 { (((*prcl).left >= (*prcl).right) || ((*prcl).top >= (*prcl).bottom)) as u8 }
#[inline(always)] fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }
#[inline(always)] fn min(a: i32, b: i32) -> i32 { if a < b { a } else { b } }
#[cfg(target_arch = "x86")] extern "fastcall" { fn CLIPPING_UpdateGCRegion(pdc: PDC); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn CLIPPING_UpdateGCRegion(pdc: PDC); }

// ── DC-lifecycle shared preamble (hand-added; macros/consts rustconv can't see) ──
use ntkrnl_sys::base::STATUS_SUCCESS;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as i32;
// GDITAG_HMGR_LOOKASIDE_DC_TYPE = '1alG' (big-endian multichar)
// EXLATEOBJ_* real ENGAPI (xlateobj.rs) — declare extern, same-ABI (mirror fillshap).
extern "system" {
    fn EXLATEOBJ_vInitialize(pexlo: PEXLATEOBJ, ppalSrc: *mut PALETTE, ppalDst: *mut PALETTE, c1: COLORREF, c2: COLORREF, c3: COLORREF);
    fn EXLATEOBJ_vCleanup(exlo: *mut EXLATEOBJ);
}
#[cfg(target_arch = "x86")] extern "fastcall" { fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH; }
#[cfg(not(target_arch = "x86"))] extern "C" { fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH; }
#[inline(always)] fn UlongToHandle(u: ULONG) -> HANDLE { u as usize as HANDLE }
#[inline(always)] unsafe fn InterlockedCompareExchangePointer(dst: *mut PVOID, xchg: PVOID, cmp: PVOID) -> PVOID {
    let a = &*(dst as *const core::sync::atomic::AtomicPtr<core::ffi::c_void>);
    match a.compare_exchange(cmp, xchg, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) { Ok(v) => v, Err(v) => v }
}
const TAG_DC: u32 = 0x31616c47;
const INT_MAX: INT = i32::MAX;
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(s as *const u8, d as *mut u8, n);
}
// GDI object share/lock macros (surface.h/palette.h/path.h/text.h) — inline.
#[inline(always)] unsafe fn LFONT_ShareUnlockFont(p: PLFONT) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn LFONT_ShareLockFont(h: HGDIOBJ) -> PLFONT { GDIOBJ_ReferenceObjectByHandle(h, GDIObjType_LFONT_TYPE as u8) as PLFONT }
#[inline(always)] unsafe fn SURFACE_ShareLockSurface(h: HGDIOBJ) -> PSURFACE { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE }
#[inline(always)] unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn PALETTE_ShareLockPalette(h: HGDIOBJ) -> PPALETTE { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_PALETTE as DWORD) as PPALETTE }
#[inline(always)] unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }
#[inline(always)] unsafe fn PATH_LockPath(h: HGDIOBJ) -> PPATH { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_PATH as DWORD) as PPATH }
#[inline(always)] unsafe fn PATH_UnlockPath(p: PPATH) { GDIOBJ_vDereferenceObject(p as POBJ); }
// FLOATOBJ constants (engobjects.h) — per-arch (i386 struct / non-x86 f32).
#[cfg(target_arch = "x86")] const FLOATOBJ_0: FLOATOBJ = FLOATOBJ { ul1: 0x00000000, ul2: 0x00000000 };
#[cfg(target_arch = "x86")] const FLOATOBJ_1: FLOATOBJ = FLOATOBJ { ul1: 0x40000000, ul2: 0x00000002 };
#[cfg(target_arch = "x86")] const FLOATOBJ_16: FLOATOBJ = FLOATOBJ { ul1: 0x40000000, ul2: 0x00000006 };
#[cfg(target_arch = "x86")] const FLOATOBJ_1_16: FLOATOBJ = FLOATOBJ { ul1: 0x40000000, ul2: 0xfffffffe };
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_0: FLOATOBJ = 0.;
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_1: FLOATOBJ = 1.;
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_16: FLOATOBJ = 16.;
#[cfg(not(target_arch = "x86"))] const FLOATOBJ_1_16: FLOATOBJ = 1. / 16.;
// ── end shared preamble ──
extern "system" { fn SafeProbeForRead(a: *const core::ffi::c_void, l: usize, al: u32) -> i32; fn SafeProbeForWrite(a: *mut core::ffi::c_void, l: usize, al: u32) -> i32; }

// ABI: RECTL_bUnionRect, RECTL_vMakeWellOrdered, SetLastNtError are all FASTCALL
// (@Name@N on i386). rustconv mis-declared them extern "C"/"system" → on i386 the
// decoration mismatches the fastcall definitions → CC collision folds callers to ud2.
#[cfg(target_arch = "x86")] extern "fastcall" {
    fn RECTL_bUnionRect(dst: PRECTL, src1: *const RECTL, src2: *const RECTL) -> i32;
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))] extern "C" {
    fn RECTL_bUnionRect(dst: PRECTL, src1: *const RECTL, src2: *const RECTL) -> i32;
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn SetLastNtError(Status: NTSTATUS);
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the provided preamble

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreDPtoLP(hdc: HDC, lpPoints: LPPOINT, nCount: i32) -> i32 {
    let dc: PDC;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    IntDPtoLP(dc, lpPoints as LPPOINT, nCount);
    DC_UnlockDc(dc);
    return 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreDPtoLP(hdc: HDC, lpPoints: LPPOINT, nCount: i32) -> i32 {
    let dc: PDC;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    IntDPtoLP(dc, lpPoints as LPPOINT, nCount);
    DC_UnlockDc(dc);
    return 1;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreLPtoDP(hdc: HDC, lpPoints: LPPOINT, nCount: i32) -> i32 {
    let dc: PDC;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    IntLPtoDP(dc, lpPoints, nCount);
    DC_UnlockDc(dc);
    return 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreLPtoDP(hdc: HDC, lpPoints: LPPOINT, nCount: i32) -> i32 {
    let dc: PDC;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    IntLPtoDP(dc, lpPoints, nCount);
    DC_UnlockDc(dc);
    return 1;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetBkMode(hdc: HDC) -> i32 {
    let dc: PDC;
    let lBkMode: i32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    lBkMode = (*(*dc).pdcattr).lBkMode;
    DC_UnlockDc(dc);
    return lBkMode;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetBkMode(hdc: HDC) -> i32 {
    let dc: PDC;
    let lBkMode: i32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    lBkMode = (*(*dc).pdcattr).lBkMode;
    DC_UnlockDc(dc);
    return lBkMode;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetBkColor(hdc: HDC) -> COLORREF {
    let dc: PDC;
    let crBk: COLORREF;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    crBk = (*(*dc).pdcattr).ulBackgroundClr;
    DC_UnlockDc(dc);
    return crBk;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetBkColor(hdc: HDC) -> COLORREF {
    let dc: PDC;
    let crBk: COLORREF;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    crBk = (*(*dc).pdcattr).ulBackgroundClr;
    DC_UnlockDc(dc);
    return crBk;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetMapMode(hdc: HDC) -> i32 {
    let dc: PDC;
    let iMapMode: i32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    iMapMode = (*(*dc).pdcattr).iMapMode;
    DC_UnlockDc(dc);
    return iMapMode;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetMapMode(hdc: HDC) -> i32 {
    let dc: PDC;
    let iMapMode: i32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    iMapMode = (*(*dc).pdcattr).iMapMode;
    DC_UnlockDc(dc);
    return iMapMode;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetTextColor(hdc: HDC) -> COLORREF {
    let dc: PDC;
    let ulForegroundClr: u32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    ulForegroundClr = (*(*dc).pdcattr).ulForegroundClr;
    DC_UnlockDc(dc);
    return ulForegroundClr;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetTextColor(hdc: HDC) -> COLORREF {
    let dc: PDC;
    let ulForegroundClr: u32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    ulForegroundClr = (*(*dc).pdcattr).ulForegroundClr;
    DC_UnlockDc(dc);
    return ulForegroundClr;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGdiSetBkColor(hDC: HDC, color: COLORREF) -> COLORREF {
    let mut oldColor: COLORREF;
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let hBrush: HBRUSH;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    pdcattr = (*dc).pdcattr;

    oldColor = (*pdcattr).ulBackgroundClr;
    (*pdcattr).ulBackgroundClr = color;

    if (*pdcattr).crBackgroundClr != color {
        (*pdcattr).ulDirty_ |= DIRTY_BACKGROUND | DIRTY_LINE | DIRTY_FILL;
        (*pdcattr).crBackgroundClr = color;
    }
    hBrush = (*pdcattr).hbrush as _;
    DC_UnlockDc(dc);
    NtGdiSelectBrush(hDC, hBrush);
    return oldColor;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGdiSetBkColor(hDC: HDC, color: COLORREF) -> COLORREF {
    let mut oldColor: COLORREF;
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let hBrush: HBRUSH;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    pdcattr = (*dc).pdcattr;

    oldColor = (*pdcattr).ulBackgroundClr;
    (*pdcattr).ulBackgroundClr = color;

    if (*pdcattr).crBackgroundClr != color {
        (*pdcattr).ulDirty_ |= DIRTY_BACKGROUND | DIRTY_LINE | DIRTY_FILL;
        (*pdcattr).crBackgroundClr = color;
    }
    hBrush = (*pdcattr).hbrush as _;
    DC_UnlockDc(dc);
    NtGdiSelectBrush(hDC, hBrush);
    return oldColor;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiSetBkMode(hDC: HDC, Mode: i32) -> i32 {
    let oldMode: COLORREF;
    let dc: PDC;
    let pdcattr: PDC_ATTR;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    pdcattr = (*dc).pdcattr;
    oldMode = (*pdcattr).lBkMode as u32;
    (*pdcattr).jBkMode = Mode as u8;
    (*pdcattr).lBkMode = Mode as i32;
    DC_UnlockDc(dc);
    return oldMode as _;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiSetBkMode(hDC: HDC, Mode: i32) -> i32 {
    let oldMode: COLORREF;
    let dc: PDC;
    let pdcattr: PDC_ATTR;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    pdcattr = (*dc).pdcattr;
    oldMode = (*pdcattr).lBkMode as u32;
    (*pdcattr).jBkMode = Mode as u8;
    (*pdcattr).lBkMode = Mode as i32;
    DC_UnlockDc(dc);
    return oldMode as _;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiSetTextAlign(
    hDC: HDC,
    Mode: u32,
) -> u32 {
    let mut prevAlign: u32;
    let dc: *mut DC;
    let pdcattr: PDC_ATTR;
    let mut mode_adjusted: u32;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }
    pdcattr = (*dc).pdcattr;
    prevAlign = (*pdcattr).lTextAlign as u32;
    (*pdcattr).lTextAlign = Mode as i32;
    mode_adjusted = Mode;
    if ((*pdcattr).dwLayout & LAYOUT_RTL) != 0 {
        if (mode_adjusted & TA_CENTER) != TA_CENTER {
            mode_adjusted ^= TA_RIGHT;
        }
    }
    (*pdcattr).flTextAlign = mode_adjusted & TA_MASK;
    DC_UnlockDc(dc);
    return prevAlign;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiSetTextAlign(
    hDC: HDC,
    Mode: u32,
) -> u32 {
    let mut prevAlign: u32;
    let dc: *mut DC;
    let pdcattr: PDC_ATTR;
    let mut mode_adjusted: u32;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }
    pdcattr = (*dc).pdcattr;
    prevAlign = (*pdcattr).lTextAlign as u32;
    (*pdcattr).lTextAlign = Mode as i32;
    mode_adjusted = Mode;
    if ((*pdcattr).dwLayout & LAYOUT_RTL) != 0 {
        if (mode_adjusted & TA_CENTER) != TA_CENTER {
            mode_adjusted ^= TA_RIGHT;
        }
    }
    (*pdcattr).flTextAlign = mode_adjusted & TA_MASK;
    DC_UnlockDc(dc);
    return prevAlign;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiSetTextColor(hDC: HDC, color: COLORREF) -> COLORREF {
    let crOldColor: COLORREF;
    let pdc: PDC;
    let pdcattr: PDC_ATTR;

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    pdcattr = (*pdc).pdcattr;

    crOldColor = (*pdcattr).ulForegroundClr as COLORREF;
    (*pdcattr).ulForegroundClr = color as u32;

    if (*pdcattr).crForegroundClr != color {
        (*pdcattr).ulDirty_ |= DIRTY_TEXT | DIRTY_LINE | DIRTY_FILL;
        (*pdcattr).crForegroundClr = color;
    }

    DC_vUpdateTextBrush(pdc);
    DC_vUpdateLineBrush(pdc);
    DC_vUpdateFillBrush(pdc);

    DC_UnlockDc(pdc);

    return crOldColor;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiSetTextColor(hDC: HDC, color: COLORREF) -> COLORREF {
    let crOldColor: COLORREF;
    let pdc: PDC;
    let pdcattr: PDC_ATTR;

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    pdcattr = (*pdc).pdcattr;

    crOldColor = (*pdcattr).ulForegroundClr as COLORREF;
    (*pdcattr).ulForegroundClr = color as u32;

    if (*pdcattr).crForegroundClr != color {
        (*pdcattr).ulDirty_ |= DIRTY_TEXT | DIRTY_LINE | DIRTY_FILL;
        (*pdcattr).crForegroundClr = color;
    }

    DC_vUpdateTextBrush(pdc);
    DC_vUpdateLineBrush(pdc);
    DC_vUpdateFillBrush(pdc);

    DC_UnlockDc(pdc);

    return crOldColor;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSetDCBrushColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let mut OldColor: COLORREF = CLR_INVALID as u32;
    let dc: PDC = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    } else {
        OldColor = (*(*dc).pdcattr).ulBrushClr as COLORREF;
        (*(*dc).pdcattr).ulBrushClr = crColor as u32;

        if (*(*dc).pdcattr).crBrushClr != crColor {
            (*(*dc).pdcattr).ulDirty_ |= DIRTY_FILL;
            (*(*dc).pdcattr).crBrushClr = crColor;
        }
    }
    DC_UnlockDc(dc);
    return OldColor;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSetDCBrushColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let mut OldColor: COLORREF = CLR_INVALID as u32;
    let dc: PDC = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    } else {
        OldColor = (*(*dc).pdcattr).ulBrushClr as COLORREF;
        (*(*dc).pdcattr).ulBrushClr = crColor as u32;

        if (*(*dc).pdcattr).crBrushClr != crColor {
            (*(*dc).pdcattr).ulDirty_ |= DIRTY_FILL;
            (*(*dc).pdcattr).crBrushClr = crColor;
        }
    }
    DC_UnlockDc(dc);
    return OldColor;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreSetBrushOrg(
    hdc: HDC,
    x: i32,
    y: i32,
    pptOut: LPPOINT,
) -> u8 {
    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if !pptOut.is_null() {
        *(pptOut as *mut _POINTL) = (*(*pdc).pdcattr).ptlBrushOrigin;
    }

    (*(*pdc).pdcattr).ptlBrushOrigin.x = x;
    (*(*pdc).pdcattr).ptlBrushOrigin.y = y;

    DC_vSetBrushOrigin(pdc, x, y);

    DC_UnlockDc(pdc);
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreSetBrushOrg(
    hdc: HDC,
    x: i32,
    y: i32,
    pptOut: LPPOINT,
) -> u8 {
    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if !pptOut.is_null() {
        *(pptOut as *mut _POINTL) = (*(*pdc).pdcattr).ptlBrushOrigin;
    }

    (*(*pdc).pdcattr).ptlBrushOrigin.x = x;
    (*(*pdc).pdcattr).ptlBrushOrigin.y = y;

    DC_vSetBrushOrigin(pdc, x, y);

    DC_UnlockDc(pdc);
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntSetDCPenColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let OldColor: COLORREF;
    let dc: PDC;
    
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID as _;
    }

    OldColor = (*(*dc).pdcattr).ulPenClr as COLORREF;
    (*(*dc).pdcattr).ulPenClr = crColor as u32;

    if (*(*dc).pdcattr).crPenClr != crColor {
        (*(*dc).pdcattr).ulDirty_ |= DIRTY_LINE;
        (*(*dc).pdcattr).crPenClr = crColor;
    }
    DC_UnlockDc(dc);
    return OldColor;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntSetDCPenColor(hdc: HDC, crColor: COLORREF) -> COLORREF {
    let OldColor: COLORREF;
    let dc: PDC;
    
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return CLR_INVALID as _;
    }

    OldColor = (*(*dc).pdcattr).ulPenClr as COLORREF;
    (*(*dc).pdcattr).ulPenClr = crColor as u32;

    if (*(*dc).pdcattr).crPenClr != crColor {
        (*(*dc).pdcattr).ulDirty_ |= DIRTY_LINE;
        (*(*dc).pdcattr).crPenClr = crColor;
    }
    DC_UnlockDc(dc);
    return OldColor;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreSetStretchBltMode(hDC: HDC, iStretchMode: i32) -> i32 {
    let mut pdc: PDC;
    let mut pdcattr: PDC_ATTR;
    let mut oSMode: i32 = 0;
    let mut iStretchMode = iStretchMode;

    pdc = DC_LockDc(hDC);
    if !pdc.is_null() {
        pdcattr = (*pdc).pdcattr;
        oSMode = (*pdcattr).lStretchBltMode;
        (*pdcattr).lStretchBltMode = iStretchMode;

        if (iStretchMode <= 0) || (iStretchMode > MAXSTRETCHBLTMODE as i32) {
            iStretchMode = WHITEONBLACK as i32;
        }

        (*pdcattr).jStretchBltMode = iStretchMode as u8;
        DC_UnlockDc(pdc);
    }
    oSMode
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreSetStretchBltMode(hDC: HDC, iStretchMode: i32) -> i32 {
    let mut pdc: PDC;
    let mut pdcattr: PDC_ATTR;
    let mut oSMode: i32 = 0;
    let mut iStretchMode = iStretchMode;

    pdc = DC_LockDc(hDC);
    if !pdc.is_null() {
        pdcattr = (*pdc).pdcattr;
        oSMode = (*pdcattr).lStretchBltMode;
        (*pdcattr).lStretchBltMode = iStretchMode;

        if (iStretchMode <= 0) || (iStretchMode > MAXSTRETCHBLTMODE as i32) {
            iStretchMode = WHITEONBLACK as i32;
        }

        (*pdcattr).jStretchBltMode = iStretchMode as u8;
        DC_UnlockDc(pdc);
    }
    oSMode
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreGetGraphicsMode(hdc: HDC) -> i32 {
    let dc: PDC;
    let GraphicsMode: i32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    GraphicsMode = (*(*dc).pdcattr).iGraphicsMode;
    DC_UnlockDc(dc);
    return GraphicsMode;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreGetGraphicsMode(hdc: HDC) -> i32 {
    let dc: PDC;
    let GraphicsMode: i32;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID as _;
    }
    GraphicsMode = (*(*dc).pdcattr).iGraphicsMode;
    DC_UnlockDc(dc);
    return GraphicsMode;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DCU_SetDcUndeletable(hDC: HDC) {
    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return;
    }

    (*dc).fs |= (DC_PERMANANT as i32);
    DC_UnlockDc(dc);
    return;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DCU_SetDcUndeletable(hDC: HDC) {
    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return;
    }

    (*dc).fs |= DC_PERMANANT as i32 ;
    DC_UnlockDc(dc);
    return;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntIsPrimarySurface(SurfObj: *mut SURFOBJ) -> BOOL { 0 /* #if 0 dead code in C; PrimarySurface global does not exist */ }

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntIsPrimarySurface(SurfObj: *mut SURFOBJ) -> BOOL { 0 /* #if 0 dead code in C; PrimarySurface global does not exist */ }

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntSetDefaultRegion(pdc: PDC) -> u8 {
    let mut pSurface: PSURFACE;
    let mut prgn: PREGION;
    let mut rclWnd: RECTL = core::mem::zeroed();
    let mut rclClip: RECTL;

    IntGdiReleaseRaoRgn(pdc);

    rclWnd.left = 0;
    rclWnd.top = 0;
    rclWnd.right = (*pdc).dclevel.sizl.cx;
    rclWnd.bottom = (*pdc).dclevel.sizl.cy;
    rclClip = rclWnd;

    if (*(*pdc).ppdev).flFlags & PDEV_META_DEVICE != 0 {
        pSurface = (*pdc).dclevel.pSurface;
        if !pSurface.is_null() && (*pSurface).flags & PDEV_SURFACE != 0 {
            rclClip.left += (*(*pdc).ppdev).ptlOrigion.x;
            rclClip.top += (*(*pdc).ppdev).ptlOrigion.y;
            rclClip.right += (*(*pdc).ppdev).ptlOrigion.x;
            rclClip.bottom += (*(*pdc).ppdev).ptlOrigion.y;
        }
    }

    prgn = (*pdc).prgnVis;

    if !prgn.is_null() && prgn != prgnDefault {
        REGION_SetRectRgn(
            prgn,
            rclClip.left,
            rclClip.top,
            rclClip.right,
            rclClip.bottom,
        );
    } else {
        prgn = IntSysCreateRectpRgn(
            rclClip.left,
            rclClip.top,
            rclClip.right,
            rclClip.bottom,
        );
        (*pdc).prgnVis = prgn;
    }

    if !prgn.is_null() {
        (*pdc).ptlDCOrig.x = 0;
        (*pdc).ptlDCOrig.y = 0;
        (*pdc).erclWindow = rclWnd;
        (*pdc).erclClip = rclClip;
        (*pdc).ptlFillOrigin = (*pdc).dcattr.ptlBrushOrigin;
        return 1;
    }

    (*pdc).prgnVis = prgnDefault;
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntSetDefaultRegion(pdc: PDC) -> u8 {
    let mut pSurface: PSURFACE;
    let mut prgn: PREGION;
    let mut rclWnd: RECTL = core::mem::zeroed();
    let mut rclClip: RECTL;

    IntGdiReleaseRaoRgn(pdc);

    rclWnd.left = 0;
    rclWnd.top = 0;
    rclWnd.right = (*pdc).dclevel.sizl.cx;
    rclWnd.bottom = (*pdc).dclevel.sizl.cy;
    rclClip = rclWnd;

    if (*(*pdc).ppdev).flFlags & PDEV_META_DEVICE != 0 {
        pSurface = (*pdc).dclevel.pSurface;
        if !pSurface.is_null() && (*pSurface).flags & PDEV_SURFACE != 0 {
            rclClip.left += (*(*pdc).ppdev).ptlOrigion.x;
            rclClip.top += (*(*pdc).ppdev).ptlOrigion.y;
            rclClip.right += (*(*pdc).ppdev).ptlOrigion.x;
            rclClip.bottom += (*(*pdc).ppdev).ptlOrigion.y;
        }
    }

    prgn = (*pdc).prgnVis;

    if !prgn.is_null() && prgn != prgnDefault {
        REGION_SetRectRgn(
            prgn,
            rclClip.left,
            rclClip.top,
            rclClip.right,
            rclClip.bottom,
        );
    } else {
        prgn = IntSysCreateRectpRgn(
            rclClip.left,
            rclClip.top,
            rclClip.right,
            rclClip.bottom,
        );
        (*pdc).prgnVis = prgn;
    }

    if !prgn.is_null() {
        (*pdc).ptlDCOrig.x = 0;
        (*pdc).ptlDCOrig.y = 0;
        (*pdc).erclWindow = rclWnd;
        (*pdc).erclClip = rclClip;
        (*pdc).ptlFillOrigin = (*pdc).dcattr.ptlBrushOrigin;
        return 1;
    }

    (*pdc).prgnVis = prgnDefault;
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCancelDC(hDC: HDC) -> i32 {
    0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn IntGdiSetHookFlags(hDC: HDC, Flags: u16) -> u16 {
    let wRet: u16;
    let dc: *mut DC = DC_LockDc(hDC);

    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    wRet = ((*dc).fs & (DC_DIRTY_RAO as i32)) as u16;

    if (Flags & DCHF_INVALIDATEVISRGN as u16) != 0 {
        (*dc).fs |= (DC_DIRTY_RAO as i32);
    } else if (Flags & DCHF_VALIDATEVISRGN as u16) != 0 || Flags == 0 {
        //(*dc).fs &= !(DC_DIRTY_RAO as i32);
    }

    DC_UnlockDc(dc);

    wRet
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn IntGdiSetHookFlags(hDC: HDC, Flags: u16) -> u16 {
    let wRet: u16;
    let dc: *mut DC = DC_LockDc(hDC);

    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    wRet = ((*dc).fs & (DC_DIRTY_RAO as i32)) as u16;

    if (Flags & DCHF_INVALIDATEVISRGN as u16) != 0 {
        (*dc).fs |= DC_DIRTY_RAO as i32 ;
    } else if (Flags & DCHF_VALIDATEVISRGN as u16) != 0 || Flags == 0 {
        //(*dc).fs &= !(DC_DIRTY_RAO as i32);
    }

    DC_UnlockDc(dc);

    wRet
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDCDword(
    hDC: HDC,
    u: UINT,
    Result: *mut DWORD,
) -> BOOL {
    let mut Ret: BOOL = 1;
    let pdc: PDC;
    let pdcattr: PDC_ATTR;

    let mut SafeResult: DWORD = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if Result.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*pdc).pdcattr;

    match u {
        GdiGetJournal => {}

        GdiGetRelAbs => {
            SafeResult = (*pdcattr).lRelAbs as u32;
        }

        GdiGetBreakExtra => {
            SafeResult = (*pdcattr).lBreakExtra as u32;
        }

        GdiGetCharBreak => {
            SafeResult = (*pdcattr).cBreak as u32;
        }

        GdiGetArcDirection => {
            if ((*pdcattr).dwLayout & LAYOUT_RTL) != 0 {
                SafeResult = AD_CLOCKWISE - ((((*pdc).dclevel.flPath & DCPATH_CLOCKWISE) != 0) as DWORD);
            } else {
                SafeResult = ((((*pdc).dclevel.flPath & DCPATH_CLOCKWISE) != 0) as DWORD) + AD_COUNTERCLOCKWISE;
            }
        }

        GdiGetEMFRestorDc => {
            SafeResult = (*pdc).dclevel.lSaveDepth as DWORD;
        }

        GdiGetFontLanguageInfo => {
            SafeResult = IntGetFontLanguageInfo(pdc);
        }

        GdiGetIsMemDc => {
            SafeResult = (*pdc).dctype;
        }

        GdiGetMapMode => {
            SafeResult = (*pdcattr).iMapMode as DWORD;
        }

        GdiGetTextCharExtra => {
            SafeResult = (*pdcattr).lTextExtra as u32;
        }

        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = 0;
        }
    }

    if Ret != 0 {

        Status = SafeProbeForWrite(Result as *mut core::ffi::c_void, core::mem::size_of::<DWORD>(), 1);
        if NT_SUCCESS(Status)
            // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND
            // access inside one _SEH2_TRY, so the access must be the
            // fault-tolerant copy.
            && !crate::copy::publish_to_user(
                Result as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeResult) as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        }
    }

    DC_UnlockDc(pdc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetAndSetDCDword(
    hdc: HDC,
    u: UINT,
    dwIn: DWORD,
    pdwResult: *mut DWORD,
) -> BOOL {
    let mut Ret: BOOL = 1;
    let pdc: PDC;
    let pdcattr: PDC_ATTR;

    let mut SafeResult: DWORD = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if pdwResult.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    pdcattr = (*pdc).pdcattr;

    match u {
        GdiGetSetEPSPrintingEscape => {
            SafeResult = ((*pdc).fs & (DC_EPSPRINTINGESCAPE as i32)) as u32;
            (*pdc).fs &= !(DC_EPSPRINTINGESCAPE as i32);
        }

        GdiGetSetCopyCount => {
            SafeResult = (*pdc).ulCopyCount;
            (*pdc).ulCopyCount = dwIn;
        }

        GdiGetSetTextAlign => {
            SafeResult = (*pdcattr).lTextAlign as u32;
            (*pdcattr).lTextAlign = dwIn as i32;
        }

        GdiGetSetRelAbs => {
            SafeResult = (*pdcattr).lRelAbs as u32;
            (*pdcattr).lRelAbs = dwIn as i32;
        }

        GdiGetSetTextCharExtra => {
            SafeResult = (*pdcattr).lTextExtra as u32;
            (*pdcattr).lTextExtra = dwIn as i32;
        }

        GdiGetSetSelectFont => {}

        GdiGetSetMapperFlagsInternal => {
            if dwIn & !1 != 0 {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                Ret = 0;
            } else {
                SafeResult = (*pdcattr).flFontMapper;
                (*pdcattr).flFontMapper = dwIn;
            }
        }

        GdiGetSetMapMode => {
            SafeResult = IntGdiSetMapMode(pdc, dwIn as i32) as u32;
        }

        GdiGetSetArcDirection => {
            if dwIn != AD_COUNTERCLOCKWISE && dwIn != AD_CLOCKWISE {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                Ret = 0;
            } else {
                if (*pdcattr).dwLayout & LAYOUT_RTL != 0 {
                    SafeResult = AD_CLOCKWISE - (((*pdc).dclevel.flPath & DCPATH_CLOCKWISE) != 0) as DWORD;
                    if dwIn == AD_CLOCKWISE {
                        (*pdc).dclevel.flPath &= !DCPATH_CLOCKWISE;
                    } else {
                        (*pdc).dclevel.flPath |= DCPATH_CLOCKWISE;
                    }
                } else {
                    SafeResult = (((*pdc).dclevel.flPath & DCPATH_CLOCKWISE) != 0) as DWORD +
                                 AD_COUNTERCLOCKWISE;
                    if dwIn == AD_COUNTERCLOCKWISE {
                        (*pdc).dclevel.flPath &= !DCPATH_CLOCKWISE;
                    } else {
                        (*pdc).dclevel.flPath |= DCPATH_CLOCKWISE;
                    }
                }
            }
        }

        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = 0;
        }
    }

    if Ret != 0 {
        Status = SafeProbeForWrite(pdwResult as PVOID, core::mem::size_of::<DWORD>() as usize, 1);
        if NT_SUCCESS(Status)
            // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND
            // access inside one _SEH2_TRY, so the access must be the
            // fault-tolerant copy.
            && !crate::copy::publish_to_user(
                pdwResult as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeResult) as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        }
    }

    DC_UnlockDc(pdc);
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntUpdateBoundsRect(pdc: PDC, pRect: PRECTL) {
    
    if ((*pdc).fs & (DC_ACCUM_APP as i32)) != 0 {
        RECTL_bUnionRect(
            core::ptr::addr_of_mut!((*pdc).erclBoundsApp),
            core::ptr::addr_of_mut!((*pdc).erclBoundsApp),
            pRect
        );
    }
    if ((*pdc).fs & (DC_ACCUM_WMGR as i32)) != 0 {
        RECTL_bUnionRect(
            core::ptr::addr_of_mut!((*pdc).erclBounds),
            core::ptr::addr_of_mut!((*pdc).erclBounds),
            pRect
        );
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntUpdateBoundsRect(pdc: PDC, pRect: PRECTL) {
    
    if ((*pdc).fs & (DC_ACCUM_APP as i32)) != 0 {
        RECTL_bUnionRect(
            core::ptr::addr_of_mut!((*pdc).erclBoundsApp),
            core::ptr::addr_of_mut!((*pdc).erclBoundsApp),
            pRect
        );
    }
    if ((*pdc).fs & (DC_ACCUM_WMGR as i32)) != 0 {
        RECTL_bUnionRect(
            core::ptr::addr_of_mut!((*pdc).erclBounds),
            core::ptr::addr_of_mut!((*pdc).erclBounds),
            pRect
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetBoundsRect(
    hdc: HDC,
    prc: LPRECT,
    flags: u32,
) -> u32 {
    let mut ret: u32;
    let pdc: PDC;
    let mut rc: RECT = core::mem::zeroed();
    let Status: i32;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    if (flags & DCB_WINDOWMGR) == 0 {
        rc = *(core::ptr::addr_of!((*pdc).erclBoundsApp) as *const tagRECT);

        if RECTL_bIsEmptyRect(&rc as *const RECT as *const RECTL) != 0 {
            rc.left = 0;
            rc.top = 0;
            rc.right = 0;
            rc.bottom = 0;
            ret = DCB_RESET;
        } else {
            let mut rcRgn: RECTL = core::mem::zeroed();
            if ((*pdc).fs & (DC_DIRTY_RAO as i32)) != 0 {
                CLIPPING_UpdateGCRegion(pdc);
            }
            if REGION_GetRgnBox((*pdc).prgnRao, &mut rcRgn) == 0 {
                REGION_GetRgnBox((*pdc).prgnVis, &mut rcRgn);
            }
            rc.left = max(rc.left, 0);
            rc.top = max(rc.top, 0);
            rc.right = min(rc.right, rcRgn.right - rcRgn.left);
            rc.bottom = min(rc.bottom, rcRgn.bottom - rcRgn.top);
            ret = DCB_SET;
        }
        IntDPtoLP(pdc, &mut rc as *mut RECT as LPPOINT, 2);
    } else {
        rc = *(core::ptr::addr_of!((*pdc).erclBounds) as *const tagRECT);
        ret = DCB_SET;
    }

    Status = SafeProbeForWrite(prc as *mut core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
    if !NT_SUCCESS(Status) {
        ret = 0;
    } else if !crate::copy::publish_to_user(
        prc as *mut core::ffi::c_void,
        core::ptr::addr_of!(rc) as *const core::ffi::c_void,
        core::mem::size_of::<RECT>(),
    ) {
            // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND
            // access inside one _SEH2_TRY, so the access must be the
            // fault-tolerant copy.
        ret = 0;
    }

    if (flags & DCB_RESET) != 0 {
        if (flags & DCB_WINDOWMGR) == 0 {
            (*pdc).erclBoundsApp.left = INT_MAX;
            (*pdc).erclBoundsApp.top = INT_MAX;
            (*pdc).erclBoundsApp.right = INT_MIN;
            (*pdc).erclBoundsApp.bottom = INT_MIN;
        } else {
            (*pdc).erclBounds.left = INT_MAX;
            (*pdc).erclBounds.top = INT_MAX;
            (*pdc).erclBounds.right = INT_MIN;
            (*pdc).erclBounds.bottom = INT_MIN;
        }
    }

    DC_UnlockDc(pdc);
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetBoundsRect(
    hdc: HDC,
    prc: LPRECT,
    flags: u32,
) -> u32 {
    let mut ret: u32;
    let pdc: PDC;
    let mut rcl: RECTL = core::mem::zeroed();

    if (flags & DCB_ENABLE) != 0 && (flags & DCB_DISABLE) != 0 {
        return 0;
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    ret = DCB_RESET;
    ret = (if ((*pdc).fs & ((DC_ACCUM_APP as i32) | (DC_ACCUM_WMGR as i32))) != 0 { DCB_ENABLE } else { DCB_DISABLE }) |
          (if RECTL_bIsEmptyRect(core::ptr::addr_of!((*pdc).erclBoundsApp)) != 0 { ret & DCB_SET } else { DCB_SET });
    ret |= flags & DCB_WINDOWMGR;

    if (flags & DCB_RESET) != 0 {
        if (flags & DCB_WINDOWMGR) == 0 {
            (*pdc).erclBoundsApp.left = INT_MAX;
            (*pdc).erclBoundsApp.top = INT_MAX;
            (*pdc).erclBoundsApp.right = INT_MIN;
            (*pdc).erclBoundsApp.bottom = INT_MIN;
        } else {
            (*pdc).erclBounds.left = INT_MAX;
            (*pdc).erclBounds.top = INT_MAX;
            (*pdc).erclBounds.right = INT_MIN;
            (*pdc).erclBounds.bottom = INT_MIN;
        }
    }

    if (flags & DCB_ACCUMULATE) != 0 && !prc.is_null() {
        {
            let __SehStatus = SafeProbeForRead(prc as *const core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
            if !NT_SUCCESS(__SehStatus) {
                DC_UnlockDc(pdc);
                return 0;
            }
        }
            // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND
            // access inside one _SEH2_TRY, so the access must be the
            // fault-tolerant copy.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(rcl) as *mut core::ffi::c_void,
            prc as *const core::ffi::c_void,
            core::mem::size_of::<_RECTL>(),
        ) {
            DC_UnlockDc(pdc);
            return 0;
        }

        RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(rcl));

        if (flags & DCB_WINDOWMGR) == 0 {
            IntLPtoDP(pdc, core::ptr::addr_of_mut!(rcl) as *mut POINT, 2);
            RECTL_bUnionRect(
                core::ptr::addr_of_mut!((*pdc).erclBoundsApp),
                core::ptr::addr_of!((*pdc).erclBoundsApp),
                core::ptr::addr_of!(rcl),
            );
        } else {
            RECTL_bUnionRect(
                core::ptr::addr_of_mut!((*pdc).erclBounds),
                core::ptr::addr_of!((*pdc).erclBounds),
                core::ptr::addr_of!(rcl),
            );
        }
    }

    if (flags & DCB_ENABLE) != 0 {
        if (flags & DCB_WINDOWMGR) == 0 {
            (*pdc).fs |= (DC_ACCUM_APP as i32);
        } else {
            (*pdc).fs |= (DC_ACCUM_WMGR as i32);
        }
    }
    if (flags & DCB_DISABLE) != 0 {
        if (flags & DCB_WINDOWMGR) == 0 {
            (*pdc).fs &= !(DC_ACCUM_APP as i32);
        } else {
            (*pdc).fs &= !(DC_ACCUM_WMGR as i32);
        }
    }
    DC_UnlockDc(pdc);
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetBoundsRect(
    hdc: HDC,
    prc: LPRECT,
    flags: u32,
) -> u32 {
    let mut ret: u32;
    let pdc: PDC;
    let mut rcl: RECTL = core::mem::zeroed();

    if (flags & DCB_ENABLE) != 0 && (flags & DCB_DISABLE) != 0 {
        return 0;
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    ret = DCB_RESET;
    ret = (if ((*pdc).fs & ((DC_ACCUM_APP as i32) | (DC_ACCUM_WMGR as i32))) != 0 { DCB_ENABLE } else { DCB_DISABLE }) |
          (if RECTL_bIsEmptyRect(core::ptr::addr_of!((*pdc).erclBoundsApp)) != 0 { ret & DCB_SET } else { DCB_SET });
    ret |= flags & DCB_WINDOWMGR;

    if (flags & DCB_RESET) != 0 {
        if (flags & DCB_WINDOWMGR) == 0 {
            (*pdc).erclBoundsApp.left = INT_MAX;
            (*pdc).erclBoundsApp.top = INT_MAX;
            (*pdc).erclBoundsApp.right = INT_MIN;
            (*pdc).erclBoundsApp.bottom = INT_MIN;
        } else {
            (*pdc).erclBounds.left = INT_MAX;
            (*pdc).erclBounds.top = INT_MAX;
            (*pdc).erclBounds.right = INT_MIN;
            (*pdc).erclBounds.bottom = INT_MIN;
        }
    }

    if (flags & DCB_ACCUMULATE) != 0 && !prc.is_null() {
        {
            let __SehStatus = SafeProbeForRead(prc as *const core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
            if !NT_SUCCESS(__SehStatus) {
                DC_UnlockDc(pdc);
                return 0;
            }
        }
            // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND
            // access inside one _SEH2_TRY, so the access must be the
            // fault-tolerant copy.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(rcl) as *mut core::ffi::c_void,
            prc as *const core::ffi::c_void,
            core::mem::size_of::<_RECTL>(),
        ) {
            DC_UnlockDc(pdc);
            return 0;
        }

        RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(rcl));

        if (flags & DCB_WINDOWMGR) == 0 {
            IntLPtoDP(pdc, core::ptr::addr_of_mut!(rcl) as *mut POINT, 2);
            RECTL_bUnionRect(
                core::ptr::addr_of_mut!((*pdc).erclBoundsApp),
                core::ptr::addr_of!((*pdc).erclBoundsApp),
                core::ptr::addr_of!(rcl),
            );
        } else {
            RECTL_bUnionRect(
                core::ptr::addr_of_mut!((*pdc).erclBounds),
                core::ptr::addr_of!((*pdc).erclBounds),
                core::ptr::addr_of!(rcl),
            );
        }
    }

    if (flags & DCB_ENABLE) != 0 {
        if (flags & DCB_WINDOWMGR) == 0 {
            (*pdc).fs |= DC_ACCUM_APP as i32 ;
        } else {
            (*pdc).fs |= DC_ACCUM_WMGR as i32 ;
        }
    }
    if (flags & DCB_DISABLE) != 0 {
        if (flags & DCB_WINDOWMGR) == 0 {
            (*pdc).fs &= !(DC_ACCUM_APP as i32);
        } else {
            (*pdc).fs &= !(DC_ACCUM_WMGR as i32);
        }
    }
    DC_UnlockDc(pdc);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn TranslateCOLORREF(pdc: PDC, crColor: COLORREF) -> u32 {
    let mut psurfDC: PSURFACE;
    let ppalDC: PPALETTE;
    let mut index: u32;
    let ulColor: u32;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();

    psurfDC = (*pdc).dclevel.pSurface;

    if psurfDC.is_null() {
        psurfDC = psurfDefaultBitmap;
    }

    let mut crColor = crColor;

    if (crColor & 0xFF000000) == 0 {
    }
    else if (crColor & 0x01000000) != 0 {
        index = crColor & 0xFFFF;
        ppalDC = (*pdc).dclevel.ppal;
        if index >= (*ppalDC).NumColors {
            index = 0;
        }
        crColor = PALETTE_ulGetRGBColorFromIndex(ppalDC, index);
    }
    else if (crColor & 0x02000000) != 0 {
        if (*pdc).dclevel.hpal != (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_PALETTE as usize)) as _ {
            ppalDC = (*pdc).dclevel.ppal;
            index = PALETTE_ulGetNearestIndex(ppalDC, crColor & 0xFFFFFF);
            crColor = PALETTE_ulGetRGBColorFromIndex(ppalDC, index);
        }
        else {
            crColor = crColor & 0x00FFFFFF;
        }
    }
    else if (crColor & 0x10FF0000) == 0x10FF0000 {
        if ((*(*psurfDC).ppal).flFlags & PAL_INDEXED) != 0 {
            index = crColor & 0xFF;
            if index >= (*(*psurfDC).ppal).NumColors {
                index = 0;
            }
            return index;
        }
        else {
            crColor = crColor & 0x00FFFFFF;
        }
    }
    else {
        crColor = 0;
    }

    EXLATEOBJ_vInitialize(
        core::ptr::addr_of_mut!(exlo),
        core::ptr::addr_of_mut!(gpalRGB),
        (*psurfDC).ppal,
        CLR_INVALID,
        CLR_INVALID,
        CLR_INVALID
    );

    ulColor = XLATEOBJ_iXlate(core::ptr::addr_of_mut!(exlo.xlo), crColor);

    EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));

    ulColor
}

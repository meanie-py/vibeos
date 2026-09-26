use win32k_sys::win32k::*;
use core::ffi::c_void;

// ── DC-lifecycle shared preamble (hand-added; macros/consts rustconv can't see) ──
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as i32;
// GDITAG_HMGR_LOOKASIDE_DC_TYPE = '1alG' (big-endian multichar)
use crate::dib_helpers::BitsPerFormat;
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
extern "system" { fn SafeProbeForWrite(a: *mut core::ffi::c_void, l: usize, al: u32) -> i32; }
#[cfg(target_arch = "x86")] extern "fastcall" { fn TextIntRealizeFont(hfont: HFONT, pto: PTEXTOBJ) -> NTSTATUS; }
#[cfg(not(target_arch = "x86"))] extern "C" { fn TextIntRealizeFont(hfont: HFONT, pto: PTEXTOBJ) -> NTSTATUS; }
#[cfg(target_arch = "x86")] extern "fastcall" { fn CLIPPING_UpdateGCRegion(pdc: PDC); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn CLIPPING_UpdateGCRegion(pdc: PDC); }
#[inline(always)] unsafe fn BRUSH_ShareLockBrush(h: HGDIOBJ) -> PBRUSH { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BRUSH as DWORD) as PBRUSH }
#[inline(always)] unsafe fn BRUSH_ShareUnlockBrush(pbr: PBRUSH) { GDIOBJ_vDereferenceObject(pbr as POBJ); }
// DC_vSelect*/BRUSH_ShareUnlockBrush are FORCEINLINE (dc.h/brush.h) — inline locally.
#[inline(always)] unsafe fn DC_vSelectSurface(pdc: PDC, psurfNew: PSURFACE) {
    let psurfOld = (*pdc).dclevel.pSurface;
    if !psurfOld.is_null() { (*psurfOld).hdc = core::ptr::null_mut(); SURFACE_ShareUnlockSurface(psurfOld); }
    if !psurfNew.is_null() { GDIOBJ_vReferenceObjectByPointer(psurfNew as POBJ); }
    (*pdc).dclevel.pSurface = psurfNew;
}
#[inline(always)] unsafe fn DC_vSelectFillBrush(pdc: PDC, pbrFill: PBRUSH) {
    let o = (*pdc).dclevel.pbrFill;
    if !o.is_null() { BRUSH_ShareUnlockBrush(o); }
    if !pbrFill.is_null() { GDIOBJ_vReferenceObjectByPointer(pbrFill as POBJ); }
    (*pdc).dclevel.pbrFill = pbrFill;
}
#[inline(always)] unsafe fn DC_vSelectLineBrush(pdc: PDC, pbrLine: PBRUSH) {
    let o = (*pdc).dclevel.pbrLine;
    if !o.is_null() { BRUSH_ShareUnlockBrush(o); }
    if !pbrLine.is_null() { GDIOBJ_vReferenceObjectByPointer(pbrLine as POBJ); }
    (*pdc).dclevel.pbrLine = pbrLine;
}
#[inline(always)] unsafe fn DC_vSelectPalette(pdc: PDC, ppal: PPALETTE) {
    let o = (*pdc).dclevel.ppal;
    if !o.is_null() { PALETTE_ShareUnlockPalette(o); }
    if !ppal.is_null() { GDIOBJ_vReferenceObjectByPointer(ppal as POBJ); }
    (*pdc).dclevel.ppal = ppal;
}

// ABI: EBRUSHOBJ_vSetSolidRGBColor=FASTCALL (dual-cfg), EBRUSHOBJ_vUpdateFromDC=NTAPI(system).
// rustconv mis-declared both extern "C" → on i386 the decoration mismatches the real
// definitions → crate-wide CC collision folds callers to ud2. (feedback_dup_abi_name_collision)
#[cfg(target_arch = "x86")] extern "fastcall" { fn EBRUSHOBJ_vSetSolidRGBColor(pebo: *mut EBRUSHOBJ, crColor: COLORREF); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn EBRUSHOBJ_vSetSolidRGBColor(pebo: *mut EBRUSHOBJ, crColor: COLORREF); }
extern "system" {
    fn EBRUSHOBJ_vUpdateFromDC(pebo: *mut EBRUSHOBJ, pbrush: PBRUSH, pdc: PDC);
}
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): LFONT_ShareLockFont, LFONT_ShareUnlockFont — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
extern "C" {
}
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): PATH_LockPath, PATH_UnlockPath — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the provided preamble.
// The preamble only contains preprocessor directives and SAL annotations,
// which do not result in runtime declarations.

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DC_vUpdateFillBrush(pdc: PDC) {
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;
    let mut pbrFill: PBRUSH;

    if (*pdcattr).hbrush != (*(*pdc).dclevel.pbrFill).BaseObject.hHmgr {
        pbrFill = BRUSH_ShareLockBrush((*pdcattr).hbrush as _);
        if !pbrFill.is_null() {
            BRUSH_ShareUnlockBrush((*pdc).dclevel.pbrFill);
            (*pdc).dclevel.pbrFill = pbrFill;

            (*pdcattr).ulDirty_ |= DIRTY_FILL;
        } else {
            (*pdcattr).hbrush = (*(*pdc).dclevel.pbrFill).BaseObject.hHmgr as _;
        }
    }

    if ((*pdcattr).ulDirty_ & DIRTY_FILL) != 0 {
        EBRUSHOBJ_vUpdateFromDC(
            core::ptr::addr_of_mut!((*pdc).eboFill),
            (*pdc).dclevel.pbrFill,
            pdc,
        );
    }

    if (*pdcattr).hbrush == (*(*(&raw const StockObjects)).as_ptr().add(DC_BRUSH as usize)) as _ {
        EBRUSHOBJ_vSetSolidRGBColor(
            core::ptr::addr_of_mut!((*pdc).eboFill),
            (*pdcattr).crBrushClr,
        );
    }

    (*pdcattr).ulDirty_ &= !(DIRTY_FILL | DC_BRUSH_DIRTY);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DC_vUpdateFillBrush(pdc: PDC) {
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;
    let mut pbrFill: PBRUSH;

    if (*pdcattr).hbrush != (*(*pdc).dclevel.pbrFill).BaseObject.hHmgr {
        pbrFill = BRUSH_ShareLockBrush((*pdcattr).hbrush as _);
        if !pbrFill.is_null() {
            BRUSH_ShareUnlockBrush((*pdc).dclevel.pbrFill);
            (*pdc).dclevel.pbrFill = pbrFill;

            (*pdcattr).ulDirty_ |= DIRTY_FILL;
        } else {
            (*pdcattr).hbrush = (*(*pdc).dclevel.pbrFill).BaseObject.hHmgr as _;
        }
    }

    if ((*pdcattr).ulDirty_ & DIRTY_FILL) != 0 {
        EBRUSHOBJ_vUpdateFromDC(
            core::ptr::addr_of_mut!((*pdc).eboFill),
            (*pdc).dclevel.pbrFill,
            pdc,
        );
    }

    if (*pdcattr).hbrush == (*(*(&raw const StockObjects)).as_ptr().add(DC_BRUSH as usize)) as _ {
        EBRUSHOBJ_vSetSolidRGBColor(
            core::ptr::addr_of_mut!((*pdc).eboFill),
            (*pdcattr).crBrushClr,
        );
    }

    (*pdcattr).ulDirty_ &= !(DIRTY_FILL | DC_BRUSH_DIRTY);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DC_vUpdateLineBrush(pdc: PDC) {
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;
    let mut pbrLine: PBRUSH;

    if (*pdcattr).hpen != (*(*pdc).dclevel.pbrLine).BaseObject.hHmgr {
        pbrLine = PEN_ShareLockPen((*pdcattr).hpen as _);
        if !pbrLine.is_null() {
            BRUSH_ShareUnlockBrush((*pdc).dclevel.pbrLine);
            (*pdc).dclevel.pbrLine = pbrLine;

            (*pdcattr).ulDirty_ |= DIRTY_LINE;
        } else {
            (*pdcattr).hpen = (*(*pdc).dclevel.pbrLine).BaseObject.hHmgr as _;
        }
    }

    if (*pdcattr).ulDirty_ & DIRTY_LINE != 0 {
        EBRUSHOBJ_vUpdateFromDC(
            core::ptr::addr_of_mut!((*pdc).eboLine),
            (*pdc).dclevel.pbrLine,
            pdc,
        );
    }

    if (*pdcattr).hpen == (*(*(&raw const StockObjects)).as_ptr().add(DC_PEN as usize)) as _ {
        EBRUSHOBJ_vSetSolidRGBColor(
            core::ptr::addr_of_mut!((*pdc).eboLine),
            (*pdcattr).crPenClr,
        );
    }

    (*pdcattr).ulDirty_ &= !(DIRTY_LINE | DC_PEN_DIRTY);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DC_vUpdateLineBrush(pdc: PDC) {
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;
    let mut pbrLine: PBRUSH;

    if (*pdcattr).hpen != (*(*pdc).dclevel.pbrLine).BaseObject.hHmgr {
        pbrLine = PEN_ShareLockPen((*pdcattr).hpen as _);
        if !pbrLine.is_null() {
            BRUSH_ShareUnlockBrush((*pdc).dclevel.pbrLine);
            (*pdc).dclevel.pbrLine = pbrLine;

            (*pdcattr).ulDirty_ |= DIRTY_LINE;
        } else {
            (*pdcattr).hpen = (*(*pdc).dclevel.pbrLine).BaseObject.hHmgr as _;
        }
    }

    if (*pdcattr).ulDirty_ & DIRTY_LINE != 0 {
        EBRUSHOBJ_vUpdateFromDC(
            core::ptr::addr_of_mut!((*pdc).eboLine),
            (*pdc).dclevel.pbrLine,
            pdc,
        );
    }

    if (*pdcattr).hpen == (*(*(&raw const StockObjects)).as_ptr().add(DC_PEN as usize)) as _ {
        EBRUSHOBJ_vSetSolidRGBColor(
            core::ptr::addr_of_mut!((*pdc).eboLine),
            (*pdcattr).crPenClr,
        );
    }

    (*pdcattr).ulDirty_ &= !(DIRTY_LINE | DC_PEN_DIRTY);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
unsafe extern "fastcall" fn DC_vUpdateTextBrush(pdc: PDC)
{
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;

    if ((*pdcattr).ulDirty_ & DIRTY_TEXT) != 0 {
        EBRUSHOBJ_vUpdateFromDC(core::ptr::addr_of_mut!((*pdc).eboText) as _, pbrDefaultBrush as _, pdc);
    }

    EBRUSHOBJ_vSetSolidRGBColor(core::ptr::addr_of_mut!((*pdc).eboText) as _, (*pdcattr).crForegroundClr);

    (*pdcattr).ulDirty_ &= !DIRTY_TEXT;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
unsafe extern "C" fn DC_vUpdateTextBrush(pdc: PDC)
{
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;

    if ((*pdcattr).ulDirty_ & DIRTY_TEXT) != 0 {
        EBRUSHOBJ_vUpdateFromDC(core::ptr::addr_of_mut!((*pdc).eboText) as _, pbrDefaultBrush as _, pdc);
    }

    EBRUSHOBJ_vSetSolidRGBColor(core::ptr::addr_of_mut!((*pdc).eboText) as _, (*pdcattr).crForegroundClr);

    (*pdcattr).ulDirty_ &= !DIRTY_TEXT;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn DC_vUpdateBackgroundBrush(pdc: PDC) {
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;

    if ((*pdcattr).ulDirty_ & DIRTY_BACKGROUND) != 0 {
        EBRUSHOBJ_vUpdateFromDC(core::ptr::addr_of_mut!((*pdc).eboBackground) as _, pbrDefaultBrush as _, pdc);
    }

    EBRUSHOBJ_vSetSolidRGBColor(core::ptr::addr_of_mut!((*pdc).eboBackground) as _, (*pdcattr).crBackgroundClr);

    (*pdcattr).ulDirty_ &= !DIRTY_BACKGROUND;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn DC_vUpdateBackgroundBrush(pdc: PDC) {
    let pdcattr: PDC_ATTR = (*pdc).pdcattr;

    if ((*pdcattr).ulDirty_ & DIRTY_BACKGROUND) != 0 {
        EBRUSHOBJ_vUpdateFromDC(core::ptr::addr_of_mut!((*pdc).eboBackground) as _, pbrDefaultBrush as _, pdc);
    }

    EBRUSHOBJ_vSetSolidRGBColor(core::ptr::addr_of_mut!((*pdc).eboBackground) as _, (*pdcattr).crBackgroundClr);

    (*pdcattr).ulDirty_ &= !DIRTY_BACKGROUND;
}
#[no_mangle]
pub unsafe extern "system" fn DC_vSetBrushOrigin(pdc: PDC, x: LONG, y: LONG) {
    (*pdc).dclevel.ptlBrushOrigin.x = x;
    (*pdc).dclevel.ptlBrushOrigin.y = y;

    (*pdc).ptlFillOrigin.x = x + (*pdc).ptlDCOrig.x;
    (*pdc).ptlFillOrigin.y = y + (*pdc).ptlDCOrig.y;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetBrushOrg(
    hdc: HDC,
    x: i32,
    y: i32,
    pptOut: LPPOINT,
) -> i32 {
    let mut ptOut: POINT = core::mem::zeroed();
    let Ret = GreSetBrushOrg(hdc, x, y, core::ptr::addr_of_mut!(ptOut) as *mut _POINTL) as i32;
    if Ret != 0 {
        if !pptOut.is_null() {
            let __SehStatus = SafeProbeForWrite(
                pptOut as *mut core::ffi::c_void,
                core::mem::size_of::<POINT>(),
                1,
            );
            if !NT_SUCCESS(__SehStatus) {
                return 0;
            }
            // Probe is validate-only; the C had probe AND store in one _SEH2_TRY.
            if !crate::copy::publish_to_user(
                pptOut as *mut core::ffi::c_void,
                core::ptr::addr_of!(ptOut) as *const core::ffi::c_void,
                core::mem::size_of::<POINT>(),
            ) {
                return 0;
            }
        }
    }
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn GdiSelectPalette(
    hDC: HDC,
    hpal: HPALETTE,
    ForceBackground: i32,
) -> HPALETTE {
    let mut oldPal: HPALETTE = core::ptr::null_mut();

    let pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        return core::ptr::null_mut();
    }

    let ppal = PALETTE_ShareLockPalette(hpal as _);
    if ppal.is_null() {
        DC_UnlockDc(pdc);
        return core::ptr::null_mut();
    }

    if (*pdc).dclevel.pSurface.is_null() ||
       (BitsPerFormat((*(*pdc).dclevel.pSurface).SurfObj.iBitmapFormat) <= 8 &&
        ((*ppal).flFlags & PAL_INDEXED) != 0) ||
       (BitsPerFormat((*(*pdc).dclevel.pSurface).SurfObj.iBitmapFormat) > 8)
    {
        oldPal = (*pdc).dclevel.hpal;
        (*pdc).dclevel.hpal = hpal;
        DC_vSelectPalette(pdc, ppal);

        (*(*pdc).pdcattr).ulDirty_ |= DIRTY_FILL | DIRTY_LINE |
                                      DIRTY_BACKGROUND | DIRTY_TEXT;
    }

    if (*pdc).dctype == DCTYPE_MEMORY {
    }

    PALETTE_ShareUnlockPalette(ppal);
    DC_UnlockDc(pdc);

    oldPal
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSelectBrush(
    hDC: HDC,
    hBrush: HBRUSH,
) -> HBRUSH {
    let pDC: PDC;
    let hOrgBrush: HBRUSH;

    if hDC.is_null() || hBrush.is_null() {
        return core::ptr::null_mut();
    }

    pDC = DC_LockDc(hDC);
    if pDC.is_null() {
        return core::ptr::null_mut();
    }

    hOrgBrush = (*(*pDC).pdcattr).hbrush as _;
    (*(*pDC).pdcattr).hbrush = hBrush as _;
    DC_vUpdateFillBrush(pDC);

    DC_UnlockDc(pDC);

    hOrgBrush
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSelectPen(
    hDC: HDC,
    hPen: HPEN,
) -> HPEN {
    let pDC: PDC;
    let hOrgPen: HPEN;

    if hDC.is_null() || hPen.is_null() {
        return core::ptr::null_mut();
    }

    pDC = DC_LockDc(hDC);
    if pDC.is_null() {
        return core::ptr::null_mut();
    }

    hOrgPen = (*(*pDC).pdcattr).hpen as _;
    (*(*pDC).pdcattr).hpen = hPen as _;
    DC_vUpdateLineBrush(pDC);

    DC_UnlockDc(pDC);

    hOrgPen
}

#[no_mangle]
pub unsafe extern "system" fn DC_bIsBitmapCompatible(pdc: PDC, psurf: PSURFACE) -> BOOL {
    let cBitsPixel: ULONG;

    if ((*psurf).flags & API_BITMAP) == 0 {
        return FALSE as _;
    }

    if (*psurf).hSecure != core::ptr::null_mut() {
        return TRUE as _;
    }

    if (*psurf).SurfObj.hdev == (*pdc).ppdev as HDEV {
        return TRUE as _;
    }

    cBitsPixel = BitsPerFormat((*psurf).SurfObj.iBitmapFormat as _) as _;

    if (cBitsPixel == 1) || (cBitsPixel == (*(*pdc).ppdev).gdiinfo.cBitsPixel) {
        return TRUE as _;
    }

    FALSE as _
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSelectBitmap(
    hdc: HDC,
    hbmp: HBITMAP,
) -> HBITMAP {
    let pdc: PDC;
    let hbmpOld: HBITMAP;
    let psurfNew: PSURFACE;
    let psurfOld: PSURFACE;
    let hdcOld: HDC;

    if hdc == core::ptr::null_mut() || hbmp == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    pdc = DC_LockDc(hdc);
    if pdc == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    if (*pdc).dctype != DCTYPE_MEMORY {
        DC_UnlockDc(pdc);
        return core::ptr::null_mut();
    }

    psurfOld = (*pdc).dclevel.pSurface;

    if psurfOld != core::ptr::null_mut() {
        hbmpOld = (*psurfOld).BaseObject.hHmgr as _;
    } else {
        hbmpOld = (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_BITMAP as usize)) as _;
    }

    if hbmp == hbmpOld {
        DC_UnlockDc(pdc);
        return hbmpOld;
    }

    if hbmp == (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_BITMAP as usize)) as _ {
        psurfNew = core::ptr::null_mut();

        (*pdc).dclevel.sizl.cx = 1;
        (*pdc).dclevel.sizl.cy = 1;
    } else {
        psurfNew = SURFACE_ShareLockSurface(hbmp as _);
        if psurfNew == core::ptr::null_mut() {
            DC_UnlockDc(pdc);
            return core::ptr::null_mut();
        }

        if DC_bIsBitmapCompatible(pdc, psurfNew) == 0 {
            SURFACE_ShareUnlockSurface(psurfNew);
            DC_UnlockDc(pdc);
            return core::ptr::null_mut();
        }

        hdcOld = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*psurfNew).hdc) as *mut *mut core::ffi::c_void,
            hdc as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        ) as HDC;
        if hdcOld != core::ptr::null_mut() {
            SURFACE_ShareUnlockSurface(psurfNew);
            DC_UnlockDc(pdc);
            return core::ptr::null_mut();
        }

        (*pdc).dclevel.sizl = (*psurfNew).SurfObj.sizlBitmap;

        if (*psurfNew).hSecure != core::ptr::null_mut() {
            (*(*pdc).pdcattr).ulDirty_ |= DC_DIBSECTION;
        } else {
            (*(*pdc).pdcattr).ulDirty_ &= !DC_DIBSECTION;
        }
    }

    (*pdc).dclevel.pSurface = psurfNew;

    if psurfOld != core::ptr::null_mut() {
        (*psurfOld).hdc = core::ptr::null_mut();

        SURFACE_ShareUnlockSurface(psurfOld);
    }

    (*(*pdc).pdcattr).ulDirty_ |= DIRTY_FILL | DIRTY_LINE;
    (*pdc).fs |= (DC_DIRTY_RAO as i32);

    REGION_SetRectRgn(
        (*pdc).prgnVis,
        0,
        0,
        (*pdc).dclevel.sizl.cx,
        (*pdc).dclevel.sizl.cy,
    );

    DC_UnlockDc(pdc);

    return hbmpOld;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiSelectBitmap(
    hdc: HDC,
    hbmp: HBITMAP,
) -> HBITMAP {
    let pdc: PDC;
    let hbmpOld: HBITMAP;
    let psurfNew: PSURFACE;
    let psurfOld: PSURFACE;
    let hdcOld: HDC;

    if hdc == core::ptr::null_mut() || hbmp == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    pdc = DC_LockDc(hdc);
    if pdc == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    if (*pdc).dctype != DCTYPE_MEMORY {
        DC_UnlockDc(pdc);
        return core::ptr::null_mut();
    }

    psurfOld = (*pdc).dclevel.pSurface;

    if psurfOld != core::ptr::null_mut() {
        hbmpOld = (*psurfOld).BaseObject.hHmgr as _;
    } else {
        hbmpOld = (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_BITMAP as usize)) as _;
    }

    if hbmp == hbmpOld {
        DC_UnlockDc(pdc);
        return hbmpOld;
    }

    if hbmp == (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_BITMAP as usize)) as _ {
        psurfNew = core::ptr::null_mut();

        (*pdc).dclevel.sizl.cx = 1;
        (*pdc).dclevel.sizl.cy = 1;
    } else {
        psurfNew = SURFACE_ShareLockSurface(hbmp as _);
        if psurfNew == core::ptr::null_mut() {
            DC_UnlockDc(pdc);
            return core::ptr::null_mut();
        }

        if DC_bIsBitmapCompatible(pdc, psurfNew) == 0 {
            SURFACE_ShareUnlockSurface(psurfNew);
            DC_UnlockDc(pdc);
            return core::ptr::null_mut();
        }

        hdcOld = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*psurfNew).hdc) as *mut *mut core::ffi::c_void,
            hdc as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        ) as HDC;
        if hdcOld != core::ptr::null_mut() {
            SURFACE_ShareUnlockSurface(psurfNew);
            DC_UnlockDc(pdc);
            return core::ptr::null_mut();
        }

        (*pdc).dclevel.sizl = (*psurfNew).SurfObj.sizlBitmap;

        if (*psurfNew).hSecure != core::ptr::null_mut() {
            (*(*pdc).pdcattr).ulDirty_ |= DC_DIBSECTION;
        } else {
            (*(*pdc).pdcattr).ulDirty_ &= !DC_DIBSECTION;
        }
    }

    (*pdc).dclevel.pSurface = psurfNew;

    if psurfOld != core::ptr::null_mut() {
        (*psurfOld).hdc = core::ptr::null_mut();

        SURFACE_ShareUnlockSurface(psurfOld);
    }

    (*(*pdc).pdcattr).ulDirty_ |= DIRTY_FILL | DIRTY_LINE;
    (*pdc).fs |= DC_DIRTY_RAO as i32 ;

    REGION_SetRectRgn(
        (*pdc).prgnVis,
        0,
        0,
        (*pdc).dclevel.sizl.cx,
        (*pdc).dclevel.sizl.cy,
    );

    DC_UnlockDc(pdc);

    return hbmpOld;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSelectClipPath(
    hDC: HDC,
    Mode: i32,
) -> i32 {
    let mut RgnPath: PREGION;
    let pPath: PPATH;
    let pNewPath: PPATH;
    let mut success: i32 = 0;
    let pdcattr: PDC_ATTR;
    let pdc: PDC;

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    pdcattr = (*pdc).pdcattr;

    let hPath = (*pdc).dclevel.hPath;
    pPath = PATH_LockPath(hPath);
    if pPath.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }

    if (*pPath).state != PATH_Closed {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        success = 0;
        PATH_UnlockPath(pPath);
        PATH_Delete((*pdc).dclevel.hPath);
        (*pdc).dclevel.flPath &= !DCPATH_ACTIVE;
        (*pdc).dclevel.hPath = core::ptr::null_mut();
        DC_UnlockDc(pdc);
        return success;
    }

    RgnPath = IntSysCreateRectpRgn(0, 0, 0, 0);
    if RgnPath.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        DC_UnlockDc(pdc);
        return 0;
    }

    pNewPath = PATH_FlattenPath(pPath);

    success = PATH_PathToRegion(pNewPath, (*pdcattr).jFillMode as i32, RgnPath) as i32;

    PATH_UnlockPath(pNewPath);
    PATH_Delete((*pNewPath).BaseObject.hHmgr);

    if success != 0 {
        success = if IntGdiExtSelectClipRgn(pdc, RgnPath, Mode as INT) != ERROR as INT { 1 } else { 0 };
    }

    REGION_Delete(RgnPath);

    PATH_UnlockPath(pPath);
    PATH_Delete((*pdc).dclevel.hPath);
    (*pdc).dclevel.flPath &= !DCPATH_ACTIVE;
    (*pdc).dclevel.hPath = core::ptr::null_mut();

    DC_UnlockDc(pdc);

    success
}
#[no_mangle]
pub unsafe extern "system" fn DC_hSelectFont(
    pdc: PDC,
    hlfntNew: HFONT,
) -> HFONT {
    let plfntNew: PLFONT;
    let mut hlfntOld: HFONT;

    if !NT_SUCCESS(TextIntRealizeFont(hlfntNew, core::ptr::null_mut())) {
        return core::ptr::null_mut();
    }

    hlfntOld = (*(*pdc).dclevel.plfnt).BaseObject.hHmgr as _;

    if hlfntNew != hlfntOld {
        plfntNew = LFONT_ShareLockFont(hlfntNew as _);
        if !plfntNew.is_null() {
            LFONT_ShareUnlockFont((*pdc).dclevel.plfnt);

            (*pdc).dclevel.plfnt = plfntNew;
            (*(*pdc).pdcattr).hlfntNew = hlfntNew as _;

            (*(*pdc).pdcattr).ulDirty_ |= DIRTY_CHARSET;
            (*(*pdc).pdcattr).ulDirty_ &= !SLOW_WIDTHS;
        } else {
            (*(*pdc).pdcattr).hlfntNew = hlfntOld as _;
            hlfntOld = core::ptr::null_mut();
        }
    }

    hlfntOld
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSelectFont(
    hdc: HDC,
    hfont: HFONT,
) -> HFONT {
    let hfontOld: HFONT;
    let pdc: PDC;

    if hdc.is_null() || hfont.is_null() {
        return core::ptr::null_mut();
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return core::ptr::null_mut();
    }

    hfontOld = DC_hSelectFont(pdc, hfont);

    DC_UnlockDc(pdc);

    return hfontOld;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDCObject(hDC: HDC, ObjectType: INT) -> HANDLE {
    let SelObject: HGDIOBJ;
    let pdc: *mut DC;
    let pdcattr: PDC_ATTR;

    if hDC.is_null() {
        return core::ptr::null_mut();
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        return core::ptr::null_mut();
    }
    pdcattr = (*pdc).pdcattr;

    if ((*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY)) != 0 {
        #[cfg(target_arch = "x86")]
        {
            DC_vUpdateFillBrush(pdc);
        }
        #[cfg(not(target_arch = "x86"))]
        {
            DC_vUpdateFillBrush(pdc);
        }
    }

    if ((*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY)) != 0 {
        #[cfg(target_arch = "x86")]
        {
            DC_vUpdateLineBrush(pdc);
        }
        #[cfg(not(target_arch = "x86"))]
        {
            DC_vUpdateLineBrush(pdc);
        }
    }

    match ObjectType as u32 {
        GDI_OBJECT_TYPE_EXTPEN | GDI_OBJECT_TYPE_PEN => {
            SelObject = (*pdcattr).hpen as _;
        }

        GDI_OBJECT_TYPE_BRUSH => {
            SelObject = (*pdcattr).hbrush as _;
        }

        GDI_OBJECT_TYPE_PALETTE => {
            SelObject = (*pdc).dclevel.hpal as _;
        }

        GDI_OBJECT_TYPE_FONT => {
            SelObject = (*pdcattr).hlfntNew as _;
        }

        GDI_OBJECT_TYPE_BITMAP => {
            let psurf: *mut SURFACE = (*pdc).dclevel.pSurface;
            SelObject = if !psurf.is_null() {
                (*psurf).BaseObject.hHmgr
            } else {
                (*(*(&raw const StockObjects)).as_ptr().add(DEFAULT_BITMAP as usize))
            };
        }

        GDI_OBJECT_TYPE_COLORSPACE => {
            SelObject = core::ptr::null_mut();
        }

        _ => {
            SelObject = core::ptr::null_mut();
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    DC_UnlockDc(pdc);
    SelObject
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDCObject(hDC: HDC, ObjectType: INT) -> HANDLE {
    let SelObject: HGDIOBJ;
    let pdc: *mut DC;
    let pdcattr: PDC_ATTR;

    if hDC.is_null() {
        return core::ptr::null_mut();
    }

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        return core::ptr::null_mut();
    }
    pdcattr = (*pdc).pdcattr;

    if ((*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY)) != 0 {
        #[cfg(target_arch = "x86")]
        {
            DC_vUpdateFillBrush(pdc);
        }
        #[cfg(not(target_arch = "x86"))]
        {
            DC_vUpdateFillBrush(pdc);
        }
    }

    if ((*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY)) != 0 {
        #[cfg(target_arch = "x86")]
        {
            DC_vUpdateLineBrush(pdc);
        }
        #[cfg(not(target_arch = "x86"))]
        {
            DC_vUpdateLineBrush(pdc);
        }
    }

    match ObjectType as u32 {
        GDI_OBJECT_TYPE_EXTPEN | GDI_OBJECT_TYPE_PEN => {
            SelObject = (*pdcattr).hpen as _;
        }

        GDI_OBJECT_TYPE_BRUSH => {
            SelObject = (*pdcattr).hbrush as _;
        }

        GDI_OBJECT_TYPE_PALETTE => {
            SelObject = (*pdc).dclevel.hpal as _;
        }

        GDI_OBJECT_TYPE_FONT => {
            SelObject = (*pdcattr).hlfntNew as _;
        }

        GDI_OBJECT_TYPE_BITMAP => {
            let psurf: *mut SURFACE = (*pdc).dclevel.pSurface;
            SelObject = if !psurf.is_null() {
                (*psurf).BaseObject.hHmgr
            } else {
                *(*(&raw const StockObjects)).as_ptr().add(DEFAULT_BITMAP as usize)
            };
        }

        GDI_OBJECT_TYPE_COLORSPACE => {
            SelObject = core::ptr::null_mut();
        }

        _ => {
            SelObject = core::ptr::null_mut();
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    DC_UnlockDc(pdc);
    SelObject
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetRandomRgn(
    hdc: HDC,
    hrgnDest: HRGN,
    iCode: i32,
) -> i32 {
    let mut ret: i32 = 0;
    let pdc: PDC;
    let mut prgnSrc: PREGION = core::ptr::null_mut();

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return -1;
    }

    match iCode as u32 {
        CLIPRGN => {
            prgnSrc = (*pdc).dclevel.prgnClip;
        }

        METARGN => {
            prgnSrc = (*pdc).dclevel.prgnMeta;
        }

        APIRGN => {
            if (*pdc).fs & (DC_DIRTY_RAO as i32) != 0 {
                CLIPPING_UpdateGCRegion(pdc);
            }
            if !(*pdc).prgnAPI.is_null() {
                prgnSrc = (*pdc).prgnAPI;
            } else if !(*pdc).dclevel.prgnClip.is_null() {
                prgnSrc = (*pdc).dclevel.prgnClip;
            } else if !(*pdc).dclevel.prgnMeta.is_null() {
                prgnSrc = (*pdc).dclevel.prgnMeta;
            }
        }

        SYSRGN => {
            prgnSrc = (*pdc).prgnVis;
        }

        _ => {}
    }

    if !prgnSrc.is_null() {
        let prgnDest: PREGION = REGION_LockRgn(hrgnDest);
        if !prgnDest.is_null() {
            ret = if IntGdiCombineRgn(prgnDest, prgnSrc, core::ptr::null_mut(), RGN_COPY as INT) == ERROR as i32 {
                -1
            } else {
                1
            };
            if (ret == 1) && (iCode as u32 == SYSRGN) {
                ret = REGION_bOffsetRgn(prgnDest, (*pdc).ptlDCOrig.x, (*pdc).ptlDCOrig.y) as i32;
            }
            REGION_UnlockRgn(prgnDest);
        } else {
            ret = -1;
        }
    }

    DC_UnlockDc(pdc);

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEnumObjects(
    hdc: HDC,
    iObjectType: INT,
    cjBuf: ULONG,
    pvBuf: PVOID,
) -> ULONG {
    0
}

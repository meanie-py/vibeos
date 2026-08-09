use win32k_sys::win32k::*;
use ntkrnl_sys::STATUS_SUCCESS;

// C debug macros — silenced via macro_rules! (variadic no-op).
#[allow(unused_macros)] macro_rules! TRACE { ($($t:tt)*) => { () }; }
#[allow(unused_macros)] macro_rules! ERR { ($($t:tt)*) => { () }; }
#[allow(unused_macros)] macro_rules! WARN { ($($t:tt)*) => { () }; }
#[allow(unused_macros)] macro_rules! FIXME { ($($t:tt)*) => { () }; }

// PALETTE_ulGetRGBColorFromIndex / PALETTE_vSetRGBColorForIndex are
// FORCEINLINE in win32ss/gdi/ntgdi/palette.h:140-159 — provide #[no_mangle]
// cdecl wrappers so still-C TUs can link against them too.
#[no_mangle]
pub unsafe extern "C" fn PALETTE_ulGetRGBColorFromIndex(
    ppal: PPALETTE,
    ulIndex: ULONG,
) -> ULONG {
    if ulIndex >= (*ppal).NumColors { return 0; }
    let e = (*ppal).IndexedColors.add(ulIndex as usize);
    // RGB(r,g,b) = r | (g<<8) | (b<<16)
    (((*e).peRed as ULONG) & 0xFF)
        | (((*e).peGreen as ULONG) & 0xFF) << 8
        | (((*e).peBlue as ULONG) & 0xFF) << 16
}

#[no_mangle]
pub unsafe extern "C" fn PALETTE_vSetRGBColorForIndex(
    ppal: PPALETTE,
    ulIndex: ULONG,
    crColor: COLORREF,
) {
    if ulIndex >= (*ppal).NumColors { return; }
    let e = (*ppal).IndexedColors.add(ulIndex as usize);
    (*e).peRed = (crColor & 0xFF) as u8;
    (*e).peGreen = ((crColor >> 8) & 0xFF) as u8;
    (*e).peBlue = ((crColor >> 16) & 0xFF) as u8;
}

// EXLATEOBJ_vInitialize/Cleanup are NTAPI in C (win32ss/gdi/eng/xlateobj.c).
// NTAPI = stdcall on i386, cdecl on amd64/arm64; `extern "system"` resolves
// per-arch. ppalSrc/ppalDst are typed `PALETTE *` in C — must match here
// for type compatibility with our typed `gpalRGB` reference.
extern "system" {
    fn EXLATEOBJ_vInitialize(
        exlo: *mut EXLATEOBJ,
        ppalSrc: *mut _PALETTE,
        ppalDst: *mut _PALETTE,
        crSrcBackColor: COLORREF,
        crDstBackColor: COLORREF,
        crDstForeColor: COLORREF,
    );
    fn EXLATEOBJ_vCleanup(exlo: *mut EXLATEOBJ);
}

// ERR/DPRINT macros from <debug.h>: variadic. The C uses NDEBUG=0/1 to gate;
// for kernel binary, ERR is a real symbol that takes (channel, fmt, ...).
// Mirror the pattern used in ldevobj.rs/pdevobj.rs.
extern "C" {

}

// BitsPerFormat(Format) is `gajBitsPerFormat[Format]` (win32ss/gdi/eng/surface.h:109).
// WIDTH_BYTES_ALIGN32(cx, bpp) is `((((cx)*(bpp)+31) & ~31) >> 3)`
// (win32ss/gdi/gdi32/objects/bitmap.c:55). Both are macros, not exported
// functions — inline them here per win32k_translation_prompt.md §6.
#[inline(always)]
unsafe fn BitsPerFormat(format: ULONG) -> ULONG {
    // gajBitsPerFormat is `[UCHAR; 0]` in bindgen (flexible array). The actual
    // C is `extern UCHAR gajBitsPerFormat[]`. Read via raw pointer indexing.
    let base = core::ptr::addr_of!(gajBitsPerFormat) as *const UCHAR;
    *base.add(format as usize) as ULONG
}

#[inline(always)]
fn WIDTH_BYTES_ALIGN32(cx: LONG, bpp: ULONG) -> LONG {
    ((((cx as i64) * (bpp as i64) + 31) & !31) >> 3) as LONG
}

// gpalRGB is a `PALETTE` (struct, not pointer) defined in palette.c. Bindgen
// declares it as `pub static mut gpalRGB: PALETTE;` — we don't need our own
// extern decl, just reference it via &raw mut from `win32k_sys::win32k::*`.

// BRUSHBODY view — the C `_BRUSH` struct embeds `BRUSHBODY` anonymously
// (win32ss/gdi/ntgdi/brush.h:58); bindgen lost the field names and emits
// `__bindgen_padding_0`. We reconstruct a typed view over the bytes after
// `BaseObject` so we can access `iHatch`/`hbmPattern`/`flAttrs`/`BrushAttr`.
// Layout matches BRUSHBODY exactly through `BrushAttr` (the only fields
// engbrush.c touches).
#[repr(C)]
struct BRUSHBODY_View {
    iHatch: ULONG,
    hbmPattern: HBITMAP,
    hbmClient: HBITMAP,
    flAttrs: ULONG,
    ulBrushUnique: ULONG,
    pBrushAttr: *mut _BRUSH_ATTR,
    BrushAttr: _BRUSH_ATTR,
    // Trailing BRUSHBODY fields elided — engbrush.c doesn't touch them.
}

#[inline(always)]
unsafe fn brushbody(pbrush: PBRUSH) -> *mut BRUSHBODY_View {
    // BRUSHBODY immediately follows BaseObject in `_BRUSH`.
    (pbrush as *mut u8).add(core::mem::size_of::<BASEOBJECT>()) as *mut BRUSHBODY_View
}

// ---- file-scope preamble (translated) ----
pub static gaulHatchBrushes: [[ULONG; 8]; 6] = [
    [0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF],
    [0xF7, 0xF7, 0xF7, 0xF7, 0xF7, 0xF7, 0xF7, 0xF7],
    [0xFE, 0xFD, 0xFB, 0xF7, 0xEF, 0xDF, 0xBF, 0x7F],
    [0x7F, 0xBF, 0xDF, 0xEF, 0xF7, 0xFB, 0xFD, 0xFE],
    [0xF7, 0xF7, 0xF7, 0xF7, 0x00, 0xF7, 0xF7, 0xF7],
    [0x7E, 0xBD, 0xDB, 0xE7, 0xE7, 0xDB, 0xBD, 0x7E],
];

// Definition of `gahsurfHatch` — engbrush.c is the C owner (now commented out
// via cohabitation in win32ss/CMakeLists.txt). #[no_mangle] so other TUs that
// reference `extern HSURF gahsurfHatch[]` link against us.
#[no_mangle]
pub static mut gahsurfHatch: [HSURF; 6] = [core::ptr::null_mut(); 6];

#[inline(always)]
unsafe fn InitBrushImpl_impl() -> NTSTATUS {
    let mut i: ULONG;
    let sizl: SIZEL = SIZEL { cx: 8, cy: 8 };

    i = 0;
    while i < HS_DDI_MAX {
        gahsurfHatch[i as usize] = EngCreateBitmap(
            sizl,
            0,
            BMF_1BPP,
            0,
            gaulHatchBrushes[i as usize].as_ptr() as PVOID,
        ) as HSURF;
        i += 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn InitBrushImpl() -> NTSTATUS {
    InitBrushImpl_impl()
}

// EBRUSHOBJ_vInit is exported (win32k-internal, NTAPI). Real callers exist
// in still-C files (cursoricon.c, bitblt.c, etc.), so this MUST be
// `#[no_mangle]` with per-arch NTAPI ABI: stdcall on i386 (=> `_EBRUSHOBJ_vInit@24`
// decoration matching 6×4=24 bytes of args), cdecl on amd64/arm64.
#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_vInit(
    pebo: *mut EBRUSHOBJ,
    pbrush: PBRUSH,
    psurf: PSURFACE,
    crBackgroundClr: COLORREF,
    crForegroundClr: COLORREF,
    ppalDC: PPALETTE,
) { ebrushobj_v_init_impl(pebo, pbrush, psurf, crBackgroundClr, crForegroundClr, ppalDC) }

#[inline(always)]
unsafe fn ebrushobj_v_init_impl(
    pebo: *mut EBRUSHOBJ,
    pbrush: PBRUSH,
    psurf: PSURFACE,
    crBackgroundClr: COLORREF,
    crForegroundClr: COLORREF,
    ppalDC: PPALETTE,
) {
    let pbody = brushbody(pbrush);

    (*pebo).BrushObject.flColorType = 0;
    (*pebo).BrushObject.pvRbrush = core::ptr::null_mut();
    (*pebo).pbrush = pbrush;
    (*pebo).pengbrush = core::ptr::null_mut();
    (*pebo).flattrs = (*pbody).flAttrs;
    (*pebo).psoMask = core::ptr::null_mut();

    (*pebo).crCurrentBack = crBackgroundClr;
    (*pebo).crCurrentText = crForegroundClr;

    (*pebo).psurfTrg = psurf;
    if (*pebo).psurfTrg.is_null() {
        (*pebo).psurfTrg = psurfDefaultBitmap;
    }

    (*pebo).ppalSurf = (*(*pebo).psurfTrg).ppal;
    GDIOBJ_vReferenceObjectByPointer(&mut (*(*pebo).ppalSurf).BaseObject as POBJ);
    (*pebo).ppalDC = ppalDC;
    if (*pebo).ppalDC.is_null() {
        (*pebo).ppalDC = gppalDefault;
    }
    GDIOBJ_vReferenceObjectByPointer(&mut (*(*pebo).ppalDC).BaseObject as POBJ);
    (*pebo).ppalDIB = core::ptr::null_mut();

    if (*pbody).flAttrs & BR_IS_NULL != 0 {
        (*pebo).BrushObject.iSolidColor = 0;
    } else if (*pbody).flAttrs & BR_IS_SOLID != 0 {
        EBRUSHOBJ_vSetSolidRGBColor(pebo, (*pbody).BrushAttr.lbColor);
    } else {
        (*pebo).BrushObject.iSolidColor = 0xFFFFFFFF;

        if (*pbody).flAttrs & BR_IS_HATCH != 0 {
            (*pebo).crCurrentText = (*pbody).BrushAttr.lbColor;
        }
    }
}

// EBRUSHOBJ_vInitFromDC is also exported (NTAPI, 3 args = `@12` on i386).
#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_vInitFromDC(
    pebo: *mut EBRUSHOBJ,
    pbrush: PBRUSH,
    pdc: PDC,
) { ebrushobj_v_init_from_dc_impl(pebo, pbrush, pdc) }

#[inline(always)]
unsafe fn ebrushobj_v_init_from_dc_impl(
    pebo: *mut EBRUSHOBJ,
    pbrush: PBRUSH,
    pdc: PDC,
) {
    // pdcattr is `PDC_ATTR` (pointer to _DC_ATTR) in the C struct — needs
    // an extra deref before field access.
    ebrushobj_v_init_impl(
        pebo,
        pbrush,
        (*pdc).dclevel.pSurface,
        (*(*pdc).pdcattr).crBackgroundClr,
        (*(*pdc).pdcattr).crForegroundClr,
        (*pdc).dclevel.ppal,
    );
}

// FASTCALL in C — `@EBRUSHOBJ_vSetSolidRGBColor@8` decoration on i386.
// Exported because callers exist in still-C TUs (brush.c etc.).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn EBRUSHOBJ_vSetSolidRGBColor(pebo: *mut EBRUSHOBJ, crColor: COLORREF) {
    EBRUSHOBJ_vSetSolidRGBColor_impl(pebo, crColor)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EBRUSHOBJ_vSetSolidRGBColor(pebo: *mut EBRUSHOBJ, crColor: COLORREF) {
    EBRUSHOBJ_vSetSolidRGBColor_impl(pebo, crColor)
}

unsafe fn EBRUSHOBJ_vSetSolidRGBColor_impl(pebo: *mut EBRUSHOBJ, crColor: COLORREF) {
    let iSolidColor: ULONG;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();

    let crColor = crColor & 0xFFFFFF;
    (*pebo).crRealize = crColor;
    (*pebo).ulRGBColor = crColor;

    if (*(*pebo).ppalSurf).flFlags & (PAL_MONOCHROME as ULONG) != 0 {
        let iBackIndex = PALETTE_ulGetNearestPaletteIndex((*pebo).ppalSurf, (*pebo).crCurrentBack);
        let iForeIndex = iBackIndex ^ 1;

        let rgbBack = PALETTE_ulGetRGBColorFromIndex((*pebo).ppalSurf, iBackIndex);

        if crColor == rgbBack || crColor == (*pebo).crCurrentBack {
            (*pebo).BrushObject.iSolidColor = iBackIndex;
        } else {
            (*pebo).BrushObject.iSolidColor = iForeIndex;
        }

        (*pebo).flattrs |= BR_NEED_BK_CLR;
    } else {
        EXLATEOBJ_vInitialize(
            &mut exlo,
            core::ptr::addr_of_mut!(gpalRGB),
            (*pebo).ppalSurf,
            (*pebo).crCurrentBack,
            0,
            0,
        );

        iSolidColor = XLATEOBJ_iXlate(core::ptr::addr_of_mut!(exlo.xlo), crColor);
        (*pebo).BrushObject.iSolidColor = iSolidColor;

        EXLATEOBJ_vCleanup(&mut exlo);
    }
}

// NTAPI in C (brush.h:165) — called from win32ss/gdi/ntgdi/dclife.c,
// gdibatch.c, bitblt.split2.c, user/ntuser/cursoricon.split2.c.
// `extern "system"` resolves to stdcall on i386, cdecl on amd64/arm64.
#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_vCleanup(pebo: *mut EBRUSHOBJ) {
    if !(*pebo).pengbrush.is_null() {
        // SURFACE_ShareUnlockSurface(psurf) → GDIOBJ_vDereferenceObject(psurf as POBJ)
        // (win32ss/gdi/eng/surface.h:102). Per win32k_translation_prompt.md §6.
        GDIOBJ_vDereferenceObject((*pebo).pengbrush as POBJ);
        (*pebo).pengbrush = core::ptr::null_mut();
    }

    if !(*pebo).BrushObject.pvRbrush.is_null() {
        EngFreeMem((*pebo).BrushObject.pvRbrush);
        (*pebo).BrushObject.pvRbrush = core::ptr::null_mut();
    }

    if !(*pebo).psoMask.is_null() {
        // SURFOBJ inhabits the head of SURFACE (psoMask points at the SurfObj
        // field, which is at offset 0 of SURFACE on all arches). The C
        // SURFACE_ShareUnlockSurface(p) takes any pointer that resolves to
        // the SURFACE base via cast — mirror by casting to POBJ.
        GDIOBJ_vDereferenceObject((*pebo).psoMask as POBJ);
        (*pebo).psoMask = core::ptr::null_mut();
    }

    if !(*pebo).ppalSurf.is_null() {
        // PALETTE_ShareUnlockPalette(ppal) → GDIOBJ_vDereferenceObject(&ppal->BaseObject)
        // (win32ss/gdi/ntgdi/palette.h:59). Per §6.
        GDIOBJ_vDereferenceObject(&mut (*(*pebo).ppalSurf).BaseObject as POBJ);
    }
    if !(*pebo).ppalDC.is_null() {
        GDIOBJ_vDereferenceObject(&mut (*(*pebo).ppalDC).BaseObject as POBJ);
    }
    if !(*pebo).ppalDIB.is_null() {
        GDIOBJ_vDereferenceObject(&mut (*(*pebo).ppalDIB).BaseObject as POBJ);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_vUpdateFromDC(
    pebo: *mut EBRUSHOBJ,
    pbrush: PBRUSH,
    pdc: PDC,
) {
    EBRUSHOBJ_vCleanup(pebo);
    EBRUSHOBJ_vInitFromDC(pebo, pbrush, pdc);
}

// EngRealizeBrush is documented APIENTRY (= __stdcall on i386, cdecl
// elsewhere) — `extern "system"` resolves correctly per-arch.
#[no_mangle]
pub unsafe extern "system" fn EngRealizeBrush(
    pbo: *mut BRUSHOBJ,
    psoDst: *mut SURFOBJ,
    psoPattern: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    iHatch: ULONG,
) -> BOOL {
    let _ = psoMask; // C function ignores psoMask
    let _ = iHatch;
    let hbmpRealize: HBITMAP;
    let psoRealize: *mut SURFOBJ;
    let psurfRealize: PSURFACE;
    let mut ptlSrc = POINTL { x: 0, y: 0 };
    let mut rclDest: RECTL = core::mem::zeroed();
    let lWidth: LONG;

    lWidth = WIDTH_BYTES_ALIGN32(
        (*psoPattern).sizlBitmap.cx,
        BitsPerFormat((*psoDst).iBitmapFormat),
    );

    hbmpRealize = EngCreateBitmap(
        (*psoPattern).sizlBitmap,
        lWidth,
        (*psoDst).iBitmapFormat,
        BMF_NOZEROINIT,
        core::ptr::null_mut(),
    );
    if hbmpRealize.is_null() {
        return FALSE as BOOL;
    }

    // SURFACE_ShareLockSurface(h) →
    //   GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE
    // (win32ss/gdi/eng/surface.h:91). Per win32k_translation_prompt.md §6.
    psurfRealize = GDIOBJ_ShareLockObj(hbmpRealize as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE;

    EngDeleteSurface(hbmpRealize as HSURF);

    if psurfRealize.is_null() {
        return FALSE as BOOL;
    }

    rclDest.left = 0;
    rclDest.top = 0;
    rclDest.right = (*psoPattern).sizlBitmap.cx;
    rclDest.bottom = (*psoPattern).sizlBitmap.cy;
    psoRealize = core::ptr::addr_of_mut!((*psurfRealize).SurfObj);
    EngCopyBits(psoRealize, psoPattern, core::ptr::null_mut(), pxlo, &mut rclDest, &mut ptlSrc);

    let pebo = (pbo as *mut u8)
        .offset(-(core::mem::offset_of!(EBRUSHOBJ, BrushObject) as isize))
        as *mut EBRUSHOBJ;
    (*pebo).pengbrush = psurfRealize as PVOID;

    TRUE as BOOL
}

unsafe fn FixupDIBBrushPalette(
    ppalDIB: PPALETTE,
    ppalDC: PPALETTE,
) -> PPALETTE {
    let ppalNew: PPALETTE;
    let mut i: ULONG;
    let mut iPalIndex: ULONG;
    let mut crColor: ULONG;

    ppalNew = PALETTE_AllocPalette(
        PAL_INDEXED as ULONG,
        (*ppalDIB).NumColors,
        core::ptr::null_mut(),
        0,
        0,
        0,
    );
    if ppalNew.is_null() {
        ERR!(b"Failed to allcate palette for brush\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    i = 0;
    while i < (*ppalDIB).NumColors {
        iPalIndex = PALETTE_ulGetRGBColorFromIndex(ppalDIB, i);

        iPalIndex %= (*ppalDC).NumColors;

        crColor = PALETTE_ulGetRGBColorFromIndex(ppalDC, iPalIndex);
        PALETTE_vSetRGBColorForIndex(ppalNew, i, crColor);

        i += 1;
    }

    ppalNew
}

#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_bRealizeBrush(
    pebo: *mut EBRUSHOBJ,
    bCallDriver: BOOL,
) -> BOOL {
    let bResult: BOOL;
    let pfnRealizeBrush: PFN_DrvRealizeBrush;
    let psurfPattern: PSURFACE;
    let psoMask: *mut SURFOBJ;
    let mut ppdev: PPDEVOBJ;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let ppalPattern: PPALETTE;
    let pbr: PBRUSH = (*pebo).pbrush;
    let pbrBody = brushbody(pbr);
    let hbmPattern: HBITMAP;
    let iHatch: ULONG;

    ppdev = (*(*pebo).psurfTrg).SurfObj.hdev as PPDEVOBJ;
    if ppdev.is_null() {
        ppdev = (*gpmdev).ppdevGlobal;
    }

    if bCallDriver != FALSE as BOOL {
        // _PDEVOBJ has DriverFunctions inside an anonymous union (bindgen).
        pfnRealizeBrush = (*ppdev).__bindgen_anon_1.DriverFunctions.RealizeBrush;
        if pfnRealizeBrush.is_none() {
            ERR!(b"No DrvRealizeBrush. Cannot realize brush\n\0".as_ptr() as *const i8);
            return FALSE as BOOL;
        }

        psoMask = EBRUSHOBJ_psoMask(pebo);
    } else {
        // PFN_DrvRealizeBrush is spelled `system` in the island now, the same as
        // EngRealizeBrush, so this assigns directly. It used to need a transmute per arch to
        // reach bindgen's literal `stdcall`/`C` — and that transmute erased the six-argument
        // signature along with the convention, so a drift in either would have compiled.
        pfnRealizeBrush = Some(EngRealizeBrush);

        psoMask = core::ptr::null_mut();
    }

    if (*pbrBody).flAttrs & BR_IS_HATCH != 0 {
        hbmPattern = (*ppdev).ahsurf[(*pbrBody).iHatch as usize] as HBITMAP;
        iHatch = (*pbrBody).iHatch;
    } else {
        hbmPattern = (*pbrBody).hbmPattern;
        iHatch = 0xFFFFFFFFu32;
    }

    // SURFACE_ShareLockSurface(h) → GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BITMAP) as PSURFACE
    psurfPattern = GDIOBJ_ShareLockObj(hbmPattern as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE;

    if (*pbrBody).flAttrs & BR_IS_DIBPALCOLORS != 0 {
        ppalPattern = FixupDIBBrushPalette((*psurfPattern).ppal, (*pebo).ppalDC);
        if ppalPattern.is_null() {
            ERR!(b"FixupDIBBrushPalette() failed.\n\0".as_ptr() as *const i8);
            return FALSE as BOOL;
        }

        (*pebo).ppalDIB = ppalPattern;
    } else {
        ppalPattern = (*psurfPattern).ppal;
    }

    EXLATEOBJ_vInitialize(
        &mut exlo,
        ppalPattern,
        (*(*pebo).psurfTrg).ppal,
        0,
        (*pebo).crCurrentBack,
        (*pebo).crCurrentText,
    );

    bResult = pfnRealizeBrush.unwrap()(
        &mut (*pebo).BrushObject,
        &mut (*(*pebo).psurfTrg).SurfObj,
        &mut (*psurfPattern).SurfObj,
        psoMask,
        &mut exlo.xlo,
        iHatch,
    );

    EXLATEOBJ_vCleanup(&mut exlo);

    // SURFACE_ShareUnlockSurface(psurf) → GDIOBJ_vDereferenceObject(psurf as POBJ)
    GDIOBJ_vDereferenceObject(psurfPattern as POBJ);

    bResult
}

// EBRUSHOBJ_psoMask — NTAPI, called via BRUSHOBJ_psoMask macro from
// gdi/eng/bitblt.c and bitblt_new.c; declared in brush.h:180. The C body
// is at win32ss/gdi/eng/engbrush.c:466 (kept in the dropped comment block
// at file tail). Translated here because the rustconv pipeline silently
// dropped it. See feedback_rustconv_dropped_fn_calls.md.
#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_psoMask(pebo: *mut EBRUSHOBJ) -> *mut SURFOBJ {
    if (*pebo).psoMask.is_null() {
        if (*pebo).flattrs & BR_IS_HATCH != 0 {
            let mut ppdev: PPDEVOBJ = (*(*pebo).psurfTrg).SurfObj.hdev as PPDEVOBJ;
            if ppdev.is_null() {
                ppdev = (*gpmdev).ppdevGlobal;
            }

            let pbrBody = brushbody((*pebo).pbrush);
            let hbmMask: HBITMAP = (*ppdev).ahsurf[(*pbrBody).iHatch as usize] as HBITMAP;
            // SURFACE_ShareLockSurface inline-expanded per §6.
            let psurfMask: PSURFACE =
                GDIOBJ_ShareLockObj(hbmMask as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE;
            if psurfMask.is_null() {
                ERR!(b"Failed to lock hatch brush\n\0".as_ptr() as *const i8);
                return core::ptr::null_mut();
            }

            (*pebo).psoMask = core::ptr::addr_of_mut!((*psurfMask).SurfObj);
        }
    }

    (*pebo).psoMask
}

// EBRUSHOBJ_psoPattern — NTAPI, called via BRUSHOBJ_psoPattern macro from
// bitblt.c/stretchblt.c. Declared in brush.h:173. Translated here for the
// same reason as EBRUSHOBJ_psoMask above.
#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_psoPattern(pebo: *mut EBRUSHOBJ) -> *mut SURFOBJ {
    let psurfPattern: PSURFACE = EBRUSHOBJ_pvGetEngBrush(pebo) as PSURFACE;
    if psurfPattern.is_null() {
        core::ptr::null_mut()
    } else {
        core::ptr::addr_of_mut!((*psurfPattern).SurfObj)
    }
}

#[no_mangle]
pub unsafe extern "system" fn EBRUSHOBJ_pvGetEngBrush(pebo: *mut EBRUSHOBJ) -> PVOID {
    let bResult: BOOL;

    if (*pebo).pengbrush.is_null() {
        bResult = EBRUSHOBJ_bRealizeBrush(pebo, FALSE as BOOL);
        if bResult == FALSE as BOOL {
            if !(*pebo).pengbrush.is_null() {
                EngDeleteSurface((*pebo).pengbrush as HSURF);
            }
            (*pebo).pengbrush = core::ptr::null_mut();
        }
    }

    (*pebo).pengbrush
}
// BRUSHOBJ_* are APIENTRY (= __stdcall on i386, __cdecl on amd64/arm64).
// `extern "system"` resolves to the correct per-arch ABI; #[no_mangle]
// preserves the symbol name (i386 gets `_BRUSHOBJ_pvAllocRbrush@8`
// decoration automatically via extern "system" + no_mangle).
// GDITAG_RBRUSH = 'rbsh' = 0x68736272 (little-endian).
#[no_mangle]
pub unsafe extern "system" fn BRUSHOBJ_pvAllocRbrush(
    pbo: *mut BRUSHOBJ,
    cj: ULONG,
) -> PVOID {
    (*pbo).pvRbrush = EngAllocMem(0, cj, 0x68736272u32);
    (*pbo).pvRbrush
}

#[no_mangle]
pub unsafe extern "system" fn BRUSHOBJ_pvGetRbrush(
    pbo: *mut BRUSHOBJ,
) -> PVOID {
    let pebo = (pbo as *mut u8)
        .offset(-(core::mem::offset_of!(EBRUSHOBJ, BrushObject) as isize))
        as *mut EBRUSHOBJ;
    let bResult: BOOL;

    if (*pbo).pvRbrush.is_null() {
        bResult = EBRUSHOBJ_bRealizeBrush(pebo, TRUE as BOOL);
        if bResult == FALSE as BOOL {
            if !(*pbo).pvRbrush.is_null() {
                EngFreeMem((*pbo).pvRbrush);
                (*pbo).pvRbrush = core::ptr::null_mut();
            }
        }
    }

    (*pbo).pvRbrush
}

#[no_mangle]
pub unsafe extern "system" fn BRUSHOBJ_ulGetBrushColor(
    pbo: *mut BRUSHOBJ,
) -> ULONG {
    let pebo = (pbo as *mut u8)
        .offset(-(core::mem::offset_of!(EBRUSHOBJ, BrushObject) as isize))
        as *mut EBRUSHOBJ;
    (*pebo).ulRGBColor
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Driver Brush Functions
//  * FILE:              win32ss/gdi/eng/engbrush.c
//  * PROGRAMER:         Jason Filby
//  *                    Timo Kreuzer
//  */
//
// #include <win32k.h>
//
// DBG_DEFAULT_CHANNEL(EngBrush);
//
// static const ULONG gaulHatchBrushes[HS_DDI_MAX][8] =
// {
//     {0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF}, /* HS_HORIZONTAL */
//     {0xF7, 0xF7, 0xF7, 0xF7, 0xF7, 0xF7, 0xF7, 0xF7}, /* HS_VERTICAL   */
//     {0xFE, 0xFD, 0xFB, 0xF7, 0xEF, 0xDF, 0xBF, 0x7F}, /* HS_FDIAGONAL  */
//     {0x7F, 0xBF, 0xDF, 0xEF, 0xF7, 0xFB, 0xFD, 0xFE}, /* HS_BDIAGONAL  */
//     {0xF7, 0xF7, 0xF7, 0xF7, 0x00, 0xF7, 0xF7, 0xF7}, /* HS_CROSS      */
//     {0x7E, 0xBD, 0xDB, 0xE7, 0xE7, 0xDB, 0xBD, 0x7E}  /* HS_DIAGCROSS  */
// };
//
// HSURF gahsurfHatch[HS_DDI_MAX];
//
// /** Internal functions ********************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitBrushImpl(VOID)
// {
//     ULONG i;
//     SIZEL sizl = {8, 8};
//
//     /* Loop all hatch styles */
//     for (i = 0; i < HS_DDI_MAX; i++)
//     {
//         /* Create a default hatch bitmap */
//         gahsurfHatch[i] = (HSURF)EngCreateBitmap(sizl,
//                                                  0,
//                                                  BMF_1BPP,
//                                                  0,
//                                                  (PVOID)gaulHatchBrushes[i]);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EBRUSHOBJ_vInit(EBRUSHOBJ *pebo,
//     PBRUSH pbrush,
//     PSURFACE psurf,
//     COLORREF crBackgroundClr,
//     COLORREF crForegroundClr,
//     PPALETTE ppalDC)
// {
//     ASSERT(pebo);
//     ASSERT(pbrush);
//
//     pebo->BrushObject.flColorType = 0;
//     pebo->BrushObject.pvRbrush = NULL;
//     pebo->pbrush = pbrush;
//     pebo->pengbrush = NULL;
//     pebo->flattrs = pbrush->flAttrs;
//     pebo->psoMask = NULL;
//
//     /* Initialize 1 bpp fore and back colors */
//     pebo->crCurrentBack = crBackgroundClr;
//     pebo->crCurrentText = crForegroundClr;
//
//     pebo->psurfTrg = psurf;
//     /* We are initializing for a new memory DC */
//     if(!pebo->psurfTrg)
//         pebo->psurfTrg = psurfDefaultBitmap;
//     ASSERT(pebo->psurfTrg);
//     ASSERT(pebo->psurfTrg->ppal);
//
//     /* Initialize palettes */
//     pebo->ppalSurf = pebo->psurfTrg->ppal;
//     GDIOBJ_vReferenceObjectByPointer(&pebo->ppalSurf->BaseObject);
//     pebo->ppalDC = ppalDC;
//     if(!pebo->ppalDC)
//         pebo->ppalDC = gppalDefault;
//     GDIOBJ_vReferenceObjectByPointer(&pebo->ppalDC->BaseObject);
//     pebo->ppalDIB = NULL;
//
//     if (pbrush->flAttrs & BR_IS_NULL)
//     {
//         /* NULL brushes don't need a color */
//         pebo->BrushObject.iSolidColor = 0;
//     }
//     else if (pbrush->flAttrs & BR_IS_SOLID)
//     {
//         /* Set the RGB color */
//         EBRUSHOBJ_vSetSolidRGBColor(pebo, pbrush->BrushAttr.lbColor);
//     }
//     else
//     {
//         /* This is a pattern brush that needs realization */
//         pebo->BrushObject.iSolidColor = 0xFFFFFFFF;
//
//         /* Use foreground color of hatch brushes */
//         if (pbrush->flAttrs & BR_IS_HATCH)
//             pebo->crCurrentText = pbrush->BrushAttr.lbColor;
//     }
// }
//
// VOID
// NTAPI
// EBRUSHOBJ_vInitFromDC(EBRUSHOBJ *pebo,
//     PBRUSH pbrush, PDC pdc)
// {
//     EBRUSHOBJ_vInit(pebo, pbrush, pdc->dclevel.pSurface,
//         pdc->pdcattr->crBackgroundClr, pdc->pdcattr->crForegroundClr,
//         pdc->dclevel.ppal);
// }
//
// VOID
// FASTCALL
// EBRUSHOBJ_vSetSolidRGBColor(EBRUSHOBJ *pebo, COLORREF crColor)
// {
//     ULONG iSolidColor;
//     EXLATEOBJ exlo;
//
//     /* Never use with non-solid brushes */
//     ASSERT(pebo->flattrs & BR_IS_SOLID);
//
//     /* Set the RGB color */
//     crColor &= 0xFFFFFF;
//     pebo->crRealize = crColor;
//     pebo->ulRGBColor = crColor;
//
//     /* Special handling for mono-surfaces */
//     if (pebo->ppalSurf->flFlags & PAL_MONOCHROME)
//     {
//         /* Determine the indices for back and fore color */
//         ULONG iBackIndex =
//             PALETTE_ulGetNearestPaletteIndex(pebo->ppalSurf, pebo->crCurrentBack);
//         ULONG iForeIndex = iBackIndex ^ 1;
//
//         /* Get the translated back color */
//         ULONG rgbBack = PALETTE_ulGetRGBColorFromIndex(pebo->ppalSurf, iBackIndex);
//
//         /* Match the pen color against RGB and translated background color */
//         if ((crColor == rgbBack) || (crColor == pebo->crCurrentBack))
//                 pebo->BrushObject.iSolidColor = iBackIndex;
//         else
//             pebo->BrushObject.iSolidColor = iForeIndex;
//
//         pebo->flattrs |= BR_NEED_BK_CLR;
//     }
//     else
//     {
//         /* Initialize an XLATEOBJ RGB -> surface */
//         EXLATEOBJ_vInitialize(&exlo,
//                               &gpalRGB,
//                               pebo->ppalSurf,
//                               pebo->crCurrentBack,
//                               0,
//                               0);
//
//         /* Translate the brush color to the target format */
//         iSolidColor = XLATEOBJ_iXlate(&exlo.xlo, crColor);
//         pebo->BrushObject.iSolidColor = iSolidColor;
//
//         /* Clean up the XLATEOBJ */
//         EXLATEOBJ_vCleanup(&exlo);
//     }
// }
//
// VOID
// NTAPI
// EBRUSHOBJ_vCleanup(EBRUSHOBJ *pebo)
// {
//     /* Check if there's a GDI realisation */
//     if (pebo->pengbrush)
//     {
//         /* Unlock the bitmap again */
//         SURFACE_ShareUnlockSurface(pebo->pengbrush);
//         pebo->pengbrush = NULL;
//     }
//
//     /* Check if there's a driver's realisation */
//     if (pebo->BrushObject.pvRbrush)
//     {
//         /* Free allocated driver memory */
//         EngFreeMem(pebo->BrushObject.pvRbrush);
//         pebo->BrushObject.pvRbrush = NULL;
//     }
//
//     if (pebo->psoMask != NULL)
//     {
//         SURFACE_ShareUnlockSurface(pebo->psoMask);
//         pebo->psoMask = NULL;
//     }
//
//     /* Dereference the palettes */
//     if (pebo->ppalSurf)
//     {
//         PALETTE_ShareUnlockPalette(pebo->ppalSurf);
//     }
//     if (pebo->ppalDC)
//     {
//         PALETTE_ShareUnlockPalette(pebo->ppalDC);
//     }
//     if (pebo->ppalDIB)
//     {
//         PALETTE_ShareUnlockPalette(pebo->ppalDIB);
//     }
// }
//
// VOID
// NTAPI
// EBRUSHOBJ_vUpdateFromDC(
//     EBRUSHOBJ *pebo,
//     PBRUSH pbrush,
//     PDC pdc)
// {
//     /* Cleanup the brush */
//     EBRUSHOBJ_vCleanup(pebo);
//
//     /* Reinitialize */
//     EBRUSHOBJ_vInitFromDC(pebo, pbrush, pdc);
// }
//
// /**
//  * This function is not exported, because it makes no sense for
//  * The driver to punt back to this function */
// BOOL
// APIENTRY
// EngRealizeBrush(
//     BRUSHOBJ *pbo,
//     SURFOBJ  *psoDst,
//     SURFOBJ  *psoPattern,
//     SURFOBJ  *psoMask,
//     XLATEOBJ *pxlo,
//     ULONG    iHatch)
// {
//     EBRUSHOBJ *pebo;
//     HBITMAP hbmpRealize;
//     SURFOBJ *psoRealize;
//     PSURFACE psurfRealize;
//     POINTL ptlSrc = {0, 0};
//     RECTL rclDest;
//     ULONG lWidth;
//
//     /* Calculate width in bytes of the realized brush */
//     lWidth = WIDTH_BYTES_ALIGN32(psoPattern->sizlBitmap.cx,
//                                   BitsPerFormat(psoDst->iBitmapFormat));
//
//     /* Allocate a bitmap */
//     hbmpRealize = EngCreateBitmap(psoPattern->sizlBitmap,
//                                   lWidth,
//                                   psoDst->iBitmapFormat,
//                                   BMF_NOZEROINIT,
//                                   NULL);
//     if (!hbmpRealize)
//     {
//         return FALSE;
//     }
//
//     /* Lock the bitmap */
//     psurfRealize = SURFACE_ShareLockSurface(hbmpRealize);
//
//     /* Already delete the pattern bitmap (will be kept until dereferenced) */
//     EngDeleteSurface((HSURF)hbmpRealize);
//
//     if (!psurfRealize)
//     {
//         return FALSE;
//     }
//
//     /* Copy the bits to the new format bitmap */
//     rclDest.left = rclDest.top = 0;
//     rclDest.right = psoPattern->sizlBitmap.cx;
//     rclDest.bottom = psoPattern->sizlBitmap.cy;
//     psoRealize = &psurfRealize->SurfObj;
//     EngCopyBits(psoRealize, psoPattern, NULL, pxlo, &rclDest, &ptlSrc);
//
//
//     pebo = CONTAINING_RECORD(pbo, EBRUSHOBJ, BrushObject);
//     pebo->pengbrush = (PVOID)psurfRealize;
//
//     return TRUE;
// }
//
// static
// PPALETTE
// FixupDIBBrushPalette(
//     _In_ PPALETTE ppalDIB,
//     _In_ PPALETTE ppalDC)
// {
//     PPALETTE ppalNew;
//     ULONG i, iPalIndex, crColor;
//
//     /* Allocate a new palette */
//     ppalNew = PALETTE_AllocPalette(PAL_INDEXED,
//                                    ppalDIB->NumColors,
//                                    NULL,
//                                    0,
//                                    0,
//                                    0);
//     if (ppalNew == NULL)
//     {
//         ERR("Failed to allcate palette for brush\n");
//         return NULL;
//     }
//
//     /* Loop all colors */
//     for (i = 0; i < ppalDIB->NumColors; i++)
//     {
//         /* Get the RGB color, which is the index into the DC palette */
//         iPalIndex = PALETTE_ulGetRGBColorFromIndex(ppalDIB, i);
//
//         /* Roll over when index is too big */
//         iPalIndex %= ppalDC->NumColors;
//
//         /* Set the indexed DC color as the new color */
//         crColor = PALETTE_ulGetRGBColorFromIndex(ppalDC, iPalIndex);
//         PALETTE_vSetRGBColorForIndex(ppalNew, i, crColor);
//     }
//
//     /* Return the new palette */
//     return ppalNew;
// }
//
// BOOL
// NTAPI
// EBRUSHOBJ_bRealizeBrush(EBRUSHOBJ *pebo, BOOL bCallDriver)
// {
//     BOOL bResult;
//     PFN_DrvRealizeBrush pfnRealizeBrush = NULL;
//     PSURFACE psurfPattern;
//     SURFOBJ *psoMask;
//     PPDEVOBJ ppdev;
//     EXLATEOBJ exlo;
//     PPALETTE ppalPattern;
//     PBRUSH pbr = pebo->pbrush;
//     HBITMAP hbmPattern;
//     ULONG iHatch;
//
//     /* All EBRUSHOBJs have a surface, see EBRUSHOBJ_vInit */
//     ASSERT(pebo->psurfTrg);
//
//     ppdev = (PPDEVOBJ)pebo->psurfTrg->SurfObj.hdev;
//     if (!ppdev)
//         ppdev = gpmdev->ppdevGlobal;
//
//     if (bCallDriver)
//     {
//         /* Get the Drv function */
//         pfnRealizeBrush = ppdev->DriverFunctions.RealizeBrush;
//         if (pfnRealizeBrush == NULL)
//         {
//             ERR("No DrvRealizeBrush. Cannot realize brush\n");
//             return FALSE;
//         }
//
//         /* Get the mask */
//         psoMask = EBRUSHOBJ_psoMask(pebo);
//     }
//     else
//     {
//         /* Use the Eng function */
//         pfnRealizeBrush = EngRealizeBrush;
//
//         /* We don't handle the mask bitmap here. We do this only on demand */
//         psoMask = NULL;
//     }
//
//     /* Check if this is a hatch brush */
//     if (pbr->flAttrs & BR_IS_HATCH)
//     {
//         /* Get the hatch brush pattern from the PDEV */
//         hbmPattern = (HBITMAP)ppdev->ahsurf[pbr->iHatch];
//         iHatch = pbr->iHatch;
//     }
//     else
//     {
//         /* Use the brushes pattern */
//         hbmPattern = pbr->hbmPattern;
//         iHatch = -1;
//     }
//
//     psurfPattern = SURFACE_ShareLockSurface(hbmPattern);
//     ASSERT(psurfPattern);
//     ASSERT(psurfPattern->ppal);
//
//     /* DIB brushes with DIB_PAL_COLORS usage need a new palette */
//     if (pbr->flAttrs & BR_IS_DIBPALCOLORS)
//     {
//         /* Create a palette with the colors from the DC */
//         ppalPattern = FixupDIBBrushPalette(psurfPattern->ppal, pebo->ppalDC);
//         if (ppalPattern == NULL)
//         {
//             ERR("FixupDIBBrushPalette() failed.\n");
//             return FALSE;
//         }
//
//         pebo->ppalDIB = ppalPattern;
//     }
//     else
//     {
//         /* The palette is already as it should be */
//         ppalPattern = psurfPattern->ppal;
//     }
//
//     /* Initialize XLATEOBJ for the brush */
//     EXLATEOBJ_vInitialize(&exlo,
//                           ppalPattern,
//                           pebo->psurfTrg->ppal,
//                           0,
//                           pebo->crCurrentBack,
//                           pebo->crCurrentText);
//
//     /* Create the realization */
//     bResult = pfnRealizeBrush(&pebo->BrushObject,
//                               &pebo->psurfTrg->SurfObj,
//                               &psurfPattern->SurfObj,
//                               psoMask,
//                               &exlo.xlo,
//                               iHatch);
//
//     /* Cleanup the XLATEOBJ */
//     EXLATEOBJ_vCleanup(&exlo);
//
//     /* Unlock surface */
//     SURFACE_ShareUnlockSurface(psurfPattern);
//
//     return bResult;
// }
//
// PVOID
// NTAPI
// EBRUSHOBJ_pvGetEngBrush(EBRUSHOBJ *pebo)
// {
//     BOOL bResult;
//
//     if (!pebo->pengbrush)
//     {
//         bResult = EBRUSHOBJ_bRealizeBrush(pebo, FALSE);
//         if (!bResult)
//         {
//             if (pebo->pengbrush)
//                 EngDeleteSurface(pebo->pengbrush);
//             pebo->pengbrush = NULL;
//         }
//     }
//
//     return pebo->pengbrush;
// }
//
// SURFOBJ*
// NTAPI
// EBRUSHOBJ_psoPattern(EBRUSHOBJ *pebo)
// {
//     PSURFACE psurfPattern;
//
//     psurfPattern = EBRUSHOBJ_pvGetEngBrush(pebo);
//
//     return psurfPattern ? &psurfPattern->SurfObj : NULL;
// }
//
// SURFOBJ*
// NTAPI
// EBRUSHOBJ_psoMask(EBRUSHOBJ *pebo)
// {
//     HBITMAP hbmMask;
//     PSURFACE psurfMask;
//     PPDEVOBJ ppdev;
//
//     /* Check if we don't have a mask yet */
//     if (pebo->psoMask == NULL)
//     {
//         /* Check if this is a hatch brush */
//         if (pebo->flattrs & BR_IS_HATCH)
//         {
//             /* Get the PDEV */
//             ppdev = (PPDEVOBJ)pebo->psurfTrg->SurfObj.hdev;
//             if (!ppdev)
//                 ppdev = gpmdev->ppdevGlobal;
//
//             /* Use the hatch bitmap as the mask */
//             hbmMask = (HBITMAP)ppdev->ahsurf[pebo->pbrush->iHatch];
//             psurfMask = SURFACE_ShareLockSurface(hbmMask);
//             if (psurfMask == NULL)
//             {
//                 ERR("Failed to lock hatch brush for PDEV %p, iHatch %lu\n",
//                     ppdev, pebo->pbrush->iHatch);
//                 return NULL;
//             }
//
//             NT_ASSERT(psurfMask->SurfObj.iBitmapFormat == BMF_1BPP);
//             pebo->psoMask = &psurfMask->SurfObj;
//         }
//     }
//
//     return pebo->psoMask;
// }
//
// /** Exported DDI functions ****************************************************/
//
// /*
//  * @implemented
//  */
// PVOID APIENTRY
// BRUSHOBJ_pvAllocRbrush(
//     IN BRUSHOBJ *pbo,
//     IN ULONG cj)
// {
//     pbo->pvRbrush = EngAllocMem(0, cj, GDITAG_RBRUSH);
//     return pbo->pvRbrush;
// }
//
// /*
//  * @implemented
//  */
// PVOID APIENTRY
// BRUSHOBJ_pvGetRbrush(
//     IN BRUSHOBJ *pbo)
// {
//     EBRUSHOBJ *pebo = CONTAINING_RECORD(pbo, EBRUSHOBJ, BrushObject);
//     BOOL bResult;
//
//     if (!pbo->pvRbrush)
//     {
//         bResult = EBRUSHOBJ_bRealizeBrush(pebo, TRUE);
//         if (!bResult)
//         {
//             if (pbo->pvRbrush)
//             {
//                 EngFreeMem(pbo->pvRbrush);
//                 pbo->pvRbrush = NULL;
//             }
//         }
//     }
//
//     return pbo->pvRbrush;
// }
//
// /*
//  * @implemented
//  */
// ULONG APIENTRY
// BRUSHOBJ_ulGetBrushColor(
//     IN BRUSHOBJ *pbo)
// {
//     EBRUSHOBJ *pebo = CONTAINING_RECORD(pbo, EBRUSHOBJ, BrushObject);
//     return pebo->ulRGBColor;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

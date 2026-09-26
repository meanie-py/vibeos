use win32k_sys::win32k::*;

// drop_gdiobj_macro_externs: dropped extern decls for GDI macros (SURFACE_*/PALETTE_*) —
// macros in win32ss/gdi/eng/surface.h and gdi/ntgdi/palette.h, not real fns.
// Per win32k_translation_prompt.md §6, inline-expand via GDIOBJ_* primitives.
//
// Macro expansions:
//   SURFACE_ShareLockSurface(h)   -> GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP) as PSURFACE
//   SURFACE_ShareUnlockSurface(p) -> GDIOBJ_vDereferenceObject(p as POBJ)
//   PALETTE_ShareLockPalette(h)   -> GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_PALETTE) as PPALETTE
//   PALETTE_ShareUnlockPalette(p) -> GDIOBJ_vDereferenceObject(&(*p).BaseObject)
//
// Plus other dropped macro externs:
//   GDIDEV(pso)              -> ((*pso).hdev) as PPDEVOBJ        (surface.h:105)
//   GDIDEVFUNCS(pso)         -> (*GDIDEV(pso)).DriverFunctions   (surface.h:106)
//   BitsPerFormat(f)         -> gajBitsPerFormat[f]              (surface.h:109)
//   WIDTH_BYTES_ALIGN32(cx,b)-> (((cx*b + 31) & !31) >> 3)       (surface.h:111)
//   ROP4_FROM_INDEX(i)       -> (i | (i << 8))                   (inteng.h:42)
//   RGB(r,g,b)               -> ((r as u32) | ((g as u32)<<8) | ((b as u32)<<16))  (wingdi.h)
//
// EXLATEOBJ_vInitialize / EXLATEOBJ_vCleanup are real NTAPI C functions (xlateobj.c),
// not in bindgen because they're win32k-internal. Declared with correct stdcall ABI on i386.
//
// gpalRGB, gexloTrivial, gppalMono, gpsi, gajBitsPerFormat are pulled from win32k_sys::win32k::*.

unsafe extern "system" {
    fn EXLATEOBJ_vInitialize(
        pexlo: *mut EXLATEOBJ,
        ppalSrc: PPALETTE,
        ppalDst: PPALETTE,
        crSrcBack: COLORREF,
        crDstBack: COLORREF,
        crDstFore: COLORREF,
    );
    fn EXLATEOBJ_vCleanup(pexlo: *mut EXLATEOBJ);
}

// ---- file-scope inline macro helpers (translated, not externed) ----
#[inline(always)]
unsafe fn gdidev(pso: *mut SURFOBJ) -> PPDEVOBJ {
    (*pso).hdev as PPDEVOBJ
}

#[inline(always)]
unsafe fn bits_per_format(format: u32) -> u32 {
    // gajBitsPerFormat is a flexible array; index it as a pointer.
    let base = core::ptr::addr_of!(gajBitsPerFormat) as *const u8;
    *base.add(format as usize) as u32
}

#[inline(always)]
fn width_bytes_align32(cx: i32, bpp: u32) -> i32 {
    (((cx as i64) * (bpp as i64) + 31) as i32 & !31) >> 3
}

#[inline(always)]
fn rop4_from_index(idx: u32) -> u32 {
    idx | (idx << 8)
}

#[inline(always)]
fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r & 0xff) | ((g & 0xff) << 8) | ((b & 0xff) << 16)
}

// ---- gpsi->ptCursor accessor ----
// tagSERVERINFO (bindgen) ends with __bindgen_padding_0: [u32; 647usize] which
// represents the anonymous PERUSERSERVERINFO member. We cast that tail to
// *mut _PERUSERSERVERINFO to access ptCursor.
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    let base = gpsi as *mut u8;
    base.add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn EngSetPointerTag(
    _hdev: HDEV,
    _psoMask: *mut SURFOBJ,
    _psoColor: *mut SURFOBJ,
    _pxlo: *mut XLATEOBJ,
    _fl: FLONG,
) -> BOOL {
    // Obsolete since Windows 2000 — always returns FALSE.
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn EngSetPointerTag(
    _hdev: HDEV,
    _psoMask: *mut SURFOBJ,
    _psoColor: *mut SURFOBJ,
    _pxlo: *mut XLATEOBJ,
    _fl: FLONG,
) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn MouseSafetyOnDrawStart(
    ppdev: PPDEVOBJ,
    HazardX1: LONG,
    HazardY1: LONG,
    HazardX2: LONG,
    HazardY2: LONG,
) -> BOOL {
    mouse_safety_on_draw_start_impl(ppdev, HazardX1, HazardY1, HazardX2, HazardY2)
}

unsafe fn mouse_safety_on_draw_start_impl(
    ppdev: PPDEVOBJ,
    HazardX1: LONG,
    HazardY1: LONG,
    HazardX2: LONG,
    HazardY2: LONG,
) -> BOOL {
    let mut HazardX1 = HazardX1;
    let mut HazardY1 = HazardY1;
    let mut HazardX2 = HazardX2;
    let mut HazardY2 = HazardY2;

    let pgp: *mut GDIPOINTER = core::ptr::addr_of_mut!((*ppdev).Pointer);

    if (*pgp).Exclude.right == -1 {
        return 0;
    }

    (*ppdev).SafetyRemoveCount += 1;

    if (*ppdev).SafetyRemoveLevel != 0 {
        return 0;
    }

    if HazardX1 > HazardX2 {
        let tmp = HazardX2;
        HazardX2 = HazardX1;
        HazardX1 = tmp;
    }
    if HazardY1 > HazardY2 {
        let tmp = HazardY2;
        HazardY2 = HazardY1;
        HazardY1 = tmp;
    }

    if (*pgp).Exclude.right >= HazardX1
        && (*pgp).Exclude.left <= HazardX2
        && (*pgp).Exclude.bottom >= HazardY1
        && (*pgp).Exclude.top <= HazardY2
    {
        (*ppdev).SafetyRemoveLevel = (*ppdev).SafetyRemoveCount;
        if (*ppdev).flFlags & PDEV_HARDWARE_POINTER != 0 {
            if let Some(pfn) = (*ppdev).pfnMovePointer {
                pfn(
                    core::ptr::addr_of_mut!((*(*ppdev).pSurface).SurfObj),
                    -1,
                    -1,
                    core::ptr::null_mut(),
                );
            }
        } else if (*ppdev).flFlags & PDEV_SOFTWARE_POINTER != 0 {
            EngMovePointer(
                core::ptr::addr_of_mut!((*(*ppdev).pSurface).SurfObj),
                -1,
                -1,
                core::ptr::null_mut(),
            );
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn MouseSafetyOnDrawEnd(ppdev: PPDEVOBJ) -> BOOL {
    mouse_safety_on_draw_end_impl(ppdev)
}

unsafe fn mouse_safety_on_draw_end_impl(ppdev: PPDEVOBJ) -> BOOL {
    let pgp: *mut GDIPOINTER = core::ptr::addr_of_mut!((*ppdev).Pointer);

    if (*pgp).Exclude.right == -1 {
        return 0;
    }

    (*ppdev).SafetyRemoveCount = (*ppdev).SafetyRemoveCount.wrapping_sub(1);
    if (*ppdev).SafetyRemoveCount >= (*ppdev).SafetyRemoveLevel {
        return 0;
    }

    let perinfo = gpsi_per_user();

    if ((*ppdev).flFlags & PDEV_HARDWARE_POINTER) != 0 {
        if let Some(pfn) = (*ppdev).pfnMovePointer {
            pfn(
                core::ptr::addr_of_mut!((*(*ppdev).pSurface).SurfObj),
                (*perinfo).ptCursor.x,
                (*perinfo).ptCursor.y,
                core::ptr::addr_of_mut!((*pgp).Exclude),
            );
        }
    } else if ((*ppdev).flFlags & PDEV_SOFTWARE_POINTER) != 0 {
        EngMovePointer(
            core::ptr::addr_of_mut!((*(*ppdev).pSurface).SurfObj),
            (*perinfo).ptCursor.x,
            (*perinfo).ptCursor.y,
            core::ptr::addr_of_mut!((*pgp).Exclude),
        );
    }

    (*ppdev).SafetyRemoveLevel = 0;

    1
}

unsafe fn IntHideMousePointer(ppdev: *mut PDEVOBJ, psoDest: *mut SURFOBJ) {
    let pgp: *mut GDIPOINTER = core::ptr::addr_of_mut!((*ppdev).Pointer);

    if (*pgp).Enabled == 0 {
        return;
    }

    (*pgp).Enabled = 0;

    if (*pgp).psurfSave.is_null() {
        return;
    }

    let mut pt: POINTL = core::mem::zeroed();
    pt.x = (*ppdev).ptlPointer.x - (*pgp).HotSpot.x;
    pt.y = (*ppdev).ptlPointer.y - (*pgp).HotSpot.y;

    let mut rclDest: RECTL = core::mem::zeroed();
    rclDest.left = core::cmp::max(pt.x, 0);
    rclDest.top = core::cmp::max(pt.y, 0);
    rclDest.right = core::cmp::min(pt.x + (*pgp).Size.cx, (*psoDest).sizlBitmap.cx);
    rclDest.bottom = core::cmp::min(pt.y + (*pgp).Size.cy, (*psoDest).sizlBitmap.cy);

    let mut ptlSave: POINTL = core::mem::zeroed();
    ptlSave.x = rclDest.left - pt.x;
    ptlSave.y = rclDest.top - pt.y;

    IntEngBitBlt(
        psoDest,
        core::ptr::addr_of_mut!((*(*pgp).psurfSave).SurfObj),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(rclDest),
        core::ptr::addr_of_mut!(ptlSave),
        core::ptr::addr_of_mut!(ptlSave),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        rop4_from_index(R3_OPINDEX_SRCCOPY as u32) as ROP4,
    );
}

unsafe fn IntShowMousePointer(ppdev: *mut PDEVOBJ, psoDest: *mut SURFOBJ) {
    let pgp: *mut GDIPOINTER = core::ptr::addr_of_mut!((*ppdev).Pointer);

    if (*pgp).Enabled != 0 {
        return;
    }

    (*pgp).Enabled = 1;

    if (*pgp).psurfSave.is_null() {
        return;
    }

    let mut pt: POINTL = POINTL { x: 0, y: 0 };
    pt.x = (*ppdev).ptlPointer.x - (*pgp).HotSpot.x;
    pt.y = (*ppdev).ptlPointer.y - (*pgp).HotSpot.y;

    let mut rclSurf: RECTL = RECTL { left: 0, top: 0, right: 0, bottom: 0 };
    rclSurf.left = core::cmp::max(pt.x, 0);
    rclSurf.top = core::cmp::max(pt.y, 0);
    rclSurf.right = core::cmp::min(pt.x + (*pgp).Size.cx, (*psoDest).sizlBitmap.cx);
    rclSurf.bottom = core::cmp::min(pt.y + (*pgp).Size.cy, (*psoDest).sizlBitmap.cy);

    let mut rclPointer: RECTL = RECTL { left: 0, top: 0, right: 0, bottom: 0 };
    rclPointer.left = rclSurf.left - pt.x;
    rclPointer.top = rclSurf.top - pt.y;
    rclPointer.right = core::cmp::min((*pgp).Size.cx, (*psoDest).sizlBitmap.cx - pt.x);
    rclPointer.bottom = core::cmp::min((*pgp).Size.cy, (*psoDest).sizlBitmap.cy - pt.y);

    IntEngBitBlt(
        core::ptr::addr_of_mut!((*(*pgp).psurfSave).SurfObj),
        psoDest,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(rclPointer),
        core::ptr::addr_of_mut!(rclSurf) as *mut POINTL,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        rop4_from_index(R3_OPINDEX_SRCCOPY as u32) as ROP4,
    );

    if !(*pgp).psurfColor.is_null() {
        if ((*pgp).flags & SPS_ALPHA) == 0 {
            IntEngBitBlt(
                psoDest,
                core::ptr::addr_of_mut!((*(*pgp).psurfMask).SurfObj),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(rclSurf),
                core::ptr::addr_of_mut!(rclPointer) as *mut POINTL,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                ROP4_SRCAND as ROP4,
            );

            IntEngBitBlt(
                psoDest,
                core::ptr::addr_of_mut!((*(*pgp).psurfColor).SurfObj),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(rclSurf),
                core::ptr::addr_of_mut!(rclPointer) as *mut POINTL,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                ROP4_SRCINVERT as ROP4,
            );
        } else {
            let mut blendobj: BLENDOBJ = BLENDOBJ {
                BlendFunction: BLENDFUNCTION {
                    BlendOp: AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: 255,
                    AlphaFormat: AC_SRC_ALPHA as u8,
                },
            };
            let mut exlo: EXLATEOBJ = core::mem::zeroed();
            EXLATEOBJ_vInitialize(
                core::ptr::addr_of_mut!(exlo),
                &raw mut gpalRGB as PPALETTE,
                (*ppdev).ppalSurf,
                0,
                0,
                0,
            );
            IntEngAlphaBlend(
                psoDest,
                core::ptr::addr_of_mut!((*(*pgp).psurfColor).SurfObj),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(exlo.xlo),
                core::ptr::addr_of_mut!(rclSurf),
                core::ptr::addr_of_mut!(rclPointer),
                core::ptr::addr_of_mut!(blendobj),
            );
            EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
        }
    } else {
        IntEngBitBlt(
            psoDest,
            core::ptr::addr_of_mut!((*(*pgp).psurfMask).SurfObj),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(rclSurf),
            core::ptr::addr_of_mut!(rclPointer) as *mut POINTL,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            rop4_from_index(R3_OPINDEX_SRCAND as u32) as ROP4,
        );

        rclPointer.top += (*pgp).Size.cy;

        IntEngBitBlt(
            psoDest,
            core::ptr::addr_of_mut!((*(*pgp).psurfMask).SurfObj),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(rclSurf),
            core::ptr::addr_of_mut!(rclPointer) as *mut POINTL,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            rop4_from_index(R3_OPINDEX_SRCINVERT as u32) as ROP4,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EngSetPointerShape(
    pso: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    psoColor: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    xHot: LONG,
    yHot: LONG,
    x: LONG,
    y: LONG,
    prcl: *mut RECTL,
    fl: FLONG,
) -> ULONG {
    eng_set_pointer_shape_impl(pso, psoMask, psoColor, pxlo, xHot, yHot, x, y, prcl, fl)
}

unsafe fn eng_set_pointer_shape_impl(
    pso: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    psoColor: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    xHot: LONG,
    yHot: LONG,
    x: LONG,
    y: LONG,
    prcl: *mut RECTL,
    fl: FLONG,
) -> ULONG {
    let mut pxlo = pxlo;
    let mut lDelta: LONG = 0;
    let mut hbmSave: HBITMAP = core::ptr::null_mut();
    let mut hbmColor: HBITMAP = core::ptr::null_mut();
    let mut hbmMask: HBITMAP = core::ptr::null_mut();
    let mut psurfSave: PSURFACE = core::ptr::null_mut();
    let mut psurfColor: PSURFACE = core::ptr::null_mut();
    let mut psurfMask: PSURFACE = core::ptr::null_mut();
    let mut rectl: RECTL = core::mem::zeroed();
    let mut sizel: SIZEL = core::mem::zeroed();

    let ppdev: *mut PDEVOBJ = gdidev(pso);
    let pgp: *mut GDIPOINTER = core::ptr::addr_of_mut!((*ppdev).Pointer);

    if pxlo.is_null() {
        pxlo = &raw mut gexloTrivial.xlo;
    }

    // Macro: failure cleanup — inline-expanded at every error path.
    // Equivalent of C `goto failure;`.
    macro_rules! goto_failure {
        () => {{
            if !hbmMask.is_null() { EngDeleteSurface(hbmMask as HSURF); }
            if !psurfMask.is_null() { GDIOBJ_vDereferenceObject(psurfMask as POBJ); }
            if !hbmColor.is_null() { EngDeleteSurface(hbmColor as HSURF); }
            if !psurfColor.is_null() { GDIOBJ_vDereferenceObject(psurfColor as POBJ); }
            if !hbmSave.is_null() { EngDeleteSurface(hbmSave as HSURF); }
            if !psurfSave.is_null() { GDIOBJ_vDereferenceObject(psurfSave as POBJ); }
            return SPS_ERROR;
        }};
    }

    if !psoColor.is_null() || !psoMask.is_null() {
        if !psoColor.is_null() {
            sizel.cx = (*psoColor).sizlBitmap.cx;
            sizel.cy = (*psoColor).sizlBitmap.cy;
        } else {
            sizel.cx = (*psoMask).sizlBitmap.cx;
            sizel.cy = (*psoMask).sizlBitmap.cy / 2;
        }

        rectl.left = 0;
        rectl.top = 0;
        rectl.right = sizel.cx;
        rectl.bottom = sizel.cy;

        lDelta = width_bytes_align32(sizel.cx, bits_per_format((*pso).iBitmapFormat));

        hbmSave = EngCreateBitmap(
            sizel,
            lDelta,
            (*pso).iBitmapFormat,
            (BMF_TOPDOWN | BMF_NOZEROINIT) as FLONG,
            core::ptr::null_mut(),
        );
        // SURFACE_ShareLockSurface(hbmSave)
        psurfSave = GDIOBJ_ShareLockObj(hbmSave as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE;
        if psurfSave.is_null() {
            goto_failure!();
        }
    }

    if !psoColor.is_null() {
        if (fl & SPS_ALPHA) != 0 {
            let mut exloSrcRGB: EXLATEOBJ = core::mem::zeroed();
            // pexlo = CONTAINING_RECORD(pxlo, EXLATEOBJ, xlo)
            let pexlo: *mut EXLATEOBJ =
                (pxlo as *mut u8).sub(core::mem::offset_of!(EXLATEOBJ, xlo)) as *mut EXLATEOBJ;
            EXLATEOBJ_vInitialize(
                core::ptr::addr_of_mut!(exloSrcRGB),
                (*pexlo).ppalSrc,
                &raw mut gpalRGB as PPALETTE,
                0,
                0,
                0,
            );

            hbmColor = EngCreateBitmap(
                (*psoColor).sizlBitmap,
                width_bytes_align32(sizel.cx, 32),
                BMF_32BPP as ULONG,
                (BMF_TOPDOWN | BMF_NOZEROINIT) as FLONG,
                core::ptr::null_mut(),
            );
            psurfColor = GDIOBJ_ShareLockObj(hbmColor as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE;
            if psurfColor.is_null() {
                EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exloSrcRGB));
                goto_failure!();
            }

            rectl.bottom = (*psoColor).sizlBitmap.cy;
            IntEngCopyBits(
                core::ptr::addr_of_mut!((*psurfColor).SurfObj),
                psoColor,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(exloSrcRGB.xlo),
                core::ptr::addr_of_mut!(rectl),
                core::ptr::addr_of_mut!(rectl) as *mut POINTL,
            );

            EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exloSrcRGB));
        } else {
            if (*psoColor).iBitmapFormat != (*pso).iBitmapFormat {
                goto_failure!();
            }

            hbmColor = EngCreateBitmap(
                (*psoColor).sizlBitmap,
                lDelta,
                (*pso).iBitmapFormat,
                (BMF_TOPDOWN | BMF_NOZEROINIT) as FLONG,
                core::ptr::null_mut(),
            );
            psurfColor = GDIOBJ_ShareLockObj(hbmColor as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE;
            if psurfColor.is_null() {
                goto_failure!();
            }

            rectl.bottom = (*psoColor).sizlBitmap.cy;
            IntEngCopyBits(
                core::ptr::addr_of_mut!((*psurfColor).SurfObj),
                psoColor,
                core::ptr::null_mut(),
                pxlo,
                core::ptr::addr_of_mut!(rectl),
                core::ptr::addr_of_mut!(rectl) as *mut POINTL,
            );
        }
    }

    if !psoMask.is_null() {
        let mut exlo: EXLATEOBJ = core::mem::zeroed();

        lDelta = width_bytes_align32(sizel.cx, bits_per_format((*pso).iBitmapFormat));

        hbmMask = EngCreateBitmap(
            (*psoMask).sizlBitmap,
            lDelta,
            (*pso).iBitmapFormat,
            (BMF_TOPDOWN | BMF_NOZEROINIT) as FLONG,
            core::ptr::null_mut(),
        );
        psurfMask = GDIOBJ_ShareLockObj(hbmMask as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE;
        if psurfMask.is_null() {
            goto_failure!();
        }

        // PALETTE_ShareLockPalette(ppdev->devinfo.hpalDefault)
        let ppal: PPALETTE = GDIOBJ_ShareLockObj(
            (*ppdev).devinfo.hpalDefault as HGDIOBJ,
            GDI_OBJECT_TYPE_PALETTE as DWORD,
        ) as PPALETTE;
        EXLATEOBJ_vInitialize(
            core::ptr::addr_of_mut!(exlo),
            gppalMono,
            ppal,
            0,
            rgb(0xff, 0xff, 0xff),
            rgb(0, 0, 0),
        );

        rectl.bottom = (*psoMask).sizlBitmap.cy;
        IntEngCopyBits(
            core::ptr::addr_of_mut!((*psurfMask).SurfObj),
            psoMask,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(exlo.xlo),
            core::ptr::addr_of_mut!(rectl),
            core::ptr::addr_of_mut!(rectl) as *mut POINTL,
        );

        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
        if !ppal.is_null() {
            // PALETTE_ShareUnlockPalette(ppal) -> GDIOBJ_vDereferenceObject(&ppal->BaseObject)
            GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*ppal).BaseObject) as POBJ);
        }
    }

    IntHideMousePointer(ppdev, pso);

    if !(*pgp).psurfColor.is_null() {
        EngDeleteSurface((*(*pgp).psurfColor).BaseObject.hHmgr as HSURF);
        GDIOBJ_vDereferenceObject((*pgp).psurfColor as POBJ);
        (*pgp).psurfColor = core::ptr::null_mut();
    }

    if !(*pgp).psurfMask.is_null() {
        EngDeleteSurface((*(*pgp).psurfMask).BaseObject.hHmgr as HSURF);
        GDIOBJ_vDereferenceObject((*pgp).psurfMask as POBJ);
        (*pgp).psurfMask = core::ptr::null_mut();
    }

    if !(*pgp).psurfSave.is_null() {
        EngDeleteSurface((*(*pgp).psurfSave).BaseObject.hHmgr as HSURF);
        GDIOBJ_vDereferenceObject((*pgp).psurfSave as POBJ);
        (*pgp).psurfSave = core::ptr::null_mut();
    }

    if psoMask.is_null() && psoColor.is_null() {
        return SPS_ACCEPT_NOEXCLUDE;
    }

    (*pgp).psurfColor = psurfColor;
    (*pgp).psurfMask = psurfMask;
    (*pgp).psurfSave = psurfSave;
    (*pgp).HotSpot.x = xHot;
    (*pgp).HotSpot.y = yHot;
    (*pgp).Size = sizel;
    (*pgp).flags = fl;

    if x != -1 {
        (*ppdev).ptlPointer.x = x;
        (*ppdev).ptlPointer.y = y;

        IntShowMousePointer(ppdev, pso);

        if !prcl.is_null() {
            (*prcl).left = x - (*pgp).HotSpot.x;
            (*prcl).top = y - (*pgp).HotSpot.y;
            (*prcl).right = (*prcl).left + (*pgp).Size.cx;
            (*prcl).bottom = (*prcl).top + (*pgp).Size.cy;
        }
    } else if !prcl.is_null() {
        (*prcl).left = -1;
        (*prcl).top = -1;
        (*prcl).right = -1;
        (*prcl).bottom = -1;
    }

    SPS_ACCEPT_NOEXCLUDE
}

#[no_mangle]
pub unsafe extern "system" fn EngMovePointer(
    pso: *mut SURFOBJ,
    x: LONG,
    y: LONG,
    prcl: *mut RECTL,
) {
    eng_move_pointer_impl(pso, x, y, prcl)
}

unsafe fn eng_move_pointer_impl(pso: *mut SURFOBJ, x: LONG, y: LONG, prcl: *mut RECTL) {
    let ppdev: *mut PDEVOBJ = gdidev(pso);
    let pgp: *mut GDIPOINTER = core::ptr::addr_of_mut!((*ppdev).Pointer);

    IntHideMousePointer(ppdev, pso);

    (*ppdev).ptlPointer.x = x;
    (*ppdev).ptlPointer.y = y;

    if x != -1 {
        IntShowMousePointer(ppdev, pso);
        if !prcl.is_null() {
            (*prcl).left = x - (*pgp).HotSpot.x;
            (*prcl).top = y - (*pgp).HotSpot.y;
            (*prcl).right = (*prcl).left + (*pgp).Size.cx;
            (*prcl).bottom = (*prcl).top + (*pgp).Size.cy;
        }
    } else if !prcl.is_null() {
        (*prcl).left = -1;
        (*prcl).top = -1;
        (*prcl).right = -1;
        (*prcl).bottom = -1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IntEngSetPointerShape(
    pso: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    psoColor: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    xHot: LONG,
    yHot: LONG,
    x: LONG,
    y: LONG,
    prcl: *mut RECTL,
    fl: FLONG,
) -> ULONG {
    int_eng_set_pointer_shape_impl(pso, psoMask, psoColor, pxlo, xHot, yHot, x, y, prcl, fl)
}

unsafe fn int_eng_set_pointer_shape_impl(
    pso: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    psoColor: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    xHot: LONG,
    yHot: LONG,
    x: LONG,
    y: LONG,
    prcl: *mut RECTL,
    fl: FLONG,
) -> ULONG {
    let mut ulResult: ULONG = SPS_DECLINE;
    let ppdev: PPDEVOBJ = gdidev(pso);
    let mut bHardwarePointer: BOOL = 0;
    let mut bSoftwarePointer: BOOL = 1;

    // GDIDEVFUNCS(pso).SetPointerShape — DriverFunctions is a struct, not a fn ptr.
    let pfnSetPointerShape: PFN_DrvSetPointerShape = (*ppdev).__bindgen_anon_1.DriverFunctions.SetPointerShape;

    if pfnSetPointerShape.is_some() {
        let mut pxlo_local = pxlo;
        if pxlo_local.is_null() {
            pxlo_local = &raw mut gexloTrivial.xlo;
        }

        ulResult = pfnSetPointerShape.unwrap()(
            pso, psoMask, psoColor, pxlo_local, xHot, yHot, x, y, prcl, fl,
        );

        if ulResult == SPS_ACCEPT_NOEXCLUDE {
            bHardwarePointer = 1;
        }

        bSoftwarePointer = if bHardwarePointer != 0 { 0 } else { 1 };
    }

    if bSoftwarePointer != 0 {
        ulResult = EngSetPointerShape(pso, psoMask, psoColor, pxlo, xHot, yHot, x, y, prcl, fl);
    }

    if bSoftwarePointer == 0 && ((*ppdev).flFlags & PDEV_SOFTWARE_POINTER) != 0 {
        EngMovePointer(pso, -1, -1, core::ptr::null_mut());
    }

    if bHardwarePointer == 0 && ((*ppdev).flFlags & PDEV_HARDWARE_POINTER) != 0 {
        if let Some(pfn) = (*ppdev).pfnMovePointer {
            pfn(pso, -1, -1, core::ptr::null_mut());
        }
    }

    if bSoftwarePointer != 0 {
        (*ppdev).flFlags |= PDEV_SOFTWARE_POINTER as FLONG;
    } else {
        (*ppdev).flFlags &= !(PDEV_SOFTWARE_POINTER as FLONG);
    }

    if bHardwarePointer != 0 {
        (*ppdev).flFlags |= PDEV_HARDWARE_POINTER as FLONG;
    } else {
        (*ppdev).flFlags &= !(PDEV_HARDWARE_POINTER as FLONG);
    }

    ulResult
}

#[no_mangle]
pub unsafe extern "system" fn GreSetPointerShape(
    hdc: HDC,
    hbmMask: HBITMAP,
    hbmColor: HBITMAP,
    xHot: LONG,
    yHot: LONG,
    x: LONG,
    y: LONG,
    fl: FLONG,
) -> ULONG {
    gre_set_pointer_shape_impl(hdc, hbmMask, hbmColor, xHot, yHot, x, y, fl)
}

unsafe fn gre_set_pointer_shape_impl(
    hdc: HDC,
    hbmMask: HBITMAP,
    hbmColor: HBITMAP,
    xHot: LONG,
    yHot: LONG,
    x: LONG,
    y: LONG,
    fl: FLONG,
) -> ULONG {
    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    EngAcquireSemaphore((*(*pdc).ppdev).hsemDevLock);
    let psurf: PSURFACE = (*(*pdc).ppdev).pSurface;
    if psurf.is_null() {
        EngReleaseSemaphore((*(*pdc).ppdev).hsemDevLock);
        DC_UnlockDc(pdc);
        return 0;
    }

    // SURFACE_ShareLockSurface(hbmMask)
    let psurfMask: PSURFACE = if !hbmMask.is_null() {
        GDIOBJ_ShareLockObj(hbmMask as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE
    } else {
        core::ptr::null_mut()
    };

    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    // SURFACE_ShareLockSurface(hbmColor) + EXLATEOBJ init if non-null
    let psurfColor: PSURFACE = if !hbmColor.is_null() {
        let psc: PSURFACE =
            GDIOBJ_ShareLockObj(hbmColor as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE;
        if !psc.is_null() {
            EXLATEOBJ_vInitialize(
                core::ptr::addr_of_mut!(exlo),
                (*psc).ppal,
                (*psurf).ppal,
                0,
                0,
                0,
            );
        }
        psc
    } else {
        core::ptr::null_mut()
    };

    let ulResult: ULONG = IntEngSetPointerShape(
        core::ptr::addr_of_mut!((*psurf).SurfObj),
        if !psurfMask.is_null() {
            core::ptr::addr_of_mut!((*psurfMask).SurfObj)
        } else {
            core::ptr::null_mut()
        },
        if !psurfColor.is_null() {
            core::ptr::addr_of_mut!((*psurfColor).SurfObj)
        } else {
            core::ptr::null_mut()
        },
        if !psurfColor.is_null() {
            core::ptr::addr_of_mut!(exlo.xlo)
        } else {
            core::ptr::null_mut()
        },
        xHot,
        yHot,
        x,
        y,
        core::ptr::addr_of_mut!((*(*pdc).ppdev).Pointer.Exclude),
        fl | SPS_CHANGE,
    );

    if !psurfColor.is_null() {
        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
        // SURFACE_ShareUnlockSurface(psurfColor)
        GDIOBJ_vDereferenceObject(psurfColor as POBJ);
    }

    if !psurfMask.is_null() {
        GDIOBJ_vDereferenceObject(psurfMask as POBJ);
    }

    EngReleaseSemaphore((*(*pdc).ppdev).hsemDevLock);
    DC_UnlockDc(pdc);

    ulResult
}

#[no_mangle]
pub unsafe extern "system" fn GreMovePointer(hdc: HDC, x: LONG, y: LONG) {
    gre_move_pointer_impl(hdc, x, y)
}

unsafe fn gre_move_pointer_impl(hdc: HDC, x: LONG, y: LONG) {
    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        return;
    }

    EngAcquireSemaphore((*(*pdc).ppdev).hsemDevLock);

    if (*(*pdc).ppdev).SafetyRemoveLevel == 0 {
        let pso: *mut SURFOBJ = core::ptr::addr_of_mut!((*(*(*pdc).ppdev).pSurface).SurfObj);

        let prcl: PRECTL = core::ptr::addr_of_mut!((*(*pdc).ppdev).Pointer.Exclude);

        if (*(*pdc).ppdev).flFlags & PDEV_HARDWARE_POINTER != 0 {
            if let Some(pfn) = (*(*pdc).ppdev).pfnMovePointer {
                pfn(pso, x, y, prcl);
            }
        } else if (*(*pdc).ppdev).flFlags & PDEV_SOFTWARE_POINTER != 0 {
            EngMovePointer(pso, x, y, prcl);
        }

        if (*(*pdc).ppdev).devinfo.flGraphicsCaps & GCAPS_PANNING != 0 && y >= 0 {
            if let Some(pfn) = (*(*pdc).ppdev).pfnMovePointer {
                pfn(pso, x, y - (*pso).sizlBitmap.cy, core::ptr::null_mut());
            }
        }
    }

    EngReleaseSemaphore((*(*pdc).ppdev).hsemDevLock);

    DC_UnlockDc(pdc);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS win32 subsystem
//  * PURPOSE:          Mouse pointer functions
//  * FILE:             win32ss/gdi/eng/mouse.c
//  * PROGRAMER:        Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *                   Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
// /* INCLUDES ******************************************************************/
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// __drv_preferredFunction("(see documentation)", "Obsolete, always returns false. ")
// BOOL
// APIENTRY
// EngSetPointerTag(
//     _In_ HDEV hdev,
//     _In_opt_ SURFOBJ *psoMask,
//     _In_opt_ SURFOBJ *psoColor,
//     _Reserved_ XLATEOBJ *pxlo,
//     _In_ FLONG fl)
// {
//     return FALSE;
// }
//
// /* ... see mouse.c for full original C source ... */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

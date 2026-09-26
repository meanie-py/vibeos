//! ORPHAN-BY-DESIGN: `bitblt_new.c` sits under `if(USE_DIBLIB)` in
//! win32ss/CMakeLists.txt, and that file sets `USE_DIBLIB FALSE` — so neither the
//! C nor this translation is built. The live blt path is bitblt.rs (EngBitBlt,
//! IntEngBitBlt, NtGdiEngBitBlt) + copybits.rs (EngCopyBits, IntEngCopyBits);
//! wiring this file up would collide with all five.
//!
use win32k_sys::win32k::*;
use core::ffi::c_void;

extern "system" {
    fn BRUSHOBJ_psoMask(pbo: *mut BRUSHOBJ) -> *mut SURFOBJ;
    fn BRUSHOBJ_psoPattern(pbo: *mut BRUSHOBJ) -> *mut SURFOBJ;
    fn CLIPOBJ_bEnum(pco: *mut CLIPOBJ, cj: ULONG, pv: *mut ULONG) -> u8;
    fn CLIPOBJ_cEnumStart(pco: *mut CLIPOBJ, bAll: i32, iType: u32, iDirection: u32, cMaxRects: u32,) -> u32;
    fn NT_ASSERT(expr: BOOL);
    fn ROP4_BKGND(rop4: ROP4) -> ULONG;
    fn ROP4_FGND(rop4: ROP4) -> ULONG;
    fn ROP4_USES_DEST(rop4: ROP4) -> BOOL;
    fn ROP4_USES_MASK(rop4: ROP4) -> BOOL;
    fn ROP4_USES_PATTERN(rop4: ROP4) -> BOOL;
    fn ROP4_USES_SOURCE(rop4: ROP4) -> BOOL;
    fn __debugbreak();
    fn IS_VALID_ROP4(rop4: ROP4) -> u8;
    fn RECTL_bClipRectBySize(rcOut: *mut RECTL, rcIn: *const RECTL, size: *const SIZEL) -> u8;
    fn SURFOBJ_flags(pso: *const SURFOBJ) -> u32;
    fn GDIDEVFUNCS(pso: *const SURFOBJ) -> *const core::ffi::c_void;
    fn ERR(msg: *const u8);
    fn SafeProbeForRead(Address: *const core::ffi::c_void, Length: usize, Alignment: u32) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub static mut gxcoTrivial: XCLIPOBJ = unsafe { core::mem::zeroed() };

unsafe fn CalculateCoordinates(
    pbltdata: *mut BLTDATA,
    prclClipped: PRECTL,
    prclOrg: PRECTL,
    pptlSrc: PPOINTL,
    pptlMask: PPOINTL,
    pptlPat: PPOINTL,
    psizlPat: PSIZEL,
)
{
    let cx: ULONG;
    let cy: ULONG;

    (*pbltdata).ulWidth = ((*prclClipped).right - (*prclClipped).left) as ULONG;
    (*pbltdata).ulHeight = ((*prclClipped).bottom - (*prclClipped).top) as ULONG;

    if (*pbltdata).siDst.iFormat == 0 {
        cx = ((*prclClipped).right - 1 - (*prclOrg).left) as ULONG;
    } else {
        cx = ((*prclClipped).left - (*prclOrg).left) as ULONG;
    }

    if (*pbltdata).dy < 0 {
        cy = ((*prclClipped).bottom - 1 - (*prclOrg).top) as ULONG;
    } else {
        cy = ((*prclClipped).top - (*prclOrg).top) as ULONG;
    }

    (*pbltdata).siDst.ptOrig.x = (*prclOrg).left + cx as LONG;
    (*pbltdata).siDst.ptOrig.y = (*prclOrg).top + cy as LONG;

    (*pbltdata).siDst.pjBase = (*pbltdata).siDst.pvScan0;
    (*pbltdata).siDst.pjBase = (*pbltdata).siDst.pjBase.offset(
        ((*pbltdata).siDst.ptOrig.y as isize) * (*pbltdata).siDst.lDelta as isize
    );
    (*pbltdata).siDst.pjBase = (*pbltdata).siDst.pjBase.offset(
        ((*pbltdata).siDst.ptOrig.x as isize) * ((*pbltdata).siDst.jBpp as isize) / 8
    );

    if !pptlSrc.is_null() {
        (*pbltdata).siSrc.ptOrig.x = (*pptlSrc).x + cx as LONG;
        (*pbltdata).siSrc.ptOrig.y = (*pptlSrc).y + cy as LONG;
        (*pbltdata).siSrc.pjBase = (*pbltdata).siSrc.pvScan0;
        (*pbltdata).siSrc.pjBase = (*pbltdata).siSrc.pjBase.offset(
            ((*pbltdata).siSrc.ptOrig.y as isize) * (*pbltdata).siSrc.lDelta as isize
        );
        (*pbltdata).siSrc.pjBase = (*pbltdata).siSrc.pjBase.offset(
            ((*pbltdata).siSrc.ptOrig.x as isize) * ((*pbltdata).siSrc.jBpp as isize) / 8
        );
    }

    if !pptlMask.is_null() {
        (*pbltdata).siMsk.ptOrig.x = (*pptlMask).x + cx as LONG;
        (*pbltdata).siMsk.ptOrig.y = (*pptlMask).y + cy as LONG;
        (*pbltdata).siMsk.pjBase = (*pbltdata).siMsk.pvScan0;
        (*pbltdata).siMsk.pjBase = (*pbltdata).siMsk.pjBase.offset(
            ((*pbltdata).siMsk.ptOrig.y as isize) * (*pbltdata).siMsk.lDelta as isize
        );
        (*pbltdata).siMsk.pjBase = (*pbltdata).siMsk.pjBase.offset(
            ((*pbltdata).siMsk.ptOrig.x as isize) * ((*pbltdata).siMsk.jBpp as isize) / 8
        );
    }

    if !pptlPat.is_null() {
        (*pbltdata).siPat.ptOrig.x = ((*pptlPat).x + cx as LONG) % (*psizlPat).cx;
        (*pbltdata).siPat.ptOrig.y = ((*pptlPat).y + cy as LONG) % (*psizlPat).cy;
        (*pbltdata).siPat.pjBase = (*pbltdata).siPat.pvScan0;

        if (*pbltdata).dy < 0 {
            (*pbltdata).siPat.pjBase = (*pbltdata).siPat.pjBase.offset(
                (((*psizlPat).cy - 1) as isize) * (*pbltdata).siPat.lDelta as isize
            );
            (*pbltdata).siPat.ptOrig.y = (*psizlPat).cy - 1 - (*pbltdata).siPat.ptOrig.y;
        }

        if (*pbltdata).siDst.iFormat == 0 {
            (*pbltdata).siPat.pjBase = (*pbltdata).siPat.pjBase.offset(
                (((*psizlPat).cx - 1) as isize) * ((*pbltdata).siMsk.jBpp as isize) / 8
            );
            (*pbltdata).siPat.ptOrig.x = (*psizlPat).cx - 1 - (*pbltdata).siPat.ptOrig.x;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn EngBitBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    rop4: ROP4,
) -> BOOL {

    let mut bltdata: BLTDATA = core::mem::zeroed();
    let mut i: ULONG;
    let iFunctionIndex: ULONG;
    let mut iDirection: ULONG = CD_ANY;
    let rcTrg: RECTL;
    let pfnBitBlt: PFN_DIBFUNCTION;
    let mut bEnumMore: BOOL;
    let mut rcenum: RECT_ENUM = core::mem::zeroed();
    let mut psizlPat: PSIZEL;
    let mut psoPattern: *mut SURFOBJ;
    let mut pptlBrush_local = pptlBrush;
    let mut psoMask_local = psoMask;
    let mut pxlo_local = pxlo;
    let mut rop4_local = rop4;

    rcTrg = *prclTrg;

    bltdata.dy = 1;
    bltdata.rop4 = rop4_local;
    bltdata.apfnDoRop[0] = gapfnRop[ROP4_BKGND(rop4_local) as usize];
    bltdata.apfnDoRop[1] = gapfnRop[ROP4_FGND(rop4_local) as usize];
    if pxlo_local.is_null() {
        pxlo_local = core::ptr::addr_of_mut!(gexloTrivial.xlo);
    }
    bltdata.pxlo = pxlo_local;
    bltdata.pfnXlate = XLATEOBJ_pfnXlate(pxlo_local);

    if ROP4_USES_SOURCE(rop4_local) != 0 {
        if psoSrc == psoTrg {
            if rcTrg.top > (*pptlSrc).y {
                iDirection = if rcTrg.left < (*pptlSrc).x { CD_RIGHTUP } else { CD_LEFTUP };
                bltdata.dy = -1;
            } else {
                iDirection = if rcTrg.left < (*pptlSrc).x { CD_RIGHTDOWN } else { CD_LEFTDOWN };
            }

            if (rcTrg.top == (*pptlSrc).y) && (rcTrg.left > (*pptlSrc).x) {
                bltdata.siDst.iFormat = 0;
                bltdata.siSrc.iFormat = (*psoSrc).iBitmapFormat;
            } else {
                bltdata.siDst.iFormat = (*psoTrg).iBitmapFormat;
                bltdata.siSrc.iFormat = 0;
            }
        } else {
            bltdata.siDst.iFormat = (*psoTrg).iBitmapFormat;
            bltdata.siSrc.iFormat = (*psoSrc).iBitmapFormat;
        }

        bltdata.siSrc.pvScan0 = (*psoSrc).pvScan0;
        bltdata.siSrc.lDelta = (*psoSrc).lDelta;
        bltdata.siSrc.cjAdvanceY = bltdata.dy * (*psoSrc).lDelta;
        bltdata.siSrc.jBpp = gajBitsPerFormat[(*psoSrc).iBitmapFormat as usize];
    } else {
        bltdata.siDst.iFormat = (*psoTrg).iBitmapFormat;
    }

    bltdata.siDst.pvScan0 = (*psoTrg).pvScan0;
    bltdata.siDst.lDelta = (*psoTrg).lDelta;
    bltdata.siDst.cjAdvanceY = bltdata.dy * (*psoTrg).lDelta;
    bltdata.siDst.jBpp = gajBitsPerFormat[(*psoTrg).iBitmapFormat as usize];

    if ROP4_USES_PATTERN(rop4_local) != 0 {
        NT_ASSERT(!pbo.is_null());

        bltdata.ulSolidColor = (*pbo).iSolidColor;

        if (*pbo).iSolidColor == 0xFFFFFFFF {
            psoPattern = BRUSHOBJ_psoPattern(pbo);
            if psoPattern.is_null() {
                __debugbreak();
                return 0;
            }

            bltdata.siPat.iFormat = (*psoPattern).iBitmapFormat;
            bltdata.siPat.pvScan0 = (*psoPattern).pvScan0;
            bltdata.siPat.lDelta = (*psoPattern).lDelta;
            bltdata.siPat.cjAdvanceY = bltdata.dy * (*psoPattern).lDelta;
            bltdata.siPat.jBpp = gajBitsPerFormat[(*psoPattern).iBitmapFormat as usize];

            bltdata.ulPatWidth = (*psoPattern).sizlBitmap.cx as ULONG;
            bltdata.ulPatHeight = (*psoPattern).sizlBitmap.cy as ULONG;

            psizlPat = core::ptr::addr_of_mut!((*psoPattern).sizlBitmap);
        } else {
            pptlBrush_local = core::ptr::null_mut();
            psizlPat = core::ptr::null_mut();
        }
    } else {
        pptlBrush_local = core::ptr::null_mut();
        psizlPat = core::ptr::null_mut();
    }

    if ROP4_USES_MASK(rop4_local) != 0 {
        if psoMask_local.is_null() {
            NT_ASSERT(!pbo.is_null());

            psoMask_local = BRUSHOBJ_psoMask(pbo);
            if psoMask_local.is_null() {
                rop4_local = (rop4_local & 0xFF) || ((rop4_local & 0xFF) << 8);
            }
        }

        bltdata.siMsk.iFormat = (*psoMask_local).iBitmapFormat;
        bltdata.siMsk.pvScan0 = (*psoMask_local).pvScan0;
        bltdata.siMsk.lDelta = (*psoMask_local).lDelta;
        bltdata.siMsk.cjAdvanceY = bltdata.dy * (*psoMask_local).lDelta;
        bltdata.siMsk.jBpp = 1;

        iFunctionIndex = if ROP4_USES_PATTERN(rop4_local) != 0 { 1 } else { 0 }
            | if ROP4_USES_SOURCE(rop4_local) != 0 { 2 } else { 0 }
            | if ROP4_USES_DEST(rop4_local) != 0 { 4 } else { 0 };

        pfnBitBlt = gapfnMaskFunction[iFunctionIndex as usize];
    } else {
        iFunctionIndex = gajIndexPerRop[ROP4_FGND(rop4_local) as usize] as ULONG;

        pfnBitBlt = gapfnDibFunction[iFunctionIndex as usize];
    }

    let pco_local = if pco.is_null() {
        core::ptr::addr_of!(gxcoTrivial) as *mut CLIPOBJ
    } else {
        pco
    };

    if (*pco_local).iDComplexity == DC_COMPLEX {
        CLIPOBJ_cEnumStart(pco_local, 0, CT_RECTANGLES, iDirection, 0);
        bEnumMore = CLIPOBJ_bEnum(pco_local, core::mem::size_of::<RECT_ENUM>() as ULONG, core::ptr::addr_of_mut!(rcenum) as *mut ULONG);
    } else if (*pco_local).iDComplexity == DC_RECT {
        rcenum.arcl[0] = (*pco_local).rclBounds;
        rcenum.c = 1;
        bEnumMore = 0;
    } else {
        rcenum.arcl[0] = rcTrg;
        rcenum.c = 1;
        bEnumMore = 0;
    }

    loop {
        i = 0;
        while i < rcenum.c {
            if RECTL_bEnum(core::ptr::addr_of_mut!(rcenum.arcl[i as usize]), core::ptr::addr_of!(rcenum.arcl[i as usize]), core::ptr::addr_of!(rcTrg)) == 0 {
                i += 1;
                continue;
            }

            CalculateCoordinates(
                core::ptr::addr_of_mut!(bltdata),
                core::ptr::addr_of_mut!(rcenum.arcl[i as usize]),
                prclTrg,
                pptlSrc,
                pptlMask,
                pptlBrush_local,
                psizlPat,
            );

            pfnBitBlt.unwrap()(core::ptr::addr_of_mut!(bltdata));

            i += 1;
        }

        if bEnumMore == 0 {
            break;
        }

        bEnumMore = CLIPOBJ_bEnum(pco_local, core::mem::size_of::<RECT_ENUM>() as ULONG, core::ptr::addr_of_mut!(rcenum) as *mut ULONG);
    }

    1
}

unsafe fn AdjustOffsetAndSize(
    pptOffset: PPOINTL,
    psizTrg: PSIZEL,
    pptlSrc: PPOINTL,
    psizSrc: PSIZEL)
{
    let mut x: i32;
    let mut y: i32;
    let mut cxMax: i32;
    let mut cyMax: i32;

    x = (*pptlSrc).x + (*pptOffset).x;
    if x < 0 {
        (*pptOffset).x -= x;
        x = 0;
    }

    cxMax = (*psizSrc).cx - x;
    if (*psizTrg).cx > cxMax {
        (*psizTrg).cx = cxMax;
    }

    y = (*pptlSrc).y + (*pptOffset).y;
    if y < 0 {
        (*pptOffset).y -= y;
        y = 0;
    }

    cyMax = (*psizSrc).cy - y;
    if (*psizTrg).cy > cyMax {
        (*psizTrg).cy = cyMax;
    }
}
#[no_mangle]
pub unsafe extern "system" fn IntEngBitBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    rop4: ROP4,
) -> u8 {
    let mut bResult: u8;
    let mut rcClipped: RECTL = core::mem::zeroed();
    let mut ptOffset: POINTL;
    let mut ptSrc: POINTL;
    let mut ptMask: POINTL;
    let mut ptBrush: POINTL = core::mem::zeroed();
    let mut sizTrg: SIZEL;
    let mut pfnBitBlt: PFN_DrvBitBlt;
    let mut pco_mut = pco;

    if RECTL_bClipRectBySize(&mut rcClipped, prclTrg, &(*psoTrg).sizlBitmap) == 0 {
        return 1u8;
    }

    if pco_mut.is_null() {
        pco_mut = &gxcoTrivial as *const CLIPOBJ as *mut CLIPOBJ;
    }

    if (*pco_mut).iDComplexity != DC_TRIVIAL {
        if RECTL_bIntersectRect(&mut rcClipped, &rcClipped, &(*pco_mut).rclBounds) == 0 {
            return 1u8;
        }
    }

    if (*pco_mut).iDComplexity == DC_RECT {
        pco_mut = &gxcoTrivial as *const CLIPOBJ as *mut CLIPOBJ;
    }

    ptOffset.x = rcClipped.left - (*prclTrg).left;
    ptOffset.y = rcClipped.top - (*prclTrg).top;
    sizTrg.cx = rcClipped.right - rcClipped.left;
    sizTrg.cy = rcClipped.bottom - rcClipped.top;

    let mut psoSrc_mut = psoSrc;
    if ROP4_USES_SOURCE(rop4) != 0 {
        ptSrc = *pptlSrc;
        AdjustOffsetAndSize(&mut ptOffset, &mut sizTrg, &mut ptSrc, &(*psoSrc).sizlBitmap);
    } else {
        psoSrc_mut = core::ptr::null_mut();
        ptSrc.x = 0;
        ptSrc.y = 0;
    }

    let mut psoMask_mut = psoMask;
    if ROP4_USES_MASK(rop4) != 0 {
        ptMask = *pptlMask;
        AdjustOffsetAndSize(&mut ptOffset, &mut sizTrg, &mut ptMask, &(*psoMask).sizlBitmap);
    } else {
        psoMask_mut = core::ptr::null_mut();
        ptMask.x = 0;
        ptMask.y = 0;
    }

    if sizTrg.cx <= 0 || sizTrg.cy <= 0 {
        return 1u8;
    }

    ptSrc.x += ptOffset.x;
    ptSrc.y += ptOffset.y;
    ptMask.x += ptOffset.x;
    ptMask.y += ptOffset.y;

    if !pptlBrush.is_null() {
        ptBrush.x = (*pptlBrush).x + ptOffset.x;
        ptBrush.y = (*pptlBrush).y + ptOffset.y;
    }

    rcClipped.left = (*prclTrg).left + ptOffset.x;
    rcClipped.top = (*prclTrg).top + ptOffset.y;
    rcClipped.right = rcClipped.left + sizTrg.cx;
    rcClipped.bottom = rcClipped.top + sizTrg.cy;

    if (SURFOBJ_flags(psoTrg) & HOOK_BITBLT) != 0 {
        if !psoSrc_mut.is_null()
            && (*psoSrc_mut).hdev != (*psoTrg).hdev
            && (SURFOBJ_flags(psoSrc_mut) & HOOK_BITBLT) != 0
        {
            ERR(b"Need to copy to standard bitmap format!\n\0".as_ptr());
        }

        let gdi_funcs = GDIDEVFUNCS(psoTrg) as *const *const PFN_DrvBitBlt;
        pfnBitBlt = *gdi_funcs;
    } else if !psoSrc_mut.is_null() && (SURFOBJ_flags(psoSrc_mut) & HOOK_BITBLT) != 0 {
        let gdi_funcs = GDIDEVFUNCS(psoSrc_mut) as *const *const PFN_DrvBitBlt;
        pfnBitBlt = *gdi_funcs;
    } else {
        pfnBitBlt = EngBitBlt;
    }

    bResult = pfnBitBlt(
        psoTrg,
        psoSrc_mut,
        psoMask_mut,
        pco_mut,
        pxlo,
        &mut rcClipped,
        if !psoSrc_mut.is_null() { &mut ptSrc } else { core::ptr::null_mut() },
        if !psoMask_mut.is_null() { &mut ptMask } else { core::ptr::null_mut() },
        pbo,
        if !pptlBrush.is_null() { &mut ptBrush } else { core::ptr::null_mut() },
        rop4,
    );

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngBitBlt(
    psoTrgUMPD: *mut SURFOBJ,
    psoSrcUMPD: *mut SURFOBJ,
    psoMaskUMPD: *mut SURFOBJ,
    pcoUMPD: *mut CLIPOBJ,
    pxloUMPD: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
    pptlMask: *mut POINTL,
    pboUMPD: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    rop4: ROP4,
) -> BOOL {

    let mut rclTrg: RECTL = core::mem::zeroed();
    let mut ptlSrc: POINTL = core::mem::zeroed();
    let mut ptlMask: POINTL = core::mem::zeroed();
    let mut ptlBrush: POINTL = core::mem::zeroed();
    let mut hsurfTrg: HSURF = core::ptr::null_mut();
    let mut hsurfSrc: HSURF = core::ptr::null_mut();
    let mut hsurfMask: HSURF = core::ptr::null_mut();
    let mut hBrushObj: HANDLE = core::ptr::null_mut();
    let psoTrg: *mut SURFOBJ;
    let psoSrc: *mut SURFOBJ;
    let psoMask: *mut SURFOBJ;
    let pco: *mut CLIPOBJ;
    let pxlo: *mut XLATEOBJ;
    let pbo: *mut BRUSHOBJ;
    let bResult: BOOL;
    let mut Status: NTSTATUS;

    Status = SafeProbeForRead(prclTrg as *const core::ffi::c_void, core::mem::size_of::<RECTL>(), 1);
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(psoTrgUMPD as *const core::ffi::c_void, core::mem::size_of::<SURFOBJ>(), 1);
    }
    if NT_SUCCESS(Status) && ROP4_USES_SOURCE(rop4) != 0 {
        Status = SafeProbeForRead(pptlSrc as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
    }
    if NT_SUCCESS(Status) && ROP4_USES_SOURCE(rop4) != 0 {
        Status = SafeProbeForRead(psoSrcUMPD as *const core::ffi::c_void, core::mem::size_of::<SURFOBJ>(), 1);
    }
    if NT_SUCCESS(Status) && ROP4_USES_MASK(rop4) != 0 {
        Status = SafeProbeForRead(pptlMask as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
    }
    if NT_SUCCESS(Status) && ROP4_USES_MASK(rop4) != 0 {
        Status = SafeProbeForRead(psoMaskUMPD as *const core::ffi::c_void, core::mem::size_of::<SURFOBJ>(), 1);
    }
    if NT_SUCCESS(Status) && ROP4_USES_PATTERN(rop4) != 0 {
        Status = SafeProbeForRead(pptlBrush as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
    }
    if NT_SUCCESS(Status) && ROP4_USES_PATTERN(rop4) != 0 {
        Status = SafeProbeForRead(psoSrcUMPD as *const core::ffi::c_void, core::mem::size_of::<SURFOBJ>(), 1);
    }
    if !NT_SUCCESS(Status) {
        return 0;
    } else {
        macro_rules! grab {
            ($dst:expr, $src:expr, $ty:ty) => {
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!($dst) as *mut core::ffi::c_void,
                    $src as *const core::ffi::c_void,
                    core::mem::size_of::<$ty>(),
                ) {
                    return 0;
                }
            };
        }

        grab!(rclTrg, prclTrg, RECTL);
        grab!(hsurfTrg, core::ptr::addr_of!((*psoTrgUMPD).hsurf), HSURF);

        if ROP4_USES_SOURCE(rop4) != 0 {
            grab!(ptlSrc, pptlSrc, POINTL);
            grab!(hsurfSrc, core::ptr::addr_of!((*psoSrcUMPD).hsurf), HSURF);
        }

        if ROP4_USES_MASK(rop4) != 0 {
            grab!(ptlMask, pptlMask, POINTL);
            grab!(hsurfMask, core::ptr::addr_of!((*psoMaskUMPD).hsurf), HSURF);
        }

        if ROP4_USES_PATTERN(rop4) != 0 {
            grab!(ptlBrush, pptlBrush, POINTL);
            grab!(hBrushObj, core::ptr::addr_of!((*pboUMPD).pvRbrush), HANDLE);
        }
    }

    psoTrg = core::ptr::null_mut();
    psoSrc = core::ptr::null_mut();
    psoMask = core::ptr::null_mut();
    pco = core::ptr::null_mut();
    pxlo = core::ptr::null_mut();
    pbo = core::ptr::null_mut();

    bResult = EngBitBlt(
        psoTrg,
        psoSrc,
        psoMask,
        pco,
        pxlo,
        core::ptr::addr_of_mut!(rclTrg),
        if !pptlSrc.is_null() { core::ptr::addr_of_mut!(ptlSrc) } else { core::ptr::null_mut() },
        if !pptlMask.is_null() { core::ptr::addr_of_mut!(ptlMask) } else { core::ptr::null_mut() },
        pbo,
        if !pptlBrush.is_null() { core::ptr::addr_of_mut!(ptlBrush) } else { core::ptr::null_mut() },
        rop4,
    );

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn EngCopyBits(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
) -> BOOL {
    let pfnCopyBits: PFN_DrvCopyBits;

    if SURFOBJ_flags(psoTrg) & HOOK_COPYBITS != 0 {
        pfnCopyBits = GDIDEVFUNCS(psoTrg).CopyBits;
    } else if SURFOBJ_flags(psoSrc) & HOOK_COPYBITS != 0 {
        pfnCopyBits = GDIDEVFUNCS(psoSrc).CopyBits;
    } else {
        return EngBitBlt(
            psoTrg,
            psoSrc,
            core::ptr::null_mut(),
            pco,
            pxlo,
            prclTrg,
            pptlSrc,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            ROP4_SRCCOPY,
        );
    }

    pfnCopyBits(psoTrg, psoSrc, pco, pxlo, prclTrg, pptlSrc)
}
#[no_mangle]
pub unsafe extern "system" fn IntEngCopyBits(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
) -> BOOL {
    EngCopyBits(psoTrg, psoSrc, pco, pxlo, prclTrg, pptlSrc)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
//
// #include <win32k.h>
// #include <safe_user_io.h>
// #include "../diblib/DibLib_interface.h"
// DBG_DEFAULT_CHANNEL(GdiBlt);
//
// #define SURFOBJ_flags(pso) (CONTAINING_RECORD(pso, SURFACE, SurfObj)->flags)
//
// // FIXME this needs to be updated, once we use the new structure
// extern XCLIPOBJ gxcoTrivial;
// /*
// {
//     {0, {LONG_MIN, LONG_MIN, LONG_MAX, LONG_MAX}, DC_TRIVIAL, FC_RECT, TC_RECTANGLES, 0},
//     0, 0, 0
// };
// */
//
// static
// void
// CalculateCoordinates(
//     PBLTDATA pbltdata,
//     PRECTL prclClipped,
//     PRECTL prclOrg,
//     PPOINTL pptlSrc,
//     PPOINTL pptlMask,
//     PPOINTL pptlPat,
//     PSIZEL psizlPat)
// {
//     ULONG cx, cy;
//
//     /* Calculate width and height of this rect */
//     pbltdata->ulWidth = prclClipped->right - prclClipped->left;
//     pbltdata->ulHeight = prclClipped->bottom - prclClipped->top;
//
//     /* Calculate the x offset to the origin coordinates */
//     if (pbltdata->siDst.iFormat == 0)
//         cx = (prclClipped->right - 1 - prclOrg->left);
//     else
//         cx = (prclClipped->left - prclOrg->left);
//
//     /* Calculate the y offset to the origin coordinates */
//     if (pbltdata->dy < 0)
//         cy = (prclClipped->bottom - 1 - prclOrg->top);
//     else
//         cy = (prclClipped->top - prclOrg->top);
//
//     /* Calculate the target start point */
//     pbltdata->siDst.ptOrig.x = prclOrg->left + cx;
//     pbltdata->siDst.ptOrig.y = prclOrg->top + cy;
//
//     /* Calculate start position for target */
//     pbltdata->siDst.pjBase = pbltdata->siDst.pvScan0;
//     pbltdata->siDst.pjBase += pbltdata->siDst.ptOrig.y * pbltdata->siDst.lDelta;
//     pbltdata->siDst.pjBase += pbltdata->siDst.ptOrig.x * pbltdata->siDst.jBpp / 8;
//
//     if (pptlSrc)
//     {
//         /* Calculate start point and bitpointer for source */
//         pbltdata->siSrc.ptOrig.x = pptlSrc->x + cx;
//         pbltdata->siSrc.ptOrig.y = pptlSrc->y + cy;
//         pbltdata->siSrc.pjBase = pbltdata->siSrc.pvScan0;
//         pbltdata->siSrc.pjBase += pbltdata->siSrc.ptOrig.y * pbltdata->siSrc.lDelta;
//         pbltdata->siSrc.pjBase += pbltdata->siSrc.ptOrig.x * pbltdata->siSrc.jBpp / 8;
//     }
//
//     if (pptlMask)
//     {
//         /* Calculate start point and bitpointer for mask */
//         pbltdata->siMsk.ptOrig.x = pptlMask->x + cx;
//         pbltdata->siMsk.ptOrig.y = pptlMask->y + cy;
//         pbltdata->siMsk.pjBase = pbltdata->siMsk.pvScan0;
//         pbltdata->siMsk.pjBase += pbltdata->siMsk.ptOrig.y * pbltdata->siMsk.lDelta;
//         pbltdata->siMsk.pjBase += pbltdata->siMsk.ptOrig.x * pbltdata->siMsk.jBpp / 8;
//     }
//
//     if (pptlPat)
//     {
//         /* Calculate start point and bitpointer for pattern */
//         pbltdata->siPat.ptOrig.x = (pptlPat->x + cx) % psizlPat->cx;
//         pbltdata->siPat.ptOrig.y = (pptlPat->y + cy) % psizlPat->cy;
//         pbltdata->siPat.pjBase = pbltdata->siPat.pvScan0;
//
//         /* Check for bottom-up case */
//         if (pbltdata->dy < 0)
//         {
//             pbltdata->siPat.pjBase += (psizlPat->cy - 1) * pbltdata->siPat.lDelta;
//             pbltdata->siPat.ptOrig.y = psizlPat->cy - 1 - pbltdata->siPat.ptOrig.y;
//         }
//
//         /* Check for right-to-left case */
//         if (pbltdata->siDst.iFormat == 0)
//         {
//             pbltdata->siPat.pjBase += (psizlPat->cx - 1) * pbltdata->siMsk.jBpp / 8;
//             pbltdata->siPat.ptOrig.x = psizlPat->cx - 1 - pbltdata->siPat.ptOrig.x;
//         }
//     }
// }
//
// BOOL
// APIENTRY
// EngBitBlt(
//     _Inout_ SURFOBJ *psoTrg,
//     _In_opt_ SURFOBJ *psoSrc,
//     _In_opt_ SURFOBJ *psoMask,
//     _In_opt_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ RECTL *prclTrg,
//     _When_(psoSrc, _In_) POINTL *pptlSrc,
//     _When_(psoMask, _In_) POINTL *pptlMask,
//     _In_opt_ BRUSHOBJ *pbo,
//     _When_(pbo, _In_) POINTL *pptlBrush,
//     _In_ ROP4 rop4)
// {
//     BLTDATA bltdata;
//     ULONG i, iFunctionIndex, iDirection = CD_ANY;
//     RECTL rcTrg;
//     PFN_DIBFUNCTION pfnBitBlt;
//     BOOL bEnumMore;
//     RECT_ENUM rcenum;
//     PSIZEL psizlPat;
//     SURFOBJ *psoPattern;
//
// //static int count = 0;
// //if (++count >= 1230) __debugbreak();
//
//     /* Sanity checks */
//     ASSERT(psoTrg);
//     ASSERT(psoTrg->iBitmapFormat >= BMF_1BPP);
//     ASSERT(psoTrg->iBitmapFormat <= BMF_32BPP);
//     ASSERT(prclTrg);
//     ASSERT(prclTrg->left >= 0);
//     ASSERT(prclTrg->top >= 0);
//     ASSERT(prclTrg->right <= psoTrg->sizlBitmap.cx);
//     ASSERT(prclTrg->bottom <= psoTrg->sizlBitmap.cy);
//
//     rcTrg = *prclTrg;
//
//     bltdata.dy = 1;
//     bltdata.rop4 = rop4;
//     bltdata.apfnDoRop[0] = gapfnRop[ROP4_BKGND(rop4)];
//     bltdata.apfnDoRop[1] = gapfnRop[ROP4_FGND(rop4)];
//     if (!pxlo) pxlo = &gexloTrivial.xlo;
//     bltdata.pxlo = pxlo;
//     bltdata.pfnXlate = XLATEOBJ_pfnXlate(pxlo);
//
//     /* Check if the ROP uses a source */
//     if (ROP4_USES_SOURCE(rop4))
//     {
//         /* Sanity checks */
//         ASSERT(psoSrc);
//         ASSERT(psoSrc->iBitmapFormat >= BMF_1BPP);
//         ASSERT(psoSrc->iBitmapFormat <= BMF_32BPP);
//         ASSERT(pptlSrc);
//         ASSERT(pptlSrc->x >= 0);
//         ASSERT(pptlSrc->y >= 0);
//         ASSERT(pptlSrc->x <= psoSrc->sizlBitmap.cx);
//         ASSERT(pptlSrc->y <= psoSrc->sizlBitmap.cy);
//
//         /* Check if source and target are equal */
//         if (psoSrc == psoTrg)
//         {
//             /* Analyze the copying direction */
//             if (rcTrg.top > pptlSrc->y)
//             {
//                 /* Need to copy from bottom to top */
//                 iDirection = rcTrg.left < pptlSrc->x ? CD_RIGHTUP : CD_LEFTUP;
//                 bltdata.dy = -1;
//             }
//             else
//                 iDirection = rcTrg.left < pptlSrc->x ? CD_RIGHTDOWN : CD_LEFTDOWN;
//
//             /* Check for special right to left case */
//             if ((rcTrg.top == pptlSrc->y) && (rcTrg.left > pptlSrc->x))
//             {
//                 /* Use 0 as target format to get special right to left versions */
//                 bltdata.siDst.iFormat = 0;
//                 bltdata.siSrc.iFormat = psoSrc->iBitmapFormat;
//                 //__debugbreak();
//             }
//             else
//             {
//                 /* Use 0 as source format to get special equal surface versions */
//                 bltdata.siDst.iFormat = psoTrg->iBitmapFormat;
//                 bltdata.siSrc.iFormat = 0;
//             }
//         }
//         else
//         {
//             bltdata.siDst.iFormat = psoTrg->iBitmapFormat;
//             bltdata.siSrc.iFormat = psoSrc->iBitmapFormat;
//         }
//
//         /* Set the source format info */
//         bltdata.siSrc.pvScan0 = psoSrc->pvScan0;
//         bltdata.siSrc.lDelta = psoSrc->lDelta;
//         bltdata.siSrc.cjAdvanceY = bltdata.dy * psoSrc->lDelta;
//         bltdata.siSrc.jBpp = gajBitsPerFormat[psoSrc->iBitmapFormat];
//     }
//     else
//     {
//         bltdata.siDst.iFormat = psoTrg->iBitmapFormat;
//     }
//
//     /* Set the destination format info */
//     bltdata.siDst.pvScan0 = psoTrg->pvScan0;
//     bltdata.siDst.lDelta = psoTrg->lDelta;
//     bltdata.siDst.cjAdvanceY = bltdata.dy * psoTrg->lDelta;
//     bltdata.siDst.jBpp = gajBitsPerFormat[psoTrg->iBitmapFormat];
//
//     /* Check if the ROP uses a pattern / brush */
//     if (ROP4_USES_PATTERN(rop4))
//     {
//         /* Must have a brush */
//         NT_ASSERT(pbo); // FIXME: test this!
//
//         /* Copy the solid color */
//         bltdata.ulSolidColor = pbo->iSolidColor;
//
//         /* Check if this is a pattern brush */
//         if (pbo->iSolidColor == 0xFFFFFFFF)
//         {
//             /* Get the realized pattern bitmap */
//             psoPattern = BRUSHOBJ_psoPattern(pbo);
//             if (!psoPattern)
//             {
//                 __debugbreak();
//                 return FALSE;
//             }
//
//             /* Set the pattern format info */
//             bltdata.siPat.iFormat = psoPattern->iBitmapFormat;
//             bltdata.siPat.pvScan0 = psoPattern->pvScan0;
//             bltdata.siPat.lDelta = psoPattern->lDelta;
//             bltdata.siPat.cjAdvanceY = bltdata.dy * psoPattern->lDelta;
//             bltdata.siPat.jBpp = gajBitsPerFormat[psoPattern->iBitmapFormat];
//
//             bltdata.ulPatWidth = psoPattern->sizlBitmap.cx;
//             bltdata.ulPatHeight = psoPattern->sizlBitmap.cy;
//
//             psizlPat = &psoPattern->sizlBitmap;
//         }
//         else
//         {
//             pptlBrush = NULL;
//             psizlPat = NULL;
//         }
//     }
//     else
//     {
//         pptlBrush = NULL;
//         psizlPat = NULL;
//     }
//
//     /* Check if the ROP uses a mask */
//     if (ROP4_USES_MASK(rop4))
//     {
//         //__debugbreak();
//
//         /* Check if we don't have a mask surface */
//         if (psoMask == NULL)
//         {
//             /* Must have a brush */
//             NT_ASSERT(pbo); // FIXME: test this!
//
//             /* Check if the BRUSHOBJ can provide the mask */
//             psoMask = BRUSHOBJ_psoMask(pbo);
//             if (psoMask == NULL)
//             {
//                 /* We have no mask, assume the mask is all foreground */
//                 rop4 = (rop4 & 0xFF) || ((rop4 & 0xFF) << 8);
//             }
//         }
//
//         /* Set the mask format info */
//         bltdata.siMsk.iFormat = psoMask->iBitmapFormat;
//         bltdata.siMsk.pvScan0 = psoMask->pvScan0;
//         bltdata.siMsk.lDelta = psoMask->lDelta;
//         bltdata.siMsk.cjAdvanceY = bltdata.dy * psoMask->lDelta;
//         bltdata.siMsk.jBpp = 1;
//
//         /* Calculate the masking function index */
//         iFunctionIndex = ROP4_USES_PATTERN(rop4) ? 1 : 0;
//         iFunctionIndex |= ROP4_USES_SOURCE(rop4) ? 2 : 0;
//         iFunctionIndex |= ROP4_USES_DEST(rop4) ? 4 : 0;
//
//         /* Get the masking function */
//         pfnBitBlt = gapfnMaskFunction[iFunctionIndex];
//     }
//     else
//     {
//         /* Get the function index from the foreground ROP index*/
//         iFunctionIndex = gajIndexPerRop[ROP4_FGND(rop4)];
//
//         /* Get the dib function */
//         pfnBitBlt = gapfnDibFunction[iFunctionIndex];
//     }
//
//     /* If no clip object is given, use trivial one */
//     if (!pco) pco = (CLIPOBJ*)&gxcoTrivial;
//
//     /* Check if we need to enumerate rects */
//     if (pco->iDComplexity == DC_COMPLEX)
//     {
//         /* Start the enumeration of the clip object */
//         CLIPOBJ_cEnumStart(pco, FALSE, CT_RECTANGLES, iDirection, 0);
//         bEnumMore = CLIPOBJ_bEnum(pco, sizeof(rcenum), (ULONG*)&rcenum);
//     }
//     else if (pco->iDComplexity == DC_RECT)
//     {
//         /* Use the clip bounds */
//         rcenum.arcl[0] = pco->rclBounds;
//         rcenum.c = 1;
//         bEnumMore = FALSE;
//     }
//     else
//     {
//         /* Use the target rect */
//         rcenum.arcl[0] = rcTrg;
//         rcenum.c = 1;
//         bEnumMore = FALSE;
//     }
//
//     /* Enter enumeration loop */
//     while (TRUE)
//     {
//         /* Loop all rectangles we got */
//         for (i = 0; i < rcenum.c; i++)
//         {
//             /* Intersect this rect with the target rect */
//             if (!RECTL_bIntersectRect(&rcenum.arcl[i], &rcenum.arcl[i], &rcTrg))
//             {
//                 /* This rect is outside the bounds, continue */
//                 continue;
//             }
//
//             /* Calculate coordinates and pointers */
//             CalculateCoordinates(&bltdata,
//                                  &rcenum.arcl[i],
//                                  prclTrg,
//                                  pptlSrc,
//                                  pptlMask,
//                                  pptlBrush,
//                                  psizlPat);
//
//             /* Call the dib function */
//             pfnBitBlt(&bltdata);
//         }
//
//         /* Bail out, when there's nothing more to do */
//         if(!bEnumMore) break;
//
//         /* Enumerate more rectangles */
//         bEnumMore = CLIPOBJ_bEnum(pco, sizeof(rcenum), (ULONG*)&rcenum);
//     }
//
//     return TRUE;
// }
//
//
// static
// VOID
// AdjustOffsetAndSize(
//     _Out_ PPOINTL pptOffset,
//     _Out_ PSIZEL psizTrg,
//     _In_ PPOINTL pptlSrc,
//     _In_ PSIZEL psizSrc)
// {
//     LONG x, y, cxMax, cyMax;
//
//     x = pptlSrc->x + pptOffset->x;
//     if (x < 0) pptOffset->x -= x, x = 0;
//
//     cxMax = psizSrc->cx - x;
//     if (psizTrg->cx > cxMax) psizTrg->cx = cxMax;
//
//     y = pptlSrc->y + pptOffset->y;
//     if (y < 0) pptOffset->y -= y, y = 0;
//
//     cyMax = psizSrc->cy - y;
//     if (psizTrg->cy > cyMax) psizTrg->cy = cyMax;
// }
//
// BOOL
// APIENTRY
// IntEngBitBlt(
//     _Inout_ SURFOBJ *psoTrg,
//     _In_opt_ SURFOBJ *psoSrc,
//     _In_opt_ SURFOBJ *psoMask,
//     _In_opt_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ RECTL *prclTrg,
//     _When_(psoSrc, _In_) POINTL *pptlSrc,
//     _When_(psoMask, _In_) POINTL *pptlMask,
//     _In_opt_ BRUSHOBJ *pbo,
//     _When_(pbo, _In_) POINTL *pptlBrush,
//     _In_ ROP4 rop4)
// {
//     BOOL bResult;
//     RECTL rcClipped;
//     POINTL ptOffset, ptSrc, ptMask, ptBrush;
//     SIZEL sizTrg;
//     PFN_DrvBitBlt pfnBitBlt;
//
// //__debugbreak();
//
//     /* Sanity checks */
//     ASSERT(IS_VALID_ROP4(rop4));
//     ASSERT(psoTrg);
//     ASSERT(psoTrg->iBitmapFormat >= BMF_1BPP);
//     ASSERT(psoTrg->iBitmapFormat <= BMF_32BPP);
//     ASSERT(prclTrg);
//
//     /* Clip the target rect to the extents of the target surface */
//     if (!RECTL_bClipRectBySize(&rcClipped, prclTrg, &psoTrg->sizlBitmap))
//     {
//         /* Nothing left */
//         return TRUE;
//     }
//
//     /* If no clip object is given, use trivial one */
//     if (!pco) pco = (CLIPOBJ*)&gxcoTrivial;
//
//     /* Check if there is something to clip */
//     if (pco->iDComplexity != DC_TRIVIAL)
//     {
//         /* Clip the target rect to the bounds of the clipping region */
//         if (!RECTL_bIntersectRect(&rcClipped, &rcClipped, &pco->rclBounds))
//         {
//             /* Nothing left */
//             return TRUE;
//         }
//     }
//
//     /* Don't pass a clip object with a single rectangle */
//     if (pco->iDComplexity == DC_RECT) pco = (CLIPOBJ*)&gxcoTrivial;
//
//     /* Calculate initial offset and size */
//     ptOffset.x = rcClipped.left - prclTrg->left;
//     ptOffset.y = rcClipped.top - prclTrg->top;
//     sizTrg.cx = rcClipped.right - rcClipped.left;
//     sizTrg.cy = rcClipped.bottom - rcClipped.top;
//
//     /* Check if the ROP uses a source */
//     if (ROP4_USES_SOURCE(rop4))
//     {
//         /* Must have a source surface and point */
//         ASSERT(psoSrc);
//         ASSERT(pptlSrc);
//
//         /* Get the source point */
//         ptSrc = *pptlSrc;
//
//         /* Clip against the extents of the source surface */
//         AdjustOffsetAndSize(&ptOffset, &sizTrg, &ptSrc, &psoSrc->sizlBitmap);
//     }
//     else
//     {
//         psoSrc = NULL;
//         ptSrc.x = 0;
//         ptSrc.y = 0;
//     }
//
//     /* Check if the ROP uses a mask */
//     if (ROP4_USES_MASK(rop4))
//     {
//         /* Must have a mask surface and point */
//         ASSERT(psoMask);
//         ASSERT(pptlMask);
//
//         /* Get the mask point */
//         ptMask = *pptlMask;
//
//         /* Clip against the extents of the mask surface */
//         AdjustOffsetAndSize(&ptOffset, &sizTrg, &ptMask, &psoMask->sizlBitmap);
//     }
//     else
//     {
//         psoMask = NULL;
//         ptMask.x = 0;
//         ptMask.y = 0;
//     }
//
//     /* Check if all has been clipped away */
//     if ((sizTrg.cx <= 0) || (sizTrg.cy <= 0))
//         return TRUE;
//
//     /* Adjust the points */
//     ptSrc.x += ptOffset.x;
//     ptSrc.y += ptOffset.y;
//     ptMask.x += ptOffset.x;
//     ptMask.y += ptOffset.y;
//
//     /* Check if we have a brush origin */
//     if (pptlBrush)
//     {
//         /* calculate the new brush origin */
//         ptBrush.x = pptlBrush->x + ptOffset.x;
//         ptBrush.y = pptlBrush->y + ptOffset.y;
//     }
//
//     /* Recalculate the target rect */
//     rcClipped.left = prclTrg->left + ptOffset.x;
//     rcClipped.top = prclTrg->top + ptOffset.y;
//     rcClipped.right = rcClipped.left + sizTrg.cx;
//     rcClipped.bottom = rcClipped.top + sizTrg.cy;
//
//     /* Is the target surface device managed? */
//     if (SURFOBJ_flags(psoTrg) & HOOK_BITBLT)
//     {
//         /* Is the source a different device managed surface? */
//         if (psoSrc && (psoSrc->hdev != psoTrg->hdev) &&
//             (SURFOBJ_flags(psoSrc) & HOOK_BITBLT))
//         {
//             ERR("Need to copy to standard bitmap format!\n");
//             ASSERT(FALSE);
//         }
//
//         pfnBitBlt = GDIDEVFUNCS(psoTrg).BitBlt;
//     }
//     /* Otherwise is the source surface device managed? */
//     else if (psoSrc && (SURFOBJ_flags(psoSrc) & HOOK_BITBLT))
//     {
//         pfnBitBlt = GDIDEVFUNCS(psoSrc).BitBlt;
//     }
//     else
//     {
//         pfnBitBlt = EngBitBlt;
//     }
//
//     bResult = pfnBitBlt(psoTrg,
//                         psoSrc,
//                         psoMask,
//                         pco,
//                         pxlo,
//                         &rcClipped,
//                         psoSrc ? &ptSrc : NULL,
//                         psoMask ? &ptMask : NULL,
//                         pbo,
//                         pptlBrush ? &ptBrush : NULL,
//                         rop4);
//
//     // FIXME: cleanup temp surface!
//
//     return bResult;
// }
//
// BOOL
// APIENTRY
// NtGdiEngBitBlt(
//     IN SURFOBJ *psoTrgUMPD,
//     IN SURFOBJ *psoSrcUMPD,
//     IN SURFOBJ *psoMaskUMPD,
//     IN CLIPOBJ *pcoUMPD,
//     IN XLATEOBJ *pxloUMPD,
//     IN RECTL *prclTrg,
//     IN POINTL *pptlSrc,
//     IN POINTL *pptlMask,
//     IN BRUSHOBJ *pboUMPD,
//     IN POINTL *pptlBrush,
//     IN ROP4 rop4)
// {
//     RECTL  rclTrg;
//     POINTL ptlSrc, ptlMask, ptlBrush;
//     HSURF hsurfTrg, hsurfSrc = NULL, hsurfMask = NULL;
//     HANDLE hBrushObj; // HUMPDOBJ
//     SURFOBJ *psoTrg, *psoSrc, *psoMask;
//     CLIPOBJ *pco;
//     XLATEOBJ *pxlo;
//     BRUSHOBJ *pbo;
//     BOOL bResult;
//     NTSTATUS Status;
//
//     Status = SafeProbeForRead(prclTrg, sizeof(RECTL), 1);
//     if (NT_SUCCESS(Status)) Status = SafeProbeForRead(psoTrgUMPD, sizeof(SURFOBJ), 1);
//     if (NT_SUCCESS(Status) && ROP4_USES_SOURCE(rop4)) Status = SafeProbeForRead(pptlSrc, sizeof(POINTL), 1);
//     if (NT_SUCCESS(Status) && ROP4_USES_SOURCE(rop4)) Status = SafeProbeForRead(psoSrcUMPD, sizeof(SURFOBJ), 1);
//     if (NT_SUCCESS(Status) && ROP4_USES_MASK(rop4)) Status = SafeProbeForRead(pptlMask, sizeof(POINTL), 1);
//     if (NT_SUCCESS(Status) && ROP4_USES_MASK(rop4)) Status = SafeProbeForRead(psoMaskUMPD, sizeof(SURFOBJ), 1);
//     if (NT_SUCCESS(Status) && ROP4_USES_PATTERN(rop4)) Status = SafeProbeForRead(pptlBrush, sizeof(POINTL), 1);
//     if (NT_SUCCESS(Status) && ROP4_USES_PATTERN(rop4)) Status = SafeProbeForRead(psoSrcUMPD, sizeof(SURFOBJ), 1);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//     else
//     {
//         rclTrg = *prclTrg;
//
//         hsurfTrg = psoTrgUMPD->hsurf;
//
//         if (ROP4_USES_SOURCE(rop4))
//         {
//             ptlSrc = *pptlSrc;
//
//             hsurfSrc = psoSrcUMPD->hsurf;
//         }
//
//         if (ROP4_USES_MASK(rop4))
//         {
//             ptlMask = *pptlMask;
//
//             hsurfMask = psoMaskUMPD->hsurf;
//         }
//
//         if (ROP4_USES_PATTERN(rop4))
//         {
//             ptlBrush = *pptlBrush;
//
//             hBrushObj = pboUMPD->pvRbrush; // FIXME
//         }
//     }
//
//     // FIXME: these need to be converted/locked!
//     psoTrg = NULL;
//     psoSrc = NULL;
//     psoMask = NULL;
//     pco = NULL;
//     pxlo = NULL;
//     pbo = NULL;
//
//     bResult = EngBitBlt(psoTrg,
//                         psoSrc,
//                         psoMask,
//                         pco,
//                         pxlo,
//                         &rclTrg,
//                         pptlSrc ? &ptlSrc : NULL,
//                         pptlMask ? &ptlMask : NULL,
//                         pbo,
//                         pptlBrush ? &ptlBrush : NULL,
//                         rop4);
//
//     return bResult;
// }
//
//
// BOOL
// APIENTRY
// EngCopyBits(
//     SURFOBJ *psoTrg,
//     SURFOBJ *psoSrc,
//     CLIPOBJ *pco,
//     XLATEOBJ *pxlo,
//     RECTL *prclTrg,
//     POINTL *pptlSrc)
// {
//     PFN_DrvCopyBits pfnCopyBits;
//
//     /* Is the target surface device managed? */
//     if (SURFOBJ_flags(psoTrg) & HOOK_COPYBITS)
//     {
//         pfnCopyBits = GDIDEVFUNCS(psoTrg).CopyBits;
//     }
//     if (SURFOBJ_flags(psoSrc) & HOOK_COPYBITS)
//     {
//         pfnCopyBits = GDIDEVFUNCS(psoSrc).CopyBits;
//     }
//     else
//     {
//         /* Use SRCCOPY for 2 bitmaps */
//         return EngBitBlt(psoTrg,
//                          psoSrc,
//                          NULL,
//                          pco,
//                          pxlo,
//                          prclTrg,
//                          pptlSrc,
//                          NULL,
//                          NULL,
//                          NULL,
//                          ROP4_SRCCOPY);
//     }
//
//     /* Forward to the driver */
//     return pfnCopyBits(psoTrg, psoSrc, pco, pxlo, prclTrg, pptlSrc);
// }
//
// BOOL
// APIENTRY
// IntEngCopyBits(
//     SURFOBJ *psoTrg,
//     SURFOBJ *psoSrc,
//     CLIPOBJ *pco,
//     XLATEOBJ *pxlo,
//     RECTL *prclTrg,
//     POINTL *pptlSrc)
// {
//     return EngCopyBits(psoTrg, psoSrc, pco, pxlo, prclTrg, pptlSrc);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}
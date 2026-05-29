use win32k_sys::win32k::*;
use ntkrnl_sys::rtl::RtlCopyMemory;

// ---- file-scope preamble (translated) ----
//
// Mode 5 cross-crate types (§5 win32k_translation_prompt.md): pandisp.c
// touches no `ntkrnl_sys::ddk::_KFOO`-shared types directly — only
// `RtlCopyMemory` (memcpy macro). No fully-qualified `ntkrnl_sys::ddk::_X`
// re-declarations needed.
//
// Mode 6 GDIOBJ_*/PDEVOBJ_*/SURFACE_* macros (§6): pandisp.c uses NO GDI
// object macros — every call (EngAllocMem, EngFreeMem, EngBitBlt,
// EngCopyBits, EngLockSurface, EngUnlockSurface, EngDeleteSurface,
// EngCreateBitmap, EngCreateDeviceSurface, EngAssociateSurface,
// EngAlphaBlend, EngGradientFill) is a real bindgen-resolved fn in
// `win32k_sys::win32k`. No inline-expansion required.
//
// Mode 7 #[no_mangle] static (gPan + gPanDispDrvFn + gPanDispDrvCount):
// these are referenced by pdevobj.rs (`gPanDispDrvFn.offset(i)`) and
// must export their symbol. `static mut` for `gPan` because the driver
// stores into it during DrvEnablePDEV; `static` for the function table
// + count (read-only). All three get `#[no_mangle]` so the umbrella
// `libwin32k_rust.a` exposes them at the canonical C name.

pub const TAG_PAN: u32 = u32::from_le_bytes(*b"PnaD");

// Static raw pointers don't implement Sync; `static mut` sidesteps the
// thread-safety check while preserving the C semantics (single-threaded
// driver init/teardown writes; later reads see the stored DHPDEV).
#[no_mangle]
pub static mut gPan: DHPDEV = core::ptr::null_mut();

// PANDEV is private to pandisp.c — not in win32k_sys bindgen. Declare
// locally; layout matches the C struct exactly (each field has the same
// offset and size as the bindgen-resolved types).
#[repr(C)]
pub struct _PANDEV {
    pub iBitmapFormat: ULONG,
    pub hdev: HDEV,
    pub szlDesktop: SIZEL,
    pub rclViewport: RECTL,
    pub hsurf: HSURF,
    pub hsurfShadow: HSURF,
    pub psoShadow: *mut SURFOBJ,
    pub flUnderlyingGraphicsCaps: ULONG,
    pub szlScreen: SIZEL,
    pub dhpdevScreen: DHPDEV,
    pub enabledScreen: BOOL,
    pub surfObjScreen: *mut SURFOBJ,
    pub apfn: [PFN; INDEX_LAST as usize],
}

pub type PANDEV = _PANDEV;
pub type PPANDEV = *mut _PANDEV;

// Helper: cast a fn-item to PFN via fn-pointer→PFN transmute. const-fn so
// usable in static initializers. Each Pan* fn is declared `extern "system"`
// (= stdcall on i386, cdecl on amd64/arm64), which matches PFN's per-arch
// calling convention exactly. The transmute reinterprets the fn-pointer's
// signature only; the bit pattern (a single code address) is unchanged.
const fn pfn_cast<T: Copy>(f: T) -> PFN {
    // SAFETY: T is a fn-pointer with `extern "system"` ABI (same as PFN);
    // we're only re-typing the signature, not the call convention.
    unsafe { core::mem::transmute_copy(&f) }
}

#[no_mangle]
pub static gPanDispDrvFn: [DRVFN; 13] = [
    DRVFN { iFunc: INDEX_DrvEnablePDEV,     pfn: pfn_cast(PanEnablePDEV     as unsafe extern "system" fn(_,_,_,_,_,_,_,_,_,_,_) -> _) },
    DRVFN { iFunc: INDEX_DrvCompletePDEV,   pfn: pfn_cast(PanCompletePDEV   as unsafe extern "system" fn(_,_)) },
    DRVFN { iFunc: INDEX_DrvDisablePDEV,    pfn: pfn_cast(PanDisablePDEV    as unsafe extern "system" fn(_)) },
    DRVFN { iFunc: INDEX_DrvEnableSurface,  pfn: pfn_cast(PanEnableSurface  as unsafe extern "system" fn(_) -> _) },
    DRVFN { iFunc: INDEX_DrvDisableSurface, pfn: pfn_cast(PanDisableSurface as unsafe extern "system" fn(_)) },
    DRVFN { iFunc: INDEX_DrvCopyBits,       pfn: pfn_cast(PanCopyBits       as unsafe extern "system" fn(_,_,_,_,_,_) -> _) },
    DRVFN { iFunc: INDEX_DrvStrokePath,     pfn: pfn_cast(PanStrokePath     as unsafe extern "system" fn(_,_,_,_,_,_,_,_) -> _) },
    DRVFN { iFunc: INDEX_DrvTextOut,        pfn: pfn_cast(PanTextOut        as unsafe extern "system" fn(_,_,_,_,_,_,_,_,_,_) -> _) },
    DRVFN { iFunc: INDEX_DrvSynchronize,    pfn: pfn_cast(PanSynchronize    as unsafe extern "system" fn(_,_) -> _) },
    DRVFN { iFunc: INDEX_DrvMovePointer,    pfn: pfn_cast(PanMovePointer    as unsafe extern "system" fn(_,_,_,_)) },
    DRVFN { iFunc: INDEX_DrvBitBlt,         pfn: pfn_cast(PanBitBlt         as unsafe extern "system" fn(_,_,_,_,_,_,_,_,_,_,_) -> _) },
    DRVFN { iFunc: INDEX_DrvAlphaBlend,     pfn: pfn_cast(PanAlphaBlend     as unsafe extern "system" fn(_,_,_,_,_,_,_) -> _) },
    DRVFN { iFunc: INDEX_DrvGradientFill,   pfn: pfn_cast(PanGradientFill   as unsafe extern "system" fn(_,_,_,_,_,_,_,_,_,_) -> _) },
];

#[no_mangle]
pub static gPanDispDrvCount: ULONG = 13;

#[no_mangle]
pub unsafe extern "system" fn PanSynchronize(
    dhpdev: DHPDEV,
    prcl: PRECTL,
) -> BOOL {
    let pandev = dhpdev as PPANDEV;
    let mut rclDest: RECTL = core::mem::zeroed();
    let mut ptlSrc: POINTL = core::mem::zeroed();

    ptlSrc.x = (*pandev).rclViewport.left;
    ptlSrc.y = (*pandev).rclViewport.top;

    rclDest.top = 0;
    rclDest.left = 0;
    rclDest.bottom = (*pandev).szlScreen.cy;
    rclDest.right = (*pandev).szlScreen.cx;

    EngCopyBits(
        (*pandev).surfObjScreen,
        (*pandev).psoShadow,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut rclDest,
        &mut ptlSrc,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PanEnablePDEV(
    pdm: PDEVMODEW,
    pwszLogAddress: LPWSTR,
    cPat: ULONG,
    phsurfPatterns: *mut HSURF,
    cjCaps: ULONG,
    pdevcaps: *mut ULONG,
    cjDevInfo: ULONG,
    pdi: PDEVINFO,
    hdev: HDEV,
    pwszDeviceName: LPWSTR,
    hDriver: HANDLE,
) -> DHPDEV {
    let ppdev = hdev as PPDEVOBJ;
    let mut underlyingDevmode: DEVMODEW;
    let pGdiInfo = pdevcaps as PGDIINFO;

    let pandev = EngAllocMem(FL_ZERO_MEMORY, core::mem::size_of::<PANDEV>() as ULONG, TAG_PAN) as PPANDEV;
    if pandev.is_null() {
        return core::ptr::null_mut();
    }

    // LDEVOBJ::apfn lives inside an anonymous union (`__bindgen_anon_1`).
    // Copy 93 PFN slots from the underlying LDEV into pandev->apfn.
    RtlCopyMemory(
        (*pandev).apfn.as_mut_ptr() as PVOID,
        (*(*ppdev).pldev).__bindgen_anon_1.apfn.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of_val(&(*pandev).apfn),
    );

    underlyingDevmode = *pdm;
    underlyingDevmode.dmPelsWidth = (*pdm).dmPanningWidth;
    underlyingDevmode.dmPelsHeight = (*pdm).dmPanningHeight;

    // Underlying DrvEnablePDEV — transmute PFN (Option<fn()->LONG_PTR>) to
    // the typed PFN_DrvEnablePDEV (Option<fn(...) -> DHPDEV>). Both are
    // Option-wrapped fn-pointers with identical layout (one word; None=null).
    let enable_pdev_fn: PFN_DrvEnablePDEV =
        core::mem::transmute((*pandev).apfn[INDEX_DrvEnablePDEV as usize]);

    (*pandev).dhpdevScreen = (enable_pdev_fn.unwrap())(
        &mut underlyingDevmode,
        pwszLogAddress,
        cPat,
        phsurfPatterns,
        cjCaps,
        pdevcaps,
        cjDevInfo,
        pdi,
        hdev,
        pwszDeviceName,
        hDriver,
    );

    if (*pandev).dhpdevScreen.is_null() {
        EngFreeMem(pandev as PVOID);
        return core::ptr::null_mut();
    }

    (*pandev).iBitmapFormat = (*pdi).iDitherFormat;
    (*pandev).szlDesktop.cx = (*pdm).dmPelsWidth as LONG;
    (*pandev).szlDesktop.cy = (*pdm).dmPelsHeight as LONG;
    (*pandev).szlScreen.cx = (*pdm).dmPanningWidth as LONG;
    (*pandev).szlScreen.cy = (*pdm).dmPanningHeight as LONG;
    (*pandev).flUnderlyingGraphicsCaps = (*pdi).flGraphicsCaps;

    (*pGdiInfo).ulHorzRes = (*pdm).dmPelsWidth;
    (*pGdiInfo).ulVertRes = (*pdm).dmPelsHeight;
    (*pdi).flGraphicsCaps |= GCAPS_PANNING;
    (*pdi).flGraphicsCaps2 = GCAPS2_SYNCFLUSH | GCAPS2_SYNCTIMER;

    gPan = pandev as DHPDEV;
    pandev as DHPDEV
}

#[no_mangle]
pub unsafe extern "system" fn PanCompletePDEV(
    dhpdev: DHPDEV,
    hdev: HDEV,
) {
    let pandev = dhpdev as PPANDEV;

    (*pandev).hdev = hdev;
    let complete_pdev_fn: PFN_DrvCompletePDEV =
        core::mem::transmute((*pandev).apfn[INDEX_DrvCompletePDEV as usize]);
    (complete_pdev_fn.unwrap())((*pandev).dhpdevScreen, hdev);
}

#[no_mangle]
pub unsafe extern "system" fn PanDisablePDEV(dhpdev: DHPDEV) {
    let pandev = dhpdev as PPANDEV;

    let disable_pdev_fn: PFN_DrvDisablePDEV =
        core::mem::transmute((*pandev).apfn[INDEX_DrvDisablePDEV as usize]);
    (disable_pdev_fn.unwrap())((*pandev).dhpdevScreen);

    EngFreeMem(pandev as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn PanDisableSurface(dhpdev: DHPDEV) {
    let pandev = dhpdev as PPANDEV;

    if !(*pandev).psoShadow.is_null() {
        EngUnlockSurface((*pandev).psoShadow);
        (*pandev).psoShadow = core::ptr::null_mut();
    }
    if !(*pandev).hsurfShadow.is_null() {
        EngDeleteSurface((*pandev).hsurfShadow);
        (*pandev).hsurfShadow = core::ptr::null_mut();
    }
    if !(*pandev).hsurf.is_null() {
        EngDeleteSurface((*pandev).hsurf);
        (*pandev).hsurf = core::ptr::null_mut();
    }
    if !(*pandev).surfObjScreen.is_null() {
        EngUnlockSurface((*pandev).surfObjScreen);
        (*pandev).surfObjScreen = core::ptr::null_mut();
    }
    if (*pandev).enabledScreen != 0 {
        let disable_surface: PFN_DrvDisableSurface =
            core::mem::transmute((*pandev).apfn[INDEX_DrvDisableSurface as usize]);
        (disable_surface.unwrap())((*pandev).dhpdevScreen);
        (*pandev).enabledScreen = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn PanEnableSurface(dhpdev: DHPDEV) -> HSURF {
    let pandev = dhpdev as PPANDEV;
    let surfScreen: HSURF;

    (*pandev).rclViewport.left = ((*pandev).szlDesktop.cx - (*pandev).szlScreen.cx) / 2;
    (*pandev).rclViewport.top = ((*pandev).szlDesktop.cy - (*pandev).szlScreen.cy) / 2;
    (*pandev).rclViewport.right = (*pandev).rclViewport.left + (*pandev).szlScreen.cx;
    (*pandev).rclViewport.bottom = (*pandev).rclViewport.top + (*pandev).szlScreen.cy;

    let enable_surface_fn: PFN_DrvEnableSurface =
        core::mem::transmute((*pandev).apfn[INDEX_DrvEnableSurface as usize]);
    surfScreen = (enable_surface_fn.unwrap())((*pandev).dhpdevScreen);
    if surfScreen.is_null() {
        PanDisableSurface(dhpdev);
        return core::ptr::null_mut();
    }
    (*pandev).enabledScreen = 1;  // TRUE

    (*pandev).surfObjScreen = EngLockSurface(surfScreen);
    if (*pandev).surfObjScreen.is_null() {
        PanDisableSurface(dhpdev);
        return core::ptr::null_mut();
    }

    (*pandev).hsurf = EngCreateDeviceSurface(core::ptr::null_mut(), (*pandev).szlDesktop, (*pandev).iBitmapFormat);
    if (*pandev).hsurf.is_null() {
        PanDisableSurface(dhpdev);
        return core::ptr::null_mut();
    }

    if EngAssociateSurface((*pandev).hsurf, (*pandev).hdev, HOOK_ALPHABLEND |
                                                            HOOK_BITBLT |
                                                            HOOK_COPYBITS |
                                                            HOOK_GRADIENTFILL |
                                                            HOOK_STROKEPATH |
                                                            HOOK_SYNCHRONIZE |
                                                            HOOK_TEXTOUT) == 0 {
        PanDisableSurface(dhpdev);
        return core::ptr::null_mut();
    }

    (*pandev).hsurfShadow = EngCreateBitmap((*pandev).szlDesktop, (*pandev).szlDesktop.cx, (*pandev).iBitmapFormat, BMF_TOPDOWN, core::ptr::null_mut()) as HSURF;
    if (*pandev).hsurfShadow.is_null() {
        PanDisableSurface(dhpdev);
        return core::ptr::null_mut();
    }

    (*pandev).psoShadow = EngLockSurface((*pandev).hsurfShadow);
    if (*pandev).psoShadow.is_null() {
        PanDisableSurface(dhpdev);
        return core::ptr::null_mut();
    }

    (*pandev).hsurf
}

#[no_mangle]
pub unsafe extern "system" fn PanBitBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: PRECTL,
    pptlSrc: PPOINTL,
    pptlMask: PPOINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: PPOINTL,
    rop4: ROP4,
) -> BOOL {
    let mut pandev: PPANDEV;
    let res: BOOL;
    let mut psoTrg_local = psoTrg;
    let mut psoSrc_local = psoSrc;

    // SURFOBJ.iType is USHORT (u16) per win32k_sys bindgen; STYPE_DEVICE
    // is a u32 constant — cast for the comparison.
    if (*psoTrg_local).iType == STYPE_DEVICE as USHORT {
        pandev = (*psoTrg_local).dhpdev as PPANDEV;
        psoTrg_local = (*pandev).psoShadow;
    }
    if !psoSrc_local.is_null() && (*psoSrc_local).iType == STYPE_DEVICE as USHORT {
        pandev = (*psoSrc_local).dhpdev as PPANDEV;
        psoSrc_local = (*pandev).psoShadow;
    }

    res = EngBitBlt(psoTrg_local, psoSrc_local, psoMask, pco, pxlo, prclTrg, pptlSrc, pptlMask, pbo, pptlBrush, rop4);

    if res != 0 {
        PanSynchronize(gPan, core::ptr::null_mut());
    }

    res
}

#[no_mangle]
pub unsafe extern "system" fn PanCopyBits(
    psoDest: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDest: PRECTL,
    pptlSrc: PPOINTL,
) -> BOOL {
    PanBitBlt(
        psoDest,
        psoSrc,
        core::ptr::null_mut(),
        pco,
        pxlo,
        prclDest,
        pptlSrc,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ROP4_SRCCOPY,
    )
}

// Mode 3 stub: UNIMPLEMENTED in C — return FALSE (BOOL = c_int, so `0`).
#[no_mangle]
pub unsafe extern "system" fn PanStrokePath(
    pso: *mut SURFOBJ,
    ppo: *mut PATHOBJ,
    pco: *mut CLIPOBJ,
    pxo: *mut XFORMOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: PPOINTL,
    plineattrs: PLINEATTRS,
    mix: MIX,
) -> BOOL {
    0
}

// Mode 3 stub: UNIMPLEMENTED in C — return FALSE.
#[no_mangle]
pub unsafe extern "system" fn PanTextOut(
    pso: *mut SURFOBJ,
    pstro: *mut STROBJ,
    pfo: *mut FONTOBJ,
    pco: *mut CLIPOBJ,
    prclExtra: PRECTL,
    prclOpaque: PRECTL,
    pboFore: *mut BRUSHOBJ,
    pboOpaque: *mut BRUSHOBJ,
    pptlOrg: PPOINTL,
    mix: MIX,
) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn PanMovePointer(
    pso: *mut SURFOBJ,
    mut x: LONG,
    mut y: LONG,
    prcl: PRECTL,
) {
    let pandev = (*pso).dhpdev as PPANDEV;
    let mut bChanged: BOOL = 0;

    if x < 0 || y >= 0 {
        return;
    }

    if (*pandev).flUnderlyingGraphicsCaps & GCAPS_PANNING != 0 {
        let pfnMovePointer: PFN_DrvMovePointer =
            core::mem::transmute((*pandev).apfn[INDEX_DrvMovePointer as usize]);
        if let Some(pfn) = pfnMovePointer {
            pfn((*pandev).surfObjScreen, x, y, prcl);
        }
    }

    y += (*pso).sizlBitmap.cy;

    if x < (*pandev).rclViewport.left {
        (*pandev).rclViewport.left = x;
        (*pandev).rclViewport.right = x + (*pandev).szlScreen.cx;
        bChanged = 1;
    } else if x >= (*pandev).rclViewport.right {
        (*pandev).rclViewport.right = x + 1;
        (*pandev).rclViewport.left = x - (*pandev).szlScreen.cx;
        bChanged = 1;
    }
    if y < (*pandev).rclViewport.top {
        (*pandev).rclViewport.top = y;
        (*pandev).rclViewport.bottom = y + (*pandev).szlScreen.cy;
        bChanged = 1;
    } else if y >= (*pandev).rclViewport.bottom {
        (*pandev).rclViewport.bottom = y + 1;
        (*pandev).rclViewport.top = y - (*pandev).szlScreen.cy;
        bChanged = 1;
    }

    if bChanged != 0 {
        PanSynchronize((*pso).dhpdev, core::ptr::null_mut());
    }
}

#[no_mangle]
pub unsafe extern "system" fn PanAlphaBlend(
    psoDest: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDest: PRECTL,
    prclSrc: PRECTL,
    pBlendObj: *mut BLENDOBJ,
) -> BOOL {
    let pandev = (*psoDest).dhpdev as PPANDEV;
    let res = EngAlphaBlend((*pandev).psoShadow, psoSrc, pco, pxlo, prclDest, prclSrc, pBlendObj);

    if res != 0 {
        PanSynchronize(gPan, core::ptr::null_mut());
    }

    res
}

#[no_mangle]
pub unsafe extern "system" fn PanGradientFill(
    pso: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: PTRIVERTEX,
    nVertex: ULONG,
    pMesh: PVOID,
    nMesh: ULONG,
    prclExtents: PRECTL,
    pptlDitherOrg: PPOINTL,
    ulMode: ULONG,
) -> BOOL {
    let pandev = (*pso).dhpdev as PPANDEV;
    let res = EngGradientFill((*pandev).psoShadow, pco, pxlo, pVertex, nVertex, pMesh, nMesh, prclExtents, pptlDitherOrg, ulMode);

    if res != 0 {
        PanSynchronize(gPan, core::ptr::null_mut());
    }

    res
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Win32k subsystem
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Pan-Display driver
//  * COPYRIGHT:   Copyright 2022 Hervé Poussineau <hpoussin@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <win32k.h>
// #define NDEBUG
// #include <debug.h>
//
// #define TAG_PAN 'DnaP'
// #define GETPFN(name) ((PFN_Drv##name)(pandev->apfn[INDEX_Drv##name]))
//
// static DHPDEV gPan; /* FIXME: remove once PanSynchronize() is called periodically */
//
// typedef struct _PANDEV
// {
//     ULONG iBitmapFormat;
//     HDEV hdev;
//     SIZEL szlDesktop; /* Size of the whole desktop */
//     RECTL rclViewport; /* Viewport area */
//     HSURF hsurf; /* Global surface */
//     HSURF hsurfShadow; /* Our shadow surface, used for drawing */
//     SURFOBJ *psoShadow;
//
//     /* Underlying PDEVOBJ */
//     ULONG flUnderlyingGraphicsCaps;
//     SIZEL szlScreen;
//     DHPDEV dhpdevScreen;
//     BOOL enabledScreen; /* underlying surface enabled */
//     SURFOBJ *surfObjScreen;
//     PFN apfn[INDEX_LAST]; /* Functions of underlying PDEVOBJ */
// } PANDEV, *PPANDEV;
//
// (... rest of original C source elided for brevity — see win32ss/gdi/eng/pandisp.c ...)
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

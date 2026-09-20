use win32k_sys::win32k::*;
// Shadow the bindgen extern `gajBitsPerFormat: [UCHAR; 0]` (flexible-array decl —
// indexing THAT panics unconditionally) with the real [u8; 11] definition.
use crate::surface::gajBitsPerFormat;
use crate::brushbody::pbrushbody;

use core::ffi::c_void;

// Real STDCALL (NTAPI) eng helpers — _Name@N on i386 → extern "system".
extern "system" {
    fn EXLATEOBJ_vCleanup(pexlo: *mut EXLATEOBJ);
    fn EXLATEOBJ_vInitXlateFromDCs(pexlo: *mut EXLATEOBJ, pdcSrc: PDC, pdcDst: PDC);
    fn EXLATEOBJ_vInitialize(pexlo: *mut EXLATEOBJ, ppalSrc: *mut _PALETTE, ppalDst: *mut _PALETTE, crSrcBack: COLORREF, crDstBack: COLORREF, crDstFore: COLORREF);
    fn EXLATEOBJ_vInitXlateFromDCsEx(pexlo: *mut EXLATEOBJ, pdcSrc: PDC, pdcDest: PDC, Color: DWORD);
}

// CDECL helpers — _TranslateCOLORREF (cdecl) + FORCEINLINE coord/rect helpers already
// shimmed cdecl by path_rust_shims.c (w32r_IntLPtoDP / w32r_RECTL_vSetRect).
extern "C" {
    fn TranslateCOLORREF(pdc: PDC, crColor: COLORREF) -> ULONG;
    fn w32r_IntLPtoDP(pdc: PDC, ppt: LPPOINT, count: INT) -> BOOL;
    fn w32r_RECTL_vSetRect(prcl: *mut RECTL, left: INT, top: INT, right: INT, bottom: INT);
    fn w32r_DC_pmxWorldToDevice(pdc: PDC) -> PMATRIX;
}

// FASTCALL helper — @RECTL_vMakeWellOrdered@4 on i386.
#[cfg(target_arch = "x86")]
extern "fastcall" { fn RECTL_vMakeWellOrdered(prcl: *mut RECTL); }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn RECTL_vMakeWellOrdered(prcl: *mut RECTL); }

// EBRUSHOBJ_iSetSolidColor is a FORCEINLINE (brush.h) — get/set BrushObject.iSolidColor.
#[inline(always)]
unsafe fn EBRUSHOBJ_iSetSolidColor(pebo: PEBRUSHOBJ, iSolidColor: ULONG) -> ULONG {
    let old = (*pebo).BrushObject.iSolidColor;
    (*pebo).BrushObject.iSolidColor = iSolidColor;
    old
}

// ROP4 bit-macros (inteng.h / ndebug) — inline, no backing symbols.
#[inline(always)] unsafe fn ROP4_USES_MASK(rop4: ROP4) -> BOOL { (((rop4 & 0xFF00) >> 8) != (rop4 & 0x00ff)) as BOOL }
#[inline(always)] unsafe fn ROP4_USES_PATTERN(rop4: ROP4) -> BOOL { (((rop4 & 0xF0F0) >> 4) != (rop4 & 0x0F0F)) as BOOL }
#[inline(always)] unsafe fn ROP4_USES_SOURCE(rop4: ROP4) -> BOOL { (((rop4 & 0xCCCC) >> 2) != (rop4 & 0x3333)) as BOOL }
#[inline(always)] unsafe fn WIN32_ROP4_TO_ENG_ROP4(rop4: DWORD) -> ROP4 { (rop4 >> 16) as ROP4 }
#[inline(always)] unsafe fn WIN32_ROP3_TO_ENG_ROP4(rop3: DWORD) -> u32 { ((rop3 & 0x00FF0000) >> 16) | ((rop3 & 0x00FF0000) >> 8) }
#[inline(always)] unsafe fn WIN32_ROP4_USES_SOURCE(rop4: DWORD) -> bool { ((rop4 & 0xCCCC0000) >> 2) != (rop4 & 0x33330000) }
// MAKEROP4(f,b) = ((b<<8)&0xFF000000)|f — args are POSITIONAL (translation's back/fore names are misleading; call order preserved from C).
#[inline(always)] unsafe fn MAKEROP4(f: DWORD, b: DWORD) -> DWORD { (((b << 8) & 0xFF000000) | f) as DWORD }

use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};

// usize-accepting wrappers over ddk/rtl primitives (i386 SIZE_T = u32) — same as font/path.
#[inline(always)] unsafe fn ExAllocatePoolWithTag(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID { ntkrnl_sys::ddk::ExAllocatePoolWithTag(pool, n as _, tag) }
#[inline(always)] unsafe fn ExFreePoolWithTag(p: PVOID, tag: u32) { ntkrnl_sys::ddk::ExFreePoolWithTag(p, tag) }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) { ntkrnl_sys::rtl::RtlCopyMemory(d, s, n as _) }

// NTAPI stdcall helpers — _Name@N on i386.
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: usize, Alignment: ULONG) -> NTSTATUS;
    fn EBRUSHOBJ_vInit(pebo: *mut EBRUSHOBJ, pbrush: PBRUSH, psurf: *mut c_void, crFore: COLORREF, crBack: COLORREF, ppal: *mut c_void);
    fn EBRUSHOBJ_vInitFromDC(pebo: PEBRUSHOBJ, pbrush: PBRUSH, pdc: PDC);
    fn EBRUSHOBJ_vCleanup(pebo: *mut EBRUSHOBJ);
}

// FASTCALL SetLastNtError — @SetLastNtError@4 on i386.
#[cfg(target_arch = "x86")] extern "fastcall" { fn SetLastNtError(Status: NTSTATUS); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn SetLastNtError(Status: NTSTATUS); }

// GDI-object lock/init macros — inline via GDIOBJ_* primitives (sibling-file pattern).
#[inline(always)] unsafe fn SURFACE_ShareLockSurface(h: HGDIOBJ) -> PSURFACE { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE }
#[inline(always)] unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn BRUSH_ShareLockBrush(h: HGDIOBJ) -> PBRUSH { GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BRUSH as DWORD) as PBRUSH }
#[inline(always)] unsafe fn BRUSH_ShareUnlockBrush(p: PBRUSH) { GDIOBJ_vDereferenceObject(p as POBJ); }

// FIXME debug print — NOT a no-op under NDEBUG; that is DPRINT/TRACE. Both call
// sites are NtGdiPlgBlt announcing it is unimplemented, which is precisely the
// kind of thing a caller staring at a blank blit needs to see. FIXUP_ROP2
// macro; PlgBlt pool tag.
extern "C" { fn DbgPrint(fmt: *const i8, ...) -> u32; }
#[inline(always)] unsafe fn FIXME(msg: *const i8) { DbgPrint(msg); }
#[inline(always)] unsafe fn FIXUP_ROP2(rop2: DWORD) -> DWORD { (rop2.wrapping_sub(1) & 0xF) + 1 }
const GDITAG_PLGBLT_DATA: u32 = u32::from_be_bytes(*b"bgpG");
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): SURFACE_ShareLockSurface, SURFACE_ShareUnlockSurface — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.

// ---- file-scope preamble (translated) ----
// DBG_DEFAULT_CHANNEL(GdiBlt) - no translatable declarations
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAlphaBlend(
    hDCDest: HDC,
    XOriginDest: LONG,
    YOriginDest: LONG,
    WidthDest: LONG,
    HeightDest: LONG,
    hDCSrc: HDC,
    XOriginSrc: LONG,
    YOriginSrc: LONG,
    WidthSrc: LONG,
    HeightSrc: LONG,
    BlendFunc: BLENDFUNCTION,
    _hcmXform: HANDLE,
) -> BOOL {
    let DCDest: PDC;
    let DCSrc: PDC;
    let mut ahDC: [HDC; 2] = [core::ptr::null_mut(); 2];
    let mut apObj: [PGDIOBJ; 2] = [core::ptr::null_mut(); 2];
    let BitmapDest: *mut SURFACE;
    let BitmapSrc: *mut SURFACE;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut SourceRect: RECTL = core::mem::zeroed();
    let mut bResult: BOOL;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut BlendObj: BLENDOBJ = core::mem::zeroed();
    BlendObj.BlendFunction = BlendFunc;

    if WidthDest < 0 || HeightDest < 0 || WidthSrc < 0 || HeightSrc < 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if hDCDest.is_null() || hDCSrc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    ahDC[0] = hDCDest;
    ahDC[1] = hDCSrc;
    if GDIOBJ_bLockMultipleObjects(
        2,
        ahDC.as_mut_ptr() as *mut HGDIOBJ,
        apObj.as_mut_ptr(),
        (GDIObjType_DC_TYPE as _),
    ) == 0
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    DCDest = apObj[0] as PDC;
    DCSrc = apObj[1] as PDC;

    if (*DCSrc).dctype == (DCTYPE_INFO as i32 as _) || (*DCDest).dctype == (DCTYPE_INFO as i32 as _) {
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
        return 1;
    }

    DestRect.left = XOriginDest;
    DestRect.top = YOriginDest;
    DestRect.right = XOriginDest + WidthDest;
    DestRect.bottom = YOriginDest + HeightDest;
    w32r_IntLPtoDP(DCDest, core::ptr::addr_of_mut!(DestRect) as LPPOINT, 2);

    DestRect.left += (*DCDest).ptlDCOrig.x;
    DestRect.top += (*DCDest).ptlDCOrig.y;
    DestRect.right += (*DCDest).ptlDCOrig.x;
    DestRect.bottom += (*DCDest).ptlDCOrig.y;

    if (*DCDest).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(DCDest, core::ptr::addr_of_mut!(DestRect));
    }

    SourceRect.left = XOriginSrc;
    SourceRect.top = YOriginSrc;
    SourceRect.right = XOriginSrc + WidthSrc;
    SourceRect.bottom = YOriginSrc + HeightSrc;
    w32r_IntLPtoDP(DCSrc, core::ptr::addr_of_mut!(SourceRect) as LPPOINT, 2);

    SourceRect.left += (*DCSrc).ptlDCOrig.x;
    SourceRect.top += (*DCSrc).ptlDCOrig.y;
    SourceRect.right += (*DCSrc).ptlDCOrig.x;
    SourceRect.bottom += (*DCSrc).ptlDCOrig.y;

    if DestRect.right == 0
        || DestRect.bottom == 0
        || SourceRect.right == 0
        || SourceRect.bottom == 0
    {
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
        return 1;
    }

    DC_vPrepareDCsForBlit(
        DCDest,
        core::ptr::addr_of_mut!(DestRect),
        DCSrc,
        core::ptr::addr_of_mut!(SourceRect),
    );

    'leave: {
        BitmapDest = (*DCDest).dclevel.pSurface;
        if BitmapDest.is_null() {
            bResult = 0;
            break 'leave;
        }

        BitmapSrc = (*DCSrc).dclevel.pSurface;
        if BitmapSrc.is_null() {
            bResult = 0;
            break 'leave;
        }

        EXLATEOBJ_vInitXlateFromDCs(core::ptr::addr_of_mut!(exlo), DCSrc, DCDest);

        bResult = IntEngAlphaBlend(
            core::ptr::addr_of_mut!((*BitmapDest).SurfObj),
            core::ptr::addr_of_mut!((*BitmapSrc).SurfObj),
            core::ptr::addr_of_mut!((*DCDest).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!(exlo.xlo),
            core::ptr::addr_of_mut!(DestRect),
            core::ptr::addr_of_mut!(SourceRect),
            core::ptr::addr_of_mut!(BlendObj),
        );

        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
    }

    DC_vFinishBlit(DCDest, DCSrc);
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));

    bResult
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiAlphaBlend(
    hDCDest: HDC,
    XOriginDest: LONG,
    YOriginDest: LONG,
    WidthDest: LONG,
    HeightDest: LONG,
    hDCSrc: HDC,
    XOriginSrc: LONG,
    YOriginSrc: LONG,
    WidthSrc: LONG,
    HeightSrc: LONG,
    BlendFunc: BLENDFUNCTION,
    _hcmXform: HANDLE,
) -> BOOL {
    let DCDest: PDC;
    let DCSrc: PDC;
    let mut ahDC: [HDC; 2] = [core::ptr::null_mut(); 2];
    let mut apObj: [PGDIOBJ; 2] = [core::ptr::null_mut(); 2];
    let BitmapDest: *mut SURFACE;
    let BitmapSrc: *mut SURFACE;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut SourceRect: RECTL = core::mem::zeroed();
    let mut bResult: BOOL;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut BlendObj: BLENDOBJ = core::mem::zeroed();
    BlendObj.BlendFunction = BlendFunc;

    if WidthDest < 0 || HeightDest < 0 || WidthSrc < 0 || HeightSrc < 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if hDCDest.is_null() || hDCSrc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    ahDC[0] = hDCDest;
    ahDC[1] = hDCSrc;
    if GDIOBJ_bLockMultipleObjects(
        2,
        ahDC.as_mut_ptr() as *mut HGDIOBJ,
        apObj.as_mut_ptr(),
        GDIObjType_DC_TYPE as _ ,
    ) == 0
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    DCDest = apObj[0] as PDC;
    DCSrc = apObj[1] as PDC;

    if (*DCSrc).dctype == (DCTYPE_INFO as i32 as _) || (*DCDest).dctype == (DCTYPE_INFO as i32 as _) {
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
        return 1;
    }

    DestRect.left = XOriginDest;
    DestRect.top = YOriginDest;
    DestRect.right = XOriginDest + WidthDest;
    DestRect.bottom = YOriginDest + HeightDest;
    w32r_IntLPtoDP(DCDest, core::ptr::addr_of_mut!(DestRect) as LPPOINT, 2);

    DestRect.left += (*DCDest).ptlDCOrig.x;
    DestRect.top += (*DCDest).ptlDCOrig.y;
    DestRect.right += (*DCDest).ptlDCOrig.x;
    DestRect.bottom += (*DCDest).ptlDCOrig.y;

    if (*DCDest).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(DCDest, core::ptr::addr_of_mut!(DestRect));
    }

    SourceRect.left = XOriginSrc;
    SourceRect.top = YOriginSrc;
    SourceRect.right = XOriginSrc + WidthSrc;
    SourceRect.bottom = YOriginSrc + HeightSrc;
    w32r_IntLPtoDP(DCSrc, core::ptr::addr_of_mut!(SourceRect) as LPPOINT, 2);

    SourceRect.left += (*DCSrc).ptlDCOrig.x;
    SourceRect.top += (*DCSrc).ptlDCOrig.y;
    SourceRect.right += (*DCSrc).ptlDCOrig.x;
    SourceRect.bottom += (*DCSrc).ptlDCOrig.y;

    if DestRect.right == 0
        || DestRect.bottom == 0
        || SourceRect.right == 0
        || SourceRect.bottom == 0
    {
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
        return 1;
    }

    DC_vPrepareDCsForBlit(
        DCDest,
        core::ptr::addr_of_mut!(DestRect),
        DCSrc,
        core::ptr::addr_of_mut!(SourceRect),
    );

    'leave: {
        BitmapDest = (*DCDest).dclevel.pSurface;
        if BitmapDest.is_null() {
            bResult = 0;
            break 'leave;
        }

        BitmapSrc = (*DCSrc).dclevel.pSurface;
        if BitmapSrc.is_null() {
            bResult = 0;
            break 'leave;
        }

        EXLATEOBJ_vInitXlateFromDCs(core::ptr::addr_of_mut!(exlo), DCSrc, DCDest);

        bResult = IntEngAlphaBlend(
            core::ptr::addr_of_mut!((*BitmapDest).SurfObj),
            core::ptr::addr_of_mut!((*BitmapSrc).SurfObj),
            core::ptr::addr_of_mut!((*DCDest).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!(exlo.xlo),
            core::ptr::addr_of_mut!(DestRect),
            core::ptr::addr_of_mut!(SourceRect),
            core::ptr::addr_of_mut!(BlendObj),
        );

        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
    }

    DC_vFinishBlit(DCDest, DCSrc);
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiBitBlt(
    hDCDest: HDC,
    XDest: INT,
    YDest: INT,
    Width: INT,
    Height: INT,
    hDCSrc: HDC,
    XSrc: INT,
    YSrc: INT,
    mut dwRop: DWORD,
    crBackColor: DWORD,
    fl: FLONG,
) -> BOOL {
    if dwRop & CAPTUREBLT != 0 {
        return NtGdiStretchBlt(
            hDCDest,
            XDest,
            YDest,
            Width,
            Height,
            hDCSrc,
            XSrc,
            YSrc,
            Width,
            Height,
            dwRop,
            crBackColor,
        );
    }

    dwRop = dwRop & !(NOMIRRORBITMAP | CAPTUREBLT);

    NtGdiMaskBlt(
        hDCDest,
        XDest,
        YDest,
        Width,
        Height,
        hDCSrc,
        XSrc,
        YSrc,
        core::ptr::null_mut(),
        0,
        0,
        MAKEROP4(dwRop, dwRop),
        crBackColor,
    )
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiTransparentBlt(
    hdcDst: HDC,
    xDst: INT,
    yDst: INT,
    cxDst: INT,
    cyDst: INT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    cxSrc: INT,
    cySrc: INT,
    TransColor: COLORREF,
) -> BOOL {
    let DCDest: PDC;
    let DCSrc: PDC;
    let mut ahDC: [HDC; 2] = [core::ptr::null_mut(); 2];
    let mut apObj: [PGDIOBJ; 2] = [core::ptr::null_mut(); 2];
    let mut rcDest: RECTL = core::mem::zeroed();
    let mut rcSrc: RECTL = core::mem::zeroed();
    let BitmapDest: *mut SURFACE;
    let mut BitmapSrc: *mut SURFACE = core::ptr::null_mut();
    let mut TransparentColor: ULONG = 0;
    let mut Ret: BOOL = 0u8 as BOOL;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();

    if hdcDst.is_null() || hdcSrc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0u8 as BOOL;
    }

    ahDC[0] = hdcDst;
    ahDC[1] = hdcSrc;
    if GDIOBJ_bLockMultipleObjects(2, ahDC.as_mut_ptr() as *mut HGDIOBJ, apObj.as_mut_ptr(), (GDIObjType_DC_TYPE as _)) == 0 {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0u8 as BOOL;
    }
    DCDest = apObj[0] as PDC;
    DCSrc = apObj[1] as PDC;

    if (*DCSrc).dctype == DCTYPE_INFO as _ || (*DCDest).dctype == DCTYPE_INFO as _ {
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
        return 1u8 as BOOL;
    }

    rcDest.left = xDst;
    rcDest.top = yDst;
    rcDest.right = rcDest.left + cxDst;
    rcDest.bottom = rcDest.top + cyDst;
    w32r_IntLPtoDP(DCDest, core::ptr::addr_of_mut!(rcDest) as LPPOINT, 2);

    rcDest.left += (*DCDest).ptlDCOrig.x;
    rcDest.top += (*DCDest).ptlDCOrig.y;
    rcDest.right += (*DCDest).ptlDCOrig.x;
    rcDest.bottom += (*DCDest).ptlDCOrig.y;

    rcSrc.left = xSrc;
    rcSrc.top = ySrc;
    rcSrc.right = rcSrc.left + cxSrc;
    rcSrc.bottom = rcSrc.top + cySrc;
    w32r_IntLPtoDP(DCSrc, core::ptr::addr_of_mut!(rcSrc) as LPPOINT, 2);

    rcSrc.left += (*DCSrc).ptlDCOrig.x;
    rcSrc.top += (*DCSrc).ptlDCOrig.y;
    rcSrc.right += (*DCSrc).ptlDCOrig.x;
    rcSrc.bottom += (*DCSrc).ptlDCOrig.y;

    if (*DCDest).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(DCDest, core::ptr::addr_of_mut!(rcDest));
    }

    DC_vPrepareDCsForBlit(DCDest, core::ptr::addr_of_mut!(rcDest), DCSrc, core::ptr::addr_of_mut!(rcSrc));

    'done: {
        BitmapDest = (*DCDest).dclevel.pSurface;
        if BitmapDest.is_null() {
            break 'done;
        }

        BitmapSrc = (*DCSrc).dclevel.pSurface;
        if BitmapSrc.is_null() {
            break 'done;
        }

        EXLATEOBJ_vInitialize(
            core::ptr::addr_of_mut!(exlo),
            core::ptr::addr_of_mut!(gpalRGB),
            (*BitmapSrc).ppal,
            0,
            0,
            0,
        );
        TransparentColor = XLATEOBJ_iXlate(core::ptr::addr_of_mut!(exlo.xlo), TransColor as ULONG);
        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));

        EXLATEOBJ_vInitXlateFromDCs(core::ptr::addr_of_mut!(exlo), DCSrc, DCDest);

        Ret = IntEngTransparentBlt(
            core::ptr::addr_of_mut!((*BitmapDest).SurfObj),
            core::ptr::addr_of_mut!((*BitmapSrc).SurfObj),
            core::ptr::addr_of_mut!((*DCDest).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!(exlo.xlo),
            core::ptr::addr_of_mut!(rcDest),
            core::ptr::addr_of_mut!(rcSrc),
            TransparentColor,
            0,
        );

        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
    }

    DC_vFinishBlit(DCDest, DCSrc);
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiTransparentBlt(
    hdcDst: HDC,
    xDst: INT,
    yDst: INT,
    cxDst: INT,
    cyDst: INT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    cxSrc: INT,
    cySrc: INT,
    TransColor: COLORREF,
) -> BOOL {
    let DCDest: PDC;
    let DCSrc: PDC;
    let mut ahDC: [HDC; 2] = [core::ptr::null_mut(); 2];
    let mut apObj: [PGDIOBJ; 2] = [core::ptr::null_mut(); 2];
    let mut rcDest: RECTL = core::mem::zeroed();
    let mut rcSrc: RECTL = core::mem::zeroed();
    let BitmapDest: *mut SURFACE;
    let mut BitmapSrc: *mut SURFACE = core::ptr::null_mut();
    let mut TransparentColor: ULONG = 0;
    let mut Ret: BOOL = 0u8 as BOOL;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();

    if hdcDst.is_null() || hdcSrc.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0u8 as BOOL;
    }

    ahDC[0] = hdcDst;
    ahDC[1] = hdcSrc;
    if GDIOBJ_bLockMultipleObjects(2, ahDC.as_mut_ptr() as *mut HGDIOBJ, apObj.as_mut_ptr(), GDIObjType_DC_TYPE as _ ) == 0 {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0u8 as BOOL;
    }
    DCDest = apObj[0] as PDC;
    DCSrc = apObj[1] as PDC;

    if (*DCSrc).dctype == DCTYPE_INFO as _ || (*DCDest).dctype == DCTYPE_INFO as _ {
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
        return 1u8 as BOOL;
    }

    rcDest.left = xDst;
    rcDest.top = yDst;
    rcDest.right = rcDest.left + cxDst;
    rcDest.bottom = rcDest.top + cyDst;
    w32r_IntLPtoDP(DCDest, core::ptr::addr_of_mut!(rcDest) as LPPOINT, 2);

    rcDest.left += (*DCDest).ptlDCOrig.x;
    rcDest.top += (*DCDest).ptlDCOrig.y;
    rcDest.right += (*DCDest).ptlDCOrig.x;
    rcDest.bottom += (*DCDest).ptlDCOrig.y;

    rcSrc.left = xSrc;
    rcSrc.top = ySrc;
    rcSrc.right = rcSrc.left + cxSrc;
    rcSrc.bottom = rcSrc.top + cySrc;
    w32r_IntLPtoDP(DCSrc, core::ptr::addr_of_mut!(rcSrc) as LPPOINT, 2);

    rcSrc.left += (*DCSrc).ptlDCOrig.x;
    rcSrc.top += (*DCSrc).ptlDCOrig.y;
    rcSrc.right += (*DCSrc).ptlDCOrig.x;
    rcSrc.bottom += (*DCSrc).ptlDCOrig.y;

    if (*DCDest).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(DCDest, core::ptr::addr_of_mut!(rcDest));
    }

    DC_vPrepareDCsForBlit(DCDest, core::ptr::addr_of_mut!(rcDest), DCSrc, core::ptr::addr_of_mut!(rcSrc));

    'done: {
        BitmapDest = (*DCDest).dclevel.pSurface;
        if BitmapDest.is_null() {
            break 'done;
        }

        BitmapSrc = (*DCSrc).dclevel.pSurface;
        if BitmapSrc.is_null() {
            break 'done;
        }

        EXLATEOBJ_vInitialize(
            core::ptr::addr_of_mut!(exlo),
            core::ptr::addr_of_mut!(gpalRGB),
            (*BitmapSrc).ppal,
            0,
            0,
            0,
        );
        TransparentColor = XLATEOBJ_iXlate(core::ptr::addr_of_mut!(exlo.xlo), TransColor as ULONG);
        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));

        EXLATEOBJ_vInitXlateFromDCs(core::ptr::addr_of_mut!(exlo), DCSrc, DCDest);

        Ret = IntEngTransparentBlt(
            core::ptr::addr_of_mut!((*BitmapDest).SurfObj),
            core::ptr::addr_of_mut!((*BitmapSrc).SurfObj),
            core::ptr::addr_of_mut!((*DCDest).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!(exlo.xlo),
            core::ptr::addr_of_mut!(rcDest),
            core::ptr::addr_of_mut!(rcSrc),
            TransparentColor,
            0,
        );

        EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
    }

    DC_vFinishBlit(DCDest, DCSrc);
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject));
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject));

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiMaskBlt(
    hdcDest: HDC,
    nXDest: INT,
    nYDest: INT,
    mut nWidth: INT,
    mut nHeight: INT,
    hdcSrc: HDC,
    nXSrc: INT,
    nYSrc: INT,
    hbmMask: HBITMAP,
    xMask: INT,
    yMask: INT,
    dwRop4: DWORD,
    crBackColor: DWORD,
) -> BOOL {

    let DCDest: PDC;
    let mut DCSrc: PDC = core::ptr::null_mut();
    let mut ahDC: [HDC; 2] = [core::ptr::null_mut(); 2];
    let mut apObj: [PGDIOBJ; 2] = [core::ptr::null_mut(); 2];
    let pdcattr: PDC_ATTR;
    let BitmapDest: *mut SURFACE;
    let mut BitmapSrc: *mut SURFACE = core::ptr::null_mut();
    let mut psurfMask: *mut SURFACE;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut SourceRect: RECTL = core::mem::zeroed();
    let mut SourcePoint: POINTL = core::mem::zeroed();
    let mut MaskPoint: POINTL = core::mem::zeroed();
    let mut Status: BOOL = 0;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut XlateObj: *mut XLATEOBJ = core::ptr::null_mut();
    let UsesSource: BOOL;
    let UsesPattern: BOOL;
    let rop4: ROP4;

    rop4 = WIN32_ROP4_TO_ENG_ROP4(dwRop4);

    if hdcDest.is_null() {
        return 0;
    }

    UsesSource = ROP4_USES_SOURCE(rop4);
    UsesPattern = ROP4_USES_PATTERN(rop4);
    if hdcSrc.is_null() && (UsesSource != 0 || UsesPattern != 0) {
        return 0;
    }

    if ROP4_USES_MASK(rop4) != 0 && !hbmMask.is_null() {
        psurfMask = SURFACE_ShareLockSurface((hbmMask as _));
        if psurfMask.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }

        // C indexes gajBitsPerFormat[] unchecked (an OOB read on a garbage
        // iBitmapFormat is UB there but in practice rejects); in Rust that
        // panics the kernel — bounds-check explicitly and reject like != 1.
        let fmt = (*psurfMask).SurfObj.iBitmapFormat as usize;
        if fmt >= gajBitsPerFormat.len() || gajBitsPerFormat[fmt] != 1 {
            SURFACE_ShareUnlockSurface(psurfMask);
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }
    } else {
        psurfMask = core::ptr::null_mut();
    }

    MaskPoint.x = xMask;
    MaskPoint.y = yMask;

    ahDC[0] = hdcDest;
    ahDC[1] = if UsesSource != 0 { hdcSrc } else { core::ptr::null_mut() };
    if GDIOBJ_bLockMultipleObjects(2, ahDC.as_mut_ptr() as *mut HGDIOBJ, apObj.as_mut_ptr(), (GDIObjType_DC_TYPE as _)) == 0 {
        if !psurfMask.is_null() {
            SURFACE_ShareUnlockSurface(psurfMask);
        }
        return 0;
    }
    DCDest = apObj[0] as PDC;
    DCSrc = apObj[1] as PDC;

    if DCDest.is_null() {
        if !DCSrc.is_null() {
            DC_UnlockDc(DCSrc);
        }
        if !psurfMask.is_null() {
            SURFACE_ShareUnlockSurface(psurfMask);
        }
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*DCDest).dctype == (DCTYPE_INFO as i32 as _) {
        if !DCSrc.is_null() {
            DC_UnlockDc(DCSrc);
        }
        DC_UnlockDc(DCDest);
        if !psurfMask.is_null() {
            SURFACE_ShareUnlockSurface(psurfMask);
        }
        return 1;
    }

    if UsesSource != 0 {
        if (*DCSrc).dctype == (DCTYPE_INFO as i32 as _) {
            DC_UnlockDc(DCDest);
            DC_UnlockDc(DCSrc);
            if !psurfMask.is_null() {
                SURFACE_ShareUnlockSurface(psurfMask);
            }
            return 1;
        }
    }

    pdcattr = (*DCDest).pdcattr;

    DestRect.left = nXDest;
    DestRect.top = nYDest;
    DestRect.right = nXDest + nWidth;
    DestRect.bottom = nYDest + nHeight;
    w32r_IntLPtoDP(DCDest, core::ptr::addr_of_mut!(DestRect) as LPPOINT, 2);

    DestRect.left += (*DCDest).ptlDCOrig.x;
    DestRect.top += (*DCDest).ptlDCOrig.y;
    DestRect.right += (*DCDest).ptlDCOrig.x;
    DestRect.bottom += (*DCDest).ptlDCOrig.y;

    if (*DCDest).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(DCDest, &mut DestRect);
    }

    SourcePoint.x = nXSrc;
    SourcePoint.y = nYSrc;

    if UsesSource != 0 {
        w32r_IntLPtoDP(DCSrc, core::ptr::addr_of_mut!(SourcePoint) as LPPOINT, 1);

        SourcePoint.x += (*DCSrc).ptlDCOrig.x;
        SourcePoint.y += (*DCSrc).ptlDCOrig.y;
        SourceRect.left = SourcePoint.x;
        SourceRect.top = SourcePoint.y;
        SourceRect.right = SourcePoint.x + DestRect.right - DestRect.left;
        SourceRect.bottom = SourcePoint.y + DestRect.bottom - DestRect.top;
    } else {
        SourceRect.left = 0;
        SourceRect.top = 0;
        SourceRect.right = 0;
        SourceRect.bottom = 0;
    }

    DC_vPrepareDCsForBlit(DCDest, &mut DestRect, DCSrc, &mut SourceRect);

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(DCDest);
    }

    'cleanup: {
        BitmapDest = (*DCDest).dclevel.pSurface;
        if BitmapDest.is_null() {
            break 'cleanup;
        }

        if UsesSource != 0 {
            BitmapSrc = (*DCSrc).dclevel.pSurface;
            if BitmapSrc.is_null() {
                break 'cleanup;
            }

            EXLATEOBJ_vInitXlateFromDCsEx(&mut exlo, DCSrc, DCDest, crBackColor);
            XlateObj = &mut exlo.xlo;
        }

        if DestRect.left > DestRect.right && nWidth < 0 {
            SourcePoint.x += nWidth;
            nWidth = -nWidth;
        }

        if DestRect.top > DestRect.bottom && nHeight < 0 {
            SourcePoint.y += nHeight;
            nHeight = -nHeight;
        }

        RECTL_vMakeWellOrdered(&mut DestRect);

        Status = IntEngBitBlt(
            &mut (*BitmapDest).SurfObj,
            if !BitmapSrc.is_null() { &mut (*BitmapSrc).SurfObj } else { core::ptr::null_mut() },
            if !psurfMask.is_null() { &mut (*psurfMask).SurfObj } else { core::ptr::null_mut() },
            core::ptr::addr_of_mut!((*DCDest).co) as *mut CLIPOBJ,
            XlateObj,
            &mut DestRect,
            &mut SourcePoint,
            &mut MaskPoint,
            &mut (*DCDest).eboFill.BrushObject,
            &mut (*pbrushbody((*DCDest).dclevel.pbrFill)).ptOrigin,
            rop4,
        );

        if UsesSource != 0 {
            EXLATEOBJ_vCleanup(&mut exlo);
        }
    }

    DC_vFinishBlit(DCDest, DCSrc);
    if UsesSource != 0 {
        DC_UnlockDc(DCSrc);
    }
    DC_UnlockDc(DCDest);
    if !psurfMask.is_null() {
        SURFACE_ShareUnlockSurface(psurfMask);
    }

    if Status == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
    }

    Status
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiMaskBlt(
    hdcDest: HDC,
    nXDest: INT,
    nYDest: INT,
    mut nWidth: INT,
    mut nHeight: INT,
    hdcSrc: HDC,
    nXSrc: INT,
    nYSrc: INT,
    hbmMask: HBITMAP,
    xMask: INT,
    yMask: INT,
    dwRop4: DWORD,
    crBackColor: DWORD,
) -> BOOL {

    let DCDest: PDC;
    let mut DCSrc: PDC = core::ptr::null_mut();
    let mut ahDC: [HDC; 2] = [core::ptr::null_mut(); 2];
    let mut apObj: [PGDIOBJ; 2] = [core::ptr::null_mut(); 2];
    let pdcattr: PDC_ATTR;
    let BitmapDest: *mut SURFACE;
    let mut BitmapSrc: *mut SURFACE = core::ptr::null_mut();
    let mut psurfMask: *mut SURFACE;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut SourceRect: RECTL = core::mem::zeroed();
    let mut SourcePoint: POINTL = core::mem::zeroed();
    let mut MaskPoint: POINTL = core::mem::zeroed();
    let mut Status: BOOL = 0;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut XlateObj: *mut XLATEOBJ = core::ptr::null_mut();
    let UsesSource: BOOL;
    let UsesPattern: BOOL;
    let rop4: ROP4;

    rop4 = WIN32_ROP4_TO_ENG_ROP4(dwRop4);

    if hdcDest.is_null() {
        return 0;
    }

    UsesSource = ROP4_USES_SOURCE(rop4);
    UsesPattern = ROP4_USES_PATTERN(rop4);
    if hdcSrc.is_null() && (UsesSource != 0 || UsesPattern != 0) {
        return 0;
    }

    if ROP4_USES_MASK(rop4) != 0 && !hbmMask.is_null() {
        psurfMask = SURFACE_ShareLockSurface(hbmMask as _ );
        if psurfMask.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }

        // C indexes gajBitsPerFormat[] unchecked (an OOB read on a garbage
        // iBitmapFormat is UB there but in practice rejects); in Rust that
        // panics the kernel — bounds-check explicitly and reject like != 1.
        let fmt = (*psurfMask).SurfObj.iBitmapFormat as usize;
        if fmt >= gajBitsPerFormat.len() || gajBitsPerFormat[fmt] != 1 {
            SURFACE_ShareUnlockSurface(psurfMask);
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }
    } else {
        psurfMask = core::ptr::null_mut();
    }

    MaskPoint.x = xMask;
    MaskPoint.y = yMask;

    ahDC[0] = hdcDest;
    ahDC[1] = if UsesSource != 0 { hdcSrc } else { core::ptr::null_mut() };
    if GDIOBJ_bLockMultipleObjects(2, ahDC.as_mut_ptr() as *mut HGDIOBJ, apObj.as_mut_ptr(), GDIObjType_DC_TYPE as _ ) == 0 {
        if !psurfMask.is_null() {
            SURFACE_ShareUnlockSurface(psurfMask);
        }
        return 0;
    }
    DCDest = apObj[0] as PDC;
    DCSrc = apObj[1] as PDC;

    if DCDest.is_null() {
        if !DCSrc.is_null() {
            DC_UnlockDc(DCSrc);
        }
        if !psurfMask.is_null() {
            SURFACE_ShareUnlockSurface(psurfMask);
        }
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*DCDest).dctype == (DCTYPE_INFO as i32 as _) {
        if !DCSrc.is_null() {
            DC_UnlockDc(DCSrc);
        }
        DC_UnlockDc(DCDest);
        if !psurfMask.is_null() {
            SURFACE_ShareUnlockSurface(psurfMask);
        }
        return 1;
    }

    if UsesSource != 0 {
        if (*DCSrc).dctype == (DCTYPE_INFO as i32 as _) {
            DC_UnlockDc(DCDest);
            DC_UnlockDc(DCSrc);
            if !psurfMask.is_null() {
                SURFACE_ShareUnlockSurface(psurfMask);
            }
            return 1;
        }
    }

    pdcattr = (*DCDest).pdcattr;

    DestRect.left = nXDest;
    DestRect.top = nYDest;
    DestRect.right = nXDest + nWidth;
    DestRect.bottom = nYDest + nHeight;
    w32r_IntLPtoDP(DCDest, core::ptr::addr_of_mut!(DestRect) as LPPOINT, 2);

    DestRect.left += (*DCDest).ptlDCOrig.x;
    DestRect.top += (*DCDest).ptlDCOrig.y;
    DestRect.right += (*DCDest).ptlDCOrig.x;
    DestRect.bottom += (*DCDest).ptlDCOrig.y;

    if (*DCDest).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(DCDest, &mut DestRect);
    }

    SourcePoint.x = nXSrc;
    SourcePoint.y = nYSrc;

    if UsesSource != 0 {
        w32r_IntLPtoDP(DCSrc, core::ptr::addr_of_mut!(SourcePoint) as LPPOINT, 1);

        SourcePoint.x += (*DCSrc).ptlDCOrig.x;
        SourcePoint.y += (*DCSrc).ptlDCOrig.y;
        SourceRect.left = SourcePoint.x;
        SourceRect.top = SourcePoint.y;
        SourceRect.right = SourcePoint.x + DestRect.right - DestRect.left;
        SourceRect.bottom = SourcePoint.y + DestRect.bottom - DestRect.top;
    } else {
        SourceRect.left = 0;
        SourceRect.top = 0;
        SourceRect.right = 0;
        SourceRect.bottom = 0;
    }

    DC_vPrepareDCsForBlit(DCDest, &mut DestRect, DCSrc, &mut SourceRect);

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(DCDest);
    }

    'cleanup: {
        BitmapDest = (*DCDest).dclevel.pSurface;
        if BitmapDest.is_null() {
            break 'cleanup;
        }

        if UsesSource != 0 {
            BitmapSrc = (*DCSrc).dclevel.pSurface;
            if BitmapSrc.is_null() {
                break 'cleanup;
            }

            EXLATEOBJ_vInitXlateFromDCsEx(&mut exlo, DCSrc, DCDest, crBackColor);
            XlateObj = &mut exlo.xlo;
        }

        if DestRect.left > DestRect.right && nWidth < 0 {
            SourcePoint.x += nWidth;
            nWidth = -nWidth;
        }

        if DestRect.top > DestRect.bottom && nHeight < 0 {
            SourcePoint.y += nHeight;
            nHeight = -nHeight;
        }

        RECTL_vMakeWellOrdered(&mut DestRect);

        Status = IntEngBitBlt(
            &mut (*BitmapDest).SurfObj,
            if !BitmapSrc.is_null() { &mut (*BitmapSrc).SurfObj } else { core::ptr::null_mut() },
            if !psurfMask.is_null() { &mut (*psurfMask).SurfObj } else { core::ptr::null_mut() },
            core::ptr::addr_of_mut!((*DCDest).co) as *mut CLIPOBJ,
            XlateObj,
            &mut DestRect,
            &mut SourcePoint,
            &mut MaskPoint,
            &mut (*DCDest).eboFill.BrushObject,
            &mut (*pbrushbody((*DCDest).dclevel.pbrFill)).ptOrigin,
            rop4,
        );

        if UsesSource != 0 {
            EXLATEOBJ_vCleanup(&mut exlo);
        }
    }

    DC_vFinishBlit(DCDest, DCSrc);
    if UsesSource != 0 {
        DC_UnlockDc(DCSrc);
    }
    DC_UnlockDc(DCDest);
    if !psurfMask.is_null() {
        SURFACE_ShareUnlockSurface(psurfMask);
    }

    if Status == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPlgBlt(
    hdcTrg: HDC,
    pptlTrg: LPPOINT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    cxSrc: INT,
    cySrc: INT,
    hbmMask: HBITMAP,
    xMask: INT,
    yMask: INT,
    crBackColor: DWORD,
) -> BOOL {
    FIXME(b"NtGdiPlgBlt: unimplemented.\n\0".as_ptr() as *const i8);
    0
}

#[no_mangle]
pub unsafe extern "system" fn GreStretchBltMask(
    hDCDest: HDC,
    XOriginDest: INT,
    YOriginDest: INT,
    WidthDest: INT,
    HeightDest: INT,
    hDCSrc: HDC,
    XOriginSrc: INT,
    YOriginSrc: INT,
    WidthSrc: INT,
    HeightSrc: INT,
    dwRop4: DWORD,
    dwBackColor: DWORD,
    hDCMask: HDC,
    XOriginMask: INT,
    YOriginMask: INT,
) -> BOOL {

    let mut DCSrc: PDC = core::ptr::null_mut();
    let mut DCMask: PDC = core::ptr::null_mut();
    let mut ahDC: [HDC; 3] = [core::ptr::null_mut(); 3];
    let mut apObj: [PGDIOBJ; 3] = [core::ptr::null_mut(); 3];
    let mut BitmapSrc: *mut SURFACE = core::ptr::null_mut();
    let mut BitmapMask: *mut SURFACE = core::ptr::null_mut();
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut SourceRect: RECTL = core::mem::zeroed();
    let mut MaskPoint: POINTL = core::mem::zeroed();
    let mut Status: BOOL = 0;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut XlateObj: *mut XLATEOBJ = core::ptr::null_mut();
    let mut BrushOrigin: POINTL = core::mem::zeroed();

    let rop4: ROP4 = WIN32_ROP4_TO_ENG_ROP4(dwRop4);

    let UsesSource: BOOL = ROP4_USES_SOURCE(rop4);
    let UsesMask: BOOL = ROP4_USES_MASK(rop4);

    if 0 == WidthDest || 0 == HeightDest || 0 == WidthSrc || 0 == HeightSrc {
        EngSetLastError(ERROR_INVALID_PARAMETER as _);
        return 1;
    }

    if hDCDest.is_null()
        || (UsesSource != 0 && hDCSrc.is_null())
        || (UsesMask != 0 && hDCMask.is_null())
    {
        EngSetLastError(ERROR_INVALID_PARAMETER as _);
        return 0;
    }

    ahDC[0] = hDCDest;
    ahDC[1] = if UsesSource != 0 { hDCSrc } else { core::ptr::null_mut() };
    ahDC[2] = if UsesMask != 0 { hDCMask } else { core::ptr::null_mut() };
    if GDIOBJ_bLockMultipleObjects(
        3,
        ahDC.as_mut_ptr() as *mut HGDIOBJ,
        apObj.as_mut_ptr(),
        GDIObjType_DC_TYPE as _ ,
    ) == 0
    {
        EngSetLastError(ERROR_INVALID_HANDLE as _);
        return 0;
    }
    let DCDest: PDC = apObj[0] as PDC;
    DCSrc = apObj[1] as PDC;
    DCMask = apObj[2] as PDC;

    if (*DCDest).dctype == DCTYPE_INFO as _ {
        if !DCSrc.is_null() {
            GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject) as PGDIOBJ as _ );
        }
        if !DCMask.is_null() {
            GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCMask).BaseObject) as PGDIOBJ as _ );
        }
        GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject) as PGDIOBJ as _ );
        return 1;
    }

    if UsesSource != 0 {
        if (*DCSrc).dctype == DCTYPE_INFO as _ {
            GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCDest).BaseObject) as PGDIOBJ as _ );
            GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCSrc).BaseObject) as PGDIOBJ as _ );
            if !DCMask.is_null() {
                GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*DCMask).BaseObject) as PGDIOBJ as _ );
            }
            return 1;
        }
    }

    let Case0000 = (WidthDest < 0) && (HeightDest < 0) && (WidthSrc < 0) && (HeightSrc < 0);
    let Case0101 = (WidthDest < 0) && (HeightDest > 0) && (WidthSrc < 0) && (HeightSrc > 0);
    let Case1010 = (WidthDest > 0) && (HeightDest < 0) && (WidthSrc > 0) && (HeightSrc < 0);
    let CaseExcept = Case0000 || Case0101 || Case1010;

    let pdcattr: PDC_ATTR = (*DCDest).pdcattr;

    DestRect.left = XOriginDest;
    DestRect.top = YOriginDest;
    DestRect.right = XOriginDest + WidthDest;
    DestRect.bottom = YOriginDest + HeightDest;

    if (WidthDest < 0) && !CaseExcept {
        DestRect.left += 1;
        DestRect.right += 1;
    }
    if (HeightDest < 0) && !CaseExcept {
        DestRect.top += 1;
        DestRect.bottom += 1;
    }

    w32r_IntLPtoDP(DCDest, core::ptr::addr_of_mut!(DestRect) as LPPOINT, 2);

    DestRect.left += (*DCDest).ptlDCOrig.x;
    DestRect.top += (*DCDest).ptlDCOrig.y;
    DestRect.right += (*DCDest).ptlDCOrig.x;
    DestRect.bottom += (*DCDest).ptlDCOrig.y;

    if (*DCDest).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(DCDest, core::ptr::addr_of_mut!(DestRect));
    }

    SourceRect.left = XOriginSrc;
    SourceRect.top = YOriginSrc;
    SourceRect.right = XOriginSrc + WidthSrc;
    SourceRect.bottom = YOriginSrc + HeightSrc;

    if (WidthSrc < 0) && !CaseExcept {
        SourceRect.left += 1;
        SourceRect.right += 1;
    }
    if (HeightSrc < 0) && !CaseExcept {
        SourceRect.top += 1;
        SourceRect.bottom += 1;
    }

    if UsesSource != 0 {
        w32r_IntLPtoDP(DCSrc, core::ptr::addr_of_mut!(SourceRect) as LPPOINT, 2);

        SourceRect.left += (*DCSrc).ptlDCOrig.x;
        SourceRect.top += (*DCSrc).ptlDCOrig.y;
        SourceRect.right += (*DCSrc).ptlDCOrig.x;
        SourceRect.bottom += (*DCSrc).ptlDCOrig.y;
    }

    BrushOrigin.x = 0;
    BrushOrigin.y = 0;

    DC_vPrepareDCsForBlit(DCDest, core::ptr::addr_of_mut!(DestRect), DCSrc, core::ptr::addr_of_mut!(SourceRect));

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(DCDest);
    }

    let BitmapDest: *mut SURFACE = (*DCDest).dclevel.pSurface;

    'failed: {
        if BitmapDest.is_null() {
            break 'failed;
        }
        if UsesSource != 0 {
            BitmapSrc = (*DCSrc).dclevel.pSurface;
            if BitmapSrc.is_null() {
                break 'failed;
            }

            EXLATEOBJ_vInitXlateFromDCsEx(core::ptr::addr_of_mut!(exlo), DCSrc, DCDest, dwBackColor);
            XlateObj = core::ptr::addr_of_mut!(exlo.xlo);
        }

        BrushOrigin.x += (*DCDest).ptlDCOrig.x;
        BrushOrigin.y += (*DCDest).ptlDCOrig.y;

        if !BitmapSrc.is_null() && !DCMask.is_null() {
            BitmapMask = (*DCMask).dclevel.pSurface;
            if !BitmapMask.is_null()
                && ((*BitmapMask).SurfObj.sizlBitmap.cx < WidthSrc
                    || (*BitmapMask).SurfObj.sizlBitmap.cy < HeightSrc)
            {
                EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
                break 'failed;
            }
            MaskPoint.x = XOriginMask;
            MaskPoint.y = YOriginMask;
            w32r_IntLPtoDP(DCMask, core::ptr::addr_of_mut!(MaskPoint) as _ , 1);
            MaskPoint.x += (*DCMask).ptlDCOrig.x;
            MaskPoint.y += (*DCMask).ptlDCOrig.y;
        }

        Status = IntEngStretchBlt(
            core::ptr::addr_of_mut!((*BitmapDest).SurfObj),
            if !BitmapSrc.is_null() {
                core::ptr::addr_of_mut!((*BitmapSrc).SurfObj)
            } else {
                core::ptr::null_mut()
            },
            if !BitmapMask.is_null() {
                core::ptr::addr_of_mut!((*BitmapMask).SurfObj)
            } else {
                core::ptr::null_mut()
            },
            core::ptr::addr_of_mut!((*DCDest).co) as *mut CLIPOBJ,
            XlateObj,
            core::ptr::addr_of_mut!((*DCDest).dclevel.ca),
            core::ptr::addr_of_mut!(DestRect),
            core::ptr::addr_of_mut!(SourceRect),
            if !BitmapMask.is_null() {
                core::ptr::addr_of_mut!(MaskPoint)
            } else {
                core::ptr::null_mut()
            },
            core::ptr::addr_of_mut!((*DCDest).eboFill.BrushObject),
            core::ptr::addr_of_mut!(BrushOrigin),
            rop4,
        );
        if UsesSource != 0 {
            EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
        }
    }

    DC_vFinishBlit(DCDest, DCSrc);
    if UsesSource != 0 {
        DC_UnlockDc(DCSrc);
    }
    if !DCMask.is_null() {
        DC_UnlockDc(DCMask);
    }
    DC_UnlockDc(DCDest);

    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiStretchBlt(
    hDCDest: HDC,
    XOriginDest: INT,
    YOriginDest: INT,
    WidthDest: INT,
    HeightDest: INT,
    hDCSrc: HDC,
    XOriginSrc: INT,
    YOriginSrc: INT,
    WidthSrc: INT,
    HeightSrc: INT,
    mut dwRop3: DWORD,
    dwBackColor: DWORD,
) -> BOOL {

    dwRop3 = dwRop3 & !(NOMIRRORBITMAP | CAPTUREBLT);

    GreStretchBltMask(
        hDCDest,
        XOriginDest,
        YOriginDest,
        WidthDest,
        HeightDest,
        hDCSrc,
        XOriginSrc,
        YOriginSrc,
        WidthSrc,
        HeightSrc,
        MAKEROP4(dwRop3 & 0xFF0000, dwRop3),
        dwBackColor,
        core::ptr::null_mut(),
        0,
        0,
    )
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntPatBlt(
    pdc: PDC,
    XLeft: INT,
    YLeft: INT,
    Width: INT,
    Height: INT,
    dwRop3: DWORD,
    pebo: PEBRUSHOBJ,
) -> BOOL {
    IntPatBlt_impl(pdc, XLeft, YLeft, Width, Height, dwRop3, pebo)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntPatBlt(
    pdc: PDC,
    XLeft: INT,
    YLeft: INT,
    Width: INT,
    Height: INT,
    dwRop3: DWORD,
    pebo: PEBRUSHOBJ,
) -> BOOL {
    IntPatBlt_impl(pdc, XLeft, YLeft, Width, Height, dwRop3, pebo)
}

unsafe fn IntPatBlt_impl(
    pdc: PDC,
    XLeft: INT,
    YLeft: INT,
    Width: INT,
    Height: INT,
    dwRop3: DWORD,
    pebo: PEBRUSHOBJ,
) -> BOOL {

    let mut DestRect: RECTL = core::mem::zeroed();
    let psurf: *mut SURFACE;
    let mut BrushOrigin: POINTL = core::mem::zeroed();
    let ret: BOOL;
    let pbrush: PBRUSH;

    pbrush = (*pebo).pbrush;

    if (*pbrushbody(pbrush)).flAttrs & BR_IS_NULL != 0 {
        return 1;
    }

    if Width >= 0 {
        DestRect.left = XLeft;
        DestRect.right = XLeft + Width;
    } else {
        DestRect.left = XLeft + Width;
        DestRect.right = XLeft;
    }

    if Height >= 0 {
        DestRect.top = YLeft;
        DestRect.bottom = YLeft + Height;
    } else {
        DestRect.top = YLeft + Height;
        DestRect.bottom = YLeft;
    }

    w32r_IntLPtoDP(pdc, core::ptr::addr_of_mut!(DestRect) as LPPOINT, 2);

    DestRect.left += (*pdc).ptlDCOrig.x;
    DestRect.top += (*pdc).ptlDCOrig.y;
    DestRect.right += (*pdc).ptlDCOrig.x;
    DestRect.bottom += (*pdc).ptlDCOrig.y;

    if (*pdc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        IntUpdateBoundsRect(pdc, core::ptr::addr_of_mut!(DestRect));
    }

    BrushOrigin.x = (*pbrushbody(pbrush)).ptOrigin.x + (*pdc).ptlDCOrig.x;
    BrushOrigin.y = (*pbrushbody(pbrush)).ptOrigin.y + (*pdc).ptlDCOrig.y;

    DC_vPrepareDCsForBlit(
        pdc,
        core::ptr::addr_of_mut!(DestRect),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    psurf = (*pdc).dclevel.pSurface;

    ret = IntEngBitBlt(
        core::ptr::addr_of_mut!((*psurf).SurfObj),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pdc).co) as *mut CLIPOBJ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(DestRect),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pebo).BrushObject),
        core::ptr::addr_of_mut!(BrushOrigin),
        WIN32_ROP3_TO_ENG_ROP4(dwRop3),
    );

    DC_vFinishBlit(pdc, core::ptr::null_mut());

    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiPolyPatBlt(
    hDC: HDC,
    dwRop: DWORD,
    pRects: PPATRECT,
    cRects: INT,
    Reserved: ULONG,
) -> BOOL {
    IntGdiPolyPatBlt_impl(hDC, dwRop, pRects, cRects, Reserved)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiPolyPatBlt(
    hDC: HDC,
    dwRop: DWORD,
    pRects: PPATRECT,
    cRects: INT,
    Reserved: ULONG,
) -> BOOL {
    IntGdiPolyPatBlt_impl(hDC, dwRop, pRects, cRects, Reserved)
}

unsafe fn IntGdiPolyPatBlt_impl(
    hDC: HDC,
    dwRop: DWORD,
    mut pRects: PPATRECT,
    cRects: INT,
    _Reserved: ULONG,
) -> BOOL {
    let mut i: INT;
    let mut pbrush: PBRUSH;
    let pdc: PDC;
    let mut eboFill: EBRUSHOBJ = core::mem::zeroed();

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if (*pdc).dctype == DCTYPE_INFO {
        DC_UnlockDc(pdc);
        return 1;
    }

    i = 0;
    while i < cRects {
        pbrush = BRUSH_ShareLockBrush((*pRects).hBrush as _ );

        if !pbrush.is_null() {
            EBRUSHOBJ_vInitFromDC(core::ptr::addr_of_mut!(eboFill), pbrush, pdc as _ );

            IntPatBlt(
                pdc,
                (*pRects).r.left,
                (*pRects).r.top,
                (*pRects).r.right,
                (*pRects).r.bottom,
                dwRop,
                core::ptr::addr_of_mut!(eboFill),
            );

            EBRUSHOBJ_vCleanup(core::ptr::addr_of_mut!(eboFill));
            BRUSH_ShareUnlockBrush(pbrush);
        }
        pRects = pRects.add(1);
        i += 1;
    }

    DC_UnlockDc(pdc);

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPatBlt(
    hdcDest: HDC,
    x: INT,
    y: INT,
    cx: INT,
    cy: INT,
    mut dwRop: DWORD,
) -> BOOL {
    let bResult: BOOL;
    let pdc: PDC;

    dwRop = MAKEROP4(dwRop & 0xFF0000, dwRop);

    if WIN32_ROP4_USES_SOURCE(dwRop) {
        return 0;
    }

    pdc = DC_LockDc(hdcDest);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if (*pdc).dclevel.pSurface.is_null() {
        DC_UnlockDc(pdc);
        return 1;
    }

    if (*(*pdc).pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(pdc);
    }

    bResult = IntPatBlt(pdc, x, y, cx, cy, dwRop, core::ptr::addr_of_mut!((*pdc).eboFill));

    DC_UnlockDc(pdc);
    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPolyPatBlt(
    hDC: HDC,
    dwRop: DWORD,
    pRects: PPOLYPATBLT,
    cRects: DWORD,
    Mode: DWORD,
) -> BOOL {
    let mut rb: PPATRECT = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Ret: BOOL;

    if cRects > 0 {
        rb = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<PATRECT>() * cRects as usize,
            GDITAG_PLGBLT_DATA,
        ) as PPATRECT;
        if rb.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        Status = SafeProbeForRead(
            pRects as *mut core::ffi::c_void,
            cRects as usize * core::mem::size_of::<PATRECT>(),
            1,
        );
        if NT_SUCCESS(Status) {
            RtlCopyMemory(
                rb as *mut core::ffi::c_void,
                pRects as *mut core::ffi::c_void,
                cRects as usize * core::mem::size_of::<PATRECT>(),
            );
        }

        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(rb as *mut core::ffi::c_void, GDITAG_PLGBLT_DATA);
            SetLastNtError(Status);
            return 0;
        }
    }

    Ret = IntGdiPolyPatBlt(hDC, dwRop, rb, cRects as INT, Mode);

    if cRects > 0 {
        ExFreePoolWithTag(rb as *mut core::ffi::c_void, GDITAG_PLGBLT_DATA);
    }

    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn REGION_LPTODP(
    pdc: PDC,
    prgnDest: PREGION,
    prgnSrc: PREGION,
) -> BOOL {
    if IntGdiCombineRgn(prgnDest, prgnSrc, core::ptr::null_mut(), RGN_COPY as _) == ERROR as _ {
        return 0;
    }

    REGION_bXformRgn(prgnDest, w32r_DC_pmxWorldToDevice(pdc))
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn REGION_LPTODP(
    pdc: PDC,
    prgnDest: PREGION,
    prgnSrc: PREGION,
) -> BOOL {
    if IntGdiCombineRgn(prgnDest, prgnSrc, core::ptr::null_mut(), RGN_COPY as _) == ERROR as _ {
        return 0;
    }

    REGION_bXformRgn(prgnDest, w32r_DC_pmxWorldToDevice(pdc))
}

#[no_mangle]
pub unsafe extern "system" fn IntGdiBitBltRgn(
    pdc: PDC,
    prgn: PREGION,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    rop4: ROP4,
) -> BOOL {
    let prgnClip: PREGION;
    let mut xcoClip: XCLIPOBJ = core::mem::zeroed();
    let bResult: BOOL;

    /* Check if we have a surface */
    if (*pdc).dclevel.pSurface.is_null() {
        return 1;
    }

    /* Create an empty clip region */
    prgnClip = IntSysCreateRectpRgn(0, 0, 0, 0);
    if prgnClip.is_null() {
        return 0;
    }

    /* Transform given region into device coordinates */
    if REGION_LPTODP(pdc, prgnClip, prgn) == 0 {
        REGION_Delete(prgnClip);
        return 0;
    }

    /* Intersect with the system or RAO region (these are (atm) without DC-origin) */
    if !(*pdc).prgnRao.is_null() {
        IntGdiCombineRgn(prgnClip, prgnClip, (*pdc).prgnRao, RGN_AND as _);
    } else {
        IntGdiCombineRgn(prgnClip, prgnClip, (*pdc).prgnVis, RGN_AND as _);
    }

    /* Now account for the DC-origin */
    if REGION_bOffsetRgn(prgnClip, (*pdc).ptlDCOrig.x, (*pdc).ptlDCOrig.y) == 0 {
        REGION_Delete(prgnClip);
        return 0;
    }

    if (*pdc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        let mut rcrgn: RECTL = core::mem::zeroed();
        REGION_GetRgnBox(prgnClip, &mut rcrgn);
        IntUpdateBoundsRect(pdc, &mut rcrgn);
    }

    /* Prepare the DC */
    DC_vPrepareDCsForBlit(
        pdc,
        core::ptr::addr_of_mut!((*prgnClip).rdh.rcBound),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    /* Initialize a clip object */
    IntEngInitClipObj(&mut xcoClip);
    IntEngUpdateClipRegion(
        &mut xcoClip,
        (*prgnClip).rdh.nCount,
        (*prgnClip).Buffer,
        core::ptr::addr_of_mut!((*prgnClip).rdh.rcBound),
    );

    /* Call the Eng or Drv function */
    bResult = IntEngBitBlt(
        &mut (*(*pdc).dclevel.pSurface).SurfObj,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut xcoClip as *mut XCLIPOBJ as *mut CLIPOBJ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*prgnClip).rdh.rcBound),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pbo,
        pptlBrush,
        rop4,
    );

    /* Cleanup */
    DC_vFinishBlit(pdc, core::ptr::null_mut());
    REGION_Delete(prgnClip);
    IntEngFreeClipResources(&mut xcoClip);

    /* Return the result */
    bResult
}

#[no_mangle]
pub unsafe extern "C" fn IntGdiFillRgn(
    pdc: PDC,
    prgn: PREGION,
    pbrFill: PBRUSH,
) -> BOOL {
    let prgnClip: PREGION;
    let mut xcoClip: XCLIPOBJ = core::mem::zeroed();
    let mut eboFill: EBRUSHOBJ = core::mem::zeroed();
    let pbo: *mut BRUSHOBJ;
    let bRet: BOOL;
    let rop2Fg: DWORD;
    let mix: MIX;

    if (*pdc).dclevel.pSurface.is_null() {
        return 1;
    }

    prgnClip = IntSysCreateRectpRgn(0, 0, 0, 0);
    if prgnClip.is_null() {
        return 0;
    }

    if REGION_LPTODP(pdc, prgnClip, prgn) == 0 {
        REGION_Delete(prgnClip);
        return 0;
    }

    if !(*pdc).prgnRao.is_null() {
        IntGdiCombineRgn(prgnClip, prgnClip, (*pdc).prgnRao, RGN_AND as _);
    } else {
        IntGdiCombineRgn(prgnClip, prgnClip, (*pdc).prgnVis, RGN_AND as _);
    }

    if REGION_bOffsetRgn(prgnClip, (*pdc).ptlDCOrig.x, (*pdc).ptlDCOrig.y) == 0 {
        REGION_Delete(prgnClip);
        return 0;
    }

    if (*pdc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        let mut rcrgn: RECTL = core::mem::zeroed();
        REGION_GetRgnBox(prgnClip, &mut rcrgn);
        IntUpdateBoundsRect(pdc, &mut rcrgn);
    }

    IntEngInitClipObj(&mut xcoClip);
    IntEngUpdateClipRegion(
        &mut xcoClip,
        (*prgnClip).rdh.nCount,
        (*prgnClip).Buffer,
        &mut (*prgnClip).rdh.rcBound,
    );

    rop2Fg = FIXUP_ROP2((*(*pdc).pdcattr).jROP2 as _ );
    mix = rop2Fg
        | ((if (*(*pdc).pdcattr).jBkMode == OPAQUE as _ {
            rop2Fg
        } else {
            R2_NOP as _
        }) << 8);

    DC_vPrepareDCsForBlit(
        pdc,
        &mut (*prgnClip).rdh.rcBound,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !pbrFill.is_null() {
        EBRUSHOBJ_vInit(
            &mut eboFill,
            pbrFill,
            (*pdc).dclevel.pSurface as _ ,
            0x00FFFFFF,
            0,
            core::ptr::null_mut(),
        );
        pbo = core::ptr::addr_of_mut!(eboFill.BrushObject);
    } else {
        if (*(*pdc).pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
            DC_vUpdateFillBrush(pdc);
        }
        pbo = core::ptr::addr_of_mut!((*pdc).eboFill.BrushObject);
    }

    bRet = IntEngPaint(
        &mut (*(*pdc).dclevel.pSurface).SurfObj,
        &mut xcoClip as *mut XCLIPOBJ as *mut CLIPOBJ,
        pbo,
        // ptlFillOrigin, not the raw brush origin -- same fix, same commit; the
        // field already exists here and is used correctly in dcobjs/dcutil/coord.
        &mut (*pdc).ptlFillOrigin,
        mix,
    );

    DC_vFinishBlit(pdc, core::ptr::null_mut());
    REGION_Delete(prgnClip);
    IntEngFreeClipResources(&mut xcoClip);

    bRet
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiPaintRgn(pdc: PDC, prgn: PREGION) -> BOOL {
    IntGdiFillRgn(pdc, prgn, core::ptr::null_mut())
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiPaintRgn(pdc: PDC, prgn: PREGION) -> BOOL {
    IntGdiFillRgn(pdc, prgn, core::ptr::null_mut())
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiFillRgn(
    hdc: HDC,
    hrgn: HRGN,
    hbrush: HBRUSH,
) -> BOOL {
    let pdc: PDC;
    let prgn: PREGION;
    let pbrFill: PBRUSH;
    let bResult: BOOL;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    if (*pdc).dclevel.pSurface.is_null() {
        DC_UnlockDc(pdc);
        return 1;
    }

    prgn = REGION_LockRgn(hrgn);
    if prgn.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }

    pbrFill = BRUSH_ShareLockBrush((hbrush as _));
    if pbrFill.is_null() {
        REGION_UnlockRgn(prgn);
        DC_UnlockDc(pdc);
        return 0;
    }

    bResult = IntGdiFillRgn(pdc, prgn, pbrFill);

    BRUSH_ShareUnlockBrush(pbrFill);
    REGION_UnlockRgn(prgn);
    DC_UnlockDc(pdc);

    bResult
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiFillRgn(
    hdc: HDC,
    hrgn: HRGN,
    hbrush: HBRUSH,
) -> BOOL {
    let pdc: PDC;
    let prgn: PREGION;
    let pbrFill: PBRUSH;
    let bResult: BOOL;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    if (*pdc).dclevel.pSurface.is_null() {
        DC_UnlockDc(pdc);
        return 1;
    }

    prgn = REGION_LockRgn(hrgn);
    if prgn.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }

    pbrFill = BRUSH_ShareLockBrush(hbrush as _ );
    if pbrFill.is_null() {
        REGION_UnlockRgn(prgn);
        DC_UnlockDc(pdc);
        return 0;
    }

    bResult = IntGdiFillRgn(pdc, prgn, pbrFill);

    BRUSH_ShareUnlockBrush(pbrFill);
    REGION_UnlockRgn(prgn);
    DC_UnlockDc(pdc);

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiFrameRgn(
    hdc: HDC,
    hrgn: HRGN,
    hbrush: HBRUSH,
    xWidth: INT,
    yHeight: INT,
) -> BOOL {
    let hrgnFrame: HRGN;
    let bResult: BOOL;

    hrgnFrame = GreCreateFrameRgn(hrgn, xWidth, yHeight);
    if hrgnFrame.is_null() {
        return 0;
    }

    bResult = NtGdiFillRgn(hdc, hrgnFrame, hbrush);

    GreDeleteObject(hrgnFrame as _);
    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiInvertRgn(hdc: HDC, hrgn: HRGN) -> BOOL {
    let bResult: BOOL;
    let pdc: PDC;
    let prgn: PREGION;

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Check if the DC has no surface (empty mem or info DC) */
    if (*pdc).dclevel.pSurface.is_null() {
        /* Nothing to do, Windows returns TRUE! */
        DC_UnlockDc(pdc);
        return 1;
    }

    /* Lock the region */
    prgn = REGION_LockRgn(hrgn);
    if prgn.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }

    /* Call the internal function */
    bResult = IntGdiBitBltRgn(
        pdc,
        prgn,
        core::ptr::null_mut(), // pbo
        core::ptr::null_mut(), // pptlBrush,
        ROP4_DSTINVERT,
    );

    /* Unlock the region and DC and return the result */
    REGION_UnlockRgn(prgn);
    DC_UnlockDc(pdc);
    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetPixel(
    hdc: HDC,
    x: INT,
    y: INT,
    mut crColor: COLORREF,
) -> COLORREF {
    let pdc: PDC;
    let iOldColor: ULONG;
    let iSolidColor: ULONG;
    let bResult: BOOL;
    let pebo: PEBRUSHOBJ;
    let ulDirty: ULONG;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return CLR_INVALID;
    }

    /* Check if the DC has no surface (empty mem or info DC) */
    if (*pdc).dclevel.pSurface.is_null() {
        /* Fail! */
        DC_UnlockDc(pdc);
        return CLR_INVALID;
    }

    if (*pdc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR) != 0 {
        let mut rcDst: RECTL = core::mem::zeroed();

        w32r_RECTL_vSetRect(&mut rcDst, x, y, x + 1, y + 1);

        w32r_IntLPtoDP(pdc, &mut rcDst as *mut RECTL as LPPOINT, 2);

        rcDst.left += (*pdc).ptlDCOrig.x;
        rcDst.top += (*pdc).ptlDCOrig.y;
        rcDst.right += (*pdc).ptlDCOrig.x;
        rcDst.bottom += (*pdc).ptlDCOrig.y;

        IntUpdateBoundsRect(pdc, &mut rcDst);
    }

    /* Translate the color to the target format */
    iSolidColor = TranslateCOLORREF(pdc, crColor);

    /* Use the DC's text brush, which is always a solid brush */
    pebo = core::ptr::addr_of_mut!((*pdc).eboText);

    /* Save the old solid color and set the one for the pixel */
    iOldColor = EBRUSHOBJ_iSetSolidColor(pebo, iSolidColor);

    /* Save dirty flags and reset dirty text brush flag */
    ulDirty = (*(*pdc).pdcattr).ulDirty_;
    (*(*pdc).pdcattr).ulDirty_ &= !DIRTY_TEXT;

    /* Call the internal function */
    bResult = IntPatBlt(pdc, x, y, 1, 1, PATCOPY, pebo);

    /* Restore old text brush color and dirty flags */
    EBRUSHOBJ_iSetSolidColor(pebo, iOldColor);
    (*(*pdc).pdcattr).ulDirty_ = ulDirty;

    /* Initialize an XLATEOBJ from the target surface to RGB without using BkColor */
    EXLATEOBJ_vInitialize(
        &mut exlo,
        (*(*pdc).dclevel.pSurface).ppal,
        core::ptr::addr_of_mut!(gpalRGB),
        CLR_INVALID,
        CLR_INVALID,
        CLR_INVALID,
    );

    /* Translate the color back to RGB */
    crColor = XLATEOBJ_iXlate(&mut exlo.xlo, iSolidColor);

    /* Cleanup and return the target format color */
    EXLATEOBJ_vCleanup(&mut exlo);

    /* Unlock the DC */
    DC_UnlockDc(pdc);

    /* Return the new RGB color or CLR_INVALID (-1) on failure */
    if bResult != 0 {
        crColor
    } else {
        CLR_INVALID
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetPixel(hdc: HDC, x: INT, y: INT) -> COLORREF {
    let pdc: PDC;
    let mut ulRGBColor: ULONG = CLR_INVALID;
    let mut ptlSrc: POINTL = core::mem::zeroed();
    let mut rcDest: RECT = core::mem::zeroed();
    let psurfSrc: PSURFACE;
    let psurfDest: PSURFACE;

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID;
    }

    'leave: {
        /* Check if the DC has no surface (empty mem or info DC) */
        if (*pdc).dclevel.pSurface.is_null() {
            break 'leave;
        }

        /* Get the logical coordinates */
        ptlSrc.x = x;
        ptlSrc.y = y;

        /* Translate coordinates to device coordinates */
        w32r_IntLPtoDP(pdc, (core::ptr::addr_of_mut!(ptlSrc) as _), 1);
        ptlSrc.x += (*pdc).ptlDCOrig.x;
        ptlSrc.y += (*pdc).ptlDCOrig.y;

        // ptlSrc is the LPtoDP-mapped, DC-origin-translated point computed just
        // above; sampling raw x/y reads the wrong pixel on any DC whose origin is
        // not (0,0). Upstream-Commit: 7f459b49f0a
        rcDest.left = ptlSrc.x;
        rcDest.top = ptlSrc.y;
        rcDest.right = ptlSrc.x + 1;
        rcDest.bottom = ptlSrc.y + 1;

        /* Prepare DC for blit */
        DC_vPrepareDCsForBlit(pdc, (core::ptr::addr_of_mut!(rcDest) as _), core::ptr::null_mut(), core::ptr::null_mut());

        /* Check if the pixel is outside the surface */
        psurfSrc = (*pdc).dclevel.pSurface;
        if (ptlSrc.x >= (*psurfSrc).SurfObj.sizlBitmap.cx)
            || (ptlSrc.y >= (*psurfSrc).SurfObj.sizlBitmap.cy)
            || (ptlSrc.x < 0)
            || (ptlSrc.y < 0)
        {
            break 'leave;
        }

        /* Allocate a surface */
        psurfDest = SURFACE_AllocSurface(
            (STYPE_BITMAP as _),
            1,
            1,
            BMF_32BPP,
            0,
            0,
            0,
            &mut ulRGBColor as *mut ULONG as *mut _,
        );
        if !psurfDest.is_null() {
            let mut rclDest: RECTL = RECTL { left: 0, top: 0, right: 1, bottom: 1 };
            let mut exlo: EXLATEOBJ = core::mem::zeroed();

            /* Translate from the source palette to RGB color without BkColor */
            EXLATEOBJ_vInitialize(
                &mut exlo,
                (*psurfSrc).ppal,
                &raw mut gpalRGB,
                CLR_INVALID,
                CLR_INVALID,
                CLR_INVALID,
            );

            /* Call the copy bits function */
            EngCopyBits(
                &mut (*psurfDest).SurfObj,
                &mut (*psurfSrc).SurfObj,
                core::ptr::null_mut(),
                &mut exlo.xlo,
                &mut rclDest,
                &mut ptlSrc,
            );

            /* Cleanup the XLATEOBJ */
            EXLATEOBJ_vCleanup(&mut exlo);

            /* Delete the surface */
            GDIOBJ_vDeleteObject(&mut (*psurfDest).BaseObject);

            /* The top byte is zero */
            ulRGBColor &= 0x00FFFFFF;
        }
    }

    /* Unlock the DC */
    DC_vFinishBlit(pdc, core::ptr::null_mut());
    DC_UnlockDc(pdc);

    /* Return the new RGB color or -1 on failure */
    ulRGBColor
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetPixel(hdc: HDC, x: INT, y: INT) -> COLORREF {
    let pdc: PDC;
    let mut ulRGBColor: ULONG = CLR_INVALID;
    let mut ptlSrc: POINTL = core::mem::zeroed();
    let mut rcDest: RECT = core::mem::zeroed();
    let psurfSrc: PSURFACE;
    let psurfDest: PSURFACE;

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID;
    }

    'leave: {
        /* Check if the DC has no surface (empty mem or info DC) */
        if (*pdc).dclevel.pSurface.is_null() {
            break 'leave;
        }

        /* Get the logical coordinates */
        ptlSrc.x = x;
        ptlSrc.y = y;

        /* Translate coordinates to device coordinates */
        w32r_IntLPtoDP(pdc, core::ptr::addr_of_mut!(ptlSrc) as _ , 1);
        ptlSrc.x += (*pdc).ptlDCOrig.x;
        ptlSrc.y += (*pdc).ptlDCOrig.y;

        // ptlSrc is the LPtoDP-mapped, DC-origin-translated point computed just
        // above; sampling raw x/y reads the wrong pixel on any DC whose origin is
        // not (0,0). Upstream-Commit: 7f459b49f0a
        rcDest.left = ptlSrc.x;
        rcDest.top = ptlSrc.y;
        rcDest.right = ptlSrc.x + 1;
        rcDest.bottom = ptlSrc.y + 1;

        /* Prepare DC for blit */
        DC_vPrepareDCsForBlit(pdc, core::ptr::addr_of_mut!(rcDest) as _ , core::ptr::null_mut(), core::ptr::null_mut());

        /* Check if the pixel is outside the surface */
        psurfSrc = (*pdc).dclevel.pSurface;
        if (ptlSrc.x >= (*psurfSrc).SurfObj.sizlBitmap.cx)
            || (ptlSrc.y >= (*psurfSrc).SurfObj.sizlBitmap.cy)
            || (ptlSrc.x < 0)
            || (ptlSrc.y < 0)
        {
            break 'leave;
        }

        /* Allocate a surface */
        psurfDest = SURFACE_AllocSurface(
            STYPE_BITMAP as _ ,
            1,
            1,
            BMF_32BPP,
            0,
            0,
            0,
            &mut ulRGBColor as *mut ULONG as *mut _,
        );
        if !psurfDest.is_null() {
            let mut rclDest: RECTL = RECTL { left: 0, top: 0, right: 1, bottom: 1 };
            let mut exlo: EXLATEOBJ = core::mem::zeroed();

            /* Translate from the source palette to RGB color without BkColor */
            EXLATEOBJ_vInitialize(
                &mut exlo,
                (*psurfSrc).ppal,
                &raw mut gpalRGB,
                CLR_INVALID,
                CLR_INVALID,
                CLR_INVALID,
            );

            /* Call the copy bits function */
            EngCopyBits(
                &mut (*psurfDest).SurfObj,
                &mut (*psurfSrc).SurfObj,
                core::ptr::null_mut(),
                &mut exlo.xlo,
                &mut rclDest,
                &mut ptlSrc,
            );

            /* Cleanup the XLATEOBJ */
            EXLATEOBJ_vCleanup(&mut exlo);

            /* Delete the surface */
            GDIOBJ_vDeleteObject(&mut (*psurfDest).BaseObject);

            /* The top byte is zero */
            ulRGBColor &= 0x00FFFFFF;
        }
    }

    /* Unlock the DC */
    DC_vFinishBlit(pdc, core::ptr::null_mut());
    DC_UnlockDc(pdc);

    /* Return the new RGB color or -1 on failure */
    ulRGBColor
}

type INT = i32;
type ULONG = u32;
